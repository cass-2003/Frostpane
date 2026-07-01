use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub icon_size: Option<i32>,
    pub animation_level: Option<String>,
    pub locale: Option<String>,
}

fn settings_file() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config").join("settings.json")
}

#[tauri::command]
pub fn save_app_settings(settings: AppSettings) -> Result<(), String> {
    let path = settings_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    let json =
        serde_json::to_string_pretty(&settings).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Write error: {}", e))
}

#[tauri::command]
pub fn load_app_settings() -> Result<AppSettings, String> {
    let path = settings_file();
    if !path.exists() {
        return Ok(AppSettings {
            icon_size: None,
            animation_level: None,
            locale: None,
        });
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Parse error: {}", e))
}
