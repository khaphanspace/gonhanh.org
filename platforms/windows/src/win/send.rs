//! Text output through SendInput

use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

use crate::keys;

/// Marks our own injected keys so the hook skips them ("GNH ")
pub const INJECTED_MARKER: usize = 0x474E_4820;

fn key(vk: u16, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: INJECTED_MARKER,
            },
        },
    }
}

fn press(out: &mut Vec<INPUT>, vk: u16, extra: KEYBD_EVENT_FLAGS) {
    out.push(key(vk, 0, extra));
    out.push(key(vk, 0, extra | KEYEVENTF_KEYUP));
}

/// Delete `backspaces` chars, type `text`, then press `trailing_vk` (0 = none) — one SendInput batch,
/// so the order is kept (Shift held by the user still applies to the trailing key)
pub fn send_text(text: &str, backspaces: usize, trailing_vk: u16) {
    let mut inputs = Vec::with_capacity(backspaces * 2 + text.len() * 2 + 2);
    for _ in 0..backspaces {
        press(&mut inputs, VK_BACK, 0);
    }
    let mut prev = '\0';
    for c in text.chars() {
        // Issue #343: newline in a shortcut replacement = Enter key (CRLF sends one Enter)
        if c == '\n' || c == '\r' {
            if !(c == '\n' && prev == '\r') {
                press(&mut inputs, VK_RETURN, 0);
            }
            prev = c;
            continue;
        }
        for unit in c.encode_utf16(&mut [0; 2]) {
            inputs.push(key(0, *unit, KEYEVENTF_UNICODE));
            inputs.push(key(0, *unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
        }
        prev = c;
    }
    if trailing_vk != 0 {
        press(
            &mut inputs,
            trailing_vk,
            if keys::is_navigation(trailing_vk) {
                KEYEVENTF_EXTENDEDKEY
            } else {
                0
            },
        );
    }
    if !inputs.is_empty() {
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            )
        };
    }
}
