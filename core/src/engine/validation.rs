//! Vietnamese Syllable Validation
//!
//! Rule-based validation for Vietnamese syllables.
//! Each rule is a simple function that returns Some(error) if invalid, None if OK.

use super::syllable::{parse, Syllable};
use crate::data::keys;

/// Validation result
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    InvalidInitial,
    InvalidFinal,
    InvalidSpelling,
    NoVowel,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }
}

// =============================================================================
// VALIDATION RULES - Each rule is a simple check function
// =============================================================================

/// Rule type: takes buffer keys and parsed syllable, returns error or None
type Rule = fn(&[u16], &Syllable) -> Option<ValidationResult>;

/// All validation rules in order of priority
const RULES: &[Rule] = &[
    rule_has_vowel,
    rule_valid_initial,
    rule_all_chars_parsed,
    rule_spelling,
    rule_valid_final,
];

// =============================================================================
// RULE IMPLEMENTATIONS
// =============================================================================

/// Rule 1: Must have at least one vowel
fn rule_has_vowel(_keys: &[u16], syllable: &Syllable) -> Option<ValidationResult> {
    if syllable.is_empty() {
        return Some(ValidationResult::NoVowel);
    }
    None
}

/// Rule 2: Initial consonant must be valid Vietnamese
fn rule_valid_initial(keys: &[u16], syllable: &Syllable) -> Option<ValidationResult> {
    if syllable.initial.is_empty() {
        return None; // No initial = starts with vowel, OK
    }

    let initial: Vec<u16> = syllable.initial.iter().map(|&i| keys[i]).collect();

    let is_valid = match initial.len() {
        1 => VALID_INITIALS_1.contains(&initial[0]),
        2 => VALID_INITIALS_2
            .iter()
            .any(|p| p[0] == initial[0] && p[1] == initial[1]),
        3 => initial[0] == keys::N && initial[1] == keys::G && initial[2] == keys::H,
        _ => false,
    };

    if !is_valid {
        return Some(ValidationResult::InvalidInitial);
    }
    None
}

/// Rule 3: All characters must be parsed into syllable structure
fn rule_all_chars_parsed(keys: &[u16], syllable: &Syllable) -> Option<ValidationResult> {
    let parsed = syllable.initial.len()
        + syllable.glide.map_or(0, |_| 1)
        + syllable.vowel.len()
        + syllable.final_c.len();

    if parsed != keys.len() {
        return Some(ValidationResult::InvalidFinal);
    }
    None
}

/// Rule 4: Vietnamese spelling rules (c/k, g/gh, ng/ngh)
fn rule_spelling(keys: &[u16], syllable: &Syllable) -> Option<ValidationResult> {
    if syllable.initial.is_empty() || syllable.vowel.is_empty() {
        return None;
    }

    let initial: Vec<u16> = syllable.initial.iter().map(|&i| keys[i]).collect();
    let first_vowel = keys[syllable.glide.unwrap_or(syllable.vowel[0])];

    // Check all spelling rules
    for &(consonant, vowels, _msg) in SPELLING_RULES {
        if initial == consonant && vowels.contains(&first_vowel) {
            return Some(ValidationResult::InvalidSpelling);
        }
    }

    None
}

/// Rule 5: Final consonant must be valid
fn rule_valid_final(keys: &[u16], syllable: &Syllable) -> Option<ValidationResult> {
    if syllable.final_c.is_empty() {
        return None;
    }

    let final_c: Vec<u16> = syllable.final_c.iter().map(|&i| keys[i]).collect();

    let is_valid = match final_c.len() {
        1 => VALID_FINALS_1.contains(&final_c[0]),
        2 => VALID_FINALS_2
            .iter()
            .any(|p| p[0] == final_c[0] && p[1] == final_c[1]),
        _ => false,
    };

    if !is_valid {
        return Some(ValidationResult::InvalidFinal);
    }
    None
}

// =============================================================================
// DATA TABLES
// =============================================================================

/// Valid single initial consonants
const VALID_INITIALS_1: &[u16] = &[
    keys::B,
    keys::C,
    keys::D,
    keys::G,
    keys::H,
    keys::K,
    keys::L,
    keys::M,
    keys::N,
    keys::P,
    keys::Q,
    keys::R,
    keys::S,
    keys::T,
    keys::V,
    keys::X,
];

/// Valid double initial consonants
const VALID_INITIALS_2: &[[u16; 2]] = &[
    [keys::C, keys::H], // ch
    [keys::G, keys::H], // gh
    [keys::G, keys::I], // gi
    [keys::K, keys::H], // kh
    [keys::N, keys::G], // ng
    [keys::N, keys::H], // nh
    [keys::P, keys::H], // ph
    [keys::Q, keys::U], // qu
    [keys::T, keys::H], // th
    [keys::T, keys::R], // tr
];

/// Valid single final consonants
const VALID_FINALS_1: &[u16] = &[
    keys::C,
    keys::M,
    keys::N,
    keys::P,
    keys::T,
    keys::I,
    keys::Y,
    keys::O,
    keys::U, // semi-vowels
];

/// Valid double final consonants
const VALID_FINALS_2: &[[u16; 2]] = &[
    [keys::C, keys::H], // ch
    [keys::N, keys::G], // ng
    [keys::N, keys::H], // nh
];

/// Spelling rules: (consonant, invalid_vowels, description)
/// If consonant + vowel matches, it's INVALID
const SPELLING_RULES: &[(&[u16], &[u16], &str)] = &[
    // c before e, i, y → invalid (should use k)
    (&[keys::C], &[keys::E, keys::I, keys::Y], "c before e/i/y"),
    // k before a, o, u → invalid (should use c)
    (&[keys::K], &[keys::A, keys::O, keys::U], "k before a/o/u"),
    // g before e → invalid (should use gh)
    (&[keys::G], &[keys::E], "g before e"),
    // ng before e, i → invalid (should use ngh)
    (&[keys::N, keys::G], &[keys::E, keys::I], "ng before e/i"),
    // gh before a, o, u → invalid (should use g)
    (
        &[keys::G, keys::H],
        &[keys::A, keys::O, keys::U],
        "gh before a/o/u",
    ),
    // ngh before a, o, u → invalid (should use ng)
    (
        &[keys::N, keys::G, keys::H],
        &[keys::A, keys::O, keys::U],
        "ngh before a/o/u",
    ),
];

// =============================================================================
// PUBLIC API
// =============================================================================

/// Validate buffer as Vietnamese syllable - runs all rules
pub fn validate(buffer_keys: &[u16]) -> ValidationResult {
    if buffer_keys.is_empty() {
        return ValidationResult::NoVowel;
    }

    let syllable = parse(buffer_keys);

    // Run all rules in order
    for rule in RULES {
        if let Some(error) = rule(buffer_keys, &syllable) {
            return error;
        }
    }

    ValidationResult::Valid
}

/// Quick check if buffer could be valid Vietnamese
pub fn is_valid(buffer_keys: &[u16]) -> bool {
    validate(buffer_keys).is_valid()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::keys_from_str;

    /// Valid Vietnamese syllables
    const VALID: &[&str] = &[
        "ba", "ca", "an", "em", "gi", "gia", "giau", "ke", "ki", "ky", "nghe", "nghi", "nghieng",
        "truong", "nguoi", "duoc",
    ];

    /// Invalid: no vowel
    const INVALID_NO_VOWEL: &[&str] = &["bcd", "bcdfgh"];

    /// Invalid: bad initial
    const INVALID_INITIAL: &[&str] = &["clau", "john", "bla", "string", "chrome"];

    /// Invalid: spelling violations
    const INVALID_SPELLING: &[&str] = &["ci", "ce", "cy", "ka", "ko", "ku", "ngi", "nge", "ge"];

    /// Invalid: foreign words
    const INVALID_FOREIGN: &[&str] = &["exp", "expect", "test", "claudeco", "claus"];

    fn assert_all_valid(words: &[&str]) {
        for w in words {
            assert!(is_valid(&keys_from_str(w)), "'{}' should be valid", w);
        }
    }

    fn assert_all_invalid(words: &[&str]) {
        for w in words {
            assert!(!is_valid(&keys_from_str(w)), "'{}' should be invalid", w);
        }
    }

    #[test]
    fn test_valid() {
        assert_all_valid(VALID);
    }

    #[test]
    fn test_invalid_no_vowel() {
        assert_all_invalid(INVALID_NO_VOWEL);
    }

    #[test]
    fn test_invalid_initial() {
        assert_all_invalid(INVALID_INITIAL);
    }

    #[test]
    fn test_invalid_spelling() {
        assert_all_invalid(INVALID_SPELLING);
    }

    #[test]
    fn test_invalid_foreign() {
        assert_all_invalid(INVALID_FOREIGN);
    }
}
