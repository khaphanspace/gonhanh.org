//! V2 Type definitions for Vietnamese IME
//!
//! Core types used throughout the V2 engine pipeline.

/// Key classification result from Step 0
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyType {
    /// Regular letter (a-z, A-Z)
    Letter(u8),
    /// Tone key (Telex: s,f,r,x,j or VNI: 1-5)
    Tone(u8),
    /// Mark key (w,aa,oo,ee,dd or VNI: 6-9,0)
    Mark(MarkType),
    /// Word boundary (space, punctuation)
    Terminator,
    /// Special key (backspace, delete, esc)
    Special,
    /// Pass through without processing
    Passthrough,
}

/// Mark types for Vietnamese diacritics
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkType {
    /// Stroke: dd -> d
    Stroke,
    /// Circumflex: aa,oo,ee -> a,o,e
    Circumflex,
    /// Horn or Breve: w -> o/u (horn) or a (breve)
    HornOrBreve,
}

/// Vietnamese tones (6 including neutral)
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(u8)]
pub enum Tone {
    #[default]
    None = 0,
    /// sac (acute accent)
    Sac = 1,
    /// huyen (grave accent)
    Huyen = 2,
    /// hoi (hook above)
    Hoi = 3,
    /// nga (tilde)
    Nga = 4,
    /// nang (dot below)
    Nang = 5,
}

impl From<u8> for Tone {
    fn from(v: u8) -> Self {
        match v {
            1 => Tone::Sac,
            2 => Tone::Huyen,
            3 => Tone::Hoi,
            4 => Tone::Nga,
            5 => Tone::Nang,
            _ => Tone::None,
        }
    }
}

/// Action to take after key classification
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    /// Continue processing (add letter to buffer)
    Continue,
    /// Add tone to vowel
    AddTone(Tone),
    /// Revert tone (double-key pattern)
    RevertTone,
    /// Add mark (circumflex, horn, breve)
    AddMark(MarkType),
    /// Add stroke (d -> d)
    AddStroke,
    /// Check if should restore to raw (terminator pressed)
    CheckRestore,
    /// Pass through without modification
    Passthrough,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_from_u8() {
        assert_eq!(Tone::from(0), Tone::None);
        assert_eq!(Tone::from(1), Tone::Sac);
        assert_eq!(Tone::from(2), Tone::Huyen);
        assert_eq!(Tone::from(3), Tone::Hoi);
        assert_eq!(Tone::from(4), Tone::Nga);
        assert_eq!(Tone::from(5), Tone::Nang);
        assert_eq!(Tone::from(99), Tone::None);
    }

    #[test]
    fn test_key_type_equality() {
        assert_eq!(KeyType::Letter(b'a'), KeyType::Letter(b'a'));
        assert_ne!(KeyType::Letter(b'a'), KeyType::Letter(b'b'));
        assert_eq!(KeyType::Tone(1), KeyType::Tone(1));
        assert_eq!(
            KeyType::Mark(MarkType::Stroke),
            KeyType::Mark(MarkType::Stroke)
        );
    }

    #[test]
    fn test_action_equality() {
        assert_eq!(Action::Continue, Action::Continue);
        assert_eq!(Action::AddTone(Tone::Sac), Action::AddTone(Tone::Sac));
        assert_ne!(Action::AddTone(Tone::Sac), Action::AddTone(Tone::Huyen));
    }
}
