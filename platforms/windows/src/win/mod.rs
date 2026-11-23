//! Win32 layer: hooks, text output, foreground watcher, tray, settings window, small OS helpers

pub mod dark;
pub mod dialogs;
pub mod foreground;
pub mod hook;
pub mod icon;
pub mod registry;
pub mod send;
pub mod settings_window;
pub mod theme;
pub mod tray;

use std::ptr::null_mut;

use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::Graphics::Gdi::{CreateFontIndirectW, FW_SEMIBOLD, HFONT};
use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};
use windows_sys::Win32::Networking::WinInet::*;
use windows_sys::Win32::System::Diagnostics::Debug::OutputDebugStringW;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;
use windows_sys::Win32::UI::HiDpi::SystemParametersInfoForDpi;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetKeyNameTextW, MapVirtualKeyW, MAPVK_VK_TO_VSC};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::keys;

/// UTF-16, NUL-terminated
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn from_wide(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

pub fn debug_output(message: &str) {
    let w = wide(&format!("{message}\n"));
    unsafe { OutputDebugStringW(w.as_ptr()) };
}

pub fn local_time() -> (u16, u16, u16, u16) {
    let mut t: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&mut t) };
    (t.wHour, t.wMinute, t.wSecond, t.wMilliseconds)
}

/// Layout-aware key name from Windows (for keys without a fixed name)
pub fn key_name_text(vk: u16) -> Option<String> {
    let mut scan = (unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as i32) << 16;
    if keys::is_navigation(vk) || vk == keys::DELETE {
        scan |= 1 << 24; // extended key
    }
    let mut buf = [0u16; 64];
    let n = unsafe { GetKeyNameTextW(scan, buf.as_mut_ptr(), 64) };
    (n > 0).then(|| from_wide(&buf))
}

/// Open a URL (or a file with its default app)
pub fn open_url(url: &str) {
    let (verb, target) = (wide("open"), wide(url));
    unsafe { ShellExecuteW(null_mut(), verb.as_ptr(), target.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL) };
}

/// Short Windows "Speech On/Off" sounds, like macOS Tink/Pop
pub fn play_toggle_sound(on: bool) {
    let dir = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    let file = format!("{dir}\\Media\\{}", if on { "Speech On.wav" } else { "Speech Off.wav" });
    let w = wide(&file);
    // SND_NODEFAULT: silent if the file is missing (sound is cosmetic)
    unsafe { PlaySoundW(w.as_ptr(), null_mut(), SND_FILENAME | SND_ASYNC | SND_NODEFAULT) };
}

/// HTTPS GET via WinINet (system proxy settings apply)
pub fn http_get(url: &str) -> Result<Vec<u8>, String> {
    struct Handle(*mut std::ffi::c_void);
    impl Drop for Handle {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { InternetCloseHandle(self.0) };
            }
        }
    }
    let agent = wide("GoNhanh-Windows");
    let net = Handle(unsafe { InternetOpenW(agent.as_ptr(), INTERNET_OPEN_TYPE_PRECONFIG, std::ptr::null(), std::ptr::null(), 0) });
    if net.0.is_null() {
        return Err("Không mở được kết nối mạng".into());
    }
    let (url_w, headers) = (wide(url), wide("Accept: application/vnd.github.v3+json\r\n"));
    let flags = INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE | INTERNET_FLAG_SECURE;
    let req = Handle(unsafe { InternetOpenUrlW(net.0, url_w.as_ptr(), headers.as_ptr(), u32::MAX, flags, 0) });
    if req.0.is_null() {
        return Err("Không kết nối được tới GitHub".into());
    }
    let (mut status, mut size) = (0u32, 4u32);
    unsafe { HttpQueryInfoW(req.0, HTTP_QUERY_STATUS_CODE | HTTP_QUERY_FLAG_NUMBER, &mut status as *mut _ as _, &mut size, null_mut()) };
    if status != 200 {
        return Err(format!("GitHub trả lỗi {status}"));
    }
    let mut body = Vec::new();
    let mut buf = vec![0u8; 16384];
    loop {
        let mut read = 0u32;
        if unsafe { InternetReadFile(req.0, buf.as_mut_ptr() as _, buf.len() as u32, &mut read) } == 0 {
            return Err("Tải dữ liệu bị gián đoạn".into());
        }
        if read == 0 {
            break;
        }
        body.extend_from_slice(&buf[..read as usize]);
    }
    Ok(body)
}

// ---------------------------------------------------------------- UI helpers

pub fn scale(px: i32, dpi: u32) -> i32 {
    px * dpi as i32 / 96
}

/// System message font (Segoe UI 9pt) at `dpi`, optionally bold / resized by `point_delta`
pub fn ui_font(dpi: u32, bold: bool, point_delta: i32) -> HFONT {
    unsafe {
        let mut ncm: NONCLIENTMETRICSW = std::mem::zeroed();
        ncm.cbSize = std::mem::size_of::<NONCLIENTMETRICSW>() as u32;
        SystemParametersInfoForDpi(SPI_GETNONCLIENTMETRICS, ncm.cbSize, &mut ncm as *mut _ as _, 0, dpi);
        let mut lf = ncm.lfMessageFont;
        if point_delta != 0 {
            let points = -lf.lfHeight * 72 / dpi as i32 + point_delta;
            lf.lfHeight = -(points * dpi as i32 / 72);
        }
        if bold {
            lf.lfWeight = FW_SEMIBOLD as i32;
        }
        CreateFontIndirectW(&lf)
    }
}

/// MAKEINTRESOURCEW: a resource id smuggled in a pointer (not a dangling pointer, id 1 must stay 1)
#[allow(clippy::manual_dangling_ptr)]
pub fn make_int_resource(id: u16) -> *const u16 {
    id as usize as *const u16
}

/// NUL-terminated UTF-16 string from a pointer (null → ""), reading at most `max` units
///
/// # Safety
/// `p` must be null or point to a NUL-terminated string (or at least `max` readable units)
pub unsafe fn read_wide_z(p: *const u16, max: usize) -> String {
    if p.is_null() {
        return String::new();
    }
    let mut len = 0;
    while len < max && unsafe { *p.add(len) } != 0 {
        len += 1;
    }
    String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(p, len) })
}
