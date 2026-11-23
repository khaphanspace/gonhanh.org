//! Types through InputController + the real core engine and checks what ends up on "screen"

use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::hotkey::Hotkey;
use crate::settings::{self, Method, ShortcutItem};

/// Script: plain chars are keys; '⇧' = next key with Shift, '^' = next key with Ctrl; ⎋ ← ⏎ ⌫ special keys
fn key_of(c: char) -> (u16, bool) {
    match c {
        '.' => (keys::OEM_PERIOD, false),
        ',' => (keys::OEM_COMMA, false),
        '-' => (keys::OEM_MINUS, false),
        ']' => (keys::OEM_6, false),
        ' ' => (keys::SPACE, false),
        '⎋' => (keys::ESCAPE, false),
        '←' => (keys::LEFT, false),
        '⏎' => (keys::RETURN, false),
        '⌫' => (keys::BACK, false),
        c => (c.to_ascii_uppercase() as u16, c.is_ascii_uppercase()),
    }
}

/// What the app would show for a key that passes through
fn printed(vk: u16, shift: bool) -> String {
    if keys::is_letter(vk) {
        let c = vk as u8 as char;
        return if shift { c } else { c.to_ascii_lowercase() }.to_string();
    }
    if keys::is_number(vk) {
        return if shift && vk == b'1' as u16 { "!".into() } else { (vk as u8 as char).to_string() };
    }
    match vk {
        keys::OEM_PERIOD => if shift { ">" } else { "." },
        keys::OEM_COMMA => ",",
        keys::OEM_MINUS => "-",
        keys::OEM_6 => "]",
        keys::SPACE => " ",
        keys::RETURN => "\n",
        keys::ESCAPE => "⎋", // visible so tests can see ESC reaching the app
        _ => "",
    }
    .into()
}

fn type_with(script: &str, setup: impl FnOnce(&mut Settings), non_latin: bool) -> String {
    let screen = Rc::new(RefCell::new(String::new()));
    let out = screen.clone();
    let mut s = Settings::default();
    setup(&mut s);
    let send: SendFn = Box::new(move |text, bs, trailing| {
        let mut sc = out.borrow_mut();
        for _ in 0..bs {
            sc.pop();
        }
        sc.push_str(text);
        if trailing != 0 {
            sc.push_str(&printed(trailing, false));
        }
    });
    let mut c = InputController::new(s, send);
    c.persist = false;
    c.is_non_latin_layout = Box::new(move || non_latin);

    let (mut shift_next, mut ctrl_next) = (false, false);
    for ch in script.chars() {
        match ch {
            '⇧' => shift_next = true,
            '^' => ctrl_next = true,
            _ => {
                let (vk, sh) = key_of(ch);
                let sh = sh || shift_next;
                let mods = if sh { MOD_SHIFT } else { 0 } | if ctrl_next { MOD_CTRL } else { 0 };
                shift_next = false;
                ctrl_next = false;
                let mut e = KeyEvent::new(vk, mods, false);
                c.on_key(&mut e);
                if e.handled || mods & MOD_CTRL != 0 {
                    continue;
                }
                if vk == keys::BACK {
                    screen.borrow_mut().pop();
                } else {
                    screen.borrow_mut().push_str(&printed(vk, sh));
                }
            }
        }
    }
    let result = screen.borrow().clone();
    result
}

fn type_keys(script: &str) -> String {
    type_with(script, |_| {}, false)
}

fn hcm(s: &mut Settings) {
    s.shortcuts[2].enabled = true;
}
fn restore_on(s: &mut Settings) {
    s.restore_shortcut_enabled = true;
}

#[test]
fn telex_basics() {
    assert_eq!(type_keys("tieengs"), "tiếng");
    assert_eq!(type_keys("Vieetj"), "Việt");
    assert_eq!(type_keys("tieen⌫⌫eengs"), "tiếng");
    assert_eq!(type_keys("vieetj⏎"), "việt\n");
}

#[test]
fn english_auto_restore_on_word_end() {
    assert_eq!(type_keys("text "), "text ");
    assert_eq!(type_keys("text."), "text.");
    assert_eq!(type_keys("text⏎"), "text\n");
}

#[test]
fn buffer_resets() {
    assert_eq!(type_keys("ba.s"), "ba.s"); // punctuation
    assert_eq!(type_keys("ba,s"), "ba,s");
    assert_eq!(type_keys("ba←s"), "bas"); // arrow
    assert_eq!(type_keys("ba^as"), "bas"); // Ctrl+key
}

#[test]
fn text_shortcuts() {
    assert_eq!(type_with("hcm ", hcm, false), "Hồ Chí Minh ");
    assert_eq!(type_with("hcm.", hcm, false), "Hồ Chí Minh.");
    assert_eq!(type_with("-⇧.", |s| s.shortcuts.push(ShortcutItem::new("->", "→", true)), false), "→");
    // shortcuts keep working with Vietnamese off (Issue #129)
    assert_eq!(type_with("hcm ", |s| { s.enabled = false; hcm(s) }, false), "Hồ Chí Minh ");
}

#[test]
fn options() {
    assert_eq!(type_with("ok. ban", |s| s.auto_capitalize = true, false), "ok. Ban");
    assert_eq!(type_keys("w"), "ư");
    assert_eq!(type_with("w", |s| s.auto_w_shortcut = false, false), "w");
    assert_eq!(type_with("]", |s| s.bracket_shortcut = true, false), "ư");
    assert_eq!(type_keys("]"), "]");
    assert_eq!(type_keys("hoaf"), "hoà");
    assert_eq!(type_with("hoaf", |s| s.modern_tone = false, false), "hòa");
    assert_eq!(type_keys("zas"), "zas");
    assert_eq!(type_with("vieetj ", |s| s.enabled = false, false), "vieetj ");
}

#[test]
fn vni() {
    let vni = |s: &mut Settings| s.method = Method::Vni;
    assert_eq!(type_with("vie65t", vni, false), "việt");
    assert_eq!(type_with("a⇧1", vni, false), "a!");
}

#[test]
fn esc_restore() {
    assert_eq!(type_with("vieetj⎋", restore_on, false), "vieetj"); // restored, ESC swallowed
    assert_eq!(type_with("ban⎋", restore_on, false), "ban⎋"); // nothing to restore, ESC passes
    assert_eq!(type_with("vieetj ⎋", restore_on, false), "việt ⎋");
    let custom = |s: &mut Settings| {
        s.restore_shortcut_enabled = true;
        s.restore_shortcut = Hotkey { vk: b'Z' as u16, mods: MOD_CTRL };
    };
    assert_eq!(type_with("vieetj⎋", custom, false), "việt⎋");
    assert_eq!(type_keys("vieetj⎋"), "việt⎋");
}

#[test]
fn non_latin_keyboard_passes_through() {
    assert_eq!(type_with("aa", |_| {}, true), "aa");
}

#[test]
fn hotkeys() {
    assert!(Hotkey::DEFAULT_TOGGLE.matches(keys::SPACE, MOD_CTRL));
    assert!(Hotkey { vk: 0, mods: MOD_CTRL | MOD_SHIFT }.matches_modifier_only(MOD_CTRL | MOD_SHIFT));
    assert_eq!(Hotkey::from_int(Hotkey::DEFAULT_SECONDARY_TOGGLE.to_int()).to_string(), "Ctrl + Shift + Space");
}

#[test]
fn toggle_hotkey_switches_off_and_blocks_key() {
    let mut c = InputController::new(Settings::default(), Box::new(|_, _, _| {}));
    c.persist = false;
    let mut e = KeyEvent::new(keys::SPACE, MOD_CTRL, false);
    c.on_key(&mut e);
    assert!(e.handled);
    assert!(!c.settings.enabled);
}

#[test]
fn shortcut_files() {
    let mut imported = Vec::new();
    settings::import_shortcuts(&settings::export_shortcuts(&[ShortcutItem::new("dc", "dòng 1\ndòng 2", true)]), &mut imported);
    assert_eq!(imported, vec![ShortcutItem::new("dc", "dòng 1\ndòng 2", true)]);
    assert_eq!(settings::unescape(&settings::escape("a\\b\tc\r\nd")), "a\\b\tc\r\nd");
    assert_eq!(settings::normalize_app(" Code "), "Code.exe");
    let s = Settings { auto_capitalize_excluded_apps: "code.exe, WindowsTerminal".into(), ..Settings::default() };
    assert!(s.is_auto_capitalize_excluded("windowsterminal.exe"));
}

#[test]
fn per_app_profile_overrides_and_restores() {
    let mut s = Settings::default();
    s.per_app_profiles.insert("game.exe".into(), -1);
    let mut c = InputController::new(s, Box::new(|_, _, _| {}));
    c.persist = false;
    c.on_app_changed("Game.exe");
    assert!(!c.settings.enabled);
    c.on_app_changed("notepad.exe");
    assert!(c.settings.enabled);
}
