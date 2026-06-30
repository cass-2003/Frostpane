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
fn find_worker_w() -> Option<HWND> {
    unsafe {
        WORKER_W = None;
        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(0));
        WORKER_W
    }
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

        // Try to find existing WorkerW first
        let mut worker = find_worker_w();

        if worker.is_none() {
            // Send the undocumented message to spawn WorkerW
            // Try different parameter combinations
            SendMessageTimeoutW(progman, 0x052C, WPARAM(0xD), LPARAM(0x1), SMTO_NORMAL, 1000, None);
            std::thread::sleep(std::time::Duration::from_millis(100));
            worker = find_worker_w();
        }

        if worker.is_none() {
            // Try with different params
            SendMessageTimeoutW(progman, 0x052C, WPARAM(0), LPARAM(0), SMTO_NORMAL, 1000, None);
            std::thread::sleep(std::time::Duration::from_millis(100));
            worker = find_worker_w();
        }

        let worker = match worker {
            Some(w) => {
                info!("Frostpane: Found WorkerW, embedding as child");
                w
            }
            None => {
                // Final fallback: embed directly under Progman
                info!("Frostpane: WorkerW not found, embedding under Progman");
                progman
            }
        };

        let _ = SetParent(hwnd, Some(worker));

        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let new_style = (style & !(WS_POPUP.0)) | WS_CHILD.0;
        SetWindowLongW(hwnd, GWL_STYLE, new_style as i32);

        // Remove WS_EX_APPWINDOW so it doesn't show in taskbar
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        let new_ex_style = ex_style & !(WS_EX_APPWINDOW.0);
        SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex_style as i32);

        let mut rect = std::mem::zeroed();
        let _ = GetClientRect(worker, &mut rect);

        // If rect is zero (Progman might report zero), use screen size
        let (w, h) = if rect.right > 0 && rect.bottom > 0 {
            (rect.right - rect.left, rect.bottom - rect.top)
        } else {
            (
                GetSystemMetrics(SM_CXSCREEN),
                GetSystemMetrics(SM_CYSCREEN),
            )
        };

        let _ = SetWindowPos(
            hwnd, None, 0, 0, w, h,
            SWP_NOZORDER | SWP_FRAMECHANGED,
        );

        let _ = ShowWindow(hwnd, SW_SHOW);
        info!("Frostpane: Embedded in desktop ({}x{})", w, h);
    }
}

#[cfg(windows)]
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _lparam: LPARAM) -> windows::core::BOOL {
    if let Some(_shell_view) = find_window_ex(Some(hwnd), None, "SHELLDLL_DefView") {
        // Found SHELLDLL_DefView inside this window.
        // The WorkerW we need is the NEXT sibling WorkerW after this window.
        if let Some(next_worker) = find_window_ex(None, Some(hwnd), "WorkerW") {
            WORKER_W = Some(next_worker);
            return windows::core::BOOL(0); // stop
        }
    }
    windows::core::BOOL(1)
}

#[cfg(windows)]
pub fn hide_desktop_icons() {
    unsafe {
        if let Some(shell_view) = find_shell_def_view() {
            if let Some(list_view) = find_window_ex(Some(shell_view), None, "SysListView32") {
                let _ = ShowWindow(list_view, SW_HIDE);
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
                let _ = ShowWindow(list_view, SW_SHOW);
                info!("Frostpane: Desktop icons restored");
            }
        }
    }
}

#[cfg(windows)]
fn find_shell_def_view() -> Option<HWND> {
    if let Some(progman) = find_window("Progman") {
        if let Some(sv) = find_window_ex(Some(progman), None, "SHELLDLL_DefView") {
            return Some(sv);
        }
    }
    // Fallback: enumerate all top-level windows
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
