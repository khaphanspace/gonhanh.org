//! Tray glyph like the Windows language indicator (ENG/VIE): just the letter, no background,
//! white on a dark taskbar and black on a light one.

use std::ptr::null_mut;

use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, HICON, ICONINFO};

use super::{registry, wide};

const PERSONALIZE: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";

/// Taskbar/tray uses the light theme (Settings › Personalization › Colors › "Windows mode")
pub fn taskbar_is_light() -> bool {
    registry::get_dword(PERSONALIZE, "SystemUsesLightTheme") == Some(1)
}

/// 32-bit top-down DIB; returns (bitmap, pixel pointer)
fn dib(w: i32, h: i32) -> (HBITMAP, *mut u32) {
    let mut bmi: BITMAPINFO = unsafe { std::mem::zeroed() };
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = w;
    bmi.bmiHeader.biHeight = -h; // top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;
    let mut bits = null_mut();
    let bmp =
        unsafe { CreateDIBSection(null_mut(), &bmi, DIB_RGB_COLORS, &mut bits, null_mut(), 0) };
    (bmp, bits as *mut u32)
}

/// Coverage (0..=255) of `letter` (Segoe UI Semibold) in a size×size box, centered on its ink.
/// Drawn at the real size so the font's hinting snaps stems and bars to whole pixels (crisp like system text).
pub fn glyph_coverage(letter: &str, size: i32) -> Vec<u8> {
    let canvas = size * 2; // room so nothing clips before cropping to the ink
    let mut ink = vec![0u8; (canvas * canvas) as usize];
    unsafe {
        let dc = CreateCompatibleDC(null_mut());
        let (bmp, bits) = dib(canvas, canvas);
        let old = SelectObject(dc, bmp as HGDIOBJ);
        // em = icon size → cap height ≈ 70% of the icon, like the system indicator letters
        let face = wide("Segoe UI");
        let font = CreateFontW(
            -size,
            0,
            0,
            0,
            FW_SEMIBOLD as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            0,
            0,
            ANTIALIASED_QUALITY as u32,
            0,
            face.as_ptr(),
        );
        let old_font = SelectObject(dc, font as HGDIOBJ);
        SetBkMode(dc, TRANSPARENT as i32);
        SetTextColor(dc, 0x00FF_FFFF); // white on black: any channel = coverage
        let mut r = RECT {
            left: 0,
            top: 0,
            right: canvas,
            bottom: canvas,
        };
        let text = wide(letter);
        DrawTextW(
            dc,
            text.as_ptr(),
            -1,
            &mut r,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        GdiFlush();
        let px = std::slice::from_raw_parts(bits, (canvas * canvas) as usize);
        for (dst, src) in ink.iter_mut().zip(px) {
            *dst = (src >> 16) as u8;
        }
        SelectObject(dc, old_font);
        DeleteObject(font as HGDIOBJ);
        SelectObject(dc, old);
        DeleteObject(bmp as HGDIOBJ);
        DeleteDC(dc);
    }

    // Ink bounding box
    let (mut x0, mut y0, mut x1, mut y1) = (canvas, canvas, -1, -1);
    for y in 0..canvas {
        for x in 0..canvas {
            if ink[(y * canvas + x) as usize] > 0 {
                x0 = x0.min(x);
                x1 = x1.max(x);
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
        }
    }
    let mut out = vec![0u8; (size * size) as usize];
    if x1 < 0 {
        return out;
    }
    // Whole-pixel shift that centers the ink (keeps the hinted edges on the pixel grid)
    let dx = (size - (x1 - x0 + 1)) / 2 - x0;
    let dy = (size - (y1 - y0 + 1)) / 2 - y0;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (ox, oy) = (x + dx, y + dy);
            if (0..size).contains(&ox) && (0..size).contains(&oy) {
                out[(oy * size + ox) as usize] = ink[(y * canvas + x) as usize];
            }
        }
    }
    out
}

/// Letter icon in `rgb` (0xRRGGBB) with real alpha
pub fn make(letter: &str, size: i32, rgb: u32) -> HICON {
    let coverage = glyph_coverage(letter, size);
    let (color, bits) = dib(size, size);
    unsafe {
        let px = std::slice::from_raw_parts_mut(bits, (size * size) as usize);
        for (p, a) in px.iter_mut().zip(&coverage) {
            *p = (*a as u32) << 24 | (rgb & 0x00FF_FFFF); // BGRA in memory, straight alpha
        }
        let mask = CreateBitmap(size, size, 1, 1, null_mut()); // all 0: alpha decides
        let ii = ICONINFO {
            fIcon: 1,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask,
            hbmColor: color,
        };
        let icon = CreateIconIndirect(&ii);
        DeleteObject(color as HGDIOBJ);
        DeleteObject(mask as HGDIOBJ);
        icon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_have_ink_and_are_centered() {
        for letter in ["V", "E"] {
            for size in [16, 20, 24, 32] {
                let c = glyph_coverage(letter, size);
                let rows: Vec<i32> = (0..size)
                    .filter(|y| (0..size).any(|x| c[(y * size + x) as usize] > 32))
                    .collect();
                let cols: Vec<i32> = (0..size)
                    .filter(|x| (0..size).any(|y| c[(y * size + x) as usize] > 32))
                    .collect();
                assert!(!rows.is_empty(), "{letter}@{size} has no ink");
                let (top, bottom) = (rows[0], size - 1 - rows[rows.len() - 1]);
                let (left, right) = (cols[0], size - 1 - cols[cols.len() - 1]);
                assert!(
                    (top - bottom).abs() <= 1 && (left - right).abs() <= 1,
                    "{letter}@{size} off-center: t{top} b{bottom} l{left} r{right}"
                );
                let height = rows.len() as i32;
                assert!(
                    height * 100 >= size * 55 && height * 100 <= size * 85,
                    "{letter}@{size} height {height}"
                );
            }
        }
    }

    /// Writes enlarged previews on dark and light backgrounds to %TEMP%\gonhanh_icons\*.bmp
    #[test]
    #[ignore]
    fn dump_previews() {
        let dir = std::env::temp_dir().join("gonhanh_icons");
        std::fs::create_dir_all(&dir).unwrap();
        for size in [16, 20, 24, 32] {
            for (bg, fg, name) in [(0x20u8, 255u8, "dark"), (0xF3, 0, "light")] {
                let zoom = 12;
                let w = size * 2 * zoom + zoom; // V, gap, E
                let h = size * zoom;
                let mut rgb = vec![bg; (w * h * 3) as usize];
                for (i, letter) in ["V", "E"].iter().enumerate() {
                    let c = glyph_coverage(letter, size);
                    for y in 0..h {
                        for x in 0..size * zoom {
                            let a = c[((y / zoom) * size + x / zoom) as usize] as u32;
                            let v = ((fg as u32 * a + bg as u32 * (255 - a)) / 255) as u8;
                            let px = x + i as i32 * (size * zoom + zoom);
                            let o = ((y * w + px) * 3) as usize;
                            rgb[o..o + 3].copy_from_slice(&[v, v, v]);
                        }
                    }
                }
                std::fs::write(dir.join(format!("{name}_{size}.bmp")), bmp24(w, h, &rgb)).unwrap();
            }
        }
    }

    fn bmp24(w: i32, h: i32, rgb: &[u8]) -> Vec<u8> {
        let row = ((w * 3 + 3) / 4 * 4) as usize;
        let size = 54 + row * h as usize;
        let mut out = Vec::with_capacity(size);
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&(size as u32).to_le_bytes());
        out.extend_from_slice(&[0; 4]);
        out.extend_from_slice(&54u32.to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&w.to_le_bytes());
        out.extend_from_slice(&(-h).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        out.extend_from_slice(&[0; 24]);
        for y in 0..h as usize {
            let start = out.len();
            out.extend_from_slice(&rgb[y * w as usize * 3..(y + 1) * w as usize * 3]);
            out.resize(start + row, 0);
        }
        out
    }
}
