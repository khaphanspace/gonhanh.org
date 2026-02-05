//! Regression tests for the key consumption (double-typing) fix
//!
//! These tests verify the `consumed` flag is set when the engine transforms input.
//! Without this flag, the OS types the literal key AFTER the engine's output.
//!
//! Issue: typing "tiens" would produce "tiếngs" (literal 's' appears twice)
//!
//! Note: Basic tone/mark functionality is tested in keyboard_layout_test.rs
//! This file focuses ONLY on FLAG_KEY_CONSUMED behavior.

mod common;

use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine, Result, FLAG_KEY_CONSUMED};

/// Check if result has consumed flag set
fn is_consumed(result: &Result) -> bool {
    result.flags & FLAG_KEY_CONSUMED != 0
}

/// Type a string into the engine
fn type_keys(engine: &mut Engine, s: &str) {
    for c in s.chars() {
        let key = gonhanh_core::utils::char_to_key(c.to_ascii_lowercase());
        if key != 255 {
            engine.on_key(key, c.is_uppercase(), false);
        }
    }
}

// REGRESSION TESTS - These MUST FAIL without the consumption fix

#[test]
fn test_telex_tone_sets_consumed_flag() {
    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_enabled(true);

    type_keys(&mut engine, "ba");
    let result = engine.on_key(keys::S, false, false); // sắc tone

    assert!(
        result.action != Action::None as u8,
        "Tone should produce output"
    );
    assert!(
        is_consumed(&result),
        "Tone transformation must set FLAG_KEY_CONSUMED"
    );
}

#[test]
fn test_telex_mark_sets_consumed_flag() {
    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_enabled(true);

    type_keys(&mut engine, "a");
    let result = engine.on_key(keys::A, false, false); // aa → â

    assert!(
        result.action != Action::None as u8,
        "Mark should produce output"
    );
    assert!(
        is_consumed(&result),
        "Mark transformation must set FLAG_KEY_CONSUMED"
    );
}

#[test]
fn test_telex_stroke_sets_consumed_flag() {
    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_enabled(true);

    type_keys(&mut engine, "d");
    let result = engine.on_key(keys::D, false, false); // dd → đ

    assert!(
        result.action != Action::None as u8,
        "Stroke should produce output"
    );
    assert!(
        is_consumed(&result),
        "Stroke transformation must set FLAG_KEY_CONSUMED"
    );
}

#[test]
fn test_vni_tone_sets_consumed_flag() {
    let mut engine = Engine::new();
    engine.set_method(1); // VNI
    engine.set_enabled(true);

    type_keys(&mut engine, "a");
    let result = engine.on_key(keys::N1, false, false); // 1 = sắc

    assert!(
        result.action != Action::None as u8,
        "VNI tone should produce output"
    );
    assert!(is_consumed(&result), "VNI tone must set FLAG_KEY_CONSUMED");
}

#[test]
fn test_on_key_with_char_sets_consumed_flag() {
    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_enabled(true);

    engine.on_key_with_char(keys::B, false, false, false, Some('b'));
    engine.on_key_with_char(keys::A, false, false, false, Some('a'));
    let result = engine.on_key_with_char(keys::S, false, false, false, Some('s'));

    assert!(
        result.action != Action::None as u8,
        "Tone via char API should work"
    );
    assert!(
        is_consumed(&result),
        "on_key_with_char must also set FLAG_KEY_CONSUMED"
    );
}

/// This test verifies the exact bug scenario reported by user
#[test]
fn test_regression_double_typing_bug() {
    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_enabled(true);

    type_keys(&mut engine, "tieng");
    let result = engine.on_key(keys::S, false, false);

    assert!(
        is_consumed(&result),
        "REGRESSION: Without consumption, 's' would cause double-typing (tiếngs instead of tiếng)"
    );
}

#[test]
fn test_regression_all_telex_modifiers_consumed() {
    let modifiers = [
        (keys::S, "ba"), // sắc
        (keys::F, "ba"), // huyền
        (keys::R, "ba"), // hỏi
        (keys::X, "ba"), // ngã
        (keys::J, "ba"), // nặng
    ];

    for (key, prefix) in modifiers {
        let mut engine = Engine::new();
        engine.set_method(0);
        engine.set_enabled(true);

        type_keys(&mut engine, prefix);
        let result = engine.on_key(key, false, false);

        assert!(
            is_consumed(&result),
            "REGRESSION: Telex modifier key {:?} must be consumed",
            key
        );
    }
}

/// Test that empty output does NOT consume the key
/// Bug: "người" + 'w' → w consumed but output nothing, causing buffer desync
#[test]
fn test_empty_output_not_consumed() {
    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_enabled(true);

    // Type "nguoi" then w to get "ngươi", then w again (no valid target)
    type_keys(&mut engine, "nguoi");
    engine.on_key(keys::W, false, false); // → ngươi
    let result = engine.on_key(keys::W, false, false); // → should NOT consume

    // If action is Send but no output, key should NOT be consumed
    // so the OS types the literal 'w'
    assert!(
        !is_consumed(&result) || result.count > 0 || result.backspace > 0,
        "Empty output must NOT consume key - otherwise causes buffer desync"
    );
}
