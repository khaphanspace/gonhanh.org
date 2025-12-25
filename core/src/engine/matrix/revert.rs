//! U5: Revert Key Matrix
//!
//! Handles double-key revert patterns:
//! - "dd" → "đ" (first d), then "dd" → "đd" (second d reverts)
//! - "aa" → "â" (circumflex), then "aa" → "aa" (reverts)
//! - "ee" → "ê" (circumflex), then "ee" → "ee" (reverts)
//! - "oo" → "ô" (circumflex), then "oo" → "oo" (reverts)
//! - "aw" → "ă" (breve), then "aww" → "aw" (reverts)
//! - "ow" → "ơ" (horn), then "oww" → "ow" (reverts)
//! - "uw" → "ư" (horn), then "uww" → "uw" (reverts)
//! - Tone keys: "vies" → "viế", "viess" → "vies" (reverts)
//!
//! Memory: ~11 bytes (transform type → revert trigger)

use crate::data::keys;

/// Transform types for revert tracking
pub mod xform {
    /// No transform
    pub const NONE: u8 = 0;
    /// Stroke (d → đ)
    pub const STROKE: u8 = 1;
    /// Circumflex (a→â, e→ê, o→ô)
    pub const CIRCUMFLEX: u8 = 2;
    /// Breve (a → ă)
    pub const BREVE: u8 = 3;
    /// Horn (o→ơ, u→ư)
    pub const HORN: u8 = 4;
    /// Tone mark (sắc, huyền, hỏi, ngã, nặng)
    pub const TONE: u8 = 5;
    /// W as vowel (w → ư)
    pub const W_VOWEL: u8 = 6;
}

/// Revert trigger lookup: transform_type → trigger_key
///
/// Returns the key that triggers revert for a given transform type.
/// STROKE(1) → D, CIRCUMFLEX(2) → same vowel, etc.
///
/// For context-dependent reverts (circumflex, horn), the caller
/// must check if the triggering key matches the transformed vowel.
#[inline]
pub fn get_revert_trigger(transform: u8) -> Option<u16> {
    match transform {
        xform::STROKE => Some(keys::D),
        xform::BREVE => Some(keys::W),
        xform::HORN => Some(keys::W),
        xform::W_VOWEL => Some(keys::W),
        // Circumflex and Tone need context - see should_revert()
        _ => None,
    }
}

/// Check if key should trigger revert for the last transform
///
/// Parameters:
/// - `transform`: The last transform type applied
/// - `transform_key`: The key that received the transform (for vowel-dependent cases)
/// - `current_key`: The key just pressed
///
/// Returns true if current_key should revert the transform.
#[inline]
pub fn should_revert(transform: u8, transform_key: u16, current_key: u16) -> bool {
    match transform {
        xform::STROKE => current_key == keys::D,
        xform::BREVE => current_key == keys::W,
        xform::HORN => current_key == keys::W,
        xform::W_VOWEL => current_key == keys::W,
        xform::CIRCUMFLEX => {
            // Circumflex reverts when same vowel is pressed again
            // aa → â, aaa → aa (revert), aaaa → âa, etc.
            current_key == transform_key
        }
        xform::TONE => {
            // Tone reverts when same tone key is pressed again
            // vies → viế, viess → vies
            current_key == transform_key
        }
        _ => false,
    }
}

/// Revert state for tracking last transform
#[derive(Clone, Copy, Default)]
pub struct RevertState {
    /// Type of last transform
    pub transform: u8,
    /// Key that was transformed (for context-dependent reverts)
    pub key: u16,
    /// Position in buffer where transform was applied
    pub position: u8,
    /// Whether revert has already happened (prevent oscillation)
    pub reverted: bool,
}

impl RevertState {
    /// Create empty revert state
    #[inline]
    pub const fn none() -> Self {
        Self {
            transform: xform::NONE,
            key: 0,
            position: 0,
            reverted: false,
        }
    }

    /// Record a new transform
    #[inline]
    pub fn record(&mut self, transform: u8, key: u16, position: u8) {
        self.transform = transform;
        self.key = key;
        self.position = position;
        self.reverted = false;
    }

    /// Mark as reverted (prevents re-triggering)
    #[inline]
    pub fn mark_reverted(&mut self) {
        self.reverted = true;
    }

    /// Clear state
    #[inline]
    pub fn clear(&mut self) {
        self.transform = xform::NONE;
        self.key = 0;
        self.position = 0;
        self.reverted = false;
    }

    /// Check if key should trigger revert
    #[inline]
    pub fn check_revert(&self, current_key: u16) -> bool {
        if self.transform == xform::NONE || self.reverted {
            return false;
        }
        should_revert(self.transform, self.key, current_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stroke_revert() {
        assert!(should_revert(xform::STROKE, keys::D, keys::D));
        assert!(!should_revert(xform::STROKE, keys::D, keys::A));
    }

    #[test]
    fn test_breve_revert() {
        assert!(should_revert(xform::BREVE, keys::A, keys::W));
        assert!(!should_revert(xform::BREVE, keys::A, keys::A));
    }

    #[test]
    fn test_horn_revert() {
        assert!(should_revert(xform::HORN, keys::O, keys::W));
        assert!(should_revert(xform::HORN, keys::U, keys::W));
        assert!(!should_revert(xform::HORN, keys::O, keys::O));
    }

    #[test]
    fn test_circumflex_revert() {
        // Same vowel triggers revert
        assert!(should_revert(xform::CIRCUMFLEX, keys::A, keys::A));
        assert!(should_revert(xform::CIRCUMFLEX, keys::E, keys::E));
        assert!(should_revert(xform::CIRCUMFLEX, keys::O, keys::O));
        // Different key doesn't revert
        assert!(!should_revert(xform::CIRCUMFLEX, keys::A, keys::E));
    }

    #[test]
    fn test_tone_revert() {
        // Same tone key triggers revert
        assert!(should_revert(xform::TONE, keys::S, keys::S)); // sắc
        assert!(should_revert(xform::TONE, keys::F, keys::F)); // huyền
        // Different tone doesn't revert (replaces instead)
        assert!(!should_revert(xform::TONE, keys::S, keys::F));
    }

    #[test]
    fn test_revert_state() {
        let mut state = RevertState::none();
        assert!(!state.check_revert(keys::D));

        // Record stroke transform
        state.record(xform::STROKE, keys::D, 0);
        assert!(state.check_revert(keys::D));
        assert!(!state.check_revert(keys::A));

        // After revert, shouldn't trigger again
        state.mark_reverted();
        assert!(!state.check_revert(keys::D));
    }

    #[test]
    fn test_revert_state_clear() {
        let mut state = RevertState::none();
        state.record(xform::STROKE, keys::D, 0);
        assert!(state.check_revert(keys::D));

        state.clear();
        assert!(!state.check_revert(keys::D));
    }

    #[test]
    fn test_get_revert_trigger() {
        assert_eq!(get_revert_trigger(xform::STROKE), Some(keys::D));
        assert_eq!(get_revert_trigger(xform::BREVE), Some(keys::W));
        assert_eq!(get_revert_trigger(xform::HORN), Some(keys::W));
        assert_eq!(get_revert_trigger(xform::CIRCUMFLEX), None); // Context-dependent
        assert_eq!(get_revert_trigger(xform::TONE), None); // Context-dependent
    }
}
