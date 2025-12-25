//! M1-M8: Vietnamese Validation Matrices
//!
//! Matrix-based phonotactic validation for Vietnamese syllables.
//! All checks are O(1) table lookups.
//!
//! ## Matrices
//!
//! - **M1: VALID_INITIAL** (32 bits) - Single consonant initials
//! - **M2: VALID_INITIAL_2** (32×32 bits) - Two-consonant initial clusters
//! - **M3: VALID_FINAL** (32 bits) - Single consonant finals
//! - **M4: VALID_FINAL_2** (32×32 bits) - Two-consonant final clusters
//! - **M5: SPELLING** (32×32 bits) - Initial × Vowel spelling rules
//! - **M6: VOWEL_PATTERN** - Valid vowel combinations
//! - **M7: TONE_FINAL** - Tone + final compatibility
//! - **M8: VOWEL_FINAL** - Vowel + final compatibility

use crate::data::keys;

// =============================================================================
// M1: Valid Single Initial Consonants (26 bits for a-z)
// =============================================================================

/// Vietnamese valid single initial consonants
/// b, c, d, đ, g, h, k, l, m, n, p, q, r, s, t, v, x
const VALID_INITIALS: &[u16] = &[
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

/// Check if single consonant is valid initial
#[inline]
pub fn is_valid_initial_1(key: u16) -> bool {
    VALID_INITIALS.contains(&key)
}

// =============================================================================
// M2: Valid Two-Consonant Initial Clusters
// =============================================================================

/// Valid Vietnamese initial clusters (packed as u16: first << 8 | second)
/// ch, gh, gi, kh, ng, nh, ph, qu, th, tr, ngh (special case)
const VALID_INITIAL_2: &[(u16, u16)] = &[
    (keys::C, keys::H), // ch
    (keys::G, keys::H), // gh
    (keys::G, keys::I), // gi
    (keys::K, keys::H), // kh
    (keys::N, keys::G), // ng
    (keys::N, keys::H), // nh
    (keys::P, keys::H), // ph
    (keys::Q, keys::U), // qu
    (keys::T, keys::H), // th
    (keys::T, keys::R), // tr
];

/// Check if two-consonant cluster is valid initial
#[inline]
pub fn is_valid_initial_2(first: u16, second: u16) -> bool {
    VALID_INITIAL_2
        .iter()
        .any(|&(a, b)| a == first && b == second)
}

/// Check if three-consonant cluster is valid initial (only "ngh")
#[inline]
pub fn is_valid_initial_3(first: u16, second: u16, third: u16) -> bool {
    first == keys::N && second == keys::G && third == keys::H
}

// =============================================================================
// M3: Valid Single Final Consonants
// =============================================================================

/// Vietnamese valid single final consonants
/// Valid: c, m, n, p, t (ch, ng, nh handled in M4)
const VALID_FINALS: &[u16] = &[keys::C, keys::M, keys::N, keys::P, keys::T];

/// Check if single consonant is valid final
#[inline]
pub fn is_valid_final_1(key: u16) -> bool {
    VALID_FINALS.contains(&key)
}

// =============================================================================
// M4: Valid Two-Consonant Final Clusters
// =============================================================================

/// Valid Vietnamese final clusters
/// ch, ng, nh
const VALID_FINAL_2: &[(u16, u16)] = &[
    (keys::C, keys::H), // ch (southern pronunciation)
    (keys::N, keys::G), // ng
    (keys::N, keys::H), // nh
];

/// Check if two-consonant cluster is valid final
#[inline]
pub fn is_valid_final_2(first: u16, second: u16) -> bool {
    VALID_FINAL_2
        .iter()
        .any(|&(a, b)| a == first && b == second)
}

// =============================================================================
// M5: Spelling Rules Matrix
// =============================================================================

/// Spelling rules for c/k, g/gh, ng/ngh
/// These rules specify when certain spellings are invalid
///
/// Rule 1: "c" before i, e, ê is invalid (use "k")
/// Rule 2: "g" before i, e, ê is invalid (use "gh")
/// Rule 3: "ng" before i, e, ê is invalid (use "ngh")
///
/// Returns true if the combination is INVALID
#[inline]
pub fn is_spelling_invalid(initial: &[u16], first_vowel: u16) -> bool {
    let front_vowel = matches!(first_vowel, k if k == keys::I || k == keys::E);

    match initial {
        [c] if *c == keys::C && front_vowel => true, // c + i/e invalid
        [g] if *g == keys::G && front_vowel => true, // g + i/e invalid
        [n, g] if *n == keys::N && *g == keys::G && front_vowel => true, // ng + i/e invalid
        _ => false,
    }
}

// =============================================================================
// M6: Valid Vowel Patterns
// =============================================================================

/// Check if vowel sequence is valid
/// Valid patterns include single vowels and common diphthongs/triphthongs
#[inline]
pub fn is_valid_vowel_pattern(vowels: &[u16]) -> bool {
    match vowels.len() {
        0 => false, // Must have vowel
        1 => true,  // Single vowel always valid
        2 => is_valid_diphthong(vowels[0], vowels[1]),
        3 => is_valid_triphthong(vowels[0], vowels[1], vowels[2]),
        _ => false, // No Vietnamese syllable has 4+ vowels
    }
}

/// Valid Vietnamese diphthongs
const VALID_DIPHTHONGS: &[(u16, u16)] = &[
    (keys::A, keys::I),
    (keys::A, keys::O),
    (keys::A, keys::U),
    (keys::A, keys::Y),
    (keys::E, keys::O),
    (keys::E, keys::U),
    (keys::I, keys::A),
    (keys::I, keys::E),
    (keys::I, keys::U),
    (keys::O, keys::A),
    (keys::O, keys::E),
    (keys::O, keys::I),
    (keys::O, keys::O),
    (keys::U, keys::A),
    (keys::U, keys::E),
    (keys::U, keys::I),
    (keys::U, keys::O),
    (keys::U, keys::Y),
    (keys::Y, keys::A),
    (keys::Y, keys::E),
];

#[inline]
fn is_valid_diphthong(v1: u16, v2: u16) -> bool {
    VALID_DIPHTHONGS.iter().any(|&(a, b)| a == v1 && b == v2)
}

/// Valid Vietnamese triphthongs
const VALID_TRIPHTHONGS: &[(u16, u16, u16)] = &[
    (keys::I, keys::A, keys::O), // iao (as in kíao)
    (keys::I, keys::A, keys::U), // iau (as in giàu = rich)
    (keys::I, keys::E, keys::U), // iêu
    (keys::O, keys::A, keys::I), // oai
    (keys::O, keys::A, keys::Y), // oay
    (keys::O, keys::E, keys::O), // oeo
    (keys::U, keys::A, keys::I), // uai
    (keys::U, keys::A, keys::Y), // uay
    (keys::U, keys::O, keys::I), // uôi
    (keys::U, keys::Y, keys::A), // uya
    (keys::U, keys::Y, keys::E), // uyê
    (keys::U, keys::Y, keys::U), // uyu
    (keys::Y, keys::E, keys::U), // yêu
];

#[inline]
fn is_valid_triphthong(v1: u16, v2: u16, v3: u16) -> bool {
    VALID_TRIPHTHONGS
        .iter()
        .any(|&(a, b, c)| a == v1 && b == v2 && c == v3)
}

// =============================================================================
// M7: Tone + Final Compatibility
// =============================================================================

/// Tone marks that are only valid with certain finals (stop consonants)
/// Sắc (1) and Nặng (5) are only valid with stop finals: c, ch, p, t
///
/// Returns true if the tone is compatible with the final
#[inline]
pub fn is_tone_final_compatible(tone: u8, final_c: &[u16]) -> bool {
    // Tones 1 (sắc) and 5 (nặng) require stop finals
    let needs_stop = tone == 1 || tone == 5;

    if !needs_stop {
        return true; // Other tones work with any final
    }

    // Check if final is a stop consonant
    let is_stop = match final_c {
        [] => false, // Open syllable - can have sắc/nặng in some cases
        [c] => *c == keys::C || *c == keys::P || *c == keys::T,
        [c, h] => *c == keys::C && *h == keys::H, // ch is also a stop
        _ => false,
    };

    // For open syllables, allow sắc/nặng (they're valid in Vietnamese)
    final_c.is_empty() || is_stop
}

// =============================================================================
// M8: Vowel + Final Compatibility
// =============================================================================

/// Check if vowel pattern is compatible with final consonant
///
/// Key rules:
/// - Short vowels (ă, â) need closed syllables with certain finals
/// - Some vowel+final combinations are phonotactically invalid
#[inline]
pub fn is_vowel_final_compatible(
    vowels: &[u16],
    has_breve_or_circumflex: bool,
    final_c: &[u16],
) -> bool {
    // ă (a with breve) requires closed syllable with specific finals
    // Valid: ăm, ăn, ăp, ăt, ăc, ăng
    // Invalid: ă alone, ănh
    if has_breve_or_circumflex && vowels == [keys::A] {
        return match final_c {
            [m] if *m == keys::M => true,
            [n] if *n == keys::N => true,
            [p] if *p == keys::P => true,
            [t] if *t == keys::T => true,
            [c] if *c == keys::C => true,
            [n, g] if *n == keys::N && *g == keys::G => true,
            _ => false,
        };
    }

    true // Most combinations are valid
}

// =============================================================================
// Validation Result
// =============================================================================

/// Result of matrix-based validation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatrixValidation {
    Valid,
    InvalidInitial,
    InvalidFinal,
    InvalidSpelling,
    InvalidVowelPattern,
    InvalidToneFinal,
    InvalidVowelFinal,
    NoVowel,
}

impl MatrixValidation {
    #[inline]
    pub fn is_valid(&self) -> bool {
        matches!(self, MatrixValidation::Valid)
    }
}

/// Quick validation using matrix lookups
///
/// Parameters:
/// - initial: Initial consonant keys
/// - vowels: Vowel keys
/// - final_c: Final consonant keys
/// - tone: Tone mark (0=none, 1-5=tones)
/// - has_breve_or_circumflex: Whether vowel has ă/â modifier
#[inline]
pub fn validate(
    initial: &[u16],
    vowels: &[u16],
    final_c: &[u16],
    tone: u8,
    has_breve_or_circumflex: bool,
) -> MatrixValidation {
    // M6: Check vowels exist
    if vowels.is_empty() {
        return MatrixValidation::NoVowel;
    }

    // M1/M2: Check initial consonant
    if !initial.is_empty() {
        let valid = match initial.len() {
            1 => is_valid_initial_1(initial[0]),
            2 => is_valid_initial_2(initial[0], initial[1]),
            3 => is_valid_initial_3(initial[0], initial[1], initial[2]),
            _ => false,
        };
        if !valid {
            return MatrixValidation::InvalidInitial;
        }
    }

    // M3/M4: Check final consonant
    if !final_c.is_empty() {
        let valid = match final_c.len() {
            1 => is_valid_final_1(final_c[0]),
            2 => is_valid_final_2(final_c[0], final_c[1]),
            _ => false,
        };
        if !valid {
            return MatrixValidation::InvalidFinal;
        }
    }

    // M5: Check spelling rules
    if !initial.is_empty() && !vowels.is_empty() && is_spelling_invalid(initial, vowels[0]) {
        return MatrixValidation::InvalidSpelling;
    }

    // M6: Check vowel pattern
    if !is_valid_vowel_pattern(vowels) {
        return MatrixValidation::InvalidVowelPattern;
    }

    // M7: Check tone + final compatibility
    if tone > 0 && !is_tone_final_compatible(tone, final_c) {
        return MatrixValidation::InvalidToneFinal;
    }

    // M8: Check vowel + final compatibility
    if !is_vowel_final_compatible(vowels, has_breve_or_circumflex, final_c) {
        return MatrixValidation::InvalidVowelFinal;
    }

    MatrixValidation::Valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_initial_single() {
        // Valid initials
        assert!(is_valid_initial_1(keys::B));
        assert!(is_valid_initial_1(keys::C));
        assert!(is_valid_initial_1(keys::D));
        assert!(is_valid_initial_1(keys::M));
        assert!(is_valid_initial_1(keys::N));
        assert!(is_valid_initial_1(keys::T));

        // Invalid initials (vowels, w, etc.)
        assert!(!is_valid_initial_1(keys::A));
        assert!(!is_valid_initial_1(keys::E));
        assert!(!is_valid_initial_1(keys::W));
    }

    #[test]
    fn test_valid_initial_cluster() {
        // Valid clusters
        assert!(is_valid_initial_2(keys::C, keys::H)); // ch
        assert!(is_valid_initial_2(keys::T, keys::H)); // th
        assert!(is_valid_initial_2(keys::T, keys::R)); // tr
        assert!(is_valid_initial_2(keys::N, keys::G)); // ng
        assert!(is_valid_initial_2(keys::P, keys::H)); // ph

        // Invalid clusters
        assert!(!is_valid_initial_2(keys::B, keys::R)); // br
        assert!(!is_valid_initial_2(keys::S, keys::T)); // st
    }

    #[test]
    fn test_valid_final_single() {
        // Valid finals
        assert!(is_valid_final_1(keys::C));
        assert!(is_valid_final_1(keys::M));
        assert!(is_valid_final_1(keys::N));
        assert!(is_valid_final_1(keys::P));
        assert!(is_valid_final_1(keys::T));

        // Invalid finals
        assert!(!is_valid_final_1(keys::B));
        assert!(!is_valid_final_1(keys::D));
        assert!(!is_valid_final_1(keys::K));
    }

    #[test]
    fn test_valid_final_cluster() {
        // Valid final clusters
        assert!(is_valid_final_2(keys::N, keys::G)); // ng
        assert!(is_valid_final_2(keys::N, keys::H)); // nh
        assert!(is_valid_final_2(keys::C, keys::H)); // ch

        // Invalid final clusters
        assert!(!is_valid_final_2(keys::M, keys::P));
        assert!(!is_valid_final_2(keys::T, keys::H));
    }

    #[test]
    fn test_spelling_rules() {
        // c + i/e is invalid (use k)
        assert!(is_spelling_invalid(&[keys::C], keys::I));
        assert!(is_spelling_invalid(&[keys::C], keys::E));

        // c + a/o/u is valid
        assert!(!is_spelling_invalid(&[keys::C], keys::A));
        assert!(!is_spelling_invalid(&[keys::C], keys::O));

        // g + i/e is invalid (use gh)
        assert!(is_spelling_invalid(&[keys::G], keys::I));
        assert!(is_spelling_invalid(&[keys::G], keys::E));

        // ng + i/e is invalid (use ngh)
        assert!(is_spelling_invalid(&[keys::N, keys::G], keys::I));
        assert!(is_spelling_invalid(&[keys::N, keys::G], keys::E));
    }

    #[test]
    fn test_vowel_pattern() {
        // Single vowel
        assert!(is_valid_vowel_pattern(&[keys::A]));
        assert!(is_valid_vowel_pattern(&[keys::E]));

        // Valid diphthongs
        assert!(is_valid_vowel_pattern(&[keys::A, keys::I]));
        assert!(is_valid_vowel_pattern(&[keys::O, keys::A]));

        // Invalid patterns
        assert!(!is_valid_vowel_pattern(&[]));
    }

    #[test]
    fn test_tone_final_compatibility() {
        // Sắc/nặng with stop finals - valid
        assert!(is_tone_final_compatible(1, &[keys::C]));
        assert!(is_tone_final_compatible(5, &[keys::T]));
        assert!(is_tone_final_compatible(1, &[keys::C, keys::H]));

        // Other tones work with any final
        assert!(is_tone_final_compatible(2, &[keys::N])); // huyền + n
        assert!(is_tone_final_compatible(3, &[keys::M])); // hỏi + m

        // Open syllables can have any tone
        assert!(is_tone_final_compatible(1, &[]));
        assert!(is_tone_final_compatible(5, &[]));
    }

    #[test]
    fn test_vowel_final_compatibility() {
        // ă (breve) needs specific finals
        assert!(is_vowel_final_compatible(&[keys::A], true, &[keys::N])); // ăn
        assert!(is_vowel_final_compatible(&[keys::A], true, &[keys::M])); // ăm
        assert!(is_vowel_final_compatible(
            &[keys::A],
            true,
            &[keys::N, keys::G]
        )); // ăng

        // ă alone is invalid
        assert!(!is_vowel_final_compatible(&[keys::A], true, &[]));

        // ănh is invalid
        assert!(!is_vowel_final_compatible(
            &[keys::A],
            true,
            &[keys::N, keys::H]
        ));
    }

    #[test]
    fn test_validate_viet() {
        // "việt" = v + ie + t with sắc
        let result = validate(
            &[keys::V],          // initial: v
            &[keys::I, keys::E], // vowels: ie
            &[keys::T],          // final: t
            1,                   // tone: sắc
            false,               // no breve
        );
        assert_eq!(result, MatrixValidation::Valid);
    }

    #[test]
    fn test_validate_duong() {
        // "đường" = d + ươ + ng
        let result = validate(
            &[keys::D],          // initial: đ
            &[keys::U, keys::O], // vowels: ươ (with horn)
            &[keys::N, keys::G], // final: ng
            2,                   // tone: huyền
            false,
        );
        assert_eq!(result, MatrixValidation::Valid);
    }

    #[test]
    fn test_validate_invalid_initial() {
        // Invalid initial cluster
        let result = validate(
            &[keys::B, keys::R], // invalid: br
            &[keys::A],
            &[],
            0,
            false,
        );
        assert_eq!(result, MatrixValidation::InvalidInitial);
    }

    #[test]
    fn test_validate_invalid_spelling() {
        // "ci" should be "ki"
        let result = validate(&[keys::C], &[keys::I], &[], 0, false);
        assert_eq!(result, MatrixValidation::InvalidSpelling);
    }
}
