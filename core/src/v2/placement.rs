//! Step 3: Tone and Mark Placement
//!
//! Implements Vietnamese tone placement rules (Spec Section 6.1):
//! - Rule 1: Single vowel → place on that vowel
//! - Rule 2: Diphthong + final consonant → place on FIRST vowel (ái, áo)
//! - Rule 3: Diphthong + no final → place on SECOND vowel (ia, oà)
//! - Rule 4: Triphthong → place on MIDDLE vowel (oái, uối)
//! - Rule 5: oa, oe, uy → place on SECOND vowel (always)
//!
//! Also implements mark placement for circumflex, horn, breve, stroke.

/// Vowel info for tone placement
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VowelInfo {
    /// Position in the string (char index)
    pub position: usize,
    /// The vowel character (lowercase)
    pub vowel: char,
    /// Whether this vowel already has a mark (circumflex/horn/breve)
    pub has_modifier: bool,
}

/// Find the position where tone should be placed
///
/// Implements the 5 rules from the spec:
/// 1. Single vowel → place on it
/// 2. Diphthong + final → first vowel
/// 3. Diphthong + no final → second vowel
/// 4. Triphthong → middle vowel
/// 5. oa, oe, uy → always second vowel
pub fn find_tone_position(vowels: &[VowelInfo], has_final: bool) -> Option<usize> {
    match vowels.len() {
        0 => None,

        // Rule 1: Single vowel
        1 => Some(vowels[0].position),

        // Rules 2, 3, 5: Diphthong
        2 => {
            let v1 = vowels[0].vowel;
            let v2 = vowels[1].vowel;

            // Rule 5: oa, oe, uy → always on SECOND
            if is_special_diphthong(v1, v2) {
                return Some(vowels[1].position);
            }

            // If either vowel has modifier (â, ê, ô, ơ, ư, ă), prefer it
            if vowels[0].has_modifier && !vowels[1].has_modifier {
                return Some(vowels[0].position);
            }
            if vowels[1].has_modifier && !vowels[0].has_modifier {
                return Some(vowels[1].position);
            }

            // Rule 2: Diphthong + final → FIRST vowel
            // Rule 3: Diphthong + no final → SECOND vowel
            if has_final {
                Some(vowels[0].position)
            } else {
                Some(vowels[1].position)
            }
        }

        // Rule 4: Triphthong → MIDDLE vowel
        3 => Some(vowels[1].position),

        _ => None,
    }
}

/// Check for special diphthongs that always place tone on second vowel
#[inline]
fn is_special_diphthong(v1: char, v2: char) -> bool {
    // oa, oe → always on second (a, e)
    (v1 == 'o' && (v2 == 'a' || v2 == 'e')) ||
    // uy → always on second (y)
    (v1 == 'u' && v2 == 'y') ||
    // ươ → always on ơ (second): được, nước, ướt
    (v1 == 'ư' && v2 == 'ơ') ||
    // iê → always on ê (second): tiếng, biết
    (v1 == 'i' && v2 == 'ê')
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
    fn test_diphthong_no_final() {
        // "ia" - tone on second vowel
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
        // Without final → second vowel
        assert_eq!(find_tone_position(&vowels, false), Some(1));
    }

    #[test]
    fn test_special_diphthong_oa() {
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
        // oa → always on second (a)
        assert_eq!(find_tone_position(&vowels, false), Some(1));
        assert_eq!(find_tone_position(&vowels, true), Some(1));
    }

    #[test]
    fn test_special_diphthong_uy() {
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
        // uy → always on second (y)
        assert_eq!(find_tone_position(&vowels, false), Some(1));
        assert_eq!(find_tone_position(&vowels, true), Some(1));
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
}
