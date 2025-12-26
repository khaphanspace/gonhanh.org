//! U4: Defer Resolution Matrix
//!
//! Handles deferred actions that depend on subsequent input.
//! Vietnamese has several cases where we can't immediately decide:
//!
//! 1. **Breve on 'a'**: "aw" in open syllable is invalid (no "ă" alone)
//!    - Defer until final consonant confirms: "awn" → "ăn" (valid)
//!    - Or reject if word ends: "aw " → "aw" (no breve)
//!
//! 2. **Horn on 'u' in "uo"**: Only 'o' gets horn initially
//!    - If final follows: "duowc" → "dược" (both u and o get horn)
//!    - If no final: "huow" → "huơ" (only o has horn)
//!
//! 3. **Tone placement**: Depends on syllable structure
//!    - With final: "hoang" + tone → "hoàng" (tone on second vowel)
//!    - Without: "hoa" + tone → depends on modern_tone setting
//!
//! Memory: ~8 bytes (defer types + flags)

/// Defer type constants
pub mod defertype {
    /// No pending defer
    pub const NONE: u8 = 0;
    /// Pending breve on 'a' (waiting for valid final)
    pub const BREVE_A: u8 = 1;
    /// Pending horn on 'u' in "uơ" pattern
    pub const HORN_U: u8 = 2;
    /// Pending tone placement (waiting for syllable completion)
    pub const TONE_PLACE: u8 = 3;
    /// Pending horn on 'o' in "ưo" pattern (from vowel+W)
    /// Example: "đuwoc" → defer horn on 'o', resolve to "đươc" when 'c' added
    pub const HORN_O: u8 = 4;
    /// Pending stroke on 'd' (waiting for mark/circumflex key)
    /// Example: "ded" → defer stroke, "dede" applies stroke+circumflex → "đê"
    /// But "dedicated" → no mark key, stroke discarded → stays "dedicated"
    pub const STROKE_D: u8 = 5;
}

/// Defer context for pending actions
#[derive(Clone, Copy, Default)]
pub struct DeferState {
    /// Type of pending defer
    pub kind: u8,
    /// Position in buffer where defer applies
    pub position: u8,
    /// Additional data (e.g., tone value for TONE_PLACE)
    pub data: u8,
}

impl DeferState {
    /// Create empty defer state
    #[inline]
    pub const fn none() -> Self {
        Self {
            kind: defertype::NONE,
            position: 0,
            data: 0,
        }
    }

    /// Create breve defer on 'a' at position
    #[inline]
    pub const fn breve_a(position: u8) -> Self {
        Self {
            kind: defertype::BREVE_A,
            position,
            data: 0,
        }
    }

    /// Create horn defer on 'u' at position
    #[inline]
    pub const fn horn_u(position: u8) -> Self {
        Self {
            kind: defertype::HORN_U,
            position,
            data: 0,
        }
    }

    /// Create horn defer on 'o' at position (for "ưo" pattern from vowel+W)
    #[inline]
    pub const fn horn_o(position: u8) -> Self {
        Self {
            kind: defertype::HORN_O,
            position,
            data: 0,
        }
    }

    /// Create stroke defer on 'd' at position (for non-adjacent d+vowel+d pattern)
    #[inline]
    pub const fn stroke_d(position: u8) -> Self {
        Self {
            kind: defertype::STROKE_D,
            position,
            data: 0,
        }
    }

    /// Create tone placement defer
    #[inline]
    pub const fn tone_place(position: u8, tone: u8) -> Self {
        Self {
            kind: defertype::TONE_PLACE,
            position,
            data: tone,
        }
    }

    /// Check if there's a pending defer
    #[inline]
    pub const fn is_pending(&self) -> bool {
        self.kind != defertype::NONE
    }

    /// Clear the defer state
    #[inline]
    pub fn clear(&mut self) {
        self.kind = defertype::NONE;
        self.position = 0;
        self.data = 0;
    }
}

/// Valid finals for breve on 'a'
///
/// Breve (ă) can only appear with certain finals:
/// - Valid: ăm, ăn, ăp, ăt, ăc, ăng
/// - Invalid: ănh (use anh instead), standalone ă
///
/// Returns true if the final consonant(s) allow breve on preceding 'a'.
#[inline]
pub fn breve_valid_with_final(final_key: u16, next_key: Option<u16>) -> bool {
    use crate::data::keys;

    match (final_key, next_key) {
        // ng cluster is valid for breve
        (keys::N, Some(keys::G)) => true,
        // nh cluster is NOT valid for breve (use anh, not ănh)
        (keys::N, Some(keys::H)) => false,
        // Single consonant finals: m, n, p, t, c
        (keys::M, _) | (keys::N, None) | (keys::P, _) | (keys::T, _) | (keys::C, _) => true,
        // Invalid finals for breve
        _ => false,
    }
}

/// Valid finals for horn on 'u' in "ươ" pattern
///
/// In "ươ" compounds, both 'ư' and 'ơ' get horn only with certain finals:
/// - "dược" (d + ươ + c) → both have horn
/// - "huơ" (h + uơ, no final) → only 'ơ' has horn
///
/// Returns true if the final confirms both vowels should have horn.
#[inline]
pub fn horn_u_valid_with_final(final_key: u16) -> bool {
    use crate::data::keys;

    // Finals that trigger horn on both vowels in ươ
    matches!(
        final_key,
        keys::C | keys::M | keys::N | keys::P | keys::T | keys::G
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::keys;

    #[test]
    fn test_defer_state_none() {
        let state = DeferState::none();
        assert!(!state.is_pending());
        assert_eq!(state.kind, defertype::NONE);
    }

    #[test]
    fn test_defer_state_breve() {
        let state = DeferState::breve_a(2);
        assert!(state.is_pending());
        assert_eq!(state.kind, defertype::BREVE_A);
        assert_eq!(state.position, 2);
    }

    #[test]
    fn test_defer_state_horn() {
        let state = DeferState::horn_u(1);
        assert!(state.is_pending());
        assert_eq!(state.kind, defertype::HORN_U);
        assert_eq!(state.position, 1);
    }

    #[test]
    fn test_defer_state_clear() {
        let mut state = DeferState::breve_a(2);
        assert!(state.is_pending());
        state.clear();
        assert!(!state.is_pending());
    }

    #[test]
    fn test_breve_valid_finals() {
        // Valid finals for breve
        assert!(breve_valid_with_final(keys::M, None)); // ăm
        assert!(breve_valid_with_final(keys::N, None)); // ăn
        assert!(breve_valid_with_final(keys::P, None)); // ăp
        assert!(breve_valid_with_final(keys::T, None)); // ăt
        assert!(breve_valid_with_final(keys::C, None)); // ăc
        assert!(breve_valid_with_final(keys::N, Some(keys::G))); // ăng
    }

    #[test]
    fn test_breve_invalid_finals() {
        // Invalid: nh final (use anh instead of ănh)
        assert!(!breve_valid_with_final(keys::N, Some(keys::H)));
        // Invalid: standalone
        assert!(!breve_valid_with_final(keys::B, None));
    }

    #[test]
    fn test_horn_u_valid_finals() {
        // Finals that give horn to both vowels in ươ
        assert!(horn_u_valid_with_final(keys::C)); // dược
        assert!(horn_u_valid_with_final(keys::N)); // được
        assert!(horn_u_valid_with_final(keys::M)); // dượm
        assert!(horn_u_valid_with_final(keys::T)); // dượt
    }
}
