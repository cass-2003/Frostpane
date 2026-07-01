use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IconPosition {
    pub path: String,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FenceLayout {
    pub id: String,
    pub title: String,
    pub emoji: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub collapsed: bool,
    pub icon_paths: Vec<String>,
}

fn config_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config")
}

fn layout_file() -> PathBuf {
    config_dir().join("layout.json")
}

fn fences_file() -> PathBuf {
    config_dir().join("fences.json")
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

#[tauri::command]
pub fn save_fence_layout(fences: Vec<FenceLayout>) -> Result<(), String> {
    let path = fences_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    let json =
        serde_json::to_string_pretty(&fences).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Write error: {}", e))
}

#[tauri::command]
pub fn load_fence_layout() -> Result<Vec<FenceLayout>, String> {
    let path = fences_file();
    if !path.exists() {
        return Ok(vec![]);
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Parse error: {}", e))
}
