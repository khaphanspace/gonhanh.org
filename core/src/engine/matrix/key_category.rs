//! U2: Key Category Matrix
//!
//! Maps macOS keycodes to dispatch categories for the state machine.
//! Categories are designed to minimize dispatch table size while
//! preserving all necessary distinctions.
//!
//! Memory: ~50 bytes (sparse mapping for relevant keys)

use super::cat;
use crate::data::keys;

/// Maximum keycode we handle (macOS keycodes go up to ~127)
const MAX_KEY: usize = 128;

/// Key category lookup table
///
/// Maps keycode → category for dispatch table lookup.
/// Uses cat::OTHER (7) for non-letter/non-relevant keys.
///
/// Categories:
/// - VOWEL (0): a, e, i, o, u, y
/// - INIT_ONLY (1): b, g, h, k, l, q, r, v, x, z
/// - INIT_FINAL (2): c, m, n, p, t
/// - FINAL_PART (3): (reserved for context-dependent finals)
/// - SPECIAL_W (4): w
/// - TONE_KEY (5): s, f, r, x, j (telex tones)
/// - D_KEY (6): d (consonant or stroke trigger)
/// - OTHER (7): everything else
///
/// Note: In Telex mode, some letters serve dual purpose:
/// - s = sắc, f = huyền, r = hỏi, x = ngã, j = nặng
/// - But they're also consonants. Context determines interpretation.
/// - The dispatch table handles this by checking current state.
static KEY_CAT: [u8; MAX_KEY] = {
    let mut table = [cat::OTHER; MAX_KEY];

    // Vowels
    table[keys::A as usize] = cat::VOWEL;
    table[keys::E as usize] = cat::VOWEL;
    table[keys::I as usize] = cat::VOWEL;
    table[keys::O as usize] = cat::VOWEL;
    table[keys::U as usize] = cat::VOWEL;
    table[keys::Y as usize] = cat::VOWEL;

    // Initial-only consonants (cannot be finals)
    table[keys::B as usize] = cat::INIT_ONLY;
    table[keys::G as usize] = cat::INIT_ONLY;  // Note: g in ng is handled separately
    table[keys::H as usize] = cat::INIT_ONLY;  // Note: h in ch/nh is handled separately
    table[keys::K as usize] = cat::INIT_ONLY;
    table[keys::L as usize] = cat::INIT_ONLY;
    table[keys::Q as usize] = cat::INIT_ONLY;
    table[keys::V as usize] = cat::INIT_ONLY;
    table[keys::Z as usize] = cat::INIT_ONLY;

    // Initial+Final consonants
    table[keys::C as usize] = cat::INIT_FINAL;
    table[keys::M as usize] = cat::INIT_FINAL;
    table[keys::N as usize] = cat::INIT_FINAL;
    table[keys::P as usize] = cat::INIT_FINAL;
    table[keys::T as usize] = cat::INIT_FINAL;

    // Tone keys (Telex: s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng)
    // These are context-dependent: tone in VOW/DIA state, consonant otherwise
    table[keys::S as usize] = cat::TONE_KEY;
    table[keys::F as usize] = cat::TONE_KEY;
    table[keys::R as usize] = cat::TONE_KEY;
    table[keys::X as usize] = cat::TONE_KEY;
    table[keys::J as usize] = cat::TONE_KEY;

    // Special keys
    table[keys::W as usize] = cat::SPECIAL_W;
    table[keys::D as usize] = cat::D_KEY;

    table
};

/// Get category for a keycode
///
/// Returns cat::OTHER for unknown/non-letter keys.
#[inline]
pub fn get_key_category(key: u16) -> u8 {
    if (key as usize) < MAX_KEY {
        KEY_CAT[key as usize]
    } else {
        cat::OTHER
    }
}

/// Check if key is a vowel category
#[inline]
pub fn is_vowel_key(key: u16) -> bool {
    get_key_category(key) == cat::VOWEL
}

/// Check if key is a tone key (Telex mode)
#[inline]
pub fn is_tone_key(key: u16) -> bool {
    get_key_category(key) == cat::TONE_KEY
}

/// Check if key can be initial consonant
#[inline]
pub fn is_initial_key(key: u16) -> bool {
    let cat = get_key_category(key);
    cat == cat::INIT_ONLY || cat == cat::INIT_FINAL || cat == cat::D_KEY
}

/// Check if key can be final consonant
#[inline]
pub fn is_final_key(key: u16) -> bool {
    let cat = get_key_category(key);
    cat == cat::INIT_FINAL
}

/// Map tone key to tone value
///
/// Telex mapping:
/// - s → sắc (1)
/// - f → huyền (2)
/// - r → hỏi (3)
/// - x → ngã (4)
/// - j → nặng (5)
/// - z → remove tone (0)
#[inline]
pub fn tone_key_to_value(key: u16) -> Option<u8> {
    match key {
        keys::S => Some(1), // sắc
        keys::F => Some(2), // huyền
        keys::R => Some(3), // hỏi
        keys::X => Some(4), // ngã
        keys::J => Some(5), // nặng
        keys::Z => Some(0), // remove tone
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vowel_categories() {
        for key in [keys::A, keys::E, keys::I, keys::O, keys::U, keys::Y] {
            assert_eq!(get_key_category(key), cat::VOWEL, "Key {:?} should be VOWEL", key);
            assert!(is_vowel_key(key));
        }
    }

    #[test]
    fn test_tone_key_categories() {
        for key in [keys::S, keys::F, keys::R, keys::X, keys::J] {
            assert_eq!(get_key_category(key), cat::TONE_KEY, "Key {:?} should be TONE_KEY", key);
            assert!(is_tone_key(key));
        }
    }

    #[test]
    fn test_special_categories() {
        assert_eq!(get_key_category(keys::W), cat::SPECIAL_W);
        assert_eq!(get_key_category(keys::D), cat::D_KEY);
    }

    #[test]
    fn test_init_only_categories() {
        for key in [keys::B, keys::G, keys::H, keys::K, keys::L, keys::Q, keys::V, keys::Z] {
            assert_eq!(get_key_category(key), cat::INIT_ONLY, "Key {:?} should be INIT_ONLY", key);
        }
    }

    #[test]
    fn test_init_final_categories() {
        for key in [keys::C, keys::M, keys::N, keys::P, keys::T] {
            assert_eq!(get_key_category(key), cat::INIT_FINAL, "Key {:?} should be INIT_FINAL", key);
        }
    }

    #[test]
    fn test_tone_values() {
        assert_eq!(tone_key_to_value(keys::S), Some(1)); // sắc
        assert_eq!(tone_key_to_value(keys::F), Some(2)); // huyền
        assert_eq!(tone_key_to_value(keys::R), Some(3)); // hỏi
        assert_eq!(tone_key_to_value(keys::X), Some(4)); // ngã
        assert_eq!(tone_key_to_value(keys::J), Some(5)); // nặng
        assert_eq!(tone_key_to_value(keys::Z), Some(0)); // remove
        assert_eq!(tone_key_to_value(keys::A), None);    // not a tone key
    }

    #[test]
    fn test_unknown_keys() {
        // Keys outside our range should return OTHER
        assert_eq!(get_key_category(200), cat::OTHER);
        assert_eq!(get_key_category(keys::SPACE), cat::OTHER);
        assert_eq!(get_key_category(keys::RETURN), cat::OTHER);
    }

    #[test]
    fn test_initial_key_check() {
        // These should be valid initials
        assert!(is_initial_key(keys::B));
        assert!(is_initial_key(keys::C));
        assert!(is_initial_key(keys::D));

        // Vowels are not initials
        assert!(!is_initial_key(keys::A));

        // W is special, not initial
        assert!(!is_initial_key(keys::W));
    }

    #[test]
    fn test_final_key_check() {
        // These can be finals
        assert!(is_final_key(keys::C));
        assert!(is_final_key(keys::M));
        assert!(is_final_key(keys::N));
        assert!(is_final_key(keys::P));
        assert!(is_final_key(keys::T));

        // These cannot be finals
        assert!(!is_final_key(keys::B));
        assert!(!is_final_key(keys::K));
        assert!(!is_final_key(keys::L));
    }
}
