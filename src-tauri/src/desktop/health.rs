use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrokenShortcut {
    pub path: String,
    pub name: String,
    pub target: String,
}

fn desktop_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        dirs.push(PathBuf::from(user_profile).join("Desktop"));
    }
    if let Some(public) = std::env::var_os("PUBLIC") {
        dirs.push(PathBuf::from(public).join("Desktop"));
    }
    dirs
}

fn resolve_lnk_target(lnk_path: &str) -> Option<String> {
    let escaped = lnk_path.replace('\'', "''");
    let script = format!(
        "(New-Object -ComObject WScript.Shell).CreateShortcut('{}').TargetPath",
        escaped
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let target = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if target.is_empty() { None } else { Some(target) }
}

#[tauri::command]
pub fn check_broken_shortcuts() -> Result<Vec<BrokenShortcut>, String> {
    let mut broken = Vec::new();

    for dir in desktop_dirs() {
        if !dir.exists() {
            continue;
        }
        let entries = std::fs::read_dir(&dir)
            .map_err(|e| format!("Failed to read desktop dir: {e}"))?;

        for entry in entries.flatten() {
            let path = entry.path();
            let is_lnk = path
                .extension()
                .map(|e| e.eq_ignore_ascii_case("lnk"))
                .unwrap_or(false);

            if !is_lnk {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            let name = path
                .file_stem()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if let Some(target) = resolve_lnk_target(&path_str) {
                if !Path::new(&target).exists() {
                    broken.push(BrokenShortcut { path: path_str, name, target });
                }
            }
        }
    }

    Ok(broken)
}

#[tauri::command]
pub fn delete_broken_shortcuts(paths: Vec<String>) -> Result<usize, String> {
    if paths.is_empty() {
        return Ok(0);
    }

    let count = paths.len();

    #[cfg(windows)]
    {
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::*;

        // SHFileOperationW requires a double-null-terminated string;
        // multiple paths are separated by single nulls.
        let mut wide: Vec<u16> = Vec::new();
        for p in &paths {
            wide.extend(p.encode_utf16());
            wide.push(0);
        }
        wide.push(0);

        let mut op = SHFILEOPSTRUCTW {
            wFunc: FO_DELETE,
            pFrom: PCWSTR(wide.as_ptr()),
            fFlags: (FOF_ALLOWUNDO.0 | FOF_NOCONFIRMATION.0 | FOF_SILENT.0) as u16,
            ..Default::default()
        };

        unsafe {
            let ret = SHFileOperationW(&mut op);
            if ret != 0 {
                return Err(format!("SHFileOperationW failed: {ret}"));
            }
        }
    }

    Ok(count)
}
