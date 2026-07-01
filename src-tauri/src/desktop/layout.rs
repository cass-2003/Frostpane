use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

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
    #[serde(rename = "viewMode")]
    pub view_mode: Option<String>,
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

// ── Scene/Profile support ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SceneProfile {
    pub id: String,
    pub name: String,
    pub icon_positions: Vec<IconPosition>,
    pub fences: Vec<FenceLayout>,
}

fn scenes_dir() -> PathBuf {
    config_dir().join("scenes")
}

#[tauri::command]
pub fn save_scene(profile: SceneProfile) -> Result<(), String> {
    let dir = scenes_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create dir: {}", e))?;
    let path = dir.join(format!("{}.json", profile.id));
    let json =
        serde_json::to_string_pretty(&profile).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Write error: {}", e))
}

#[tauri::command]
pub fn load_scene(id: String) -> Result<SceneProfile, String> {
    let path = scenes_dir().join(format!("{}.json", id));
    if !path.exists() {
        return Err(format!("Scene '{}' not found", id));
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Parse error: {}", e))
}

#[tauri::command]
pub fn list_scenes() -> Result<Vec<SceneProfile>, String> {
    let dir = scenes_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut scenes = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| format!("Read dir error: {}", e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            if let Ok(json) = fs::read_to_string(&path) {
                if let Ok(scene) = serde_json::from_str::<SceneProfile>(&json) {
                    scenes.push(scene);
                }
            }
        }
    }
    Ok(scenes)
}

#[tauri::command]
pub fn delete_scene(id: String) -> Result<(), String> {
    let path = scenes_dir().join(format!("{}.json", id));
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Delete error: {}", e))?;
    }
    Ok(())
}

// ── Auto-backup ──

fn backup_dir() -> PathBuf {
    config_dir().join("backups")
}

#[tauri::command]
pub fn create_backup() -> Result<String, String> {
    let dir = backup_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create dir: {}", e))?;

    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let name = format!("backup_{}.json", ts);

    let positions = load_icon_positions().unwrap_or_default();
    let fences = load_fence_layout().unwrap_or_default();
    let snapshot = SceneProfile {
        id: format!("backup_{}", ts),
        name: "Auto backup".to_string(),
        icon_positions: positions,
        fences,
    };

    let path = dir.join(&name);
    let json =
        serde_json::to_string_pretty(&snapshot).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Write error: {}", e))?;

    // Keep only last 10 backups
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .map_err(|e| format!("Read dir error: {}", e))?
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by_key(|e| e.path());
    while entries.len() > 10 {
        if let Some(oldest) = entries.first() {
            let _ = fs::remove_file(oldest.path());
        }
        entries.remove(0);
    }

    Ok(name)
}

#[tauri::command]
pub fn restore_backup(name: String) -> Result<SceneProfile, String> {
    let path = backup_dir().join(&name);
    if !path.exists() {
        return Err("Backup not found".to_string());
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Parse error: {}", e))
}
