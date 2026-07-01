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

                let (icon_data, icon_size) = extract_icon_png(&path);

                let x = start_x + col * icon_spacing_x;
                let y = start_y + row * icon_spacing_y;

                items.push(DesktopIcon {
                    name: display,
                    path: path.to_string_lossy().to_string(),
                    icon_data,
                    icon_size,
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
pub fn extract_icon_png(path: &Path) -> (String, i32) {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::Controls::IImageList;
    use windows::Win32::UI::Shell::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    // SHIL_JUMBO = 4 → 256x256, SHIL_EXTRALARGE = 2 → 48x48
    const SHIL_JUMBO: i32 = 0x4;

    let render_size: i32 = 256;

    let wide_path: Vec<u16> = path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        // Step 1: Get the system icon index for this file
        let mut shfi = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_SYSICONINDEX,
        );

        if result == 0 {
            return (String::new(), 48);
        }

        let icon_index = shfi.iIcon;

        // Step 2: Get the jumbo image list and extract HICON at full resolution
        let icon = get_icon_from_imagelist(icon_index, SHIL_JUMBO, render_size)
            .or_else(|| {
                // Fallback to SHGFI_ICON if image list fails
                let mut shfi2 = SHFILEINFOW::default();
                SHGetFileInfoW(
                    PCWSTR(wide_path.as_ptr()),
                    windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
                    Some(&mut shfi2),
                    std::mem::size_of::<SHFILEINFOW>() as u32,
                    SHGFI_ICON | SHGFI_LARGEICON,
                );
                if !shfi2.hIcon.is_invalid() {
                    Some(shfi2.hIcon)
                } else {
                    None
                }
            });

        let icon = match icon {
            Some(h) => h,
            None => return (String::new(), 48),
        };

        // Step 3: Render HICON to a DIB section with alpha
        let (rgba, actual_size) = render_hicon_to_rgba(icon, render_size);
        let _ = DestroyIcon(icon);

        if rgba.is_empty() {
            return (String::new(), 48);
        }

        // Step 4: Encode as PNG
        let png_data = encode_png(&rgba, actual_size as u32, actual_size as u32);
        (STANDARD.encode(&png_data), actual_size)
    }
}

#[cfg(windows)]
unsafe fn get_icon_from_imagelist(
    icon_index: i32,
    image_list_type: i32,
    _size: i32,
) -> Option<windows::Win32::UI::WindowsAndMessaging::HICON> {
    use windows::Win32::UI::Controls::IImageList;
    use windows::Win32::UI::Shell::SHGetImageList;

    let image_list: IImageList = SHGetImageList(image_list_type).ok()?;

    // ILD_TRANSPARENT = 1
    let hicon = image_list.GetIcon(icon_index, 0x1).ok()?;

    if hicon.is_invalid() {
        None
    } else {
        Some(hicon)
    }
}

#[cfg(windows)]
unsafe fn render_hicon_to_rgba(
    icon: windows::Win32::UI::WindowsAndMessaging::HICON,
    size: i32,
) -> (Vec<u8>, i32) {
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -(size),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let hdc_screen = GetDC(None);
    let hdc_mem = CreateCompatibleDC(Some(hdc_screen));

    let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let hbmp = CreateDIBSection(
        Some(hdc_mem),
        &bmi,
        DIB_RGB_COLORS,
        &mut bits_ptr,
        None,
        0,
    );

    if hbmp.is_err() || bits_ptr.is_null() {
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);
        return (Vec::new(), size);
    }

    let hbmp = hbmp.unwrap();
    let old_bmp = SelectObject(hdc_mem, hbmp.into());

    let pixel_count = (size * size) as usize;
    std::ptr::write_bytes(bits_ptr as *mut u8, 0, pixel_count * 4);

    let _ = DrawIconEx(hdc_mem, 0, 0, icon, size, size, 0, None, DI_NORMAL);

    let pixels_slice = std::slice::from_raw_parts(bits_ptr as *const u8, pixel_count * 4);

    let mut rgba = vec![0u8; pixel_count * 4];
    for i in 0..pixel_count {
        let off = i * 4;
        rgba[off] = pixels_slice[off + 2];     // R
        rgba[off + 1] = pixels_slice[off + 1]; // G
        rgba[off + 2] = pixels_slice[off];     // B
        rgba[off + 3] = pixels_slice[off + 3]; // A
    }

    // Fix alpha: if all alpha values are 0, the icon lacks alpha channel
    let all_alpha_zero = rgba.chunks(4).all(|p| p[3] == 0);
    if all_alpha_zero {
        for chunk in rgba.chunks_exact_mut(4) {
            if chunk[0] != 0 || chunk[1] != 0 || chunk[2] != 0 {
                chunk[3] = 255;
            }
        }
    }

    SelectObject(hdc_mem, old_bmp);
    let _ = DeleteObject(hbmp.into());
    let _ = DeleteDC(hdc_mem);
    ReleaseDC(None, hdc_screen);

    (rgba, size)
}

fn encode_png(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut buf, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        if let Ok(mut writer) = encoder.write_header() {
            let _ = writer.write_image_data(rgba);
        }
    }
    buf
}

#[cfg(not(windows))]
pub fn extract_icon_png(_path: &Path) -> (String, i32) {
    (String::new(), 48)
}
