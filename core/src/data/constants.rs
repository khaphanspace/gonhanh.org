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

/// Foreign consonants that can optionally be allowed as valid initials
/// These letters (z, w, j, f) are not part of standard Vietnamese but are used
/// in loanwords. When the "allow foreign consonants" option is enabled,
/// these are treated as valid initial consonants.
pub const FOREIGN_INITIALS: &[u16] = &[keys::Z, keys::W, keys::J, keys::F];

/// Valid double initial consonants (11 digraphs)
/// Note: Kr is included for ethnic minority place names (Krông Búk)
pub const VALID_INITIALS_2: &[[u16; 2]] = &[
    [keys::C, keys::H], // ch
    [keys::G, keys::H], // gh
    [keys::G, keys::I], // gi
    [keys::K, keys::H], // kh
    [keys::K, keys::R], // kr - for ethnic minority words (Krông)
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
/// Note: K is included for ethnic minority language place names (e.g., Đắk Lắk)
pub const VALID_FINALS_1: &[u16] = &[
    keys::C,
    keys::K, // for ethnic minority words (Đắk Lắk)
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

/// Valid triphthong base key patterns (13 patterns from docs 7.6.1-B)
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
    [keys::U, keys::E, keys::U], // #42 uêu (nguều - requires circumflex on E)
    [keys::O, keys::A, keys::O], // #43 oao (ngoào - modifier on A for tones)
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

// =============================================================================
// AUTO-RESTORE RULES
// These patterns are used by auto-restore to detect invalid Vietnamese
// =============================================================================

/// Invalid rhyme patterns: -ing + tone mark is NOT valid Vietnamese
/// Vietnamese uses -inh (tính, kính), not -ing with tone marks
/// The rhyme "-ing" exists only in loanwords without tone marks
pub const INVALID_RHYME_ING: &[[u16; 2]] = &[
    [keys::N, keys::G], // -ng final after 'i' with tone = invalid
];

/// Open diphthongs (vần mở) - CANNOT take consonant finals (C/K/M/N/P/T/CH/NG/NH)
/// These diphthongs end with semi-vowel I/O/U/Y that completes the rhyme.
/// Examples:
///   - "ai" (tài) ✓, "ain" ✗ (invalid)
///   - "ao" (cào) ✓, "aon" ✗ (invalid)  ← catches "mason" → "máon"
///   - "au" (đau) ✓, "aum" ✗ (invalid)
///   - "oi" (tôi) ✓, "oin" ✗ (invalid)
///
/// Note: Diphthongs ia/iê, oa/oă, ua/ưa, uê, uô/ươ, uy CAN take consonant finals.
pub const OPEN_DIPHTHONGS: &[[u16; 2]] = &[
    [keys::A, keys::I], // ai - ends with semi-vowel I
    [keys::A, keys::O], // ao - ends with semi-vowel O
    [keys::A, keys::U], // au/âu - ends with semi-vowel U
    [keys::A, keys::Y], // ay/ây - ends with semi-vowel Y
    [keys::E, keys::O], // eo - ends with semi-vowel O
    [keys::I, keys::U], // iu - ends with semi-vowel U
    [keys::O, keys::I], // oi/ôi/ơi - ends with semi-vowel I
    [keys::U, keys::I], // ui/ưi - ends with semi-vowel I
    [keys::U, keys::U], // ưu - ends with semi-vowel U
];

/// Common Vietnamese single-vowel interjections (should NOT be restored)
/// These standalone vowels with tone marks are valid Vietnamese words
/// Example: à (ah), ồ (oh!), ừ (yeah)
/// Note: We also include ò, ì to skip "of"/"if" restore - keep current behavior
pub const COMMON_SINGLE_VOWEL_WORDS: &[(u16, u8)] = &[
    // Using mark values: 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
    (keys::A, 1), // á - very common interjection "huh?", "what?"
    (keys::A, 2), // à - common interjection "ah, I see"
    (keys::A, 4), // ã - interjection
    (keys::U, 2), // ù - exists (ù ù = buzzing sound)
    (keys::U, 4), // ũ - exists
    (keys::O, 2), // ò - skip "of" restore (keep current behavior)
    (keys::I, 2), // ì - skip "if" restore (keep current behavior)
    (keys::Y, 1), // ý - common word meaning "idea/opinion/intention"
];

/// Common Vietnamese single-vowel with CIRCUMFLEX + mark combinations
/// These are typed as VCV pattern (vowel + consonant + vowel) producing one char
/// Example: ofo → ồ (oh!), afa → ầ (hmm)
/// Format: (key, tone, mark) where tone=1 is circumflex
pub const COMMON_CIRCUMFLEX_VOWEL_WITH_MARK: &[(u16, u8, u8)] = &[
    (keys::O, 1, 2), // ồ - common exclamation "oh!" (o + circumflex + huyền)
    (keys::A, 1, 2), // ầ - exists (a + circumflex + huyền)
    (keys::E, 1, 2), // ề - exists (e + circumflex + huyền)
    // Also add sắc variants for completeness
    (keys::O, 1, 1), // ố - exists (o + circumflex + sắc)
    (keys::A, 1, 1), // ấ - exists (a + circumflex + sắc)
    (keys::E, 1, 1), // ế - exists (e + circumflex + sắc)
];

/// Common Vietnamese words: C + circumflex vowel (from double vowel) + no final
/// These should NOT be restored to English
/// Example: bê (calf), mê (obsessed), lê (pear), đê (dike)
pub const COMMON_CIRCUMFLEX_NO_FINAL: &[u16] = &[
    keys::B, // bê - calf
    keys::M, // mê - obsessed
    keys::L, // lê - pear
    keys::D, // đê - dike (with stroke)
    keys::K, // kê - to list/declare
             // Note: sê, tê, pê, gê removed - keep as Vietnamese (Vietnamese-first principle)
];

/// Initials that are UNCOMMON with circumflex vowel + no final
/// VIETNAMESE PRIORITY: Keep this list minimal to maximize Vietnamese pass rate
/// Words like "tê" (numb), "bô" (uncle), etc. ARE valid Vietnamese
/// Only F is truly invalid as Vietnamese initial
pub const UNCOMMON_CIRCUMFLEX_NO_FINAL: &[u16] = &[
    keys::F, // fê - F is invalid initial anyway
];
