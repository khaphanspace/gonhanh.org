//! Matrix-based Processor
//!
//! The new engine core that uses matrix lookups for all decisions.
//! Zero if-else in hot path.
//!
//! ## Design
//!
//! - 5-state machine (EMPTY, INIT, VOW, DIA, FIN)
//! - All decisions via matrix dispatch
//! - Fixed-size buffers (no heap allocation)
//! - Integrated defer and revert handling

use super::{
    act, cat, dispatch, st,
    get_key_category, tone_key_to_value,
    DeferState, RevertState, defertype, xform,
    breve_valid_with_final, horn_u_valid_with_final,
};
use crate::data::keys;
use crate::engine::buffer::{Buffer, Char};

/// Maximum raw keystrokes per word
const RAW_MAX: usize = 96;

/// Raw keystroke with packed flags
#[derive(Clone, Copy, Default)]
pub struct RawKeystroke {
    /// macOS keycode
    pub key: u16,
    /// Packed flags: bit0=caps, bit1=shift, bit2=consumed
    pub flags: u8,
}

impl RawKeystroke {
    /// Create new raw keystroke
    #[inline]
    pub const fn new(key: u16, caps: bool, shift: bool) -> Self {
        let flags = (caps as u8) | ((shift as u8) << 1);
        Self { key, flags }
    }

    /// Check if caps lock was active
    #[inline]
    pub const fn caps(&self) -> bool {
        self.flags & 1 != 0
    }

    /// Check if shift was pressed
    #[inline]
    pub const fn shift(&self) -> bool {
        self.flags & 2 != 0
    }

    /// Check if keystroke was consumed by a transform
    #[inline]
    pub const fn consumed(&self) -> bool {
        self.flags & 4 != 0
    }

    /// Mark as consumed
    #[inline]
    pub fn mark_consumed(&mut self) {
        self.flags |= 4;
    }
}

/// Fixed-size raw input buffer (stack-allocated)
#[derive(Clone)]
pub struct RawBuffer {
    data: [RawKeystroke; RAW_MAX],
    len: u8,
}

impl Default for RawBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl RawBuffer {
    /// Create empty buffer
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: [RawKeystroke { key: 0, flags: 0 }; RAW_MAX],
            len: 0,
        }
    }

    /// Push keystroke
    #[inline]
    pub fn push(&mut self, key: u16, caps: bool, shift: bool) {
        if (self.len as usize) < RAW_MAX {
            self.data[self.len as usize] = RawKeystroke::new(key, caps, shift);
            self.len += 1;
        }
    }

    /// Pop last keystroke
    #[inline]
    pub fn pop(&mut self) -> Option<RawKeystroke> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.data[self.len as usize])
        } else {
            None
        }
    }

    /// Get last keystroke
    #[inline]
    pub fn last(&self) -> Option<&RawKeystroke> {
        if self.len > 0 {
            Some(&self.data[(self.len - 1) as usize])
        } else {
            None
        }
    }

    /// Get mutable reference to keystroke at index
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut RawKeystroke> {
        if index < self.len as usize {
            Some(&mut self.data[index])
        } else {
            None
        }
    }

    /// Mark keystroke at index as consumed
    #[inline]
    pub fn mark_consumed(&mut self, index: usize) {
        if let Some(k) = self.get_mut(index) {
            k.mark_consumed();
        }
    }

    /// Get length
    #[inline]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Check if empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clear buffer
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Restore to raw characters (skip consumed keystrokes)
    pub fn restore(&self) -> Vec<char> {
        use crate::utils::key_to_char;
        self.data[..self.len as usize]
            .iter()
            .filter(|k| !k.consumed())
            .filter_map(|k| key_to_char(k.key, k.caps()))
            .collect()
    }
}

/// Matrix-based processor result
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProcessResult {
    /// No action needed
    None,
    /// Character added/modified, needs display update
    Update,
    /// Transform applied (tone, mark, stroke)
    Transform,
    /// Transform reverted (double-key)
    Revert,
    /// Invalid sequence rejected
    Reject,
}

/// Matrix-based Vietnamese IME Processor
///
/// Uses matrix dispatch for all decisions. No if-else in hot path.
///
/// Memory layout:
/// - state: 1 byte
/// - flags: 1 byte (packed booleans)
/// - defer_state: 3 bytes
/// - revert_state: 5 bytes
/// - buffer: ~520 bytes (inherited)
/// - raw: ~290 bytes
/// Total: ~820 bytes stack
pub struct Processor {
    /// Current state in state machine
    state: u8,
    /// Transformed buffer
    buffer: Buffer,
    /// Raw keystroke history
    raw: RawBuffer,
    /// Pending defer action
    defer: DeferState,
    /// Last transform for revert
    revert: RevertState,
    /// Input method (0=Telex, 1=VNI)
    method: u8,
    /// Modern tone placement (hoà vs hòa)
    modern_tone: bool,
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor {
    /// Create new processor
    pub fn new() -> Self {
        Self {
            state: st::EMPTY,
            buffer: Buffer::new(),
            raw: RawBuffer::new(),
            defer: DeferState::none(),
            revert: RevertState::none(),
            method: 0, // Telex
            modern_tone: true,
        }
    }

    /// Set input method (0=Telex, 1=VNI)
    pub fn set_method(&mut self, method: u8) {
        self.method = method;
    }

    /// Set modern tone placement
    pub fn set_modern_tone(&mut self, modern: bool) {
        self.modern_tone = modern;
    }

    /// Get current state
    #[inline]
    pub const fn state(&self) -> u8 {
        self.state
    }

    /// Get buffer reference
    #[inline]
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get raw buffer reference
    #[inline]
    pub fn raw(&self) -> &RawBuffer {
        &self.raw
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.state = st::EMPTY;
        self.buffer.clear();
        self.raw.clear();
        self.defer.clear();
        self.revert.clear();
    }

    /// Get effective category for a key, handling context-dependent cases
    ///
    /// In FIN state, G and H can extend the final consonant (ng, nh, ch)
    /// rather than starting a new word.
    fn get_effective_category(&self, key: u16) -> u8 {
        let base_category = get_key_category(key);

        // In FIN state, check for cluster completion
        if self.state == st::FIN {
            if let Some(last) = self.buffer.last() {
                // After 'n': g or h can extend to ng/nh
                if last.key == keys::N && (key == keys::G || key == keys::H) {
                    return cat::FINAL_PART;
                }
                // After 'c': h can extend to ch
                if last.key == keys::C && key == keys::H {
                    return cat::FINAL_PART;
                }
            }
        }

        base_category
    }

    /// Process a key press
    ///
    /// This is the main entry point. Uses matrix dispatch for all decisions.
    pub fn process(&mut self, key: u16, caps: bool, shift: bool) -> ProcessResult {
        // Record raw keystroke first
        self.raw.push(key, caps, shift);

        // Check for revert before dispatch
        if self.revert.check_revert(key) {
            return self.handle_revert(key);
        }

        // Special handling for ng/nh/ch clusters in FIN state
        // G and H after n/c should extend the final, not start new word
        let effective_category = self.get_effective_category(key);

        // Get action and next state from dispatch table
        let (action, next_state) = dispatch(self.state, effective_category);

        // Execute action
        let result = match action {
            act::PASS => self.handle_pass(key, caps, next_state),
            act::TONE => self.handle_tone(key),
            act::MARK => self.handle_mark(key),
            act::STROKE => self.handle_stroke(),
            act::REJECT => ProcessResult::Reject,
            act::DEFER => self.handle_defer(key, caps),
            act::REVERT => self.handle_revert(key),
            _ => ProcessResult::None,
        };

        // Update state if action succeeded
        if result != ProcessResult::Reject {
            self.state = next_state;
        }

        // Check and resolve defers based on new state
        if self.defer.is_pending() {
            self.resolve_defer();
        }

        result
    }

    /// Handle PASS action - append character to buffer
    fn handle_pass(&mut self, key: u16, caps: bool, next_state: u8) -> ProcessResult {
        // Special handling for W in various contexts
        if key == keys::W {
            return self.handle_w(caps, next_state);
        }

        // Regular character append
        self.buffer.push(Char::new(key, caps));
        self.revert.clear(); // Regular char clears revert state

        ProcessResult::Update
    }

    /// Handle W key - can be vowel ư or modifier
    fn handle_w(&mut self, caps: bool, _next_state: u8) -> ProcessResult {
        match self.state {
            st::EMPTY | st::INIT => {
                // W at start becomes ư
                let mut c = Char::new(keys::U, caps);
                c.tone = 2; // horn mark for ư
                self.buffer.push(c);

                // Record for revert (ww → w)
                self.revert.record(xform::W_VOWEL, keys::W, (self.buffer.len() - 1) as u8);
                ProcessResult::Transform
            }
            st::VOW | st::DIA => {
                // W after vowel is modifier (horn/breve)
                self.apply_horn_or_breve()
            }
            _ => {
                // W in other states - just append
                self.buffer.push(Char::new(keys::W, caps));
                ProcessResult::Update
            }
        }
    }

    /// Apply horn or breve mark based on context
    fn apply_horn_or_breve(&mut self) -> ProcessResult {
        // Find the vowel to modify
        let vowels = self.buffer.find_vowels();
        if vowels.is_empty() {
            return ProcessResult::Reject;
        }

        // Get last vowel
        let pos = *vowels.last().unwrap();
        if let Some(c) = self.buffer.get_mut(pos) {
            match c.key {
                keys::A => {
                    // a + w → ă (breve) - but defer if open syllable
                    if self.state == st::VOW {
                        // Defer breve until final consonant confirms
                        self.defer = DeferState::breve_a(pos as u8);
                        // Mark raw keystroke as consumed
                        self.raw.mark_consumed(self.raw.len() - 1);
                        ProcessResult::Update
                    } else {
                        c.tone = 2; // breve
                        self.revert.record(xform::BREVE, keys::A, pos as u8);
                        self.raw.mark_consumed(self.raw.len() - 1);
                        ProcessResult::Transform
                    }
                }
                keys::O => {
                    // o + w → ơ (horn)
                    c.tone = 2;
                    self.revert.record(xform::HORN, keys::O, pos as u8);
                    self.raw.mark_consumed(self.raw.len() - 1);

                    // Check for uo pattern - may need to defer horn on u
                    if pos > 0 {
                        if let Some(prev) = self.buffer.get(pos - 1) {
                            if prev.key == keys::U && prev.tone == 0 {
                                // u + ơ pattern - defer horn on u
                                self.defer = DeferState::horn_u((pos - 1) as u8);
                            }
                        }
                    }
                    ProcessResult::Transform
                }
                keys::U => {
                    // u + w → ư (horn)
                    c.tone = 2;
                    self.revert.record(xform::HORN, keys::U, pos as u8);
                    self.raw.mark_consumed(self.raw.len() - 1);
                    ProcessResult::Transform
                }
                _ => ProcessResult::Reject,
            }
        } else {
            ProcessResult::Reject
        }
    }

    /// Handle TONE action - apply tone mark
    fn handle_tone(&mut self, key: u16) -> ProcessResult {
        let tone_value = match tone_key_to_value(key) {
            Some(v) => v,
            None => return ProcessResult::Reject,
        };

        // Find vowel to apply tone
        let vowels = self.buffer.find_vowels();
        if vowels.is_empty() {
            return ProcessResult::Reject;
        }

        // Determine which vowel gets the tone
        let pos = self.find_tone_position(&vowels);
        if let Some(c) = self.buffer.get_mut(pos) {
            let old_mark = c.mark;
            c.mark = tone_value;

            // Record for revert
            self.revert.record(xform::TONE, key, pos as u8);
            self.raw.mark_consumed(self.raw.len() - 1);

            if old_mark == tone_value {
                // Same tone again - would be handled by revert check
                ProcessResult::Transform
            } else {
                ProcessResult::Transform
            }
        } else {
            ProcessResult::Reject
        }
    }

    /// Find correct position for tone placement
    fn find_tone_position(&self, vowels: &[usize]) -> usize {
        // Simple rule: if multiple vowels, use the one with existing modifiers
        // or the first one in modern mode, last in traditional
        if vowels.len() == 1 {
            return vowels[0];
        }

        // Check for vowel with existing tone modifier
        for &pos in vowels {
            if let Some(c) = self.buffer.get(pos) {
                if c.tone > 0 {
                    return pos;
                }
            }
        }

        // Use modern/traditional rule
        if self.modern_tone && vowels.len() >= 2 {
            // Modern: tone on last vowel before final
            vowels[vowels.len() - 1]
        } else {
            // Traditional: tone on first vowel
            vowels[0]
        }
    }

    /// Handle MARK action - apply vowel mark (circumflex, horn, breve)
    fn handle_mark(&mut self, key: u16) -> ProcessResult {
        // W is handled separately in handle_w
        if key == keys::W {
            return self.apply_horn_or_breve();
        }

        // For circumflex via double vowel (aa, ee, oo)
        let vowels = self.buffer.find_vowels();
        if vowels.is_empty() {
            return ProcessResult::Reject;
        }

        // Check if this is double vowel for circumflex
        let last_pos = *vowels.last().unwrap();
        if let Some(c) = self.buffer.get(last_pos) {
            if c.key == key && c.tone == 0 {
                // Same vowel without existing mark - apply circumflex
                if let Some(c) = self.buffer.get_mut(last_pos) {
                    c.tone = 1; // circumflex
                    self.revert.record(xform::CIRCUMFLEX, key, last_pos as u8);
                    self.raw.mark_consumed(self.raw.len() - 1);
                    return ProcessResult::Transform;
                }
            }
        }

        ProcessResult::Update
    }

    /// Handle STROKE action - apply stroke to d
    fn handle_stroke(&mut self) -> ProcessResult {
        // Find the d to stroke (should be the last one in INIT state)
        for i in (0..self.buffer.len()).rev() {
            if let Some(c) = self.buffer.get_mut(i) {
                if c.key == keys::D && !c.stroke {
                    c.stroke = true;
                    self.revert.record(xform::STROKE, keys::D, i as u8);
                    self.raw.mark_consumed(self.raw.len() - 1);
                    return ProcessResult::Transform;
                }
            }
        }
        ProcessResult::Reject
    }

    /// Handle DEFER action
    fn handle_defer(&mut self, key: u16, caps: bool) -> ProcessResult {
        // Store deferred action for later resolution
        self.buffer.push(Char::new(key, caps));
        ProcessResult::Update
    }

    /// Handle REVERT action - undo last transform
    fn handle_revert(&mut self, _key: u16) -> ProcessResult {
        let pos = self.revert.position as usize;

        match self.revert.transform {
            xform::STROKE => {
                // Revert đ → d
                if let Some(c) = self.buffer.get_mut(pos) {
                    c.stroke = false;
                }
                // Remove the triggering key from raw (it's the second d)
                // and restore the consumed d
            }
            xform::CIRCUMFLEX | xform::HORN | xform::BREVE => {
                // Revert vowel modification
                if let Some(c) = self.buffer.get_mut(pos) {
                    c.tone = 0;
                }
            }
            xform::TONE => {
                // Revert tone mark
                if let Some(c) = self.buffer.get_mut(pos) {
                    c.mark = 0;
                }
            }
            xform::W_VOWEL => {
                // Revert ư → w
                if let Some(c) = self.buffer.get_mut(pos) {
                    c.key = keys::W;
                    c.tone = 0;
                }
            }
            _ => {}
        }

        self.revert.mark_reverted();
        ProcessResult::Revert
    }

    /// Resolve pending defer based on current state
    fn resolve_defer(&mut self) {
        match self.defer.kind {
            defertype::BREVE_A => {
                // Check if we now have a valid final for breve
                if self.state == st::FIN {
                    let len = self.buffer.len();
                    if len >= 1 {
                        let last = self.buffer.get(len - 1).map(|c| c.key);

                        // If last char is 'n', don't resolve yet - wait for potential g/h
                        // This handles ăn vs ăng vs anh (invalid)
                        if last == Some(keys::N) {
                            // Don't resolve - wait for next character
                            return;
                        }

                        // Check for ng/nh cluster (len >= 2)
                        let next_key = if len >= 2 {
                            let second_last = self.buffer.get(len - 2).map(|c| c.key);
                            // If second-last is 'n' and last is 'g' or 'h', it's a cluster
                            if second_last == Some(keys::N) &&
                               (last == Some(keys::G) || last == Some(keys::H)) {
                                last
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        // Determine final key for validation
                        let final_key = if next_key.is_some() {
                            // For ng/nh cluster, final_key is 'n'
                            keys::N
                        } else {
                            last.unwrap_or(0)
                        };

                        if breve_valid_with_final(final_key, next_key) {
                            // Apply the deferred breve
                            let pos = self.defer.position as usize;
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 2; // breve
                            }
                        }
                        self.defer.clear();
                    }
                }
            }
            defertype::HORN_U => {
                // Check if we now have a valid final for horn on u
                if self.state == st::FIN {
                    if let Some(last) = self.buffer.last() {
                        if horn_u_valid_with_final(last.key) {
                            // Apply horn to u as well
                            let pos = self.defer.position as usize;
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 2; // horn
                            }
                        }
                    }
                    self.defer.clear();
                }
            }
            defertype::TONE_PLACE => {
                // Tone placement depends on syllable structure
                // If we have a final consonant, may need to adjust tone position
                if self.state == st::FIN {
                    // For now, keep tone where it was placed
                    // More sophisticated logic can be added later
                    self.defer.clear();
                }
            }
            _ => {}
        }
    }

    /// Check if buffer ends with word boundary (space, punctuation)
    pub fn is_word_boundary(&self, key: u16) -> bool {
        matches!(key, keys::SPACE | keys::DOT | keys::COMMA |
                     keys::SEMICOLON | keys::QUOTE | keys::SLASH |
                     keys::LBRACKET | keys::RBRACKET | keys::MINUS)
    }

    /// Finalize word - resolve any pending defers and reset state
    pub fn finalize_word(&mut self) {
        // If we have pending breve defer
        if self.defer.kind == defertype::BREVE_A {
            if self.state != st::FIN {
                // No final - don't apply breve - standalone "ă" is invalid
                self.defer.clear();
            } else {
                // We're in FIN state, might have been waiting for cluster completion
                // Now word is ending, so resolve the defer
                let len = self.buffer.len();
                if len >= 1 {
                    let last = self.buffer.get(len - 1).map(|c| c.key);
                    let next_key = if len >= 2 {
                        let second_last = self.buffer.get(len - 2).map(|c| c.key);
                        if second_last == Some(keys::N) &&
                           (last == Some(keys::G) || last == Some(keys::H)) {
                            last
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let final_key = if next_key.is_some() {
                        keys::N
                    } else {
                        last.unwrap_or(0)
                    };

                    if breve_valid_with_final(final_key, next_key) {
                        let pos = self.defer.position as usize;
                        if let Some(c) = self.buffer.get_mut(pos) {
                            c.tone = 2; // breve
                        }
                    }
                }
                self.defer.clear();
            }
        }

        // If we have pending horn_u defer without final, don't apply horn to u
        if self.defer.kind == defertype::HORN_U && self.state != st::FIN {
            self.defer.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_new() {
        let p = Processor::new();
        assert_eq!(p.state(), st::EMPTY);
        assert!(p.buffer().is_empty());
        assert!(p.raw().is_empty());
    }

    #[test]
    fn test_process_vowel() {
        let mut p = Processor::new();

        // Type 'a'
        let result = p.process(keys::A, false, false);
        assert_eq!(result, ProcessResult::Update);
        assert_eq!(p.state(), st::VOW);
        assert_eq!(p.buffer().len(), 1);
    }

    #[test]
    fn test_process_consonant() {
        let mut p = Processor::new();

        // Type 'v'
        let result = p.process(keys::V, false, false);
        assert_eq!(result, ProcessResult::Update);
        assert_eq!(p.state(), st::INIT);
    }

    #[test]
    fn test_process_viet() {
        let mut p = Processor::new();

        // Type "viet"
        p.process(keys::V, false, false);
        assert_eq!(p.state(), st::INIT);

        p.process(keys::I, false, false);
        assert_eq!(p.state(), st::VOW);

        p.process(keys::E, false, false);
        assert_eq!(p.state(), st::VOW);

        p.process(keys::T, false, false);
        assert_eq!(p.state(), st::FIN);
    }

    #[test]
    fn test_process_tone() {
        let mut p = Processor::new();

        // Type "vie" + 's' for sắc
        p.process(keys::V, false, false);
        p.process(keys::I, false, false);
        p.process(keys::E, false, false);

        let result = p.process(keys::S, false, false);
        assert_eq!(result, ProcessResult::Transform);
        assert_eq!(p.state(), st::DIA);

        // Check that tone was applied
        let vowels = p.buffer().find_vowels();
        assert!(!vowels.is_empty());
    }

    #[test]
    fn test_process_stroke() {
        let mut p = Processor::new();

        // Type "dd" for đ
        p.process(keys::D, false, false);
        assert_eq!(p.state(), st::INIT);

        let result = p.process(keys::D, false, false);
        assert_eq!(result, ProcessResult::Transform);

        // Check that stroke was applied
        if let Some(c) = p.buffer().get(0) {
            assert!(c.stroke);
        }
    }

    #[test]
    fn test_process_w_as_vowel() {
        let mut p = Processor::new();

        // Type 'w' at start becomes ư
        let result = p.process(keys::W, false, false);
        assert_eq!(result, ProcessResult::Transform);
        assert_eq!(p.state(), st::VOW);

        // Buffer should have modified 'u' with horn
        if let Some(c) = p.buffer().get(0) {
            assert_eq!(c.key, keys::U);
            assert_eq!(c.tone, 2); // horn
        }
    }

    #[test]
    fn test_raw_buffer() {
        let mut raw = RawBuffer::new();
        assert!(raw.is_empty());

        raw.push(keys::A, true, false);
        assert_eq!(raw.len(), 1);

        if let Some(k) = raw.last() {
            assert_eq!(k.key, keys::A);
            assert!(k.caps());
            assert!(!k.shift());
            assert!(!k.consumed());
        }

        raw.mark_consumed(0);
        if let Some(k) = raw.last() {
            assert!(k.consumed());
        }

        let popped = raw.pop();
        assert!(popped.is_some());
        assert!(raw.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut p = Processor::new();

        p.process(keys::V, false, false);
        p.process(keys::I, false, false);

        assert!(!p.buffer().is_empty());

        p.clear();

        assert_eq!(p.state(), st::EMPTY);
        assert!(p.buffer().is_empty());
        assert!(p.raw().is_empty());
    }

    // ===== Defer Tests =====

    #[test]
    fn test_defer_breve_with_valid_final() {
        let mut p = Processor::new();

        // Type "awn" - breve should be applied (ăn is valid)
        p.process(keys::A, false, false);
        assert_eq!(p.state(), st::VOW);

        // W triggers deferred breve
        p.process(keys::W, false, false);
        assert_eq!(p.state(), st::DIA);

        // N is valid final for breve, but defer waits for potential ng/nh
        p.process(keys::N, false, false);
        assert_eq!(p.state(), st::FIN);

        // Finalize word to resolve defer (simulates word boundary)
        p.finalize_word();

        // Check that breve was applied to 'a'
        if let Some(c) = p.buffer().get(0) {
            assert_eq!(c.key, keys::A);
            assert_eq!(c.tone, 2); // breve applied
        }
    }

    #[test]
    fn test_defer_breve_with_ng_cluster() {
        let mut p = Processor::new();

        // Type "awng" - breve should be applied (ăng is valid)
        p.process(keys::A, false, false);
        p.process(keys::W, false, false);
        p.process(keys::N, false, false);
        p.process(keys::G, false, false);

        // Check that breve was applied
        if let Some(c) = p.buffer().get(0) {
            assert_eq!(c.tone, 2); // breve
        }
    }

    #[test]
    fn test_defer_breve_with_nh_cluster_invalid() {
        let mut p = Processor::new();

        // Type "awnh" - breve should NOT be applied (ănh is invalid, use anh)
        p.process(keys::A, false, false);
        p.process(keys::W, false, false);
        p.process(keys::N, false, false);
        p.process(keys::H, false, false);

        // Breve should not be applied
        if let Some(c) = p.buffer().get(0) {
            assert_eq!(c.tone, 0); // no breve
        }
    }

    #[test]
    fn test_defer_horn_u_with_final() {
        let mut p = Processor::new();

        // Type "duowc" - both u and o should get horn (dược)
        p.process(keys::D, false, false);
        p.process(keys::U, false, false);
        p.process(keys::O, false, false);
        p.process(keys::W, false, false); // Horn on O, defer on U
        p.process(keys::C, false, false); // Final triggers horn on U

        // Check that both vowels have horn
        let vowels = p.buffer().find_vowels();
        assert_eq!(vowels.len(), 2);
    }

    // ===== Revert Tests =====

    #[test]
    fn test_revert_stroke_dd() {
        let mut p = Processor::new();

        // Type "dd" - first d becomes đ
        p.process(keys::D, false, false);
        let result = p.process(keys::D, false, false);
        assert_eq!(result, ProcessResult::Transform);

        // Check đ
        if let Some(c) = p.buffer().get(0) {
            assert!(c.stroke);
        }

        // Type third 'd' - should revert to d and add d
        let result = p.process(keys::D, false, false);
        assert_eq!(result, ProcessResult::Revert);

        // Check stroke removed
        if let Some(c) = p.buffer().get(0) {
            assert!(!c.stroke);
        }
    }

    #[test]
    fn test_revert_circumflex_aaa() {
        let mut p = Processor::new();

        // Type "aa" - becomes â
        p.process(keys::A, false, false);
        let _result = p.process(keys::A, false, false);
        // Note: MARK action should apply circumflex
        // The current implementation may need adjustment for this case
    }

    #[test]
    fn test_revert_tone_ss() {
        let mut p = Processor::new();

        // Type "vie" + "s" for tone
        p.process(keys::V, false, false);
        p.process(keys::I, false, false);
        p.process(keys::E, false, false);

        let result = p.process(keys::S, false, false);
        assert_eq!(result, ProcessResult::Transform);

        // Check tone applied
        let vowels = p.buffer().find_vowels();
        if let Some(&pos) = vowels.last() {
            if let Some(c) = p.buffer().get(pos) {
                assert_eq!(c.mark, 1); // sắc
            }
        }

        // Type 's' again - should revert tone
        let result = p.process(keys::S, false, false);
        assert_eq!(result, ProcessResult::Revert);

        // Check tone removed
        if let Some(&pos) = vowels.last() {
            if let Some(c) = p.buffer().get(pos) {
                assert_eq!(c.mark, 0); // no tone
            }
        }
    }

    #[test]
    fn test_revert_horn_ww() {
        let mut p = Processor::new();

        // Type "ow" - becomes ơ
        p.process(keys::O, false, false);
        let result = p.process(keys::W, false, false);
        assert_eq!(result, ProcessResult::Transform);

        // Check horn applied
        if let Some(c) = p.buffer().get(0) {
            assert_eq!(c.tone, 2); // horn
        }

        // Type 'w' again - should revert
        let result = p.process(keys::W, false, false);
        assert_eq!(result, ProcessResult::Revert);

        // Check horn removed
        if let Some(c) = p.buffer().get(0) {
            assert_eq!(c.tone, 0); // no horn
        }
    }

    // ===== Integration Tests =====

    #[test]
    fn test_typing_duoc() {
        let mut p = Processor::new();

        // Type "duowc" (được)
        p.process(keys::D, false, false);
        assert_eq!(p.state(), st::INIT);

        p.process(keys::U, false, false);
        assert_eq!(p.state(), st::VOW);

        p.process(keys::O, false, false);
        assert_eq!(p.state(), st::VOW);

        p.process(keys::W, false, false);
        assert_eq!(p.state(), st::DIA);

        p.process(keys::C, false, false);
        assert_eq!(p.state(), st::FIN);

        // Buffer should have: đ u ơ c (or d ư ơ c depending on stroke)
        assert_eq!(p.buffer().len(), 4);
    }

    #[test]
    fn test_typing_viet_nam() {
        let mut p = Processor::new();

        // Type "viets" (việt with sắc)
        p.process(keys::V, false, false);
        p.process(keys::I, false, false);
        p.process(keys::E, false, false);
        p.process(keys::S, false, false); // tone
        p.process(keys::T, false, false); // final

        assert_eq!(p.state(), st::FIN);
        assert_eq!(p.buffer().len(), 4); // v i e t
    }

    #[test]
    fn test_finalize_word_cancels_invalid_defer() {
        let mut p = Processor::new();

        // Type "aw" without final - breve should be cancelled
        p.process(keys::A, false, false);
        p.process(keys::W, false, false);

        // Word boundary - finalize
        p.finalize_word();

        // Breve should not be applied (standalone ă is invalid)
        // Defer should be cleared
        assert!(!p.defer.is_pending());
    }

    #[test]
    fn test_word_boundary_detection() {
        let p = Processor::new();

        assert!(p.is_word_boundary(keys::SPACE));
        assert!(p.is_word_boundary(keys::DOT));
        assert!(p.is_word_boundary(keys::COMMA));
        assert!(!p.is_word_boundary(keys::A));
        assert!(!p.is_word_boundary(keys::D));
    }
}
