//! Step 3: Tone and Mark Placement
//!
//! Implements Vietnamese tone placement rules based on phonological patterns.
//!
//! ## Pattern-Based Placement (from V1 phonology)
//!
//! **Diphthongs with tone on FIRST vowel (main + glide):**
//! ai, ao, au, ay, eo, êu, ia, iu, oi, ui, ưu
//!
//! **Diphthongs with tone on SECOND vowel (medial + main, compound):**
//! oa, oe, uê, uy, iê, uô/ươ
//!
//! **Triphthongs (tone on MIDDLE vowel):**
//! oai, oay, oeo, uây, uôi, ươi, ươu, iêu, yêu
//!
//! **Special triphthongs (tone on LAST vowel):**
//! uyê (khuyến, quyền)
//!
//! ## Context-Aware Handling
//! - qu-initial: 'u' is part of consonant (quý → tone on y)
//! - gi-initial: 'i' is part of consonant (già → tone on a)
//! - ua pattern: open syllable → tone on u (mùa), closed → tone on a (chuẩn)
//!
//! Also implements mark placement for circumflex, horn, breve, stroke.

// ============================================================
// PATTERN TABLES (from V1 phonology)
// ============================================================

/// Diphthongs with tone on FIRST vowel (main + glide patterns)
/// These patterns ALWAYS get mark on first vowel regardless of final consonant
const TONE_FIRST_PATTERNS: &[[char; 2]] = &[
    ['a', 'i'], // ai: mái, hài
    ['a', 'o'], // ao: cáo, sào
    ['a', 'u'], // au: sáu, màu
    ['a', 'y'], // ay: máy, tày
    ['e', 'o'], // eo: kéo, trèo
    ['ê', 'u'], // êu: nếu, kêu (ê already has circumflex)
    ['i', 'a'], // ia: kìa, mía (NOT after gi-initial)
    ['i', 'u'], // iu: dịu, kíu
    ['o', 'i'], // oi: đói, còi
    ['u', 'i'], // ui: túi, mùi
    ['ư', 'u'], // ưu: lưu, hưu (ư already has horn)
];

/// Diphthongs with tone on SECOND vowel (medial + main, compound patterns)
const TONE_SECOND_PATTERNS: &[[char; 2]] = &[
    ['o', 'a'], // oa: hoà, toá
    ['o', 'e'], // oe: khoẻ, xoè
    ['u', 'ê'], // uê: huế, tuệ
    ['u', 'y'], // uy: quý, thuỳ
    ['i', 'ê'], // iê: tiên, biết (compound)
    ['u', 'ô'], // uô: cuối, muốn (compound)
    ['ư', 'ơ'], // ươ: được, nước (compound - both have horn)
];

/// Triphthong patterns with tone position
/// Position: 0 = first, 1 = middle, 2 = last
const TRIPHTHONG_PATTERNS: &[([char; 3], usize)] = &[
    (['i', 'ê', 'u'], 1), // iêu: tiếu (middle = ê)
    (['y', 'ê', 'u'], 1), // yêu: yếu (middle = ê)
    (['o', 'a', 'i'], 1), // oai: ngoài (middle = a)
    (['o', 'a', 'y'], 1), // oay: xoáy (middle = a)
    (['o', 'e', 'o'], 1), // oeo: khoèo (middle = e)
    (['u', 'â', 'y'], 1), // uây: khuấy (middle = â)
    (['u', 'ô', 'i'], 1), // uôi: cuối (middle = ô)
    (['ư', 'ơ', 'i'], 1), // ươi: mười (middle = ơ)
    (['ư', 'ơ', 'u'], 1), // ươu: rượu (middle = ơ)
    (['u', 'y', 'ê'], 2), // uyê: khuyến, quyền (LAST = ê)
];

// ============================================================
// TYPES
// ============================================================

/// Vowel info for tone placement
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VowelInfo {
    /// Position in the string (char index)
    pub position: usize,
    /// The vowel character (lowercase base form)
    pub vowel: char,
    /// Whether this vowel already has a mark (circumflex/horn/breve)
    pub has_modifier: bool,
}

/// Context for placement decisions
#[derive(Clone, Copy, Debug)]
pub struct PlacementContext {
    /// Whether buffer starts with "qu" (u is part of consonant)
    pub has_qu_initial: bool,
    /// Whether buffer starts with "gi" (i is part of consonant)
    pub has_gi_initial: bool,
    /// Modern tone placement style (affects oa, oe, uy patterns)
    /// - Modern: hoà, hoè, thuỳ (tone on second vowel)
    /// - Traditional: hòa, hòe, thùy (tone on first vowel)
    pub modern: bool,
}

impl Default for PlacementContext {
    fn default() -> Self {
        Self {
            has_qu_initial: false,
            has_gi_initial: false,
            modern: true, // Default to modern style (oà, oé, uỳ)
        }
    }
}

/// Find the position where tone should be placed
///
/// Uses pattern-based lookup from TONE_*_PATTERNS and TRIPHTHONG_PATTERNS.
/// Supports context-aware handling for qu-initial and gi-initial.
///
/// ## Rules (in order of priority)
/// 1. Single vowel → place on it
/// 2. Triphthong patterns → use TRIPHTHONG_PATTERNS lookup
/// 3. Diphthong TONE_FIRST_PATTERNS → always first vowel
/// 4. Modifier preference → vowel with modifier (â, ê, ô, ơ, ư, ă) gets priority
/// 5. Diphthong TONE_SECOND_PATTERNS → second vowel
/// 6. Default for diphthong: with final → first, without → second
pub fn find_tone_position(vowels: &[VowelInfo], has_final: bool) -> Option<usize> {
    find_tone_position_with_context(vowels, has_final, &PlacementContext::default())
}

/// Find tone position with context (qu-initial, gi-initial awareness)
pub fn find_tone_position_with_context(
    vowels: &[VowelInfo],
    has_final: bool,
    ctx: &PlacementContext,
) -> Option<usize> {
    match vowels.len() {
        0 => None,

        // Rule 1: Single vowel
        1 => Some(vowels[0].position),

        // Diphthong: use pattern tables
        2 => find_diphthong_position(vowels, has_final, ctx),

        // Triphthong: use pattern lookup, then fallback to middle
        3 => find_triphthong_position(vowels),

        // 4+ vowels: check triphthong in first 3, else fallback
        _ => {
            // Try matching triphthong in first 3 vowels
            if let Some(pos) = find_triphthong_position(&vowels[0..3]) {
                return Some(pos);
            }
            // Fallback: middle vowel
            Some(vowels[vowels.len() / 2].position)
        }
    }
}

/// Find tone position for diphthongs using pattern tables
fn find_diphthong_position(
    vowels: &[VowelInfo],
    has_final: bool,
    ctx: &PlacementContext,
) -> Option<usize> {
    let v1 = vowels[0].vowel;
    let v2 = vowels[1].vowel;
    let pair = [v1, v2];

    // Context: gi-initial makes first 'i' part of consonant
    // Example: "già" → treat as single vowel 'a', not diphthong "ia"
    if ctx.has_gi_initial && v1 == 'i' {
        return Some(vowels[1].position);
    }

    // Context: qu-initial makes first 'u' part of consonant
    // Example: "quý" → treat as single vowel 'y', not diphthong "uy"
    if ctx.has_qu_initial && v1 == 'u' {
        return Some(vowels[1].position);
    }

    // Rule: TONE_FIRST_PATTERNS always get mark on first vowel
    // These are main+glide patterns (ai, ao, au, ay, eo, ia, iu, oi, ui)
    if TONE_FIRST_PATTERNS
        .iter()
        .any(|p| p[0] == pair[0] && p[1] == pair[1])
    {
        return Some(vowels[0].position);
    }

    // Rule: Modifier preference (vowel with diacritic gets priority)
    if vowels[0].has_modifier && !vowels[1].has_modifier {
        return Some(vowels[0].position);
    }
    if vowels[1].has_modifier && !vowels[0].has_modifier {
        return Some(vowels[1].position);
    }

    // Rule: Modern vs Traditional placement for oa, oe, uy
    // - Modern: hoà, hoè, thuỳ (second vowel)
    // - Traditional: hòa, hòe, thùy (first vowel)
    let is_modern_pattern = (v1 == 'o' && (v2 == 'a' || v2 == 'e')) || (v1 == 'u' && v2 == 'y');
    if is_modern_pattern {
        return if ctx.modern {
            Some(vowels[1].position) // Modern: second vowel
        } else {
            Some(vowels[0].position) // Traditional: first vowel
        };
    }

    // Rule: TONE_SECOND_PATTERNS get mark on second vowel (compound vowels)
    // iê, uê, uô, ươ → always second vowel (not affected by modern setting)
    if TONE_SECOND_PATTERNS
        .iter()
        .any(|p| p[0] == pair[0] && p[1] == pair[1])
    {
        return Some(vowels[1].position);
    }

    // Special: "ua" pattern - context-dependent
    // Open syllable (mùa): u is main → tone on u
    // Closed syllable (chuẩn): a is main → tone on a
    if v1 == 'u' && v2 == 'a' {
        return if has_final {
            Some(vowels[1].position) // Closed: tone on 'a'
        } else {
            Some(vowels[0].position) // Open: tone on 'u'
        };
    }

    // Default: with final → first, without → second
    if has_final {
        Some(vowels[0].position)
    } else {
        Some(vowels[1].position)
    }
}

/// Find tone position for triphthongs using pattern lookup
fn find_triphthong_position(vowels: &[VowelInfo]) -> Option<usize> {
    if vowels.len() < 3 {
        return None;
    }

    let v1 = vowels[0].vowel;
    let v2 = vowels[1].vowel;
    let v3 = vowels[2].vowel;
    let triple = [v1, v2, v3];

    // Pattern table lookup
    for (pattern, pos_idx) in TRIPHTHONG_PATTERNS {
        if pattern[0] == triple[0] && pattern[1] == triple[1] && pattern[2] == triple[2] {
            return Some(vowels[*pos_idx].position);
        }
    }

    // Fallback: check if first two form a TONE_FIRST pattern
    let pair = [v1, v2];
    if TONE_FIRST_PATTERNS
        .iter()
        .any(|p| p[0] == pair[0] && p[1] == pair[1])
    {
        return Some(vowels[0].position);
    }

    // Fallback: modifier preference
    if vowels[1].has_modifier {
        return Some(vowels[1].position);
    }
    if vowels[0].has_modifier {
        return Some(vowels[0].position);
    }

    // Default: middle vowel
    Some(vowels[1].position)
}

/// Detect placement context from buffer (qu-initial, gi-initial)
///
/// Call this before `find_tone_position_with_context` to get proper context.
pub fn detect_context(buffer: &str) -> PlacementContext {
    let lower = buffer.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();

    PlacementContext {
        // "qu" initial: u is part of consonant (quý, quà, quốc)
        has_qu_initial: chars.len() >= 2 && chars[0] == 'q' && chars[1] == 'u',
        // "gi" initial: i is part of consonant (già, giờ, giữ)
        // But NOT "gì" (single vowel), so need vowel after
        has_gi_initial: chars.len() >= 3
            && chars[0] == 'g'
            && chars[1] == 'i'
            && is_vowel(chars[2]),
        modern: true, // Default to modern style (oà, oé, uỳ), caller can override for traditional (òa, óe, ùy)
    }
}

/// Check if character is a base vowel
#[inline]
pub fn is_vowel(c: char) -> bool {
    matches!(
        c.to_ascii_lowercase(),
        'a' | 'e'
            | 'i'
            | 'o'
            | 'u'
            | 'y'
            | '\u{0103}'
            | '\u{00e2}'
            | '\u{00ea}'
            | '\u{00f4}'
            | '\u{01a1}'
            | '\u{01b0}'
    )
}

/// Check if character is a modified vowel (â, ê, ô, ơ, ư, ă)
#[inline]
pub fn has_vowel_modifier(c: char) -> bool {
    let lower = c.to_ascii_lowercase();
    matches!(
        lower,
        '\u{0103}' | // ă
        '\u{00e2}' | // â
        '\u{00ea}' | // ê
        '\u{00f4}' | // ô
        '\u{01a1}' | // ơ
        '\u{01b0}' // ư
    ) || is_modified_vowel(c)
}

/// Check if character has any tone mark
#[inline]
fn is_modified_vowel(c: char) -> bool {
    // Vietnamese vowels with tone marks
    matches!(
        c,
        // a variants with marks
        '\u{00e1}' | '\u{00c1}' | // á, Á
        '\u{00e0}' | '\u{00c0}' | // à, À
        '\u{1ea3}' | '\u{1ea2}' | // ả, Ả
        '\u{00e3}' | '\u{00c3}' | // ã, Ã
        '\u{1ea1}' | '\u{1ea0}' | // ạ, Ạ
        // ă variants
        '\u{1eaf}' | '\u{1eae}' | // ắ, Ắ
        '\u{1eb1}' | '\u{1eb0}' | // ằ, Ằ
        '\u{1eb3}' | '\u{1eb2}' | // ẳ, Ẳ
        '\u{1eb5}' | '\u{1eb4}' | // ẵ, Ẵ
        '\u{1eb7}' | '\u{1eb6}' | // ặ, Ặ
        // â variants
        '\u{1ea5}' | '\u{1ea4}' | // ấ, Ấ
        '\u{1ea7}' | '\u{1ea6}' | // ầ, Ầ
        '\u{1ea9}' | '\u{1ea8}' | // ẩ, Ẩ
        '\u{1eab}' | '\u{1eaa}' | // ẫ, Ẫ
        '\u{1ead}' | '\u{1eac}' | // ậ, Ậ
        // e variants
        '\u{00e9}' | '\u{00c9}' | // é, É
        '\u{00e8}' | '\u{00c8}' | // è, È
        '\u{1ebb}' | '\u{1eba}' | // ẻ, Ẻ
        '\u{1ebd}' | '\u{1ebc}' | // ẽ, Ẽ
        '\u{1eb9}' | '\u{1eb8}' | // ẹ, Ẹ
        // ê variants
        '\u{1ebf}' | '\u{1ebe}' | // ế, Ế
        '\u{1ec1}' | '\u{1ec0}' | // ề, Ề
        '\u{1ec3}' | '\u{1ec2}' | // ể, Ể
        '\u{1ec5}' | '\u{1ec4}' | // ễ, Ễ
        '\u{1ec7}' | '\u{1ec6}' | // ệ, Ệ
        // i variants
        '\u{00ed}' | '\u{00cd}' | // í, Í
        '\u{00ec}' | '\u{00cc}' | // ì, Ì
        '\u{1ec9}' | '\u{1ec8}' | // ỉ, Ỉ
        '\u{0129}' | '\u{0128}' | // ĩ, Ĩ
        '\u{1ecb}' | '\u{1eca}' | // ị, Ị
        // o variants
        '\u{00f3}' | '\u{00d3}' | // ó, Ó
        '\u{00f2}' | '\u{00d2}' | // ò, Ò
        '\u{1ecf}' | '\u{1ece}' | // ỏ, Ỏ
        '\u{00f5}' | '\u{00d5}' | // õ, Õ
        '\u{1ecd}' | '\u{1ecc}' | // ọ, Ọ
        // ô variants
        '\u{1ed1}' | '\u{1ed0}' | // ố, Ố
        '\u{1ed3}' | '\u{1ed2}' | // ồ, Ồ
        '\u{1ed5}' | '\u{1ed4}' | // ổ, Ổ
        '\u{1ed7}' | '\u{1ed6}' | // ỗ, Ỗ
        '\u{1ed9}' | '\u{1ed8}' | // ộ, Ộ
        // ơ variants
        '\u{1edb}' | '\u{1eda}' | // ớ, Ớ
        '\u{1edd}' | '\u{1edc}' | // ờ, Ờ
        '\u{1edf}' | '\u{1ede}' | // ở, Ở
        '\u{1ee1}' | '\u{1ee0}' | // ỡ, Ỡ
        '\u{1ee3}' | '\u{1ee2}' | // ợ, Ợ
        // u variants
        '\u{00fa}' | '\u{00da}' | // ú, Ú
        '\u{00f9}' | '\u{00d9}' | // ù, Ù
        '\u{1ee7}' | '\u{1ee6}' | // ủ, Ủ
        '\u{0169}' | '\u{0168}' | // ũ, Ũ
        '\u{1ee5}' | '\u{1ee4}' | // ụ, Ụ
        // ư variants
        '\u{1ee9}' | '\u{1ee8}' | // ứ, Ứ
        '\u{1eeb}' | '\u{1eea}' | // ừ, Ừ
        '\u{1eed}' | '\u{1eec}' | // ử, Ử
        '\u{1eef}' | '\u{1eee}' | // ữ, Ữ
        '\u{1ef1}' | '\u{1ef0}' | // ự, Ự
        // y variants
        '\u{00fd}' | '\u{00dd}' | // ý, Ý
        '\u{1ef3}' | '\u{1ef2}' | // ỳ, Ỳ
        '\u{1ef7}' | '\u{1ef6}' | // ỷ, Ỷ
        '\u{1ef9}' | '\u{1ef8}' | // ỹ, Ỹ
        '\u{1ef5}' | '\u{1ef4}' // ỵ, Ỵ
    )
}

/// Extract vowels from a string with their positions
pub fn extract_vowels(s: &str) -> Vec<VowelInfo> {
    s.chars()
        .enumerate()
        .filter(|(_, c)| is_vowel(*c))
        .map(|(pos, c)| VowelInfo {
            position: pos,
            vowel: get_base_vowel(c),
            has_modifier: has_vowel_modifier(c),
        })
        .collect()
}

/// Get base vowel without tone marks (preserves circumflex/horn/breve modifiers)
#[inline]
pub fn get_base_vowel(c: char) -> char {
    match c {
        // a with tones → a
        'á' | 'à' | 'ả' | 'ã' | 'ạ' | 'Á' | 'À' | 'Ả' | 'Ã' | 'Ạ' => 'a',
        // ă with tones → ă
        'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' | 'Ắ' | 'Ằ' | 'Ẳ' | 'Ẵ' | 'Ặ' => 'ă',
        // â with tones → â
        'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' | 'Ấ' | 'Ầ' | 'Ẩ' | 'Ẫ' | 'Ậ' => 'â',
        // e with tones → e
        'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' | 'É' | 'È' | 'Ẻ' | 'Ẽ' | 'Ẹ' => 'e',
        // ê with tones → ê
        'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' | 'Ế' | 'Ề' | 'Ể' | 'Ễ' | 'Ệ' => 'ê',
        // i with tones → i
        'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' | 'Í' | 'Ì' | 'Ỉ' | 'Ĩ' | 'Ị' => 'i',
        // o with tones → o
        'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' | 'Ó' | 'Ò' | 'Ỏ' | 'Õ' | 'Ọ' => 'o',
        // ô with tones → ô
        'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' | 'Ố' | 'Ồ' | 'Ổ' | 'Ỗ' | 'Ộ' => 'ô',
        // ơ with tones → ơ
        'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' | 'Ớ' | 'Ờ' | 'Ở' | 'Ỡ' | 'Ợ' => 'ơ',
        // u with tones → u
        'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' | 'Ú' | 'Ù' | 'Ủ' | 'Ũ' | 'Ụ' => 'u',
        // ư with tones → ư
        'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' | 'Ứ' | 'Ừ' | 'Ử' | 'Ữ' | 'Ự' => 'ư',
        // y with tones → y
        'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' | 'Ý' | 'Ỳ' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'y',
        // Uppercase base vowels - return lowercase equivalent
        'A' => 'a',
        'Ă' => 'ă',
        'Â' => 'â',
        'E' => 'e',
        'Ê' => 'ê',
        'I' => 'i',
        'O' => 'o',
        'Ô' => 'ô',
        'Ơ' => 'ơ',
        'U' => 'u',
        'Ư' => 'ư',
        'Y' => 'y',
        // Already lowercase base - return as is
        _ => c,
    }
}

/// Check if buffer has a final consonant after the last vowel
pub fn has_final_consonant(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return false;
    }

    // Find last vowel position
    let last_vowel_pos = chars.iter().rposition(|c| is_vowel(*c));

    match last_vowel_pos {
        Some(pos) => {
            // Check if there's a consonant after the last vowel
            let after_vowel = &chars[pos + 1..];
            after_vowel
                .iter()
                .any(|c| c.is_ascii_alphabetic() && !is_vowel(*c))
        }
        None => false,
    }
}

// ============================================================
// MARK PLACEMENT
// ============================================================

/// Apply circumflex mark to last applicable vowel (a→â, o→ô, e→ê)
pub fn apply_circumflex(s: &str) -> Option<(usize, char)> {
    for (i, c) in s.chars().rev().enumerate() {
        let pos = s.chars().count() - 1 - i;
        match c {
            'a' => return Some((pos, '\u{00e2}')), // â
            'A' => return Some((pos, '\u{00c2}')), // Â
            'o' => return Some((pos, '\u{00f4}')), // ô
            'O' => return Some((pos, '\u{00d4}')), // Ô
            'e' => return Some((pos, '\u{00ea}')), // ê
            'E' => return Some((pos, '\u{00ca}')), // Ê
            _ => continue,
        }
    }
    None
}

/// Apply horn mark to last applicable vowel (o→ơ, u→ư)
pub fn apply_horn(s: &str) -> Option<(usize, char)> {
    for (i, c) in s.chars().rev().enumerate() {
        let pos = s.chars().count() - 1 - i;
        match c {
            'o' => return Some((pos, '\u{01a1}')), // ơ
            'O' => return Some((pos, '\u{01a0}')), // Ơ
            'u' => return Some((pos, '\u{01b0}')), // ư
            'U' => return Some((pos, '\u{01af}')), // Ư
            _ => continue,
        }
    }
    None
}

/// Apply breve mark to 'a' (a→ă)
pub fn apply_breve(s: &str) -> Option<(usize, char)> {
    for (i, c) in s.chars().rev().enumerate() {
        let pos = s.chars().count() - 1 - i;
        match c {
            'a' => return Some((pos, '\u{0103}')), // ă
            'A' => return Some((pos, '\u{0102}')), // Ă
            _ => continue,
        }
    }
    None
}

/// Apply stroke to 'd' (d→đ)
pub fn apply_stroke(s: &str) -> Option<(usize, char)> {
    for (i, c) in s.chars().enumerate() {
        match c {
            'd' => return Some((i, '\u{0111}')), // đ
            'D' => return Some((i, '\u{0110}')), // Đ
            _ => continue,
        }
    }
    None
}

/// Check if buffer contains "uo" compound that needs horn on both
pub fn is_uo_compound(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.contains("uo") || lower.contains("u\u{01a1}")
}

// ============================================================
// TONE APPLICATION
// ============================================================

/// Apply a tone (1-5) to a vowel character
/// Returns the toned vowel, or the original if no mapping exists
pub fn apply_tone_to_vowel(vowel: char, tone: u8) -> char {
    // Get base vowel without existing tone
    let base = get_base_vowel(vowel);
    let is_upper = vowel.is_uppercase();

    let result = match (base, tone) {
        // a variants
        ('a', 0) => 'a',
        ('a', 1) => 'á',
        ('a', 2) => 'à',
        ('a', 3) => 'ả',
        ('a', 4) => 'ã',
        ('a', 5) => 'ạ',

        // ă variants
        ('ă', 0) => 'ă',
        ('ă', 1) => 'ắ',
        ('ă', 2) => 'ằ',
        ('ă', 3) => 'ẳ',
        ('ă', 4) => 'ẵ',
        ('ă', 5) => 'ặ',

        // â variants
        ('â', 0) => 'â',
        ('â', 1) => 'ấ',
        ('â', 2) => 'ầ',
        ('â', 3) => 'ẩ',
        ('â', 4) => 'ẫ',
        ('â', 5) => 'ậ',

        // e variants
        ('e', 0) => 'e',
        ('e', 1) => 'é',
        ('e', 2) => 'è',
        ('e', 3) => 'ẻ',
        ('e', 4) => 'ẽ',
        ('e', 5) => 'ẹ',

        // ê variants
        ('ê', 0) => 'ê',
        ('ê', 1) => 'ế',
        ('ê', 2) => 'ề',
        ('ê', 3) => 'ể',
        ('ê', 4) => 'ễ',
        ('ê', 5) => 'ệ',

        // i variants
        ('i', 0) => 'i',
        ('i', 1) => 'í',
        ('i', 2) => 'ì',
        ('i', 3) => 'ỉ',
        ('i', 4) => 'ĩ',
        ('i', 5) => 'ị',

        // o variants
        ('o', 0) => 'o',
        ('o', 1) => 'ó',
        ('o', 2) => 'ò',
        ('o', 3) => 'ỏ',
        ('o', 4) => 'õ',
        ('o', 5) => 'ọ',

        // ô variants
        ('ô', 0) => 'ô',
        ('ô', 1) => 'ố',
        ('ô', 2) => 'ồ',
        ('ô', 3) => 'ổ',
        ('ô', 4) => 'ỗ',
        ('ô', 5) => 'ộ',

        // ơ variants
        ('ơ', 0) => 'ơ',
        ('ơ', 1) => 'ớ',
        ('ơ', 2) => 'ờ',
        ('ơ', 3) => 'ở',
        ('ơ', 4) => 'ỡ',
        ('ơ', 5) => 'ợ',

        // u variants
        ('u', 0) => 'u',
        ('u', 1) => 'ú',
        ('u', 2) => 'ù',
        ('u', 3) => 'ủ',
        ('u', 4) => 'ũ',
        ('u', 5) => 'ụ',

        // ư variants
        ('ư', 0) => 'ư',
        ('ư', 1) => 'ứ',
        ('ư', 2) => 'ừ',
        ('ư', 3) => 'ử',
        ('ư', 4) => 'ữ',
        ('ư', 5) => 'ự',

        // y variants
        ('y', 0) => 'y',
        ('y', 1) => 'ý',
        ('y', 2) => 'ỳ',
        ('y', 3) => 'ỷ',
        ('y', 4) => 'ỹ',
        ('y', 5) => 'ỵ',

        // No match - return original
        _ => vowel,
    };

    // Handle uppercase
    if is_upper {
        result.to_uppercase().next().unwrap_or(result)
    } else {
        result
    }
}

/// Convert Telex key to tone value
#[inline]
pub fn telex_key_to_tone(key: u8) -> u8 {
    match key {
        b's' | b'S' => 1, // sắc
        b'f' | b'F' => 2, // huyền
        b'r' | b'R' => 3, // hỏi
        b'x' | b'X' => 4, // ngã
        b'j' | b'J' => 5, // nặng
        _ => 0,
    }
}

/// Replace a character at position in a string
pub fn replace_char_at(s: &str, pos: usize, new_char: char) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if i == pos {
            result.push(new_char);
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Tone Placement Tests =====

    #[test]
    fn test_single_vowel() {
        let vowels = vec![VowelInfo {
            position: 1,
            vowel: 'a',
            has_modifier: false,
        }];
        assert_eq!(find_tone_position(&vowels, false), Some(1));
        assert_eq!(find_tone_position(&vowels, true), Some(1));
    }

    #[test]
    fn test_diphthong_with_final() {
        // "bán" - tone on first vowel
        let vowels = vec![
            VowelInfo {
                position: 1,
                vowel: 'a',
                has_modifier: false,
            },
            VowelInfo {
                position: 2,
                vowel: 'i',
                has_modifier: false,
            },
        ];
        // With final consonant → first vowel
        assert_eq!(find_tone_position(&vowels, true), Some(1));
    }

    #[test]
    fn test_diphthong_ia_pattern() {
        // "ia" is a main+glide pattern (kìa, mía) → tone on FIRST vowel (i)
        let vowels = vec![
            VowelInfo {
                position: 0,
                vowel: 'i',
                has_modifier: false,
            },
            VowelInfo {
                position: 1,
                vowel: 'a',
                has_modifier: false,
            },
        ];
        // ia pattern → first vowel
        assert_eq!(find_tone_position(&vowels, false), Some(0));
    }

    #[test]
    fn test_diphthong_gi_initial() {
        // "gia" with gi-initial: 'i' is part of consonant → tone on 'a'
        let vowels = vec![
            VowelInfo {
                position: 1,
                vowel: 'i',
                has_modifier: false,
            },
            VowelInfo {
                position: 2,
                vowel: 'a',
                has_modifier: false,
            },
        ];
        let ctx = PlacementContext {
            has_qu_initial: false,
            has_gi_initial: true,
            modern: false,
        };
        assert_eq!(
            find_tone_position_with_context(&vowels, false, &ctx),
            Some(2)
        );
    }

    #[test]
    fn test_diphthong_oa_modern_vs_traditional() {
        let vowels = vec![
            VowelInfo {
                position: 0,
                vowel: 'o',
                has_modifier: false,
            },
            VowelInfo {
                position: 1,
                vowel: 'a',
                has_modifier: false,
            },
        ];

        // Modern: hoà → second vowel
        let modern_ctx = PlacementContext {
            modern: true,
            ..Default::default()
        };
        assert_eq!(
            find_tone_position_with_context(&vowels, false, &modern_ctx),
            Some(1),
            "oa modern → second"
        );

        // Traditional: hòa → first vowel
        let traditional_ctx = PlacementContext {
            modern: false,
            ..Default::default()
        };
        assert_eq!(
            find_tone_position_with_context(&vowels, false, &traditional_ctx),
            Some(0),
            "oa traditional → first"
        );
    }

    #[test]
    fn test_diphthong_uy_modern_vs_traditional() {
        let vowels = vec![
            VowelInfo {
                position: 0,
                vowel: 'u',
                has_modifier: false,
            },
            VowelInfo {
                position: 1,
                vowel: 'y',
                has_modifier: false,
            },
        ];

        // Modern: thuỳ → second vowel
        let modern_ctx = PlacementContext {
            modern: true,
            ..Default::default()
        };
        assert_eq!(
            find_tone_position_with_context(&vowels, false, &modern_ctx),
            Some(1),
            "uy modern → second"
        );

        // Traditional: thùy → first vowel
        let traditional_ctx = PlacementContext {
            modern: false,
            ..Default::default()
        };
        assert_eq!(
            find_tone_position_with_context(&vowels, false, &traditional_ctx),
            Some(0),
            "uy traditional → first"
        );
    }

    #[test]
    fn test_triphthong() {
        // "oai" - tone on middle vowel (a)
        let vowels = vec![
            VowelInfo {
                position: 0,
                vowel: 'o',
                has_modifier: false,
            },
            VowelInfo {
                position: 1,
                vowel: 'a',
                has_modifier: false,
            },
            VowelInfo {
                position: 2,
                vowel: 'i',
                has_modifier: false,
            },
        ];
        assert_eq!(find_tone_position(&vowels, false), Some(1));
        assert_eq!(find_tone_position(&vowels, true), Some(1));
    }

    #[test]
    fn test_modified_vowel_preference() {
        // If one vowel has modifier, prefer it
        let vowels = vec![
            VowelInfo {
                position: 0,
                vowel: 'u',
                has_modifier: false,
            },
            VowelInfo {
                position: 1,
                vowel: 'o',
                has_modifier: true,
            }, // ơ
        ];
        // Modified vowel gets preference
        assert_eq!(find_tone_position(&vowels, false), Some(1));
    }

    // ===== Mark Placement Tests =====

    #[test]
    fn test_apply_circumflex() {
        assert_eq!(apply_circumflex("ca"), Some((1, '\u{00e2}')));
        assert_eq!(apply_circumflex("co"), Some((1, '\u{00f4}')));
        assert_eq!(apply_circumflex("ce"), Some((1, '\u{00ea}')));
        assert_eq!(apply_circumflex("CA"), Some((1, '\u{00c2}')));
        assert_eq!(apply_circumflex("xyz"), None);
    }

    #[test]
    fn test_apply_horn() {
        assert_eq!(apply_horn("mo"), Some((1, '\u{01a1}')));
        assert_eq!(apply_horn("mu"), Some((1, '\u{01b0}')));
        assert_eq!(apply_horn("MO"), Some((1, '\u{01a0}')));
        assert_eq!(apply_horn("abc"), None);
    }

    #[test]
    fn test_apply_breve() {
        assert_eq!(apply_breve("la"), Some((1, '\u{0103}')));
        assert_eq!(apply_breve("LA"), Some((1, '\u{0102}')));
        assert_eq!(apply_breve("xyz"), None);
    }

    #[test]
    fn test_apply_stroke() {
        assert_eq!(apply_stroke("da"), Some((0, '\u{0111}')));
        assert_eq!(apply_stroke("DA"), Some((0, '\u{0110}')));
        assert_eq!(apply_stroke("abc"), None);
    }

    // ===== Helper Tests =====

    #[test]
    fn test_is_vowel() {
        assert!(is_vowel('a'));
        assert!(is_vowel('e'));
        assert!(is_vowel('i'));
        assert!(is_vowel('o'));
        assert!(is_vowel('u'));
        assert!(is_vowel('y'));
        assert!(is_vowel('\u{0103}')); // ă
        assert!(is_vowel('\u{00e2}')); // â
        assert!(!is_vowel('b'));
        assert!(!is_vowel('c'));
    }

    #[test]
    fn test_has_final_consonant() {
        assert!(has_final_consonant("an"));
        assert!(has_final_consonant("anh"));
        assert!(!has_final_consonant("a"));
        assert!(!has_final_consonant("ai"));
    }

    #[test]
    fn test_extract_vowels() {
        let vowels = extract_vowels("ban");
        assert_eq!(vowels.len(), 1);
        assert_eq!(vowels[0].position, 1);
        assert_eq!(vowels[0].vowel, 'a');

        let vowels = extract_vowels("oai");
        assert_eq!(vowels.len(), 3);
    }

    #[test]
    fn test_is_uo_compound() {
        assert!(is_uo_compound("tuo"));
        assert!(is_uo_compound("TUO"));
        assert!(!is_uo_compound("abc"));
    }

    // ===== Pattern Table Tests =====

    #[test]
    fn test_tone_first_patterns() {
        // ai, ao, au: all FIRST vowel patterns
        for (v1, v2, desc) in [
            ('a', 'i', "ai: mái"),
            ('a', 'o', "ao: cáo"),
            ('a', 'u', "au: sáu"),
            ('o', 'i', "oi: đói"),
            ('u', 'i', "ui: túi"),
        ] {
            let vowels = vec![
                VowelInfo {
                    position: 0,
                    vowel: v1,
                    has_modifier: false,
                },
                VowelInfo {
                    position: 1,
                    vowel: v2,
                    has_modifier: false,
                },
            ];
            assert_eq!(
                find_tone_position(&vowels, false),
                Some(0),
                "{} should place tone on first vowel",
                desc
            );
        }
    }

    #[test]
    fn test_tone_second_patterns() {
        // oa, oe: SECOND vowel patterns
        for (v1, v2, desc) in [('o', 'a', "oa: hoà"), ('o', 'e', "oe: khoẻ")] {
            let vowels = vec![
                VowelInfo {
                    position: 0,
                    vowel: v1,
                    has_modifier: false,
                },
                VowelInfo {
                    position: 1,
                    vowel: v2,
                    has_modifier: false,
                },
            ];
            assert_eq!(
                find_tone_position(&vowels, false),
                Some(1),
                "{} should place tone on second vowel",
                desc
            );
        }
    }

    #[test]
    fn test_ua_context_dependent() {
        let vowels = vec![
            VowelInfo {
                position: 0,
                vowel: 'u',
                has_modifier: false,
            },
            VowelInfo {
                position: 1,
                vowel: 'a',
                has_modifier: false,
            },
        ];

        // Open syllable (mùa) → tone on u
        assert_eq!(
            find_tone_position(&vowels, false),
            Some(0),
            "ua open syllable should place on first (mùa)"
        );

        // Closed syllable (chuẩn) → tone on a
        assert_eq!(
            find_tone_position(&vowels, true),
            Some(1),
            "ua closed syllable should place on second (chuẩn)"
        );
    }

    #[test]
    fn test_triphthong_patterns() {
        // oai → middle (a)
        let vowels = vec![
            VowelInfo {
                position: 0,
                vowel: 'o',
                has_modifier: false,
            },
            VowelInfo {
                position: 1,
                vowel: 'a',
                has_modifier: false,
            },
            VowelInfo {
                position: 2,
                vowel: 'i',
                has_modifier: false,
            },
        ];
        assert_eq!(find_tone_position(&vowels, false), Some(1), "oai → middle");
    }

    #[test]
    fn test_triphthong_uye_last() {
        // uyê → LAST (ê) - special case
        let vowels = vec![
            VowelInfo {
                position: 0,
                vowel: 'u',
                has_modifier: false,
            },
            VowelInfo {
                position: 1,
                vowel: 'y',
                has_modifier: false,
            },
            VowelInfo {
                position: 2,
                vowel: 'ê',
                has_modifier: true,
            },
        ];
        assert_eq!(
            find_tone_position(&vowels, false),
            Some(2),
            "uyê → last (ê)"
        );
    }

    #[test]
    fn test_detect_context_qu() {
        let ctx = detect_context("qua");
        assert!(ctx.has_qu_initial);
        assert!(!ctx.has_gi_initial);

        let ctx = detect_context("quý");
        assert!(ctx.has_qu_initial);
    }

    #[test]
    fn test_detect_context_gi() {
        // "gia" - gi + a vowel, gi-initial applies
        let ctx = detect_context("gia");
        assert!(ctx.has_gi_initial);
        assert!(!ctx.has_qu_initial);

        // "giau" - gi + au, gi-initial applies
        let ctx = detect_context("giau");
        assert!(ctx.has_gi_initial);

        // "gi" only has 2 chars, need 3 for gi-initial
        let ctx = detect_context("gi");
        assert!(!ctx.has_gi_initial);
    }

    #[test]
    fn test_qu_initial_placement() {
        // "quý" - u is part of consonant, tone on y
        let vowels = vec![
            VowelInfo {
                position: 1,
                vowel: 'u',
                has_modifier: false,
            },
            VowelInfo {
                position: 2,
                vowel: 'y',
                has_modifier: false,
            },
        ];
        let ctx = PlacementContext {
            has_qu_initial: true,
            has_gi_initial: false,
            modern: false,
        };
        assert_eq!(
            find_tone_position_with_context(&vowels, false, &ctx),
            Some(2),
            "quý should place tone on y"
        );
    }
}
