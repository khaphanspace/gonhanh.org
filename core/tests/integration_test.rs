//! Integration Tests - Engine state, settings, method switching

mod common;
use common::{assert_action, assert_passthrough, telex, type_word};
use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine};

// ============================================================
// ENGINE STATE: Enable/Disable
// ============================================================

#[test]
fn disabled_engine_passes_through() {
    let mut e = Engine::new();
    e.set_enabled(false);

    assert_passthrough(&mut e, keys::A);
    assert_passthrough(&mut e, keys::S);
}

#[test]
fn re_enable_engine_works() {
    let mut e = Engine::new();
    e.set_enabled(false);
    e.set_enabled(true);

    let result = type_word(&mut e, "as");
    assert_eq!(result, "á");
}

// ============================================================
// CTRL/CMD: Modifier key handling
// ============================================================

#[test]
fn ctrl_key_passes_through() {
    let mut e = Engine::new();

    // Ctrl+A should pass through
    assert_action(&mut e, keys::A, false, true, Action::None);

    // Buffer cleared, 's' alone passes through
    assert_passthrough(&mut e, keys::S);
}

#[test]
fn ctrl_clears_buffer() {
    let mut e = Engine::new();

    e.on_key(keys::A, false, false);
    e.on_key(keys::C, false, true); // Ctrl+C clears buffer
    assert_passthrough(&mut e, keys::S);
}

// ============================================================
// METHOD SWITCHING: Telex <-> VNI
// ============================================================

#[test]
fn method_switch_preserves_buffer() {
    let mut e = Engine::new();

    e.on_key(keys::A, false, false);
    e.set_method(1); // Switch to VNI

    // VNI tone '1' works on previous 'a'
    let r = e.on_key(keys::N1, false, false);
    assert_eq!(r.action, Action::Send as u8);
}

#[test]
fn invalid_method_defaults_to_telex() {
    let mut e = Engine::new();
    e.set_method(99); // Invalid

    let result = type_word(&mut e, "as");
    assert_eq!(result, "á");
}

#[test]
fn switch_telex_to_vni_and_back() {
    let mut e = Engine::new();

    // Telex mode
    let result = type_word(&mut e, "as");
    assert_eq!(result, "á");

    // Switch to VNI
    e.set_method(1);
    e.clear();
    let result = type_word(&mut e, "a1");
    assert_eq!(result, "á");

    // Back to Telex
    e.set_method(0);
    e.clear();
    let result = type_word(&mut e, "af");
    assert_eq!(result, "à");
}

// ============================================================
// UNKNOWN KEYS
// ============================================================

#[test]
fn unknown_key_passes_through() {
    let mut e = Engine::new();
    assert_passthrough(&mut e, 255);
}

#[test]
fn space_clears_buffer() {
    let mut e = Engine::new();

    e.on_key(keys::A, false, false);
    e.on_key(keys::SPACE, false, false);
    assert_passthrough(&mut e, keys::S);
}

// ============================================================
// EMPTY BUFFER: Tone/mark without vowel
// ============================================================

#[test]
fn tone_without_vowel_passes_through() {
    let mut e = Engine::new();

    assert_passthrough(&mut e, keys::S);
    assert_passthrough(&mut e, keys::F);
}

#[test]
fn telex_w_as_vowel_standalone() {
    // In Telex mode, standalone "w" → "ư" (valid Vietnamese)
    let mut e = Engine::new();
    let result = e.on_key(keys::W, false, false);
    assert_eq!(result.action, 1); // Action::Send
    assert_eq!(result.count, 1);
    assert_eq!(result.chars[0], 'ư' as u32);
}

#[test]
fn telex_w_as_vowel_after_valid_consonant() {
    // "nhw" → "như" (valid: nh + ư)
    let mut e = Engine::new();
    e.on_key(keys::N, false, false);
    e.on_key(keys::H, false, false);
    let result = e.on_key(keys::W, false, false);
    assert_eq!(result.action, 1);
    assert_eq!(result.chars[0], 'ư' as u32);
}

#[test]
fn telex_w_passthrough_after_invalid_consonant() {
    // "kw" → "kw" (invalid: k cannot precede ư)
    let mut e = Engine::new();
    e.on_key(keys::K, false, false);
    let result = e.on_key(keys::W, false, false);
    assert_eq!(result.action, 0); // passthrough
}

#[test]
fn telex_ww_reverts() {
    // "ww" → revert to "w" (shortcut skipped)
    // User typing pattern: w→ư, ww→w, www→ww
    let mut e = Engine::new();

    // First w → ư
    let result = e.on_key(keys::W, false, false);
    assert_eq!(result.action, 1);
    assert_eq!(result.chars[0], 'ư' as u32);

    // Second w → revert to "w" (single w, shortcut skipped)
    let result = e.on_key(keys::W, false, false);
    assert_eq!(result.action, 1);
    assert_eq!(result.backspace, 1); // delete "ư"
    assert_eq!(result.count, 1); // output "w"
    assert_eq!(result.chars[0], 'w' as u32);

    // Third w → just adds w (shortcut was skipped, not retried)
    let result = e.on_key(keys::W, false, false);
    assert_eq!(result.action, 0); // Pass through (normal letter)
}

#[test]
fn vni_w_passes_through() {
    // In VNI mode, "w" should pass through
    let mut e = Engine::new();
    e.set_method(1); // VNI
    assert_passthrough(&mut e, keys::W);
}

#[test]
fn word_boundary_shortcut_vn() {
    // Default shortcuts are currently disabled
    // This test verifies that "vn" + space just passes through
    let mut e = Engine::new();

    // Type "vn"
    e.on_key(keys::V, false, false);
    e.on_key(keys::N, false, false);

    // Space - no shortcut expansion (shortcuts disabled)
    let result = e.on_key(keys::SPACE, false, false);
    assert_eq!(result.action, 0); // Action::None (passthrough)
}

#[test]
fn vni_tone_without_vowel_passes_through() {
    let mut e = Engine::new();
    e.set_method(1);
    assert_passthrough(&mut e, keys::N1);
}

// ============================================================
// BACKSPACE HANDLING
// ============================================================

#[test]
fn backspace_on_empty_buffer() {
    let mut e = Engine::new();
    assert_passthrough(&mut e, keys::DELETE);
}

#[test]
fn multiple_backspace_clears_all() {
    let mut e = Engine::new();

    e.on_key(keys::A, false, false);
    e.on_key(keys::B, false, false);
    e.on_key(keys::DELETE, false, false);
    e.on_key(keys::DELETE, false, false);

    assert_passthrough(&mut e, keys::S);
}

// ============================================================
// CONSONANT-ONLY WORDS
// ============================================================

#[test]
fn consonant_only_no_conversion() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "bcd");
    assert_eq!(result, "bcd");
}

#[test]
fn tone_after_consonant_only() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "bcs");
    assert_eq!(result, "bcs");
}

// ============================================================
// CLEAR BUFFER
// ============================================================

#[test]
fn clear_resets_state() {
    let mut e = Engine::new();

    e.on_key(keys::A, false, false);
    e.clear();
    assert_passthrough(&mut e, keys::S);
}

// ============================================================
// ORTHOGRAPHY: Modern style (hoà, not hòa)
// ============================================================

#[test]
fn modern_orthography_hoa() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "hoaf");
    assert_eq!(result, "hoà"); // Modern: tone on last vowel
}

const MODERN_ORTHO_CASES: &[(&str, &str)] = &[
    ("hoaf", "hoà"),
    ("hoas", "hoá"),
    ("hoar", "hoả"),
    ("hoax", "hoã"),
    ("hoaj", "hoạ"),
];

#[test]
fn modern_orthography_full() {
    for (input, expected) in MODERN_ORTHO_CASES {
        let mut e = Engine::new();
        let result = type_word(&mut e, input);
        assert_eq!(result, *expected, "Modern: {} → {}", input, result);
    }
}

// ============================================================
// REVERT BEHAVIOR
// ============================================================

#[test]
fn double_tone_reverts() {
    telex(&[("ass", "as")]);
}

#[test]
fn double_mark_reverts() {
    telex(&[("aaa", "aa")]);
}

#[test]
fn triple_same_key_behavior() {
    let mut e = Engine::new();
    // a → a, aa → â, aaa → aa, aaaa → aâ
    let result = type_word(&mut e, "aaaa");
    assert_eq!(result, "aâ");
}

// ============================================================
// STRESS TEST: Long input sequences
// ============================================================

#[test]
fn long_input_sequence() {
    let mut e = Engine::new();

    // Type a very long Vietnamese sentence
    let input = "vieetj nam laf mootj quoocs gia ddepj vaf giauf truyeenf thoongs vawn hoas";
    let expected = "việt nam là một quốc gia đẹp và giàu truyền thống văn hoá";

    let result = type_word(&mut e, input);
    assert_eq!(result, expected);
}

#[test]
fn repeated_typing_sessions() {
    let mut e = Engine::new();

    // Multiple words, clearing between
    let result1 = type_word(&mut e, "xin");
    e.clear();
    let result2 = type_word(&mut e, "chaof");

    assert_eq!(result1, "xin");
    assert_eq!(result2, "chào");
}

// ============================================================
// EDGE: Special sequences
// ============================================================

#[test]
fn only_modifiers() {
    let mut e = Engine::new();

    // Only modifier keys, no vowels
    let result = type_word(&mut e, "sssss");
    assert_eq!(result, "sssss");
}

#[test]
fn alternating_vowel_modifier() {
    let mut e = Engine::new();

    // a→á→as→á→as... (alternating)
    let result = type_word(&mut e, "asasas");
    // as→á, as→á, as→á = áá (actually depends on engine buffer)
    // Each pair resets: a+s=á, then new a+s=á, etc
    // But with single engine instance, buffer accumulates
    assert!(!result.is_empty());
}

// ============================================================
// FOREIGN WORDS: Should NOT transform
// ============================================================

#[test]
fn foreign_word_claudeco_not_transformed() {
    let mut e = Engine::new();
    // "claudeco" has invalid initial "cl" → stroke should NOT apply
    let result = type_word(&mut e, "claudecod");
    // Should remain as normal "d", not "đ"
    assert!(
        !result.contains('đ'),
        "claudeco+d should not become đ, got: {}",
        result
    );
}

#[test]
fn foreign_word_no_tone() {
    let mut e = Engine::new();
    // "expect" is invalid → tone modifiers should not apply
    let result = type_word(&mut e, "expects");
    assert!(
        !result.contains('é'),
        "expect+s should not add tone, got: {}",
        result
    );
}

#[test]
fn foreign_word_exp_no_circumflex() {
    let mut e = Engine::new();
    // "exp" is invalid → circumflex should not apply when typing 'e'
    let result = type_word(&mut e, "expe");
    assert!(
        !result.contains('ê'),
        "exp+e should not become eêp, got: {}",
        result
    );
}

#[test]
fn foreign_word_exxpe_no_transform() {
    let mut e = Engine::new();
    // When typing "exxpe":
    // - 'e' → buffer="e"
    // - 'x' → mark applied → screen="ẽ"
    // - 'x' → revert (same key) → screen="ex", buffer="ex"
    // - 'p' → screen="exp", buffer="exp" (invalid Vietnamese)
    // - 'e' → buffer="expe" invalid → no circumflex applied, just adds 'e'
    // Result: "expe" (the first x was consumed/reverted)
    let result = type_word(&mut e, "exxpe");
    assert_eq!(
        result, "expe",
        "exxpe should become expe (first x consumed by mark/revert), got: {}",
        result
    );
}

// ============================================================
// VNI: SHIFT+NUMBER PASSTHROUGH (for symbols like @, #, $)
// ============================================================

#[test]
fn vni_shift_2_passes_through_for_at_symbol() {
    // VNI: "hi" + Shift+2 should NOT apply huyền mark
    // User wants to type "hi@", not "hì"
    let mut e = Engine::new();
    e.set_method(1); // VNI

    // Type "hi"
    e.on_key(keys::H, false, false);
    e.on_key(keys::I, false, false);

    // Shift+2 (for @) - should pass through, not apply mark
    let r = e.on_key_ext(keys::N2, true, false, true); // caps=true, ctrl=false, shift=true
    assert_eq!(
        r.action,
        Action::None as u8,
        "Shift+2 should pass through in VNI"
    );
}

#[test]
fn vni_shift_numbers_all_pass_through() {
    // All Shift+number combinations should pass through in VNI
    let mut e = Engine::new();
    e.set_method(1); // VNI

    // Type a vowel first
    e.on_key(keys::A, false, false);

    // All number keys with shift should pass through
    let number_keys = [
        keys::N1,
        keys::N2,
        keys::N3,
        keys::N4,
        keys::N5,
        keys::N6,
        keys::N7,
        keys::N8,
        keys::N9,
        keys::N0,
    ];

    for &key in &number_keys {
        let r = e.on_key_ext(key, true, false, true); // shift=true
        assert_eq!(
            r.action,
            Action::None as u8,
            "Shift+{} should pass through in VNI",
            key
        );
    }
}

#[test]
fn vni_without_shift_still_applies_marks() {
    // VNI: Without shift, number keys should still apply marks
    let mut e = Engine::new();
    e.set_method(1); // VNI

    // Type "a" + "2" (no shift) = à
    e.on_key(keys::A, false, false);
    let r = e.on_key_ext(keys::N2, false, false, false); // shift=false

    assert_eq!(
        r.action,
        Action::Send as u8,
        "VNI mark should apply without shift"
    );
    assert_eq!(r.chars[0], 'à' as u32, "a2 should produce à");
}

#[test]
fn telex_shift_not_affected() {
    // Telex mode should not be affected by the shift parameter
    // (Telex doesn't use number keys for marks)
    let mut e = Engine::new();
    e.set_method(0); // Telex

    // Type "a" + Shift+2 - should just pass through (2 is not a Telex modifier)
    e.on_key(keys::A, false, false);
    let r = e.on_key_ext(keys::N2, true, false, true);
    assert_eq!(
        r.action,
        Action::None as u8,
        "Telex should ignore number keys"
    );
}

// ============================================================
// SHORTCUT TESTS
// ============================================================

use gonhanh_core::engine::shortcut::Shortcut;

#[test]
fn shortcut_triggers_on_space() {
    let mut e = Engine::new();

    // Add shortcut: "vn" → "Việt Nam"
    e.shortcuts_mut().add(Shortcut::new("vn", "Việt Nam"));

    // Type "vn" + space
    let result = type_word(&mut e, "vn ");
    assert_eq!(
        result, "Việt Nam ",
        "vn + space should expand to 'Việt Nam '"
    );
}

#[test]
fn shortcut_tphcm_expands() {
    let mut e = Engine::new();

    // Add shortcut: "tphcm" → "Thành phố Hồ Chí Minh"
    e.shortcuts_mut()
        .add(Shortcut::new("tphcm", "Thành phố Hồ Chí Minh"));

    // Type "tphcm" + space
    let result = type_word(&mut e, "tphcm ");
    assert_eq!(
        result, "Thành phố Hồ Chí Minh ",
        "tphcm + space should expand"
    );
}

#[test]
fn shortcut_tphcm_raw_result() {
    let mut e = Engine::new();

    // Add shortcut
    e.shortcuts_mut()
        .add(Shortcut::new("tphcm", "Thành phố Hồ Chí Minh"));

    // Type "tphcm"
    for key in [keys::T, keys::P, keys::H, keys::C, keys::M] {
        e.on_key(key, false, false);
    }

    // Press space - should trigger shortcut
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, Action::Send as u8, "action should be Send");
    assert_eq!(r.backspace, 5, "should backspace 5 chars (tphcm)");

    // Collect output
    let output: String = r.chars[..r.count as usize]
        .iter()
        .filter_map(|&c| char::from_u32(c))
        .collect();

    assert_eq!(output, "Thành phố Hồ Chí Minh ", "output should match");
    assert_eq!(r.count, 22, "count should be 22 chars");
}

#[test]
fn shortcut_does_not_trigger_without_space() {
    let mut e = Engine::new();

    // Add shortcut: "vn" → "Việt Nam"
    e.shortcuts_mut().add(Shortcut::new("vn", "Việt Nam"));

    // Type "vn" without space - should remain "vn"
    let result = type_word(&mut e, "vn");
    assert_eq!(result, "vn", "vn without space should not expand");
}

#[test]
fn shortcut_multiple_shortcuts() {
    let mut e = Engine::new();

    // Add multiple shortcuts
    e.shortcuts_mut().add(Shortcut::new("hn", "Hà Nội"));
    e.shortcuts_mut().add(Shortcut::new("sg", "Sài Gòn"));

    // Test first shortcut
    let result1 = type_word(&mut e, "hn ");
    assert_eq!(result1, "Hà Nội ");

    e.clear();

    // Test second shortcut
    let result2 = type_word(&mut e, "sg ");
    assert_eq!(result2, "Sài Gòn ");
}

#[test]
fn shortcut_case_sensitive_no_match() {
    let mut e = Engine::new();

    // Add lowercase shortcut "vn"
    e.shortcuts_mut().add(Shortcut::new("vn", "Việt Nam"));

    // Typing uppercase "VN" does NOT match lowercase "vn" (case-sensitive)
    let result = type_word(&mut e, "VN ");
    assert_eq!(result, "VN ", "VN should NOT match lowercase 'vn' shortcut");
}

#[test]
fn shortcut_case_sensitive_exact_match() {
    let mut e = Engine::new();

    // Add uppercase shortcut "VN"
    e.shortcuts_mut().add(Shortcut::new("VN", "Việt Nam"));

    // Typing "VN" matches exactly
    let result = type_word(&mut e, "VN ");
    assert_eq!(result, "Việt Nam ", "VN should match 'VN' shortcut exactly");

    e.clear();

    // Typing "vn" does NOT match uppercase "VN"
    let result = type_word(&mut e, "vn ");
    assert_eq!(result, "vn ", "vn should NOT match uppercase 'VN' shortcut");
}

#[test]
fn shortcut_no_partial_match() {
    let mut e = Engine::new();

    // Add shortcut: "vietnam" → "Việt Nam"
    e.shortcuts_mut().add(Shortcut::new("vietnam", "Việt Nam"));

    // Type "vn" + space should NOT match "vietnam"
    let result = type_word(&mut e, "vn ");
    assert_eq!(result, "vn ", "partial match should not trigger shortcut");
}

#[test]
fn shortcut_removed_does_not_trigger() {
    let mut e = Engine::new();

    // Add and then remove shortcut
    e.shortcuts_mut().add(Shortcut::new("vn", "Việt Nam"));
    e.shortcuts_mut().remove("vn");

    // Type "vn" + space - should not expand
    let result = type_word(&mut e, "vn ");
    assert_eq!(result, "vn ", "removed shortcut should not trigger");
}

#[test]
fn shortcut_clear_all() {
    let mut e = Engine::new();

    // Add shortcuts
    e.shortcuts_mut().add(Shortcut::new("vn", "Việt Nam"));
    e.shortcuts_mut().add(Shortcut::new("hn", "Hà Nội"));

    // Clear all
    e.shortcuts_mut().clear();

    // Neither should expand
    let result1 = type_word(&mut e, "vn ");
    assert_eq!(result1, "vn ");

    e.clear();

    let result2 = type_word(&mut e, "hn ");
    assert_eq!(result2, "hn ");
}

#[test]
fn shortcut_with_vietnamese_output() {
    let mut e = Engine::new();

    // Shortcut with complex Vietnamese output
    e.shortcuts_mut().add(Shortcut::new("kb", "không biết"));

    let result = type_word(&mut e, "kb ");
    assert_eq!(result, "không biết ", "shortcut with Vietnamese output");
}

#[test]
fn shortcut_vni_mode() {
    // Shortcuts should work in VNI mode too
    let mut e = Engine::new();
    e.set_method(1); // VNI

    e.shortcuts_mut().add(Shortcut::new("vn", "Việt Nam"));

    let result = type_word(&mut e, "vn ");
    assert_eq!(result, "Việt Nam ", "shortcut should work in VNI mode");
}

#[test]
fn shortcut_only_triggers_on_space_not_punctuation() {
    let mut e = Engine::new();

    e.shortcuts_mut().add(Shortcut::new("vn", "Việt Nam"));

    // Type "vn" + period - should NOT trigger shortcut
    // Just type "vn" then clear buffer on period
    e.on_key(keys::V, false, false);
    e.on_key(keys::N, false, false);
    let r = e.on_key(keys::DOT, false, false);
    assert_eq!(
        r.action,
        Action::None as u8,
        "period should not trigger shortcut"
    );
}

#[test]
fn shortcut_not_triggered_by_comma() {
    let mut e = Engine::new();

    e.shortcuts_mut().add(Shortcut::new("vn", "Việt Nam"));

    // Type "vn" + comma - should NOT trigger shortcut
    e.on_key(keys::V, false, false);
    e.on_key(keys::N, false, false);
    let r = e.on_key(keys::COMMA, false, false);
    assert_eq!(
        r.action,
        Action::None as u8,
        "comma should not trigger shortcut"
    );
}

#[test]
fn shortcut_not_triggered_by_letter() {
    let mut e = Engine::new();

    e.shortcuts_mut().add(Shortcut::new("vn", "Việt Nam"));

    // Type "vn" + "a" - should NOT trigger shortcut, just add "a"
    e.on_key(keys::V, false, false);
    e.on_key(keys::N, false, false);
    let r = e.on_key(keys::A, false, false);
    // "a" is a normal letter, should pass through (not a shortcut trigger)
    assert_eq!(
        r.action,
        Action::None as u8,
        "letter should not trigger shortcut"
    );
}
