mod desktop;

use desktop::context_menu;
use desktop::icons::{self, DesktopIcon};
use desktop::layout;
use desktop::overlay;
use tauri::Manager;

#[tauri::command]
fn get_desktop_icons() -> Vec<DesktopIcon> {
    icons::enumerate_desktop_items()
}

#[tauri::command]
fn open_item(path: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows::core::w;
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let result = ShellExecuteW(
                None,
                w!("open"),
                PCWSTR(wide_path.as_ptr()),
                None,
                None,
                SW_SHOWNORMAL,
            );
            let code = result.0 as isize;
            if code <= 32 {
                return Err(format!("ShellExecute failed with code {}", code));
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn open_item_admin(path: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows::core::w;
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let result = ShellExecuteW(
                None,
                w!("runas"),
                PCWSTR(wide_path.as_ptr()),
                None,
                None,
                SW_SHOWNORMAL,
            );
            let code = result.0 as isize;
            if code <= 32 {
                return Err(format!("ShellExecute (runas) failed with code {}", code));
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn open_file_location(path: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("explorer.exe")
            .arg(format!("/select,{}", path))
            .spawn()
            .map_err(|e| format!("Failed to open location: {}", e))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // M0: run as maximized window (desktop-level embedding deferred)
            // overlay::embed_in_desktop() has WebView2 transparency issues as Progman child
            // TODO: investigate WS_EX_TOOLWINDOW + always-on-bottom for desktop-like behavior
            let _window = app
                .get_webview_window("main")
                .expect("main window not found");

            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                overlay::show_desktop_icons();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_desktop_icons,
            open_item,
            open_item_admin,
            open_file_location,
            context_menu::get_quick_menu_items,
            context_menu::copy_path_to_clipboard,
            context_menu::delete_to_recycle_bin,
            context_menu::show_native_context_menu,
            layout::save_icon_positions,
            layout::load_icon_positions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
