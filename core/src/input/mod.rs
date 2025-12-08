//! Input Methods
//!
//! Defines key mappings for Vietnamese input methods.
//! Engine handles all pattern matching based on buffer scan.

pub mod telex;
pub mod vni;

pub use telex::Telex;
pub use vni::Vni;

use crate::data::chars::tone;

/// Tone modifier type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToneType {
    /// Circumflex: â, ê, ô
    Circumflex,
    /// Horn: ơ, ư (and ă for Telex)
    Horn,
    /// Breve: ă (VNI only)
    Breve,
}

impl ToneType {
    pub fn value(&self) -> u8 {
        match self {
            ToneType::Circumflex => tone::CIRCUMFLEX,
            ToneType::Horn => tone::HORN,
            ToneType::Breve => tone::HORN, // ă uses same internal value
        }
    }
}

/// Input method trait - defines key mappings only
pub trait Method {
    /// Check if key is a mark modifier
    /// Returns: 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
    fn mark(&self, key: u16) -> Option<u8>;

    /// Check if key is a tone modifier
    /// Returns tone type if this key can modify vowels
    fn tone(&self, key: u16) -> Option<ToneType>;

    /// Get valid targets for tone key
    /// Returns list of vowel keys this tone can apply to
    fn tone_targets(&self, key: u16) -> &'static [u16];

    /// Check if key is stroke modifier (d → đ)
    fn stroke(&self, key: u16) -> bool;

    /// Check if key removes diacritics
    fn remove(&self, key: u16) -> bool;
}

/// Get method by id
pub fn get(id: u8) -> Box<dyn Method> {
    match id {
        1 => Box::new(Vni),
        _ => Box::new(Telex),
    }
}
