use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct DesktopIcon {
    pub name: String,
    pub path: String,
    pub icon_data: String,
    pub icon_size: i32,
    pub x: i32,
    pub y: i32,
    pub is_shortcut: bool,
}

pub fn enumerate_desktop_items() -> Vec<DesktopIcon> {
    let mut items = Vec::new();
    let mut col = 0;
    let mut row = 0;
    let icon_spacing_x = 90;
    let icon_spacing_y = 100;
    let start_x = 20;
    let start_y = 20;
    let max_rows = 8;

    for dir in desktop_dirs() {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = display_name(&path);

                if name.starts_with('.') || name == "desktop.ini" {
                    continue;
                }

                let is_shortcut = path
                    .extension()
                    .map(|e| e.eq_ignore_ascii_case("lnk"))
                    .unwrap_or(false);

                let display = if is_shortcut {
                    name.trim_end_matches(".lnk").to_string()
                } else {
                    name.clone()
                };

                let icon_data = extract_icon_base64(&path);

                let x = start_x + col * icon_spacing_x;
                let y = start_y + row * icon_spacing_y;

                items.push(DesktopIcon {
                    name: display,
                    path: path.to_string_lossy().to_string(),
                    icon_data,
                    icon_size: 48,
                    x,
                    y,
                    is_shortcut,
                });

                row += 1;
                if row >= max_rows {
                    row = 0;
                    col += 1;
                }
            }
        }
    }

    items
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

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(windows)]
fn extract_icon_base64(path: &Path) -> String {
    use windows::Win32::UI::Shell::SHGetFileInfoW;
    use windows::Win32::UI::Shell::SHFILEINFOW;
    use windows::Win32::UI::Shell::SHGFI_ICON;
    use windows::Win32::UI::Shell::SHGFI_LARGEICON;
    use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
        SelectObject, CreateCompatibleBitmap, GetDC, ReleaseDC,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::WindowsAndMessaging::DrawIconEx;
    use windows::Win32::UI::WindowsAndMessaging::DI_NORMAL;
    use windows::core::PCWSTR;
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;

    let wide_path: Vec<u16> = path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut shfi = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        if result == 0 || shfi.hIcon.is_invalid() {
            return String::new();
        }

        let icon = shfi.hIcon;
        let size: i32 = 48;

        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let hbmp = CreateCompatibleBitmap(hdc_screen, size, size);
        let old_bmp = SelectObject(hdc_mem, hbmp.into());

        // Fill with transparent black
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let _ = DrawIconEx(
            hdc_mem,
            0, 0,
            icon,
            size, size,
            0,
            None,
            DI_NORMAL,
        );

        let mut pixels = vec![0u8; (size * size * 4) as usize];
        GetDIBits(
            hdc_mem,
            hbmp,
            0,
            size as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // BGRA -> RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        // Encode as PNG (simple: we'll use raw RGBA for now, wrapped in base64)
        // For M0 we'll just pass raw pixel data; a proper PNG encoder comes later
        let encoded = STANDARD.encode(&pixels);

        SelectObject(hdc_mem, old_bmp);
        let _ = DeleteObject(hbmp.into());
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);
        let _ = DestroyIcon(icon);

        encoded
    }
}

#[cfg(not(windows))]
fn extract_icon_base64(_path: &Path) -> String {
    String::new()
}
