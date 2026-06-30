use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct QuickMenuItem {
    pub id: String,
    pub label: String,
    pub separator: bool,
    pub shortcut: String,
}

#[tauri::command]
pub fn get_quick_menu_items() -> Vec<QuickMenuItem> {
    vec![
        QuickMenuItem { id: "open".into(), label: "Open".into(), separator: false, shortcut: "Enter".into() },
        QuickMenuItem { id: "run_as_admin".into(), label: "Run as administrator".into(), separator: false, shortcut: String::new() },
        QuickMenuItem { id: "open_location".into(), label: "Open file location".into(), separator: false, shortcut: String::new() },
        QuickMenuItem { id: "rename".into(), label: "Rename".into(), separator: false, shortcut: "F2".into() },
        QuickMenuItem { id: "delete".into(), label: "Delete".into(), separator: false, shortcut: "Del".into() },
        QuickMenuItem { id: "copy_path".into(), label: "Copy path".into(), separator: false, shortcut: "Ctrl+Shift+C".into() },
        QuickMenuItem { id: "_sep1".into(), label: String::new(), separator: true, shortcut: String::new() },
        QuickMenuItem { id: "more_actions".into(), label: "More actions\u{2026}".into(), separator: false, shortcut: String::new() },
    ]
}

#[tauri::command]
pub fn copy_path_to_clipboard(path: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows::Win32::System::DataExchange::*;
        use windows::Win32::System::Memory::*;

        unsafe {
            let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
            let byte_len = wide.len() * 2;

            let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_len)
                .map_err(|e| format!("GlobalAlloc: {e}"))?;

            let ptr = GlobalLock(hmem);
            if ptr.is_null() {
                return Err("GlobalLock failed".into());
            }
            std::ptr::copy_nonoverlapping(wide.as_ptr() as *const u8, ptr as *mut u8, byte_len);
            let _ = GlobalUnlock(hmem);

            OpenClipboard(None).map_err(|e| format!("OpenClipboard: {e}"))?;
            let _ = EmptyClipboard();

            let result = SetClipboardData(13, Some(windows::Win32::Foundation::HANDLE(hmem.0 as *mut _)));
            let _ = CloseClipboard();

            if result.is_err() {
                return Err("SetClipboardData failed".into());
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn delete_to_recycle_bin(path: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::*;

        let mut wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
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
    Ok(())
}

#[tauri::command]
pub fn show_native_context_menu(
    app: tauri::AppHandle,
    path: String,
    screen_x: i32,
    screen_y: i32,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows::core::PCWSTR;
        use windows::Win32::System::Com::*;
        use windows::Win32::UI::Shell::Common::ITEMIDLIST;
        use windows::Win32::UI::Shell::*;
        use windows::Win32::UI::WindowsAndMessaging::*;

        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let hwnd = get_tauri_hwnd(&app)?;

            let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

            let mut pidl_raw: *mut ITEMIDLIST = std::ptr::null_mut();
            SHParseDisplayName(PCWSTR(wide_path.as_ptr()), None, &mut pidl_raw, 0, None)
                .map_err(|e| format!("SHParseDisplayName: {e}"))?;

            let mut child_pidl: *mut ITEMIDLIST = std::ptr::null_mut();
            let parent_folder: IShellFolder =
                SHBindToParent(pidl_raw, Some(&mut child_pidl))
                    .map_err(|e| format!("SHBindToParent: {e}"))?;

            let child_const: *const ITEMIDLIST = child_pidl;

            let ctx_menu: IContextMenu = parent_folder
                .GetUIObjectOf(hwnd, &[child_const], None)
                .map_err(|e| format!("GetUIObjectOf: {e}"))?;

            CoTaskMemFree(Some(pidl_raw as *const _ as _));

            let hmenu = CreatePopupMenu().map_err(|e| format!("CreatePopupMenu: {e}"))?;

            let hr = ctx_menu.QueryContextMenu(hmenu, 0, 1, 30000, CMF_NORMAL);
            if hr.is_err() {
                let _ = DestroyMenu(hmenu);
                return Err(format!("QueryContextMenu failed: {hr:?}"));
            }

            let cmd = TrackPopupMenu(
                hmenu,
                TPM_RETURNCMD | TPM_LEFTBUTTON | TPM_RIGHTBUTTON,
                screen_x,
                screen_y,
                None,
                hwnd,
                None,
            );

            if cmd.as_bool() {
                let cmd_id = cmd.0 as u32;
                if cmd_id >= 1 {
                    let info = CMINVOKECOMMANDINFO {
                        cbSize: std::mem::size_of::<CMINVOKECOMMANDINFO>() as u32,
                        hwnd,
                        lpVerb: windows::core::PCSTR((cmd_id - 1) as *const u8),
                        nShow: SW_SHOWNORMAL.0,
                        ..Default::default()
                    };
                    let _ = ctx_menu.InvokeCommand(&info);
                }
            }

            let _ = DestroyMenu(hmenu);
        }
    }
    Ok(())
}

#[cfg(windows)]
unsafe fn get_tauri_hwnd(
    app: &tauri::AppHandle,
) -> Result<windows::Win32::Foundation::HWND, String> {
    use tauri::Manager;

    let window = app.get_webview_window("main").ok_or("No main window")?;
    let hwnd_raw = window.hwnd().map_err(|e| format!("hwnd: {e}"))?;
    Ok(windows::Win32::Foundation::HWND(hwnd_raw.0))
}
