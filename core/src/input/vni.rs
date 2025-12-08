//! VNI Input Method
//!
//! Key mappings:
//! - Marks: 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
//! - Tones: 6=circumflex, 7=horn, 8=breve
//! - Stroke: 9
//! - Remove: 0

use super::{Method, ToneType};
use crate::data::keys;

/// Vowels that can receive circumflex (^)
const CIRCUMFLEX_TARGETS: &[u16] = &[keys::A, keys::E, keys::O];

/// Vowels that can receive horn
const HORN_TARGETS: &[u16] = &[keys::O, keys::U];

/// Vowels that can receive breve
const BREVE_TARGETS: &[u16] = &[keys::A];

pub struct Vni;

impl Method for Vni {
    fn mark(&self, key: u16) -> Option<u8> {
        match key {
            keys::N1 => Some(1), // sắc
            keys::N2 => Some(2), // huyền
            keys::N3 => Some(3), // hỏi
            keys::N4 => Some(4), // ngã
            keys::N5 => Some(5), // nặng
            _ => None,
        }
    }

    fn tone(&self, key: u16) -> Option<ToneType> {
        match key {
            keys::N6 => Some(ToneType::Circumflex),
            keys::N7 => Some(ToneType::Horn),
            keys::N8 => Some(ToneType::Breve),
            _ => None,
        }
    }

    fn tone_targets(&self, key: u16) -> &'static [u16] {
        match key {
            keys::N6 => CIRCUMFLEX_TARGETS,
            keys::N7 => HORN_TARGETS,
            keys::N8 => BREVE_TARGETS,
            _ => &[],
        }
    }

    fn stroke(&self, key: u16) -> bool {
        key == keys::N9
    }

    fn remove(&self, key: u16) -> bool {
        key == keys::N0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marks() {
        let v = Vni;
        assert_eq!(v.mark(keys::N1), Some(1));
        assert_eq!(v.mark(keys::N5), Some(5));
        assert_eq!(v.mark(keys::A), None);
    }

    #[test]
    fn test_tones() {
        let v = Vni;
        assert_eq!(v.tone(keys::N6), Some(ToneType::Circumflex));
        assert_eq!(v.tone(keys::N7), Some(ToneType::Horn));
        assert_eq!(v.tone(keys::N8), Some(ToneType::Breve));
    }

    #[test]
    fn test_stroke() {
        let v = Vni;
        assert!(v.stroke(keys::N9));
        assert!(!v.stroke(keys::D));
    }
}
