//! Test shortcuts work when IME is disabled (English mode)

use gonhanh_core::data::keys;
use gonhanh_core::engine::{shortcut::Shortcut, Engine};

#[test]
fn test_word_shortcut_when_disabled_space() {
    let mut e = Engine::new();
    e.set_enabled(false); // Disable IME (English mode)

    // Add a test shortcut: "btw" → "by the way"
    e.shortcuts_mut().add(Shortcut::new("btw", "by the way"));

    // Type "btw"
    e.on_key(keys::B, false, false);
    e.on_key(keys::T, false, false);
    e.on_key(keys::W, false, false);

    // Press Space to trigger shortcut
    let r = e.on_key(keys::SPACE, false, false);

    println!(
        "Space result: action={}, bs={}, count={}",
        r.action, r.backspace, r.count
    );

    assert_eq!(r.action, 1, "Space should trigger shortcut when disabled");
    assert_eq!(r.backspace, 3, "Should backspace 3 chars (btw)");

    let output: String = (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect();
    assert_eq!(output, "by the way ", "Should output 'by the way '");
}

#[test]
fn test_word_shortcut_when_disabled_enter() {
    let mut e = Engine::new();
    e.set_enabled(false);

    e.shortcuts_mut().add(Shortcut::new("btw", "by the way"));

    e.on_key(keys::B, false, false);
    e.on_key(keys::T, false, false);
    e.on_key(keys::W, false, false);

    // Press Enter to trigger shortcut
    let r = e.on_key(keys::RETURN, false, false);

    println!(
        "Enter result: action={}, bs={}, count={}",
        r.action, r.backspace, r.count
    );

    assert_eq!(r.action, 1, "Enter should trigger shortcut when disabled");
    assert_eq!(r.backspace, 3, "Should backspace 3 chars");

    let output: String = (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect();
    assert_eq!(
        output, "by the way",
        "Should output 'by the way' (no space for Enter)"
    );
}

#[test]
fn test_symbol_shortcut_when_disabled() {
    let mut e = Engine::new();
    e.set_enabled(false);

    e.shortcuts_mut().add(Shortcut::immediate("->", "→"));

    // Type "->"
    e.on_key(keys::MINUS, false, false);
    let r = e.on_key_ext(keys::DOT, false, false, true); // Shift+. = >

    println!(
        "Symbol shortcut result: action={}, bs={}",
        r.action, r.backspace
    );

    assert_eq!(r.action, 1, "Symbol shortcut should work when disabled");
}

// Issue #161: Shortcuts with numbers should work

#[test]
fn test_shortcut_with_number_f1_telex() {
    let mut e = Engine::new();
    e.shortcuts_mut().add(Shortcut::new("f1", "formula one"));

    e.on_key(keys::F, false, false);
    e.on_key(keys::N1, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, 1, "f1 shortcut should trigger in Telex mode");
    let output: String = (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect();
    assert_eq!(output, "formula one ", "Should output 'formula one '");
}

#[test]
fn test_shortcut_with_number_f1_vni() {
    let mut e = Engine::new();
    e.set_method(1); // VNI
    e.shortcuts_mut().add(Shortcut::new("f1", "formula one"));

    e.on_key(keys::F, false, false);
    e.on_key(keys::N1, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, 1, "f1 shortcut should trigger in VNI mode");
    let output: String = (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect();
    assert_eq!(output, "formula one ", "Should output 'formula one '");
}

#[test]
fn test_shortcut_with_number_a1() {
    let mut e = Engine::new();
    e.shortcuts_mut().add(Shortcut::new("a1", "alpha one"));

    e.on_key(keys::A, false, false);
    e.on_key(keys::N1, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, 1, "a1 shortcut should trigger");
    let output: String = (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect();
    assert_eq!(output, "alpha one ", "Should output 'alpha one '");
}

#[test]
fn test_shortcut_with_number_disabled_mode() {
    let mut e = Engine::new();
    e.set_enabled(false);
    e.shortcuts_mut().add(Shortcut::new("f1", "formula one"));

    e.on_key(keys::F, false, false);
    e.on_key(keys::N1, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, 1, "f1 shortcut should trigger when disabled");
    let output: String = (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect();
    assert_eq!(output, "formula one ", "Should output 'formula one '");
}
