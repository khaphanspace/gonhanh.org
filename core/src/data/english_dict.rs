//! English dictionary for auto-restore detection (static list, see lexicon.rs).
//! Only restores to English when raw_input is a known English word.

use super::lexicon::EN;

/// Check if a word is in the English dictionary (case-insensitive)
pub fn is_english_word(word: &str) -> bool {
    EN.contains_lower(word)
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
        assert!(EN.len() >= 17000); // ~18k words (10k + double telex)
    }
}
