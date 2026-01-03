//! Letter classification for Vietnamese phonology
//!
//! Each letter is classified into one of 4 categories:
//! - Vowel: a, e, i, o, u, y (+ modified forms)
//! - Consonant: b, c, d, g, h, k, l, m, n, p, q, r, s, t, v, x
//! - Final: c, m, n, ng, nh, p, t (valid syllable endings)
//! - Stop: c, p, t, ch (checked tones only)

/// Letter classification enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LetterClass {
    /// Vowel letter (a, e, i, o, u, y)
    Vowel = 0,
    /// Consonant letter (b, c, d, etc.)
    Consonant = 1,
    /// Can be final consonant (c, m, n, p, t)
    Final = 2,
    /// Stop consonant - restricts tones (c, p, t)
    Stop = 3,
    /// Invalid/unknown letter
    Invalid = 255,
}

/// Classification table for a-z (26 bytes)
/// Index: letter - 'a'
pub const LETTER_CLASS: [LetterClass; 26] = [
    LetterClass::Vowel,     // a - vowel
    LetterClass::Consonant, // b - consonant
    LetterClass::Stop,      // c - stop final
    LetterClass::Consonant, // d - consonant
    LetterClass::Vowel,     // e - vowel
    LetterClass::Consonant, // f - consonant (foreign)
    LetterClass::Consonant, // g - consonant
    LetterClass::Consonant, // h - consonant
    LetterClass::Vowel,     // i - vowel
    LetterClass::Consonant, // j - consonant (foreign)
    LetterClass::Consonant, // k - consonant
    LetterClass::Consonant, // l - consonant
    LetterClass::Final,     // m - final
    LetterClass::Final,     // n - final
    LetterClass::Vowel,     // o - vowel
    LetterClass::Stop,      // p - stop final
    LetterClass::Consonant, // q - consonant
    LetterClass::Consonant, // r - consonant
    LetterClass::Consonant, // s - consonant
    LetterClass::Stop,      // t - stop final
    LetterClass::Vowel,     // u - vowel
    LetterClass::Consonant, // v - consonant
    LetterClass::Consonant, // w - consonant (foreign)
    LetterClass::Consonant, // x - consonant
    LetterClass::Vowel,     // y - vowel
    LetterClass::Consonant, // z - consonant (foreign)
];

/// Get letter class for a character
#[inline]
pub fn get_class(c: char) -> LetterClass {
    let lower = c.to_ascii_lowercase();
    if ('a'..='z').contains(&lower) {
        LETTER_CLASS[(lower as u8 - b'a') as usize]
    } else {
        LetterClass::Invalid
    }
}

/// Check if character is a vowel
#[inline]
pub fn is_vowel(c: char) -> bool {
    matches!(get_class(c), LetterClass::Vowel)
}

/// Check if character can be a final consonant
#[inline]
pub fn is_final(c: char) -> bool {
    matches!(get_class(c), LetterClass::Final | LetterClass::Stop)
}

/// Check if character is a stop consonant
#[inline]
pub fn is_stop(c: char) -> bool {
    matches!(get_class(c), LetterClass::Stop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vowels() {
        assert!(is_vowel('a'));
        assert!(is_vowel('e'));
        assert!(is_vowel('i'));
        assert!(is_vowel('o'));
        assert!(is_vowel('u'));
        assert!(is_vowel('y'));
        assert!(is_vowel('A')); // uppercase
    }

    #[test]
    fn test_consonants() {
        assert!(!is_vowel('b'));
        assert!(!is_vowel('c'));
        assert!(!is_vowel('d'));
    }

    #[test]
    fn test_finals() {
        assert!(is_final('m'));
        assert!(is_final('n'));
        assert!(is_final('c'));
        assert!(is_final('p'));
        assert!(is_final('t'));
    }

    #[test]
    fn test_stops() {
        assert!(is_stop('c'));
        assert!(is_stop('p'));
        assert!(is_stop('t'));
        assert!(!is_stop('m'));
        assert!(!is_stop('n'));
    }
}
