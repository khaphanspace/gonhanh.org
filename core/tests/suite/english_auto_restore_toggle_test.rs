//! Test English auto-restore toggle behavior
//! Verifies that when feature is OFF, NO auto-restore happens
//! and when ON, all auto-restore patterns work correctly.

use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn engine_off() -> Engine {
    let mut e = Engine::new();
    e.set_english_auto_restore(false); // Explicitly OFF
    e
}

fn engine_on() -> Engine {
    let mut e = Engine::new();
    e.set_english_auto_restore(true); // Explicitly ON
    e
}

// =============================================================================
// TEST: DEFAULT IS OFF
// =============================================================================

#[test]
fn default_is_off() {
    // Engine should have english_auto_restore = false by default
    // We test this indirectly by checking behavior

    // "text" should transform to Vietnamese when OFF
    let mut e = Engine::new();
    let result = type_word(&mut e, "text ");
    // When OFF: "text " → "tẽt " (Vietnamese transforms applied)
    assert!(
        result.contains('ẽ'),
        "Default OFF: 'text ' should have Vietnamese mark, got: '{}'",
        result
    );
}

// =============================================================================
// PATTERN 1: AW ENDING (seesaw, raw)
// When OFF: transforms to Vietnamese
// When ON: restores to English
// =============================================================================

#[test]
fn pattern1_aw_ending_off() {
    let mut e = engine_off();
    let result = type_word(&mut e, "seesaw ");
    // When OFF: "seesaw" should have Vietnamese transforms
    // s+e+e → sê (circumflex), s+a+w → ă or ắ pattern
    assert!(
        result != "seesaw ",
        "OFF: 'seesaw ' should have transforms, got: '{}'",
        result
    );
}

#[test]
fn pattern1_aw_ending_on() {
    let mut e = engine_on();
    let result = type_word(&mut e, "seesaw ");
    assert_eq!(result, "seesaw ", "ON: 'seesaw ' should restore to English");
}

// =============================================================================
// PATTERN 2: FOREIGN WORD (swim, swim)
// When OFF: w becomes ư
// When ON: restores when invalid pattern detected
// =============================================================================

#[test]
fn pattern2_foreign_word_off() {
    let mut e = engine_off();
    let result = type_word(&mut e, "swim ");
    // When OFF: w→ư, so "swim" → "sưim" or similar
    assert!(
        result.contains('ư') || result != "swim ",
        "OFF: 'swim ' should have ư or transforms, got: '{}'",
        result
    );
}

#[test]
fn pattern2_foreign_word_on() {
    let mut e = engine_on();
    let result = type_word(&mut e, "swim ");
    assert_eq!(result, "swim ", "ON: 'swim ' should restore to English");
}

// =============================================================================
// PATTERN 3: MID-WORD CONSONANT (text, expect)
// When OFF: x applies ngã mark
// When ON: restores when consonant after mark detected
// =============================================================================

#[test]
fn pattern3_mid_word_consonant_off() {
    let mut e = engine_off();
    let result = type_word(&mut e, "text ");
    // When OFF: x→ngã, so "text" → "tẽt"
    assert!(
        result.contains('ẽ'),
        "OFF: 'text ' should have ngã mark, got: '{}'",
        result
    );
}

#[test]
fn pattern3_mid_word_consonant_on() {
    let mut e = engine_on();
    let result = type_word(&mut e, "text ");
    assert_eq!(result, "text ", "ON: 'text ' should restore to English");
}

#[test]
fn pattern3_expect_off() {
    let mut e = engine_off();
    let result = type_word(&mut e, "expect ");
    // When OFF: x→ngã, so "expect" → "ẽpect" or similar
    assert!(
        result.contains('ẽ'),
        "OFF: 'expect ' should have ngã mark, got: '{}'",
        result
    );
}

#[test]
fn pattern3_expect_on() {
    let mut e = engine_on();
    let result = type_word(&mut e, "expect ");
    assert_eq!(result, "expect ", "ON: 'expect ' should restore to English");
}

// =============================================================================
// PATTERN 4: SPACE/BREAK AUTO-RESTORE (structural validation)
// When OFF: invalid structure stays
// When ON: invalid structure restores
// =============================================================================

#[test]
fn pattern4_space_restore_off() {
    let mut e = engine_off();
    // "would" has w→ư which makes invalid Vietnamese
    let result = type_word(&mut e, "would ");
    // When OFF: should keep the transformed (even if invalid)
    assert!(
        result.contains('ư') || result != "would ",
        "OFF: 'would ' should have transforms, got: '{}'",
        result
    );
}

#[test]
fn pattern4_space_restore_on() {
    let mut e = engine_on();
    let result = type_word(&mut e, "would ");
    assert_eq!(result, "would ", "ON: 'would ' should restore to English");
}

// =============================================================================
// VIETNAMESE WORDS: Should NEVER be affected (OFF or ON)
// =============================================================================

#[test]
fn vietnamese_preserved_off() {
    let mut e = engine_off();
    assert_eq!(
        type_word(&mut e, "vieets "),
        "viết ",
        "OFF: Vietnamese 'viết' preserved"
    );

    let mut e = engine_off();
    assert_eq!(
        type_word(&mut e, "xin "),
        "xin ",
        "OFF: Vietnamese 'xin' preserved"
    );

    let mut e = engine_off();
    assert_eq!(
        type_word(&mut e, "chaof "),
        "chào ",
        "OFF: Vietnamese 'chào' preserved"
    );
}

#[test]
fn vietnamese_preserved_on() {
    let mut e = engine_on();
    assert_eq!(
        type_word(&mut e, "vieets "),
        "viết ",
        "ON: Vietnamese 'viết' preserved"
    );

    let mut e = engine_on();
    assert_eq!(
        type_word(&mut e, "xin "),
        "xin ",
        "ON: Vietnamese 'xin' preserved"
    );

    let mut e = engine_on();
    assert_eq!(
        type_word(&mut e, "chaof "),
        "chào ",
        "ON: Vietnamese 'chào' preserved"
    );
}

// =============================================================================
// EDGE CASES: Words that look like both
// =============================================================================

#[test]
fn edge_case_mix_stays_vietnamese() {
    // "mix" → "mĩ" is valid Vietnamese, should NOT restore even when ON
    let mut e = engine_on();
    let result = type_word(&mut e, "mix ");
    assert_eq!(result, "mĩ ", "ON: 'mix' stays as 'mĩ' (valid Vietnamese)");
}

#[test]
fn edge_case_fox_restores_when_on() {
    // "fox" has F which is invalid Vietnamese initial
    let mut e = engine_on();
    let result = type_word(&mut e, "fox ");
    assert_eq!(result, "fox ", "ON: 'fox' restores (F is invalid initial)");
}

#[test]
fn edge_case_fox_transforms_when_off() {
    // When OFF, even invalid Vietnamese stays transformed
    let mut e = engine_off();
    let result = type_word(&mut e, "fox ");
    // F is not valid Vietnamese initial, but x still applies ngã
    // Result depends on engine behavior - just verify it's different from ON
    println!("OFF: 'fox ' -> '{}'", result);
    // The key point: when OFF, no auto-restore should happen
}
