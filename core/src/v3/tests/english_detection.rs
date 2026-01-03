//! English detection tests
//!
//! Tests for English word pattern detection.

use crate::v3::constants::english::{
    english_confidence, has_english_suffix, has_impossible_coda, has_impossible_onset,
    has_invalid_initial, EnglishConfidence,
};

#[test]
fn test_invalid_initials() {
    // F, J, W, Z are invalid Vietnamese initials
    assert!(has_invalid_initial("file"));
    assert!(has_invalid_initial("jazz"));
    assert!(has_invalid_initial("web"));
    assert!(has_invalid_initial("zone"));

    // Valid Vietnamese initials
    assert!(!has_invalid_initial("text"));
    assert!(!has_invalid_initial("hello"));
}

#[test]
fn test_impossible_onset_clusters() {
    // Impossible in Vietnamese
    assert!(has_impossible_onset("black"));
    assert!(has_impossible_onset("class"));
    assert!(has_impossible_onset("drink"));
    assert!(has_impossible_onset("float"));
    assert!(has_impossible_onset("green"));
    assert!(has_impossible_onset("string"));
    assert!(has_impossible_onset("tray"));

    // Possible in Vietnamese
    assert!(!has_impossible_onset("than")); // th is valid
    assert!(!has_impossible_onset("chau")); // ch is valid
}

#[test]
fn test_impossible_coda_clusters() {
    // Impossible in Vietnamese
    assert!(has_impossible_coda("text")); // xt
    assert!(has_impossible_coda("world")); // ld
    assert!(has_impossible_coda("point")); // nt
    assert!(has_impossible_coda("help")); // lp

    // Possible in Vietnamese
    assert!(!has_impossible_coda("ban")); // n is valid final
    assert!(!has_impossible_coda("cam")); // m is valid final
}

#[test]
fn test_english_suffixes() {
    assert!(has_english_suffix("action")); // -tion
    assert!(has_english_suffix("mission")); // -sion
    assert!(has_english_suffix("running")); // -ing
    assert!(has_english_suffix("beautiful")); // -ful

    // Not matching
    assert!(!has_english_suffix("test"));
    assert!(!has_english_suffix("ing")); // too short
}

#[test]
fn test_confidence_levels() {
    // Tier 1 (100%): Invalid initial (F, J, W, Z)
    assert_eq!(english_confidence("file"), EnglishConfidence::Certain);
    assert_eq!(english_confidence("jazz"), EnglishConfidence::Certain);
    assert_eq!(english_confidence("world"), EnglishConfidence::Certain); // 'w' invalid VN initial

    // Tier 2 (95%): Onset clusters (cl, st, etc.)
    assert_eq!(english_confidence("class"), EnglishConfidence::OnsetCluster);
    assert_eq!(
        english_confidence("string"),
        EnglishConfidence::OnsetCluster
    );

    // Tier 3 (90%): Coda clusters (xt, lp, st, etc.)
    assert_eq!(english_confidence("text"), EnglishConfidence::CodaCluster);
    assert_eq!(english_confidence("help"), EnglishConfidence::CodaCluster); // lp coda

    // Tier 5 (80%): Suffixes
    assert_eq!(english_confidence("action"), EnglishConfidence::HasSuffix);

    // None: No pattern
    assert_eq!(english_confidence("ban"), EnglishConfidence::None);
}

#[test]
fn test_programming_terms() {
    // Common programming terms should be detected
    assert!(english_confidence("function") >= EnglishConfidence::Medium);
    assert!(english_confidence("class") >= EnglishConfidence::High);
    assert!(english_confidence("string") >= EnglishConfidence::High);
    assert!(english_confidence("print") >= EnglishConfidence::High);
}

#[test]
fn test_common_english_words() {
    // Words that might get transformed by Vietnamese IME
    assert!(english_confidence("file") >= EnglishConfidence::High); // Certain (f = invalid initial)
    assert!(english_confidence("world") >= EnglishConfidence::Medium); // CodaCluster (ld)
    assert!(english_confidence("text") >= EnglishConfidence::Medium); // CodaCluster (xt)
}
