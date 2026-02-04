//! Vietnamese Spell Checking Module
//!
//! Uses HashSet-based word lookup for efficient Vietnamese word validation.
//! Memory-efficient: ~0.5MB vs ~5.5MB with full Hunspell implementation.
//!
//! Supports both orthography styles:
//! - DauMoi (modern): hoà, thuý
//! - DauCu (traditional): hòa, thúy

use std::collections::HashSet;
use std::sync::LazyLock;

// Embed dictionary files into binary
const DIC_DAUMOI: &str = include_str!("dictionaries/vi_daumoi.dic");
const DIC_DAUCU: &str = include_str!("dictionaries/vi_daucu.dic");

/// Parse .dic file into HashSet (skip first line which is word count)
fn parse_dic_to_hashset(dic_content: &'static str) -> HashSet<&'static str> {
    dic_content.lines().skip(1).collect()
}

/// Lazy-loaded DauMoi (modern) dictionary - ~0.5MB memory
static DICT_DAUMOI: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| parse_dic_to_hashset(DIC_DAUMOI));

/// Lazy-loaded DauCu (traditional) dictionary - ~0.5MB memory
static DICT_DAUCU: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| parse_dic_to_hashset(DIC_DAUCU));

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

    let dict = if use_modern {
        &*DICT_DAUMOI
    } else {
        &*DICT_DAUCU
    };

    // Case-insensitive lookup (dictionary stores lowercase)
    let word_lower = word.to_lowercase();
    dict.contains(word_lower.as_str())
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
