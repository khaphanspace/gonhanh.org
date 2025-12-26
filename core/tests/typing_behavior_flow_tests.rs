//! Step-by-step tests for Vietnamese IME typing behavior flow
//!
//! Tests based on docs/typing-behavior-flow.md
//! Each example is tested step-by-step with VN(R), VN(B), EN(R), EN(B) validation.

mod common;

use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

/// Test step result
#[derive(Debug, Clone)]
struct StepResult {
    /// Raw buffer as string (for debugging)
    raw: String,
    /// Transformed buffer output
    output: String,
    /// VN(R) - Raw cluster validation (true=valid, false=invalid/has modifiers)
    vn_raw_valid: bool,
    /// VN(B) - Buffer syllable validation (only when VN(R)=false)
    vn_buf_valid: Option<bool>,
    /// In English mode
    in_english_mode: bool,
    /// Restore was triggered
    restore_triggered: bool,
}

/// Helper to type keys and capture state at each step
fn type_and_capture(keys_seq: &[u16]) -> Vec<StepResult> {
    let mut engine = Engine::new();
    let mut results = Vec::new();

    for &key in keys_seq {
        let r = engine.on_key(key, false, false);

        // Get raw buffer as string
        let raw = engine.get_raw_string();

        // Get output
        let output = engine.get_buffer_string();

        // Check if raw contains modifiers (w, j, s, f, r, x)
        let has_modifier = raw
            .chars()
            .any(|c| matches!(c, 'w' | 'j' | 's' | 'f' | 'r' | 'x'));

        // VN(R) = false if raw has modifiers or impossible clusters
        let vn_raw_valid = !has_modifier && !has_impossible_cluster(&raw);

        // VN(B) only checked when VN(R)=false
        let vn_buf_valid = if !vn_raw_valid {
            Some(engine.is_valid_vietnamese())
        } else {
            None
        };

        let in_english_mode = engine.is_english_mode();
        let restore_triggered = r.action == gonhanh_core::engine::Action::Restore as u8;

        results.push(StepResult {
            raw,
            output,
            vn_raw_valid,
            vn_buf_valid,
            in_english_mode,
            restore_triggered,
        });
    }

    results
}

/// Check if raw string has impossible Vietnamese clusters
fn has_impossible_cluster(raw: &str) -> bool {
    let bytes: Vec<char> = raw.chars().collect();
    if bytes.len() < 2 {
        return false;
    }

    // Check for impossible onset clusters (cl, xp, xt, etc.)
    for i in 0..bytes.len() - 1 {
        let pair = (bytes[i], bytes[i + 1]);
        match pair {
            ('c', 'l')
            | ('x', 'p')
            | ('x', 't')
            | ('b', 'r')
            | ('c', 'r')
            | ('d', 'r')
            | ('f', 'l')
            | ('f', 'r')
            | ('g', 'l')
            | ('g', 'r')
            | ('p', 'r')
            | ('s', 'c')
            | ('s', 'k')
            | ('s', 'l')
            | ('s', 'm')
            | ('s', 'n')
            | ('s', 'p')
            | ('s', 't')
            | ('s', 'w')
            | ('t', 'w')
            | ('w', 'h')
            | ('w', 'r') => return true,
            _ => {}
        }
    }
    false
}

// =============================================================================
// EXAMPLE 1: `d u o w c j d` + space → "được "
// =============================================================================

#[test]
fn test_example_1_duoc_step_by_step() {
    let keys = [
        keys::D,
        keys::U,
        keys::O,
        keys::W,
        keys::C,
        keys::J,
        keys::D,
    ];

    let mut engine = Engine::new();

    // Step 1: d
    engine.on_key(keys::D, false, false);
    assert_eq!(engine.get_buffer_string(), "d");
    assert!(!engine.is_english_mode(), "Step 1: Should be in VN mode");

    // Step 2: u
    engine.on_key(keys::U, false, false);
    assert_eq!(engine.get_buffer_string(), "du");
    assert!(!engine.is_english_mode(), "Step 2: Should be in VN mode");

    // Step 3: o
    engine.on_key(keys::O, false, false);
    assert_eq!(engine.get_buffer_string(), "duo");
    assert!(!engine.is_english_mode(), "Step 3: Should be in VN mode");

    // Step 4: w (HORN) - only o→ơ, u horn is deferred (Issue #133)
    engine.on_key(keys::W, false, false);
    assert_eq!(engine.get_buffer_string(), "duơ");
    assert!(
        !engine.is_english_mode(),
        "Step 4: VN(R)=✗ but VN(B)=✓, should stay VN"
    );

    // Step 5: c - final consonant triggers deferred horn on u
    engine.on_key(keys::C, false, false);
    assert_eq!(engine.get_buffer_string(), "dươc");
    assert!(!engine.is_english_mode(), "Step 5: Should stay in VN mode");

    // Step 6: j (TONE nặng)
    engine.on_key(keys::J, false, false);
    assert_eq!(engine.get_buffer_string(), "dược");
    assert!(!engine.is_english_mode(), "Step 6: Should stay in VN mode");

    // Step 7: d (STROKE - non-consecutive dd→đ)
    engine.on_key(keys::D, false, false);
    assert_eq!(engine.get_buffer_string(), "được");
    assert!(!engine.is_english_mode(), "Step 7: Should stay in VN mode");

    // Step 8: space (COMMIT)
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty(), "Buffer should be empty after commit");
}

// =============================================================================
// EXAMPLE 2: `d u w o w c d j` + space → "được "
// =============================================================================

#[test]
fn test_example_2_duoc_alternate_step_by_step() {
    let mut engine = Engine::new();

    // Step 1: d
    engine.on_key(keys::D, false, false);
    assert_eq!(engine.get_buffer_string(), "d");

    // Step 2: u
    engine.on_key(keys::U, false, false);
    assert_eq!(engine.get_buffer_string(), "du");

    // Step 3: w (HORN to u→ư)
    engine.on_key(keys::W, false, false);
    assert_eq!(engine.get_buffer_string(), "dư");

    // Step 4: o
    engine.on_key(keys::O, false, false);
    assert_eq!(engine.get_buffer_string(), "dưo");

    // Step 5: w (HORN to o→ơ)
    engine.on_key(keys::W, false, false);
    assert_eq!(engine.get_buffer_string(), "dươ");

    // Step 6: c
    engine.on_key(keys::C, false, false);
    assert_eq!(engine.get_buffer_string(), "dươc");

    // Step 7: d (STROKE)
    engine.on_key(keys::D, false, false);
    assert_eq!(engine.get_buffer_string(), "đươc");

    // Step 8: j (TONE nặng)
    engine.on_key(keys::J, false, false);
    assert_eq!(engine.get_buffer_string(), "được");

    // Step 9: space (COMMIT)
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty());
}

// =============================================================================
// EXAMPLE 3: `c l a r` + space → "clar " (FOREIGN mode)
// =============================================================================

#[test]
fn test_example_3_clar_foreign_step_by_step() {
    let mut engine = Engine::new();

    // Step 1: c
    engine.on_key(keys::C, false, false);
    assert_eq!(engine.get_buffer_string(), "c");
    assert!(!engine.is_english_mode(), "Step 1: Should be in VN mode");

    // Step 2: l - 'cl' is impossible Vietnamese cluster → FOREIGN
    engine.on_key(keys::L, false, false);
    assert_eq!(engine.get_buffer_string(), "cl");
    assert!(
        engine.is_english_mode(),
        "Step 2: 'cl' impossible → EN mode"
    );

    // Step 3: a (in EN mode, no transform)
    engine.on_key(keys::A, false, false);
    assert_eq!(engine.get_buffer_string(), "cla");

    // Step 4: r (REJECT in EN mode, just pass through)
    engine.on_key(keys::R, false, false);
    assert_eq!(engine.get_buffer_string(), "clar");

    // Step 5: space (COMMIT)
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty());
}

// =============================================================================
// EXAMPLE 4: `t e x t` + space → "text " (RESTORE triggered)
// =============================================================================

#[test]
fn test_example_4_text_restore_step_by_step() {
    let mut engine = Engine::new();

    // Step 1: t
    engine.on_key(keys::T, false, false);
    assert_eq!(engine.get_buffer_string(), "t");
    assert!(!engine.is_english_mode());

    // Step 2: e
    engine.on_key(keys::E, false, false);
    assert_eq!(engine.get_buffer_string(), "te");
    assert!(!engine.is_english_mode());

    // Step 3: x (TONE ngã) - 'x' is modifier
    engine.on_key(keys::X, false, false);
    assert_eq!(engine.get_buffer_string(), "tẽ");
    assert!(!engine.is_english_mode(), "VN(R)=✗ but VN(B)=✓");

    // Step 4: t - 'xt' is impossible → RESTORE
    let r = engine.on_key(keys::T, false, false);
    assert_eq!(engine.get_buffer_string(), "text");
    assert!(
        engine.is_english_mode(),
        "Step 4: RESTORE triggered → EN mode"
    );
    // Check restore was triggered
    assert_eq!(
        r.action,
        gonhanh_core::engine::Action::Restore as u8,
        "RESTORE action expected"
    );

    // Step 5: space
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty());
}

// =============================================================================
// EXAMPLE 5: `e x p e c t` + space → "expect " (RESTORE triggered)
// =============================================================================

#[test]
fn test_example_5_expect_restore_step_by_step() {
    let mut engine = Engine::new();

    // Step 1: e
    engine.on_key(keys::E, false, false);
    assert_eq!(engine.get_buffer_string(), "e");
    assert!(!engine.is_english_mode());

    // Step 2: x (TONE ngã)
    engine.on_key(keys::X, false, false);
    assert_eq!(engine.get_buffer_string(), "ẽ");
    assert!(!engine.is_english_mode(), "VN(R)=✗ but VN(B)=✓");

    // Step 3: p - 'xp' is impossible → RESTORE
    let r = engine.on_key(keys::P, false, false);
    assert_eq!(engine.get_buffer_string(), "exp");
    assert!(engine.is_english_mode(), "RESTORE triggered → EN mode");
    assert_eq!(r.action, gonhanh_core::engine::Action::Restore as u8);

    // Steps 4-6: e, c, t (in EN mode)
    engine.on_key(keys::E, false, false);
    assert_eq!(engine.get_buffer_string(), "expe");

    engine.on_key(keys::C, false, false);
    assert_eq!(engine.get_buffer_string(), "expec");

    engine.on_key(keys::T, false, false);
    assert_eq!(engine.get_buffer_string(), "expect");

    // Step 7: space
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty());
}

// =============================================================================
// EXAMPLE 6: `v a r` + space → "vả " (VN wins)
// =============================================================================

#[test]
fn test_example_6_var_vn_wins_step_by_step() {
    let mut engine = Engine::new();

    // Step 1: v
    engine.on_key(keys::V, false, false);
    assert_eq!(engine.get_buffer_string(), "v");
    assert!(!engine.is_english_mode());

    // Step 2: a
    engine.on_key(keys::A, false, false);
    assert_eq!(engine.get_buffer_string(), "va");
    assert!(!engine.is_english_mode());

    // Step 3: r (TONE hỏi) - valid Vietnamese "vả"
    engine.on_key(keys::R, false, false);
    assert_eq!(engine.get_buffer_string(), "vả");
    assert!(!engine.is_english_mode(), "VN(R)=✗ but VN(B)=✓ → VN wins");

    // Step 4: space
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty());
}

// =============================================================================
// EXAMPLE 7: `v a r r` + space → "var " (double-key REVERT)
// =============================================================================

#[test]
fn test_example_7_varr_revert_step_by_step() {
    let mut engine = Engine::new();

    // Step 1: v
    engine.on_key(keys::V, false, false);
    assert_eq!(engine.get_buffer_string(), "v");
    assert!(!engine.is_english_mode());

    // Step 2: a
    engine.on_key(keys::A, false, false);
    assert_eq!(engine.get_buffer_string(), "va");
    assert!(!engine.is_english_mode());

    // Step 3: r (TONE hỏi)
    engine.on_key(keys::R, false, false);
    assert_eq!(engine.get_buffer_string(), "vả");
    assert!(!engine.is_english_mode());

    // Step 4: r (REVERT - double-key 'rr')
    engine.on_key(keys::R, false, false);
    assert_eq!(engine.get_buffer_string(), "var");
    assert!(engine.is_english_mode(), "Double-key 'rr' → EN mode");

    // Step 5: space
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty());
}

// =============================================================================
// EXAMPLE 8: `d e n d e s n n n n` + space → "đếnnnnn "
// =============================================================================

#[test]
fn test_example_8_dennnnnn_step_by_step() {
    let mut engine = Engine::new();

    // Step 1: d
    engine.on_key(keys::D, false, false);
    assert_eq!(engine.get_buffer_string(), "d");

    // Step 2: e
    engine.on_key(keys::E, false, false);
    assert_eq!(engine.get_buffer_string(), "de");

    // Step 3: n
    engine.on_key(keys::N, false, false);
    assert_eq!(engine.get_buffer_string(), "den");

    // Step 4: d (STROKE - non-consecutive dd→đ)
    engine.on_key(keys::D, false, false);
    assert_eq!(engine.get_buffer_string(), "đen");

    // Step 5: e (CIRCUMFLEX - non-consecutive ee→ê)
    engine.on_key(keys::E, false, false);
    assert_eq!(engine.get_buffer_string(), "đên");

    // Step 6: s (TONE sắc) - 's' is modifier
    engine.on_key(keys::S, false, false);
    assert_eq!(engine.get_buffer_string(), "đến");
    assert!(!engine.is_english_mode(), "VN(R)=✗ but VN(B)=✓");

    // Steps 7-10: n n n n
    engine.on_key(keys::N, false, false);
    assert_eq!(engine.get_buffer_string(), "đếnn");

    engine.on_key(keys::N, false, false);
    assert_eq!(engine.get_buffer_string(), "đếnnn");

    engine.on_key(keys::N, false, false);
    assert_eq!(engine.get_buffer_string(), "đếnnnn");

    engine.on_key(keys::N, false, false);
    assert_eq!(engine.get_buffer_string(), "đếnnnnn");

    // Step 11: space
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty());
}

// =============================================================================
// EXAMPLE 9: `t o t o s` + space → "tốt "
// =============================================================================

#[test]
fn test_example_9_tot_step_by_step() {
    let mut engine = Engine::new();

    // Step 1: t
    engine.on_key(keys::T, false, false);
    assert_eq!(engine.get_buffer_string(), "t");

    // Step 2: o
    engine.on_key(keys::O, false, false);
    assert_eq!(engine.get_buffer_string(), "to");

    // Step 3: t
    engine.on_key(keys::T, false, false);
    assert_eq!(engine.get_buffer_string(), "tot");

    // Step 4: o (CIRCUMFLEX - non-consecutive oo→ô)
    engine.on_key(keys::O, false, false);
    assert_eq!(engine.get_buffer_string(), "tôt");

    // Step 5: s (TONE sắc)
    engine.on_key(keys::S, false, false);
    assert_eq!(engine.get_buffer_string(), "tốt");
    assert!(!engine.is_english_mode(), "VN(R)=✗ but VN(B)=✓");

    // Step 6: space
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty());
}

// DEBUG: console step-by-step
#[test]
fn debug_console() {
    let mut engine = Engine::new();

    engine.on_key(keys::C, false, false);
    eprintln!(
        "After C: '{}', en_mode={}",
        engine.get_buffer_string(),
        engine.is_english_mode()
    );

    engine.on_key(keys::O, false, false);
    eprintln!(
        "After O: '{}', en_mode={}",
        engine.get_buffer_string(),
        engine.is_english_mode()
    );

    engine.on_key(keys::N, false, false);
    eprintln!(
        "After N: '{}', en_mode={}",
        engine.get_buffer_string(),
        engine.is_english_mode()
    );

    let r = engine.on_key(keys::S, false, false);
    eprintln!(
        "After S: '{}', en_mode={}, action={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r2 = engine.on_key(keys::O, false, false);
    eprintln!(
        "After O: '{}', en_mode={}, action={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r2.action
    );
}

// DEBUG: Example 8 dennnnnn
#[test]
fn debug_example8() {
    let mut engine = Engine::new();

    let r = engine.on_key(keys::D, false, false);
    eprintln!(
        "1. d: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::E, false, false);
    eprintln!(
        "2. e: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::N, false, false);
    eprintln!(
        "3. n: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::D, false, false);
    eprintln!(
        "4. d: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::E, false, false);
    eprintln!(
        "5. e: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::S, false, false);
    eprintln!(
        "6. s: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::N, false, false);
    eprintln!(
        "7. n: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );
}

// DEBUG: Example 12 duong
#[test]
fn debug_example12() {
    let mut engine = Engine::new();

    let r = engine.on_key(keys::D, false, false);
    eprintln!(
        "1. d: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::U, false, false);
    eprintln!(
        "2. u: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::O, false, false);
    eprintln!(
        "3. o: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::W, false, false);
    eprintln!(
        "4. w: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::N, false, false);
    eprintln!(
        "5. n: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::G, false, false);
    eprintln!(
        "6. g: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::O, false, false);
    eprintln!(
        "7. o: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );

    let r = engine.on_key(keys::D, false, false);
    eprintln!(
        "8. d: '{}', en={}, act={}",
        engine.get_buffer_string(),
        engine.is_english_mode(),
        r.action
    );
}

// =============================================================================
// EXAMPLE 10: `c o n s o l e` + space → "console " (RESTORE triggered)
// =============================================================================

#[test]
fn test_example_10_console_restore_step_by_step() {
    let mut engine = Engine::new();

    // Step 1: c
    engine.on_key(keys::C, false, false);
    assert_eq!(engine.get_buffer_string(), "c");

    // Step 2: o
    engine.on_key(keys::O, false, false);
    assert_eq!(engine.get_buffer_string(), "co");

    // Step 3: n
    engine.on_key(keys::N, false, false);
    assert_eq!(engine.get_buffer_string(), "con");

    // Step 4: s (TONE sắc)
    engine.on_key(keys::S, false, false);
    assert_eq!(engine.get_buffer_string(), "cón");
    assert!(!engine.is_english_mode(), "VN(R)=✗ but VN(B)=✓");

    // Step 5: o
    engine.on_key(keys::O, false, false);
    assert_eq!(engine.get_buffer_string(), "cóno");

    // Step 6: l - 'l' is invalid Vietnamese final → RESTORE
    let r = engine.on_key(keys::L, false, false);
    assert_eq!(engine.get_buffer_string(), "consol");
    assert!(engine.is_english_mode(), "RESTORE triggered → EN mode");
    assert_eq!(r.action, gonhanh_core::engine::Action::Restore as u8);

    // Step 7: e
    engine.on_key(keys::E, false, false);
    assert_eq!(engine.get_buffer_string(), "console");

    // Step 8: space
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty());
}

// =============================================================================
// EXAMPLE 11: `d i s s t` + space → "dist " (double-key REVERT)
// =============================================================================

#[test]
fn test_example_11_dist_revert_step_by_step() {
    let mut engine = Engine::new();

    // Step 1: d
    engine.on_key(keys::D, false, false);
    assert_eq!(engine.get_buffer_string(), "d");

    // Step 2: i
    engine.on_key(keys::I, false, false);
    assert_eq!(engine.get_buffer_string(), "di");

    // Step 3: s (TONE sắc)
    engine.on_key(keys::S, false, false);
    assert_eq!(engine.get_buffer_string(), "dí");
    assert!(!engine.is_english_mode(), "VN(R)=✗ but VN(B)=✓");

    // Step 4: s (REVERT - double-key 'ss')
    engine.on_key(keys::S, false, false);
    assert_eq!(engine.get_buffer_string(), "dis");
    assert!(engine.is_english_mode(), "Double-key 'ss' → EN mode");

    // Step 5: t
    engine.on_key(keys::T, false, false);
    assert_eq!(engine.get_buffer_string(), "dist");

    // Step 6: space
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty());
}

// =============================================================================
// EXAMPLE 12: `d u o w n g o d` + space → "đuông "
// =============================================================================

#[test]
fn test_example_12_duong_stroke_step_by_step() {
    let mut engine = Engine::new();

    // Step 1: d
    engine.on_key(keys::D, false, false);
    assert_eq!(engine.get_buffer_string(), "d");

    // Step 2: u
    engine.on_key(keys::U, false, false);
    assert_eq!(engine.get_buffer_string(), "du");

    // Step 3: o
    engine.on_key(keys::O, false, false);
    assert_eq!(engine.get_buffer_string(), "duo");

    // Step 4: w (HORN) - only o→ơ, u horn is deferred (Issue #133)
    engine.on_key(keys::W, false, false);
    assert_eq!(engine.get_buffer_string(), "duơ");
    assert!(!engine.is_english_mode(), "VN(R)=✗ but VN(B)=✓");

    // Step 5: n - final consonant triggers deferred horn on u
    engine.on_key(keys::N, false, false);
    assert_eq!(engine.get_buffer_string(), "dươn");

    // Step 6: g
    engine.on_key(keys::G, false, false);
    assert_eq!(engine.get_buffer_string(), "dương");

    // Step 7: o (CIRCUMFLEX) - oo→ô overrides horn (ơ→ô, ư→u)
    engine.on_key(keys::O, false, false);
    assert_eq!(engine.get_buffer_string(), "duông");

    // Step 8: d (STROKE - non-consecutive dd→đ)
    engine.on_key(keys::D, false, false);
    assert_eq!(engine.get_buffer_string(), "đuông");

    // Step 9: space
    engine.on_key(keys::SPACE, false, false);
    assert!(engine.is_empty());
}

// =============================================================================
// VALIDATION FLOW TESTS
// =============================================================================

/// Test VN(R) validation: raw with modifier keys should be invalid
#[test]
fn test_vn_raw_validation_modifiers() {
    let modifier_keys = [keys::W, keys::J, keys::S, keys::F, keys::R, keys::X];

    for &modifier in &modifier_keys {
        let mut engine = Engine::new();
        engine.on_key(keys::A, false, false);
        engine.on_key(modifier, false, false);

        let raw = engine.get_raw_string();
        let has_mod = raw
            .chars()
            .any(|c| matches!(c, 'w' | 'j' | 's' | 'f' | 'r' | 'x'));
        assert!(has_mod, "Raw '{}' should contain modifier", raw);
    }
}

/// Test VN(R) validation: impossible clusters should be detected
#[test]
fn test_vn_raw_validation_impossible_clusters() {
    let impossible_clusters = [
        (keys::C, keys::L), // cl
        (keys::B, keys::R), // br
        (keys::S, keys::T), // st
        (keys::S, keys::P), // sp
    ];

    for (k1, k2) in impossible_clusters {
        let mut engine = Engine::new();
        engine.on_key(k1, false, false);
        engine.on_key(k2, false, false);

        assert!(
            engine.is_english_mode(),
            "Impossible cluster should trigger EN mode"
        );
    }
}

/// Test double-key revert behavior
#[test]
fn test_double_key_revert() {
    let double_keys = [
        (keys::S, "sắc"),
        (keys::F, "huyền"),
        (keys::R, "hỏi"),
        (keys::X, "ngã"),
        (keys::J, "nặng"),
    ];

    for (key, name) in double_keys {
        let mut engine = Engine::new();
        // Type vowel + tone key + same tone key (revert)
        engine.on_key(keys::A, false, false);
        engine.on_key(key, false, false); // Apply tone
        engine.on_key(key, false, false); // Revert

        assert!(
            engine.is_english_mode(),
            "Double '{}' should revert to EN mode",
            name
        );
    }
}

/// Test non-consecutive transforms
#[test]
fn test_non_consecutive_transforms() {
    // dd → đ (stroke)
    {
        let mut engine = Engine::new();
        engine.on_key(keys::D, false, false);
        engine.on_key(keys::A, false, false);
        engine.on_key(keys::D, false, false);
        assert_eq!(engine.get_buffer_string(), "đa");
    }

    // ee → ê (circumflex)
    {
        let mut engine = Engine::new();
        engine.on_key(keys::E, false, false);
        engine.on_key(keys::N, false, false);
        engine.on_key(keys::E, false, false);
        assert_eq!(engine.get_buffer_string(), "ên");
    }

    // oo → ô (circumflex)
    {
        let mut engine = Engine::new();
        engine.on_key(keys::O, false, false);
        engine.on_key(keys::N, false, false);
        engine.on_key(keys::O, false, false);
        assert_eq!(engine.get_buffer_string(), "ôn");
    }
}

/// Test that VN(B) is only checked when VN(R) fails
#[test]
fn test_vn_buffer_only_when_raw_fails() {
    // Case 1: Valid raw (no modifiers) - VN(B) not needed
    {
        let mut engine = Engine::new();
        engine.on_key(keys::D, false, false);
        engine.on_key(keys::U, false, false);
        engine.on_key(keys::O, false, false);
        // Raw is "duo", no modifiers → VN(R)=✓ → VN mode
        assert!(!engine.is_english_mode());
    }

    // Case 2: Raw with modifier, valid buffer - VN(B)=✓
    {
        let mut engine = Engine::new();
        engine.on_key(keys::D, false, false);
        engine.on_key(keys::U, false, false);
        engine.on_key(keys::O, false, false);
        engine.on_key(keys::W, false, false);
        // Raw has 'w' → VN(R)=✗ → check VN(B) → "dươ" valid → VN mode
        assert!(!engine.is_english_mode());
    }

    // Case 3: Raw with modifier, invalid buffer - VN(B)=✗ → RESTORE
    {
        let mut engine = Engine::new();
        engine.on_key(keys::T, false, false);
        engine.on_key(keys::E, false, false);
        engine.on_key(keys::X, false, false);
        engine.on_key(keys::T, false, false);
        // Raw has 'x' → VN(R)=✗ → check VN(B) → "text" invalid → EN mode
        assert!(engine.is_english_mode());
    }
}

// =============================================================================
// DEBUG: Horn application on "nguw"
// =============================================================================

#[test]
fn test_debug_nguw_horn() {
    let mut engine = Engine::new();

    // Type "nguw" step by step
    let test_keys = [
        (keys::N, "n"),
        (keys::G, "g"),
        (keys::U, "u"),
        (keys::W, "w"),
    ];

    for (i, &(key, name)) in test_keys.iter().enumerate() {
        let r = engine.on_key(key, false, false);
        let buffer = engine.get_buffer_string();
        let raw = engine.get_raw_string();

        eprintln!("Step {}: key={}", i, name);
        eprintln!("  action={}", r.action);
        eprintln!("  buffer='{}'", buffer);
        eprintln!("  raw='{}'", raw);
        eprintln!();
    }

    let final_buffer = engine.get_buffer_string();
    eprintln!("Final output: '{}'", final_buffer);
    eprintln!("Expected: 'ngư' (u with horn)");

    // Assert the correct output
    assert_eq!(
        final_buffer, "ngư",
        "Horn should be applied to 'u' after 'w'"
    );
}

#[test]
fn test_debug_ruwouj() {
    let mut engine = Engine::new();

    // Type "ruwouj" step by step - expected: "rượu"
    let test_keys = [
        (keys::R, "r"),
        (keys::U, "u"),
        (keys::W, "w"), // horn on u
        (keys::O, "o"),
        (keys::U, "u"),
        (keys::J, "j"), // nặng tone
    ];

    for (i, &(key, name)) in test_keys.iter().enumerate() {
        let r = engine.on_key(key, false, false);
        let buffer = engine.get_buffer_string();
        let raw = engine.get_raw_string();

        eprintln!("Step {}: key={}", i, name);
        eprintln!("  action={}", r.action);
        eprintln!("  buffer='{}'", buffer);
        eprintln!("  raw='{}'", raw);
        eprintln!();
    }

    let final_buffer = engine.get_buffer_string();
    eprintln!("Final output: '{}'", final_buffer);
    eprintln!("Expected: 'rượu'");

    // Assert the correct output
    assert_eq!(final_buffer, "rượu", "Should produce 'rượu'");
}

#[test]
fn test_debug_nguwowif_full_sequence() {
    let mut engine = Engine::new();

    // Type "nguwowif" step by step - expected: "người"
    let test_keys = [
        (keys::N, "n"),
        (keys::G, "g"),
        (keys::U, "u"),
        (keys::W, "w"), // horn on u
        (keys::O, "o"),
        (keys::W, "w"), // horn on o
        (keys::I, "i"),
        (keys::F, "f"), // huyền tone
    ];

    for (i, &(key, name)) in test_keys.iter().enumerate() {
        let r = engine.on_key(key, false, false);
        let buffer = engine.get_buffer_string();
        let raw = engine.get_raw_string();

        eprintln!("Step {}: key={}", i, name);
        eprintln!("  action={}", r.action);
        eprintln!("  buffer='{}'", buffer);
        eprintln!("  raw='{}'", raw);
        eprintln!();
    }

    let final_buffer = engine.get_buffer_string();
    eprintln!("Final output: '{}'", final_buffer);
    eprintln!("Expected: 'người'");

    // Assert the correct output
    assert_eq!(final_buffer, "người", "Should produce 'người'");
}
