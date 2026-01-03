//! Dispatch matrix for state machine transitions
//!
//! Maps (State, KeyCategory) -> Action
//! 5 states x 8 categories = 40 entries (packed as bytes)

use super::key_category::KeyCategory;

/// Actions that can be taken during keystroke processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    /// Pass key through unchanged
    Pass = 0,
    /// Add as initial consonant
    AddInitial = 1,
    /// Add as vowel
    AddVowel = 2,
    /// Apply tone mark
    ApplyTone = 3,
    /// Apply circumflex (â, ê, ô)
    ApplyCircumflex = 4,
    /// Apply horn (ơ, ư)
    ApplyHorn = 5,
    /// Apply stroke (đ)
    ApplyStroke = 6,
    /// Apply breve (ă)
    ApplyBreve = 7,
    /// Add as final consonant
    AddFinal = 8,
    /// Trigger revert (double key)
    Revert = 9,
    /// Commit word on boundary
    Commit = 10,
    /// Defer decision (context needed)
    Defer = 11,
    /// Invalid action
    Invalid = 255,
}

/// Engine states (5 states)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum State {
    /// Buffer empty, awaiting input
    Empty = 0,
    /// Has initial consonant(s), no vowel yet
    Init = 1,
    /// Has vowel (can receive tone/mark)
    Vow = 2,
    /// Has diacritic (tone or mark applied)
    Dia = 3,
    /// Has final consonant
    Fin = 4,
}

/// Dispatch table: State (5) x Category (8) = 40 entries
/// Each entry is packed as: (Action << 4) | NextState
///
/// Categories: Letter(0), Tone(1), Circumflex(2), Horn(3), Stroke(4), Breve(5), Boundary(6), Invalid(7)
pub const DISPATCH: [[u8; 8]; 5] = [
    // State::Empty
    [
        pack(Action::AddInitial, State::Init), // Letter -> Init
        pack(Action::Pass, State::Empty),      // Tone -> pass (no vowel)
        pack(Action::AddVowel, State::Vow),    // Circumflex vowel -> Vow
        pack(Action::Pass, State::Empty),      // Horn -> pass
        pack(Action::Defer, State::Init),      // Stroke -> defer (might be đ)
        pack(Action::Pass, State::Empty),      // Breve -> pass
        pack(Action::Pass, State::Empty),      // Boundary -> pass
        pack(Action::Pass, State::Empty),      // Invalid -> pass
    ],
    // State::Init
    [
        pack(Action::Defer, State::Init), // Letter -> defer (cluster or vowel?)
        pack(Action::Pass, State::Init),  // Tone -> pass (no vowel yet)
        pack(Action::AddVowel, State::Vow), // Circumflex vowel -> Vow
        pack(Action::Pass, State::Init),  // Horn -> pass
        pack(Action::ApplyStroke, State::Init), // Stroke -> đ
        pack(Action::Pass, State::Init),  // Breve -> pass
        pack(Action::Commit, State::Empty), // Boundary -> commit
        pack(Action::Pass, State::Init),  // Invalid -> pass
    ],
    // State::Vow
    [
        pack(Action::Defer, State::Vow), // Letter -> defer (final or next word?)
        pack(Action::ApplyTone, State::Dia), // Tone -> apply
        pack(Action::Defer, State::Vow), // Circumflex -> defer (double or new?)
        pack(Action::ApplyHorn, State::Dia), // Horn -> ơ/ư
        pack(Action::Pass, State::Vow),  // Stroke -> pass (invalid here)
        pack(Action::ApplyBreve, State::Dia), // Breve -> ă
        pack(Action::Commit, State::Empty), // Boundary -> commit
        pack(Action::Pass, State::Vow),  // Invalid -> pass
    ],
    // State::Dia (has diacritic)
    [
        pack(Action::Defer, State::Dia),    // Letter -> defer
        pack(Action::Defer, State::Dia),    // Tone -> defer (change or revert?)
        pack(Action::Defer, State::Dia),    // Circumflex -> defer
        pack(Action::Defer, State::Dia),    // Horn -> defer
        pack(Action::Pass, State::Dia),     // Stroke -> pass
        pack(Action::Defer, State::Dia),    // Breve -> defer
        pack(Action::Commit, State::Empty), // Boundary -> commit
        pack(Action::Pass, State::Dia),     // Invalid -> pass
    ],
    // State::Fin (has final)
    [
        pack(Action::Defer, State::Fin),     // Letter -> defer (cluster?)
        pack(Action::ApplyTone, State::Fin), // Tone -> apply to vowel
        pack(Action::Pass, State::Fin),      // Circumflex -> pass
        pack(Action::Pass, State::Fin),      // Horn -> pass
        pack(Action::Pass, State::Fin),      // Stroke -> pass
        pack(Action::Pass, State::Fin),      // Breve -> pass
        pack(Action::Commit, State::Empty),  // Boundary -> commit
        pack(Action::Pass, State::Fin),      // Invalid -> pass
    ],
];

/// Pack action and next state into single byte
#[inline]
pub const fn pack(action: Action, state: State) -> u8 {
    ((action as u8) << 4) | (state as u8)
}

/// Unpack action from dispatch entry
#[inline]
pub fn unpack_action(entry: u8) -> Action {
    match entry >> 4 {
        0 => Action::Pass,
        1 => Action::AddInitial,
        2 => Action::AddVowel,
        3 => Action::ApplyTone,
        4 => Action::ApplyCircumflex,
        5 => Action::ApplyHorn,
        6 => Action::ApplyStroke,
        7 => Action::ApplyBreve,
        8 => Action::AddFinal,
        9 => Action::Revert,
        10 => Action::Commit,
        11 => Action::Defer,
        _ => Action::Invalid,
    }
}

/// Unpack next state from dispatch entry
#[inline]
pub fn unpack_state(entry: u8) -> State {
    match entry & 0x0F {
        0 => State::Empty,
        1 => State::Init,
        2 => State::Vow,
        3 => State::Dia,
        4 => State::Fin,
        _ => State::Empty,
    }
}

/// Dispatch a keystroke
#[inline]
pub fn dispatch(state: State, category: KeyCategory) -> (Action, State) {
    let cat_idx = match category {
        KeyCategory::Letter => 0,
        KeyCategory::Tone => 1,
        KeyCategory::Circumflex => 2,
        KeyCategory::Horn => 3,
        KeyCategory::Stroke => 4,
        KeyCategory::Breve => 5,
        KeyCategory::Boundary => 6,
        KeyCategory::Invalid => 7,
    };
    let entry = DISPATCH[state as usize][cat_idx];
    (unpack_action(entry), unpack_state(entry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack() {
        let packed = pack(Action::ApplyTone, State::Dia);
        assert_eq!(unpack_action(packed), Action::ApplyTone);
        assert_eq!(unpack_state(packed), State::Dia);
    }

    #[test]
    fn test_dispatch_vowel_from_empty() {
        let (action, state) = dispatch(State::Empty, KeyCategory::Circumflex);
        assert_eq!(action, Action::AddVowel);
        assert_eq!(state, State::Vow);
    }

    #[test]
    fn test_dispatch_tone_on_vowel() {
        let (action, state) = dispatch(State::Vow, KeyCategory::Tone);
        assert_eq!(action, Action::ApplyTone);
        assert_eq!(state, State::Dia);
    }

    #[test]
    fn test_dispatch_boundary_commits() {
        let (action, _) = dispatch(State::Vow, KeyCategory::Boundary);
        assert_eq!(action, Action::Commit);
    }
}
