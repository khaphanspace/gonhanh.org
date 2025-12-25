//! U1: Letter Classification Matrix
//!
//! Bit flags for each letter A-Z indicating valid positions:
//! - Bit 0 (0x01): Vowel (V)
//! - Bit 1 (0x02): Initial consonant (I)
//! - Bit 2 (0x04): Final consonant (F)
//! - Bit 3 (0x08): Special handling (S)
//!
//! Memory: 26 bytes (one per letter)

/// Bit flags for letter classification
pub mod lc {
    /// Vowel: a, e, i, o, u, y
    pub const V: u8 = 0x01;
    /// Initial consonant position
    pub const I: u8 = 0x02;
    /// Final consonant position
    pub const F: u8 = 0x04;
    /// Special handling (w, d)
    pub const S: u8 = 0x08;
}

/// Letter class lookup table (26 bytes)
///
/// Index by (key_code - A_KEY_CODE) for letters A-Z.
/// This table is designed for O(1) lookup of letter properties.
///
/// Vietnamese phonotactics:
/// - Initial consonants: b, c, d, đ, g, h, k, l, m, n, p, q, r, s, t, v, x
/// - Initial clusters: ch, gh, gi, kh, ng, nh, ph, qu, th, tr, ngh
/// - Final consonants: c, m, n, p, t, ch, ng, nh
/// - Vowels: a, e, i, o, u, y (ă, â, ê, ô, ơ, ư are modified forms)
///
/// Table mapping (A=0, B=1, ..., Z=25):
/// ```text
/// A: V       (vowel)
/// B: I       (initial only: b)
/// C: I|F     (initial: c, ch; final: c, ch)
/// D: I|S     (initial: d; special: stroke → đ)
/// E: V       (vowel)
/// F: 0       (not used in Vietnamese)
/// G: I|F     (initial: g, gh, gi; final: ng - as part)
/// H: I|F     (initial: h; final: ch, nh - as part)
/// I: V       (vowel)
/// J: 0       (not used in Vietnamese)
/// K: I       (initial: k, kh)
/// L: I       (initial: l)
/// M: I|F     (initial: m; final: m)
/// N: I|F     (initial: n, ng, nh, ngh; final: n, ng, nh)
/// O: V       (vowel)
/// P: I|F     (initial: p, ph; final: p)
/// Q: I       (initial: qu)
/// R: I       (initial: r, tr)
/// S: I       (initial: s)
/// T: I|F     (initial: t, th, tr; final: t)
/// U: V       (vowel)
/// V: I       (initial: v)
/// W: S       (special: vowel ư or modifier horn/breve)
/// X: I       (initial: x)
/// Y: V       (vowel: y)
/// Z: I       (not standard Vietnamese but handle as initial)
/// ```
pub static LETTER_CLASS: [u8; 26] = [
    lc::V,         // A: vowel
    lc::I,         // B: initial only
    lc::I | lc::F, // C: initial + final
    lc::I | lc::S, // D: initial + special (stroke)
    lc::V,         // E: vowel
    0,             // F: not Vietnamese
    lc::I | lc::F, // G: initial + final (ng part)
    lc::I | lc::F, // H: initial + final (ch/nh part)
    lc::V,         // I: vowel
    0,             // J: not Vietnamese
    lc::I,         // K: initial only
    lc::I,         // L: initial only
    lc::I | lc::F, // M: initial + final
    lc::I | lc::F, // N: initial + final
    lc::V,         // O: vowel
    lc::I | lc::F, // P: initial + final
    lc::I,         // Q: initial only
    lc::I,         // R: initial only
    lc::I,         // S: initial only
    lc::I | lc::F, // T: initial + final
    lc::V,         // U: vowel
    lc::I,         // V: initial only
    lc::S,         // W: special (ư or modifier)
    lc::I,         // X: initial only
    lc::V,         // Y: vowel
    lc::I,         // Z: initial (rare)
];

/// Get letter class flags for a key code
///
/// Returns 0 for non-letter keys.
#[inline]
pub fn get_letter_class(key: u16) -> u8 {
    // Convert key to letter index (A=0, B=1, ...)
    // This assumes keys::A..keys::Z are contiguous, which they're not on macOS.
    // We need a proper mapping table instead.

    use crate::data::keys;

    // Map macOS keycode to letter index
    let idx = match key {
        keys::A => 0,
        keys::B => 1,
        keys::C => 2,
        keys::D => 3,
        keys::E => 4,
        keys::F => 5,
        keys::G => 6,
        keys::H => 7,
        keys::I => 8,
        keys::J => 9,
        keys::K => 10,
        keys::L => 11,
        keys::M => 12,
        keys::N => 13,
        keys::O => 14,
        keys::P => 15,
        keys::Q => 16,
        keys::R => 17,
        keys::S => 18,
        keys::T => 19,
        keys::U => 20,
        keys::V => 21,
        keys::W => 22,
        keys::X => 23,
        keys::Y => 24,
        keys::Z => 25,
        _ => return 0,
    };

    LETTER_CLASS[idx]
}

/// Check if letter is a vowel
#[inline]
pub fn is_vowel_class(class: u8) -> bool {
    class & lc::V != 0
}

/// Check if letter can be initial consonant
#[inline]
pub fn is_initial_class(class: u8) -> bool {
    class & lc::I != 0
}

/// Check if letter can be final consonant
#[inline]
pub fn is_final_class(class: u8) -> bool {
    class & lc::F != 0
}

/// Check if letter has special handling
#[inline]
pub fn is_special_class(class: u8) -> bool {
    class & lc::S != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::keys;

    #[test]
    fn test_vowels() {
        // All vowels should have V flag
        for key in [keys::A, keys::E, keys::I, keys::O, keys::U, keys::Y] {
            let class = get_letter_class(key);
            assert!(is_vowel_class(class), "Key {:?} should be vowel", key);
        }
    }

    #[test]
    fn test_consonants_not_vowels() {
        // Consonants should not have V flag
        for key in [
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
            keys::Z,
        ] {
            let class = get_letter_class(key);
            assert!(!is_vowel_class(class), "Key {:?} should not be vowel", key);
        }
    }

    #[test]
    fn test_final_consonants() {
        // c, m, n, p, t can be finals
        for key in [keys::C, keys::M, keys::N, keys::P, keys::T] {
            let class = get_letter_class(key);
            assert!(is_final_class(class), "Key {:?} should be final", key);
        }
        // g, h can be part of final (ng, ch, nh)
        for key in [keys::G, keys::H] {
            let class = get_letter_class(key);
            assert!(
                is_final_class(class),
                "Key {:?} should be final (part)",
                key
            );
        }
    }

    #[test]
    fn test_initial_only() {
        // These can only be initials, not finals
        for key in [
            keys::B,
            keys::K,
            keys::L,
            keys::Q,
            keys::R,
            keys::S,
            keys::V,
            keys::X,
        ] {
            let class = get_letter_class(key);
            assert!(is_initial_class(class), "Key {:?} should be initial", key);
            assert!(!is_final_class(class), "Key {:?} should not be final", key);
        }
    }

    #[test]
    fn test_special_keys() {
        // W and D have special handling
        let w_class = get_letter_class(keys::W);
        assert!(is_special_class(w_class), "W should be special");

        let d_class = get_letter_class(keys::D);
        assert!(is_special_class(d_class), "D should be special");
    }

    #[test]
    fn test_non_vietnamese() {
        // F and J are not used in Vietnamese
        let f_class = get_letter_class(keys::F);
        assert_eq!(f_class, 0, "F should have no flags");

        let j_class = get_letter_class(keys::J);
        assert_eq!(j_class, 0, "J should have no flags");
    }

    #[test]
    fn test_table_size() {
        assert_eq!(LETTER_CLASS.len(), 26);
    }
}
