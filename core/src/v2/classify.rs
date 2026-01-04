//! Step 0: Key Classification
//!
//! Classifies each keystroke into KeyType based on input method (Telex/VNI).
//! Determines how the engine should process each key.

use super::types::{KeyType, MarkType};

/// Terminator characters - shared between methods
const TERMINATORS: &[u8] = &[
    b' ', b'\n', b'\t', // whitespace
    b'.', b',', b';', b':', b'!', b'?', // punctuation
    b'"', b'\'', b'(', b')', b'[', b']', b'{', b'}', // brackets/quotes
];

/// Check if key is a terminator
#[inline]
fn is_terminator(key: u8) -> bool {
    TERMINATORS.contains(&key)
}

/// Input method
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Method {
    #[default]
    Telex = 0,
    Vni = 1,
}

/// Classify keystroke based on input method
#[inline]
pub fn classify_key(key: u8, prev_key: Option<u8>, method: Method) -> KeyType {
    match method {
        Method::Telex => classify_telex(key, prev_key),
        Method::Vni => classify_vni(key, prev_key),
    }
}

/// Classify key for Telex method
#[inline]
fn classify_telex(key: u8, prev: Option<u8>) -> KeyType {
    // Normalize to lowercase for matching
    let lower = key.to_ascii_lowercase();
    let prev_lower = prev.map(|p| p.to_ascii_lowercase());

    match lower {
        // Tone keys: s(sắc), f(huyền), r(hỏi), x(ngã), j(nặng)
        b's' | b'f' | b'r' | b'x' | b'j' => KeyType::Tone(lower),

        // Mark keys (context-dependent)
        b'd' if prev_lower == Some(b'd') => KeyType::Mark(MarkType::Stroke),
        b'w' => KeyType::Mark(MarkType::HornOrBreve),
        b'a' if prev_lower == Some(b'a') => KeyType::Mark(MarkType::Circumflex),
        b'o' if prev_lower == Some(b'o') => KeyType::Mark(MarkType::Circumflex),
        b'e' if prev_lower == Some(b'e') => KeyType::Mark(MarkType::Circumflex),

        // Terminators - word boundaries
        _ if is_terminator(key) => KeyType::Terminator,

        // Letters
        b'a'..=b'z' | b'A'..=b'Z' => KeyType::Letter(key),

        // Passthrough (digits, symbols)
        _ => KeyType::Passthrough,
    }
}

/// Classify key for VNI method
#[inline]
fn classify_vni(key: u8, _prev: Option<u8>) -> KeyType {
    match key {
        // Tone keys: 1-5
        b'1' => KeyType::Tone(1), // sắc
        b'2' => KeyType::Tone(2), // huyền
        b'3' => KeyType::Tone(3), // hỏi
        b'4' => KeyType::Tone(4), // ngã
        b'5' => KeyType::Tone(5), // nặng

        // Mark keys: 6-9, 0
        b'6' => KeyType::Mark(MarkType::Circumflex), // â, ô, ê
        b'7' => KeyType::Mark(MarkType::HornOrBreve), // ơ, ư
        b'8' => KeyType::Mark(MarkType::HornOrBreve), // ă (breve)
        b'9' => KeyType::Mark(MarkType::Stroke),     // đ (stroke)
        b'0' => KeyType::Tone(0),                    // remove tone

        // Terminators (same as Telex)
        _ if is_terminator(key) => KeyType::Terminator,

        // Letters
        b'a'..=b'z' | b'A'..=b'Z' => KeyType::Letter(key),

        // Passthrough
        _ => KeyType::Passthrough,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Telex Tests =====

    #[test]
    fn test_telex_tone_keys() {
        // All Telex tone keys
        assert!(matches!(classify_telex(b's', None), KeyType::Tone(b's')));
        assert!(matches!(classify_telex(b'f', None), KeyType::Tone(b'f')));
        assert!(matches!(classify_telex(b'r', None), KeyType::Tone(b'r')));
        assert!(matches!(classify_telex(b'x', None), KeyType::Tone(b'x')));
        assert!(matches!(classify_telex(b'j', None), KeyType::Tone(b'j')));
    }

    #[test]
    fn test_telex_mark_dd() {
        // dd -> đ (stroke)
        assert!(matches!(
            classify_telex(b'd', Some(b'd')),
            KeyType::Mark(MarkType::Stroke)
        ));
        // Single d is just a letter
        assert!(matches!(classify_telex(b'd', None), KeyType::Letter(b'd')));
        assert!(matches!(
            classify_telex(b'd', Some(b'a')),
            KeyType::Letter(b'd')
        ));
    }

    #[test]
    fn test_telex_mark_w() {
        // w is always mark (horn/breve)
        assert!(matches!(
            classify_telex(b'w', None),
            KeyType::Mark(MarkType::HornOrBreve)
        ));
        assert!(matches!(
            classify_telex(b'w', Some(b'a')),
            KeyType::Mark(MarkType::HornOrBreve)
        ));
    }

    #[test]
    fn test_telex_mark_circumflex() {
        // aa -> â, oo -> ô, ee -> ê
        assert!(matches!(
            classify_telex(b'a', Some(b'a')),
            KeyType::Mark(MarkType::Circumflex)
        ));
        assert!(matches!(
            classify_telex(b'o', Some(b'o')),
            KeyType::Mark(MarkType::Circumflex)
        ));
        assert!(matches!(
            classify_telex(b'e', Some(b'e')),
            KeyType::Mark(MarkType::Circumflex)
        ));

        // Single vowel is letter
        assert!(matches!(classify_telex(b'a', None), KeyType::Letter(b'a')));
        assert!(matches!(classify_telex(b'o', None), KeyType::Letter(b'o')));
        assert!(matches!(classify_telex(b'e', None), KeyType::Letter(b'e')));
    }

    #[test]
    fn test_telex_terminators() {
        // Whitespace
        assert!(matches!(classify_telex(b' ', None), KeyType::Terminator));
        assert!(matches!(classify_telex(b'\n', None), KeyType::Terminator));
        assert!(matches!(classify_telex(b'\t', None), KeyType::Terminator));

        // Punctuation
        assert!(matches!(classify_telex(b'.', None), KeyType::Terminator));
        assert!(matches!(classify_telex(b',', None), KeyType::Terminator));
        assert!(matches!(classify_telex(b'!', None), KeyType::Terminator));
        assert!(matches!(classify_telex(b'?', None), KeyType::Terminator));

        // Brackets
        assert!(matches!(classify_telex(b'(', None), KeyType::Terminator));
        assert!(matches!(classify_telex(b')', None), KeyType::Terminator));
        assert!(matches!(classify_telex(b'[', None), KeyType::Terminator));
        assert!(matches!(classify_telex(b']', None), KeyType::Terminator));
    }

    #[test]
    fn test_telex_letters() {
        // Letters that aren't tone/mark keys
        assert!(matches!(classify_telex(b'a', None), KeyType::Letter(b'a')));
        assert!(matches!(classify_telex(b'b', None), KeyType::Letter(b'b')));
        assert!(matches!(classify_telex(b'z', None), KeyType::Letter(b'z')));
        assert!(matches!(classify_telex(b'A', None), KeyType::Letter(b'A')));
    }

    #[test]
    fn test_telex_passthrough() {
        // Digits and symbols
        assert!(matches!(classify_telex(b'0', None), KeyType::Passthrough));
        assert!(matches!(classify_telex(b'9', None), KeyType::Passthrough));
        assert!(matches!(classify_telex(b'@', None), KeyType::Passthrough));
        assert!(matches!(classify_telex(b'#', None), KeyType::Passthrough));
    }

    // ===== VNI Tests =====

    #[test]
    fn test_vni_tone_numbers() {
        assert!(matches!(classify_vni(b'1', None), KeyType::Tone(1)));
        assert!(matches!(classify_vni(b'2', None), KeyType::Tone(2)));
        assert!(matches!(classify_vni(b'3', None), KeyType::Tone(3)));
        assert!(matches!(classify_vni(b'4', None), KeyType::Tone(4)));
        assert!(matches!(classify_vni(b'5', None), KeyType::Tone(5)));
        assert!(matches!(classify_vni(b'0', None), KeyType::Tone(0)));
    }

    #[test]
    fn test_vni_mark_numbers() {
        assert!(matches!(
            classify_vni(b'6', None),
            KeyType::Mark(MarkType::Circumflex)
        ));
        assert!(matches!(
            classify_vni(b'7', None),
            KeyType::Mark(MarkType::HornOrBreve)
        ));
        assert!(matches!(
            classify_vni(b'8', None),
            KeyType::Mark(MarkType::HornOrBreve)
        ));
        assert!(matches!(
            classify_vni(b'9', None),
            KeyType::Mark(MarkType::Stroke)
        ));
    }

    #[test]
    fn test_vni_terminators() {
        // Whitespace
        assert!(matches!(classify_vni(b' ', None), KeyType::Terminator));
        assert!(matches!(classify_vni(b'\n', None), KeyType::Terminator));
        assert!(matches!(classify_vni(b'\t', None), KeyType::Terminator));

        // Punctuation
        assert!(matches!(classify_vni(b'.', None), KeyType::Terminator));
        assert!(matches!(classify_vni(b',', None), KeyType::Terminator));
        assert!(matches!(classify_vni(b'!', None), KeyType::Terminator));
        assert!(matches!(classify_vni(b'?', None), KeyType::Terminator));

        // Brackets (now supported in VNI too)
        assert!(matches!(classify_vni(b'(', None), KeyType::Terminator));
        assert!(matches!(classify_vni(b')', None), KeyType::Terminator));
        assert!(matches!(classify_vni(b'[', None), KeyType::Terminator));
        assert!(matches!(classify_vni(b']', None), KeyType::Terminator));
        assert!(matches!(classify_vni(b'"', None), KeyType::Terminator));
    }

    #[test]
    fn test_vni_letters() {
        assert!(matches!(classify_vni(b'a', None), KeyType::Letter(b'a')));
        assert!(matches!(classify_vni(b's', None), KeyType::Letter(b's')));
        assert!(matches!(classify_vni(b'Z', None), KeyType::Letter(b'Z')));
    }

    // ===== Method dispatch Tests =====

    #[test]
    fn test_classify_key_telex() {
        assert!(matches!(
            classify_key(b's', None, Method::Telex),
            KeyType::Tone(b's')
        ));
        assert!(matches!(
            classify_key(b'd', Some(b'd'), Method::Telex),
            KeyType::Mark(MarkType::Stroke)
        ));
    }

    #[test]
    fn test_classify_key_vni() {
        assert!(matches!(
            classify_key(b'1', None, Method::Vni),
            KeyType::Tone(1)
        ));
        assert!(matches!(
            classify_key(b'9', None, Method::Vni),
            KeyType::Mark(MarkType::Stroke)
        ));
    }

    #[test]
    fn test_method_default() {
        assert_eq!(Method::default(), Method::Telex);
    }
}
