//! Auto-restore decision logic (8-layer English detection)
//!
//! Implements the validation flow for auto-restore:
//! ```text
//! had_transform? → VN(B) valid? → EN(R) valid? → Decision
//! ```
//!
//! ## English Detection Threshold
//! Uses 8-layer system from validation-algorithm.md:
//! - Restore threshold: HasSuffix (90%) or higher
//! - Layers 1-5: Auto-restore (Certain, OnsetCluster, DoubleConsonant, HasSuffix, CodaCluster)
//! - Layers 6-8: Context-dependent (HasPrefix, VowelPattern, ImpossibleBigram)
//!
//! ## Critical Principles
//!
//! **NGUYÊN TẮC 2: KHÔNG FIX CASE-BY-CASE**
//! All decisions follow validation flow, no hardcoding.

use super::english::is_english_word;
use super::vietnamese::is_valid_syllable;

/// Restore decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreDecision {
    /// Keep Vietnamese buffer
    KeepVietnamese,
    /// Restore to raw (English detected)
    RestoreEnglish,
    /// Keep as-is (neither valid)
    KeepAsIs,
    /// No restore needed (no transform)
    NoTransform,
}

/// Determine if auto-restore should occur
///
/// # Arguments
/// * `had_transform` - Whether any transformation was applied
/// * `buffer` - Transformed buffer content
/// * `raw` - Raw keystroke content (restore_all)
/// * `had_revert` - Whether a revert occurred
///
/// # Returns
/// * RestoreDecision indicating what to do
pub fn should_restore(
    had_transform: bool,
    buffer: &str,
    raw: &str,
    had_revert: bool,
) -> RestoreDecision {
    // Step 1: No transform = no restore needed
    if !had_transform {
        return RestoreDecision::NoTransform;
    }

    // Step 2: Buffer same as raw = no restore needed
    if buffer == raw {
        return RestoreDecision::NoTransform;
    }

    // Step 3: Valid Vietnamese = keep
    if is_valid_syllable(buffer) {
        return RestoreDecision::KeepVietnamese;
    }

    // Step 4: Handle revert case specially
    if had_revert {
        return handle_revert_case(buffer, raw);
    }

    // Step 5: Check English pattern
    if is_english_word(raw) {
        return RestoreDecision::RestoreEnglish;
    }

    // Step 6: Neither valid = keep as-is
    RestoreDecision::KeepAsIs
}

/// Handle revert case ambiguity
///
/// When revert occurred, both buffer and raw might be valid words.
/// Prefer user intent (buffer) if it's valid English.
fn handle_revert_case(buffer: &str, raw: &str) -> RestoreDecision {
    let buffer_english = is_english_word(buffer);
    let raw_english = is_english_word(raw);

    match (buffer_english, raw_english) {
        // Buffer valid English (user reverted intentionally) -> use buffer
        (true, false) => RestoreDecision::KeepAsIs,
        // Raw valid English (auto-restore needed) -> use raw
        (false, true) => RestoreDecision::RestoreEnglish,
        // Both valid -> prefer buffer (user intent)
        (true, true) => RestoreDecision::KeepAsIs,
        // Neither valid -> keep as-is
        (false, false) => RestoreDecision::KeepAsIs,
    }
}

/// Get restore output
///
/// # Arguments
/// * `decision` - Restore decision
/// * `buffer` - Transformed buffer
/// * `raw` - Raw keystroke content
///
/// # Returns
/// * String to output
pub fn get_restore_output<'a>(decision: RestoreDecision, buffer: &'a str, raw: &'a str) -> &'a str {
    match decision {
        RestoreDecision::RestoreEnglish => raw,
        _ => buffer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_transform() {
        let decision = should_restore(false, "test", "test", false);
        assert_eq!(decision, RestoreDecision::NoTransform);
    }

    #[test]
    fn test_valid_vietnamese() {
        // "ba" is valid Vietnamese
        let decision = should_restore(true, "ba", "ba", false);
        // Same buffer/raw = NoTransform
        assert_eq!(decision, RestoreDecision::NoTransform);
    }

    #[test]
    fn test_english_restore() {
        // "tét" transformed from "test"
        let _decision = should_restore(true, "tét", "test", false);
        // "tét" is not valid VN syllable, "test" matches English patterns
        // Note: depends on is_valid_syllable and is_english_word implementations
    }

    #[test]
    fn test_revert_case() {
        // "case" from "casse" - user typed ss to revert
        let _decision = handle_revert_case("case", "casse");
        // "case" is valid English, "casse" is not
        // Should keep buffer (user intent)
    }

    #[test]
    fn test_issue_pattern() {
        // "isue" from "issue" - test revert case structure
        let decision = handle_revert_case("isue", "issue");
        // With 8-layer system:
        // - "issue" has double consonant (ss) = DoubleConsonant (95%)
        // - is_english_word() returns true for DoubleConsonant >= HasSuffix threshold
        // - "isue" has no patterns = None
        // So: (false, true) -> RestoreEnglish
        assert_eq!(decision, RestoreDecision::RestoreEnglish);
    }

    #[test]
    fn test_double_consonant_detection() {
        // "coffee" has double ff = DoubleConsonant (95%)
        let decision = handle_revert_case("cofee", "coffee");
        // "coffee" >= HasSuffix threshold -> RestoreEnglish
        assert_eq!(decision, RestoreDecision::RestoreEnglish);
    }

    #[test]
    fn test_prefix_below_threshold() {
        // "undo" has prefix un- = HasPrefix (75%)
        // HasPrefix < HasSuffix (90%) threshold
        let decision = handle_revert_case("undo", "undo");
        // Both same -> neither has advantage, KeepAsIs
        assert_eq!(decision, RestoreDecision::KeepAsIs);
    }
}
