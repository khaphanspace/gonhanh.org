//! Vietnamese Spell Checking Module
//!
//! Uses HashSet-based word lookup for efficient Vietnamese word validation.
//! Memory-efficient: ~0.5MB vs ~5.5MB with full Hunspell implementation.

use std::collections::HashSet;
use std::sync::LazyLock;

// Embed dictionary files into binary
const DIC_VI: &str = include_str!("dictionaries/vi.dic");
const DIC_KEEP: &str = include_str!("dictionaries/keep.dic");

/// Parse .dic file into HashSet (skip first line which is word count)
fn parse_dic_to_hashset(dic_content: &'static str) -> HashSet<&'static str> {
    dic_content.lines().skip(1).collect()
}

/// Lazy-loaded Vietnamese dictionary - ~0.5MB memory
static DICT_VI: LazyLock<HashSet<&'static str>> = LazyLock::new(|| parse_dic_to_hashset(DIC_VI));

/// Lazy-loaded keep list - words that should not be auto-restored
static DICT_KEEP: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| parse_dic_to_hashset(DIC_KEEP));

/// Check if word starts with foreign consonant (z, w, j, f)
fn starts_with_foreign_consonant(word: &str) -> bool {
    matches!(
        word.as_bytes().first().map(u8::to_ascii_lowercase),
        Some(b'z' | b'w' | b'j' | b'f')
    )
}

/// Check if a word is valid Vietnamese
///
/// - `allow_foreign = true`: Allow words starting with z/w/j/f
/// - `allow_foreign = false`: Reject words starting with z/w/j/f
pub fn is_vietnamese(word: &str, allow_foreign: bool) -> bool {
    if word.is_empty() {
        return false;
    }

    // When foreign consonants NOT allowed, reject words starting with z/w/j/f
    if !allow_foreign && starts_with_foreign_consonant(word) {
        return false;
    }

    // Case-insensitive lookup (dictionary stores lowercase)
    let word_lower = word.to_lowercase();
    DICT_VI.contains(word_lower.as_str())
}

/// Check if a word is in the keep list (should not be auto-restored)
pub fn should_keep(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let word_lower = word.to_lowercase();
    DICT_KEEP.contains(word_lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_vietnamese_words() {
        assert!(is_vietnamese("xin", false));
        assert!(is_vietnamese("chào", false));
        assert!(is_vietnamese("tôi", false));
        assert!(is_vietnamese("Việt", false));
        assert!(is_vietnamese("Nam", false));
    }

    #[test]
    fn test_invalid_words() {
        // English words should not be valid Vietnamese
        assert!(!is_vietnamese("hello", false));
        assert!(!is_vietnamese("world", false));
        assert!(!is_vietnamese("view", false));
        // Gibberish
        assert!(!is_vietnamese("viêư", false));
        assert!(!is_vietnamese("hêllô", false));
    }

    #[test]
    fn test_empty_word() {
        assert!(!is_vietnamese("", false));
    }

    #[test]
    fn test_tones_and_marks() {
        // Words with various tones
        assert!(is_vietnamese("được", false));
        assert!(is_vietnamese("không", false));
        assert!(is_vietnamese("đẹp", false));
    }

    #[test]
    fn test_foreign_consonants_rejected_when_disabled() {
        // Words starting with z/w/j/f should be rejected when allow_foreign = false
        assert!(!is_vietnamese("zá", false));
        assert!(!is_vietnamese("wá", false));
        assert!(!is_vietnamese("já", false));
        assert!(!is_vietnamese("fá", false));
    }

    #[test]
    fn test_foreign_consonants_allowed_when_enabled() {
        // allow_foreign=true skips the foreign consonant check, but word must still be in dictionary
        assert!(!is_vietnamese("zá", true)); // Not in dict → false
    }
}
