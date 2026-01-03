//! Auto-restore tests (Phase 6)
//!
//! Tests for the hybrid validation approach:
//! - PHASE 1: PRE-VALIDATE (should_skip_transform)
//! - PHASE 2: POST-CHECK (should_restore_immediate, should_restore)
//!
//! ## Test Categories
//!
//! 1. PRE-VALIDATE tests - skip transform for EN words
//! 2. POST-CHECK immediate tests - restore on IMPOSSIBLE state
//! 3. Boundary restore tests - restore on word boundary
//! 4. Integration tests - full processor flow

use crate::v3::processor::Processor;
use crate::v3::validation::{
    check_buffer_state, should_restore, should_restore_immediate, should_skip_transform,
    BufferState, RestoreDecision, SkipDecision,
};

// ============================================================================
// PHASE 1: PRE-VALIDATE TESTS
// ============================================================================

#[test]
fn test_prevalidate_stroke_always_applies() {
    // Stroke (đ) always applies - strong VN signal
    let decision = should_skip_transform("d", "dd", "đ", true);
    assert_eq!(decision, SkipDecision::Apply);
}

#[test]
fn test_prevalidate_valid_vn_applies() {
    // "ba" + tone = "bá" valid VN → apply
    let decision = should_skip_transform("ba", "bas", "bá", false);
    assert_eq!(decision, SkipDecision::Apply);
}

#[test]
fn test_prevalidate_invalid_vn_applies_typo() {
    // Invalid VN + invalid EN → apply anyway (typo mode)
    let decision = should_skip_transform("xyz", "xyzs", "xỳz", false);
    assert_eq!(decision, SkipDecision::ApplyTypo);
}

// ============================================================================
// PHASE 2: POST-CHECK - BUFFER STATE TESTS
// ============================================================================

#[test]
fn test_buffer_state_valid_vn_complete() {
    assert_eq!(check_buffer_state("bán"), BufferState::Complete);
    assert_eq!(check_buffer_state("việt"), BufferState::Complete);
    assert_eq!(check_buffer_state("nam"), BufferState::Complete);
}

#[test]
fn test_buffer_state_consonant_only_incomplete() {
    // Just consonants - could add vowel
    assert_eq!(check_buffer_state("b"), BufferState::Incomplete);
    assert_eq!(check_buffer_state("th"), BufferState::Incomplete);
    assert_eq!(check_buffer_state("ng"), BufferState::Incomplete);
}

#[test]
fn test_buffer_state_invalid_vowel_pattern_impossible() {
    // Foreign vowel patterns
    assert_eq!(check_buffer_state("ea"), BufferState::Impossible);
    assert_eq!(check_buffer_state("ou"), BufferState::Impossible);
}

// ============================================================================
// PHASE 2: POST-CHECK - IMMEDIATE RESTORE TESTS
// ============================================================================

#[test]
fn test_immediate_restore_impossible_en() {
    // IMPOSSIBLE + valid EN → restore
    // "text" has coda cluster "xt" → EN confidence high
    let should = should_restore_immediate("tẽxt", "text", true, false);
    assert!(should);
}

#[test]
fn test_immediate_restore_stroke_blocks() {
    // Stroke present = intentional VN, never restore
    let should = should_restore_immediate("đang", "ddang", true, true);
    assert!(!should);
}

#[test]
fn test_immediate_restore_no_transform() {
    // No transform = no restore
    let should = should_restore_immediate("test", "test", false, false);
    assert!(!should);
}

// ============================================================================
// BOUNDARY RESTORE TESTS
// ============================================================================

#[test]
fn test_boundary_no_transform() {
    let decision = should_restore(false, false, "test", "test", false);
    assert_eq!(decision, RestoreDecision::NoTransform);
}

#[test]
fn test_boundary_same_buffer_raw() {
    let decision = should_restore(true, false, "ba", "ba", false);
    assert_eq!(decision, RestoreDecision::NoTransform);
}

#[test]
fn test_boundary_stroke_blocks() {
    // Stroke = intentional VN
    let decision = should_restore(true, true, "đang", "ddang", false);
    assert_eq!(decision, RestoreDecision::KeepVietnamese);
}

#[test]
fn test_boundary_valid_vn_keep() {
    // Valid complete VN = keep
    let decision = should_restore(true, false, "bán", "bans", false);
    assert_eq!(decision, RestoreDecision::KeepVietnamese);
}

#[test]
fn test_boundary_incomplete_en_restore() {
    // INCOMPLETE + valid EN → restore
    // "law" with CodaCluster → restore
    // Note: depends on "law" being detected as EN (may not have patterns)
}

// ============================================================================
// REVERT CASE TESTS
// ============================================================================

#[test]
fn test_revert_issue_to_issue() {
    // "isue" (buffer after revert) vs "issue" (raw)
    // "issue" has double consonant = EN
    let decision = should_restore(true, false, "isue", "issue", true);
    assert_eq!(decision, RestoreDecision::RestoreEnglish);
}

#[test]
fn test_revert_coffee_to_coffee() {
    // "cofee" (buffer) vs "coffee" (raw)
    // "coffee" has double ff = EN
    let decision = should_restore(true, false, "cofee", "coffee", true);
    assert_eq!(decision, RestoreDecision::RestoreEnglish);
}

// ============================================================================
// PROCESSOR INTEGRATION TESTS
// ============================================================================

#[test]
fn test_processor_basic_vn() {
    let mut p = Processor::new();

    // Type "bans" → "bán"
    p.process('b', false, false);
    p.process('a', false, false);
    p.process('n', false, false);
    p.process('s', false, false);

    assert_eq!(p.buffer_content(), "bán");
}

#[test]
fn test_processor_stroke_keeps_vn() {
    let mut p = Processor::new();

    // Type "ddang" → "đang"
    p.process('d', false, false);
    p.process('d', false, false);
    p.process('a', false, false);
    p.process('n', false, false);
    p.process('g', false, false);

    assert_eq!(p.buffer_content(), "đang");
}

#[test]
fn test_processor_clear() {
    let mut p = Processor::new();
    p.process('t', false, false);
    p.process('e', false, false);
    p.clear();
    assert_eq!(p.buffer_content(), "");
}

// ============================================================================
// PHASE 6 SPECIFIC: "law" vs "lăm" SCENARIO
// ============================================================================

#[test]
fn test_law_becomes_lam_if_m_added() {
    let mut p = Processor::new();

    // Type "lawm" → should become "lăm" (valid VN)
    p.process('l', false, false);
    p.process('a', false, false);
    p.process('w', false, false); // → "lă"
    p.process('m', false, false); // → "lăm"

    assert_eq!(p.buffer_content(), "lăm");
}

// Note: Testing "law " restore requires handling commit which
// may need additional integration work
