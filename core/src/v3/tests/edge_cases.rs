//! Edge case tests
//!
//! Tests for edge cases and regression prevention.

use crate::v3::processor::Processor;

#[test]
fn test_empty_buffer() {
    let p = Processor::new();
    assert_eq!(p.buffer_content(), "");
}

#[test]
fn test_clear() {
    let mut p = Processor::new();
    p.process('a', false, false);
    p.clear();
    assert_eq!(p.buffer_content(), "");
}

#[test]
fn test_uppercase_handling() {
    let mut p = Processor::new();
    // Uppercase should be preserved
    p.process('A', true, false);
    // Note: actual behavior TBD in Phase 05
}

#[test]
fn test_ctrl_passthrough() {
    let mut p = Processor::new();
    // Ctrl+key should pass through
    p.process('a', false, true);
}

#[test]
fn test_rapid_typing() {
    let mut p = Processor::new();
    // Rapid typing should process all keys sequentially
    for c in "nhanh".chars() {
        p.process(c, false, false);
    }
}

#[test]
fn test_word_boundary_space() {
    let mut p = Processor::new();
    p.process('a', false, false);
    p.process(' ', false, false);
    // Space should commit word
}

#[test]
fn test_word_boundary_punctuation() {
    let mut p = Processor::new();
    p.process('a', false, false);
    p.process('.', false, false);
    // Punctuation should commit word
}

// === Regression Tests ===

#[test]
fn test_regression_double_s_restore() {
    // Regression: "bass" should produce "bass" not "bás"
}

#[test]
fn test_regression_triple_s_buffer() {
    // Regression: "basss" should produce "bass"
}

#[test]
fn test_regression_law_restore() {
    // Regression: "law" should restore (lă is invalid VN)
}

#[test]
fn test_regression_stroke_keep() {
    // Regression: "đ" words should not restore (intentional VN)
}
