mod audio;
mod commands;
mod config;
mod overlay;
mod shortcuts;
mod text_inject;
mod transcriber;
mod tray;

use parking_lot::{Mutex, RwLock};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{Emitter, Manager};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub transcriber: Arc<RwLock<Option<transcriber::Transcriber>>>,
    pub model_loaded: Arc<AtomicBool>,
    pub recorder: Arc<Mutex<audio::AudioRecorder>>,
    pub is_recording: Arc<AtomicBool>,
    pub is_processing: Arc<AtomicBool>,
    pub recording_mode: Arc<Mutex<String>>,
    pub shortcuts_enabled: Arc<AtomicBool>,
    pub shortcut_config: Arc<RwLock<shortcuts::ShortcutConfig>>,
    pub capture_mode: Arc<AtomicBool>,
    pub capture_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<String>>>>,
}

impl AppState {
    fn new() -> Self {
        let cfg = config::load_config();
        let sc = shortcuts::ShortcutConfig {
            hold: shortcuts::parse_shortcut(&cfg.shortcut_hold),
            toggle: shortcuts::parse_shortcut(&cfg.shortcut_toggle),
            paste: shortcuts::parse_shortcut(&cfg.shortcut_paste_last),
        };
        Self {
            config: Arc::new(RwLock::new(cfg)),
            transcriber: Arc::new(RwLock::new(None)),
            model_loaded: Arc::new(AtomicBool::new(false)),
            recorder: Arc::new(Mutex::new(audio::AudioRecorder::new())),
            is_recording: Arc::new(AtomicBool::new(false)),
            is_processing: Arc::new(AtomicBool::new(false)),
            recording_mode: Arc::new(Mutex::new(String::new())),
            shortcuts_enabled: Arc::new(AtomicBool::new(true)),
            shortcut_config: Arc::new(RwLock::new(sc)),
            capture_mode: Arc::new(AtomicBool::new(false)),
            capture_tx: Arc::new(Mutex::new(None)),
        }
    }
}

pub async fn load_model_bg(state: AppState, _app: tauri::AppHandle) {
    let path = config::model_path();
    if !path.exists() {
        log::info!("Model not found at {:?}, skipping load", path);
        return;
    }
    log::info!("Loading whisper model...");
    let path_str = path.to_string_lossy().to_string();
    let result =
        tauri::async_runtime::spawn_blocking(move || transcriber::Transcriber::load(&path_str))
            .await;

    match result {
        Ok(Ok(t)) => {
            *state.transcriber.write() = Some(t);
            state.model_loaded.store(true, Ordering::SeqCst);
        }
        Ok(Err(e)) => {
            eprintln!("[wisperflow] Failed to load model: {}", e);
        }
        Err(e) => {
            eprintln!("[wisperflow] Model load task panicked: {}", e);
        }
    }
}

fn start_shortcut_listener(state: &AppState, app: tauri::AppHandle) {
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();

    let grab_handle = shortcuts::GrabHandle {
        enabled: Arc::clone(&state.shortcuts_enabled),
        config: Arc::clone(&state.shortcut_config),
        capture_mode: Arc::clone(&state.capture_mode),
        capture_tx: Arc::clone(&state.capture_tx),
    };

    shortcuts::start_grab(grab_handle, event_tx);

    let st = state.clone();
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            handle_shortcut_event(&st, &app2, event);
        }
    });
}

fn handle_shortcut_event(
    state: &AppState,
    app: &tauri::AppHandle,
    event: shortcuts::ShortcutEvent,
) {
    match event {
        shortcuts::ShortcutEvent::HoldPress => {
            let model = state.model_loaded.load(Ordering::SeqCst);
            let processing = state.is_processing.load(Ordering::SeqCst);
            let recording = state.is_recording.load(Ordering::SeqCst);
            if !model || processing || recording {
                return;
            }
            start_recording(state, app, "hold");
        }
        shortcuts::ShortcutEvent::HoldRelease => {
            if state.is_recording.load(Ordering::SeqCst)
                && *state.recording_mode.lock() == "hold"
            {
                stop_recording(state, app);
            }
        }
        shortcuts::ShortcutEvent::TogglePress => {
            let recording = state.is_recording.load(Ordering::SeqCst);
            let processing = state.is_processing.load(Ordering::SeqCst);
            let model = state.model_loaded.load(Ordering::SeqCst);
            if recording && *state.recording_mode.lock() == "toggle" {
                stop_recording(state, app);
            } else if !processing && !recording && model {
                start_recording(state, app, "toggle");
            }
        }
        shortcuts::ShortcutEvent::PastePress => {
            std::thread::spawn(move || {
                let history = config::load_history();
                if let Some(last) = history.last() {
                    if !last.text.is_empty() {
                        text_inject::type_text(&last.text);
                    }
                }
            });
        }
    }
}

fn start_recording(state: &AppState, app: &tauri::AppHandle, mode: &str) {
    state.is_recording.store(true, Ordering::SeqCst);
    *state.recording_mode.lock() = mode.to_string();

    let mic = state.config.read().mic_device.clone();
    if let Err(e) = state.recorder.lock().start(mic.as_deref()) {
        eprintln!("[wisperflow] Failed to start recording: {}", e);
        state.is_recording.store(false, Ordering::SeqCst);
        return;
    }

    overlay::show(app, mode);

    // Amplitude timer
    let st = state.clone();
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(70));
        loop {
            interval.tick().await;
            if !st.is_recording.load(Ordering::Relaxed) {
                break;
            }
            let rms = st.recorder.lock().get_rms();
            overlay::emit_amplitude(&app_clone, rms);
        }
    });
}

fn stop_recording(state: &AppState, app: &tauri::AppHandle) {
    state.is_recording.store(false, Ordering::SeqCst);
    let audio = state.recorder.lock().stop();

    let duration = audio.len() as f64 / config::SAMPLE_RATE as f64;
    if duration < 0.3 {
        overlay::hide(app);
        return;
    }

    state.is_processing.store(true, Ordering::SeqCst);
    overlay::show_processing(app);

    let st = state.clone();
    let app_clone = app.clone();
    std::thread::spawn(move || {
        // Reload context every transcription to prevent state accumulation
        {
            let path = config::model_path().to_string_lossy().to_string();
            if let Some(t) = st.transcriber.write().as_mut() {
                t.reload(&path).ok();
            }
        }

        let text = {
            let guard = st.transcriber.read();
            guard.as_ref().and_then(|t| t.transcribe(&audio))
        };

        match &text {
            Some(txt) => {
                config::append_history(txt, duration).ok();
                app_clone.emit("history-updated", ()).ok();
                text_inject::type_text(txt);
            }
            None => {}
        }

        st.is_processing.store(false, Ordering::SeqCst);
        overlay::hide(&app_clone);
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            commands::do_open_settings(app);
        }))
        .setup(|app| {
            let state = AppState::new();
            app.manage(state.clone());

            tray::setup(app)?;

            // Apply dock icon setting
            #[cfg(target_os = "macos")]
            if state.config.read().hide_dock_icon {
                commands::set_dock_icon_visible(app.handle(), false);
            }

            // Auto-hide menu bar icon after 10 seconds
            if state.config.read().hide_menu_icon {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    tray::set_visible(&handle, false);
                });
            }

            // Check onboarding
            let needs_onboarding = !state.config.read().setup_complete;
            if needs_onboarding {
                commands::do_open_onboarding(app.handle());
            } else {
                // Load model in background
                let st2 = state.clone();
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    load_model_bg(st2, handle).await;
                });
            }

            // Start shortcut listener
            start_shortcut_listener(&state, app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_shortcut,
            commands::get_microphones,
            commands::save_mic,
            commands::save_field,
            commands::get_history,
            commands::copy_text,
            commands::disable_shortcuts,
            commands::enable_shortcuts,
            commands::capture_mouse,
            commands::cancel_capture,
            commands::cancel_recording,
            commands::shortcut_display_name,
            commands::test_mic,
            commands::request_mic_permission,
            commands::download_model,
            commands::check_model_exists,
            commands::open_settings,
            commands::finish_onboarding,
        ])
        .run(tauri::generate_context!())
        .expect("error while running WhisperFlow Alternative");
}
