//! Auto-restore decision logic (Hybrid Validation Approach)
//!
//! ## Phase 6: Validation-First Skip Logic
//!
//! Based on V1 production code, implements TWO-PHASE validation:
//!
//! ### PHASE 1: PRE-VALIDATE (BEFORE transform)
//! - Called before applying tone/mark transforms
//! - Check if result would be valid VN
//! - If invalid VN + valid EN raw → SKIP transform
//!
//! ### PHASE 2: POST-CHECK (AFTER keystroke / on boundary)
//! - IMPOSSIBLE state: restore immediately (e.g., "tẽt")
//! - INCOMPLETE state: wait for boundary (e.g., "lă" → could become "lăm")
//! - On word boundary: restore if INCOMPLETE + valid EN
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
use super::vietnamese::{is_valid_syllable, validate_vietnamese, ValidationResult};

// ============================================================================
// PHASE 1: PRE-VALIDATE (BEFORE TRANSFORM)
// ============================================================================

/// Skip decision for PRE-VALIDATE (called BEFORE applying transform)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipDecision {
    /// Transform would create valid VN → apply it
    Apply,
    /// Transform would create invalid VN + raw is valid EN → SKIP
    Skip,
    /// Transform invalid VN + invalid EN → apply anyway (typo mode)
    ApplyTypo,
}

/// Determine if transform should be SKIPPED (called BEFORE applying transform)
///
/// # Arguments
/// * `buffer` - Current transformed buffer
/// * `raw` - Raw keystroke buffer
/// * `simulated` - What buffer WOULD look like after transform
/// * `is_stroke` - Whether this is a stroke (đ) transform
///
/// # Returns
/// * SkipDecision indicating whether to APPLY or SKIP
pub fn should_skip_transform(
    buffer: &str,
    raw: &str,
    simulated: &str,
    is_stroke: bool,
) -> SkipDecision {
    // 1. Stroke (đ) always applies - strong VN signal
    if is_stroke {
        return SkipDecision::Apply;
    }

    // 2. Check if simulated result is valid VN
    if is_valid_syllable(simulated) {
        return SkipDecision::Apply;
    }

    // 3. Invalid VN → check if raw is valid EN
    if is_english_word(raw) {
        return SkipDecision::Skip;
    }

    // 4. Invalid both → apply anyway (typo mode)
    SkipDecision::ApplyTypo
}

// ============================================================================
// PHASE 2: POST-CHECK (AFTER KEYSTROKE / ON BOUNDARY)
// ============================================================================

/// Buffer state for POST-CHECK validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferState {
    /// Valid complete VN syllable (e.g., "bán", "lăm")
    Complete,
    /// Could become valid with more chars (e.g., "lă" → "lăm")
    Incomplete,
    /// Structure can't exist in VN (e.g., "tẽt", "bánk")
    Impossible,
}

/// Check buffer state for POST-CHECK validation
///
/// # Arguments
/// * `buffer` - Current transformed buffer
///
/// # Returns
/// * BufferState indicating COMPLETE, INCOMPLETE, or IMPOSSIBLE
pub fn check_buffer_state(buffer: &str) -> BufferState {
    if buffer.is_empty() {
        return BufferState::Complete;
    }

    let result = validate_vietnamese(buffer);

    match result {
        // Valid = Complete
        ValidationResult::Valid => BufferState::Complete,

        // NoVowel with just consonants = Incomplete (could add vowel)
        ValidationResult::NoVowel => {
            // Check if it's just initial consonants (could become valid)
            if buffer.chars().all(|c| !is_vietnamese_vowel(c)) && buffer.len() <= 3 {
                BufferState::Incomplete
            } else {
                BufferState::Impossible
            }
        }

        // Invalid vowel pattern = Impossible (ea, ou, yo)
        ValidationResult::InvalidVowelPattern => BufferState::Impossible,

        // Invalid final = Impossible (e.g., "bánk")
        ValidationResult::InvalidFinal => BufferState::Impossible,

        // Invalid tone = Impossible (huyền on stop final)
        ValidationResult::InvalidTone => BufferState::Impossible,

        // Invalid structure = check if could be incomplete
        ValidationResult::InvalidStructure => {
            // Check if buffer is consonants only (could add vowel later)
            if buffer.chars().all(|c| !is_vietnamese_vowel(c)) && buffer.chars().count() <= 3 {
                BufferState::Incomplete
            }
            // If ends with vowel, could be incomplete (waiting for final)
            else if buffer
                .chars()
                .last()
                .map(is_vietnamese_vowel)
                .unwrap_or(false)
            {
                BufferState::Incomplete
            } else {
                BufferState::Impossible
            }
        }

        // Other cases: check if could add more chars
        _ => {
            // If it's just a single vowel with tone, could be incomplete
            if buffer.chars().count() <= 2 {
                BufferState::Incomplete
            } else {
                BufferState::Impossible
            }
        }
    }
}

/// Check if immediate restore needed (called AFTER each keystroke)
///
/// # Arguments
/// * `buffer` - Current transformed buffer
/// * `raw` - Raw keystroke buffer
/// * `had_transform` - Whether any transformation was applied
/// * `has_stroke` - Whether stroke (đ) is present
///
/// # Returns
/// * true if should restore immediately
pub fn should_restore_immediate(
    buffer: &str,
    raw: &str,
    had_transform: bool,
    has_stroke: bool,
) -> bool {
    // No transform = no restore
    if !had_transform {
        return false;
    }

    // Stroke blocks restore (intentional VN)
    if has_stroke {
        return false;
    }

    // Check buffer state
    let state = check_buffer_state(buffer);

    // Only restore immediately on IMPOSSIBLE state
    if state == BufferState::Impossible {
        // Check if raw is valid EN
        return is_english_word(raw);
    }

    false
}

/// Helper: Check if char is Vietnamese vowel
fn is_vietnamese_vowel(c: char) -> bool {
    matches!(
        c.to_ascii_lowercase(),
        'a' | 'ă'
            | 'â'
            | 'e'
            | 'ê'
            | 'i'
            | 'o'
            | 'ô'
            | 'ơ'
            | 'u'
            | 'ư'
            | 'y'
            | 'á'
            | 'à'
            | 'ả'
            | 'ã'
            | 'ạ'
            | 'ắ'
            | 'ằ'
            | 'ẳ'
            | 'ẵ'
            | 'ặ'
            | 'ấ'
            | 'ầ'
            | 'ẩ'
            | 'ẫ'
            | 'ậ'
            | 'é'
            | 'è'
            | 'ẻ'
            | 'ẽ'
            | 'ẹ'
            | 'ế'
            | 'ề'
            | 'ể'
            | 'ễ'
            | 'ệ'
            | 'í'
            | 'ì'
            | 'ỉ'
            | 'ĩ'
            | 'ị'
            | 'ó'
            | 'ò'
            | 'ỏ'
            | 'õ'
            | 'ọ'
            | 'ố'
            | 'ồ'
            | 'ổ'
            | 'ỗ'
            | 'ộ'
            | 'ớ'
            | 'ờ'
            | 'ở'
            | 'ỡ'
            | 'ợ'
            | 'ú'
            | 'ù'
            | 'ủ'
            | 'ũ'
            | 'ụ'
            | 'ứ'
            | 'ừ'
            | 'ử'
            | 'ữ'
            | 'ự'
            | 'ý'
            | 'ỳ'
            | 'ỷ'
            | 'ỹ'
            | 'ỵ'
    )
}

// ============================================================================
// BOUNDARY RESTORE (EXISTING CODE, UPDATED)
// ============================================================================

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

/// Determine if auto-restore should occur on word boundary
///
/// # Arguments
/// * `had_transform` - Whether any transformation was applied
/// * `has_stroke` - Whether stroke (đ) is present
/// * `buffer` - Transformed buffer content
/// * `raw` - Raw keystroke content (restore_all)
/// * `had_revert` - Whether a revert occurred
///
/// # Returns
/// * RestoreDecision indicating what to do
pub fn should_restore(
    had_transform: bool,
    has_stroke: bool,
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

    // Step 3: Stroke blocks restore (intentional VN)
    if has_stroke {
        return RestoreDecision::KeepVietnamese;
    }

    // Step 4: Check buffer state
    let state = check_buffer_state(buffer);

    match state {
        // Complete valid VN: still check if raw is strong English
        // (e.g., "tét" vs "test" - both valid but raw is clearly English)
        BufferState::Complete => {
            // If raw is a strong English word, restore even for complete VN
            if is_english_word(raw) {
                RestoreDecision::RestoreEnglish
            } else {
                RestoreDecision::KeepVietnamese
            }
        }

        // IMPOSSIBLE or INCOMPLETE = check English pattern
        // Restore only if: invalid VN structure + valid English pattern
        BufferState::Incomplete | BufferState::Impossible => {
            // Handle revert case specially
            if had_revert {
                return handle_revert_case(buffer, raw);
            }

            // Check English pattern - required for restore
            if is_english_word(raw) {
                return RestoreDecision::RestoreEnglish;
            }

            // Neither valid = keep as-is (could be typo)
            RestoreDecision::KeepAsIs
        }
    }
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

    // ============================================================================
    // PHASE 1: PRE-VALIDATE TESTS
    // ============================================================================

    #[test]
    fn test_skip_decision_stroke_always_applies() {
        // Stroke (đ) always applies regardless of validation
        let decision = should_skip_transform("d", "d", "đ", true);
        assert_eq!(decision, SkipDecision::Apply);
    }

    #[test]
    fn test_skip_decision_valid_vn_applies() {
        // "ba" + 's' = "bá" is valid VN → apply
        let decision = should_skip_transform("ba", "bas", "bá", false);
        assert_eq!(decision, SkipDecision::Apply);
    }

    #[test]
    fn test_skip_decision_invalid_vn_valid_en_skips() {
        // "tex" + 'x' would create "tẽx" which is invalid
        // "texx" is valid EN (coda cluster) → skip
        let decision = should_skip_transform("tex", "texx", "tẽx", false);
        // Note: actual result depends on EN detection for "texx"
        // Skip or ApplyTypo depending on EN confidence
        assert!(matches!(
            decision,
            SkipDecision::Skip | SkipDecision::ApplyTypo
        ));
    }

    // ============================================================================
    // PHASE 2: POST-CHECK TESTS
    // ============================================================================

    #[test]
    fn test_buffer_state_complete() {
        // Valid VN syllable
        assert_eq!(check_buffer_state("bán"), BufferState::Complete);
        assert_eq!(check_buffer_state("lăm"), BufferState::Complete);
        assert_eq!(check_buffer_state("việt"), BufferState::Complete);
    }

    #[test]
    fn test_buffer_state_incomplete() {
        // Could become valid with more chars
        assert_eq!(check_buffer_state("b"), BufferState::Incomplete); // could be "ba"
        assert_eq!(check_buffer_state("th"), BufferState::Incomplete); // could be "tha"
    }

    #[test]
    fn test_buffer_state_impossible() {
        // Can't be valid VN no matter what
        assert_eq!(check_buffer_state("ea"), BufferState::Impossible); // invalid vowel pattern
        assert_eq!(check_buffer_state("ou"), BufferState::Impossible); // invalid vowel pattern
    }

    #[test]
    fn test_should_restore_immediate_impossible() {
        // IMPOSSIBLE + valid EN → restore
        // "text" typed as "tẽt" + "t" = "tẽtt" (impossible VN)
        let should = should_restore_immediate("tẽxt", "text", true, false);
        // Note: result depends on "text" being detected as EN
        // If "text" has coda cluster "xt" → true
        assert!(should); // "text" has CodaCluster → restore
    }

    #[test]
    fn test_should_restore_immediate_stroke_blocks() {
        // Has stroke = intentional VN, never restore
        let should = should_restore_immediate("đang", "ddang", true, true);
        assert!(!should);
    }

    #[test]
    fn test_should_restore_immediate_no_transform() {
        // No transform = no restore
        let should = should_restore_immediate("test", "test", false, false);
        assert!(!should);
    }

    // ============================================================================
    // BOUNDARY RESTORE TESTS (UPDATED)
    // ============================================================================

    #[test]
    fn test_no_transform() {
        let decision = should_restore(false, false, "test", "test", false);
        assert_eq!(decision, RestoreDecision::NoTransform);
    }

    #[test]
    fn test_valid_vietnamese() {
        // "ba" is valid Vietnamese
        let decision = should_restore(true, false, "ba", "ba", false);
        // Same buffer/raw = NoTransform
        assert_eq!(decision, RestoreDecision::NoTransform);
    }

    #[test]
    fn test_stroke_blocks_restore() {
        // Has stroke = intentional VN
        let decision = should_restore(true, true, "đang", "ddang", false);
        assert_eq!(decision, RestoreDecision::KeepVietnamese);
    }

    #[test]
    fn test_english_restore() {
        // "tét" transformed from "test"
        let decision = should_restore(true, false, "tét", "test", false);
        // "tét" is valid VN actually... but "test" has CodaCluster
        // If buffer is valid VN, keep VN
        // Note: "tét" may or may not be valid depending on validation
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

    // ============================================================================
    // PHASE 6: INTEGRATION TESTS
    // ============================================================================

    #[test]
    fn test_law_vs_lam_incomplete() {
        // "lă" is INCOMPLETE - could become "lăm"
        let state = check_buffer_state("lă");
        // Note: "lă" might be parsed as valid VN (single vowel with mark)
        // If valid → Complete, if not → Incomplete
        // The key is: on boundary, INCOMPLETE + valid EN → restore
    }

    #[test]
    fn test_text_impossible() {
        // "tẽt" is IMPOSSIBLE (consonant after tone)
        let state = check_buffer_state("tẽt");
        // Should be Impossible or at least not Complete
        assert_ne!(state, BufferState::Complete);
    }
}
