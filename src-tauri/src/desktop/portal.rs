use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::icons::extract_icon_png;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortalItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub extension: String,
    #[serde(rename = "iconData")]
    pub icon_data: String,
    pub size: u64,
}

#[tauri::command]
pub fn list_portal_contents(folder_path: String) -> Result<Vec<PortalItem>, String> {
    let dir = Path::new(&folder_path);
    if !dir.exists() {
        return Err(format!("Folder does not exist: {}", folder_path));
    }
    if !dir.is_dir() {
        return Err(format!("Path is not a directory: {}", folder_path));
    }

    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut items: Vec<PortalItem> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        if name.starts_with('.') || name == "desktop.ini" {
            continue;
        }

        let is_dir = path.is_dir();
        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let size = if is_dir {
            0
        } else {
            fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
        };

        let (icon_data, _icon_size) = extract_icon_png(&path);

        items.push(PortalItem {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            extension,
            icon_data,
            size,
        });
    }

    items.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then_with(|| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
        })
    });

    Ok(items)
}
