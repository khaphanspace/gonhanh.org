//! Telex input method
//!
//! Marks: s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng
//! Tones: aa=â, ee=ê, oo=ô, aw=ă, ow=ơ, uw=ư, dd=đ
//! Remove: z
//!
//! Supports delayed mode: user can type tone keys after the full word
//! Example: "vieta" -> "viêt" + "a" -> finds 'e' in buffer, applies circumflex

use super::Method;
use crate::data::keys;

pub struct Telex;

impl Method for Telex {
    fn is_mark(&self, key: u16) -> Option<u8> {
        match key {
            keys::S => Some(1), // sắc
            keys::F => Some(2), // huyền
            keys::R => Some(3), // hỏi
            keys::X => Some(4), // ngã
            keys::J => Some(5), // nặng
            _ => None,
        }
    }

    fn is_tone(&self, key: u16, prev: Option<u16>) -> Option<u8> {
        let prev = prev?;

        // aa, ee, oo -> hat (^)
        if key == prev && matches!(key, keys::A | keys::E | keys::O) {
            return Some(1);
        }

        // aw -> ă, ow -> ơ, uw -> ư
        if key == keys::W {
            match prev {
                keys::A => return Some(2), // ă
                keys::O => return Some(2), // ơ
                keys::U => return Some(2), // ư
                _ => {}
            }
        }

        None
    }

    /// Telex delayed mode: find matching vowel anywhere in buffer
    /// Example: "truongw" -> 'w' finds 'u' and 'o' for ươ compound
    ///
    /// Priority for 'w':
    /// 1. If 'u' + 'o' adjacent → apply to 'u' first (for ươ compound)
    /// 2. Otherwise find last matching vowel (a, o, u)
    fn is_tone_for(&self, key: u16, vowels: &[u16]) -> Option<(u8, u16)> {
        // aa, ee, oo -> circumflex (^)
        // Find matching vowel in buffer (reverse order - last first)
        if matches!(key, keys::A | keys::E | keys::O) {
            for &v in vowels.iter().rev() {
                if v == key {
                    return Some((1, v));
                }
            }
        }

        // w -> breve/horn
        // Special case: uo pattern → apply to u first (for ươ compound)
        if key == keys::W {
            // Check for uo/ou patterns anywhere in vowels (for compound vowel ươ)
            // Pattern can be followed by other vowels like 'i' in "người"
            let len = vowels.len();
            if len >= 2 {
                // Scan for uo or ou pattern
                for i in 0..len - 1 {
                    let pair = &vowels[i..i + 2];
                    // uo → apply to u (makes ươ when both get horn)
                    if pair == [keys::U, keys::O] {
                        return Some((2, keys::U));
                    }
                    // ou → apply to o first
                    if pair == [keys::O, keys::U] {
                        return Some((2, keys::O));
                    }
                }
            }

            // Find any a, o, or u (reverse order)
            for &v in vowels.iter().rev() {
                match v {
                    keys::A => return Some((2, v)), // ă
                    keys::O => return Some((2, v)), // ơ
                    keys::U => return Some((2, v)), // ư
                    _ => {}
                }
            }
        }

        None
    }

    fn is_d(&self, key: u16, prev: Option<u16>) -> bool {
        // dd -> đ
        key == keys::D && prev == Some(keys::D)
    }

    fn is_remove(&self, key: u16) -> bool {
        key == keys::Z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marks() {
        let t = Telex;
        assert_eq!(t.is_mark(keys::S), Some(1));
        assert_eq!(t.is_mark(keys::F), Some(2));
        assert_eq!(t.is_mark(keys::A), None);
    }

    #[test]
    fn test_tones() {
        let t = Telex;
        assert_eq!(t.is_tone(keys::A, Some(keys::A)), Some(1)); // aa -> â
        assert_eq!(t.is_tone(keys::W, Some(keys::A)), Some(2)); // aw -> ă
        assert_eq!(t.is_tone(keys::W, Some(keys::O)), Some(2)); // ow -> ơ
    }

    #[test]
    fn test_d() {
        let t = Telex;
        assert!(t.is_d(keys::D, Some(keys::D)));
        assert!(!t.is_d(keys::D, Some(keys::A)));
    }
}
