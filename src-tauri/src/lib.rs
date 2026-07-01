mod desktop;

use desktop::archive;
use desktop::autostart;
use desktop::context_menu;
use desktop::health;
use desktop::icons::{self, DesktopIcon};
use desktop::layout;
use desktop::overlay;
use desktop::portal;
use desktop::rules;
use desktop::search;
use desktop::settings;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
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
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Auto-backup on startup
            if let Ok(name) = layout::create_backup() {
                log::info!("Startup backup created: {}", name);
            }

            let _window = app
                .get_webview_window("main")
                .expect("main window not found");

            // --- System tray ---
            let icons_hidden = Arc::new(AtomicBool::new(false));

            let toggle_window = MenuItemBuilder::new("Show/Hide Frostpane")
                .id("toggle")
                .build(app)?;
            let toggle_icons = MenuItemBuilder::new("Hide Desktop Icons")
                .id("hide_icons")
                .build(app)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let settings = MenuItemBuilder::new("Settings...")
                .id("settings")
                .build(app)?;
            let about = MenuItemBuilder::new("About Frostpane")
                .id("about")
                .build(app)?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let exit_item = MenuItemBuilder::new("Exit")
                .id("exit")
                .build(app)?;

            // Clone handle before menu consumes ownership via reference
            let toggle_icons_handle = toggle_icons.clone();

            let menu = MenuBuilder::new(app)
                .item(&toggle_window)
                .item(&toggle_icons)
                .item(&sep1)
                .item(&settings)
                .item(&about)
                .item(&sep2)
                .item(&exit_item)
                .build()?;

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Frostpane")
                .show_menu_on_left_click(false)
                .on_menu_event({
                    let icons_hidden = icons_hidden.clone();
                    move |app, event| match event.id().as_ref() {
                        "toggle" => {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        "hide_icons" => {
                            if icons_hidden.load(Ordering::SeqCst) {
                                overlay::show_desktop_icons();
                                icons_hidden.store(false, Ordering::SeqCst);
                                let _ = toggle_icons_handle.set_text("Hide Desktop Icons");
                            } else {
                                overlay::hide_desktop_icons();
                                icons_hidden.store(true, Ordering::SeqCst);
                                let _ = toggle_icons_handle.set_text("Show Desktop Icons");
                            }
                        }
                        "settings" => {
                            log::info!("Settings: placeholder");
                        }
                        "about" => {
                            log::info!("About: placeholder");
                        }
                        "exit" => {
                            overlay::show_desktop_icons();
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Keep tray icon alive for the app's lifetime
            app.manage(tray);

            Ok(())
        })
        .on_window_event(|window, event| match event {
            // Hide to tray instead of quitting on window close
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            // Restore desktop icons if the window is destroyed (e.g., app.exit)
            tauri::WindowEvent::Destroyed => {
                overlay::show_desktop_icons();
            }
            _ => {}
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
            layout::save_fence_layout,
            layout::load_fence_layout,
            layout::save_scene,
            layout::load_scene,
            layout::list_scenes,
            layout::delete_scene,
            layout::create_backup,
            layout::restore_backup,
            layout::save_pages,
            layout::load_pages,
            autostart::is_autostart_enabled,
            autostart::set_autostart,
            settings::save_app_settings,
            settings::load_app_settings,
            search::search_files,
            archive::preview_archive,
            archive::execute_archive,
            archive::undo_archive,
            archive::list_archive_history,
            rules::save_rules,
            rules::load_rules,
            rules::match_rules,
            health::check_broken_shortcuts,
            health::delete_broken_shortcuts,
            portal::list_portal_contents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
