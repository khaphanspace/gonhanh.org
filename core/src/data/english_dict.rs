//! English dictionary for auto-restore detection
//!
//! Uses Google's 10,000 most common English words.
//! Only restores to English when raw_input is a known English word.

use std::collections::HashSet;
use std::sync::LazyLock;

/// Embedded English word list (10k most common words)
const ENGLISH_WORDS: &str = include_str!("english_10k.txt");

/// HashSet for O(1) lookup
static DICT: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    ENGLISH_WORDS
        .lines()
        .filter(|line| !line.is_empty())
        .collect()
});

/// Check if a word is in the English dictionary (case-insensitive)
pub fn is_english_word(word: &str) -> bool {
    let lower = word.to_lowercase();
    DICT.contains(lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_words() {
        assert!(is_english_word("the"));
        assert!(is_english_word("view"));
        assert!(is_english_word("lists"));
        assert!(is_english_word("about"));
    }

    #[test]
    fn test_case_insensitive() {
        assert!(is_english_word("The"));
        assert!(is_english_word("VIEW"));
        assert!(is_english_word("Lists"));
    }

    #[test]
    fn test_not_english() {
        assert!(!is_english_word("ddc"));
        assert!(!is_english_word("nesu"));
        assert!(!is_english_word("xyz"));
        assert!(!is_english_word("đc"));
    }

    #[test]
    fn test_dict_size() {
        assert!(DICT.len() >= 9000); // Should have ~10k words
    }
}
