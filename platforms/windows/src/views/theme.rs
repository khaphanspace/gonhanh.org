//! Context menus that follow the Windows app light/dark mode, like Explorer's.
//! Uses the uxtheme exports Explorer, Notepad++ etc. rely on (not documented): if they are missing,
//! menus just stay light.

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

use crate::win::wide;

type SetPreferredAppMode = unsafe extern "system" fn(i32) -> i32; // uxtheme #135 (1 = allow dark)
type AllowDarkModeForWindow = unsafe extern "system" fn(HWND, i32) -> u8; // uxtheme #133
type FlushMenuThemes = unsafe extern "system" fn(); // uxtheme #136

fn export(ordinal: usize) -> Option<unsafe extern "system" fn() -> isize> {
    let dll = wide("uxtheme.dll");
    unsafe {
        let module = LoadLibraryW(dll.as_ptr());
        if module.is_null() {
            return None;
        }
        GetProcAddress(module, ordinal as *const u8)
    }
}

/// Call once at startup, before creating windows
pub fn init() {
    if let Some(f) = export(135) {
        let set_mode: SetPreferredAppMode = unsafe { std::mem::transmute(f) };
        unsafe { set_mode(1) };
    }
    refresh();
}

/// Menus owned by `hwnd` may go dark
pub fn allow_for_window(hwnd: HWND) {
    if let Some(f) = export(133) {
        let allow: AllowDarkModeForWindow = unsafe { std::mem::transmute(f) };
        unsafe { allow(hwnd, 1) };
    }
}

/// Re-read light/dark after the user switched it
pub fn refresh() {
    if let Some(f) = export(136) {
        let flush: FlushMenuThemes = unsafe { std::mem::transmute(f) };
        unsafe { flush() };
    }
}

/// Windows "app mode" is dark (Settings › Personalization › Colors › "Choose your default app mode")
pub fn apps_dark() -> bool {
    crate::services::registry::get_dword(
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
        "AppsUseLightTheme",
    ) == Some(0)
}
