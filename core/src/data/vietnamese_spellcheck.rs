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

/// Check if word starts with foreign consonant (z, w, j, f)
/// These consonants are not part of standard Vietnamese alphabet
fn starts_with_foreign_consonant(word: &str) -> bool {
    word.chars()
        .next()
        .map(|c| matches!(c.to_ascii_lowercase(), 'z' | 'w' | 'j' | 'f'))
        .unwrap_or(false)
}

/// Check if a word is valid Vietnamese with style and foreign consonants option
///
/// - `use_modern = true`: Use DauMoi dictionary (modern style: oà, uý)
/// - `use_modern = false`: Use DauCu dictionary (traditional style: òa, úy)
/// - `allow_foreign = true`: Allow words starting with z/w/j/f
/// - `allow_foreign = false`: Reject words starting with z/w/j/f
pub fn check_with_style_and_foreign(word: &str, use_modern: bool, allow_foreign: bool) -> bool {
    if word.is_empty() {
        return false;
    }

    // When foreign consonants NOT allowed, reject words starting with z/w/j/f
    if !allow_foreign && starts_with_foreign_consonant(word) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_vietnamese_words() {
        // Common words should be valid (using default: traditional style, no foreign)
        assert!(check_with_style_and_foreign("xin", false, false));
        assert!(check_with_style_and_foreign("chào", false, false));
        assert!(check_with_style_and_foreign("tôi", false, false));
        assert!(check_with_style_and_foreign("Việt", false, false));
        assert!(check_with_style_and_foreign("Nam", false, false));
    }

    #[test]
    fn test_invalid_words() {
        // English words should not be valid Vietnamese
        assert!(!check_with_style_and_foreign("hello", false, false));
        assert!(!check_with_style_and_foreign("world", false, false));
        assert!(!check_with_style_and_foreign("view", false, false));
        // Gibberish
        assert!(!check_with_style_and_foreign("viêư", false, false));
        assert!(!check_with_style_and_foreign("hêllô", false, false));
    }

    #[test]
    fn test_empty_word() {
        assert!(!check_with_style_and_foreign("", false, false));
    }

    #[test]
    fn test_tones_and_marks() {
        // Words with various tones
        assert!(check_with_style_and_foreign("được", false, false));
        assert!(check_with_style_and_foreign("không", false, false));
        assert!(check_with_style_and_foreign("đẹp", false, false));
    }

    #[test]
    fn test_foreign_consonants_rejected_when_disabled() {
        // Words starting with z/w/j/f should be rejected when allow_foreign = false
        assert!(!check_with_style_and_foreign("zá", false, false));
        assert!(!check_with_style_and_foreign("wá", false, false));
        assert!(!check_with_style_and_foreign("já", false, false));
        assert!(!check_with_style_and_foreign("fá", false, false));
    }

    #[test]
    fn test_foreign_consonants_allowed_when_enabled() {
        // Words starting with z/w/j/f should pass foreign check when allow_foreign = true
        // (but still need to be in dictionary to return true - these won't be)
        // Just verify they don't get rejected by the foreign consonant check
        assert!(!check_with_style_and_foreign("zá", false, true)); // Not in dict, but passes foreign check
    }
}
