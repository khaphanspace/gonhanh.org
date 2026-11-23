//! Low-level keyboard + mouse hooks (like macOS CGEventTap). Only collects events; the controller decides.

use std::cell::Cell;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, VK_CAPITAL, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use super::send::INJECTED_MARKER;
use crate::app;
use crate::controller::KeyEvent;
use crate::hotkey::{MOD_ALT, MOD_CTRL, MOD_SHIFT, MOD_WIN};
use crate::keys;

thread_local! {
    static KEYBOARD: Cell<HHOOK> = const { Cell::new(null_mut()) };
    static MOUSE: Cell<HHOOK> = const { Cell::new(null_mut()) };
    static PROCESS_INJECTED: Cell<bool> = const { Cell::new(false) };
    static SUSPENDED: Cell<bool> = const { Cell::new(false) };
    // Modifier-only shortcut: modifiers in this chord; dirty = another key/click happened or it already fired
    static CHORD_MODS: Cell<u32> = const { Cell::new(0) };
    static CHORD_DIRTY: Cell<bool> = const { Cell::new(false) };
}

fn down(vk: u16) -> bool {
    (unsafe { GetAsyncKeyState(vk as i32) } as u16 & 0x8000) != 0
}

/// Modifiers held right now (before the event being processed)
pub fn current_mods() -> u32 {
    let mut m = 0;
    if down(VK_CONTROL) {
        m |= MOD_CTRL;
    }
    if down(VK_MENU) {
        m |= MOD_ALT;
    }
    if down(VK_SHIFT) {
        m |= MOD_SHIFT;
    }
    if down(VK_LWIN) || down(VK_RWIN) {
        m |= MOD_WIN;
    }
    m
}

fn mod_flag(vk: u16) -> u32 {
    if keys::is_control(vk) {
        MOD_CTRL
    } else if keys::is_alt(vk) {
        MOD_ALT
    } else if keys::is_shift(vk) {
        MOD_SHIFT
    } else if keys::is_win(vk) {
        MOD_WIN
    } else {
        0
    }
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let next = || unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) };
    if code < 0 || SUSPENDED.get() {
        return next();
    }
    let k = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
    // Our own injected keys always pass; other programs' only outside remote desktop mode
    if k.dwExtraInfo == INJECTED_MARKER
        || (k.flags & LLKHF_INJECTED != 0 && !PROCESS_INJECTED.get())
    {
        return next();
    }

    let vk = k.vkCode as u16;
    let msg = wparam as u32;
    if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
        let flag = mod_flag(vk);
        if flag != 0 {
            if CHORD_MODS.get() == 0 {
                CHORD_DIRTY.set(false);
            }
            CHORD_MODS.set(CHORD_MODS.get() | flag);
        } else {
            CHORD_DIRTY.set(true);
        }

        let caps = unsafe { GetKeyState(VK_CAPITAL as i32) } & 1 != 0;
        let mut e = KeyEvent::new(vk, current_mods(), caps);
        // try_with: the app may be busy (e.g. a modal dialog opened from inside a handler) → pass the key
        app::try_with(|c| c.on_key(&mut e));
        if e.handled {
            return 1;
        }
    } else if (msg == WM_KEYUP || msg == WM_SYSKEYUP) && mod_flag(vk) != 0 {
        if !CHORD_DIRTY.get() && CHORD_MODS.get() != 0 {
            CHORD_DIRTY.set(true); // fire once per chord
            let mods = CHORD_MODS.get();
            app::try_with(|c| c.on_modifiers_tapped(mods));
        }
        if current_mods() & !mod_flag(vk) == 0 {
            CHORD_MODS.set(0);
        }
    }
    next()
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0
        && matches!(
            wparam as u32,
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN
        )
    {
        CHORD_DIRTY.set(true); // Ctrl+click is not a Ctrl tap
        app::try_with(|c| c.on_mouse_clicked());
    }
    unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) }
}

/// Install on the UI thread (the hooks run through its message loop)
pub fn start() -> bool {
    unsafe {
        let module = GetModuleHandleW(std::ptr::null());
        KEYBOARD.set(SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_proc),
            module,
            0,
        ));
        MOUSE.set(SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0)); // optional: only resets the buffer
    }
    !KEYBOARD.get().is_null()
}

pub fn stop() {
    for hook in [KEYBOARD.replace(null_mut()), MOUSE.replace(null_mut())] {
        if !hook.is_null() {
            unsafe { UnhookWindowsHookEx(hook) };
        }
    }
}

/// Remote desktop mode: also handle keys injected by other programs
pub fn set_process_injected(on: bool) {
    PROCESS_INJECTED.set(on);
}

/// Pass everything through (while recording a shortcut)
pub fn set_suspended(on: bool) {
    SUSPENDED.set(on);
}
