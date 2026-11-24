//! Windows virtual keys and their mapping to the macOS keycodes used by the core

pub const BACK: u16 = 0x08;
pub const TAB: u16 = 0x09;
pub const RETURN: u16 = 0x0D;
pub const SHIFT: u16 = 0x10;
pub const CONTROL: u16 = 0x11;
pub const MENU: u16 = 0x12;
pub const CAPITAL: u16 = 0x14;
pub const ESCAPE: u16 = 0x1B;
pub const SPACE: u16 = 0x20;
pub const PRIOR: u16 = 0x21;
pub const NEXT: u16 = 0x22;
pub const END: u16 = 0x23;
pub const HOME: u16 = 0x24;
pub const LEFT: u16 = 0x25;
pub const UP: u16 = 0x26;
pub const RIGHT: u16 = 0x27;
pub const DOWN: u16 = 0x28;
pub const DELETE: u16 = 0x2E;
pub const LWIN: u16 = 0x5B;
pub const RWIN: u16 = 0x5C;
pub const LSHIFT: u16 = 0xA0;
pub const RSHIFT: u16 = 0xA1;
pub const LCONTROL: u16 = 0xA2;
pub const RCONTROL: u16 = 0xA3;
pub const LMENU: u16 = 0xA4;
pub const RMENU: u16 = 0xA5;
pub const OEM_1: u16 = 0xBA; // ;:
pub const OEM_PLUS: u16 = 0xBB; // =+
pub const OEM_COMMA: u16 = 0xBC; // ,<
pub const OEM_MINUS: u16 = 0xBD; // -_
pub const OEM_PERIOD: u16 = 0xBE; // .>
pub const OEM_2: u16 = 0xBF; // /?
pub const OEM_3: u16 = 0xC0; // `~
pub const OEM_4: u16 = 0xDB; // [{
pub const OEM_5: u16 = 0xDC; // \|
pub const OEM_6: u16 = 0xDD; // ]}
pub const OEM_7: u16 = 0xDE; // '"

pub const fn is_letter(vk: u16) -> bool {
    vk >= b'A' as u16 && vk <= b'Z' as u16
}

pub const fn is_number(vk: u16) -> bool {
    vk >= b'0' as u16 && vk <= b'9' as u16
}

/// Same table as Linux KeycodeMap.h / core/src/data/keys.rs. `None` keys never reach the engine.
pub fn to_mac_key(vk: u16) -> Option<u16> {
    const LETTERS: [u16; 26] = [
        0, 11, 8, 2, 14, 3, 5, 4, 34, 38, 40, 37, 46, 45, 31, 35, 12, 15, 1, 17, 32, 9, 13, 7, 16,
        6,
    ];
    const DIGITS: [u16; 10] = [29, 18, 19, 20, 21, 23, 22, 26, 28, 25];
    if is_letter(vk) {
        return Some(LETTERS[(vk - b'A' as u16) as usize]);
    }
    if is_number(vk) {
        return Some(DIGITS[(vk - b'0' as u16) as usize]);
    }
    Some(match vk {
        BACK => 51,
        SPACE => 49,
        RETURN => 36,
        TAB => 48,
        ESCAPE => 53,
        LEFT => 123,
        RIGHT => 124,
        DOWN => 125,
        UP => 126,
        OEM_PERIOD => 47,
        OEM_COMMA => 43,
        OEM_2 => 44,
        OEM_1 => 41,
        OEM_7 => 39,
        OEM_4 => 33,
        OEM_6 => 30,
        OEM_5 => 42,
        OEM_MINUS => 27,
        OEM_PLUS => 24,
        OEM_3 => 50,
        _ => return None,
    })
}

pub const fn is_navigation(vk: u16) -> bool {
    matches!(vk, LEFT | RIGHT | UP | DOWN | HOME | END | PRIOR | NEXT)
}

/// Word-ending keys other than Space (macOS isBreakKey): typed again after an engine replacement
pub const fn is_break_key(vk: u16, shift: bool) -> bool {
    if is_number(vk) {
        return shift; // Shift+digit = a symbol (!@#)
    }
    matches!(
        vk,
        TAB | RETURN
            | ESCAPE
            | LEFT
            | RIGHT
            | UP
            | DOWN
            | OEM_PERIOD
            | OEM_COMMA
            | OEM_2
            | OEM_1
            | OEM_7
            | OEM_4
            | OEM_6
            | OEM_5
            | OEM_MINUS
            | OEM_PLUS
            | OEM_3
    )
}

/// Keys the restore shortcut lets through when nothing was restored (Issue #157)
pub const fn is_special_passthrough(vk: u16) -> bool {
    matches!(vk, ESCAPE | TAB | RETURN | DELETE) || is_navigation(vk)
}

pub const fn is_shift(vk: u16) -> bool {
    matches!(vk, SHIFT | LSHIFT | RSHIFT)
}
pub const fn is_control(vk: u16) -> bool {
    matches!(vk, CONTROL | LCONTROL | RCONTROL)
}
pub const fn is_alt(vk: u16) -> bool {
    matches!(vk, MENU | LMENU | RMENU)
}
pub const fn is_win(vk: u16) -> bool {
    matches!(vk, LWIN | RWIN)
}
pub const fn is_modifier(vk: u16) -> bool {
    is_shift(vk) || is_control(vk) || is_alt(vk) || is_win(vk)
}
