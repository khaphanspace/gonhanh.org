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
/// # Arguments
/// * `state` - Current buffer state flags
/// * `raw` - Raw input string
/// * `buffer` - Transformed buffer string
/// * `dict` - Optional dictionary for word lookup
///
/// # Returns
/// * `Decision` - Keep, Restore, or Skip
pub fn should_restore(
    state: &BufferState,
    raw: &str,
    buffer: &str,
    dict: Option<&Dict>,
) -> Decision {
    // P1: No transform = nothing to restore
    if !state.had_transform() {
        return Decision::Keep;
    }

    // P2: Stroke (đ) = 100% intentional VN
    if state.has_stroke() {
        return Decision::Keep;
    }

    // P3: Pending breve = restore (aw + terminator = law, saw)
    if state.pending_breve() {
        return Decision::Restore;
    }

    // Dictionary-based restore (primary method)
    if let Some(dict) = dict {
        if dict.contains(buffer) {
            return Decision::Keep;
        }
        if dict.contains(raw) {
            return Decision::Restore;
        }
    }

    // P4: Impossible VN should restore immediately
    if state.vn_state() == VnState::Impossible {
        return Decision::Restore;
    }

    // Check for English patterns (Tiers 3-7)
    if is_english(raw) {
        return Decision::Restore;
    }

    // Fallback when no dictionary
    should_restore_fallback(state, raw, buffer)
}

/// Fallback logic when no dictionary available
fn should_restore_fallback(
    state: &BufferState,
    raw: &str,
    buffer: &str,
) -> Decision {
    // P5: Significant char consumption = restore
    let raw_len = raw.chars().count() as i32;
    let buf_len = buffer.chars().count() as i32;
    let consumed = raw_len - buf_len;
    if consumed >= 2 {
        return Decision::Restore;
    }

    // P6: Complete + tone = intentional VN
    if state.has_tone() && state.vn_state() == VnState::Complete {
        return Decision::Keep;
    }

    // P7: Complete = valid VN
    if state.vn_state() == VnState::Complete {
        return Decision::Keep;
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
fn tier4_vowel_pattern(raw: &str) -> bool {
    const EN_VOWELS: &[&str] = &["ea", "ee", "ou", "ei", "yo", "ae", "yi", "oo"];
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
        assert!(tier4_vowel_pattern("see")); // ee
        assert!(tier4_vowel_pattern("you")); // ou
        assert!(tier4_vowel_pattern("ceiling")); // ei
        assert!(!tier4_vowel_pattern("ban")); // no EN pattern
        assert!(!tier4_vowel_pattern("toi")); // oi is VN
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
    fn test_pending_breve_restores() {
        let mut state = BufferState::new();
        state.set_pending_breve(true);
        state.set_had_transform(true);
        let result = should_restore(&state, "law", "lăw", None);
        assert_eq!(result, Decision::Restore);
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
    fn test_english_pattern_restores() {
        let mut state = BufferState::new();
        state.set_had_transform(true);
        state.set_vn_state(VnState::Complete);
        // "text" has English coda cluster "xt"
        let result = should_restore(&state, "text", "téxt", None);
        assert_eq!(result, Decision::Restore);
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
    fn test_dict_buffer_match_keeps() {
        let mut state = BufferState::new();
        state.set_had_transform(true);
        let dict = Dict::from_words(&["việt"]);
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
