//! Engine state machine
//!
//! 5-state machine for syllable tracking:
//! - EMPTY: No input
//! - INIT: Has initial consonant(s)
//! - VOW: Has vowel
//! - DIA: Has diacritic (tone/mark)
//! - FIN: Has final consonant

use crate::v3::constants::dispatch::{Action, State};

/// State transition result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    /// Action to perform
    pub action: Action,
    /// Next state
    pub next_state: State,
}

/// Engine state with additional tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineState {
    /// Current state machine state
    pub state: State,
    /// Has any transformation been applied
    pub had_transform: bool,
    /// Number of vowels in current syllable
    pub vowel_count: u8,
    /// Has tone been applied
    pub has_tone: bool,
    /// Has modifier been applied (circumflex/horn/breve)
    pub has_modifier: bool,
}

impl EngineState {
    /// Create new engine state
    pub fn new() -> Self {
        Self {
            state: State::Empty,
            had_transform: false,
            vowel_count: 0,
            has_tone: false,
            has_modifier: false,
        }
    }

    /// Reset to empty state
    pub fn reset(&mut self) {
        self.state = State::Empty;
        self.had_transform = false;
        self.vowel_count = 0;
        self.has_tone = false;
        self.has_modifier = false;
    }

    /// Mark transformation as occurred
    pub fn mark_transform(&mut self) {
        self.had_transform = true;
    }

    /// Add vowel
    pub fn add_vowel(&mut self) {
        self.vowel_count += 1;
        self.state = State::Vow;
    }

    /// Apply tone
    pub fn apply_tone(&mut self) {
        self.has_tone = true;
        self.had_transform = true;
        self.state = State::Dia;
    }

    /// Apply modifier
    pub fn apply_modifier(&mut self) {
        self.has_modifier = true;
        self.had_transform = true;
        self.state = State::Dia;
    }

    /// Check if in state where tone can be applied
    pub fn can_apply_tone(&self) -> bool {
        matches!(self.state, State::Vow | State::Dia | State::Fin)
            && self.vowel_count > 0
    }

    /// Check if in state where modifier can be applied
    pub fn can_apply_modifier(&self) -> bool {
        matches!(self.state, State::Vow | State::Dia)
            && self.vowel_count > 0
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let s = EngineState::new();
        assert_eq!(s.state, State::Empty);
        assert!(!s.had_transform);
        assert_eq!(s.vowel_count, 0);
    }

    #[test]
    fn test_add_vowel() {
        let mut s = EngineState::new();
        s.add_vowel();
        assert_eq!(s.state, State::Vow);
        assert_eq!(s.vowel_count, 1);
    }

    #[test]
    fn test_apply_tone() {
        let mut s = EngineState::new();
        s.add_vowel();
        s.apply_tone();
        assert!(s.has_tone);
        assert!(s.had_transform);
        assert_eq!(s.state, State::Dia);
    }

    #[test]
    fn test_can_apply_tone() {
        let mut s = EngineState::new();
        assert!(!s.can_apply_tone()); // no vowel

        s.add_vowel();
        assert!(s.can_apply_tone());
    }
}
