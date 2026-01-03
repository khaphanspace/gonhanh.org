//! Key category mapping for Telex input method
//!
//! Maps each key to its functional category for dispatch.
//! Telex uses modifier keys (s, f, r, x, j) for tones and
//! doubled vowels (aa, ee, oo) for circumflex marks.

/// Key category enum for dispatch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyCategory {
    /// Normal letter - pass through
    Letter = 0,
    /// Tone modifier (s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng)
    Tone = 1,
    /// Vowel that can trigger circumflex (a, e, o)
    Circumflex = 2,
    /// Horn modifier (w for ơ, ư)
    Horn = 3,
    /// Stroke modifier (d for đ)
    Stroke = 4,
    /// Breve modifier (w after a for ă)
    Breve = 5,
    /// Word boundary (space, punctuation)
    Boundary = 6,
    /// Unknown/invalid key
    Invalid = 255,
}

/// Key category lookup table for Telex (38 entries: a-z + common punctuation)
/// Index 0-25: a-z
/// Index 26-37: common punctuation/special keys
pub const KEY_CAT_TELEX: [KeyCategory; 38] = [
    KeyCategory::Circumflex, // a - can trigger â
    KeyCategory::Letter,     // b
    KeyCategory::Letter,     // c
    KeyCategory::Stroke,     // d - triggers đ
    KeyCategory::Circumflex, // e - can trigger ê
    KeyCategory::Tone,       // f - huyền
    KeyCategory::Letter,     // g
    KeyCategory::Letter,     // h
    KeyCategory::Circumflex, // i - vowel (goes to Vow state)
    KeyCategory::Tone,       // j - nặng
    KeyCategory::Letter,     // k
    KeyCategory::Letter,     // l
    KeyCategory::Letter,     // m
    KeyCategory::Letter,     // n
    KeyCategory::Circumflex, // o - can trigger ô
    KeyCategory::Letter,     // p
    KeyCategory::Letter,     // q
    KeyCategory::Tone,       // r - hỏi
    KeyCategory::Tone,       // s - sắc
    KeyCategory::Letter,     // t
    KeyCategory::Circumflex, // u - vowel (goes to Vow state)
    KeyCategory::Letter,     // v
    KeyCategory::Horn,       // w - horn/breve
    KeyCategory::Tone,       // x - ngã
    KeyCategory::Circumflex, // y - vowel (goes to Vow state)
    KeyCategory::Letter,     // z
    // Punctuation/special (index 26-37)
    KeyCategory::Boundary,   // space
    KeyCategory::Boundary,   // .
    KeyCategory::Boundary,   // ,
    KeyCategory::Boundary,   // ;
    KeyCategory::Boundary,   // :
    KeyCategory::Boundary,   // !
    KeyCategory::Boundary,   // ?
    KeyCategory::Boundary,   // '
    KeyCategory::Boundary,   // "
    KeyCategory::Boundary,   // (
    KeyCategory::Boundary,   // )
    KeyCategory::Boundary,   // -
];

/// Get key category for a character (Telex method)
#[inline]
pub fn get_category(c: char) -> KeyCategory {
    let lower = c.to_ascii_lowercase();
    if lower >= 'a' && lower <= 'z' {
        KEY_CAT_TELEX[(lower as u8 - b'a') as usize]
    } else {
        match c {
            ' ' => KeyCategory::Boundary,
            '.' | ',' | ';' | ':' | '!' | '?' => KeyCategory::Boundary,
            '\'' | '"' | '(' | ')' | '-' => KeyCategory::Boundary,
            _ => KeyCategory::Invalid,
        }
    }
}

/// Check if key is a tone modifier
#[inline]
pub fn is_tone_key(c: char) -> bool {
    matches!(get_category(c), KeyCategory::Tone)
}

/// Check if key triggers word boundary
#[inline]
pub fn is_boundary(c: char) -> bool {
    matches!(get_category(c), KeyCategory::Boundary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_keys() {
        assert!(is_tone_key('s')); // sắc
        assert!(is_tone_key('f')); // huyền
        assert!(is_tone_key('r')); // hỏi
        assert!(is_tone_key('x')); // ngã
        assert!(is_tone_key('j')); // nặng
        assert!(!is_tone_key('a'));
    }

    #[test]
    fn test_circumflex_keys() {
        assert_eq!(get_category('a'), KeyCategory::Circumflex);
        assert_eq!(get_category('e'), KeyCategory::Circumflex);
        assert_eq!(get_category('o'), KeyCategory::Circumflex);
    }

    #[test]
    fn test_boundary() {
        assert!(is_boundary(' '));
        assert!(is_boundary('.'));
        assert!(is_boundary(','));
        assert!(!is_boundary('a'));
    }

    #[test]
    fn test_stroke() {
        assert_eq!(get_category('d'), KeyCategory::Stroke);
    }

    #[test]
    fn test_horn() {
        assert_eq!(get_category('w'), KeyCategory::Horn);
    }
}
