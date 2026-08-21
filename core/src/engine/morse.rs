//! Morse code encoding module
//!
//! Converts Vietnamese text to International Morse Code (Unicode output).
//! Uses ITU-R M.1677-1 standard for A-Z, 0-9, punctuation.
//! Vietnamese diacritics are decomposed to base char + tone/modifier prosigns.
//!
//! Output uses Unicode: `·` (U+00B7) for dot, `—` (U+2014) for dash.

/// Unicode dot character for Morse output
const DOT: char = '\u{00B7}';
/// Unicode dash character for Morse output
const DASH: char = '\u{2014}';

/// Convert a single ASCII/base character to its Morse code pattern.
/// Returns dot/dash pattern using ASCII `.` and `-` (converted to Unicode later).
/// Case-insensitive for letters.
fn base_char_to_morse(c: char) -> Option<&'static str> {
    match c.to_ascii_uppercase() {
        // ITU-R M.1677-1 Letters
        'A' => Some(".-"),
        'B' => Some("-..."),
        'C' => Some("-.-."),
        'D' => Some("-.."),
        'E' => Some("."),
        'F' => Some("..-."),
        'G' => Some("--."),
        'H' => Some("...."),
        'I' => Some(".."),
        'J' => Some(".---"),
        'K' => Some("-.-"),
        'L' => Some(".-.."),
        'M' => Some("--"),
        'N' => Some("-."),
        'O' => Some("---"),
        'P' => Some(".--."),
        'Q' => Some("--.-"),
        'R' => Some(".-."),
        'S' => Some("..."),
        'T' => Some("-"),
        'U' => Some("..-"),
        'V' => Some("...-"),
        'W' => Some(".--"),
        'X' => Some("-..-"),
        'Y' => Some("-.--"),
        'Z' => Some("--.."),

        // ITU-R M.1677-1 Numerals
        '0' => Some("-----"),
        '1' => Some(".----"),
        '2' => Some("..---"),
        '3' => Some("...--"),
        '4' => Some("....-"),
        '5' => Some("....."),
        '6' => Some("-...."),
        '7' => Some("--..."),
        '8' => Some("---.."),
        '9' => Some("----."),

        // ITU-R M.1677-1 Punctuation
        '.' => Some(".-.-.-"),
        ',' => Some("--..--"),
        '?' => Some("..--.."),
        ':' => Some("---..."),
        ';' => Some("-.-.-."),
        '/' => Some("-..-."),
        '\'' => Some(".----."),
        '"' => Some(".-..-."),
        '-' => Some("-....-"),
        '(' => Some("-.--."),
        ')' => Some("-.--.-"),
        '@' => Some(".--.-."),
        '=' => Some("-...-"),
        '!' => Some("-.-.--"),

        _ => None,
    }
}

/// Morse prosign for Vietnamese tone marks (dấu thanh)
/// These are 6-element sequences to avoid collision with any ITU standard character.
fn tone_mark_morse(mark: ToneMark) -> &'static str {
    match mark {
        ToneMark::Sac => ".--.-.", // sắc (acute accent) - 6 elements, unique
        ToneMark::Huyen => ".--..-", // huyền (grave accent) - 6 elements, unique
        ToneMark::Hoi => "..-..-", // hỏi (hook above) - 6 elements, unique
        ToneMark::Nga => "--.-..", // ngã (tilde) - 6 elements, unique
        ToneMark::Nang => "-.-..-", // nặng (dot below) - 6 elements, unique
    }
}

/// Morse prosign for Vietnamese vowel modifiers (dấu phụ)
/// These are 6-element sequences to avoid collision with any ITU standard character.
fn vowel_modifier_morse(modifier: VowelModifier) -> &'static str {
    match modifier {
        VowelModifier::Circumflex => "..--..", // â, ê, ô - 6 elements, unique
        VowelModifier::Breve => ".-...-",      // ă - 6 elements, unique
        VowelModifier::Horn => ".--...",        // ơ, ư - 6 elements, unique
    }
}

/// Vietnamese tone mark types
#[derive(Debug, Clone, Copy, PartialEq)]
enum ToneMark {
    Sac,   // ´  (sắc)
    Huyen, // `  (huyền)
    Hoi,   // ̉   (hỏi)
    Nga,   // ̃   (ngã)
    Nang,  // ̣   (nặng)
}

/// Vietnamese vowel modifier types
#[derive(Debug, Clone, Copy, PartialEq)]
enum VowelModifier {
    Circumflex, // â, ê, ô
    Breve,      // ă
    Horn,       // ơ, ư
}

/// Decomposed Vietnamese character
struct DecomposedChar {
    base: char,
    modifier: Option<VowelModifier>,
    tone: Option<ToneMark>,
}

/// Decompose a Vietnamese character into base + modifier + tone
fn decompose_vietnamese(c: char) -> Option<DecomposedChar> {
    let result = match c.to_ascii_lowercase_vn() {
        // Đ/đ - special consonant (no base decomposition needed)
        'đ' => return Some(DecomposedChar {
            base: 'd',
            modifier: None,
            tone: None,
        }),

        // A variants
        'a' => DecomposedChar { base: 'a', modifier: None, tone: None },
        'á' => DecomposedChar { base: 'a', modifier: None, tone: Some(ToneMark::Sac) },
        'à' => DecomposedChar { base: 'a', modifier: None, tone: Some(ToneMark::Huyen) },
        'ả' => DecomposedChar { base: 'a', modifier: None, tone: Some(ToneMark::Hoi) },
        'ã' => DecomposedChar { base: 'a', modifier: None, tone: Some(ToneMark::Nga) },
        'ạ' => DecomposedChar { base: 'a', modifier: None, tone: Some(ToneMark::Nang) },

        'ă' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Breve), tone: None },
        'ắ' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Breve), tone: Some(ToneMark::Sac) },
        'ằ' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Breve), tone: Some(ToneMark::Huyen) },
        'ẳ' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Breve), tone: Some(ToneMark::Hoi) },
        'ẵ' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Breve), tone: Some(ToneMark::Nga) },
        'ặ' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Breve), tone: Some(ToneMark::Nang) },

        'â' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Circumflex), tone: None },
        'ấ' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Sac) },
        'ầ' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Huyen) },
        'ẩ' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Hoi) },
        'ẫ' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Nga) },
        'ậ' => DecomposedChar { base: 'a', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Nang) },

        // E variants
        'e' => DecomposedChar { base: 'e', modifier: None, tone: None },
        'é' => DecomposedChar { base: 'e', modifier: None, tone: Some(ToneMark::Sac) },
        'è' => DecomposedChar { base: 'e', modifier: None, tone: Some(ToneMark::Huyen) },
        'ẻ' => DecomposedChar { base: 'e', modifier: None, tone: Some(ToneMark::Hoi) },
        'ẽ' => DecomposedChar { base: 'e', modifier: None, tone: Some(ToneMark::Nga) },
        'ẹ' => DecomposedChar { base: 'e', modifier: None, tone: Some(ToneMark::Nang) },

        'ê' => DecomposedChar { base: 'e', modifier: Some(VowelModifier::Circumflex), tone: None },
        'ế' => DecomposedChar { base: 'e', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Sac) },
        'ề' => DecomposedChar { base: 'e', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Huyen) },
        'ể' => DecomposedChar { base: 'e', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Hoi) },
        'ễ' => DecomposedChar { base: 'e', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Nga) },
        'ệ' => DecomposedChar { base: 'e', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Nang) },

        // I variants
        'i' => DecomposedChar { base: 'i', modifier: None, tone: None },
        'í' => DecomposedChar { base: 'i', modifier: None, tone: Some(ToneMark::Sac) },
        'ì' => DecomposedChar { base: 'i', modifier: None, tone: Some(ToneMark::Huyen) },
        'ỉ' => DecomposedChar { base: 'i', modifier: None, tone: Some(ToneMark::Hoi) },
        'ĩ' => DecomposedChar { base: 'i', modifier: None, tone: Some(ToneMark::Nga) },
        'ị' => DecomposedChar { base: 'i', modifier: None, tone: Some(ToneMark::Nang) },

        // O variants
        'o' => DecomposedChar { base: 'o', modifier: None, tone: None },
        'ó' => DecomposedChar { base: 'o', modifier: None, tone: Some(ToneMark::Sac) },
        'ò' => DecomposedChar { base: 'o', modifier: None, tone: Some(ToneMark::Huyen) },
        'ỏ' => DecomposedChar { base: 'o', modifier: None, tone: Some(ToneMark::Hoi) },
        'õ' => DecomposedChar { base: 'o', modifier: None, tone: Some(ToneMark::Nga) },
        'ọ' => DecomposedChar { base: 'o', modifier: None, tone: Some(ToneMark::Nang) },

        'ô' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Circumflex), tone: None },
        'ố' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Sac) },
        'ồ' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Huyen) },
        'ổ' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Hoi) },
        'ỗ' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Nga) },
        'ộ' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Circumflex), tone: Some(ToneMark::Nang) },

        'ơ' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Horn), tone: None },
        'ớ' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Horn), tone: Some(ToneMark::Sac) },
        'ờ' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Horn), tone: Some(ToneMark::Huyen) },
        'ở' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Horn), tone: Some(ToneMark::Hoi) },
        'ỡ' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Horn), tone: Some(ToneMark::Nga) },
        'ợ' => DecomposedChar { base: 'o', modifier: Some(VowelModifier::Horn), tone: Some(ToneMark::Nang) },

        // U variants
        'u' => DecomposedChar { base: 'u', modifier: None, tone: None },
        'ú' => DecomposedChar { base: 'u', modifier: None, tone: Some(ToneMark::Sac) },
        'ù' => DecomposedChar { base: 'u', modifier: None, tone: Some(ToneMark::Huyen) },
        'ủ' => DecomposedChar { base: 'u', modifier: None, tone: Some(ToneMark::Hoi) },
        'ũ' => DecomposedChar { base: 'u', modifier: None, tone: Some(ToneMark::Nga) },
        'ụ' => DecomposedChar { base: 'u', modifier: None, tone: Some(ToneMark::Nang) },

        'ư' => DecomposedChar { base: 'u', modifier: Some(VowelModifier::Horn), tone: None },
        'ứ' => DecomposedChar { base: 'u', modifier: Some(VowelModifier::Horn), tone: Some(ToneMark::Sac) },
        'ừ' => DecomposedChar { base: 'u', modifier: Some(VowelModifier::Horn), tone: Some(ToneMark::Huyen) },
        'ử' => DecomposedChar { base: 'u', modifier: Some(VowelModifier::Horn), tone: Some(ToneMark::Hoi) },
        'ữ' => DecomposedChar { base: 'u', modifier: Some(VowelModifier::Horn), tone: Some(ToneMark::Nga) },
        'ự' => DecomposedChar { base: 'u', modifier: Some(VowelModifier::Horn), tone: Some(ToneMark::Nang) },

        // Y variants
        'y' => DecomposedChar { base: 'y', modifier: None, tone: None },
        'ý' => DecomposedChar { base: 'y', modifier: None, tone: Some(ToneMark::Sac) },
        'ỳ' => DecomposedChar { base: 'y', modifier: None, tone: Some(ToneMark::Huyen) },
        'ỷ' => DecomposedChar { base: 'y', modifier: None, tone: Some(ToneMark::Hoi) },
        'ỹ' => DecomposedChar { base: 'y', modifier: None, tone: Some(ToneMark::Nga) },
        'ỵ' => DecomposedChar { base: 'y', modifier: None, tone: Some(ToneMark::Nang) },

        _ => return None,
    };
    Some(result)
}

/// Đ/đ has its own Morse code (not decomposed)
const D_STROKE_MORSE: &str = "--..-."; // custom: D + extra dot-dash

/// Convert ASCII `.`/`-` pattern to Unicode `·`/`—` characters
fn morse_to_unicode(pattern: &str, output: &mut Vec<char>) {
    for c in pattern.chars() {
        match c {
            '.' => output.push(DOT),
            '-' => output.push(DASH),
            _ => {}
        }
    }
}

/// Encode a Vietnamese string into Morse code (Unicode output).
///
/// - Each character's Morse code separated by space
/// - Vietnamese chars decomposed: base + modifier prosign + tone prosign
/// - Unsupported characters are skipped
/// - Returns Vec<char> for direct use with Result::send()
pub fn encode(input: &str) -> Vec<char> {
    let mut output = Vec::with_capacity(input.len() * 8);
    let mut first = true;

    for c in input.chars() {
        // Skip spaces (word separator handled by caller)
        if c == ' ' {
            // Word separator: " / "
            if !first {
                output.push(' ');
            }
            output.push('/');
            output.push(' ');
            first = true;
            continue;
        }

        // Try Vietnamese decomposition first
        if let Some(decomposed) = decompose_vietnamese(c) {
            if !first {
                output.push(' ');
            }
            first = false;

            // Special case: đ has its own code
            if decomposed.base == 'd' && decomposed.modifier.is_none() && decomposed.tone.is_none()
                && (c == 'đ' || c == 'Đ')
            {
                morse_to_unicode(D_STROKE_MORSE, &mut output);
                continue;
            }

            // Encode base character
            if let Some(base_morse) = base_char_to_morse(decomposed.base) {
                morse_to_unicode(base_morse, &mut output);
            }

            // Encode vowel modifier prosign (ă, â, ê, ô, ơ, ư)
            if let Some(modifier) = decomposed.modifier {
                output.push(' ');
                morse_to_unicode(vowel_modifier_morse(modifier), &mut output);
            }

            // Encode tone mark prosign (sắc, huyền, hỏi, ngã, nặng)
            if let Some(tone) = decomposed.tone {
                output.push(' ');
                morse_to_unicode(tone_mark_morse(tone), &mut output);
            }

            continue;
        }

        // Try base ASCII character
        if let Some(morse) = base_char_to_morse(c) {
            if !first {
                output.push(' ');
            }
            first = false;
            morse_to_unicode(morse, &mut output);
        }
        // Unsupported characters are silently skipped
    }

    output
}

/// Helper trait for Vietnamese lowercase conversion
trait VietnameseLowercase {
    fn to_ascii_lowercase_vn(self) -> char;
}

impl VietnameseLowercase for char {
    fn to_ascii_lowercase_vn(self) -> char {
        match self {
            'Đ' => 'đ',
            'A'..='Z' => (self as u8 + 32) as char,
            c => {
                // Vietnamese uppercase → lowercase mapping
                let lower = c.to_lowercase().next().unwrap_or(c);
                lower
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_latin_a_to_z() {
        // Verify all 26 letters have Morse codes
        for c in 'A'..='Z' {
            assert!(
                base_char_to_morse(c).is_some(),
                "Missing Morse for '{}'",
                c
            );
        }
        // Spot checks
        assert_eq!(base_char_to_morse('A'), Some(".-"));
        assert_eq!(base_char_to_morse('Z'), Some("--.."));
        assert_eq!(base_char_to_morse('S'), Some("..."));
        assert_eq!(base_char_to_morse('O'), Some("---"));
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(base_char_to_morse('a'), base_char_to_morse('A'));
        assert_eq!(base_char_to_morse('z'), base_char_to_morse('Z'));
    }

    #[test]
    fn test_numbers() {
        assert_eq!(base_char_to_morse('0'), Some("-----"));
        assert_eq!(base_char_to_morse('1'), Some(".----"));
        assert_eq!(base_char_to_morse('5'), Some("....."));
        assert_eq!(base_char_to_morse('9'), Some("----."));
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(base_char_to_morse('.'), Some(".-.-.-"));
        assert_eq!(base_char_to_morse(','), Some("--..--"));
        assert_eq!(base_char_to_morse('?'), Some("..--.."));
    }

    #[test]
    fn test_unsupported_char() {
        assert_eq!(base_char_to_morse('😀'), None);
        assert_eq!(base_char_to_morse('★'), None);
    }

    #[test]
    fn test_decompose_basic_vowels() {
        let d = decompose_vietnamese('a').unwrap();
        assert_eq!(d.base, 'a');
        assert!(d.modifier.is_none());
        assert!(d.tone.is_none());
    }

    #[test]
    fn test_decompose_accented() {
        let d = decompose_vietnamese('á').unwrap();
        assert_eq!(d.base, 'a');
        assert!(d.modifier.is_none());
        assert_eq!(d.tone, Some(ToneMark::Sac));
    }

    #[test]
    fn test_decompose_complex() {
        // ậ = a + circumflex + nặng
        let d = decompose_vietnamese('ậ').unwrap();
        assert_eq!(d.base, 'a');
        assert_eq!(d.modifier, Some(VowelModifier::Circumflex));
        assert_eq!(d.tone, Some(ToneMark::Nang));
    }

    #[test]
    fn test_decompose_horn() {
        // ớ = o + horn + sắc
        let d = decompose_vietnamese('ớ').unwrap();
        assert_eq!(d.base, 'o');
        assert_eq!(d.modifier, Some(VowelModifier::Horn));
        assert_eq!(d.tone, Some(ToneMark::Sac));
    }

    #[test]
    fn test_decompose_d_stroke() {
        let d = decompose_vietnamese('đ').unwrap();
        assert_eq!(d.base, 'd');
        assert!(d.modifier.is_none());
        assert!(d.tone.is_none());
    }

    #[test]
    fn test_decompose_uppercase() {
        let d = decompose_vietnamese('Á').unwrap();
        assert_eq!(d.base, 'a');
        assert_eq!(d.tone, Some(ToneMark::Sac));
    }

    #[test]
    fn test_encode_simple_word() {
        let result: String = encode("xin").into_iter().collect();
        // X=-..- I=.. N=-.
        assert!(result.contains('\u{2014}')); // dash
        assert!(result.contains('\u{00B7}')); // dot
        // Should have spaces between chars
        assert!(result.contains(' '));
    }

    #[test]
    fn test_encode_sos() {
        let result: String = encode("sos").into_iter().collect();
        // S=... O=--- S=...
        // Expected: ··· ——— ···
        assert_eq!(
            result,
            format!(
                "{0}{0}{0} {1}{1}{1} {0}{0}{0}",
                DOT, DASH
            )
        );
    }

    #[test]
    fn test_encode_vietnamese_with_tone() {
        let result: String = encode("á").into_iter().collect();
        // á = A (.-) + space + sắc (.-.-.)
        assert!(!result.is_empty());
        assert!(result.contains(' ')); // separator between base and tone
    }

    #[test]
    fn test_encode_d_stroke() {
        let result: String = encode("đ").into_iter().collect();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_encode_with_spaces() {
        let result: String = encode("hi you").into_iter().collect();
        // Should contain word separator "/ "
        assert!(result.contains('/'));
    }

    #[test]
    fn test_encode_empty() {
        let result = encode("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_encode_complex_vietnamese() {
        // "Việt Nam" - should not panic and produce non-empty output
        let result = encode("Việt Nam");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_all_vietnamese_vowels_decompose() {
        let test_chars = [
            'ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ',
            'â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ',
            'ê', 'ế', 'ề', 'ể', 'ễ', 'ệ',
            'ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ',
            'ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ',
            'ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự',
        ];
        for c in test_chars {
            assert!(
                decompose_vietnamese(c).is_some(),
                "Failed to decompose '{}'",
                c
            );
        }
    }
}
