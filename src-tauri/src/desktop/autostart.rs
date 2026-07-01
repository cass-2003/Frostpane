use std::path::PathBuf;
use std::process::Command;

fn startup_shortcut_path() -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA env var not set".to_string())?;
    let mut path = PathBuf::from(appdata);
    path.push("Microsoft\\Windows\\Start Menu\\Programs\\Startup");
    path.push("Frostpane.lnk");
    Ok(path)
}

#[tauri::command]
pub fn is_autostart_enabled() -> bool {
    startup_shortcut_path().map(|p| p.exists()).unwrap_or(false)
}

#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let shortcut_path = startup_shortcut_path()?;

    if enabled {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?;
        let exe_str = exe_path.to_string_lossy();
        let lnk_str = shortcut_path.to_string_lossy();

        let script = format!(
            "$ws = New-Object -ComObject WScript.Shell; \
             $s = $ws.CreateShortcut('{lnk}'); \
             $s.TargetPath = '{exe}'; \
             $s.Save()",
            lnk = lnk_str,
            exe = exe_str,
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()
            .map_err(|e| format!("Failed to spawn PowerShell: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "PowerShell shortcut creation failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    } else if shortcut_path.exists() {
        std::fs::remove_file(&shortcut_path)
            .map_err(|e| format!("Failed to remove startup shortcut: {}", e))?;
    }

    Ok(())
}
