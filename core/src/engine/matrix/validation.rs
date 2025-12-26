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
/// ch, gh, gi, kh, kr, ng, nh, ph, qu, th, tr, ngh (special case)
const VALID_INITIAL_2: &[(u16, u16)] = &[
    (keys::C, keys::H), // ch
    (keys::G, keys::H), // gh
    (keys::G, keys::I), // gi
    (keys::K, keys::H), // kh
    (keys::K, keys::R), // kr (ethnic minority: Krông)
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
/// Valid: c, k, m, n, p, t (ch, ng, nh handled in M4)
/// Note: 'k' is used in place names like Đắk Lắk, phonetically same as 'c'
const VALID_FINALS: &[u16] = &[keys::C, keys::K, keys::M, keys::N, keys::P, keys::T];

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
    (keys::U, keys::U), // ưu (as in ưu tiên = priority, cứu = rescue)
    (keys::U, keys::Y),
    (keys::Y, keys::A),
    (keys::Y, keys::E),
];

#[inline]
pub fn is_valid_diphthong(v1: u16, v2: u16) -> bool {
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
    (keys::U, keys::O, keys::U), // ươu (as in rượu = wine/alcohol)
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
// M9: Circumflex Closed Syllable Rule
// =============================================================================

/// Check if circumflex vowel is valid in closed syllable
///
/// Rule: Closed syllable + circumflex (â, ê, ô) + NO tone = INVALID
/// ONLY for specific finals (p, t, m). Other finals (n, ng, nh, ch, c) are valid.
///
/// This is because Vietnamese has many valid words with circumflex + n/ng/nh/ch/c + no tone:
/// - "cân" (scale), "sân" (courtyard), "tên" (name), "bên" (side) - all VALID
///
/// But circumflex + p/t/m + no tone are typically English loan words:
/// - "kêp" → likely "keep", "bêp" → likely "beep"
///
/// Examples:
/// - "kêp" → INVALID (circumflex + p + no tone)
/// - "kếp" → VALID (circumflex + p + sắc tone)
/// - "cân" → VALID (circumflex + n + no tone) - real Vietnamese word
/// - "bếp" → VALID (circumflex + p + sắc) - "kitchen"
/// - "bệp" → VALID (circumflex + p + nặng)
///
/// Parameters:
/// - has_circumflex: Whether vowel has circumflex mark (â, ê, ô)
/// - final_c: Final consonant(s)
/// - tone: Tone mark (0=none, 1-5=tones)
///
/// Returns true if the combination is INVALID.
#[inline]
pub fn is_circumflex_invalid_in_closed_syllable(
    has_circumflex: bool,
    final_c: &[u16],
    tone: u8,
) -> bool {
    // Only applies when circumflex is present
    if !has_circumflex {
        return false;
    }

    // Only applies to closed syllables (has final consonant)
    if final_c.is_empty() {
        return false;
    }

    // Only invalid when NO tone mark
    // If tone > 0, it's valid (e.g., kếp, bếp, bệp)
    if tone > 0 {
        return false;
    }

    // Circumflex + closed + no tone is ONLY invalid for specific finals (p, t, m)
    // Finals n, ng, nh, ch, c are valid (e.g., cân, sân, tên, bên are real words)
    match final_c {
        [k] if *k == keys::P || *k == keys::T || *k == keys::M => true,
        _ => false,
    }
}

// =============================================================================
// M10: Foreign Pattern Detection
// =============================================================================

/// Invalid Vietnamese single initials (these letters NEVER start Vietnamese words)
/// Valid Vietnamese initials: b, c, d, đ, g, h, k, l, m, n, p, q, r, s, t, v, x
/// NOTE: 'w' is NOT included because in Telex it's a valid shortcut for 'ư'
const INVALID_SINGLE_INITIALS: &[u16] = &[
    keys::F, // English: fire, fast
    keys::J, // English: job, just
    keys::Z, // English: zero, zone
];

/// Check if single initial is foreign (invalid in Vietnamese)
#[inline]
pub fn is_invalid_single_initial(key: u16) -> bool {
    INVALID_SINGLE_INITIALS.contains(&key)
}

/// Invalid Vietnamese initial clusters (common in English/foreign words)
/// These are patterns that cannot start Vietnamese syllables
const FOREIGN_INITIAL_2: &[(u16, u16)] = &[
    (keys::B, keys::L), // bl
    (keys::B, keys::R), // br
    (keys::C, keys::L), // cl
    (keys::C, keys::R), // cr
    (keys::D, keys::R), // dr
    (keys::D, keys::W), // dw
    (keys::F, keys::L), // fl
    (keys::F, keys::R), // fr
    (keys::G, keys::L), // gl
    (keys::G, keys::R), // gr
    (keys::P, keys::L), // pl
    (keys::P, keys::R), // pr
    (keys::S, keys::C), // sc
    (keys::S, keys::H), // sh
    (keys::S, keys::K), // sk
    (keys::S, keys::L), // sl
    (keys::S, keys::M), // sm
    (keys::S, keys::N), // sn
    (keys::S, keys::P), // sp
    (keys::S, keys::T), // st
    (keys::S, keys::W), // sw
    (keys::T, keys::W), // tw
    (keys::W, keys::H), // wh
    (keys::W, keys::R), // wr
];

/// Invalid Vietnamese single finals (these letters NEVER end Vietnamese words)
/// Valid Vietnamese finals are: c, k, m, n, p, t (and clusters ch, ng, nh)
/// NOTE: 'w' is NOT included because in Telex it's a valid modifier (horn/breve)
const INVALID_SINGLE_FINALS: &[u16] = &[
    keys::B, // English: job, tab
    keys::D, // English: bad, red
    keys::F, // English: half, chef
    keys::G, // English: dog, big (but ng is valid)
    keys::H, // English: yeah, ah (but nh, ch are valid)
    keys::J, // English: Raj
    keys::L, // English: all, bell
    keys::Q, // (none)
    keys::R, // English: car, water
    keys::S, // English: yes, bus
    keys::V, // English: love, of
    keys::X, // English: box, six
    keys::Z, // English: jazz, quiz
];

/// Check if single final is foreign (invalid in Vietnamese)
#[inline]
pub fn is_invalid_single_final(key: u16) -> bool {
    INVALID_SINGLE_FINALS.contains(&key)
}

/// Invalid Vietnamese final clusters (common in English/foreign words)
const FOREIGN_FINAL_2: &[(u16, u16)] = &[
    (keys::C, keys::K), // ck
    (keys::C, keys::T), // ct
    (keys::F, keys::T), // ft
    (keys::G, keys::S), // gs
    (keys::G, keys::H), // gh (final)
    (keys::L, keys::D), // ld
    (keys::L, keys::F), // lf
    (keys::L, keys::K), // lk
    (keys::L, keys::L), // ll
    (keys::L, keys::M), // lm
    (keys::L, keys::P), // lp
    (keys::L, keys::S), // ls
    (keys::L, keys::T), // lt
    (keys::M, keys::P), // mp
    (keys::M, keys::S), // ms
    (keys::N, keys::D), // nd
    (keys::N, keys::K), // nk
    (keys::N, keys::S), // ns
    (keys::N, keys::T), // nt
    (keys::P, keys::S), // ps
    (keys::P, keys::T), // pt
    (keys::R, keys::C), // rc
    (keys::R, keys::D), // rd
    (keys::R, keys::F), // rf
    (keys::R, keys::G), // rg
    (keys::R, keys::K), // rk
    (keys::R, keys::L), // rl
    (keys::R, keys::M), // rm
    (keys::R, keys::N), // rn
    (keys::R, keys::P), // rp
    (keys::R, keys::S), // rs
    (keys::R, keys::T), // rt
    (keys::S, keys::K), // sk
    (keys::S, keys::S), // ss
    (keys::S, keys::T), // st
    (keys::T, keys::H), // th (final)
    (keys::T, keys::S), // ts
    (keys::X, keys::T), // xt
];

/// Check if initial cluster is foreign (invalid Vietnamese)
#[inline]
pub fn is_foreign_initial_2(first: u16, second: u16) -> bool {
    FOREIGN_INITIAL_2
        .iter()
        .any(|&(a, b)| a == first && b == second)
}

/// Check if final cluster is foreign (invalid Vietnamese)
#[inline]
pub fn is_foreign_final_2(first: u16, second: u16) -> bool {
    FOREIGN_FINAL_2
        .iter()
        .any(|&(a, b)| a == first && b == second)
}

/// Check if key sequence contains foreign patterns
///
/// Analyzes key sequence for patterns that cannot be Vietnamese.
/// Uses matrix-based validation instead of case-by-case detection.
///
/// NOTE: This function expects pre-filtered keys where consumed modifiers
/// (tone/mark keys) have been removed. Use processor.raw().unconsumed_keys().
#[inline]
pub fn is_foreign_pattern_keys(keys: &[u16]) -> bool {
    if keys.len() < 2 {
        return false;
    }

    // Helper: check if key is a vowel
    let is_vowel = |k: u16| {
        k == keys::A || k == keys::E || k == keys::I || k == keys::O || k == keys::U || k == keys::Y
    };

    // Check first consonant - invalid single initials (f, j, w, z)
    if !is_vowel(keys[0]) && is_invalid_single_initial(keys[0]) {
        return true;
    }

    // Check consecutive consonant pairs at start (foreign onset clusters)
    let mut i = 0;
    while i < keys.len() - 1 {
        let k1 = keys[i];
        let k2 = keys[i + 1];

        if !is_vowel(k1) && !is_vowel(k2) {
            // Consecutive consonants at word start
            if i == 0 && is_foreign_initial_2(k1, k2) {
                return true;
            }
        } else {
            break; // Hit a vowel, stop checking onset
        }
        i += 1;
    }

    // Check consecutive consonants at end (foreign coda)
    // Find first and last vowel positions
    let mut first_vowel_pos = None;
    let mut last_vowel_pos = None;
    for (pos, &k) in keys.iter().enumerate() {
        if is_vowel(k) {
            if first_vowel_pos.is_none() {
                first_vowel_pos = Some(pos);
            }
            last_vowel_pos = Some(pos);
        }
    }

    // Vietnamese rule: 'r' can ONLY be initial consonant
    // If 'r' appears after a vowel, it's definitely foreign (care, rare, pure, etc.)
    if let Some(fv) = first_vowel_pos {
        for &k in &keys[fv + 1..] {
            if k == keys::R {
                return true;
            }
        }
    }

    // Check for multiple 'w' keys (redundant modifiers)
    // In Vietnamese Telex, 'w' can:
    // - Transform to 'ư' at start
    // - Apply horn to o/u
    // - Apply breve to a
    // Having 2+ 'w's in one word is suspicious (like "wow", "window")
    let w_count = keys.iter().filter(|&&k| k == keys::W).count();
    if w_count >= 2 {
        return true;
    }

    // Helper: check if key is a Telex tone key (could be modifier)
    let is_tone_key =
        |k: u16| k == keys::S || k == keys::F || k == keys::R || k == keys::X || k == keys::J;

    if let Some(lv) = last_vowel_pos {
        // Check consonant pairs after last vowel
        // Skip if first key is a tone key directly after vowel (it's likely a modifier)
        for i in lv + 1..keys.len().saturating_sub(1) {
            let first = keys[i];
            let second = keys[i + 1];

            // Skip tone key clusters when tone key is directly after vowel
            // (e.g., "test" has 's' after 'e' which is tone modifier, not part of "st" cluster)
            if i == lv + 1 && is_tone_key(first) {
                continue;
            }

            if is_foreign_final_2(first, second) {
                return true;
            }
        }

        // Check if word ends with invalid single final
        // (only if there's exactly one consonant after the last vowel)
        // Skip if that single consonant is a tone key (it's a modifier)
        let finals_after_vowel = keys.len() - lv - 1;
        if finals_after_vowel == 1 {
            let last_key = keys[keys.len() - 1];
            // Don't flag tone keys as invalid finals - they're modifiers
            if !is_tone_key(last_key) && is_invalid_single_final(last_key) {
                return true;
            }
        }
    }

    false
}

/// Check if pattern is foreign (fails Vietnamese validation)
///
/// This is the GLOBAL approach: if it's invalid Vietnamese, it's foreign.
/// No case-by-case detection needed.
#[inline]
pub fn is_foreign_pattern(initial: &[u16], vowels: &[u16], final_c: &[u16]) -> bool {
    // Check foreign initial clusters
    if initial.len() >= 2 {
        if is_foreign_initial_2(initial[0], initial[1]) {
            return true;
        }
    }

    // Check foreign final clusters
    if final_c.len() >= 2 {
        if is_foreign_final_2(final_c[0], final_c[1]) {
            return true;
        }
    }

    // Use standard Vietnamese validation
    if !initial.is_empty() {
        let valid = match initial.len() {
            1 => is_valid_initial_1(initial[0]),
            2 => is_valid_initial_2(initial[0], initial[1]),
            3 => is_valid_initial_3(initial[0], initial[1], initial[2]),
            _ => false,
        };
        if !valid {
            return true;
        }
    }

    if !final_c.is_empty() {
        let valid = match final_c.len() {
            1 => is_valid_final_1(final_c[0]),
            2 => is_valid_final_2(final_c[0], final_c[1]),
            _ => false,
        };
        if !valid {
            return true;
        }
    }

    // Check vowel pattern
    if !vowels.is_empty() && !is_valid_vowel_pattern(vowels) {
        return true;
    }

    false
}

/// Validate transformed buffer string for valid Vietnamese structure
///
/// VN(B) validation per docs/typing-behavior-flow.md:
/// Checks if the transformed buffer forms a valid Vietnamese syllable.
///
/// Returns true if the buffer is INVALID Vietnamese (should restore)
#[inline]
pub fn is_buffer_invalid_vietnamese(buffer: &str) -> bool {
    let chars: Vec<char> = buffer.chars().collect();
    if chars.is_empty() {
        return false;
    }

    // Check 1: Breve (ă/Ă) in open syllable = invalid
    // Vietnamese rule: ă can only appear in closed syllables (with final consonant)
    // Valid: ăn, ăm, ăc, ăp, ăt, ăng | Invalid: ă, lă, să (no final)
    let has_breve = chars.iter().any(|&c| c == 'ă' || c == 'Ă');
    if has_breve {
        // Check if there's a consonant after the breve vowel
        let mut found_breve = false;
        let mut has_final = false;
        for &c in &chars {
            if c == 'ă' || c == 'Ă' {
                found_breve = true;
            } else if found_breve && is_consonant_char(c) {
                has_final = true;
                break;
            }
        }
        if found_breve && !has_final {
            return true; // ă without final = invalid
        }
    }

    // Check 2: Invalid Vietnamese diphthongs
    // ưe, ưi are NOT valid Vietnamese diphthongs
    // Valid: ưa, ươ, ưu | Invalid: ưe, ưi
    let is_u_horn = |c: char| matches!(c, 'ư' | 'Ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự');
    let is_e_any = |c: char| {
        matches!(
            c,
            'e' | 'E'
                | 'é'
                | 'è'
                | 'ẻ'
                | 'ẽ'
                | 'ẹ'
                | 'ê'
                | 'Ê'
                | 'ế'
                | 'ề'
                | 'ể'
                | 'ễ'
                | 'ệ'
        )
    };
    let is_i_any = |c: char| matches!(c, 'i' | 'I' | 'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị');

    for window in chars.windows(2) {
        let (c1, c2) = (window[0], window[1]);
        // ưe or ưi (any toned version) = invalid
        if is_u_horn(c1) && (is_e_any(c2) || is_i_any(c2)) {
            return true;
        }
    }

    // Check 3: Invalid final consonants in buffer
    // Vietnamese finals: c, ch, m, n, ng, nh, p, t (and vowels can end words)
    // Invalid finals: b, d, f, g(alone), h(alone), j, l, q, r, s, v, w, x, z
    let last_char = chars.last().copied().unwrap_or('\0');
    if is_invalid_final_char(last_char) {
        return true;
    }

    // Check 4: 'w' as consonant in buffer = foreign
    // In Vietnamese, 'w' should transform to 'ư' or apply horn/breve
    // If 'w' appears as-is in buffer, it wasn't transformed → foreign
    // Exception: 'w' appearing as part of horn application is consumed, not visible
    if chars.iter().any(|&c| c == 'w' || c == 'W') {
        return true;
    }

    // Check 5: Invalid syllable structure (vowel + consonant + vowel)
    // Vietnamese syllable: (Initial) + Vowel(s) + (Final) + (Tone)
    // After vowels + consonant, can't have another vowel (like "ươman")
    let mut state = 0u8; // 0=init, 1=seen vowel, 2=seen consonant after vowel
    for &c in &chars {
        let is_vowel_char = is_vn_vowel_char(c);
        match state {
            0 => {
                if is_vowel_char {
                    state = 1;
                }
            }
            1 => {
                if !is_vowel_char && is_consonant_char(c) {
                    state = 2; // consonant after vowel
                }
            }
            2 => {
                if is_vowel_char {
                    return true; // vowel after consonant after vowel = invalid
                }
            }
            _ => {}
        }
    }

    false
}

/// Check if character is a Vietnamese vowel (including toned versions)
#[inline]
fn is_vn_vowel_char(c: char) -> bool {
    matches!(
        c.to_ascii_lowercase(),
        'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y'
    ) || is_toned_vowel(c)
}

/// Check if character is a toned vowel
#[inline]
fn is_toned_vowel(c: char) -> bool {
    matches!(
        c,
        'á' | 'à'
            | 'ả'
            | 'ã'
            | 'ạ'
            | 'ắ'
            | 'ằ'
            | 'ẳ'
            | 'ẵ'
            | 'ặ'
            | 'ấ'
            | 'ầ'
            | 'ẩ'
            | 'ẫ'
            | 'ậ'
            | 'é'
            | 'è'
            | 'ẻ'
            | 'ẽ'
            | 'ẹ'
            | 'ế'
            | 'ề'
            | 'ể'
            | 'ễ'
            | 'ệ'
            | 'í'
            | 'ì'
            | 'ỉ'
            | 'ĩ'
            | 'ị'
            | 'ó'
            | 'ò'
            | 'ỏ'
            | 'õ'
            | 'ọ'
            | 'ố'
            | 'ồ'
            | 'ổ'
            | 'ỗ'
            | 'ộ'
            | 'ớ'
            | 'ờ'
            | 'ở'
            | 'ỡ'
            | 'ợ'
            | 'ú'
            | 'ù'
            | 'ủ'
            | 'ũ'
            | 'ụ'
            | 'ứ'
            | 'ừ'
            | 'ử'
            | 'ữ'
            | 'ự'
            | 'ý'
            | 'ỳ'
            | 'ỷ'
            | 'ỹ'
            | 'ỵ'
    )
}

/// Check if character is an invalid Vietnamese final consonant
#[inline]
fn is_invalid_final_char(c: char) -> bool {
    matches!(
        c.to_ascii_lowercase(),
        'b' | 'd' | 'f' | 'j' | 'l' | 'q' | 'r' | 's' | 'v' | 'w' | 'x' | 'z'
    )
}

/// Helper: check if character is a Vietnamese consonant
#[inline]
fn is_consonant_char(c: char) -> bool {
    matches!(
        c.to_ascii_lowercase(),
        'b' | 'c'
            | 'd'
            | 'đ'
            | 'g'
            | 'h'
            | 'k'
            | 'l'
            | 'm'
            | 'n'
            | 'p'
            | 'q'
            | 'r'
            | 's'
            | 't'
            | 'v'
            | 'x'
    )
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
    InvalidCircumflexClosed,
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

    // M9: Check circumflex closed syllable rule
    // Circumflex vowel + closed syllable + no tone = invalid
    if is_circumflex_invalid_in_closed_syllable(has_breve_or_circumflex, final_c, tone) {
        return MatrixValidation::InvalidCircumflexClosed;
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
        assert!(is_valid_final_1(keys::K)); // K is valid for place names like Đắk Lắk
        assert!(is_valid_final_1(keys::M));
        assert!(is_valid_final_1(keys::N));
        assert!(is_valid_final_1(keys::P));
        assert!(is_valid_final_1(keys::T));

        // Invalid finals
        assert!(!is_valid_final_1(keys::B));
        assert!(!is_valid_final_1(keys::D));
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

    #[test]
    fn test_circumflex_closed_syllable_invalid() {
        // "kêp" = k + ê + p, no tone → INVALID
        let result = validate(
            &[keys::K], // initial: k
            &[keys::E], // vowels: ê (circumflex)
            &[keys::P], // final: p
            0,          // tone: none
            true,       // has_circumflex = true
        );
        assert_eq!(result, MatrixValidation::InvalidCircumflexClosed);

        // "bêt" = b + ê + t, no tone → INVALID
        let result = validate(&[keys::B], &[keys::E], &[keys::T], 0, true);
        assert_eq!(result, MatrixValidation::InvalidCircumflexClosed);

        // "lêm" = l + ê + m, no tone → INVALID
        let result = validate(&[keys::L], &[keys::E], &[keys::M], 0, true);
        assert_eq!(result, MatrixValidation::InvalidCircumflexClosed);
    }

    #[test]
    fn test_circumflex_closed_syllable_valid_with_tone() {
        // "kếp" = k + ê + p + sắc → VALID
        let result = validate(
            &[keys::K],
            &[keys::E],
            &[keys::P],
            1, // sắc
            true,
        );
        assert_eq!(result, MatrixValidation::Valid);

        // "bếp" = b + ê + p + sắc → VALID
        let result = validate(
            &[keys::B],
            &[keys::E],
            &[keys::P],
            1, // sắc
            true,
        );
        assert_eq!(result, MatrixValidation::Valid);

        // "bệp" = b + ê + p + nặng → VALID
        let result = validate(
            &[keys::B],
            &[keys::E],
            &[keys::P],
            5, // nặng
            true,
        );
        assert_eq!(result, MatrixValidation::Valid);
    }

    #[test]
    fn test_circumflex_open_syllable_valid() {
        // "bê" = b + ê, no final, no tone → VALID (open syllable)
        let result = validate(
            &[keys::B],
            &[keys::E],
            &[], // no final
            0,
            true,
        );
        assert_eq!(result, MatrixValidation::Valid);
    }

    #[test]
    fn test_no_circumflex_closed_syllable_valid() {
        // "kep" = k + e + p, no circumflex → VALID
        let result = validate(
            &[keys::K],
            &[keys::E],
            &[keys::P],
            0,
            false, // no circumflex
        );
        assert_eq!(result, MatrixValidation::Valid);
    }
}
