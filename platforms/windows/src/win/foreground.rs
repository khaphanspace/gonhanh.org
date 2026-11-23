//! Foreground app watcher (macOS NSWorkspace app switch) + non-Latin keyboard check (macOS InputSourceObserver)

use std::cell::Cell;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{CloseHandle, HWND};
use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT};

use crate::app;

thread_local! {
    static HOOK: Cell<HWINEVENTHOOK> = const { Cell::new(null_mut()) };
}

/// Primary language ids (LANGID & 0x3FF) of non-Latin scripts, same list as macOS nonLatinLanguages.
/// Vietnamese is not blocked: on Windows it is commonly used with the US layout.
const NON_LATIN: &[u16] = &[
    0x11, 0x04, 0x12, // ja, zh, ko
    0x1E, 0x53, 0x54, 0x55, // th, km, lo, my
    0x39, 0x4E, 0x61, 0x4F, 0x45, 0x49, 0x4A, 0x4B, 0x4C, // hi, mr, ne, sa, bn, ta, te, kn, ml
    0x47, 0x46, 0x48, 0x5B, // gu, pa, or, si
    0x01, 0x0D, 0x29, 0x20, // ar, he, fa, ur
    0x19, 0x22, 0x23, 0x02, 0x2F, 0x08, 0x37, 0x2B, 0x5E, 0x73, // ru, uk, be, bg, mk, el, ka, hy, am, ti
];

/// Process name of a window, e.g. "chrome.exe"
pub fn process_name_of_pid(pid: u32) -> Option<String> {
    unsafe {
        let proc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if proc.is_null() {
            return None;
        }
        let mut buf = [0u16; 520];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(proc, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut size) != 0;
        CloseHandle(proc);
        ok.then(|| {
            let path = String::from_utf16_lossy(&buf[..size as usize]);
            path.rsplit('\\').next().unwrap_or_default().to_string()
        })
    }
}

pub fn process_name(hwnd: HWND) -> Option<String> {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    if pid == 0 {
        return None;
    }
    process_name_of_pid(pid)
}

pub fn current_app() -> Option<String> {
    process_name(unsafe { GetForegroundWindow() })
}

pub fn is_non_latin_layout() -> bool {
    unsafe {
        let thread = GetWindowThreadProcessId(GetForegroundWindow(), null_mut());
        let lang = GetKeyboardLayout(thread) as usize as u16;
        NON_LATIN.contains(&(lang & 0x3FF))
    }
}

unsafe extern "system" fn on_foreground(_: HWINEVENTHOOK, _: u32, _: HWND, _: i32, _: i32, _: u32, _: u32) {
    if let Some(name) = current_app() {
        app::try_with(|c| c.on_app_changed(&name));
    }
}

pub fn start() {
    let hook = unsafe { SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, null_mut(), Some(on_foreground), 0, 0, WINEVENT_OUTOFCONTEXT) };
    HOOK.set(hook);
}

pub fn stop() {
    let hook = HOOK.replace(null_mut());
    if !hook.is_null() {
        unsafe { UnhookWinEvent(hook) };
    }
}
