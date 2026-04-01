use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const SAMPLE_RATE: u32 = 16_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_hold")]
    pub shortcut_hold: String,
    #[serde(default = "default_toggle")]
    pub shortcut_toggle: String,
    #[serde(default)]
    pub shortcut_paste_last: String,
    #[serde(default, alias = "hide_tray")]
    pub hide_menu_icon: bool,
    #[serde(default)]
    pub hide_dock_icon: bool,
    #[serde(default)]
    pub start_on_login: bool,
    #[serde(default)]
    pub mic_device: Option<String>,
    #[serde(default)]
    pub setup_complete: bool,
}

fn default_hold() -> String {
    "mouse:middle".into()
}
fn default_toggle() -> String {
    "key:Alt_R".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shortcut_hold: default_hold(),
            shortcut_toggle: default_toggle(),
            shortcut_paste_last: String::new(),
            hide_menu_icon: false,
            hide_dock_icon: false,
            start_on_login: false,
            mic_device: None,
            setup_complete: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub text: String,
    pub duration_s: f64,
}

pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("no home dir")
        .join(".openbolo_config.json")
}

pub fn history_path() -> PathBuf {
    dirs::home_dir()
        .expect("no home dir")
        .join(".openbolo_history.json")
}

pub fn model_dir() -> PathBuf {
    dirs::data_dir()
        .expect("no data dir")
        .join("com.openbolo.app")
        .join("models")
}

pub fn model_path() -> PathBuf {
    model_dir().join("ggml-base.en.bin")
}

pub fn load_config() -> Config {
    let path = config_path();
    if !path.exists() {
        return Config::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

pub fn save_config(cfg: &Config) -> anyhow::Result<()> {
    let data = serde_json::to_string_pretty(cfg)?;
    std::fs::write(config_path(), data)?;
    Ok(())
}

pub fn load_history() -> Vec<HistoryEntry> {
    let path = history_path();
    if !path.exists() {
        return vec![];
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|d| serde_json::from_str(&d).ok())
        .unwrap_or_default()
}

pub fn append_history(text: &str, duration_s: f64) -> anyhow::Result<()> {
    let mut hist = load_history();
    hist.push(HistoryEntry {
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        text: text.to_string(),
        duration_s: (duration_s * 10.0).round() / 10.0,
    });
    let data = serde_json::to_string_pretty(&hist)?;
    std::fs::write(history_path(), data)?;
    Ok(())
}
