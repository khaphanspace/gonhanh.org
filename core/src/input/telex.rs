//! Telex input method
//!
//! Marks: s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng
//! Tones: aa=â, ee=ê, oo=ô, aw=ă, ow=ơ, uw=ư, dd=đ
//! Remove: z

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
