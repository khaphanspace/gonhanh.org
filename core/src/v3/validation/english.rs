//! English detection (8-layer pattern system)
//!
//! Pattern-only detection (~500 bytes), no Bloom filter.
//! Target: 98% coverage of top 20,000 English words.
//!
//! ## Layers (matched to validation-algorithm.md)
//! 1. Invalid Vietnamese initials (F, J, W, Z, pr, cl, str) - Certain (100%)
//! 2. Onset clusters (bl, br, cl, cr, dr, fl, fr, etc.) - OnsetCluster (98%)
//! 3. Double consonants (ll, ss, ff, tt, pp, mm, etc.) - DoubleConsonant (95%)
//! 4. English suffixes (-tion, -ing, -ed, -ly, etc.) - HasSuffix (90%)
//! 5. Coda clusters (st, nd, nt, ld, nk, etc.) - CodaCluster (90%)
//! 6. English prefixes (un-, re-, pre-, dis-, etc.) - HasPrefix (75%)
//! 7. Invalid vowel patterns (ea, ou, yo, oo) - VowelPattern (85%)
//! 8. Impossible bigrams in Vietnamese - ImpossibleBigram (80%)

use crate::v3::constants::english::{
    english_confidence, has_doubled_consonant, has_english_prefix, has_english_suffix,
    has_impossible_coda, has_impossible_onset, has_invalid_initial,
};

// Re-export from constants
pub use crate::v3::constants::english::EnglishConfidence;

/// Calculate English likelihood for a word
pub fn english_likelihood(word: &str) -> EnglishConfidence {
    english_confidence(word)
}

/// Check if word has ANY English pattern
/// No threshold - returns true if ANY of 8 layers match
pub fn is_english_word(word: &str) -> bool {
    english_likelihood(word) > EnglishConfidence::None
}

/// Detailed English detection result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnglishDetection {
    /// Overall confidence
    pub confidence: EnglishConfidence,
    /// Detected patterns
    pub patterns: Vec<EnglishPattern>,
}

/// English pattern types detected (8 layers)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnglishPattern {
    /// Layer 1: Invalid Vietnamese initial (F, J, W, Z)
    InvalidInitial(char),
    /// Layer 2: Impossible onset cluster
    ImpossibleOnset(String),
    /// Layer 3: Double consonant
    DoubleConsonant(String),
    /// Layer 4: English suffix
    EnglishSuffix(String),
    /// Layer 5: Impossible coda cluster
    ImpossibleCoda(String),
    /// Layer 6: English prefix
    EnglishPrefix(String),
}

/// Perform detailed English detection (8 layers)
pub fn detect_english(word: &str) -> EnglishDetection {
    let mut patterns = Vec::new();

    // Layer 1: Check invalid initial
    if has_invalid_initial(word) {
        if let Some(c) = word.chars().next() {
            patterns.push(EnglishPattern::InvalidInitial(c));
        }
    }

    // Layer 2: Check impossible onset
    if has_impossible_onset(word) {
        let prefix: String = word.chars().take(3).collect();
        patterns.push(EnglishPattern::ImpossibleOnset(prefix));
    }

    // Layer 3: Check double consonant
    if has_doubled_consonant(word) {
        patterns.push(EnglishPattern::DoubleConsonant(word.to_string()));
    }

    // Layer 4: Check suffix
    if has_english_suffix(word) {
        patterns.push(EnglishPattern::EnglishSuffix(word.to_string()));
    }

    // Layer 5: Check impossible coda
    if has_impossible_coda(word) {
        let suffix: String = word
            .chars()
            .rev()
            .take(3)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        patterns.push(EnglishPattern::ImpossibleCoda(suffix));
    }

    // Layer 6: Check prefix
    if has_english_prefix(word) {
        patterns.push(EnglishPattern::EnglishPrefix(word.to_string()));
    }

    let confidence = english_confidence(word);

    EnglishDetection {
        confidence,
        patterns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_likelihood_8_layers() {
        // Layer 1: Invalid initial = Certain (100%)
        assert_eq!(english_likelihood("file"), EnglishConfidence::Certain);
        // Layer 2: Onset cluster (cl) = OnsetCluster (98%)
        assert_eq!(english_likelihood("class"), EnglishConfidence::OnsetCluster);
        // Layer 3: Double consonant = DoubleConsonant (95%)
        assert_eq!(
            english_likelihood("coffee"),
            EnglishConfidence::DoubleConsonant
        );
        // Layer 4: Suffix = HasSuffix (90%)
        assert_eq!(english_likelihood("nation"), EnglishConfidence::HasSuffix);
        // Layer 5: Coda cluster (xt) = CodaCluster (90%)
        assert_eq!(english_likelihood("text"), EnglishConfidence::CodaCluster);
    }

    #[test]
    fn test_is_english() {
        // Has ANY English pattern = true
        assert!(is_english_word("file")); // Certain - invalid VN initial 'f'
        assert!(is_english_word("class")); // OnsetCluster - 'cl'
        assert!(is_english_word("coffee")); // DoubleConsonant - 'ff'
        assert!(is_english_word("running")); // HasSuffix - 'ing'
        assert!(is_english_word("text")); // CodaCluster - 'xt'
        assert!(is_english_word("test")); // CodaCluster - 'st'
        assert!(is_english_word("their")); // VowelPattern - 'ei'
        assert!(is_english_word("search")); // VowelPattern - 'ea'
        assert!(is_english_word("undo")); // HasPrefix - 'un'

        // No pattern = false
        assert!(!is_english_word("ban")); // None - could be VN or EN
    }

    #[test]
    fn test_detect_english_patterns() {
        // Layer 1: Invalid initial
        let result = detect_english("file");
        assert_eq!(result.confidence, EnglishConfidence::Certain);
        assert!(result
            .patterns
            .iter()
            .any(|p| matches!(p, EnglishPattern::InvalidInitial('f'))));

        // Layer 3: Double consonant
        let result = detect_english("coffee");
        assert_eq!(result.confidence, EnglishConfidence::DoubleConsonant);
        assert!(result
            .patterns
            .iter()
            .any(|p| matches!(p, EnglishPattern::DoubleConsonant(_))));
    }
}
