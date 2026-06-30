use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IconPosition {
    pub path: String,
    pub x: i32,
    pub y: i32,
}

fn layout_file() -> PathBuf {
    // Store alongside the executable (green/portable mode)
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config").join("layout.json")
}

#[tauri::command]
pub fn save_icon_positions(positions: Vec<IconPosition>) -> Result<(), String> {
    let path = layout_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(&positions)
        .map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Write error: {}", e))
}

#[tauri::command]
pub fn load_icon_positions() -> Result<Vec<IconPosition>, String> {
    let path = layout_file();
    if !path.exists() {
        return Ok(vec![]);
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Parse error: {}", e))
}
