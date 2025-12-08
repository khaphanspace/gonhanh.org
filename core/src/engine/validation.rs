//! Vietnamese Syllable Validation
//!
//! Validates if a buffer represents a valid Vietnamese syllable.
//! Used to prevent transforming non-Vietnamese words (Claus, HTTP, etc.)

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

/// Valid single initial consonants in Vietnamese
const VALID_SINGLE_INITIALS: &[u16] = &[
    keys::B,
    keys::C,
    keys::D,
    // đ is handled specially (stroke flag)
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
const VALID_DOUBLE_INITIALS: &[[u16; 2]] = &[
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

/// Valid triple initial consonant
const VALID_TRIPLE_INITIAL: [u16; 3] = [keys::N, keys::G, keys::H]; // ngh

/// Validate buffer as Vietnamese syllable
pub fn validate(buffer_keys: &[u16]) -> ValidationResult {
    if buffer_keys.is_empty() {
        return ValidationResult::NoVowel;
    }

    let syllable = parse(buffer_keys);

    // Must have vowel
    if syllable.is_empty() {
        return ValidationResult::NoVowel;
    }

    // Validate initial consonant
    if let Some(result) = validate_initial(buffer_keys, &syllable) {
        return result;
    }

    // Validate spelling rules
    if let Some(result) = validate_spelling(buffer_keys, &syllable) {
        return result;
    }

    // Validate final consonant
    if let Some(result) = validate_final(buffer_keys, &syllable) {
        return result;
    }

    ValidationResult::Valid
}

/// Quick check if buffer could be valid Vietnamese
pub fn is_valid(buffer_keys: &[u16]) -> bool {
    validate(buffer_keys).is_valid()
}

/// Validate initial consonant
fn validate_initial(keys: &[u16], syllable: &Syllable) -> Option<ValidationResult> {
    let initial_len = syllable.initial.len();

    if initial_len == 0 {
        // No initial - syllable starts with vowel, OK
        return None;
    }

    // Get all initial keys
    let initial_keys: Vec<u16> = syllable.initial.iter().map(|&i| keys[i]).collect();

    // Check if it's a valid Vietnamese initial
    match initial_len {
        1 => {
            let k = initial_keys[0];
            if !VALID_SINGLE_INITIALS.contains(&k) {
                // Could be 'd' which becomes 'đ', check for it
                if k != keys::D {
                    return Some(ValidationResult::InvalidInitial);
                }
            }
        }
        2 => {
            let pattern = [initial_keys[0], initial_keys[1]];
            if !VALID_DOUBLE_INITIALS.contains(&pattern) {
                return Some(ValidationResult::InvalidInitial);
            }
        }
        3 => {
            if [initial_keys[0], initial_keys[1], initial_keys[2]] != VALID_TRIPLE_INITIAL {
                return Some(ValidationResult::InvalidInitial);
            }
        }
        _ => return Some(ValidationResult::InvalidInitial),
    }

    None
}

/// Validate Vietnamese spelling rules
///
/// Rules:
/// - c before e, ê, i, y → must use k
/// - k before a, ă, â, o, ô, ơ, u, ư → must use c
/// - g before e, ê, i → must use gh
/// - ng before e, ê, i → must use ngh
fn validate_spelling(buffer_keys: &[u16], syllable: &Syllable) -> Option<ValidationResult> {
    if syllable.initial.is_empty() || syllable.vowel.is_empty() {
        return None;
    }

    let first_vowel_idx = syllable.glide.unwrap_or(syllable.vowel[0]);
    let first_vowel = buffer_keys[first_vowel_idx];

    let initial_keys: Vec<u16> = syllable.initial.iter().map(|&i| buffer_keys[i]).collect();

    // Single consonant spelling rules
    if initial_keys.len() == 1 {
        let consonant = initial_keys[0];

        // c before e, ê, i, y → invalid (should use k)
        if consonant == keys::C && matches!(first_vowel, keys::E | keys::I | keys::Y) {
            return Some(ValidationResult::InvalidSpelling);
        }

        // k before a, o, u → invalid (should use c)
        // Note: k + ă, â, ô, ơ, ư are also invalid but we check base vowels here
        if consonant == keys::K && matches!(first_vowel, keys::A | keys::O | keys::U) {
            return Some(ValidationResult::InvalidSpelling);
        }

        // g before e, i → invalid (should use gh)
        // Note: gi is separate initial
        if consonant == keys::G && matches!(first_vowel, keys::E | keys::I) {
            return Some(ValidationResult::InvalidSpelling);
        }
    }

    // Double consonant spelling rules
    if initial_keys.len() == 2 {
        // ng before e, ê, i → invalid (should use ngh)
        if initial_keys == [keys::N, keys::G] && matches!(first_vowel, keys::E | keys::I) {
            return Some(ValidationResult::InvalidSpelling);
        }

        // gh before a, o, u → invalid (should use g)
        if initial_keys == [keys::G, keys::H] && matches!(first_vowel, keys::A | keys::O | keys::U)
        {
            return Some(ValidationResult::InvalidSpelling);
        }
    }

    // Triple consonant spelling rules
    if initial_keys.len() == 3 {
        // ngh before a, o, u → invalid (should use ng)
        if initial_keys == [keys::N, keys::G, keys::H]
            && matches!(first_vowel, keys::A | keys::O | keys::U)
        {
            return Some(ValidationResult::InvalidSpelling);
        }
    }

    None
}

/// Validate final consonant combinations
fn validate_final(buffer_keys: &[u16], syllable: &Syllable) -> Option<ValidationResult> {
    let final_len = syllable.final_c.len();

    if final_len == 0 {
        return None;
    }

    // Valid finals: c, ch, m, n, ng, nh, p, t, i, y, o, u
    let valid_single = [
        keys::C,
        keys::M,
        keys::N,
        keys::P,
        keys::T,
        keys::I,
        keys::Y,
        keys::O,
        keys::U,
    ];

    let valid_double = [
        [keys::C, keys::H], // ch
        [keys::N, keys::G], // ng
        [keys::N, keys::H], // nh
    ];

    // Get final keys
    let final_keys: Vec<u16> = syllable.final_c.iter().map(|&i| buffer_keys[i]).collect();

    match final_len {
        1 => {
            let k = final_keys[0];
            if !valid_single.contains(&k) {
                return Some(ValidationResult::InvalidFinal);
            }
        }
        2 => {
            if !valid_double.contains(&[final_keys[0], final_keys[1]]) {
                return Some(ValidationResult::InvalidFinal);
            }
        }
        _ => return Some(ValidationResult::InvalidFinal),
    }

    // Note: Vowel + final combinations (-ch, -nh) are validated at base vowel level
    // to allow diacritics like ă and ê

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys_from_str(s: &str) -> Vec<u16> {
        s.chars()
            .filter_map(|c| match c.to_ascii_lowercase() {
                'a' => Some(keys::A),
                'b' => Some(keys::B),
                'c' => Some(keys::C),
                'd' => Some(keys::D),
                'e' => Some(keys::E),
                'f' => Some(keys::F),
                'g' => Some(keys::G),
                'h' => Some(keys::H),
                'i' => Some(keys::I),
                'j' => Some(keys::J),
                'k' => Some(keys::K),
                'l' => Some(keys::L),
                'm' => Some(keys::M),
                'n' => Some(keys::N),
                'o' => Some(keys::O),
                'p' => Some(keys::P),
                'q' => Some(keys::Q),
                'r' => Some(keys::R),
                's' => Some(keys::S),
                't' => Some(keys::T),
                'u' => Some(keys::U),
                'v' => Some(keys::V),
                'w' => Some(keys::W),
                'x' => Some(keys::X),
                'y' => Some(keys::Y),
                'z' => Some(keys::Z),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn valid_simple_words() {
        assert!(is_valid(&keys_from_str("ba")));
        assert!(is_valid(&keys_from_str("ca")));
        assert!(is_valid(&keys_from_str("an")));
        assert!(is_valid(&keys_from_str("em")));
    }

    #[test]
    fn valid_complex_words() {
        assert!(is_valid(&keys_from_str("nghieng")));
        assert!(is_valid(&keys_from_str("truong")));
        assert!(is_valid(&keys_from_str("nguoi")));
        assert!(is_valid(&keys_from_str("duoc")));
    }

    #[test]
    fn invalid_no_vowel() {
        assert_eq!(validate(&keys_from_str("bcd")), ValidationResult::NoVowel);
        assert_eq!(validate(&keys_from_str("http")), ValidationResult::NoVowel);
    }

    #[test]
    fn invalid_initial() {
        // "cl" is not valid Vietnamese initial - clau has vowel 'a'
        assert_eq!(
            validate(&keys_from_str("clau")),
            ValidationResult::InvalidInitial
        );
        // "j" is not valid Vietnamese initial - john has vowel 'o'
        assert_eq!(
            validate(&keys_from_str("john")),
            ValidationResult::InvalidInitial
        );
        // "bl" is not valid Vietnamese initial
        assert_eq!(
            validate(&keys_from_str("bla")),
            ValidationResult::InvalidInitial
        );
    }

    #[test]
    fn spelling_rules() {
        // c before i → invalid (should use k)
        assert_eq!(
            validate(&keys_from_str("ci")),
            ValidationResult::InvalidSpelling
        );
        // k before a → invalid (should use c)
        assert_eq!(
            validate(&keys_from_str("ka")),
            ValidationResult::InvalidSpelling
        );
        // ng before i → invalid (should use ngh)
        assert_eq!(
            validate(&keys_from_str("ngi")),
            ValidationResult::InvalidSpelling
        );
    }

    #[test]
    fn valid_with_k() {
        // k + e, i, y is valid
        assert!(is_valid(&keys_from_str("ke")));
        assert!(is_valid(&keys_from_str("ki")));
        assert!(is_valid(&keys_from_str("ky")));
    }

    #[test]
    fn valid_ngh() {
        // ngh + e, i is valid
        assert!(is_valid(&keys_from_str("nghe")));
        assert!(is_valid(&keys_from_str("nghi")));
    }
}
