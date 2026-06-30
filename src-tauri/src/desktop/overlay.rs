use log::info;

#[cfg(windows)]
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::*;

#[cfg(windows)]
static mut WORKER_W: Option<HWND> = None;

#[cfg(windows)]
fn find_window(class: &str) -> Option<HWND> {
    use windows::core::PCWSTR;
    let wide: Vec<u16> = class.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe { FindWindowW(PCWSTR(wide.as_ptr()), None).ok() }
}

#[cfg(windows)]
fn find_window_ex(parent: Option<HWND>, after: Option<HWND>, class: &str) -> Option<HWND> {
    use windows::core::PCWSTR;
    let wide: Vec<u16> = class.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe { FindWindowExW(parent, after, PCWSTR(wide.as_ptr()), None).ok() }
}

#[cfg(windows)]
pub fn embed_in_desktop(tauri_hwnd: isize) {
    unsafe {
        let hwnd = HWND(tauri_hwnd as *mut _);

        let progman = match find_window("Progman") {
            Some(h) => h,
            None => {
                info!("Frostpane: Progman not found");
                return;
            }
        };
        info!("Frostpane: Found Progman");

        SendMessageTimeoutW(progman, 0x052C, WPARAM(0xD), LPARAM(0x1), SMTO_NORMAL, 1000, None);

        WORKER_W = None;
        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(0));

        let worker = match WORKER_W {
            Some(w) => w,
            None => {
                info!("Frostpane: WorkerW not found, fallback to always-on-bottom");
                let _ = SetWindowPos(hwnd, Some(HWND_BOTTOM), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                ShowWindow(hwnd, SW_SHOW);
                return;
            }
        };
        info!("Frostpane: Found WorkerW");

        let _ = SetParent(hwnd, Some(worker));

        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let new_style = (style & !(WS_POPUP.0)) | WS_CHILD.0;
        SetWindowLongW(hwnd, GWL_STYLE, new_style as i32);

        let mut rect = std::mem::zeroed();
        let _ = GetClientRect(worker, &mut rect);
        let _ = SetWindowPos(
            hwnd, None, 0, 0,
            rect.right - rect.left, rect.bottom - rect.top,
            SWP_NOZORDER | SWP_FRAMECHANGED,
        );

        ShowWindow(hwnd, SW_SHOW);
        info!("Frostpane: Embedded in desktop ({}x{})", rect.right - rect.left, rect.bottom - rect.top);
    }
}

#[cfg(windows)]
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _lparam: LPARAM) -> windows::core::BOOL {
    if let Some(shell_view) = find_window_ex(Some(hwnd), None, "SHELLDLL_DefView") {
        let _ = shell_view;
        if let Some(next_worker) = find_window_ex(None, Some(hwnd), "WorkerW") {
            WORKER_W = Some(next_worker);
        }
    }
    windows::core::BOOL(1)
}

#[cfg(windows)]
pub fn hide_desktop_icons() {
    unsafe {
        if let Some(shell_view) = find_shell_def_view() {
            if let Some(list_view) = find_window_ex(Some(shell_view), None, "SysListView32") {
                ShowWindow(list_view, SW_HIDE);
                info!("Frostpane: Desktop icons hidden");
            }
        }
    }
}

#[cfg(windows)]
pub fn show_desktop_icons() {
    unsafe {
        if let Some(shell_view) = find_shell_def_view() {
            if let Some(list_view) = find_window_ex(Some(shell_view), None, "SysListView32") {
                ShowWindow(list_view, SW_SHOW);
                info!("Frostpane: Desktop icons restored");
            }
        }
    }
}

#[cfg(windows)]
fn find_shell_def_view() -> Option<HWND> {
    // Try under Progman first
    if let Some(progman) = find_window("Progman") {
        if let Some(sv) = find_window_ex(Some(progman), None, "SHELLDLL_DefView") {
            return Some(sv);
        }
    }
    // Fallback: enumerate WorkerW windows
    unsafe {
        WORKER_W = None;
        let _ = EnumWindows(Some(find_shell_view_proc), LPARAM(0));
        if let Some(parent) = WORKER_W {
            return find_window_ex(Some(parent), None, "SHELLDLL_DefView");
        }
    }
    None
}

#[cfg(windows)]
unsafe extern "system" fn find_shell_view_proc(hwnd: HWND, _lparam: LPARAM) -> windows::core::BOOL {
    if find_window_ex(Some(hwnd), None, "SHELLDLL_DefView").is_some() {
        WORKER_W = Some(hwnd);
        return windows::core::BOOL(0);
    }
    windows::core::BOOL(1)
}

#[cfg(not(windows))]
pub fn embed_in_desktop(_tauri_hwnd: isize) {}

#[cfg(not(windows))]
pub fn hide_desktop_icons() {}

#[cfg(not(windows))]
pub fn show_desktop_icons() {}
