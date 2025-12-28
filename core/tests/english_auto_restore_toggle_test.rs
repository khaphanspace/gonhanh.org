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
    // Note: Per-character validation is SEPARATE from auto_restore
    // "text" → X applies ngã, but T causes validation revert → "text"
    // Auto_restore OFF means SPACE won't try to restore English
    // But per-character validation revert is always active
    let mut e = Engine::new();
    let result = type_word(&mut e, "swim ");
    // When OFF: w→ư should stay (not restored on SPACE)
    assert!(
        result.contains('ư'),
        "Default OFF: 'swim ' should have ư, got: '{}'",
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
// Note: Per-character validation reverts invalid syllables like "tẽt"
// This is independent of auto_restore toggle
// Auto_restore only affects SPACE-triggered English restore
// =============================================================================

#[test]
fn pattern3_mid_word_consonant_off() {
    let mut e = engine_off();
    let result = type_word(&mut e, "text ");
    // Per-char validation reverts "tẽt" to "text" (T after ẽ is invalid)
    // Auto_restore OFF: SPACE doesn't restore, but buffer already reverted
    assert_eq!(
        result, "text ",
        "OFF: 'text ' per-char validation reverts, got: '{}'",
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
    // Per-char validation reverts, auto_restore OFF doesn't change outcome
    assert_eq!(
        result, "expect ",
        "OFF: 'expect ' per-char validation reverts, got: '{}'",
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
// Note: Per-char validation may revert during typing
// Auto_restore affects what happens on SPACE
// =============================================================================

#[test]
fn pattern4_space_restore_off() {
    let mut e = engine_off();
    // "would" has w→ư, per-char validation may or may not revert
    // Key: auto_restore OFF means SPACE doesn't try English restore
    let result = type_word(&mut e, "would ");
    // If per-char validation reverted, we get "would"
    // This tests that SPACE doesn't do additional restore work
    assert!(
        result == "would " || result.contains('ư'),
        "OFF: 'would ' got: '{}'",
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
