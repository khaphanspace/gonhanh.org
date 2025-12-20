//! Vietnamese Phonological Constants
//!
//! Centralized constants for valid initials, finals, vowel patterns, and spelling rules.
//! Vowel patterns based on docs/vietnamese-language-system.md Section 7.6.1

use crate::data::keys;

// =============================================================================
// INITIAL CONSONANTS
// =============================================================================

/// Valid single initial consonants (16 consonants)
pub const VALID_INITIALS_1: &[u16] = &[
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

/// Valid double initial consonants (10 digraphs)
pub const VALID_INITIALS_2: &[[u16; 2]] = &[
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

// =============================================================================
// FINAL CONSONANTS
// =============================================================================

/// Valid single final consonants
pub const VALID_FINALS_1: &[u16] = &[
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
pub const VALID_FINALS_2: &[[u16; 2]] = &[
    [keys::C, keys::H], // ch
    [keys::N, keys::G], // ng
    [keys::N, keys::H], // nh
];

// =============================================================================
// VALID VOWEL PATTERNS (Whitelist from docs 7.6.1)
// =============================================================================

/// Valid diphthong base key patterns (29 patterns from docs 7.6.1-A)
///
/// These are BASE KEYS only. Some patterns require specific modifiers:
/// - E+U requires circumflex on E (êu), NOT valid as plain eu or eư
/// - I+E requires circumflex on E (iê)
/// - Y+E requires circumflex on E (yê)
/// - etc.
///
/// Modifier requirements are checked separately via MODIFIER_REQUIRED_PATTERNS.
pub const VALID_DIPHTHONGS: &[[u16; 2]] = &[
    // A combinations: ai, ao, au, ay (also âu, ây with circumflex)
    [keys::A, keys::I], // #1 ai
    [keys::A, keys::O], // #2 ao
    [keys::A, keys::U], // #3 au, #5 âu (modifier differentiates)
    [keys::A, keys::Y], // #4 ay, #6 ây (modifier differentiates)
    // E combinations: eo, êu
    [keys::E, keys::O], // #7 eo
    [keys::E, keys::U], // #8 êu (REQUIRES circumflex on E)
    // I combinations: ia, iê, iu
    [keys::I, keys::A], // #9 ia
    [keys::I, keys::E], // #10 iê (requires circumflex on E)
    [keys::I, keys::U], // #11 iu
    // O combinations: oa, oă, oe, oi (also ôi, ơi with modifier)
    [keys::O, keys::A], // #12 oa, #13 oă (modifier differentiates)
    [keys::O, keys::E], // #14 oe
    [keys::O, keys::I], // #15 oi, #16 ôi, #17 ơi (modifier differentiates)
    // U combinations: ua, uâ, uê, ui, uô, uy (also ưa, ưi, ươ, ưu with horn)
    [keys::U, keys::A], // #18 ua, #20 uâ, #25 ưa (modifier differentiates)
    [keys::U, keys::E], // #21 uê (requires circumflex on E)
    [keys::U, keys::I], // #22 ui, #26 ưi (modifier differentiates)
    [keys::U, keys::O], // #23 uô, #27 ươ (modifier differentiates)
    [keys::U, keys::Y], // #24 uy
    [keys::U, keys::U], // #28 ưu (requires horn on first U)
    // Y combinations: yê
    [keys::Y, keys::E], // #29 yê (requires circumflex on E)
];

/// Valid triphthong base key patterns (11 patterns from docs 7.6.1-B)
pub const VALID_TRIPHTHONGS: &[[u16; 3]] = &[
    [keys::I, keys::E, keys::U], // #30 iêu
    [keys::Y, keys::E, keys::U], // #31 yêu
    [keys::O, keys::A, keys::I], // #32 oai
    [keys::O, keys::A, keys::Y], // #33 oay
    [keys::O, keys::E, keys::O], // #34 oeo
    [keys::U, keys::A, keys::Y], // #35 uây
    [keys::U, keys::O, keys::I], // #36 uôi, #38 ươi (modifier differentiates)
    [keys::U, keys::Y, keys::A], // #37 uya (khuya)
    [keys::U, keys::O, keys::U], // #39 ươu
    [keys::U, keys::Y, keys::E], // #40 uyê
    [keys::U, keys::Y, keys::U], // #41 uyu (khuỷu - elbow)
];

// =============================================================================
// MODIFIER REQUIREMENTS FOR VOWEL PATTERNS
// =============================================================================

/// Patterns requiring CIRCUMFLEX on V1 (first vowel)
/// E+U is only valid as êu (E has circumflex), not eu or eư
pub const V1_CIRCUMFLEX_REQUIRED: &[[u16; 2]] = &[
    [keys::E, keys::U], // êu: E (V1) must have circumflex
];

/// Patterns requiring CIRCUMFLEX on V2 (second vowel)
/// These patterns have E as second vowel which needs circumflex
pub const V2_CIRCUMFLEX_REQUIRED: &[[u16; 2]] = &[
    [keys::I, keys::E], // iê: E (V2) must have circumflex
    [keys::U, keys::E], // uê: E (V2) must have circumflex
    [keys::Y, keys::E], // yê: E (V2) must have circumflex
];

// =============================================================================
// SPELLING RULES
// =============================================================================

/// Spelling rules: (consonant, invalid_vowels, description)
/// If consonant + vowel matches, it's INVALID
pub const SPELLING_RULES: &[(&[u16], &[u16], &str)] = &[
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
