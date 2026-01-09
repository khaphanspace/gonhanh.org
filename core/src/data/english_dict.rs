//! English dictionary for auto-restore detection
//!
//! Uses merged dictionary: 10k common words + words with double telex chars.
//! Only restores to English when raw_input is a known English word.

use std::collections::HashSet;
use std::sync::LazyLock;

/// Embedded English word list (10k + double telex patterns)
const ENGLISH_WORDS: &str = include_str!("english_dict_merged.txt");

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
        assert!(!is_english_word("qqq"));
        assert!(!is_english_word("nesu"));
        assert!(!is_english_word("zzzz"));
        assert!(!is_english_word("đc"));
    }

    #[test]
    fn test_dict_size() {
        assert!(DICT.len() >= 17000); // Should have ~18k words (10k + double telex)
    }
}
