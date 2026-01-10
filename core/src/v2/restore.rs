//! Step 6: Restore Decision
//!
//! Determines whether buffer should be restored to raw input
//! based on 8-phase decision flow (Spec Section 9).

use super::dict::Dict;
use super::state::{BufferState, VnState};

/// Restore decision result
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Decision {
    /// Keep transformed buffer
    Keep,
    /// Restore to raw input
    Restore,
    /// Skip (don't guess, wait for more input)
    Skip,
}

/// Determine if buffer should be restored to raw
///
/// New approach: Dictionary-based restore only
/// - Only restore when raw is 100% match in English dictionary
/// - AND transformed is invalid Vietnamese
///
/// # Arguments
/// * `state` - Current buffer state flags
/// * `raw` - Raw input string
/// * `buffer` - Transformed buffer string
/// * `dict` - Optional English dictionary for word lookup
///
/// # Returns
/// * `Decision` - Keep, Restore, or Skip
pub fn should_restore(
    state: &BufferState,
    raw: &str,
    _buffer: &str,
    dict: Option<&Dict>,
) -> Decision {
    // P1: No transform = nothing to restore
    if !state.had_transform() {
        return Decision::Keep;
    }

    // P2: Stroke (đ) = 100% intentional VN
    // This is the strongest signal - user pressed 'dd' for đ
    if state.has_stroke() {
        return Decision::Keep;
    }

    // P3: Dictionary-based restore (PRIMARY METHOD)
    // If raw is in English dictionary, restore it to raw
    // This takes priority over the "has tone = Vietnamese" heuristic
    // because tones often get added accidentally (e.g., "test" → "tét" via 's' key)
    if let Some(dict) = dict {
        if dict.contains(raw) {
            return Decision::Restore;
        }
    }

    // P4: Complete VN with tone = intentional VN (when no dict match)
    // If user typed a word with tone that's NOT in English dict,
    // they're likely typing Vietnamese
    if state.has_tone() && state.vn_state() == VnState::Complete {
        return Decision::Keep;
    }

    // P5: Valid Vietnamese without tone = keep
    if state.vn_state() == VnState::Complete {
        return Decision::Keep;
    }

    // P6: Impossible VN = restore
    if state.vn_state() == VnState::Impossible {
        return Decision::Restore;
    }

    // P7: Otherwise = skip (incomplete word, don't guess)
    Decision::Skip
}

/// Fallback logic when no dictionary available
fn should_restore_fallback(state: &BufferState, raw: &str, buffer: &str) -> Decision {
    // P6: Complete + tone = intentional VN (higher priority)
    // "tiếng" with tone mark is clearly intentional Vietnamese
    if state.has_tone() && state.vn_state() == VnState::Complete {
        return Decision::Keep;
    }

    // P7: Complete = valid VN (even without tone)
    if state.vn_state() == VnState::Complete {
        return Decision::Keep;
    }

    // P5: Significant char consumption = restore
    // Only applies to incomplete/unknown patterns
    let raw_len = raw.chars().count() as i32;
    let buf_len = buffer.chars().count() as i32;
    let consumed = raw_len - buf_len;
    if consumed >= 2 {
        return Decision::Restore;
    }

    // P8: Otherwise = skip (don't guess)
    Decision::Skip
}

/// Check for English patterns in raw buffer (Tiers 3-7)
pub fn is_english(raw: &str) -> bool {
    tier3_coda_cluster(raw)
        || tier4_vowel_pattern(raw)
        || tier5_suffix(raw)
        || tier6_vcv_pattern(raw)
        || tier7_w_as_vowel(raw)
}

/// Tier 3: English-only coda clusters
/// From spec: ct, ft, ld, lf, lk, lm, lp, lt, nd, nk, nt, pt, rb, rd, rk, rm, rn, rp, rt, sk, sp, st, sh, xt
fn tier3_coda_cluster(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    if bytes.len() < 2 {
        return false;
    }

    // Check last 2 chars for EN-only coda clusters
    let last2 = &bytes[bytes.len() - 2..];
    const EN_CODA: &[[u8; 2]] = &[
        *b"ct", *b"ft", *b"ld", *b"lf", *b"lk", *b"lm", *b"lp", *b"lt", *b"nd", *b"nk", *b"nt",
        *b"pt", *b"rb", *b"rd", *b"rk", *b"rm", *b"rn", *b"rp", *b"rt", *b"sk", *b"sp", *b"st",
        *b"sh", *b"xt",
    ];
    EN_CODA.iter().any(|p| last2 == p)
}

/// Tier 4: English-only vowel patterns
/// From spec: ea, ee, ou, ei, eu, yo, ae, yi, oo, io
/// Note: "eu" excluded - can appear in VN transforms (e.g., "châu" → intermediate "chauu")
/// Note: "io" excluded - appears in VN diphthongs (e.g., "bịo")
/// Note: "ee" excluded - Telex modifier for ê (e.g., "tiếng" → raw "tieengs")
/// Note: "oo" excluded - Telex modifier for ô (e.g., "tôi" → raw "tooi")
/// Note: "aa" would also need exclusion (Telex for â), but not in original list
fn tier4_vowel_pattern(raw: &str) -> bool {
    const EN_VOWELS: &[&str] = &["ea", "ou", "ei", "yo", "ae", "yi"];
    let lower = raw.to_ascii_lowercase();
    EN_VOWELS.iter().any(|p| lower.contains(p))
}

/// Tier 5: English suffixes
/// From spec: tion, sion, ness, ment, able, ible, ing, ful, ous, ive
fn tier5_suffix(raw: &str) -> bool {
    const SUFFIXES: &[&str] = &[
        "tion", "sion", "ness", "ment", "able", "ible", "ing", "ful", "ous", "ive",
    ];
    let lower = raw.to_ascii_lowercase();
    SUFFIXES.iter().any(|s| lower.ends_with(s))
}

/// Tier 6: VCV pattern (V₁ + consonant + V₂)
/// From spec: ore, are, ase, ile, ure, ife, ose, use, ory, ary, ery
fn tier6_vcv_pattern(raw: &str) -> bool {
    const VCV: &[&str] = &[
        "ore", "are", "ase", "ile", "ure", "ife", "ose", "use", "ory", "ary", "ery",
    ];
    let lower = raw.to_ascii_lowercase();
    VCV.iter().any(|p| lower.contains(p))
}

/// Tier 7: W as vowel
/// From spec: ew, ow, aw, iew
fn tier7_w_as_vowel(raw: &str) -> bool {
    const W_VOWEL: &[&str] = &["ew", "ow", "aw", "iew"];
    let lower = raw.to_ascii_lowercase();
    W_VOWEL.iter().any(|p| lower.ends_with(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tier 3 tests
    #[test]
    fn test_tier3_coda_cluster() {
        assert!(tier3_coda_cluster("text")); // xt
        assert!(tier3_coda_cluster("push")); // sh
        assert!(tier3_coda_cluster("first")); // st
        assert!(tier3_coda_cluster("child")); // ld
        assert!(tier3_coda_cluster("self")); // lf
        assert!(!tier3_coda_cluster("ban")); // n is VN coda
        assert!(!tier3_coda_cluster("hoc")); // c is VN coda
    }

    #[test]
    fn test_tier3_uppercase() {
        assert!(tier3_coda_cluster("TEXT"));
        assert!(tier3_coda_cluster("PUSH"));
    }

    // Tier 4 tests
    #[test]
    fn test_tier4_vowel_pattern() {
        assert!(tier4_vowel_pattern("search")); // ea
        assert!(!tier4_vowel_pattern("see")); // ee excluded (Telex modifier)
        assert!(tier4_vowel_pattern("you")); // ou
        assert!(tier4_vowel_pattern("ceiling")); // ei
        assert!(!tier4_vowel_pattern("ban")); // no EN pattern
        assert!(!tier4_vowel_pattern("toi")); // oi is VN
        assert!(!tier4_vowel_pattern("too")); // oo excluded (Telex modifier)
    }

    // Tier 5 tests
    #[test]
    fn test_tier5_suffix() {
        assert!(tier5_suffix("action")); // tion
        assert!(tier5_suffix("happiness")); // ness
        assert!(tier5_suffix("movement")); // ment
        assert!(tier5_suffix("running")); // ing
        assert!(tier5_suffix("careful")); // ful
        assert!(!tier5_suffix("ban")); // no EN suffix
    }

    // Tier 6 tests
    #[test]
    fn test_tier6_vcv_pattern() {
        assert!(tier6_vcv_pattern("more")); // ore
        assert!(tier6_vcv_pattern("care")); // are
        assert!(tier6_vcv_pattern("file")); // ile
        assert!(tier6_vcv_pattern("sure")); // ure
        assert!(tier6_vcv_pattern("life")); // ife
        assert!(!tier6_vcv_pattern("ban")); // no VCV pattern
    }

    // Tier 7 tests
    #[test]
    fn test_tier7_w_as_vowel() {
        assert!(tier7_w_as_vowel("new")); // ew
        assert!(tier7_w_as_vowel("show")); // ow
        assert!(tier7_w_as_vowel("law")); // aw
        assert!(tier7_w_as_vowel("view")); // iew
        assert!(tier7_w_as_vowel("now")); // ow at end - IS w as vowel
        assert!(!tier7_w_as_vowel("ban")); // no w-vowel
        assert!(!tier7_w_as_vowel("win")); // w at start, not as vowel
    }

    // is_english tests
    #[test]
    fn test_is_english_combined() {
        assert!(is_english("text")); // tier 3
        assert!(is_english("search")); // tier 4
        assert!(is_english("action")); // tier 5
        assert!(is_english("more")); // tier 6
        assert!(is_english("new")); // tier 7
        assert!(!is_english("ban")); // no pattern
        assert!(!is_english("hoc")); // VN word
    }

    // Decision tests
    #[test]
    fn test_no_transform() {
        let state = BufferState::new();
        let result = should_restore(&state, "hello", "hello", None);
        assert_eq!(result, Decision::Keep);
    }

    #[test]
    fn test_has_stroke_keeps() {
        let mut state = BufferState::new();
        state.set_has_stroke(true);
        state.set_had_transform(true);
        let result = should_restore(&state, "dd", "đ", None);
        assert_eq!(result, Decision::Keep);
    }

    #[test]
    fn test_pending_breve_with_dict_restores() {
        // With English dictionary, "law" should restore
        let mut state = BufferState::new();
        state.set_pending_breve(true);
        state.set_had_transform(true);
        let dict = Dict::from_words(&["law"]);
        let result = should_restore(&state, "law", "lăw", Some(&dict));
        assert_eq!(result, Decision::Restore);
    }

    #[test]
    fn test_pending_breve_without_dict_skips() {
        // Without dictionary, pending breve alone doesn't trigger restore
        // (goes to P7 Skip since vn_state not set)
        let mut state = BufferState::new();
        state.set_pending_breve(true);
        state.set_had_transform(true);
        let result = should_restore(&state, "law", "lăw", None);
        assert_eq!(result, Decision::Skip);
    }

    #[test]
    fn test_impossible_restores() {
        let mut state = BufferState::new();
        state.set_had_transform(true);
        state.set_vn_state(VnState::Impossible);
        let result = should_restore(&state, "xyz", "xyz", None);
        assert_eq!(result, Decision::Restore);
    }

    #[test]
    fn test_english_pattern_with_dict_restores() {
        // With English dictionary, "text" should restore
        let mut state = BufferState::new();
        state.set_had_transform(true);
        state.set_vn_state(VnState::Complete);
        let dict = Dict::from_words(&["text"]);
        let result = should_restore(&state, "text", "téxt", Some(&dict));
        assert_eq!(result, Decision::Restore);
    }

    #[test]
    fn test_english_pattern_without_dict_keeps() {
        // Without dictionary, valid VN with tone is kept (P4)
        // English pattern detection is not used without dictionary
        let mut state = BufferState::new();
        state.set_had_transform(true);
        state.set_has_tone(true);
        state.set_vn_state(VnState::Complete);
        let result = should_restore(&state, "text", "téxt", None);
        assert_eq!(result, Decision::Keep);
    }

    #[test]
    fn test_complete_with_tone_keeps() {
        let mut state = BufferState::new();
        state.set_had_transform(true);
        state.set_has_tone(true);
        state.set_vn_state(VnState::Complete);
        let result = should_restore(&state, "bas", "bá", None);
        assert_eq!(result, Decision::Keep);
    }

    #[test]
    fn test_complete_keeps() {
        let mut state = BufferState::new();
        state.set_had_transform(true);
        state.set_vn_state(VnState::Complete);
        let result = should_restore(&state, "ban", "ban", None);
        assert_eq!(result, Decision::Keep);
    }

    #[test]
    fn test_incomplete_skips() {
        let mut state = BufferState::new();
        state.set_had_transform(true);
        state.set_vn_state(VnState::Incomplete);
        let result = should_restore(&state, "b", "b", None);
        assert_eq!(result, Decision::Skip);
    }

    // Dictionary tests
    #[test]
    fn test_dict_raw_not_match_skips() {
        // Dictionary check is for ENGLISH words (raw), not Vietnamese (buffer)
        // If raw doesn't match English dict, continues to other checks
        // Here vn_state is not set, so goes to P7 Skip
        let mut state = BufferState::new();
        state.set_had_transform(true);
        let dict = Dict::from_words(&["other"]); // "viet" not in dict
        let result = should_restore(&state, "viet", "việt", Some(&dict));
        assert_eq!(result, Decision::Skip);
    }

    #[test]
    fn test_dict_raw_not_match_but_complete_keeps() {
        // If raw doesn't match English dict but buffer is valid Vietnamese, keep
        let mut state = BufferState::new();
        state.set_had_transform(true);
        state.set_vn_state(VnState::Complete);
        let dict = Dict::from_words(&["other"]); // "viet" not in dict
        let result = should_restore(&state, "viet", "việt", Some(&dict));
        assert_eq!(result, Decision::Keep);
    }

    #[test]
    fn test_dict_raw_match_restores() {
        let mut state = BufferState::new();
        state.set_had_transform(true);
        let dict = Dict::from_words(&["hello"]);
        let result = should_restore(&state, "hello", "hellô", Some(&dict));
        assert_eq!(result, Decision::Restore);
    }
}
