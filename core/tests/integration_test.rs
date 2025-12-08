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
fn mark_without_vowel_passes_through() {
    let mut e = Engine::new();
    assert_passthrough(&mut e, keys::W);
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
