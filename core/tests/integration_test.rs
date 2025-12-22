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

#[test]
fn disabled_engine_ignores_auto_restore() {
    // When engine is disabled (e.g., macOS input source is Telex),
    // auto-restore should NOT work - all keys should pass through
    let mut e = Engine::new();
    e.set_english_auto_restore(true);
    e.set_enabled(false);

    // Even with auto-restore enabled, disabled engine returns None
    for key in [keys::T, keys::O, keys::T, keys::O, keys::SPACE] {
        let r = e.on_key(key, false, false);
        assert_eq!(
            r.action,
            Action::None as u8,
            "Disabled engine should pass through"
        );
    }
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
fn telex_uppercase_w_as_vowel_standalone() {
    // Uppercase "W" → "Ư" (should respect caps)
    let mut e = Engine::new();
    let result = e.on_key(keys::W, true, false); // caps=true for uppercase
    assert_eq!(result.action, 1); // Action::Send
    assert_eq!(result.count, 1);
    assert_eq!(result.chars[0], 'Ư' as u32);
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
fn double_mark_key_includes_both() {
    // When mark is reverted by pressing same key twice, only the reverting key appears
    // This is standard Vietnamese IME behavior (UniKey, ibus-unikey, etc.)
    // First key was modifier, second key reverts and outputs one letter
    // English words like "issue", "bass" work via auto-restore feature
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
    // - 'x' → revert (same key) → screen="ex", buffer="ex" (only reverting key appears)
    // - 'p' → screen="exp", buffer="exp"
    // - 'e' → buffer="expe" invalid → no circumflex applied, just adds 'e'
    // Result: "expe" (standard IME behavior: first x was modifier, second x reverts)
    let result = type_word(&mut e, "exxpe");
    assert_eq!(
        result, "expe",
        "exxpe should become expe (standard IME revert behavior), got: {}",
        result
    );
}

// Issue #15: "metric" should not become "mẻtic"
// When buffer has final consonant AND no existing diacritics,
// modifier keys that would create unparsed consonants should be treated as letters
#[test]
fn foreign_word_metric_no_mark() {
    let mut e = Engine::new();
    // "met" + r: buffer [M,E,T] is valid, but 'r' after final T looks like English
    // Should NOT apply hỏi mark to 'e'
    let result = type_word(&mut e, "metric");
    assert!(
        !result.contains('ẻ'),
        "metric should not become mẻtic, got: {}",
        result
    );
    assert_eq!(result, "metric", "metric should stay as metric");
}

#[test]
fn foreign_word_string_no_mark() {
    let mut e = Engine::new();
    // "string" - 'str' is invalid initial, should not apply any marks
    let result = type_word(&mut e, "string");
    assert!(
        !result.contains('ỉ'),
        "string should not have diacritics, got: {}",
        result
    );
}

/// Auto-restore now handles "express" - the "xp" pattern (x followed by consonant) is detected.
/// NOTE: Requires english_auto_restore to be enabled (experimental feature).
#[test]
fn foreign_word_express_no_mark() {
    let mut e = Engine::new();
    e.set_english_auto_restore(true); // Enable experimental feature
                                      // "express" - 'r' after 'p' should not apply mark
    let result = type_word(&mut e, "express");
    assert!(
        !result.contains('ẻ'),
        "express should not have diacritics, got: {}",
        result
    );
}

// ============================================================
// FOREIGN WORDS: Should NOT get Vietnamese diacritics
// These tests verify that common English/foreign words are not transformed
// The validation algorithm should detect invalid Vietnamese patterns
// ============================================================

// --- Words with invalid vowel patterns (not in Vietnamese) ---

#[test]
fn foreign_word_your_no_mark() {
    let mut e = Engine::new();
    // "yo" pattern doesn't exist in Vietnamese
    let result = type_word(&mut e, "your");
    assert_eq!(result, "your", "your should stay unchanged");
}

#[test]
fn foreign_word_you_no_mark() {
    let mut e = Engine::new();
    // "yo" pattern doesn't exist in Vietnamese
    let result = type_word(&mut e, "yous");
    assert_eq!(result, "yous", "yous should stay unchanged");
}

#[test]
fn foreign_word_about_no_mark() {
    let mut e = Engine::new();
    // "ou" pattern doesn't exist in Vietnamese
    let result = type_word(&mut e, "abouts");
    assert_eq!(result, "abouts", "abouts should stay unchanged");
}

#[test]
fn foreign_word_house_no_mark() {
    let mut e = Engine::new();
    // "ou" pattern doesn't exist in Vietnamese
    let result = type_word(&mut e, "houses");
    assert_eq!(result, "houses", "houses should stay unchanged");
}

#[test]
fn foreign_word_south_no_mark() {
    let mut e = Engine::new();
    // "ou" pattern doesn't exist in Vietnamese
    let result = type_word(&mut e, "souths");
    assert_eq!(result, "souths", "souths should stay unchanged");
}

#[test]
fn foreign_word_could_no_mark() {
    let mut e = Engine::new();
    // "ou" pattern doesn't exist in Vietnamese
    let result = type_word(&mut e, "coulds");
    assert_eq!(result, "coulds", "coulds should stay unchanged");
}

#[test]
fn foreign_word_would_no_mark() {
    let mut e = Engine::new();
    e.set_english_auto_restore(true); // Enable experimental feature
                                      // "ou" pattern doesn't exist in Vietnamese
    let result = type_word(&mut e, "woulds");
    assert_eq!(result, "woulds", "woulds should stay unchanged");
}

// --- Words with invalid consonant continuations (T+R, P+R, C+R) ---

#[test]
fn foreign_word_control_no_mark() {
    let mut e = Engine::new();
    // t+r pattern common in foreign words
    let result = type_word(&mut e, "control");
    assert_eq!(result, "control", "control should stay unchanged");
}

#[test]
fn foreign_word_matrix_no_mark() {
    let mut e = Engine::new();
    // t+r pattern
    let result = type_word(&mut e, "matrix");
    assert_eq!(result, "matrix", "matrix should stay unchanged");
}

#[test]
fn foreign_word_central_no_mark() {
    let mut e = Engine::new();
    // t+r pattern
    let result = type_word(&mut e, "central");
    assert_eq!(result, "central", "central should stay unchanged");
}

#[test]
fn foreign_word_spectrum_no_mark() {
    let mut e = Engine::new();
    // c+r pattern (spec-trum)
    let result = type_word(&mut e, "spectrum");
    assert_eq!(result, "spectrum", "spectrum should stay unchanged");
}

#[test]
fn foreign_word_describe_no_mark() {
    let mut e = Engine::new();
    // c+r pattern
    let result = type_word(&mut e, "describe");
    assert_eq!(result, "describe", "describe should stay unchanged");
}

#[test]
fn foreign_word_compress_no_mark() {
    let mut e = Engine::new();
    // p+r pattern
    let result = type_word(&mut e, "compress");
    assert_eq!(result, "compress", "compress should stay unchanged");
}

#[test]
fn foreign_word_supreme_no_mark() {
    let mut e = Engine::new();
    // p+r pattern
    let result = type_word(&mut e, "supreme");
    assert_eq!(result, "supreme", "supreme should stay unchanged");
}

// Vietnamese words with diacritics should still work correctly
#[test]
fn vietnamese_duoc_with_mark() {
    let mut e = Engine::new();
    // "dduwowcj" → "được" - standard typing for "được"
    // dd=đ, uwow=ươ, c=c, j=nặng
    let result = type_word(&mut e, "dduwowcj");
    assert_eq!(result, "được", "dduwowcj should become được");
}

#[test]
fn vietnamese_viet_with_mark_after_final() {
    let mut e = Engine::new();
    // "vieets" → "viết" - user types circumflex first, then mark after final
    // ee=ê (commits to Vietnamese), t=final, s=sắc mark (ế)
    let result = type_word(&mut e, "vieets");
    assert_eq!(result, "viết", "vieets should become viết");
}

#[test]
fn vietnamese_an_with_mark_after_final() {
    let mut e = Engine::new();
    // "anf" → "àn" - mark after final consonant
    // Note: This should work because buffer has no final consonant blocking the 'f'
    // (the check only blocks consonant modifier keys when they can't extend the final)
    let result = type_word(&mut e, "anf");
    assert_eq!(result, "àn", "anf should become àn");
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
fn shortcut_not_triggered_by_diacriticed_char() {
    // Bug fix: shortcut "a" should NOT match "ạ" (a with nặng mark)
    let mut e = Engine::new();

    // Add shortcut: "a" → "anh"
    e.shortcuts_mut().add(Shortcut::new("a", "anh"));

    // Type "aj" (produces "ạ" in Telex) + space - should NOT trigger shortcut
    let result = type_word(&mut e, "aj ");
    assert_eq!(result, "ạ ", "ạ should NOT match shortcut 'a'");

    e.clear();

    // Verify "a" + space still works correctly
    let result2 = type_word(&mut e, "a ");
    assert_eq!(result2, "anh ", "plain 'a' should match shortcut");
}

#[test]
fn shortcut_not_triggered_by_tone_marked_vowel() {
    // Shortcut should match exact string, not base characters
    let mut e = Engine::new();

    // Add shortcut: "duoc" → "được"
    e.shortcuts_mut().add(Shortcut::new("duoc", "được"));

    // Type "duwowcj" (produces "dược" in Telex) + space - should NOT trigger
    // because buffer contains "dược" (with ơ horn and nặng mark) not "duoc"
    let result = type_word(&mut e, "duwowcj ");
    assert_eq!(
        result, "dược ",
        "diacriticed 'dược' should NOT match shortcut 'duoc'"
    );

    e.clear();

    // Plain "duoc" + space SHOULD trigger
    let result2 = type_word(&mut e, "duoc ");
    assert_eq!(result2, "được ", "plain 'duoc' should match shortcut");
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

// Issue: "search" should not become "seảch" in Telex
// "ea" is not a valid Vietnamese vowel combination
#[test]
fn foreign_word_search_no_mark() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "search");
    assert_eq!(
        result, "search",
        "search should stay unchanged, got: {}",
        result
    );
}

// Test other English patterns that might be problematic
#[test]
fn foreign_word_teacher_no_mark() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "teacher");
    // "ea" is invalid Vietnamese pattern
    assert_eq!(
        result, "teacher",
        "teacher should stay unchanged, got: {}",
        result
    );
}

#[test]
fn foreign_word_real_no_mark() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "real");
    // "ea" is invalid Vietnamese pattern
    assert_eq!(
        result, "real",
        "real should stay unchanged, got: {}",
        result
    );
}

#[test]
fn foreign_word_beach_no_mark() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "beach");
    // "ea" is invalid Vietnamese pattern
    assert_eq!(
        result, "beach",
        "beach should stay unchanged, got: {}",
        result
    );
}

// Test that common English words with 'x' stay unchanged
// The "consonant + e + x" pattern is detected as English (tex-, nex-, etc.)
// NOTE: Requires english_auto_restore to be enabled (experimental feature).
#[test]
fn foreign_word_text_no_mark() {
    let mut e = Engine::new();
    e.set_english_auto_restore(true); // Enable experimental feature
    let text_result = type_word(&mut e, "text");

    // "text" has consonant 't' before 'e', so Check 4 catches the "tex" pattern
    println!("'text' -> '{}' (expected: 'text')", text_result);

    assert_eq!(
        text_result, "text",
        "text should stay unchanged (consonant + e + x detected as English)"
    );
}

/// Auto-restore now handles "expect" - the "xp" pattern (x followed by consonant) is detected.
/// NOTE: Requires english_auto_restore to be enabled (experimental feature).
#[test]
fn foreign_word_expect_no_mark() {
    let mut e = Engine::new();
    e.set_english_auto_restore(true); // Enable experimental feature
    let expect_result = type_word(&mut e, "expect");

    // "expect" starts with 'e', no consonant before - can't distinguish from Vietnamese "ẽ"
    println!("'expect' -> '{}' (expected: 'expect')", expect_result);

    assert_eq!(
        expect_result, "expect",
        "expect should stay unchanged (e+x detected as English)"
    );
}
/// Bug: Shortcut should NOT trigger when preceded by numbers
/// e.g., "149k" should NOT expand "k" → "không"
#[test]
fn shortcut_not_triggered_after_numbers() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.shortcuts_mut().add(Shortcut::new("k", "không"));

    // Type "149k" + SPACE - should NOT trigger shortcut
    e.on_key(keys::N1, false, false);
    e.on_key(keys::N4, false, false);
    e.on_key(keys::N9, false, false);
    e.on_key(keys::K, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    // Should NOT trigger shortcut
    assert_eq!(
        r.action,
        Action::None as u8,
        "shortcut 'k' should NOT trigger after '149'"
    );
}

/// Bug: Shortcut should NOT trigger after single number
/// e.g., "1k" should NOT expand "k" → "không"
#[test]
fn shortcut_not_triggered_after_single_number() {
    let mut e = Engine::new();
    e.set_method(0);
    e.shortcuts_mut().add(Shortcut::new("a", "anh"));

    // Type "1a" + SPACE
    e.on_key(keys::N1, false, false);
    e.on_key(keys::A, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(
        r.action,
        Action::None as u8,
        "shortcut 'a' should NOT trigger after '1'"
    );
}

/// Bug: Shortcut should NOT trigger after backspace into previous word
/// "đa" + SPACE + backspace×2 + "a" should NOT expand "a" → "anh"
#[test]
fn shortcut_not_triggered_after_backspace_into_previous_word() {
    let mut e = Engine::new();
    e.set_method(0);
    e.shortcuts_mut().add(Shortcut::new("a", "anh"));

    // Type "đa" (simulated as d+d+a for Telex)
    e.on_key(keys::D, false, false);
    e.on_key(keys::D, false, false);
    e.on_key(keys::A, false, false);

    // SPACE - clears buffer
    e.on_key(keys::SPACE, false, false);

    // Backspace twice - deleting into previous word territory
    e.on_key(keys::DELETE, false, false);
    e.on_key(keys::DELETE, false, false);

    // Type "a"
    e.on_key(keys::A, false, false);

    // SPACE - should NOT trigger shortcut
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(
        r.action,
        Action::None as u8,
        "shortcut 'a' should NOT trigger after backspace into previous word"
    );
}

/// Bug: Standalone shortcut should still work
/// "k" alone should expand to "không"
#[test]
fn shortcut_works_standalone() {
    let mut e = Engine::new();
    e.set_method(0);
    e.shortcuts_mut().add(Shortcut::new("k", "không"));

    // Type "k" + SPACE - should trigger
    e.on_key(keys::K, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, Action::Send as u8, "shortcut 'k' should trigger");
    let chars: String = (0..r.count as usize)
        .map(|i| char::from_u32(r.chars[i]).unwrap_or('?'))
        .collect();
    assert_eq!(chars, "không ");
}

/// Issue #23: Shortcut "zz" should work in Telex mode
/// Even though "z" is a remove modifier, when there's nothing to remove,
/// it should be added to buffer so shortcuts like "zz" can trigger.
#[test]
fn shortcut_zz_works_in_telex() {
    let mut e = Engine::new();
    e.set_method(0); // Telex

    e.shortcuts_mut().add(Shortcut::new("zz", "tiếp tục"));

    // Type "zz" + SPACE - should trigger shortcut
    e.on_key(keys::Z, false, false);
    e.on_key(keys::Z, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    // Should trigger shortcut with output "tiếp tục "
    assert_eq!(r.action, Action::Send as u8, "shortcut 'zz' should trigger");
    assert_eq!(r.backspace, 2, "should backspace 2 chars for 'zz'");

    // Verify output contains "tiếp tục "
    let chars: String = (0..r.count as usize)
        .map(|i| char::from_u32(r.chars[i]).unwrap_or('?'))
        .collect();
    assert_eq!(chars, "tiếp tục ", "output should be 'tiếp tục '");
}

/// Issue #23: Verify "z" still removes marks when there ARE marks to remove
#[test]
fn z_still_removes_marks_in_telex() {
    let mut e = Engine::new();
    e.set_method(0); // Telex

    // Type "as" to get "á"
    e.on_key(keys::A, false, false);
    let r = e.on_key(keys::S, false, false);
    assert_eq!(r.action, Action::Send as u8);

    // Type "z" to remove the mark - should work
    let r = e.on_key(keys::Z, false, false);
    assert_eq!(r.action, Action::Send as u8, "z should remove mark");

    // The result should be "a" (mark removed)
    let chars: String = (0..r.count as usize)
        .map(|i| char::from_u32(r.chars[i]).unwrap_or('?'))
        .collect();
    assert_eq!(chars, "a", "mark should be removed, resulting in 'a'");
}

/// Issue #24: All possible Telex combinations for "đọc"
///
/// đọc = đ (stroke) + ọ (nặng mark) + c
/// - đ: can be typed as "dd" (adjacent) OR delayed stroke (d + vowel + d)
/// - ọ: requires "o" + "j" (j can come after c)
/// - c: just "c"
///
/// Delayed stroke is now supported for valid Vietnamese patterns.
/// Second 'd' triggers stroke on initial 'd' if buffer forms valid Vietnamese.
#[test]
fn telex_doc_all_combinations() {
    // Standard patterns - dd at start (adjacent stroke)
    telex(&[
        ("ddojc", "đọc"), // dd + oj + c (most common)
        ("ddocj", "đọc"), // dd + oc + j (mark at end)
    ]);

    // Delayed stroke patterns - d at end triggers stroke on initial d
    // Second 'd' applies stroke to first 'd' when buffer is valid Vietnamese
    telex(&[
        ("dojcd", "đọc"), // d + oj + c + d (delayed stroke at end)
        ("docjd", "đọc"), // d + oc + j + d (delayed stroke at end)
        ("docdj", "đọc"), // d + oc + d + j (delayed stroke, then tone)
    ]);

    // Mixed order patterns - delayed stroke in middle
    telex(&[
        ("dojdc", "đọc"), // d + oj + d + c (delayed stroke, then c)
        ("dodjc", "đọc"), // d + o + d + j + c (delayed stroke triggers on 3rd char)
    ]);
}

/// Issue #24: Edge cases for "đọc" - invalid or unexpected patterns
#[test]
fn telex_doc_edge_cases() {
    // These patterns might not produce "đọc" - test actual behavior
    let mut e = Engine::new();
    e.set_method(0); // Telex

    // Pattern: ojcdd - typing vowel+consonant first, then both d's at end
    // This should NOT work because dd needs to transform an existing d
    let result = type_word(&mut e, "ojcdd");
    // First d after ojc starts new word context, second d makes đ
    // Result depends on engine behavior
    assert_ne!(result, "đọc", "ojcdd should not produce đọc");

    e.clear();

    // Pattern: jdocd - j before vowel (invalid)
    let result2 = type_word(&mut e, "jdocd");
    assert_ne!(result2, "đọc", "jdocd should not produce đọc");
}

/// Issue #24: Similar words to ensure no regression
#[test]
fn telex_similar_words_to_doc() {
    telex(&[
        // Words with đ (stroke)
        ("ddi", "đi"),
        ("ddaau", "đâu"),
        ("dduowcj", "được"),
        ("dduwowngf", "đường"),
        // Words with ọ (o + nặng, no circumflex)
        ("hojc", "học"), // học uses ọ not ộ
        ("tojp", "tọp"),
        ("lojm", "lọm"),
        ("sojt", "sọt"),
        // Words with ộ (ô + nặng = circumflex + nặng)
        ("toojt", "tột"), // tột uses ộ
        ("loojn", "lộn"), // lộn uses ộ
        ("coojt", "cột"), // cột uses ộ
        // Words with đ + ọ (o + nặng, no circumflex)
        ("ddojc", "đọc"), // đọc uses ọ
        // Words with đ + ộ (ô + nặng)
        ("ddooj", "độ"), // độ uses ộ (circumflex + nặng)
        // Words with đ + other marks on o
        ("ddor", "đỏ"),  // đ + ỏ (hỏi)
        ("ddos", "đó"),  // đ + ó (sắc)
        ("ddoof", "đồ"), // đ + ồ (circumflex + huyền)
        ("ddoox", "đỗ"), // đ + ỗ (circumflex + ngã)
        // Longer words with đọc pattern
        ("ddojcc", "đọcc"), // extra c - passthrough
    ]);
}

/// Issue #24: Verify "đọc" with uppercase variations
#[test]
fn telex_doc_uppercase() {
    telex(&[
        ("Ddojc", "Đọc"), // Capital Đ
        ("DDOJC", "ĐỌC"), // All caps
        ("DDojc", "Đọc"), // DD at start, rest lowercase
    ]);
}

/// Issue #24: "đọc" in context (with spaces)
#[test]
fn telex_doc_in_sentence() {
    telex(&[
        ("ddojc ", "đọc "), // with trailing space
    ]);

    // Multi-word test
    let mut e = Engine::new();
    e.set_method(0);

    let result = type_word(&mut e, "ddojc");
    assert_eq!(result, "đọc");

    e.clear(); // Word boundary

    let result2 = type_word(&mut e, "sachs");
    assert_eq!(result2, "sách");
}

// ============================================================
// SKIP W SHORTCUT: User preference for w→ư at word start
// ============================================================

/// When skip_w_shortcut is enabled, standalone "w" should NOT convert to "ư"
#[test]
fn skip_w_shortcut_standalone_w_stays_w() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.set_skip_w_shortcut(true);

    // Standalone "w" should pass through (not convert to ư)
    let r = e.on_key(keys::W, false, false);
    assert_eq!(r.action, Action::None as u8, "w should pass through");
}

/// When skip_w_shortcut is enabled, "hw" should still produce "hư"
/// (only word-start w is skipped, not w after consonants)
#[test]
fn skip_w_shortcut_hw_still_produces_hu() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.set_skip_w_shortcut(true);

    // "h" + "w" should produce "hư" (w after consonant is NOT skipped)
    e.on_key(keys::H, false, false);
    let r = e.on_key(keys::W, false, false);
    assert_eq!(r.action, Action::Send as u8, "hw should produce ư");
    assert_eq!(r.chars[0], 'ư' as u32);
}

/// When skip_w_shortcut is enabled, "nhw" should produce "như"
#[test]
fn skip_w_shortcut_nhw_produces_nhu() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.set_skip_w_shortcut(true);

    // "nh" + "w" should produce "như"
    e.on_key(keys::N, false, false);
    e.on_key(keys::H, false, false);
    let r = e.on_key(keys::W, false, false);
    assert_eq!(r.action, Action::Send as u8, "nhw should produce ư");
    assert_eq!(r.chars[0], 'ư' as u32);
}

/// When skip_w_shortcut is DISABLED (default), standalone "w" converts to "ư"
#[test]
fn skip_w_shortcut_disabled_w_converts() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.set_skip_w_shortcut(false); // Explicitly disabled (same as default)

    // Standalone "w" should convert to "ư"
    let r = e.on_key(keys::W, false, false);
    assert_eq!(r.action, Action::Send as u8, "w should convert to ư");
    assert_eq!(r.chars[0], 'ư' as u32);
}

/// skip_w_shortcut should NOT affect VNI mode
#[test]
fn skip_w_shortcut_vni_mode_unaffected() {
    let mut e = Engine::new();
    e.set_method(1); // VNI
    e.set_skip_w_shortcut(true);

    // In VNI, "w" always passes through (no w→ư shortcut in VNI)
    let r = e.on_key(keys::W, false, false);
    assert_eq!(r.action, Action::None as u8, "VNI: w should pass through");
}

/// Full word test: "như" with skip_w_shortcut enabled
#[test]
fn skip_w_shortcut_full_word_nhu() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.set_skip_w_shortcut(true);

    let result = type_word(&mut e, "nhw");
    assert_eq!(
        result, "như",
        "nhw should produce như even with skip enabled"
    );
}

/// Full word test: "tư" with skip_w_shortcut enabled
#[test]
fn skip_w_shortcut_full_word_tu() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.set_skip_w_shortcut(true);

    let result = type_word(&mut e, "tw");
    assert_eq!(result, "tư", "tw should produce tư even with skip enabled");
}

/// Full word test: "được" with skip_w_shortcut enabled
#[test]
fn skip_w_shortcut_full_word_duoc() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.set_skip_w_shortcut(true);

    // "dduwowcj" → "được"
    let result = type_word(&mut e, "dduwowcj");
    assert_eq!(result, "được", "dduwowcj should produce được");
}

/// Uppercase W with skip enabled
#[test]
fn skip_w_shortcut_uppercase_w_stays_w() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.set_skip_w_shortcut(true);

    // Uppercase "W" at word start should pass through
    let r = e.on_key(keys::W, true, false); // caps=true
    assert_eq!(r.action, Action::None as u8, "W should pass through");
}

/// Uppercase W after consonant still converts
#[test]
fn skip_w_shortcut_uppercase_hw_produces_hu() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.set_skip_w_shortcut(true);

    // "H" + "W" should produce "HƯ"
    e.on_key(keys::H, true, false); // H
    let r = e.on_key(keys::W, true, false); // W
    assert_eq!(r.action, Action::Send as u8, "HW should produce Ư");
    assert_eq!(r.chars[0], 'Ư' as u32);
}

/// Complex test: Multiple words with skip_w_shortcut
#[test]
fn skip_w_shortcut_multiple_words() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.set_skip_w_shortcut(true);

    // Word 1: "w" at start → stays "w"
    let r1 = e.on_key(keys::W, false, false);
    assert_eq!(r1.action, Action::None as u8);
    e.clear();

    // Word 2: "như" → converts "w" after "nh"
    let result2 = type_word(&mut e, "nhw");
    assert_eq!(result2, "như");
    e.clear();

    // Word 3: "uw" → "ư" (u at start, then w as horn)
    let result3 = type_word(&mut e, "uw");
    assert_eq!(result3, "ư");
}

// ============================================================
// BACKSPACE-AFTER-SPACE: Issue #32
// ============================================================

/// Basic: Add mark after space (du + SPACE + < + j → dụ)
#[test]
fn backspace_after_space_telex_add_mark() {
    let mut e = Engine::new();
    // "du " + backspace + "j" → "dụ"
    let result = type_word(&mut e, "du <j");
    assert_eq!(result, "dụ", "du + space + backspace + j should produce dụ");
}

/// Basic: Add mark after space (VNI mode)
#[test]
fn backspace_after_space_vni_add_mark() {
    let mut e = Engine::new();
    e.set_method(1); // VNI
                     // "du " + backspace + "5" → "dụ"
    let result = type_word(&mut e, "du <5");
    assert_eq!(
        result, "dụ",
        "VNI: du + space + backspace + 5 should produce dụ"
    );
}

/// Change existing mark after space (cháo + SPACE + < + f → chào)
#[test]
fn backspace_after_space_telex_change_mark() {
    let mut e = Engine::new();
    // "chaos" → "cháo", then space + backspace + "f" → "chào"
    let result = type_word(&mut e, "chaos <f");
    assert_eq!(
        result, "chào",
        "cháo + space + backspace + f should change to chào"
    );
}

/// Multiple backspaces to delete chars (doc + SPACE + << + j → dọ)
#[test]
fn backspace_after_space_multiple_backspace() {
    let mut e = Engine::new();
    // "doc " + backspace×2 + "j" → "dọ"
    let result = type_word(&mut e, "doc <<j");
    assert_eq!(
        result, "dọ",
        "doc + space + 2 backspaces + j should produce dọ"
    );
}

/// Second backspace is normal delete (not restore again)
#[test]
fn backspace_after_space_second_is_normal() {
    let mut e = Engine::new();
    // "du " + backspace (restore "du") + backspace (delete 'u') → "d"
    let result = type_word(&mut e, "du <<");
    assert_eq!(result, "d", "Second backspace should delete normally");
}

/// Break key clears history (punctuation)
#[test]
fn backspace_after_space_break_clears_history() {
    let mut e = Engine::new();
    // Type "du", space, comma (break key), then backspace
    // Comma clears history, so backspace should just delete comma
    type_word(&mut e, "du ");
    e.on_key(keys::COMMA, false, false); // Break key clears history
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r.action,
        Action::None as u8,
        "After break key, backspace is normal"
    );
}

/// Ctrl clears history
#[test]
fn backspace_after_space_ctrl_clears_history() {
    let mut e = Engine::new();
    type_word(&mut e, "du ");
    e.on_key(keys::C, false, true); // Ctrl+C clears history
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r.action,
        Action::None as u8,
        "After Ctrl, backspace is normal"
    );
}

/// History stores multiple words
#[test]
fn backspace_after_space_history_multiple_words() {
    let mut e = Engine::new();
    // Type "an " then "em " then backspace → restore "em", type "j" → "ẹm"
    let result = type_word(&mut e, "an em <j");
    assert_eq!(
        result, "an ẹm",
        "Should restore most recent word 'em' and add mark"
    );
}

/// Uppercase preserved after restore
#[test]
fn backspace_after_space_preserve_case() {
    let mut e = Engine::new();
    // "Du " + backspace + "j" → "Dụ"
    let result = type_word(&mut e, "Du <j");
    assert_eq!(result, "Dụ", "Uppercase should be preserved after restore");
}

/// Complex word with multiple transforms
#[test]
fn backspace_after_space_complex_word() {
    let mut e = Engine::new();
    // "vieejt " + backspace + "s" → changes ệ to ế
    let result = type_word(&mut e, "vieejt <s");
    assert_eq!(
        result, "viết",
        "Should be able to change mark on complex word"
    );
}

/// Add tone after space (aa + SPACE + < + s → ấ)
#[test]
fn backspace_after_space_add_tone() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "aa <s");
    assert_eq!(result, "ấ", "Should add mark to circumflex vowel");
}

/// Stroke word (đi + SPACE + < continues editing)
#[test]
fn backspace_after_space_stroke_word() {
    let mut e = Engine::new();
    // "ddi " + backspace + "s" → "đí"
    let result = type_word(&mut e, "ddi <s");
    assert_eq!(result, "đí", "Stroke should be preserved after restore");
}

// ============================================================
// BACKSPACE-AFTER-SPACE: Extended behaviors
// ============================================================

/// Multiple spaces: restore word only after ALL spaces deleted
#[test]
fn backspace_after_multiple_spaces() {
    let mut e = Engine::new();
    // "du" + space + space → spaces_after_commit = 2
    type_word(&mut e, "du ");
    e.on_key(keys::SPACE, false, false); // Second space

    // First backspace: delete one space but NOT restore yet
    let r1 = e.on_key(keys::DELETE, false, false);
    assert_eq!(r1.action, Action::Send as u8, "Should delete space");
    assert_eq!(r1.backspace, 1, "Delete one space");
    assert_eq!(r1.count, 0, "No chars to add");

    // Second backspace: delete last space AND restore word
    let r2 = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r2.action,
        Action::Send as u8,
        "Should delete space and restore"
    );
    assert_eq!(r2.backspace, 1, "Delete last space");
    // Word "du" is now restored to buffer for editing
}

/// Arrow key LEFT clears history
#[test]
fn backspace_after_space_left_arrow_clears() {
    let mut e = Engine::new();
    type_word(&mut e, "du ");
    e.on_key(keys::LEFT, false, false); // Arrow clears history
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r.action,
        Action::None as u8,
        "LEFT arrow should clear history"
    );
}

/// Arrow key RIGHT clears history
#[test]
fn backspace_after_space_right_arrow_clears() {
    let mut e = Engine::new();
    type_word(&mut e, "du ");
    e.on_key(keys::RIGHT, false, false);
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r.action,
        Action::None as u8,
        "RIGHT arrow should clear history"
    );
}

/// Tab key clears history
#[test]
fn backspace_after_space_tab_clears() {
    let mut e = Engine::new();
    type_word(&mut e, "du ");
    e.on_key(keys::TAB, false, false);
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(r.action, Action::None as u8, "TAB should clear history");
}

/// Enter key clears history
#[test]
fn backspace_after_space_enter_clears() {
    let mut e = Engine::new();
    type_word(&mut e, "du ");
    e.on_key(keys::RETURN, false, false);
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(r.action, Action::None as u8, "ENTER should clear history");
}

/// ESC key clears history
#[test]
fn backspace_after_space_esc_clears() {
    let mut e = Engine::new();
    type_word(&mut e, "du ");
    e.on_key(keys::ESC, false, false);
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(r.action, Action::None as u8, "ESC should clear history");
}

/// Dot punctuation clears history
#[test]
fn backspace_after_space_dot_clears() {
    let mut e = Engine::new();
    type_word(&mut e, "du ");
    e.on_key(keys::DOT, false, false);
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(r.action, Action::None as u8, "DOT should clear history");
}

/// Typing new word after space, backspace restores new word only
#[test]
fn backspace_after_space_new_word_typed() {
    let mut e = Engine::new();
    // "du " then type "an " then backspace → restore "an"
    let result = type_word(&mut e, "du an <s");
    assert_eq!(result, "du án", "Should restore most recent word 'an'");
}

/// Empty buffer on space doesn't push to history
#[test]
fn backspace_after_space_empty_buffer() {
    let mut e = Engine::new();
    // Just spaces, no actual word typed
    e.on_key(keys::SPACE, false, false);
    e.on_key(keys::SPACE, false, false);
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r.action,
        Action::None as u8,
        "Empty buffer should not be pushed to history"
    );
}

/// Long Vietnamese word restore
#[test]
fn backspace_after_space_long_word() {
    let mut e = Engine::new();
    // "nghiêng" + space + backspace + "s" → "nghiếng"
    let result = type_word(&mut e, "nghieeng <s");
    assert_eq!(result, "nghiếng", "Long word should be restored correctly");
}

/// Word with multiple vowels
#[test]
fn backspace_after_space_multiple_vowels() {
    let mut e = Engine::new();
    // "khuya" + space + backspace + "f" → "khuỳa" (tone on 'y' per Vietnamese phonology)
    let result = type_word(&mut e, "khuya <f");
    assert_eq!(
        result, "khuỳa",
        "Word with multiple vowels should restore correctly"
    );
}

/// Continue typing after restore without adding mark
#[test]
fn backspace_after_space_continue_typing() {
    let mut e = Engine::new();
    // "du " + backspace + "n" → "dun"
    let result = type_word(&mut e, "du <n");
    assert_eq!(result, "dun", "Should be able to add letters after restore");
}

/// Backspace all chars after restore
#[test]
fn backspace_after_space_delete_all() {
    let mut e = Engine::new();
    // "du " + backspace + delete both chars → ""
    let result = type_word(&mut e, "du <<<");
    assert_eq!(result, "", "Should delete all chars after restore");
}

/// Three words, restore only affects most recent
#[test]
fn backspace_after_space_three_words() {
    let mut e = Engine::new();
    // "toi di hoc" + space + backspace + "j" → "toi di học"
    let result = type_word(&mut e, "toi di hoc <j");
    assert_eq!(
        result, "toi di học",
        "Should only restore most recent word 'hoc'"
    );
}

/// Consecutive word commits and restores
#[test]
fn backspace_after_space_consecutive_restores() {
    let mut e = Engine::new();
    // "an " (commit) + backspace (restore) + space (commit again) + backspace (restore again)
    type_word(&mut e, "an ");
    let r1 = e.on_key(keys::DELETE, false, false); // Restore "an"
    assert_eq!(r1.action, Action::Send as u8, "First restore");
    e.on_key(keys::SPACE, false, false); // Commit "an" again
    let r2 = e.on_key(keys::DELETE, false, false); // Should restore "an" again
    assert_eq!(r2.action, Action::Send as u8, "Second restore should work");
}

/// Backspace after typing a number (numbers don't break buffer)
#[test]
fn backspace_after_space_with_number_in_word() {
    let mut e = Engine::new();
    // In Telex, typing "so1" - number is in buffer but not affected
    // space + backspace + type more
    let result = type_word(&mut e, "so1 <2");
    // "so1" + space + backspace restores "so1", then "2" is just typed
    // Since "so1" has a number, Vietnamese transforms might not apply
    assert!(
        result.contains("so"),
        "Word with number should be restorable"
    );
}

/// VNI: Multiple words with marks
#[test]
fn backspace_after_space_vni_multiple_words() {
    let mut e = Engine::new();
    e.set_method(1); // VNI
                     // VNI: 6=circumflex, 9=stroke(đ), 5=hỏi, 2=huyền
                     // "to6i d9i ho5c" + space + backspace + "2" → change hỏc → hòc
    let result = type_word(&mut e, "to6i d9i ho5c <2");
    assert_eq!(result, "tôi đi hòc", "VNI multi-word restore should work");
}

/// Uppercase in middle of word
#[test]
fn backspace_after_space_mixed_case() {
    let mut e = Engine::new();
    // "iPhone" like pattern - mixed case
    let result = type_word(&mut e, "vieEt <s");
    assert_eq!(result, "viết", "Mixed case should normalize on transform");
}

/// Single char word restore
#[test]
fn backspace_after_space_single_char() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "a <s");
    assert_eq!(result, "á", "Single char word should restore correctly");
}

/// Backspace immediately (no typing after restore)
#[test]
fn backspace_after_space_immediate() {
    let mut e = Engine::new();
    type_word(&mut e, "du ");
    let r = e.on_key(keys::DELETE, false, false);
    // Should restore "du" - action is Send with backspace=1
    assert_eq!(r.action, Action::Send as u8);
    assert_eq!(r.backspace, 1, "Should delete the space");
}

/// History capacity: ensure ring buffer works
#[test]
fn backspace_after_space_history_capacity() {
    let mut e = Engine::new();
    // Type 12 words (more than capacity of 10)
    for i in 0..12 {
        let word = format!("w{}", i);
        for c in word.chars() {
            let key = match c {
                'w' => keys::W,
                '0' => keys::N0,
                '1' => keys::N1,
                '2' => keys::N2,
                '3' => keys::N3,
                '4' => keys::N4,
                '5' => keys::N5,
                '6' => keys::N6,
                '7' => keys::N7,
                '8' => keys::N8,
                '9' => keys::N9,
                _ => continue,
            };
            e.on_key(key, false, false);
        }
        e.on_key(keys::SPACE, false, false);
    }
    // Backspace should restore most recent word (w11)
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r.action,
        Action::Send as u8,
        "Should restore from ring buffer"
    );
}

/// Quick typing pattern: word space word space backspace
#[test]
fn backspace_after_space_quick_typing() {
    let mut e = Engine::new();
    // Simulate fast typing: "toi " "di " then realize mistake
    let result = type_word(&mut e, "toi di <r");
    assert_eq!(result, "toi dỉ", "Quick typing then backspace should work");
}

/// Disable engine clears history
#[test]
fn backspace_after_space_disable_clears() {
    let mut e = Engine::new();
    type_word(&mut e, "du ");
    e.set_enabled(false);
    e.set_enabled(true);
    let r = e.on_key(keys::DELETE, false, false);
    // Disabling engine should have cleared history
    assert_eq!(r.action, Action::None as u8, "Disable should clear history");
}

/// EDGE CASE: Type word, space, new chars, delete all, delete space → restore
/// Scenario: "chà" + " " + "o" + backspace + backspace → should restore "chà"
#[test]
fn backspace_after_space_type_delete_restore() {
    let mut e = Engine::new();
    // Type "chà " (chaf + space)
    type_word(&mut e, "chaf ");
    // Type "o"
    e.on_key(keys::O, false, false);
    // Delete "o"
    e.on_key(keys::DELETE, false, false);
    // Buffer is now empty, but spaces_after_commit should still be 1
    // Delete space to restore "chà"
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(r.action, Action::Send as u8, "Should restore previous word");
    assert_eq!(r.backspace, 1, "Should delete the space");
}

/// EDGE CASE: Continue typing after restore
/// "chà" + " " + "o" + backspace + backspace + "o" → "chào"
#[test]
fn backspace_after_space_restore_then_type() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "chaf o<<o");
    assert_eq!(result, "chào", "Restore then continue typing should work");
}

/// EDGE CASE: Delete multiple chars before restore
/// "việt" + " " + "abc" + 3 backspaces + 1 backspace → restore "việt"
#[test]
fn backspace_after_space_delete_multi_chars() {
    let mut e = Engine::new();
    type_word(&mut e, "vieejt ");
    type_word(&mut e, "abc");
    // Delete "abc"
    e.on_key(keys::DELETE, false, false);
    e.on_key(keys::DELETE, false, false);
    e.on_key(keys::DELETE, false, false);
    // Now buffer is empty, delete space to restore
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r.action,
        Action::Send as u8,
        "Should restore after deleting all new chars"
    );
}

/// EDGE CASE: Type Vietnamese word after space, delete it, restore original
/// "tôi" + " " + "đi" + 2 backspaces + 1 backspace → restore "tôi"
#[test]
fn backspace_after_space_new_viet_word_then_delete() {
    let mut e = Engine::new();
    type_word(&mut e, "tooi ");
    type_word(&mut e, "ddi");
    // Delete "đi"
    e.on_key(keys::DELETE, false, false);
    e.on_key(keys::DELETE, false, false);
    // Delete space
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(r.action, Action::Send as u8);
}

/// EDGE CASE: Multiple cycles of type-delete-restore
/// "anh" + " " + "x" + backspace + backspace → restore → " " + "y" + backspace + backspace → restore
#[test]
fn backspace_after_space_multiple_restore_cycles() {
    let mut e = Engine::new();
    type_word(&mut e, "anh ");
    // Cycle 1: type x, delete x, restore
    e.on_key(keys::X, false, false);
    e.on_key(keys::DELETE, false, false);
    let r1 = e.on_key(keys::DELETE, false, false);
    assert_eq!(r1.action, Action::Send as u8, "First restore should work");

    // Type space again
    e.on_key(keys::SPACE, false, false);
    // Cycle 2: type y, delete y, restore
    e.on_key(keys::Y, false, false);
    e.on_key(keys::DELETE, false, false);
    let r2 = e.on_key(keys::DELETE, false, false);
    assert_eq!(r2.action, Action::Send as u8, "Second restore should work");
}

/// EDGE CASE: Restore then apply tone mark
/// "cha" + " " + "x" + backspace + backspace + "f" → "chà"
#[test]
fn backspace_after_space_restore_apply_tone() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "cha x<<f");
    assert_eq!(result, "chà", "Should apply tone after restore");
}

/// EDGE CASE: Restore then apply vowel mark
/// "cô" + " " + "x" + backspace + backspace + "s" → "cố"
#[test]
fn backspace_after_space_restore_apply_vowel_mark() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "coo x<<s");
    assert_eq!(result, "cố", "Should apply mark after restore");
}

/// EDGE CASE: Type number after space, delete, restore
/// "một" + " " + "123" + 3 backspaces + 1 backspace → restore
#[test]
fn backspace_after_space_type_numbers_delete() {
    let mut e = Engine::new();
    type_word(&mut e, "mootj ");
    e.on_key(keys::N1, false, false);
    e.on_key(keys::N2, false, false);
    e.on_key(keys::N3, false, false);
    // Delete numbers (numbers don't go to buffer, but we track them)
    e.on_key(keys::DELETE, false, false);
    e.on_key(keys::DELETE, false, false);
    e.on_key(keys::DELETE, false, false);
    // Delete space
    let r = e.on_key(keys::DELETE, false, false);
    // Note: numbers after space clear spaces_after_commit, so restore won't work
    assert_eq!(
        r.action,
        Action::None as u8,
        "Numbers break the restore chain"
    );
}

/// EDGE CASE: Very long sequence - type, partial delete, more type, delete all
/// "mình" + " " + "abc" + delete 1 + "def" + delete 5 + restore
#[test]
fn backspace_after_space_complex_edit_sequence() {
    let mut e = Engine::new();
    type_word(&mut e, "minhf ");
    // Type "abc"
    type_word(&mut e, "abc");
    // Delete 1 char (now "ab")
    e.on_key(keys::DELETE, false, false);
    // Type "def" (now "abdef")
    type_word(&mut e, "def");
    // Delete all 5 chars
    for _ in 0..5 {
        e.on_key(keys::DELETE, false, false);
    }
    // Buffer empty, delete space to restore
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r.action,
        Action::Send as u8,
        "Complex edit should still allow restore"
    );
}

/// EDGE CASE: Two spaces then partial backspace
/// "word" + " " + " " + backspace (one space) → should NOT restore yet
#[test]
fn backspace_after_space_two_spaces_partial() {
    let mut e = Engine::new();
    type_word(&mut e, "word ");
    e.on_key(keys::SPACE, false, false); // Second space
                                         // Delete one space
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(r.action, Action::Send as u8, "Should delete space");
    // Should NOT have restored yet - still one space remaining
    // spaces_after_commit should be 1 now
    // Delete second space to actually restore
    let r2 = e.on_key(keys::DELETE, false, false);
    assert_eq!(r2.action, Action::Send as u8, "Should restore now");
}

/// EDGE CASE: Backspace on empty buffer without history
#[test]
fn backspace_after_space_empty_no_history() {
    let mut e = Engine::new();
    // Just backspace without any prior typing
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r.action,
        Action::None as u8,
        "Empty buffer, no history = no action"
    );
}

/// EDGE CASE: Single char word restore then continue
/// "a" + " " + "x" + backspace + backspace + "s" → "á"
#[test]
fn backspace_after_space_single_char_restore_continue() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "a x<<s");
    assert_eq!(result, "á", "Single char restore then tone should work");
}

/// EDGE CASE: Word with đ, restore, continue
/// "đi" + " " + "x" + backspace + backspace + "s" → "đí"
#[test]
fn backspace_after_space_d_stroke_restore() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "ddi x<<s");
    assert_eq!(result, "đí", "đ word restore then tone should work");
}

/// EDGE CASE: VNI mode - type, delete, restore, continue
/// "chà" (VNI: cha2) + " " + "x" + backspace + backspace + "1" → "chá"
#[test]
fn backspace_after_space_vni_restore_continue() {
    let mut e = Engine::new();
    e.set_method(1); // VNI
    let result = type_word(&mut e, "cha2 x<<1");
    assert_eq!(result, "chá", "VNI restore then continue should work");
}

/// EDGE CASE: Rapid alternating - restore, type, restore, type
#[test]
fn backspace_after_space_rapid_alternating() {
    let mut e = Engine::new();
    type_word(&mut e, "mot ");
    // Restore
    e.on_key(keys::DELETE, false, false);
    // Type space (commit "mot" again)
    e.on_key(keys::SPACE, false, false);
    // Type something
    e.on_key(keys::X, false, false);
    // Delete x
    e.on_key(keys::DELETE, false, false);
    // Restore again
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r.action,
        Action::Send as u8,
        "Rapid alternating should work"
    );
}

/// EDGE CASE: Uppercase word restore then lowercase continue
/// "TÔI" + " " + "x" + backspace + backspace + type lowercase
#[test]
fn backspace_after_space_uppercase_then_lowercase() {
    let mut e = Engine::new();
    // Type "TÔI" (uppercase)
    e.on_key(keys::T, true, false);
    e.on_key(keys::O, true, false);
    e.on_key(keys::O, true, false);
    e.on_key(keys::I, true, false);
    e.on_key(keys::SPACE, false, false);
    // Type x
    e.on_key(keys::X, false, false);
    // Delete x and space
    e.on_key(keys::DELETE, false, false);
    let r = e.on_key(keys::DELETE, false, false);
    assert_eq!(
        r.action,
        Action::Send as u8,
        "Uppercase word restore should work"
    );
}

// ============================================================
// RESTORE_WORD: Buffer restoration from Vietnamese string
// ============================================================

/// Helper: simulate typing on existing screen text after restore_word
fn restore_and_type(e: &mut Engine, initial: &str, input: &str) -> String {
    use gonhanh_core::utils::char_to_key;

    // Start with screen showing initial word
    let mut screen: String = initial.into();

    // Restore buffer to match screen
    e.restore_word(initial);

    // Type each character and apply results to screen
    for c in input.chars() {
        let key = char_to_key(c);
        let is_caps = c.is_uppercase();

        let r = e.on_key(key, is_caps, false);
        if r.action == Action::Send as u8 {
            for _ in 0..r.backspace {
                screen.pop();
            }
            for i in 0..r.count as usize {
                if let Some(ch) = char::from_u32(r.chars[i]) {
                    screen.push(ch);
                }
            }
        } else {
            // Pass through if not handled
            screen.push(c);
        }
    }
    screen
}

/// Basic restore_word: restore simple Vietnamese word
#[test]
fn restore_word_simple() {
    let mut e = Engine::new();
    // Screen has "việt", restore buffer, then type 's' to change mark
    let result = restore_and_type(&mut e, "việt", "s");
    assert_eq!(result, "viết", "After restore, tone mark should work");
}

/// restore_word with multiple vowels
#[test]
fn restore_word_multiple_vowels() {
    let mut e = Engine::new();
    let result = restore_and_type(&mut e, "khuỳa", "s");
    assert_eq!(result, "khuýa", "Should change tone on restored word");
}

/// restore_word with đ (stroked consonant)
#[test]
fn restore_word_with_d_stroke() {
    let mut e = Engine::new();
    let result = restore_and_type(&mut e, "đi", "s");
    assert_eq!(result, "đí", "đ should be preserved, tone added");
}

/// restore_word with circumflex (ô, â, ê)
#[test]
fn restore_word_with_circumflex() {
    let mut e = Engine::new();
    let result = restore_and_type(&mut e, "cô", "s");
    assert_eq!(result, "cố", "Circumflex should be preserved");
}

/// restore_word with horn (ơ, ư)
#[test]
fn restore_word_with_horn() {
    let mut e = Engine::new();
    let result = restore_and_type(&mut e, "mưa", "f");
    assert_eq!(result, "mừa", "Horn should be preserved, tone added");
}

/// restore_word with uppercase
#[test]
fn restore_word_uppercase() {
    let mut e = Engine::new();
    let result = restore_and_type(&mut e, "Việt", "f");
    // Typing 'f' (huyền) should change ệ to ề
    assert_eq!(result, "Viềt", "Should change mark on uppercase word");
}

/// restore_word empty string
#[test]
fn restore_word_empty() {
    let mut e = Engine::new();
    e.restore_word("");
    // Type a new word from scratch
    let result = type_word(&mut e, "as");
    assert_eq!(result, "á", "Empty restore should allow fresh typing");
}

/// restore_word with plain ASCII word then add mark
#[test]
fn restore_word_then_add_mark() {
    let mut e = Engine::new();
    // Restore "ban", then type 's' to add sắc
    let result = restore_and_type(&mut e, "ban", "s");
    assert_eq!(result, "bán", "Should add mark to restored ASCII word");
}

/// restore_word full example: user typed "đường", wants to add/change tone
#[test]
fn restore_word_full_example() {
    let mut e = Engine::new();
    let result = restore_and_type(&mut e, "đường", "s");
    assert_eq!(result, "đướng", "Should replace huyền with sắc");
}

/// restore_word then type consonant (extends word)
#[test]
fn restore_word_then_extend() {
    let mut e = Engine::new();
    let result = restore_and_type(&mut e, "việt", "nam");
    // Vietnamese chars preserved when extending
    assert_eq!(
        result, "việtnam",
        "Should extend word preserving existing chars"
    );
}

/// Real scenario: "chào" + space + random chars + backspace to before 'o' + add mark
/// This simulates: user types "chào ", then more stuff, then backspaces back into word
#[test]
fn restore_word_chao_scenario() {
    let mut e = Engine::new();

    // Step 1: Type "chào " (word committed with space)
    let screen1 = type_word(&mut e, "chaof ");
    assert_eq!(screen1, "chào ", "Should type chào with space");

    // Step 2: Type more spaces and random chars
    let screen2 = type_word(&mut e, "  abc");
    assert_eq!(screen2, "  abc", "Should type spaces and random chars");

    // Full screen at this point: "chào   abc"

    // Step 3: Backspace to delete "abc", "  ", and "o" (7 backspaces)
    // After backspacing, screen shows "chà" (the 'o' is deleted)
    // At this point, native app should detect cursor in word and call restore_word

    // Step 4: Simulate native app calling restore_word with remaining text "chà"
    e.restore_word("chà");

    // Step 5: Type 's' to change huyền to sắc on 'à'
    let result = restore_and_type(&mut e, "chà", "s");
    assert_eq!(result, "chá", "Should change huyền to sắc after restore");
}

/// Another scenario: restore partial word and continue typing
#[test]
fn restore_word_partial_then_complete() {
    let mut e = Engine::new();

    // User had "đường" but backspaced to "đườ" and wants to continue
    let result = restore_and_type(&mut e, "đườ", "ng");
    assert_eq!(
        result, "đường",
        "Should complete word after partial restore"
    );
}

/// Scenario: restore word, change mark, then add more chars
#[test]
fn restore_word_change_mark_then_extend() {
    let mut e = Engine::new();

    // Screen has "chà", restore and change to sắc, then add 'o'
    let result = restore_and_type(&mut e, "chà", "so");
    assert_eq!(result, "cháo", "Should change mark and extend word");
}

// ============================================================
// OIW VS OWI BUG FIX TEST
// ============================================================

/// Bug: "oiw" produces error but "owi" → "ơi" works
/// Expected: Both should produce valid Vietnamese
#[test]
fn oiw_vs_owi_order() {
    // "owi" = o + w → ơ, then + i → ơi (works)
    // "oiw" = o + i → oi, then + w should → ơi (should work too)
    telex(&[
        ("owi ", "ơi "),
        ("oiw ", "ơi "), // Bug: this was failing
    ]);
}

// Debug test for oiw
#[test]
fn test_debug_oiw() {
    use gonhanh_core::data::keys;
    use gonhanh_core::engine::validation::is_valid;
    use gonhanh_core::engine::Engine;

    // First check if "oi" is considered valid Vietnamese
    let oi_keys = vec![keys::O, keys::I];
    println!("is_valid([O, I]) = {}", is_valid(&oi_keys));

    let mut e = Engine::new();

    // Step by step - simulating what type_word does
    let mut screen = String::new();

    // Type 'o'
    let r = e.on_key(keys::O, false, false);
    if r.action == 1 {
        for _ in 0..r.backspace {
            screen.pop();
        }
        for i in 0..r.count as usize {
            if let Some(ch) = char::from_u32(r.chars[i]) {
                screen.push(ch);
            }
        }
    } else {
        screen.push('o');
    }
    println!("After O: screen='{}', action={}", screen, r.action);

    // Type 'i'
    let r = e.on_key(keys::I, false, false);
    if r.action == 1 {
        for _ in 0..r.backspace {
            screen.pop();
        }
        for i in 0..r.count as usize {
            if let Some(ch) = char::from_u32(r.chars[i]) {
                screen.push(ch);
            }
        }
    } else {
        screen.push('i');
    }
    println!("After I: screen='{}', action={}", screen, r.action);

    // Type 'w'
    let r = e.on_key(keys::W, false, false);
    println!(
        "W result: action={}, backspace={}, count={}",
        r.action, r.backspace, r.count
    );
    if r.action == 1 {
        for _ in 0..r.backspace {
            screen.pop();
        }
        for i in 0..r.count as usize {
            if let Some(ch) = char::from_u32(r.chars[i]) {
                screen.push(ch);
            }
        }
    } else {
        screen.push('w');
    }
    println!("After W: screen='{}'", screen);

    // Type ' ' (space)
    let r = e.on_key(keys::SPACE, false, false);
    println!(
        "SPACE result: action={}, backspace={}, count={}",
        r.action, r.backspace, r.count
    );
    // Print the chars
    if r.count > 0 {
        let chars: String = r.chars[..r.count as usize]
            .iter()
            .filter_map(|&c| char::from_u32(c))
            .collect();
        println!("SPACE output chars: '{}'", chars);
    }

    if r.action == 1 {
        for _ in 0..r.backspace {
            screen.pop();
        }
        for i in 0..r.count as usize {
            if let Some(ch) = char::from_u32(r.chars[i]) {
                screen.push(ch);
            }
        }
    } else {
        screen.push(' ');
    }
    println!("After SPACE: screen='{}'", screen);

    assert_eq!(screen, "ơi ", "oiw followed by space should become 'ơi '");
}

// Bug: "rieneg" produces error but "rieeng" → "riêng" works
// Bug: "nafo" produces error but "naof" → "nào" works
#[test]
fn test_rieneg_vs_rieeng() {
    telex(&[
        ("rieeng ", "riêng "), // Should work (circumflex on e, n+g final)
        ("rieneg ", "riêng "), // Bug: reported as error
    ]);
}

#[test]
fn test_nafo_vs_naof() {
    telex(&[
        ("naof ", "nào "), // Should work (huyền tone on a)
        ("nafo ", "nào "), // Bug: reported as error
    ]);
}

// Debug test for rieneg - circumflex modifier order
#[test]
fn test_debug_rieneg() {
    use gonhanh_core::data::keys;
    use gonhanh_core::engine::Engine;

    let mut e = Engine::new();
    let mut screen = String::new();

    let keys_to_type = [
        (keys::R, 'r'),
        (keys::I, 'i'),
        (keys::E, 'e'),
        (keys::N, 'n'),
        (keys::E, 'e'),
        (keys::G, 'g'),
        (keys::SPACE, ' '),
    ];

    for (key, default_char) in keys_to_type {
        let r = e.on_key(key, false, false);
        println!(
            "After {:?}: action={}, backspace={}, count={}",
            default_char, r.action, r.backspace, r.count
        );
        if r.action == 1 {
            for _ in 0..r.backspace {
                screen.pop();
            }
            for i in 0..r.count as usize {
                if let Some(ch) = char::from_u32(r.chars[i]) {
                    screen.push(ch);
                }
            }
        } else {
            screen.push(default_char);
        }
        println!("  screen='{}'", screen);
    }

    println!("Final: '{}'", screen);
}

/// Issue #107: Shortcut with special character prefix should work
/// "#fne" should expand to its replacement when followed by space
#[test]
fn shortcut_with_hash_prefix() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.shortcuts_mut()
        .add(Shortcut::new("#fne", "for next episode"));

    // Type "#fne" + SPACE
    // # is Shift+3 (keycode N3 with shift=true)
    e.on_key_ext(keys::N3, false, false, true); // # (Shift+3)
    e.on_key(keys::F, false, false);
    e.on_key(keys::N, false, false);
    e.on_key(keys::E, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    // Should trigger shortcut
    assert_eq!(
        r.action,
        Action::Send as u8,
        "shortcut '#fne' should trigger"
    );
    // Backspace count should be 4 (length of "#fne"), NOT 5
    // Bug: +1 was incorrectly added, causing deletion of char from previous row
    assert_eq!(r.backspace, 4, "backspace should equal trigger length");
    let chars: String = (0..r.count as usize)
        .map(|i| char::from_u32(r.chars[i]).unwrap_or('?'))
        .collect();
    assert_eq!(chars, "for next episode ");
}

/// Issue #107: Same test but with "#fnv" shortcut
#[test]
fn shortcut_with_hash_prefix_fnv() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.shortcuts_mut()
        .add(Shortcut::new("#fnv", "for next version"));

    // Type "#fnv" + SPACE
    e.on_key_ext(keys::N3, false, false, true); // # (Shift+3)
    e.on_key(keys::F, false, false);
    e.on_key(keys::N, false, false);
    e.on_key(keys::V, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(
        r.action,
        Action::Send as u8,
        "shortcut '#fnv' should trigger"
    );
    assert_eq!(r.backspace, 4, "backspace = trigger.len() = 4");
}

/// Issue #107: Regular shortcut "đc" -> "được" should have correct backspace count
/// User types "ddc" (3 keys) but screen shows "đc" (2 chars)
/// Shortcut trigger is "đc" (2 chars) so backspace should be 2
#[test]
fn shortcut_vietnamese_dc_duoc() {
    let mut e = Engine::new();
    e.set_method(0); // Telex
    e.shortcuts_mut().add(Shortcut::new("đc", "được"));

    // Type "đc" using Telex: d + d + c
    e.on_key(keys::D, false, false); // d
    e.on_key(keys::D, false, false); // dd -> đ
    e.on_key(keys::C, false, false); // đc
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, Action::Send as u8, "shortcut 'đc' should trigger");
    // Screen shows "đc" = 2 chars, so backspace should be 2
    assert_eq!(
        r.backspace, 2,
        "backspace should be 2 (screen chars), not 3 (keys typed)"
    );
    let chars: String = (0..r.count as usize)
        .map(|i| char::from_u32(r.chars[i]).unwrap_or('?'))
        .collect();
    assert_eq!(chars, "được ");
}

/// Shortcut with circumflex: "ân" (2 chars, â=2 bytes) -> backspace=2
#[test]
fn shortcut_backspace_circumflex() {
    let mut e = Engine::new();
    e.set_method(0);
    e.shortcuts_mut().add(Shortcut::new("ân", "ân cần"));

    // Type "ân": a + a + n
    e.on_key(keys::A, false, false);
    e.on_key(keys::A, false, false); // aa -> â
    e.on_key(keys::N, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, Action::Send as u8);
    assert_eq!(r.backspace, 2, "ân = 2 chars");
}

/// Shortcut with horn: "ư" (1 char, 2 bytes) -> backspace=1
#[test]
fn shortcut_backspace_horn_single() {
    let mut e = Engine::new();
    e.set_method(0);
    e.shortcuts_mut().add(Shortcut::new("ư", "ừ"));

    // Type "ư": u + w
    e.on_key(keys::U, false, false);
    e.on_key(keys::W, false, false); // uw -> ư
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, Action::Send as u8);
    assert_eq!(r.backspace, 1, "ư = 1 char");
}

/// Shortcut with multiple Vietnamese chars: "đây" (3 chars) -> backspace=3
#[test]
fn shortcut_backspace_multiple_viet_chars() {
    let mut e = Engine::new();
    e.set_method(0);
    e.shortcuts_mut().add(Shortcut::new("đây", "đây là"));

    // Type "đây": d + d + a + a + y
    e.on_key(keys::D, false, false);
    e.on_key(keys::D, false, false); // đ
    e.on_key(keys::A, false, false);
    e.on_key(keys::A, false, false); // â
    e.on_key(keys::Y, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, Action::Send as u8);
    assert_eq!(r.backspace, 3, "đây = 3 chars");
}

/// Shortcut pure ASCII trigger: "vn" -> "Việt Nam"
#[test]
fn shortcut_backspace_ascii_trigger() {
    let mut e = Engine::new();
    e.set_method(0);
    e.shortcuts_mut().add(Shortcut::new("vn", "Việt Nam"));

    e.on_key(keys::V, false, false);
    e.on_key(keys::N, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, Action::Send as u8);
    assert_eq!(r.backspace, 2, "vn = 2 ASCII chars");
}

/// Shortcut with complex horn: "ươc" (2 chars: ươ + c)
#[test]
fn shortcut_backspace_complex_horn() {
    let mut e = Engine::new();
    e.set_method(0);
    e.shortcuts_mut().add(Shortcut::new("ươc", "ước mơ"));

    // Type "ươc": u + o + w + c
    e.on_key(keys::U, false, false);
    e.on_key(keys::O, false, false);
    e.on_key(keys::W, false, false); // uo -> ươ
    e.on_key(keys::C, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, Action::Send as u8);
    assert_eq!(r.backspace, 3, "ươc = 3 chars (ư + ơ + c)");
}

/// Shortcut "#abcd" with hash prefix: 5 chars -> backspace=5
/// Note: avoid "test" because 's' is Telex modifier (sắc)
#[test]
fn shortcut_backspace_hash_prefix_longer() {
    let mut e = Engine::new();
    e.set_method(0);
    e.shortcuts_mut().add(Shortcut::new("#abcd", "alphabet"));

    e.on_key_ext(keys::N3, false, false, true); // #
    e.on_key(keys::A, false, false);
    e.on_key(keys::B, false, false);
    e.on_key(keys::C, false, false);
    e.on_key(keys::D, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, Action::Send as u8);
    assert_eq!(r.backspace, 5, "#abcd = 5 chars");
}

/// Shortcut "@vn" with at prefix + Vietnamese output
#[test]
fn shortcut_backspace_at_prefix() {
    let mut e = Engine::new();
    e.set_method(0);
    e.shortcuts_mut().add(Shortcut::new("@vn", "Việt Nam"));

    e.on_key_ext(keys::N2, false, false, true); // @
    e.on_key(keys::V, false, false);
    e.on_key(keys::N, false, false);
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, Action::Send as u8);
    assert_eq!(r.backspace, 3, "@vn = 3 chars");
}

/// Shortcut with mark (sắc): "án" trigger (2 chars)
/// Note: "án" = a + s + n in Telex (mark then consonant, no revert)
#[test]
fn shortcut_backspace_with_mark() {
    let mut e = Engine::new();
    e.set_method(0);
    e.shortcuts_mut().add(Shortcut::new("án", "ấy nhé"));

    // Type "án": a + s + n
    e.on_key(keys::A, false, false);
    e.on_key(keys::S, false, false); // á
    e.on_key(keys::N, false, false); // án
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(r.action, Action::Send as u8);
    assert_eq!(r.backspace, 2, "án = 2 chars");
}

// ============================================================
// COMPLEX TYPING SCENARIOS: Type -> Delete -> Retype -> Clear
// Tests for "màu sắc" with Telex + auto restore
// ============================================================

/// Comprehensive continuous typing session with multiple operations
/// Simulates real user behavior: type->select->replace->arrow->backspace
#[test]
fn complex_mau_sac_continuous_session() {
    let mut e = Engine::new();
    e.set_method(0);
    e.set_english_auto_restore(true);

    // 1. Type "màu sắc", Cmd+A (select all), type new text to replace
    let r1 = type_word(&mut e, "mauf ");
    assert_eq!(r1, "màu ");
    let r2 = type_word(&mut e, "sawsc");
    assert_eq!(r2, "sắc");
    e.on_key(keys::A, false, true); // Cmd+A / Ctrl+A select all -> clears buffer
    let r3 = type_word(&mut e, "ddepj "); // Type to replace selection
    assert_eq!(r3, "đẹp ");

    // 2. Type word, Shift+Left (select chars), type to replace
    let r4 = type_word(&mut e, "vieejt");
    assert_eq!(r4, "việt");
    e.on_key_ext(keys::LEFT, false, false, true); // Shift+Left -> select, clears buffer
    let r5 = type_word(&mut e, "nam "); // Type replaces selection
    assert_eq!(r5, "nam ");

    // 3. Type word, Shift+Cmd+Left (select to line start), type to replace
    type_word(&mut e, "mauf sawsc");
    e.on_key(keys::LEFT, false, true); // Cmd+Left or Ctrl+Left -> clears buffer
    let r6 = type_word(&mut e, "chafo "); // Fresh typing
    assert_eq!(r6, "chào ");

    // 4. Type multiple words, Cmd+A, type fresh sentence
    let r7 = type_word(&mut e, "xin ");
    assert_eq!(r7, "xin ");
    type_word(&mut e, "chafo");
    e.on_key(keys::A, false, true); // Select all
    let r8 = type_word(&mut e, "mauf sawsc "); // Replace with "màu sắc "
    assert_eq!(r8, "màu sắc ");

    // 5. Type, arrow interrupt mid-word, continue fresh
    type_word(&mut e, "ddepj");
    e.on_key(keys::RIGHT, false, false); // Arrow clears buffer
    let r9 = type_word(&mut e, "lawsm ");
    assert_eq!(r9, "lắm ");

    // 6. Type word, Shift+Right (extend selection), type to replace
    let r10 = type_word(&mut e, "tuyeejt");
    assert_eq!(r10, "tuyệt");
    e.on_key_ext(keys::RIGHT, false, false, true); // Shift+Right -> clears buffer
    let r11 = type_word(&mut e, "vowfi "); // Fresh type
    assert_eq!(r11, "vời ");

    // 7. Rapid: type -> select partial -> type -> select all -> type
    let r12 = type_word(&mut e, "mauf ");
    assert_eq!(r12, "màu ");
    type_word(&mut e, "sawsc");
    e.on_key_ext(keys::LEFT, false, false, true); // Shift+Left
    e.on_key_ext(keys::LEFT, false, false, true); // Shift+Left again (more selection)
    let r13 = type_word(&mut e, "ddor "); // Replace selection
    assert_eq!(r13, "đỏ ");

    // 8. Type sentence, Tab key (word boundary), type fresh
    let r14 = type_word(&mut e, "toost ");
    assert_eq!(r14, "tốt ");
    type_word(&mut e, "lawsm");
    e.on_key(keys::TAB, false, false); // Tab clears buffer
    let r15 = type_word(&mut e, "raast "); // Fresh
    assert_eq!(r15, "rất ");

    // 9. Type, Enter key, continue fresh
    type_word(&mut e, "tuyeejt");
    e.on_key(keys::RETURN, false, false); // Enter clears buffer
    let r16 = type_word(&mut e, "vowfi ");
    assert_eq!(r16, "vời ");

    // 10. Complex: type -> partial delete -> arrow -> fresh -> Cmd+A -> replace
    let r17 = type_word(&mut e, "mauf<"); // màu -> mà (delete u)
    assert_eq!(r17, "mà");
    e.on_key(keys::LEFT, false, false); // Arrow clears buffer
    let r18 = type_word(&mut e, "sawsc ");
    assert_eq!(r18, "sắc ");
    type_word(&mut e, "ddepj");
    e.on_key(keys::A, false, true); // Select all
    let r19 = type_word(&mut e, "xong ");
    assert_eq!(r19, "xong ");
}

/// Test mark position for "aum" pattern (from debug log issue)
/// "aumf" should produce "àum" (mark on 'a'), NOT "aùm" (mark on 'u')
#[test]
fn test_aum_mark_position() {
    let mut e = Engine::new();
    e.set_method(0); // Telex

    // "aumf" = a + u + m + f(huyền) → mark should be on 'a' → "àum"
    let r = type_word(&mut e, "aumf ");
    assert_eq!(r, "àum ", "Mark should be on 'a', not 'u'");

    // Also test "aufm" = same result
    let r2 = type_word(&mut e, "aufm ");
    assert_eq!(r2, "àum ");

    // Test "aaumf" - should have circumflex from "aa" doubling → "ầum"
    let r3 = type_word(&mut e, "aaumf ");
    assert_eq!(r3, "ầum ", "aa doubling should add circumflex");

    // Test "mauf" → "màu" (mau + f)
    let r4 = type_word(&mut e, "mauf ");
    assert_eq!(r4, "màu ");

    // Test various "au" patterns
    let r5 = type_word(&mut e, "sauf "); // sàu
    assert_eq!(r5, "sàu ");

    let r6 = type_word(&mut e, "mafu "); // màu (mark before u)
    assert_eq!(r6, "màu ");
}

/// Test what input produces "ầum" (circumflex with huyền)
/// User reported getting "ầum" when trying to type "màu"
#[test]
fn test_aum_circumflex() {
    let mut e = Engine::new();
    e.set_method(0); // Telex

    // "aumf" should give "àum" (huyền on first vowel 'a' per TONE_FIRST_PATTERNS)
    let r1 = type_word(&mut e, "aumf");
    println!("aumf → '{}' (expected: àum)", r1);
    assert_eq!(r1, "àum");

    // Reset engine
    let mut e = Engine::new();
    e.set_method(0);

    // "aaumf" would give "ầum" (aa→â, then f adds huyền)
    let r2 = type_word(&mut e, "aaumf");
    println!("aaumf → '{}' (expected: ầum)", r2);
    assert_eq!(r2, "ầum", "aaumf produces ầum - this is what user sees");

    // Now test if there's a way to get ầum from "màu sắc" delete sequence
    let mut e = Engine::new();
    e.set_method(0);

    // Type "sắc " first (last word before issue)
    type_word(&mut e, "sawsc ");

    // Now simulate various delete patterns and try to get ầum
    // Pattern 1: Restore word, partially delete, then type
    e.on_key(keys::DELETE, false, false); // Restore "sắc"
    e.on_key(keys::DELETE, false, false); // sắ
    e.on_key(keys::DELETE, false, false); // s
    e.on_key(keys::DELETE, false, false); // empty
    e.on_key(keys::DELETE, false, false); // still empty

    // Now type "aumf" expecting "àum"
    let r3 = type_word(&mut e, "aumf");
    println!("After deleting sắc, aumf → '{}' (expected: àum)", r3);
    assert_eq!(r3, "àum", "Should not get ầum from this sequence");
}

/// Test DELETE restore behavior
/// After typing "màu " and pressing DELETE, buffer is restored.
/// Need 5 DELETEs to fully clear: 1 (restore) + 3 (chars) + 1 (empty)
#[test]
fn test_delete_restore_behavior() {
    let mut e = Engine::new();
    e.set_method(0); // Telex

    // Type "màu "
    let r1 = type_word(&mut e, "mauf ");
    assert_eq!(r1, "màu ");

    // DELETE behavior after commit:
    // - DELETE 1: Restores "màu" (3 chars) from word_history
    // - DELETE 2-4: Pop chars one by one
    // - DELETE 5+: Empty buffer, sets has_non_letter_prefix (affects shortcuts only)

    // DELETE 1: Restore word
    let del1 = e.on_key(keys::DELETE, false, false);
    assert_eq!(del1.action, 1, "DELETE 1 should restore");

    // DELETE 2-4: Pop from restored buffer
    for _ in 0..3 {
        e.on_key(keys::DELETE, false, false);
    }

    // DELETE 5: Empty buffer
    e.on_key(keys::DELETE, false, false);

    // Now typing fresh should work
    let r2 = type_word(&mut e, "mauf ");
    assert_eq!(r2, "màu ", "After full delete, typing should work");
}

/// Test "màu sắc" user journey - exact reproduction
/// User reported: type "màu sắc", delete nhiều lần, gõ lại → "ầum" hoặc "ầumu"
#[test]
fn test_mau_sac_user_journey() {
    let mut e = Engine::new();
    e.set_method(0); // Telex

    // === JOURNEY 1: Gõ "màu sắc" lần đầu ===
    let r1 = type_word(&mut e, "mauf ");
    assert_eq!(r1, "màu ", "Journey 1.1: mauf → màu");
    let r2 = type_word(&mut e, "sawsc ");
    assert_eq!(r2, "sắc ", "Journey 1.2: sawsc → sắc");

    // === JOURNEY 2: Gõ "màu sắc" lần 2 ===
    let r3 = type_word(&mut e, "mauf ");
    assert_eq!(r3, "màu ", "Journey 2.1: mauf → màu");
    let r4 = type_word(&mut e, "sawsc ");
    assert_eq!(r4, "sắc ", "Journey 2.2: sawsc → sắc");

    // === JOURNEY 3: Xóa nhiều lần rồi gõ lại ===
    // Xóa 10 lần (nhiều hơn cần thiết để đảm bảo buffer trống)
    for _ in 0..10 {
        e.on_key(keys::DELETE, false, false);
    }

    // Gõ lại "màu sắc" - KHÔNG được có circumflex (ầ)
    let r5 = type_word(&mut e, "mauf ");
    assert_eq!(
        r5, "màu ",
        "Journey 3.1: After 10 DELETEs, mauf → màu (not ầum!)"
    );
    let r6 = type_word(&mut e, "sawsc ");
    assert_eq!(r6, "sắc ", "Journey 3.2: sawsc → sắc");

    // === JOURNEY 4: Xóa từng chữ và gõ lại ===
    // Simulate: user delete "sắc " char by char, then type "màu"
    for _ in 0..4 {
        // "sắc " = 4 chars on screen
        e.on_key(keys::DELETE, false, false);
    }
    let r7 = type_word(&mut e, "mauf ");
    assert_eq!(r7, "màu ", "Journey 4: After deleting sắc, mauf → màu");

    // === JOURNEY 5: Repeat cycle nhiều lần ===
    for i in 0..5 {
        // Gõ màu sắc
        let m = type_word(&mut e, "mauf ");
        let s = type_word(&mut e, "sawsc ");
        assert_eq!(m, "màu ", "Journey 5.{}: mauf → màu", i);
        assert_eq!(s, "sắc ", "Journey 5.{}: sawsc → sắc", i);

        // Xóa hết
        for _ in 0..15 {
            e.on_key(keys::DELETE, false, false);
        }
    }

    // Final check: sau nhiều cycle, vẫn phải đúng
    let final_r = type_word(&mut e, "mauf ");
    assert_eq!(final_r, "màu ", "Final: After many cycles, mauf → màu");
}

/// Test specific case: sau khi xóa và gõ lại nhiều lần
/// Kiểm tra không có "ầ" (circumflex+huyền) khi chỉ gõ "màu"
#[test]
fn test_no_circumflex_contamination() {
    let mut e = Engine::new();
    e.set_method(0);

    // Pattern 1: Type → Delete all → Retype
    for round in 0..10 {
        let r = type_word(&mut e, "mauf ");
        assert_eq!(r, "màu ", "Round {}: mauf → màu (not ầum)", round);

        // Delete everything
        for _ in 0..20 {
            e.on_key(keys::DELETE, false, false);
        }
    }

    // Pattern 2: Type màu sắc alternating
    for round in 0..5 {
        let r1 = type_word(&mut e, "mauf ");
        let r2 = type_word(&mut e, "sawsc ");
        assert!(
            !r1.contains('ầ'),
            "Round {}: màu should NOT contain ầ (got: {})",
            round,
            r1
        );
        assert_eq!(r1, "màu ");
        assert_eq!(r2, "sắc ");

        // Delete random amount
        for _ in 0..(round + 5) {
            e.on_key(keys::DELETE, false, false);
        }
    }
}

/// Test edge case: partial delete trong word
#[test]
fn test_partial_delete_in_word() {
    let mut e = Engine::new();
    e.set_method(0);

    // Type "mau" then delete 'u', type 'u' again, then 'f'
    // m → a → u → DELETE → u → f → SPACE
    e.on_key_ext(keys::M, false, false, false);
    e.on_key_ext(keys::A, false, false, false);
    e.on_key_ext(keys::U, false, false, false);
    e.on_key(keys::DELETE, false, false); // Delete 'u'
    e.on_key_ext(keys::U, false, false, false); // Re-type 'u'
    let r = e.on_key_ext(keys::F, false, false, false);

    // Should produce "màu" not "ầum"
    // Check the transformation
    if r.action == 1 {
        let output: String = r.chars[..r.count as usize]
            .iter()
            .filter_map(|&c| char::from_u32(c))
            .collect();
        assert!(
            !output.contains('ầ'),
            "Partial delete: should not produce circumflex (got: {})",
            output
        );
    }
}

/// Test: gõ sai rồi sửa
#[test]
fn test_typo_correction() {
    let mut e = Engine::new();
    e.set_method(0);

    // User gõ "maau" (lỡ 2 chữ a) rồi delete 1 'a', gõ tiếp
    // m → a → a → DELETE → u → f
    e.on_key_ext(keys::M, false, false, false);
    e.on_key_ext(keys::A, false, false, false);
    let aa_result = e.on_key_ext(keys::A, false, false, false); // This creates "â"
    println!("After 'aa': action={}", aa_result.action);

    e.on_key(keys::DELETE, false, false); // Delete to fix

    e.on_key_ext(keys::U, false, false, false);
    let f_result = e.on_key_ext(keys::F, false, false, false);

    // After correction, should be "màu" not "ầu"
    if f_result.action == 1 {
        let output: String = f_result.chars[..f_result.count as usize]
            .iter()
            .filter_map(|&c| char::from_u32(c))
            .collect();
        println!("Typo correction result: {}", output);
        // Note: This might still have circumflex if 'aa' created 'â' and DELETE
        // only removed 'u', not the circumflex. This could be the bug!
    }
}

/// BUG REPRODUCTION: DELETE restore rồi gõ tiếp = append vào buffer cũ
/// User nghĩ đang gõ mới nhưng thực ra đang append vào word đã restore
#[test]
fn test_bug_delete_restore_then_type() {
    let mut e = Engine::new();
    e.set_method(0);

    // Step 1: Gõ "sắc " và commit
    let r1 = type_word(&mut e, "sawsc ");
    assert_eq!(r1, "sắc ");
    println!("After 'sắc ': committed");

    // Step 2: DELETE 1 lần = restore "sắc" vào buffer
    let del = e.on_key(keys::DELETE, false, false);
    println!("DELETE: action={}, backspace={}", del.action, del.backspace);
    // Buffer now = "sắc" (restored), NOT empty!

    // Step 3: User NGHĨ buffer trống, gõ "mauf"
    // Nhưng thực ra đang append vào "sắc" → "sắcmauf"
    let r2 = type_word(&mut e, "mauf ");
    println!("After typing 'mauf ': got '{}'", r2);

    // BUG: User expect "màu " nhưng có thể nhận được gì đó khác
    // vì buffer đang là "sắc" + "mauf" = "sắcmauf"
    assert_eq!(
        r2, "màu ",
        "BUG: User expects fresh 'màu' but buffer was not empty!"
    );
}

/// BUG REPRODUCTION 2: DELETE không đủ số lần
/// User delete 3 lần cho "sắc " nhưng cần 4 (space + 3 chars) hoặc nhiều hơn
#[test]
fn test_bug_insufficient_deletes() {
    let mut e = Engine::new();
    e.set_method(0);

    // Gõ "màu sắc "
    type_word(&mut e, "mauf ");
    type_word(&mut e, "sawsc ");
    println!("Typed 'màu sắc '");

    // User delete 3 lần - KHÔNG ĐỦ để clear hết
    // DELETE 1: restore "sắc"
    // DELETE 2: "sắ"
    // DELETE 3: "s"
    // Buffer vẫn còn "s"!
    for i in 0..3 {
        let d = e.on_key(keys::DELETE, false, false);
        println!("DELETE {}: action={}", i + 1, d.action);
    }

    // Gõ "mauf" - nhưng buffer còn "s" → "smauf"
    let r = type_word(&mut e, "mauf ");
    println!("After 3 DELETEs + 'mauf ': got '{}'", r);

    // BUG: Expect "màu " nhưng có thể nhận được khác
    assert_eq!(r, "màu ", "BUG: Insufficient deletes leaves stale buffer!");
}

/// BUG REPRODUCTION 3: Scenario chính xác từ user
/// "màu sắc" → xóa nhiều lần → gõ lại → "ầum"
#[test]
fn test_bug_mau_sac_aum_output() {
    let mut e = Engine::new();
    e.set_method(0);

    // Gõ "màu sắc" nhiều lần
    for _ in 0..3 {
        type_word(&mut e, "mauf ");
        type_word(&mut e, "sawsc ");
    }

    // Xóa - nhưng có thể không đúng số lần
    // Giả sử user xóa random số lần
    for _ in 0..7 {
        e.on_key(keys::DELETE, false, false);
    }

    // Gõ lại - expect "màu" nhưng user báo nhận "ầum"
    let result = type_word(&mut e, "mauf ");
    println!("Result after deletes: '{}'", result);

    // Check không có circumflex
    assert!(
        !result.contains('ầ'),
        "BUG REPRODUCED: Got '{}' instead of 'màu ' - circumflex contamination!",
        result
    );
    assert_eq!(result, "màu ");
}

/// Test "chuẩn bị" - tone on â (not ủ)
/// "ua" with final consonant: u is medial, a is main → tone on a
#[test]
fn test_chuan_bi_tone_position() {
    let mut e = Engine::new();
    e.set_method(0); // Telex

    // "chuẩn" = ch + u + aa + r + n
    // aa = circumflex on a → â
    // r = hỏi tone
    // With final 'n', "uâ" → tone on â (not ủ)
    let result = type_word(&mut e, "chuaarn ");
    assert_eq!(result, "chuẩn ", "ua with final consonant: tone on â, not ủ");

    // "bị" = b + i + j
    let result2 = type_word(&mut e, "bij ");
    assert_eq!(result2, "bị ");
}

/// Test "thuận lợi" - another ua + final consonant case
#[test]
fn test_thuan_loi_tone_position() {
    let mut e = Engine::new();
    e.set_method(0); // Telex

    // "thuận" = th + u + aa + j + n
    let result = type_word(&mut e, "thuaajn ");
    assert_eq!(result, "thuận ", "ua with final: tone on â");

    // "lợi" = l + o + w + j + i
    let result2 = type_word(&mut e, "lowji ");
    assert_eq!(result2, "lợi ");
}

/// Test open syllable "mùa" vs closed syllable "muán"
#[test]
fn test_ua_open_vs_closed_syllable() {
    let mut e = Engine::new();
    e.set_method(0); // Telex

    // Open syllable: tone on u (mùa)
    let r1 = type_word(&mut e, "muaf ");
    assert_eq!(r1, "mùa ", "ua open syllable: tone on u");

    // Closed syllable: tone on a (muán)
    let r2 = type_word(&mut e, "muasn ");
    assert_eq!(r2, "muán ", "ua closed syllable: tone on a");
}
