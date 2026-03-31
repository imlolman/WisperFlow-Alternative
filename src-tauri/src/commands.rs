use crate::audio::AudioRecorder;
use crate::config::{self, HistoryEntry};
use crate::shortcuts;
use crate::transcriber;
use crate::{overlay, tray, AppState};
use tauri::{AppHandle, Emitter, Manager};

pub fn do_open_settings(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("settings") {
        win.set_focus().ok();
        return;
    }
    let _ = tauri::WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("settings.html".into()),
    )
    .title("WhisperFlow Alternative Settings")
    .inner_size(380.0, 580.0)
    .resizable(false)
    .center()
    .build();
}

pub fn do_open_onboarding(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("onboarding") {
        win.set_focus().ok();
        return;
    }
    let _ = tauri::WebviewWindowBuilder::new(
        app,
        "onboarding",
        tauri::WebviewUrl::App("onboarding.html".into()),
    )
    .title("WhisperFlow Alternative Setup")
    .inner_size(420.0, 520.0)
    .resizable(false)
    .center()
    .build();
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let cfg = state.config.read().clone();
    let mut val = serde_json::to_value(&cfg).map_err(|e| e.to_string())?;
    if let Some(obj) = val.as_object_mut() {
        obj.insert(
            "_hold_display".into(),
            serde_json::Value::String(shortcuts::shortcut_display(&cfg.shortcut_hold)),
        );
        obj.insert(
            "_toggle_display".into(),
            serde_json::Value::String(shortcuts::shortcut_display(&cfg.shortcut_toggle)),
        );
        obj.insert(
            "_paste_display".into(),
            serde_json::Value::String(shortcuts::shortcut_display(&cfg.shortcut_paste_last)),
        );
    }
    Ok(val)
}

#[tauri::command]
pub fn save_shortcut(
    state: tauri::State<'_, AppState>,
    field: String,
    value: String,
) -> Result<String, String> {
    {
        let mut cfg = state.config.write();
        match field.as_str() {
            "shortcut_hold" => cfg.shortcut_hold = value.clone(),
            "shortcut_toggle" => cfg.shortcut_toggle = value.clone(),
            "shortcut_paste_last" => cfg.shortcut_paste_last = value.clone(),
            _ => return Err("Unknown field".into()),
        }
        config::save_config(&cfg).map_err(|e| e.to_string())?;
    }
    update_shortcut_config(&state);
    Ok(shortcuts::shortcut_display(&value))
}

#[tauri::command]
pub fn get_microphones() -> Result<Vec<crate::audio::DeviceInfo>, String> {
    Ok(AudioRecorder::list_devices())
}

#[tauri::command]
pub fn save_mic(state: tauri::State<'_, AppState>, device: Option<String>) -> Result<(), String> {
    let mut cfg = state.config.write();
    cfg.mic_device = device;
    config::save_config(&cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_field(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    field: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let mut cfg = state.config.write();
    match field.as_str() {
        "hide_dock_icon" => {
            let hide = value.as_bool().unwrap_or(false);
            cfg.hide_dock_icon = hide;
            #[cfg(target_os = "macos")]
            set_dock_icon_visible(&app, !hide);
        }
        "hide_menu_icon" => {
            let hide = value.as_bool().unwrap_or(false);
            cfg.hide_menu_icon = hide;
            if hide {
                tray::set_visible(&app, false);
            } else {
                tray::set_visible(&app, true);
            }
        }
        "start_on_login" => {
            let enabled = value.as_bool().unwrap_or(false);
            cfg.start_on_login = enabled;
            #[cfg(target_os = "macos")]
            {
                if enabled {
                    install_launch_agent().ok();
                } else {
                    remove_launch_agent().ok();
                }
            }
        }
        _ => return Err("Unknown field".into()),
    }
    config::save_config(&cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_history() -> Result<Vec<HistoryEntry>, String> {
    Ok(config::load_history())
}

#[tauri::command]
pub fn copy_text(text: String) -> Result<(), String> {
    std::process::Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(text.as_bytes())?;
            }
            child.wait()?;
            Ok(())
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn disable_shortcuts(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.shortcuts_enabled.store(false, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn enable_shortcuts(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.shortcuts_enabled.store(true, std::sync::atomic::Ordering::SeqCst);
    update_shortcut_config(&state);
    Ok(())
}

#[tauri::command]
pub async fn capture_mouse(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.capture_tx.lock().replace(tx);
    state.capture_mode.store(true, std::sync::atomic::Ordering::SeqCst);

    match rx.await {
        Ok(name) => {
            state.capture_mode.store(false, std::sync::atomic::Ordering::SeqCst);
            Ok(Some(name))
        }
        Err(_) => {
            state.capture_mode.store(false, std::sync::atomic::Ordering::SeqCst);
            Ok(None)
        }
    }
}

#[tauri::command]
pub fn cancel_capture(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.capture_mode.store(false, std::sync::atomic::Ordering::SeqCst);
    state.capture_tx.lock().take();
    Ok(())
}

#[tauri::command]
pub fn cancel_recording(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if state.is_recording.load(std::sync::atomic::Ordering::SeqCst) {
        state.is_recording.store(false, std::sync::atomic::Ordering::SeqCst);
        state.recorder.lock().stop();
        overlay::hide(&app);
    }
    Ok(())
}

#[tauri::command]
pub fn shortcut_display_name(value: String) -> Result<String, String> {
    Ok(shortcuts::shortcut_display(&value))
}

#[tauri::command]
pub fn test_mic(device: Option<String>) -> Result<f64, String> {
    AudioRecorder::test_mic(device.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn request_mic_permission() -> Result<(), String> {
    // Opening a short audio stream triggers macOS TCC prompt
    std::thread::spawn(|| {
        let _ = AudioRecorder::test_mic(None);
    });
    Ok(())
}

#[tauri::command]
pub async fn download_model(app: AppHandle) -> Result<(), String> {
    if transcriber::model_exists() {
        return Ok(());
    }
    let app_clone = app.clone();
    transcriber::download_model(move |downloaded, total| {
        app_clone
            .emit("model-download-progress", (downloaded, total))
            .ok();
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_model_exists() -> Result<bool, String> {
    Ok(transcriber::model_exists())
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    do_open_settings(&app);
    Ok(())
}

#[tauri::command]
pub fn finish_onboarding(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    mic: Option<String>,
    hold: String,
    toggle: String,
) -> Result<(), String> {
    {
        let mut cfg = state.config.write();
        cfg.mic_device = mic;
        cfg.shortcut_hold = hold;
        cfg.shortcut_toggle = toggle;
        cfg.setup_complete = true;
        config::save_config(&cfg).map_err(|e| e.to_string())?;
    }
    update_shortcut_config(&state);

    if let Some(win) = app.get_webview_window("onboarding") {
        win.close().ok();
    }

    // Load model in background if exists
    let state_clone = state.inner().clone();
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        crate::load_model_bg(state_clone, app_clone).await;
    });

    Ok(())
}

fn update_shortcut_config(state: &AppState) {
    let cfg = state.config.read();
    let mut sc = state.shortcut_config.write();
    sc.hold = shortcuts::parse_shortcut(&cfg.shortcut_hold);
    sc.toggle = shortcuts::parse_shortcut(&cfg.shortcut_toggle);
    sc.paste = shortcuts::parse_shortcut(&cfg.shortcut_paste_last);
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn set_dock_icon_visible(_app: &AppHandle, visible: bool) {
    use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
    unsafe {
        let app = NSApp();
        let policy = if visible {
            NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular
        } else {
            NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory
        };
        app.setActivationPolicy_(policy);
    }
}

#[cfg(target_os = "macos")]
fn install_launch_agent() -> anyhow::Result<()> {
    let plist_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("no home"))?
        .join("Library/LaunchAgents");
    std::fs::create_dir_all(&plist_dir)?;
    let exe = std::env::current_exe()?;
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.whisperflow-alternative.app</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
        exe.display()
    );
    std::fs::write(plist_dir.join("com.whisperflow-alternative.app.plist"), plist)?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn remove_launch_agent() -> anyhow::Result<()> {
    let path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("no home"))?
        .join("Library/LaunchAgents/com.whisperflow-alternative.app.plist");
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
