//! Vietnamese Spell Checking Module
//!
//! Uses zspell (pure Rust Hunspell library) with Vietnamese dictionaries
//! from hunspell-vi to validate Vietnamese words.
//!
//! Supports both orthography styles:
//! - DauMoi (modern): hoà, thuý
//! - DauCu (traditional): hòa, thúy

use std::sync::LazyLock;
use zspell::Dictionary;

// Embed dictionary files into binary
const AFF_DAUMOI: &str = include_str!("dictionaries/vi_daumoi.aff");
const DIC_DAUMOI: &str = include_str!("dictionaries/vi_daumoi.dic");
const AFF_DAUCU: &str = include_str!("dictionaries/vi_daucu.aff");
const DIC_DAUCU: &str = include_str!("dictionaries/vi_daucu.dic");

/// Lazy-loaded DauMoi (modern) dictionary
static DICT_DAUMOI: LazyLock<Option<Dictionary>> = LazyLock::new(|| {
    zspell::builder()
        .config_str(AFF_DAUMOI)
        .dict_str(DIC_DAUMOI)
        .build()
        .ok()
});

/// Lazy-loaded DauCu (traditional) dictionary
static DICT_DAUCU: LazyLock<Option<Dictionary>> = LazyLock::new(|| {
    zspell::builder()
        .config_str(AFF_DAUCU)
        .dict_str(DIC_DAUCU)
        .build()
        .ok()
});

/// Check if a word is valid Vietnamese (either DauMoi or DauCu style)
///
/// Returns true if the word is found in either dictionary.
/// Returns false if dictionaries failed to load or word not found.
pub fn is_valid_vietnamese_word(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }

    // Check DauMoi dictionary
    if let Some(ref dict) = *DICT_DAUMOI {
        if dict.check_word(word) {
            return true;
        }
    }

    // Check DauCu dictionary
    if let Some(ref dict) = *DICT_DAUCU {
        if dict.check_word(word) {
            return true;
        }
    }

    false
}

/// Check if a word is valid Vietnamese using specific dictionary based on style
///
/// - `use_modern = true`: Use DauMoi dictionary (modern style: oà, uý)
/// - `use_modern = false`: Use DauCu dictionary (traditional style: òa, úy)
pub fn check_with_style(word: &str, use_modern: bool) -> bool {
    if word.is_empty() {
        return false;
    }

    if use_modern {
        // Modern style: use DauMoi dictionary
        if let Some(ref dict) = *DICT_DAUMOI {
            return dict.check_word(word);
        }
    } else {
        // Traditional style: use DauCu dictionary
        if let Some(ref dict) = *DICT_DAUCU {
            return dict.check_word(word);
        }
    }

    false
}

/// Check if a word is valid in DauMoi (modern) style only
pub fn is_valid_daumoi(word: &str) -> bool {
    if let Some(ref dict) = *DICT_DAUMOI {
        return dict.check_word(word);
    }
    false
}

/// Check if a word is valid in DauCu (traditional) style only
pub fn is_valid_daucu(word: &str) -> bool {
    if let Some(ref dict) = *DICT_DAUCU {
        return dict.check_word(word);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_vietnamese_words() {
        // Common words should be valid
        assert!(is_valid_vietnamese_word("xin"));
        assert!(is_valid_vietnamese_word("chào"));
        assert!(is_valid_vietnamese_word("tôi"));
        assert!(is_valid_vietnamese_word("việt"));
        assert!(is_valid_vietnamese_word("nam"));
    }

    #[test]
    fn test_invalid_words() {
        // English words should not be valid Vietnamese
        assert!(!is_valid_vietnamese_word("hello"));
        assert!(!is_valid_vietnamese_word("world"));
        assert!(!is_valid_vietnamese_word("view"));
        // Gibberish
        assert!(!is_valid_vietnamese_word("viêư"));
        assert!(!is_valid_vietnamese_word("hêllô"));
    }

    #[test]
    fn test_empty_word() {
        assert!(!is_valid_vietnamese_word(""));
    }

    #[test]
    fn test_tones_and_marks() {
        // Words with various tones
        assert!(is_valid_vietnamese_word("được"));
        assert!(is_valid_vietnamese_word("không"));
        assert!(is_valid_vietnamese_word("đẹp"));
    }
}
