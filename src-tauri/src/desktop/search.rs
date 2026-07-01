use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub extension: String,
}

fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join("Desktop"));
        dirs.push(home.join("Documents"));
        dirs.push(home.join("Downloads"));
    }

    if let Some(public) = dirs::public_dir() {
        dirs.push(public.join("Desktop"));
    }

    let program_files = Path::new("C:\\Program Files");
    if program_files.exists() {
        dirs.push(program_files.to_path_buf());
    }
    let program_files_x86 = Path::new("C:\\Program Files (x86)");
    if program_files_x86.exists() {
        dirs.push(program_files_x86.to_path_buf());
    }

    // Start Menu (current user + all users)
    if let Ok(appdata) = std::env::var("APPDATA") {
        let start = PathBuf::from(&appdata)
            .join("Microsoft\\Windows\\Start Menu\\Programs");
        if start.exists() {
            dirs.push(start);
        }
    }
    let all_users_start =
        Path::new("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs");
    if all_users_start.exists() {
        dirs.push(all_users_start.to_path_buf());
    }

    dirs.retain(|d| d.exists());
    dirs
}

fn should_skip(name: &str) -> bool {
    matches!(
        name,
        "node_modules"
            | ".git"
            | "__pycache__"
            | ".svn"
            | "Temp"
            | "Cache"
            | "CacheStorage"
            | "Code Cache"
            | "$RECYCLE.BIN"
    )
}

#[tauri::command]
pub async fn search_files(query: String, max_results: i32) -> Result<Vec<SearchResult>, String> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let max = max_results.max(1) as usize;

    let results = tokio::task::spawn_blocking(move || {
        let mut out = Vec::new();
        let dirs = search_dirs();

        for dir in &dirs {
            if out.len() >= max {
                break;
            }

            let walker = WalkDir::new(dir)
                .max_depth(6)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    !name.starts_with('.') && !should_skip(&name)
                });

            for entry in walker.flatten() {
                if out.len() >= max {
                    break;
                }

                let name = entry.file_name().to_string_lossy();
                if !name.to_lowercase().contains(&query) {
                    continue;
                }

                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let ext = entry
                    .path()
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();

                out.push(SearchResult {
                    name: name.to_string(),
                    path: entry.path().to_string_lossy().to_string(),
                    is_dir: meta.is_dir(),
                    size: meta.len(),
                    extension: ext,
                });
            }
        }

        out
    })
    .await
    .map_err(|e| format!("Search task failed: {}", e))?;

    Ok(results)
}
