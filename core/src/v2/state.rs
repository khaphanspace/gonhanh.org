//! V2 BufferState - u16 bitmask for all state flags
//!
//! Replaces 7 separate boolean fields from V1 with a single u16.
//! More cache-friendly and enables O(1) state comparisons.
//!
//! Bit layout:
//! - Bit 0: had_transform - True if any VN transform applied
//! - Bit 1: has_stroke - True if d was applied
//! - Bit 2: has_tone - True if tone mark present
//! - Bit 3: has_mark - True if vowel mark present (circumflex/horn/breve)
//! - Bit 4: had_revert - True if a revert occurred
//! - Bits 5-6: revert_type (0=none, 1=tone, 2=mark, 3=both)
//! - Bit 7: pending_breve - "aw" waiting for terminator
//! - Bit 8: pending_horn - "ow/uw" waiting
//! - Bits 9-11: vn_state (VnState enum value)
//! - Bits 12-15: reserved

/// BufferState - compact u16 bitmask for tracking buffer state
#[derive(Clone, Copy, Default, Debug)]
pub struct BufferState(u16);

impl BufferState {
    // Bit positions
    const HAD_TRANSFORM: u16 = 1 << 0;
    const HAS_STROKE: u16 = 1 << 1;
    const HAS_TONE: u16 = 1 << 2;
    const HAS_MARK: u16 = 1 << 3;
    const HAD_REVERT: u16 = 1 << 4;
    // Bits 5-6: revert_type
    const REVERT_TYPE_MASK: u16 = 0b11 << 5;
    const REVERT_TYPE_SHIFT: u16 = 5;
    const PENDING_BREVE: u16 = 1 << 7;
    const PENDING_HORN: u16 = 1 << 8;
    // Bits 9-11: vn_state
    const VN_STATE_MASK: u16 = 0b111 << 9;
    const VN_STATE_SHIFT: u16 = 9;

    /// Create new empty state
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Get raw bits (for debugging/testing)
    #[inline]
    pub const fn bits(&self) -> u16 {
        self.0
    }

    // ===== had_transform =====

    /// True if any Vietnamese transform was applied
    #[inline]
    pub const fn had_transform(&self) -> bool {
        self.0 & Self::HAD_TRANSFORM != 0
    }

    /// Set had_transform flag
    #[inline]
    pub fn set_had_transform(&mut self, v: bool) {
        if v {
            self.0 |= Self::HAD_TRANSFORM;
        } else {
            self.0 &= !Self::HAD_TRANSFORM;
        }
    }

    // ===== has_stroke =====

    /// True if d stroke was applied
    #[inline]
    pub const fn has_stroke(&self) -> bool {
        self.0 & Self::HAS_STROKE != 0
    }

    /// Set has_stroke flag
    #[inline]
    pub fn set_has_stroke(&mut self, v: bool) {
        if v {
            self.0 |= Self::HAS_STROKE;
        } else {
            self.0 &= !Self::HAS_STROKE;
        }
    }

    // ===== has_tone =====

    /// True if tone mark is present
    #[inline]
    pub const fn has_tone(&self) -> bool {
        self.0 & Self::HAS_TONE != 0
    }

    /// Set has_tone flag
    #[inline]
    pub fn set_has_tone(&mut self, v: bool) {
        if v {
            self.0 |= Self::HAS_TONE;
        } else {
            self.0 &= !Self::HAS_TONE;
        }
    }

    // ===== has_mark =====

    /// True if vowel mark is present (circumflex/horn/breve)
    #[inline]
    pub const fn has_mark(&self) -> bool {
        self.0 & Self::HAS_MARK != 0
    }

    /// Set has_mark flag
    #[inline]
    pub fn set_has_mark(&mut self, v: bool) {
        if v {
            self.0 |= Self::HAS_MARK;
        } else {
            self.0 &= !Self::HAS_MARK;
        }
    }

    // ===== had_revert =====

    /// True if a revert occurred (double-key pattern)
    #[inline]
    pub const fn had_revert(&self) -> bool {
        self.0 & Self::HAD_REVERT != 0
    }

    /// Set had_revert flag
    #[inline]
    pub fn set_had_revert(&mut self, v: bool) {
        if v {
            self.0 |= Self::HAD_REVERT;
        } else {
            self.0 &= !Self::HAD_REVERT;
        }
    }

    // ===== revert_type =====

    /// Get revert type (0=none, 1=tone, 2=mark, 3=both)
    #[inline]
    pub const fn revert_type(&self) -> u8 {
        ((self.0 & Self::REVERT_TYPE_MASK) >> Self::REVERT_TYPE_SHIFT) as u8
    }

    /// Set revert type
    #[inline]
    pub fn set_revert_type(&mut self, typ: u8) {
        self.0 =
            (self.0 & !Self::REVERT_TYPE_MASK) | (((typ & 0b11) as u16) << Self::REVERT_TYPE_SHIFT);
    }

    // ===== pending_breve =====

    /// True if "aw" is waiting for terminator
    #[inline]
    pub const fn pending_breve(&self) -> bool {
        self.0 & Self::PENDING_BREVE != 0
    }

    /// Set pending_breve flag
    #[inline]
    pub fn set_pending_breve(&mut self, v: bool) {
        if v {
            self.0 |= Self::PENDING_BREVE;
        } else {
            self.0 &= !Self::PENDING_BREVE;
        }
    }

    // ===== pending_horn =====

    /// True if "ow/uw" is waiting
    #[inline]
    pub const fn pending_horn(&self) -> bool {
        self.0 & Self::PENDING_HORN != 0
    }

    /// Set pending_horn flag
    #[inline]
    pub fn set_pending_horn(&mut self, v: bool) {
        if v {
            self.0 |= Self::PENDING_HORN;
        } else {
            self.0 &= !Self::PENDING_HORN;
        }
    }

    // ===== vn_state =====

    /// Get Vietnamese validation state
    #[inline]
    pub const fn vn_state(&self) -> VnState {
        let bits = (self.0 & Self::VN_STATE_MASK) >> Self::VN_STATE_SHIFT;
        VnState::from_bits(bits)
    }

    /// Set Vietnamese validation state
    #[inline]
    pub fn set_vn_state(&mut self, state: VnState) {
        self.0 = (self.0 & !Self::VN_STATE_MASK) | ((state as u16) << Self::VN_STATE_SHIFT);
    }

    // ===== utility =====

    /// Clear all state (reset to initial)
    #[inline]
    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

/// Vietnamese validation state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u16)]
pub enum VnState {
    /// Not yet determined
    #[default]
    Unknown = 0,
    /// Valid complete Vietnamese syllable
    Complete = 1,
    /// Potentially valid, needs more input
    Incomplete = 2,
    /// Definitely not valid Vietnamese
    Impossible = 3,
}

impl VnState {
    /// Convert from raw bits (const fn for use in BufferState)
    #[inline]
    pub const fn from_bits(v: u16) -> Self {
        match v {
            1 => VnState::Complete,
            2 => VnState::Incomplete,
            3 => VnState::Impossible,
            _ => VnState::Unknown,
        }
    }
}

impl From<u16> for VnState {
    fn from(v: u16) -> Self {
        Self::from_bits(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state_is_empty() {
        let state = BufferState::new();
        assert_eq!(state.bits(), 0);
        assert!(!state.had_transform());
        assert!(!state.has_stroke());
        assert!(!state.has_tone());
        assert!(!state.has_mark());
        assert!(!state.had_revert());
        assert!(!state.pending_breve());
        assert!(!state.pending_horn());
        assert_eq!(state.vn_state(), VnState::Unknown);
    }

    #[test]
    fn test_had_transform() {
        let mut state = BufferState::new();
        assert!(!state.had_transform());

        state.set_had_transform(true);
        assert!(state.had_transform());
        assert_eq!(state.bits() & BufferState::HAD_TRANSFORM, 1);

        state.set_had_transform(false);
        assert!(!state.had_transform());
    }

    #[test]
    fn test_has_stroke() {
        let mut state = BufferState::new();
        state.set_has_stroke(true);
        assert!(state.has_stroke());

        state.set_has_stroke(false);
        assert!(!state.has_stroke());
    }

    #[test]
    fn test_has_tone() {
        let mut state = BufferState::new();
        state.set_has_tone(true);
        assert!(state.has_tone());

        state.set_has_tone(false);
        assert!(!state.has_tone());
    }

    #[test]
    fn test_has_mark() {
        let mut state = BufferState::new();
        state.set_has_mark(true);
        assert!(state.has_mark());

        state.set_has_mark(false);
        assert!(!state.has_mark());
    }

    #[test]
    fn test_had_revert() {
        let mut state = BufferState::new();
        state.set_had_revert(true);
        assert!(state.had_revert());

        state.set_had_revert(false);
        assert!(!state.had_revert());
    }

    #[test]
    fn test_revert_type() {
        let mut state = BufferState::new();
        assert_eq!(state.revert_type(), 0);

        state.set_revert_type(1);
        assert_eq!(state.revert_type(), 1);

        state.set_revert_type(2);
        assert_eq!(state.revert_type(), 2);

        state.set_revert_type(3);
        assert_eq!(state.revert_type(), 3);

        // Should mask to 2 bits
        state.set_revert_type(0xFF);
        assert_eq!(state.revert_type(), 3);
    }

    #[test]
    fn test_pending_breve() {
        let mut state = BufferState::new();
        state.set_pending_breve(true);
        assert!(state.pending_breve());

        state.set_pending_breve(false);
        assert!(!state.pending_breve());
    }

    #[test]
    fn test_pending_horn() {
        let mut state = BufferState::new();
        state.set_pending_horn(true);
        assert!(state.pending_horn());

        state.set_pending_horn(false);
        assert!(!state.pending_horn());
    }

    #[test]
    fn test_vn_state() {
        let mut state = BufferState::new();
        assert_eq!(state.vn_state(), VnState::Unknown);

        state.set_vn_state(VnState::Complete);
        assert_eq!(state.vn_state(), VnState::Complete);

        state.set_vn_state(VnState::Incomplete);
        assert_eq!(state.vn_state(), VnState::Incomplete);

        state.set_vn_state(VnState::Impossible);
        assert_eq!(state.vn_state(), VnState::Impossible);

        state.set_vn_state(VnState::Unknown);
        assert_eq!(state.vn_state(), VnState::Unknown);
    }

    #[test]
    fn test_multiple_flags() {
        let mut state = BufferState::new();

        state.set_had_transform(true);
        state.set_has_tone(true);
        state.set_vn_state(VnState::Complete);

        assert!(state.had_transform());
        assert!(state.has_tone());
        assert_eq!(state.vn_state(), VnState::Complete);

        // Other flags should be unaffected
        assert!(!state.has_stroke());
        assert!(!state.has_mark());
        assert!(!state.pending_breve());
    }

    #[test]
    fn test_clear() {
        let mut state = BufferState::new();

        state.set_had_transform(true);
        state.set_has_stroke(true);
        state.set_has_tone(true);
        state.set_has_mark(true);
        state.set_pending_breve(true);
        state.set_vn_state(VnState::Complete);

        state.clear();

        assert_eq!(state.bits(), 0);
        assert!(!state.had_transform());
        assert!(!state.has_stroke());
        assert!(!state.has_tone());
        assert!(!state.has_mark());
        assert!(!state.pending_breve());
        assert_eq!(state.vn_state(), VnState::Unknown);
    }

    #[test]
    fn test_vn_state_from_u16() {
        assert_eq!(VnState::from(0), VnState::Unknown);
        assert_eq!(VnState::from(1), VnState::Complete);
        assert_eq!(VnState::from(2), VnState::Incomplete);
        assert_eq!(VnState::from(3), VnState::Impossible);
        assert_eq!(VnState::from(99), VnState::Unknown);
    }

    #[test]
    fn test_bit_isolation() {
        // Verify setting one flag doesn't affect others
        let mut state = BufferState::new();

        state.set_vn_state(VnState::Complete);
        assert!(!state.had_transform());
        assert!(!state.has_tone());

        state.set_had_transform(true);
        assert_eq!(state.vn_state(), VnState::Complete);

        state.set_has_tone(true);
        assert!(state.had_transform());
        assert_eq!(state.vn_state(), VnState::Complete);
    }
}
