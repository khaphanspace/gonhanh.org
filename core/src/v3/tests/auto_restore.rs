//! Auto-restore tests
//!
//! Tests for auto-restore decision logic.
//!
//! ## Critical Test Cases
//!
//! 1. "issue" vs "isssue" - both produce "issue"
//! 2. "casse" vs "case" - different intent
//! 3. Double modifier patterns
//! 4. Triple key patterns
//!
//! NOTE: Some tests depend on vietnamese validation which is stub in Phase 01.
//! Full tests will pass after Phase 03 (VN validation) and Phase 04 (EN detection).

use crate::v3::validation::restore::{should_restore, RestoreDecision};

#[test]
fn test_no_transform_no_restore() {
    let decision = should_restore(false, "test", "test", false);
    assert_eq!(decision, RestoreDecision::NoTransform);
}

#[test]
fn test_same_buffer_raw_no_restore() {
    let decision = should_restore(true, "test", "test", false);
    assert_eq!(decision, RestoreDecision::NoTransform);
}

#[test]
fn test_validation_flow_same() {
    // Same buffer/raw = no transform
    let decision = should_restore(true, "an", "an", false);
    assert_eq!(decision, RestoreDecision::NoTransform);
}

// === Tests that depend on full VN/EN validation (Phase 03-04) ===
// Currently using stub validation, so these test the flow structure

#[test]
fn test_revert_case_structure() {
    // Test revert case handling - actual result depends on validation impl
    // "isue" vs "issue" - tests the revert case branch
    let _decision = should_restore(true, "isue", "issue", true);
    // Flow: had_revert=true -> handle_revert_case()
    // Actual assertion depends on is_english_word implementation
}

#[test]
fn test_no_revert_case_structure() {
    // Test non-revert case - actual result depends on validation impl
    let _decision = should_restore(true, "tét", "test", false);
    // Flow: had_revert=false -> check VN -> check EN
}

// === Placeholder tests for Phase 03-04 ===
// TODO: Enable after Phase 03-04 complete

/*
#[test]
fn test_issue_restore() {
    let decision = should_restore(true, "isue", "issue", true);
    assert_eq!(decision, RestoreDecision::RestoreEnglish);
}

#[test]
fn test_casse_keep() {
    let decision = should_restore(true, "case", "casse", true);
    assert_eq!(decision, RestoreDecision::KeepAsIs);
}
*/
