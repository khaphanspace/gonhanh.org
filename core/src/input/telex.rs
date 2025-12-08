//! Telex Input Method
//!
//! Key mappings:
//! - Marks: s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng
//! - Tones: a/e/o=circumflex, w=horn/breve
//! - Stroke: d
//! - Remove: z

use super::{Method, ToneType};
use crate::data::keys;

/// Vowels that can receive horn/breve (w key)
const HORN_TARGETS: &[u16] = &[keys::A, keys::O, keys::U];

pub struct Telex;

impl Method for Telex {
    fn mark(&self, key: u16) -> Option<u8> {
        match key {
            keys::S => Some(1), // sắc
            keys::F => Some(2), // huyền
            keys::R => Some(3), // hỏi
            keys::X => Some(4), // ngã
            keys::J => Some(5), // nặng
            _ => None,
        }
    }

    fn tone(&self, key: u16) -> Option<ToneType> {
        match key {
            keys::A | keys::E | keys::O => Some(ToneType::Circumflex),
            keys::W => Some(ToneType::Horn),
            _ => None,
        }
    }

    fn tone_targets(&self, key: u16) -> &'static [u16] {
        match key {
            keys::A => &[keys::A],
            keys::E => &[keys::E],
            keys::O => &[keys::O],
            keys::W => HORN_TARGETS,
            _ => &[],
        }
    }

    fn stroke(&self, key: u16) -> bool {
        key == keys::D
    }

    fn remove(&self, key: u16) -> bool {
        key == keys::Z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marks() {
        let t = Telex;
        assert_eq!(t.mark(keys::S), Some(1));
        assert_eq!(t.mark(keys::F), Some(2));
        assert_eq!(t.mark(keys::A), None);
    }

    #[test]
    fn test_tones() {
        let t = Telex;
        assert_eq!(t.tone(keys::A), Some(ToneType::Circumflex));
        assert_eq!(t.tone(keys::W), Some(ToneType::Horn));
        assert_eq!(t.tone(keys::B), None);
    }

    #[test]
    fn test_tone_targets() {
        let t = Telex;
        assert_eq!(t.tone_targets(keys::A), &[keys::A]);
        assert_eq!(t.tone_targets(keys::W), HORN_TARGETS);
    }
}
