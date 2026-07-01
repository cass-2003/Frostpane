use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMove {
    pub original_path: String,
    pub archived_path: String,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchiveOperation {
    pub id: String,
    pub timestamp: u64,
    pub moves: Vec<FileMove>,
    #[serde(default)]
    pub undone: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchivePreview {
    pub moves: Vec<FileMove>,
    pub target_dir: String,
    pub file_count: usize,
}

fn config_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config")
}

fn archive_log_file() -> PathBuf {
    config_dir().join("archive_log.json")
}

fn desktop_dir() -> Result<PathBuf, String> {
    dirs::desktop_dir().ok_or_else(|| "Cannot determine desktop directory".to_string())
}

fn load_archive_log() -> Vec<ArchiveOperation> {
    let path = archive_log_file();
    if !path.exists() {
        return vec![];
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn save_archive_log(ops: &[ArchiveOperation]) -> Result<(), String> {
    let path = archive_log_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let json =
        serde_json::to_string_pretty(ops).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Write error: {}", e))
}

fn is_shortcut(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
}

fn move_file(src: &Path, dst: &Path) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
    }
    // Try rename first (fast, same-drive)
    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(_) => {
            // Fallback: copy + delete (cross-drive)
            fs::copy(src, dst)
                .map_err(|e| format!("Copy failed {} -> {}: {}", src.display(), dst.display(), e))?;
            fs::remove_file(src)
                .map_err(|e| format!("Remove original failed {}: {}", src.display(), e))
        }
    }
}

#[tauri::command]
pub fn preview_archive(
    fence_title: String,
    icon_paths: Vec<String>,
) -> Result<ArchivePreview, String> {
    let desktop = desktop_dir()?;
    let target_dir = desktop.join(&fence_title);

    let moves: Vec<FileMove> = icon_paths
        .iter()
        .filter(|p| !is_shortcut(p))
        .filter(|p| Path::new(p.as_str()).exists())
        .map(|p| {
            let src = Path::new(p.as_str());
            let file_name = src
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let archived = target_dir.join(&file_name);
            FileMove {
                original_path: p.clone(),
                archived_path: archived.to_string_lossy().to_string(),
                file_name,
            }
        })
        .collect();

    let file_count = moves.len();
    Ok(ArchivePreview {
        moves,
        target_dir: target_dir.to_string_lossy().to_string(),
        file_count,
    })
}

#[tauri::command]
pub fn execute_archive(
    fence_title: String,
    icon_paths: Vec<String>,
) -> Result<ArchiveOperation, String> {
    let preview = preview_archive(fence_title, icon_paths)?;

    if preview.moves.is_empty() {
        return Err("No files to archive".to_string());
    }

    let mut completed_moves: Vec<FileMove> = Vec::new();

    for m in &preview.moves {
        let src = Path::new(&m.original_path);
        let dst = Path::new(&m.archived_path);

        // Handle name collision: append (1), (2), etc.
        let final_dst = if dst.exists() {
            let stem = dst
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let ext = dst
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            let parent = dst.parent().unwrap();
            let mut n = 1u32;
            loop {
                let candidate = parent.join(format!("{} ({}){}", stem, n, ext));
                if !candidate.exists() {
                    break candidate;
                }
                n += 1;
            }
        } else {
            dst.to_path_buf()
        };

        if let Err(e) = move_file(src, &final_dst) {
            // Rollback completed moves on failure
            for done in completed_moves.iter().rev() {
                let _ = move_file(
                    Path::new(&done.archived_path),
                    Path::new(&done.original_path),
                );
            }
            return Err(format!("Archive failed for {}: {}", m.file_name, e));
        }

        completed_moves.push(FileMove {
            original_path: m.original_path.clone(),
            archived_path: final_dst.to_string_lossy().to_string(),
            file_name: m.file_name.clone(),
        });
    }

    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let op = ArchiveOperation {
        id: format!("archive_{}_{:x}", ts, rand_u32()),
        timestamp: ts,
        moves: completed_moves,
        undone: false,
    };

    let mut log = load_archive_log();
    log.push(op.clone());
    save_archive_log(&log)?;

    Ok(op)
}

#[tauri::command]
pub fn undo_archive(operation_id: String) -> Result<(), String> {
    let mut log = load_archive_log();
    let op = log
        .iter_mut()
        .find(|o| o.id == operation_id)
        .ok_or_else(|| format!("Operation '{}' not found", operation_id))?;

    if op.undone {
        return Err("Operation already undone".to_string());
    }

    for m in op.moves.iter().rev() {
        let src = Path::new(&m.archived_path);
        let dst = Path::new(&m.original_path);
        if src.exists() {
            move_file(src, dst)?;
        }
    }

    // Clean up empty archive directory
    if let Some(first) = op.moves.first() {
        if let Some(parent) = Path::new(&first.archived_path).parent() {
            if parent.exists() && fs::read_dir(parent).map(|d| d.count() == 0).unwrap_or(false) {
                let _ = fs::remove_dir(parent);
            }
        }
    }

    op.undone = true;
    save_archive_log(&log)
}

#[tauri::command]
pub fn list_archive_history() -> Result<Vec<ArchiveOperation>, String> {
    Ok(load_archive_log())
}

fn rand_u32() -> u32 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
}
