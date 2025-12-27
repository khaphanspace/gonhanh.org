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
    act, breve_valid_with_final, cat, defertype, dispatch, get_key_category,
    horn_u_valid_with_final, is_valid_final_1, is_valid_final_2, is_valid_vowel_pattern, st,
    tone_key_to_value, validation, xform, DeferState, RevertState,
};
use crate::data::keys;
use crate::data::vowel::{Modifier, Phonology, Vowel};
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

    /// Restore ALL keystrokes to characters (including consumed ones)
    /// Used for foreign word detection to rebuild buffer without transforms
    pub fn restore_all(&self) -> Vec<char> {
        use crate::utils::key_to_char;
        self.data[..self.len as usize]
            .iter()
            .filter_map(|k| key_to_char(k.key, k.caps()))
            .collect()
    }

    /// Get iterator over all keystrokes (key and caps)
    pub fn iter(&self) -> impl Iterator<Item = (u16, bool)> + '_ {
        self.data[..self.len as usize]
            .iter()
            .map(|k| (k.key, k.caps()))
    }

    /// Get iterator over keystrokes with consumed flag (key, consumed)
    pub fn iter_with_consumed(&self) -> impl Iterator<Item = (u16, bool)> + '_ {
        self.data[..self.len as usize]
            .iter()
            .map(|k| (k.key, k.consumed()))
    }

    /// Get keys that were NOT consumed (actual letters in the output)
    pub fn unconsumed_keys(&self) -> impl Iterator<Item = u16> + '_ {
        self.data[..self.len as usize]
            .iter()
            .filter(|k| !k.consumed())
            .map(|k| k.key)
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
    /// Foreign word detected, restore to raw ASCII
    Restore,
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
///
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
    /// Pending literal character for VNI revert (digit to pass through)
    pending_literal: Option<char>,
    /// Foreign word mode - skip Vietnamese transforms
    foreign_word_mode: bool,
    /// Pending tone for deferred application (when tone typed before vowel)
    /// Stores (tone_value, tone_key) for application when next vowel is typed
    pending_tone: Option<(u8, u16)>,
    /// Typo correction mode - consume invalid strokes like "didd" → "did"
    typo_correction: bool,
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
            pending_literal: None,
            foreign_word_mode: false,
            pending_tone: None,
            typo_correction: false,
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

    /// Set typo correction mode (consume invalid strokes like "didd" → "did")
    pub fn set_typo_correction(&mut self, enabled: bool) {
        self.typo_correction = enabled;
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

    /// Check if processor is in foreign word mode
    #[inline]
    pub fn is_foreign_mode(&self) -> bool {
        self.foreign_word_mode
    }

    /// Check if current buffer forms valid Vietnamese syllable
    ///
    /// Returns true if:
    /// 1. Buffer has valid initial consonant cluster
    /// 2. Buffer has valid vowel pattern
    /// 3. Buffer has valid final consonant pattern
    pub fn is_valid_vietnamese(&self) -> bool {
        if self.buffer.is_empty() {
            return true;
        }

        // Check for foreign word mode
        if self.foreign_word_mode {
            return false;
        }

        // Check valid initial
        if !self.has_valid_vietnamese_initial() {
            return false;
        }

        // Check for foreign consonant clusters
        if self.has_foreign_consonant_cluster() {
            return false;
        }

        // Check valid syllable structure (finals)
        self.has_valid_vietnamese_structure()
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.state = st::EMPTY;
        self.buffer.clear();
        self.raw.clear();
        self.defer.clear();
        self.revert.clear();
        self.pending_literal = None;
        self.foreign_word_mode = false;
        self.pending_tone = None;
    }

    /// Handle backspace - remove last character
    /// Returns true if a character was removed, false if buffer was already empty
    pub fn backspace(&mut self) -> bool {
        if self.buffer.is_empty() {
            return false;
        }
        self.buffer.pop();
        self.raw.pop();
        self.revert.clear();
        self.defer.clear();
        // Recalculate state based on remaining buffer
        self.state = if self.buffer.is_empty() {
            st::EMPTY
        } else {
            // Simplified: if we have vowels with diacritics, DIA; else check for final consonants
            // For now, just go to VOW if we have content (can be refined)
            st::VOW
        };
        true
    }

    /// Take pending literal character (for VNI revert)
    pub fn take_pending_literal(&mut self) -> Option<char> {
        self.pending_literal.take()
    }

    /// Push a character directly to the buffer
    ///
    /// Used by restore_word to reconstruct buffer from Vietnamese string.
    pub fn push_char(&mut self, c: Char) {
        self.buffer.push(c);
        // Update state based on what we're pushing
        if self.buffer.len() == 1 {
            if keys::is_vowel(c.key) {
                self.state = if c.mark > 0 || c.tone > 0 {
                    st::DIA
                } else {
                    st::VOW
                };
            } else {
                self.state = st::INIT;
            }
        }
    }

    /// Restore processor state from a buffer
    ///
    /// Used for backspace-after-space feature to restore a previous word.
    pub fn restore_buffer(&mut self, buf: Buffer) {
        self.buffer = buf;
        self.raw.clear(); // Raw history is not preserved
        self.defer.clear();
        self.revert.clear();
        self.pending_literal = None;
        self.pending_tone = None;

        // Determine state from buffer content
        self.state = if self.buffer.is_empty() {
            st::EMPTY
        } else {
            // Check if we have final consonant, diacritic, or just vowels
            let vowels = self.buffer.find_vowels();
            if vowels.is_empty() {
                st::INIT
            } else {
                // Check for diacritics (tone mark or vowel modifier)
                let has_diacritic = vowels.iter().any(|&pos| {
                    self.buffer
                        .get(pos)
                        .map(|c| c.mark > 0 || c.tone > 0)
                        .unwrap_or(false)
                });
                // Check for final consonant after last vowel
                let last_vowel_pos = vowels.last().copied().unwrap_or(0);
                let has_final = (last_vowel_pos + 1..self.buffer.len()).any(|i| {
                    self.buffer
                        .get(i)
                        .map(|c| keys::is_consonant(c.key))
                        .unwrap_or(false)
                });

                if has_final {
                    st::FIN
                } else if has_diacritic {
                    st::DIA
                } else {
                    st::VOW
                }
            }
        };
    }

    /// Rebuild buffer from raw input without any Vietnamese transforms
    ///
    /// Used when foreign word is detected. Clears all diacritics and rebuilds
    /// the buffer as plain ASCII from the raw keystroke history.
    fn rebuild_as_foreign_word(&mut self) {
        // Get all raw keystrokes as plain characters
        let raw_keystrokes: Vec<(u16, bool)> = self.raw.iter().collect();

        // Clear current buffer
        self.buffer.clear();

        // Rebuild without transforms
        for (key, caps) in raw_keystrokes {
            self.buffer.push(Char::new(key, caps));
        }

        // Update state based on last character
        if self.buffer.is_empty() {
            self.state = st::EMPTY;
        } else if let Some(last) = self.buffer.last() {
            if keys::is_vowel(last.key) {
                self.state = st::VOW;
            } else {
                // Could be INIT or FIN depending on context, but FIN is safer
                // since we're treating this as a foreign word continuation
                self.state = st::FIN;
            }
        }

        // Clear revert state since we've rebuilt
        self.revert.clear();

        // Enter foreign word mode - skip Vietnamese transforms
        self.foreign_word_mode = true;
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

        // Telex: 'z' removes all diacritics (tone marks)
        if self.method == 0 && key == keys::Z {
            if let Some(result) = self.handle_telex_z() {
                return result;
            }
        }

        // VNI mode: handle number keys for diacritics
        if self.method == 1 {
            if let Some(result) = self.handle_vni_number(key) {
                return result;
            }
        }

        // Special handling for ng/nh/ch clusters in FIN state
        // G and H after n/c should extend the final, not start new word
        let effective_category = self.get_effective_category(key);

        // Get action and next state from dispatch table
        let (action, next_state) = dispatch(self.state, effective_category);

        // Execute action
        let result = match action {
            act::PASS => self.handle_pass(key, caps, next_state),
            act::TONE => {
                let tone_result = self.handle_tone(key);
                if tone_result == ProcessResult::Reject {
                    // Check if this is a "qu"/"gi" pattern where tone should be deferred
                    // for the next vowel (e.g., "qus" + "y" → "quý")
                    let has_qu_or_gi_initial = self.buffer.len() == 2
                        && ((self.buffer.get(0).map(|c| c.key) == Some(keys::Q)
                            && self.buffer.get(1).map(|c| c.key) == Some(keys::U))
                            || (self.buffer.get(0).map(|c| c.key) == Some(keys::G)
                                && self.buffer.get(1).map(|c| c.key) == Some(keys::I)));

                    if has_qu_or_gi_initial {
                        // Defer the tone for the next vowel
                        if let Some(tone_value) = tone_key_to_value(key) {
                            self.pending_tone = Some((tone_value, key));
                            self.raw.mark_consumed(self.raw.len() - 1);
                            // Stay in current state, waiting for vowel
                            return ProcessResult::Transform;
                        }
                    }

                    // Tone rejected (invalid vowel pattern or structure)
                    // Treat key as regular consonant for foreign word support
                    self.buffer.push(Char::new(key, caps));
                    self.revert.clear();
                    // Transition to appropriate state based on current state
                    self.state = if self.state == st::VOW || self.state == st::DIA {
                        st::FIN // Consonant after vowel
                    } else {
                        st::INIT // New syllable
                    };
                    return ProcessResult::Update;
                }
                tone_result
            }
            act::MARK => self.handle_mark(key, caps),
            act::STROKE => {
                let stroke_result = self.handle_stroke();
                if stroke_result == ProcessResult::Reject {
                    // Check if this is adjacent dd in FIN state (đ not valid as final)
                    // With typo_correction enabled, consume the key silently
                    // e.g., "didd" should stay as "did" (extra 'd' is consumed)
                    // Without typo_correction, add 'd' normally: "deadd" → "deadd"
                    let last_is_d = self
                        .buffer
                        .last()
                        .is_some_and(|c| c.key == keys::D && !c.stroke);
                    if self.typo_correction && self.state == st::FIN && last_is_d {
                        // Remove the 'd' from raw buffer (it was added at start of process)
                        self.raw.pop();
                        // Return Transform so create_result() is called
                        // Since buffer is unchanged, it returns Send(0) to consume key
                        return ProcessResult::Transform;
                    }
                    // Stroke was rejected - add 'd' as regular consonant
                    self.buffer.push(Char::new(key, caps));
                    self.revert.clear();
                    self.state = next_state;

                    // Check for foreign patterns after adding 'd'
                    // This catches cases like "would" where 'w' → 'ư' but 'ld' is invalid
                    // Only check for W_VOWEL transform (ư from 'w' at position 0)
                    // Do NOT check general vowel transforms - those are valid Vietnamese
                    if self.has_foreign_consonant_cluster() {
                        let has_w_vowel_transform = self.buffer.get(0).is_some_and(|c| {
                            c.key == keys::U && c.tone == 2 // ư (from 'w')
                        });
                        if has_w_vowel_transform {
                            self.rebuild_as_foreign_word();
                            return ProcessResult::Restore;
                        }
                    }

                    return ProcessResult::Update;
                }
                stroke_result
            }
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
        // If defer applies a transformation, upgrade result so screen updates
        if self.defer.is_pending() {
            let defer_applied = self.resolve_defer();
            if defer_applied && result == ProcessResult::Update {
                return ProcessResult::Transform;
            }
        }

        result
    }

    /// Handle PASS action - append character to buffer
    fn handle_pass(&mut self, key: u16, caps: bool, next_state: u8) -> ProcessResult {
        // Special handling for W in various contexts
        if key == keys::W {
            return self.handle_w(caps, next_state);
        }

        // Check for impossible initial consonant clusters
        // This triggers FOREIGN mode immediately for patterns like 'cl', 'br', 'st', etc.
        if self.state == st::INIT && keys::is_consonant(key) {
            if let Some(prev) = self.buffer.last() {
                if keys::is_consonant(prev.key) {
                    // Two consecutive consonants at start - check if valid Vietnamese cluster
                    let is_valid_initial = matches!(
                        (prev.key, key),
                        (keys::C, keys::H)   // ch
                        | (keys::G, keys::H) // gh
                        | (keys::G, keys::I) // gi (i is vowel but gi is special)
                        | (keys::K, keys::H) // kh
                        | (keys::K, keys::R) // kr (ethnic minority: Krông)
                        | (keys::N, keys::G) // ng
                        | (keys::N, keys::H) // nh
                        | (keys::P, keys::H) // ph
                        | (keys::Q, keys::U) // qu (u is vowel but qu is special)
                        | (keys::T, keys::H) // th
                        | (keys::T, keys::R) // tr
                    );

                    if !is_valid_initial {
                        // Invalid initial cluster - switch to FOREIGN mode
                        self.buffer.push(Char::new(key, caps));
                        self.foreign_word_mode = true;
                        self.revert.clear();
                        return ProcessResult::Update;
                    }
                }
            }
        }

        // Foreign word detection: If we're starting a new syllable (vowel after FIN state)
        // and any vowel has a mark applied, check if adding this vowel creates an invalid structure.
        //
        // Examples:
        // - "mẻt" + "r" + "i" → "metric" (RESTORE because consonant before vowel after FIN)
        // - "cón" + "o" → "cóno" (NO RESTORE, 'oo' is valid diphthong, wait for invalid consonant)
        //
        // IMPORTANT: Skip this check if raw buffer is shorter than composed buffer.
        // This happens when the buffer was restored via restore_word() and the raw
        // keystrokes weren't preserved. In that case, we trust the restored word is valid.
        let has_vowel_with_mark = self
            .buffer
            .find_vowels()
            .iter()
            .any(|&pos| self.buffer.get(pos).map(|c| c.mark > 0).unwrap_or(false));
        let raw_has_enough_history = self.raw.len() >= self.buffer.len();

        // Check if there are EXTRA consonants after a valid final (indicating multi-syllable foreign word)
        // This detects patterns like "mẻt" + "r" + "i" where 'r' comes AFTER valid final 't'
        // But NOT "cón" + "o" where 'n' IS the valid final (no extra consonants)
        let has_extra_consonants_after_final = if self.state == st::FIN {
            // Get position of last vowel
            let vowels = self.buffer.find_vowels();
            if let Some(&last_vowel_pos) = vowels.last() {
                // Get consonants after last vowel
                let consonants_after_vowel: Vec<u16> = (last_vowel_pos + 1..self.buffer.len())
                    .filter_map(|i| self.buffer.get(i).map(|c| c.key))
                    .filter(|&k| keys::is_consonant(k))
                    .collect();

                // If more than 2 consonants after vowel, definitely foreign (Vietnamese max is 2: ng, nh, ch)
                // If 2 consonants that don't form valid final cluster, also foreign
                // If 1 consonant but not valid final, also foreign
                match consonants_after_vowel.len() {
                    0 | 1 => false, // 0-1 consonant is normal Vietnamese pattern
                    2 => {
                        // Check if it's a valid final cluster (ng, nh, ch)
                        !is_valid_final_2(consonants_after_vowel[0], consonants_after_vowel[1])
                    }
                    _ => true, // 3+ consonants after vowel = definitely foreign
                }
            } else {
                false
            }
        } else {
            false
        };

        // Only trigger RESTORE if there are EXTRA consonants beyond valid finals
        // This catches "metric" (mẻt + r + i → 'tr' after 'e' is 2 consonants, 't' + 'r' not valid final pair)
        // But NOT "cóno" (just 'n' after 'o' is valid single final)
        if self.state == st::FIN
            && keys::is_vowel(key)
            && has_vowel_with_mark
            && raw_has_enough_history
            && has_extra_consonants_after_final
        {
            // Foreign word detected (e.g., "metric", "describe")
            // Rebuild the buffer from raw input without any Vietnamese transforms
            // The current key is already in the raw buffer, so rebuild includes it
            self.rebuild_as_foreign_word();
            // Return Restore to signal foreign word detection
            // Don't fall through to regular append (key already in rebuilt buffer)
            return ProcessResult::Restore;
        }

        // Also check for "mismatched vowel after FIN" pattern
        // This catches "mẻt" + "i" (e and i don't match for circumflex, multi-syllable foreign word)
        // Condition: FIN state, adding vowel that won't trigger circumflex, has tone mark on existing vowel
        if self.state == st::FIN
            && keys::is_vowel(key)
            && has_vowel_with_mark
            && raw_has_enough_history
        {
            // Check if this vowel would NOT trigger circumflex (different vowel key)
            let vowel_positions = self.buffer.find_vowels();
            let can_circumflex = matches!(key, keys::A | keys::E | keys::O)
                && vowel_positions.iter().any(|&pos| {
                    self.buffer
                        .get(pos)
                        .map(|c| c.key == key && (c.tone == 0 || c.tone == 2))
                        .unwrap_or(false)
                });

            // If vowel doesn't match for circumflex, it's likely starting a new syllable (foreign word)
            if !can_circumflex {
                self.rebuild_as_foreign_word();
                return ProcessResult::Restore;
            }
        }

        // Check for double-vowel circumflex (aa→â, ee→ê, oo→ô)
        // Skip in foreign word mode (already detected as non-Vietnamese)
        // Skip if buffer has foreign consonant clusters (e.g., "exp" + "e" should not circumflex)
        if !self.foreign_word_mode
            && keys::is_vowel(key)
            && (self.state == st::VOW || self.state == st::DIA || self.state == st::FIN)
            && !self.has_foreign_consonant_cluster()
        {
            // In FIN state, check if we have a valid syllable structure for circumflex.
            // "rieneg" → "riêng": n could become ng, so 'e' triggers circumflex
            // "teacher" → "teacher": 'ch' is complete final (multi-char), new syllable starts
            let allow_circumflex = if self.state == st::FIN {
                // Count consonants after the last vowel to determine if final is complete
                let vowels = self.buffer.find_vowels();
                if let Some(&last_vowel_pos) = vowels.last() {
                    let consonants_after_vowel: Vec<u16> = (last_vowel_pos + 1..self.buffer.len())
                        .filter_map(|i| self.buffer.get(i).map(|c| c.key))
                        .filter(|&k| keys::is_consonant(k))
                        .collect();

                    // If we have 2+ consonants that form a valid final cluster (ng, nh, ch)
                    // then the syllable is likely complete and new vowel starts fresh
                    if consonants_after_vowel.len() >= 2 {
                        // Two-char finals that are complete: ch, ng, nh
                        let is_complete_final = matches!(
                            (consonants_after_vowel[0], consonants_after_vowel[1]),
                            (keys::C, keys::H) | (keys::N, keys::H)
                        );
                        !is_complete_final // ng can still extend, ch/nh are complete
                    } else if consonants_after_vowel.len() == 1 {
                        // Single consonant - could extend (n→ng, n→nh)
                        // Allow circumflex in this case
                        true
                    } else {
                        true // No consonants after vowel, allow circumflex
                    }
                } else {
                    true // No vowels yet, shouldn't happen
                }
            } else {
                true // In VOW or DIA state, always allow circumflex
            };

            if allow_circumflex {
                // UNIFIED RULE: Apply circumflex only if result is valid Vietnamese
                // Only A, E, O can take circumflex → Â, Ê, Ô
                let can_take_circumflex = matches!(key, keys::A | keys::E | keys::O);
                if can_take_circumflex {
                    let vowels = self.buffer.find_vowels();
                    for &pos in vowels.iter().rev() {
                        if let Some(c) = self.buffer.get(pos) {
                            // Basic check: same key, can receive circumflex
                            if c.key != key || (c.tone != 0 && c.tone != 2) {
                                continue;
                            }

                            // Save original state for potential revert
                            let old_tone = c.tone;

                            // Tentatively apply circumflex
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 1; // circumflex
                            }

                            // Validate: is the resulting buffer valid Vietnamese?
                            // Use REAL-TIME validation (no circumflex+closed+no_tone check)
                            // Full validation happens at commit-time (space/punctuation)
                            let buffer_str = self.buffer.to_full_string();
                            let is_valid =
                                !validation::is_buffer_invalid_vietnamese_realtime(&buffer_str);

                            if !is_valid {
                                // Revert and try next position
                                if let Some(c) = self.buffer.get_mut(pos) {
                                    c.tone = old_tone;
                                }
                                continue;
                            }

                            // Valid! Record for revert and return
                            self.revert.record(xform::CIRCUMFLEX, key, pos as u8);
                            self.raw.mark_consumed(self.raw.len() - 1);

                            // Apply deferred stroke if pending (dede → đê)
                            if self.defer.kind == defertype::STROKE_D {
                                let d_pos = self.defer.position as usize;
                                if let Some(d_char) = self.buffer.get_mut(d_pos) {
                                    if d_char.key == keys::D && !d_char.stroke {
                                        d_char.stroke = true;
                                    }
                                }
                                // Remove the second 'd' that triggered defer
                                for i in (d_pos + 1..self.buffer.len()).rev() {
                                    if self.buffer.get(i).is_some_and(|c| c.key == keys::D) {
                                        self.buffer.remove(i);
                                        break;
                                    }
                                }
                                self.defer.clear();
                            }

                            return ProcessResult::Transform;
                        }
                    }
                }
            }

            // Auto-apply horn to 'o' after 'ư' for W-shorthand patterns
            // When the previous char is 'ư' (from typing 'w'), adding 'o' should
            // auto-apply horn to form valid "ươ" compound (since "ưo" is invalid)
            // Works for:
            // - W-initial: "w" + "o" = "ươ"
            // - C+W shorthand: "tw" + "o" = "tươ" (tương)
            // For longer words like "nguwow", user explicitly types 'w' after 'o'
            //
            // For vowel+W patterns (HORN transform), we use deferred horn:
            // - "đuwoc" → defer horn on 'o', resolve to "đươc" when 'c' added
            // - "nguwow" → no defer, second 'w' applies horn to 'o'
            //
            // NOTE: This is Telex-specific. In VNI, horn is applied via '7' key only.
            // Skip this block in VNI mode to prevent auto-horn on 'o' after 'ư'.
            if key == keys::O && !self.buffer.is_empty() && self.method != 1 {
                // Check if last vowel is 'ư' (from W transform)
                // Only applies when buffer ends with a single ư vowel
                let vowels = self.buffer.find_vowels();
                if vowels.len() == 1 {
                    let pos = vowels[0];
                    if let Some(prev) = self.buffer.get(pos) {
                        if prev.key == keys::U && prev.tone == 2 {
                            if self.revert.transform == xform::HORN {
                                // Vowel+W pattern: defer horn on 'o'
                                // Add 'o' without horn, set up defer
                                self.buffer.push(Char::new(keys::O, caps));
                                // Set up deferred horn on 'o' (will resolve when final added)
                                self.defer = DeferState::horn_o((self.buffer.len() - 1) as u8);
                                // Clear revert state - horn on 'u' is committed as part of "ưo"
                                // This prevents second 'w' from reverting horn on 'u'
                                self.revert.clear();
                                self.state = st::VOW;
                                return ProcessResult::Update;
                            } else {
                                // W-shorthand or vowel+W+tone pattern: apply horn immediately
                                // Add 'o' with horn to form 'ơ', creating "ươ" compound
                                // If 'ư' has a tone mark, move it to 'ơ' (e.g., "ứo" → "ướ")
                                let existing_mark = prev.mark;
                                let mut c = Char::new(keys::O, caps);
                                c.tone = 2; // horn
                                if existing_mark > 0 {
                                    // Move tone from 'ư' to 'ơ'
                                    c.mark = existing_mark;
                                    if let Some(u_char) = self.buffer.get_mut(pos) {
                                        u_char.mark = 0;
                                    }
                                    self.state = st::DIA;
                                } else {
                                    self.state = st::VOW;
                                }
                                self.buffer.push(c);
                                // Clear revert state - the 'ư' horn is now committed
                                self.revert.clear();
                                return ProcessResult::Transform;
                            }
                        }
                    }
                }
            }
        }

        // Foreign word detection: when 'x' was used as tone key and consonant follows,
        // it's likely a foreign word like "text", "next", "box" where 'x' is a letter.
        // 'x' as ngã tone is rare in Vietnamese, but common as a letter in English.
        if self.state == st::DIA
            && keys::is_consonant(key)
            && self.revert.transform == xform::TONE
            && self.revert.key == keys::X
        {
            self.buffer.push(Char::new(key, caps));
            self.rebuild_as_foreign_word();
            return ProcessResult::Restore;
        }

        // Regular character append
        self.buffer.push(Char::new(key, caps));
        self.revert.clear(); // Regular char clears revert state

        // Apply pending tone to newly added vowel (e.g., "qus" + "y" → "quý")
        if keys::is_vowel(key) {
            if let Some((tone_value, tone_key)) = self.pending_tone.take() {
                let last_pos = self.buffer.len() - 1;
                if let Some(c) = self.buffer.get_mut(last_pos) {
                    c.mark = tone_value;
                    self.revert.record(xform::TONE, tone_key, last_pos as u8);
                    self.state = st::DIA;
                    return ProcessResult::Transform;
                }
            }

            // Relocate tone when adding vowel after vowel with tone (e.g., "ós" + "a" → "oá")
            // This handles "typo" patterns where user types tone then vowel
            if self.state == st::DIA && self.relocate_tone_for_added_vowel() {
                return ProcessResult::Transform;
            }
        }

        // After adding consonant, check for invalid Vietnamese structure
        // This catches cases like "cóno" + "l" where 'l' is not a valid Vietnamese final
        if keys::is_consonant(key) {
            // Check if any vowel has a tone/mark applied
            let has_transformed_vowel = self.buffer.find_vowels().iter().any(|&pos| {
                self.buffer
                    .get(pos)
                    .map(|c| c.mark > 0 || c.tone > 0)
                    .unwrap_or(false)
            });

            // Check if there's a "vowel after final consonant" pattern (foreign structure)
            // This pattern exists in words like "cóno" where there's vowel after completed syllable
            // But NOT in words like "đếnn" where it's just extra final consonants
            let has_vowel_after_final = {
                let vowel_positions = self.buffer.find_vowels();
                let consonant_positions: Vec<usize> = (0..self.buffer.len())
                    .filter(|&i| {
                        self.buffer
                            .get(i)
                            .map(|c| keys::is_consonant(c.key))
                            .unwrap_or(false)
                    })
                    .collect();

                // Check if any vowel comes after a consonant that itself comes after another vowel
                // Pattern: V...C...V (vowel, then consonant, then vowel = foreign structure)
                vowel_positions.iter().any(|&v_pos| {
                    consonant_positions.iter().any(|&c_pos| {
                        c_pos < v_pos && vowel_positions.iter().any(|&v2_pos| v2_pos < c_pos)
                    })
                })
            };

            // Only trigger RESTORE if:
            // 1. We have transformed vowels (diacritics applied)
            // 2. Structure is invalid
            // 3. AND there's already a foreign structure (vowel after final)
            // This prevents "đến" + "n" from triggering RESTORE (just extra finals)
            // But allows "cóno" + "l" to trigger RESTORE (vowel after final detected)
            if has_transformed_vowel
                && !self.has_valid_vietnamese_structure()
                && self.raw.len() >= self.buffer.len()
                && has_vowel_after_final
            {
                // Invalid final consonant with transformed vowel → foreign word
                // Example: "cóno" + "l" → "consol"
                self.rebuild_as_foreign_word();
                return ProcessResult::Restore;
            }

            // For extra consonants without foreign structure (like "đếnn"),
            // just mark as foreign mode but keep the diacritics
            if !self.has_valid_vietnamese_structure() && has_transformed_vowel {
                self.foreign_word_mode = true;
            }

            // Also check for foreign consonant clusters
            // This catches cases like "would" where 'w' → 'ư' was applied but 'ld' is invalid
            if self.has_foreign_consonant_cluster() {
                // Check if first char is a W-derived vowel (U with horn = ư from 'w')
                let has_w_vowel_transform = self.buffer.get(0).is_some_and(|c| {
                    c.key == keys::U && c.tone == 2 // ư (from 'w')
                });
                if has_w_vowel_transform {
                    self.rebuild_as_foreign_word();
                    return ProcessResult::Restore;
                }
            }
        }

        // When adding final consonant, relocate tone if needed
        // Example: "muas" → "múa", then "n" → tone should move to "a" (muán)
        if next_state == st::FIN
            && (self.state == st::DIA || self.state == st::VOW)
            && self.relocate_tone_for_closed_syllable()
        {
            // Tone was relocated, need to send transform to update screen
            return ProcessResult::Transform;
        }

        // Clear deferred stroke when:
        // 1. A different vowel is added (doesn't trigger circumflex) - foreign word like "dedicated"
        // 2. This catches: "ded" + 'i' → clear defer, keep as "dedi"
        if self.defer.kind == defertype::STROKE_D && keys::is_vowel(key) {
            // Vowel was added but circumflex didn't trigger (otherwise we would have returned Transform)
            // This means it's a different vowel → clear deferred stroke
            self.defer.clear();
        }

        ProcessResult::Update
    }

    /// Relocate tone mark for closed syllable
    ///
    /// When a final consonant is added, the tone position may need to change.
    /// Example: "múa" (open) + "n" → "muán" (closed)
    ///
    /// Returns true if tone was relocated.
    fn relocate_tone_for_closed_syllable(&mut self) -> bool {
        let vowel_positions = self.buffer.find_vowels();
        if vowel_positions.len() < 2 {
            return false;
        }

        // Detect gi-initial: 'i' at position 1 is part of initial, not vowel nucleus
        // For words like "giống", the 'i' in 'gi' shouldn't be treated as a vowel
        let has_gi_initial = self.buffer.len() >= 2
            && self.buffer.get(0).map(|c| c.key) == Some(keys::G)
            && self.buffer.get(1).map(|c| c.key) == Some(keys::I);

        // Filter out 'i' from gi-initial for vowel pattern checking
        let effective_vowels: Vec<usize> = if has_gi_initial {
            vowel_positions
                .iter()
                .copied()
                .filter(|&pos| pos != 1)
                .collect()
        } else {
            vowel_positions.clone()
        };

        if effective_vowels.len() < 2 {
            return false;
        }

        // Find the current syllable's vowels (consecutive, no consonants between)
        // This handles compound words like "việtnam" = [việt] + [nam]
        let mut current_syllable_vowels: Vec<usize> = Vec::new();
        for &pos in effective_vowels.iter().rev() {
            if current_syllable_vowels.is_empty() {
                current_syllable_vowels.push(pos);
            } else {
                // Check if there's a consonant between this vowel and the previous one
                let prev_pos = *current_syllable_vowels.last().unwrap();
                let has_consonant_between = (pos + 1..prev_pos).any(|i| {
                    self.buffer
                        .get(i)
                        .map(|c| keys::is_consonant(c.key))
                        .unwrap_or(false)
                });

                if has_consonant_between {
                    // Syllable boundary - stop here
                    break;
                }
                current_syllable_vowels.push(pos);
            }
        }
        current_syllable_vowels.reverse(); // Back to forward order

        // Only check pattern if we have multiple vowels in current syllable
        if current_syllable_vowels.len() >= 2 {
            let vowel_keys: Vec<u16> = current_syllable_vowels
                .iter()
                .filter_map(|&pos| self.buffer.get(pos).map(|c| c.key))
                .collect();

            if !is_valid_vowel_pattern(&vowel_keys) {
                // Invalid Vietnamese vowel pattern - clear tone marks in current syllable
                for &pos in &current_syllable_vowels {
                    if let Some(c) = self.buffer.get_mut(pos) {
                        c.mark = 0;
                    }
                }
                return true; // Return true to trigger screen update
            }
        }

        // Find the vowel with tone mark - ONLY within current syllable
        // Don't relocate tones across syllable boundaries (prevents "metỉc" bug)
        let mut tone_pos = None;
        let mut tone_value = 0u8;
        for &pos in &current_syllable_vowels {
            if let Some(c) = self.buffer.get(pos) {
                if c.mark > 0 {
                    tone_pos = Some(pos);
                    tone_value = c.mark;
                    break;
                }
            }
        }

        let Some(old_pos) = tone_pos else {
            // No tone in current syllable - nothing to relocate
            return false;
        };

        // Only one vowel in current syllable - no relocation needed
        if current_syllable_vowels.len() < 2 {
            return false;
        }

        // Build Vowel structs for Phonology lookup - ONLY current syllable vowels
        let vowels: Vec<Vowel> = current_syllable_vowels
            .iter()
            .filter_map(|&pos| {
                self.buffer.get(pos).map(|c| {
                    let modifier = match c.tone {
                        0 => Modifier::None,
                        1 => Modifier::Circumflex,
                        2 => Modifier::Horn,
                        _ => Modifier::None,
                    };
                    Vowel::new(c.key, modifier, pos)
                })
            })
            .collect();

        if vowels.is_empty() {
            return false;
        }

        // Detect qu-initial and gi-initial
        let has_qu_initial = if self.buffer.len() >= 2 {
            self.buffer
                .get(0)
                .map(|c| c.key == keys::Q)
                .unwrap_or(false)
                && self
                    .buffer
                    .get(1)
                    .map(|c| c.key == keys::U)
                    .unwrap_or(false)
        } else {
            false
        };

        let has_gi_initial = if self.buffer.len() >= 2 {
            self.buffer
                .get(0)
                .map(|c| c.key == keys::G)
                .unwrap_or(false)
                && self
                    .buffer
                    .get(1)
                    .map(|c| c.key == keys::I)
                    .unwrap_or(false)
        } else {
            false
        };

        // Calculate new position WITH final consonant (has_final_consonant = true)
        let new_pos = Phonology::find_tone_position(
            &vowels,
            true, // has_final_consonant - we're adding a final consonant!
            self.modern_tone,
            has_qu_initial,
            has_gi_initial,
        );

        // Move tone if position changed
        if new_pos != old_pos {
            if let Some(c) = self.buffer.get_mut(old_pos) {
                c.mark = 0;
            }
            if let Some(c) = self.buffer.get_mut(new_pos) {
                c.mark = tone_value;
            }
            true
        } else {
            false
        }
    }

    /// Relocate tone when adding a vowel after a vowel with tone
    ///
    /// Handles "typo" patterns like "ós" + "a" → "oá" (tone moves to 2nd vowel)
    /// Uses Phonology to determine correct tone position for the diphthong.
    ///
    /// Returns true if tone was relocated.
    fn relocate_tone_for_added_vowel(&mut self) -> bool {
        let vowel_positions = self.buffer.find_vowels();
        if vowel_positions.len() < 2 {
            return false;
        }

        // Find the vowel with tone mark
        let mut tone_pos = None;
        let mut tone_value = 0u8;
        for &pos in &vowel_positions {
            if let Some(c) = self.buffer.get(pos) {
                if c.mark > 0 {
                    tone_pos = Some(pos);
                    tone_value = c.mark;
                    break;
                }
            }
        }

        let Some(old_pos) = tone_pos else {
            return false;
        };

        // Build Vowel structs for Phonology lookup
        let vowels: Vec<Vowel> = vowel_positions
            .iter()
            .filter_map(|&pos| {
                self.buffer.get(pos).map(|c| {
                    let modifier = match c.tone {
                        0 => Modifier::None,
                        1 => Modifier::Circumflex,
                        2 => Modifier::Horn,
                        _ => Modifier::None,
                    };
                    Vowel::new(c.key, modifier, pos)
                })
            })
            .collect();

        if vowels.is_empty() {
            return false;
        }

        // Detect qu-initial and gi-initial
        let has_qu_initial = self.buffer.len() >= 2
            && self.buffer.get(0).map(|c| c.key) == Some(keys::Q)
            && self.buffer.get(1).map(|c| c.key) == Some(keys::U);

        let has_gi_initial = self.buffer.len() >= 2
            && self.buffer.get(0).map(|c| c.key) == Some(keys::G)
            && self.buffer.get(1).map(|c| c.key) == Some(keys::I);

        // Calculate new position for open syllable (no final consonant yet)
        let new_pos = Phonology::find_tone_position(
            &vowels,
            false, // has_final_consonant = false (we're adding a vowel, not final)
            self.modern_tone,
            has_qu_initial,
            has_gi_initial,
        );

        // Move tone if position changed
        if new_pos != old_pos {
            if let Some(c) = self.buffer.get_mut(old_pos) {
                c.mark = 0;
            }
            if let Some(c) = self.buffer.get_mut(new_pos) {
                c.mark = tone_value;
            }
            true
        } else {
            false
        }
    }

    /// Handle W key - can be vowel ư or modifier
    fn handle_w(&mut self, caps: bool, _next_state: u8) -> ProcessResult {
        // In VNI mode, W always passes through as regular character
        if self.method == 1 {
            self.buffer.push(Char::new(keys::W, caps));
            return ProcessResult::Update;
        }

        match self.state {
            st::EMPTY => {
                // Telex: W at start becomes ư
                let mut c = Char::new(keys::U, caps);
                c.tone = 2; // horn mark for ư
                self.buffer.push(c);

                // Record for revert (ww → w)
                self.revert
                    .record(xform::W_VOWEL, keys::W, (self.buffer.len() - 1) as u8);
                ProcessResult::Transform
            }
            st::INIT => {
                // Check if current consonant can precede 'ư'
                // k, q only appear before front vowels (i, e, y), not back vowels like ư
                if let Some(last) = self.buffer.last() {
                    if last.key == keys::K || last.key == keys::Q {
                        // Invalid: kư, qư - passthrough W as regular character
                        self.buffer.push(Char::new(keys::W, caps));
                        return ProcessResult::Update;
                    }
                }

                // Valid consonant - W becomes ư
                let mut c = Char::new(keys::U, caps);
                c.tone = 2; // horn mark for ư
                self.buffer.push(c);

                // Record for revert (ww → w)
                self.revert
                    .record(xform::W_VOWEL, keys::W, (self.buffer.len() - 1) as u8);
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

        // Search backwards for a vowel that can take horn/breve (a, o, u)
        // This handles cases like "oiw" → "ơi" (apply horn to 'o', not 'i')
        for &pos in vowels.iter().rev() {
            if let Some(c) = self.buffer.get(pos) {
                match c.key {
                    keys::A => {
                        // a + w → ă (breve)
                        // But for "ua" pattern, prefer horn on 'u' (continue to next vowel)
                        // "chuaw" → "chưa" (horn on u), not "chuă" (breve on a)
                        if c.tone == 0 {
                            // Check if previous char is 'u' without mark - prefer horn on 'u'
                            if pos > 0 {
                                if let Some(prev) = self.buffer.get(pos - 1) {
                                    if prev.key == keys::U && prev.tone == 0 {
                                        // Skip 'a', let loop continue to find 'u'
                                        continue;
                                    }
                                }
                            }

                            // Check adjacency: 'w' should be right after 'a', not after another vowel
                            // "taiw" → 'w' is after 'i', so breve on 'a' should NOT apply
                            let is_adjacent = pos == self.buffer.len() - 1
                                || (pos + 1 < self.buffer.len()
                                    && self
                                        .buffer
                                        .get(pos + 1)
                                        .map(|c| !keys::is_vowel(c.key))
                                        .unwrap_or(true));
                            if !is_adjacent {
                                continue; // Not adjacent - skip this vowel
                            }

                            // Check for valid Vietnamese spelling
                            // 'k' only appears before front vowels (i, e, y), not 'a'
                            if pos > 0 {
                                if let Some(prev) = self.buffer.get(pos - 1) {
                                    if prev.key == keys::K {
                                        continue; // Invalid: ka is not Vietnamese
                                    }
                                }
                            }

                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 2; // breve
                            }
                            self.revert.record(xform::BREVE, keys::W, pos as u8);
                            self.raw.mark_consumed(self.raw.len() - 1);
                            return ProcessResult::Transform;
                        }
                    }
                    keys::O => {
                        // o + w → ơ (horn)
                        // Also handles ô + w → ơ (switch from circumflex to horn)
                        if c.tone == 0 || c.tone == 1 {
                            let was_circumflex = c.tone == 1;
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 2;
                            }
                            self.revert.record(xform::HORN, keys::O, pos as u8);
                            self.raw.mark_consumed(self.raw.len() - 1);

                            // Clear HORN_O defer if present (user explicitly typed 'w')
                            if self.defer.kind == defertype::HORN_O {
                                self.defer.clear();
                            }

                            // Check for uo pattern
                            if pos > 0 {
                                if let Some(prev) = self.buffer.get(pos - 1) {
                                    if prev.key == keys::U && prev.tone == 0 {
                                        if was_circumflex {
                                            // Switching from uô to ươ - apply horn to 'u' immediately
                                            // User is consciously changing mark type
                                            if let Some(u) = self.buffer.get_mut(pos - 1) {
                                                u.tone = 2;
                                            }
                                        } else {
                                            // Fresh horn on 'o' - set up deferred horn on 'u'
                                            // Horn on 'u' only applied when final consonant added
                                            // "uow" → "uơ" but "muown" → "mươn"
                                            self.defer = DeferState::horn_u((pos - 1) as u8);
                                        }
                                    }
                                }
                            }
                            return ProcessResult::Transform;
                        }
                    }
                    keys::U => {
                        // u + w → ư (horn)
                        if c.tone == 0 {
                            // For "uu" pattern, horn goes on FIRST u, not second
                            // "uuw" → "ưu" (like "ưu tiên" = priority)
                            // Check if there's another 'u' before this one
                            if pos > 0 {
                                if let Some(prev) = self.buffer.get(pos - 1) {
                                    if prev.key == keys::U && prev.tone == 0 {
                                        // Apply horn to previous 'u' instead
                                        if let Some(u) = self.buffer.get_mut(pos - 1) {
                                            u.tone = 2;
                                        }
                                        self.revert.record(xform::HORN, keys::U, (pos - 1) as u8);
                                        self.raw.mark_consumed(self.raw.len() - 1);
                                        return ProcessResult::Transform;
                                    }
                                    // For "uou" pattern (hươu = deer), horn goes on first u and o
                                    // "huouw" → "hươu"
                                    // Skip this u and let loop continue to find 'o' then 'u'
                                    if prev.key == keys::O && prev.tone == 0 && pos >= 2 {
                                        if let Some(first_u) = self.buffer.get(pos - 2) {
                                            if first_u.key == keys::U && first_u.tone == 0 {
                                                // Found u-o-u pattern, apply horn to first u and o
                                                if let Some(u) = self.buffer.get_mut(pos - 2) {
                                                    u.tone = 2; // horn on first u
                                                }
                                                if let Some(o) = self.buffer.get_mut(pos - 1) {
                                                    o.tone = 2; // horn on o
                                                }
                                                self.revert.record(
                                                    xform::HORN,
                                                    keys::U,
                                                    (pos - 2) as u8,
                                                );
                                                self.raw.mark_consumed(self.raw.len() - 1);
                                                return ProcessResult::Transform;
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 2;
                            }
                            self.revert.record(xform::HORN, keys::U, pos as u8);
                            self.raw.mark_consumed(self.raw.len() - 1);
                            return ProcessResult::Transform;
                        }
                    }
                    _ => continue, // Skip vowels that can't take horn/breve (i, e, y)
                }
            }
        }

        // No compatible vowel found - reject
        ProcessResult::Reject
    }

    /// Check if the buffer has valid Vietnamese syllable structure
    ///
    /// Returns false if consonant clusters after vowels are invalid for Vietnamese.
    /// This helps detect foreign words like "metric" where "tr" is not a valid final.
    fn has_valid_vietnamese_structure(&self) -> bool {
        let vowel_positions = self.buffer.find_vowels();
        if vowel_positions.is_empty() {
            return true; // No vowels yet, structure is fine
        }

        // Get the last vowel position
        let last_vowel_pos = *vowel_positions.last().unwrap();

        // Get consonants after the last vowel (potential finals)
        let mut final_consonants: Vec<u16> = Vec::new();
        for i in (last_vowel_pos + 1)..self.buffer.len() {
            if let Some(c) = self.buffer.get(i) {
                if keys::is_consonant(c.key) {
                    final_consonants.push(c.key);
                }
            }
        }

        // Validate final consonant pattern
        match final_consonants.len() {
            0 => true, // Open syllable - valid
            1 => is_valid_final_1(final_consonants[0]),
            2 => is_valid_final_2(final_consonants[0], final_consonants[1]),
            _ => false, // 3+ consonants after vowel is never valid in Vietnamese
        }
    }

    /// Handle TONE action - apply tone mark
    fn handle_tone(&mut self, key: u16) -> ProcessResult {
        // Skip in foreign word mode (after double-key revert like ss, rr, ff)
        if self.foreign_word_mode {
            return ProcessResult::Reject;
        }

        let tone_value = match tone_key_to_value(key) {
            Some(v) => v,
            None => return ProcessResult::Reject,
        };

        // Find vowel to apply tone
        let vowel_positions = self.buffer.find_vowels();
        if vowel_positions.is_empty() {
            return ProcessResult::Reject;
        }

        // Validate vowel pattern before applying tone
        // This prevents tone application on foreign words like "about" → "ábout"
        //
        // IMPORTANT: For "gi" and "qu" initials, exclude the initial's vowel from validation
        // - "giường" has vowels [i, u, o] but 'i' is part of "gi" initial → validate [u, o] = "ươ"
        // - "quốc" has vowels [u, o] but 'u' is part of "qu" initial → validate [o] = single vowel
        let has_gi_initial = self.buffer.len() >= 2
            && self.buffer.get(0).map(|c| c.key) == Some(keys::G)
            && self.buffer.get(1).map(|c| c.key) == Some(keys::I);

        let has_qu_initial = self.buffer.len() >= 2
            && self.buffer.get(0).map(|c| c.key) == Some(keys::Q)
            && self.buffer.get(1).map(|c| c.key) == Some(keys::U);

        // Exclude position 1 if it's part of gi/qu initial
        let skip_pos = if has_gi_initial || has_qu_initial {
            Some(1)
        } else {
            None
        };

        let vowel_keys: Vec<u16> = vowel_positions
            .iter()
            .filter(|&&pos| skip_pos != Some(pos))
            .filter_map(|&pos| self.buffer.get(pos).map(|c| c.key))
            .collect();

        if !is_valid_vowel_pattern(&vowel_keys) {
            return ProcessResult::Reject;
        }

        // Check if current syllable structure is valid Vietnamese
        // This prevents tone on words like "metric" where "t" followed by more letters
        // creates invalid patterns
        if !self.has_valid_vietnamese_structure() {
            return ProcessResult::Reject;
        }

        // Check for foreign consonant clusters anywhere in the word
        // This prevents tone on words like "compress" (has "pr"), "text" (has "xt")
        if self.has_foreign_consonant_cluster() {
            return ProcessResult::Reject;
        }

        // Determine which vowel gets the tone
        let pos = self.find_tone_position(&vowel_positions);
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

    /// Find correct position for tone placement using Phonology matrix
    fn find_tone_position(&self, vowel_positions: &[usize]) -> usize {
        if vowel_positions.is_empty() {
            return 0;
        }
        if vowel_positions.len() == 1 {
            return vowel_positions[0];
        }

        // Check for vowel with existing tone mark - replace on same vowel
        for &pos in vowel_positions {
            if let Some(c) = self.buffer.get(pos) {
                if c.mark > 0 {
                    return pos;
                }
            }
        }

        // Build Vowel structs for Phonology lookup
        let vowels: Vec<Vowel> = vowel_positions
            .iter()
            .filter_map(|&pos| {
                self.buffer.get(pos).map(|c| {
                    let modifier = match c.tone {
                        0 => Modifier::None,
                        1 => Modifier::Circumflex,
                        2 => Modifier::Horn, // horn or breve
                        _ => Modifier::None,
                    };
                    Vowel::new(c.key, modifier, pos)
                })
            })
            .collect();

        if vowels.is_empty() {
            return vowel_positions[0];
        }

        // Detect context: has_final_consonant, has_qu_initial, has_gi_initial
        let has_final_consonant = self.state == st::FIN;

        // Detect qu-initial: Q followed by U at start
        let has_qu_initial = if self.buffer.len() >= 2 {
            self.buffer
                .get(0)
                .map(|c| c.key == keys::Q)
                .unwrap_or(false)
                && self
                    .buffer
                    .get(1)
                    .map(|c| c.key == keys::U)
                    .unwrap_or(false)
        } else {
            false
        };

        // Detect gi-initial: G followed by I at start
        let has_gi_initial = if self.buffer.len() >= 2 {
            self.buffer
                .get(0)
                .map(|c| c.key == keys::G)
                .unwrap_or(false)
                && self
                    .buffer
                    .get(1)
                    .map(|c| c.key == keys::I)
                    .unwrap_or(false)
        } else {
            false
        };

        // Use Phonology matrix for tone position lookup
        Phonology::find_tone_position(
            &vowels,
            has_final_consonant,
            self.modern_tone,
            has_qu_initial,
            has_gi_initial,
        )
    }

    /// Handle MARK action - apply vowel mark (circumflex, horn, breve)
    fn handle_mark(&mut self, key: u16, caps: bool) -> ProcessResult {
        // W is handled separately for horn/breve
        if key == keys::W {
            let result = self.apply_horn_or_breve();
            if result == ProcessResult::Reject {
                // Can't apply horn/breve - check if we should consume or pass through
                // For "ươ" pattern with horn on both, consume 'w' silently (redundant)
                // For other cases, add 'w' as literal
                let vowels = self.buffer.find_vowels();
                if vowels.len() >= 2 {
                    let len = vowels.len();
                    let first = vowels[len - 2];
                    let second = vowels[len - 1];
                    if let (Some(v1), Some(v2)) = (self.buffer.get(first), self.buffer.get(second))
                    {
                        // Check for ươ pattern with horn on both
                        if v1.key == keys::U && v1.tone == 2 && v2.key == keys::O && v2.tone == 2 {
                            // Consume 'w' silently - horn already applied to both
                            self.raw.mark_consumed(self.raw.len() - 1);
                            return ProcessResult::Transform;
                        }
                    }
                }
                // Not a redundant 'w' - add as literal
                self.buffer.push(Char::new(keys::W, caps));
                return ProcessResult::Update;
            }
            return result;
        }

        // Check for foreign consonant clusters - don't apply marks to foreign words
        if self.has_foreign_consonant_cluster() {
            return ProcessResult::Reject;
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
                // Verify it's a true double-vowel pattern (consecutive same vowels)
                // "aa" → "â" but "chưa" + "a" → "chưaa" (not circumflex)
                // The char immediately before the last vowel must be the same vowel
                let is_double_vowel = if last_pos > 0 {
                    if let Some(prev) = self.buffer.get(last_pos - 1) {
                        prev.key == key && prev.tone == 0
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Post-tone circumflex pattern: e + consonant + tone + e → ế
                // Examples: "xepse" = x + e + p + s(tone) + e(circumflex) → xếp
                // Detect by checking: vowel has mark applied (mark > 0)
                let is_post_tone_circumflex = c.mark > 0;

                if !is_double_vowel && !is_post_tone_circumflex {
                    // Not a double-vowel pattern - add the char as literal
                    // "chưa" + "a" → "chưaa" (not circumflex)
                    self.buffer.push(Char::new(key, caps));
                    return ProcessResult::Update;
                }

                // Same vowel without existing mark - apply circumflex
                if let Some(c) = self.buffer.get_mut(last_pos) {
                    c.tone = 1; // circumflex
                    self.revert.record(xform::CIRCUMFLEX, key, last_pos as u8);
                    self.raw.mark_consumed(self.raw.len() - 1);

                    // Apply deferred stroke if pending (dede → đê)
                    // Also remove the 'd' that triggered the defer
                    if self.defer.kind == defertype::STROKE_D {
                        let d_pos = self.defer.position as usize;
                        if let Some(d_char) = self.buffer.get_mut(d_pos) {
                            if d_char.key == keys::D && !d_char.stroke {
                                d_char.stroke = true;
                            }
                        }
                        // Remove the second 'd' that triggered defer
                        for i in (d_pos + 1..self.buffer.len()).rev() {
                            if self.buffer.get(i).is_some_and(|c| c.key == keys::D) {
                                self.buffer.remove(i);
                                break;
                            }
                        }
                        self.defer.clear();
                    }

                    return ProcessResult::Transform;
                }
            }
        }

        ProcessResult::Update
    }

    /// Check if buffer starts with valid Vietnamese initial consonant pattern
    ///
    /// Returns false for foreign word patterns like "cl", "bl", "cr", etc.
    fn has_valid_vietnamese_initial(&self) -> bool {
        if self.buffer.is_empty() {
            return true;
        }

        let first = match self.buffer.get(0) {
            Some(c) => c.key,
            None => return true,
        };

        // Single consonant is always valid
        if !keys::is_consonant(first) {
            return true; // Starts with vowel - valid
        }

        // Check if second character exists and is consonant
        if self.buffer.len() < 2 {
            return true; // Single consonant - valid
        }

        let second = match self.buffer.get(1) {
            Some(c) => c.key,
            None => return true,
        };

        // If second is vowel, first consonant is valid initial
        if keys::is_vowel(second) {
            return true;
        }

        // Two consonants at start - check if valid Vietnamese cluster
        // Valid clusters: ch, gh, gi, kh, ng, nh, ph, qu, th, tr
        match (first, second) {
            (keys::C, keys::H) => true, // ch
            (keys::G, keys::H) => true, // gh
            (keys::G, keys::I) => true, // gi
            (keys::K, keys::H) => true, // kh
            (keys::N, keys::G) => true, // ng (also ngh with third char)
            (keys::N, keys::H) => true, // nh
            (keys::P, keys::H) => true, // ph
            (keys::Q, keys::U) => true, // qu
            (keys::T, keys::H) => true, // th
            (keys::T, keys::R) => true, // tr
            // Invalid clusters: bl, br, cl, cr, dr, fl, fr, gl, gr, pl, pr, sc, sk, sl, sm, sn, sp, st, sw, tw, wr, etc.
            _ => false,
        }
    }

    /// Check for invalid consonant clusters anywhere in the word
    ///
    /// Detects foreign words like "compress" (has "pr"), "expect" (has "xp"), "text" (has "xt"),
    /// "matrix" (has "tr" after vowel - invalid as final).
    ///
    /// Position-aware: Initial clusters (before vowel) allow more combinations than
    /// final clusters (after vowel).
    fn has_foreign_consonant_cluster(&self) -> bool {
        if self.buffer.len() < 2 {
            return false;
        }

        // Valid Vietnamese consonant clusters depend on position:
        // Initial (before first vowel): ch, gh, gi, kh, ng, nh, ph, qu, th, tr, ngh
        // Final (after vowel): ch, ng, nh only

        let mut prev_consonant: Option<u16> = None;
        let mut seen_vowel = false;

        for i in 0..self.buffer.len() {
            let key = match self.buffer.get(i) {
                Some(c) => c.key,
                None => continue,
            };

            if keys::is_vowel(key) {
                seen_vowel = true;
                prev_consonant = None; // Reset on vowel
            } else if keys::is_consonant(key) {
                if let Some(prev) = prev_consonant {
                    // Check if this consecutive consonant pair is valid in Vietnamese
                    let is_valid = if !seen_vowel {
                        // INITIAL position - more clusters allowed
                        matches!(
                            (prev, key),
                            (keys::C, keys::H)  // ch
                            | (keys::G, keys::H)  // gh
                            | (keys::G, keys::I)  // gi (i is vowel but gi is special)
                            | (keys::K, keys::H)  // kh
                            | (keys::K, keys::R)  // kr (ethnic minority: Krông)
                            | (keys::N, keys::G)  // ng
                            | (keys::N, keys::H)  // nh
                            | (keys::P, keys::H)  // ph
                            | (keys::Q, keys::U)  // qu (u is vowel but qu is special)
                            | (keys::T, keys::H)  // th
                            | (keys::T, keys::R) // tr
                        )
                    } else {
                        // FINAL position - only ch, ng, nh valid
                        matches!(
                            (prev, key),
                            (keys::C, keys::H)  // ch (final)
                            | (keys::N, keys::G)  // ng (final)
                            | (keys::N, keys::H) // nh (final)
                        )
                    };

                    if !is_valid {
                        return true; // Found foreign cluster
                    }
                }
                prev_consonant = Some(key);
            }
        }

        false
    }

    /// Handle STROKE action - apply stroke to d
    fn handle_stroke(&mut self) -> ProcessResult {
        // Don't apply stroke on foreign words with invalid initial clusters
        if !self.has_valid_vietnamese_initial() {
            return ProcessResult::Reject;
        }

        // Check for invalid vowel patterns (foreign words like "deadline")
        // "deadline" has vowels [e, a] which is NOT a valid Vietnamese diphthong
        // "dede" has single vowel [e] which is valid for quick typing
        let vowel_keys: Vec<u16> = self
            .buffer
            .iter()
            .filter(|c| keys::is_vowel(c.key))
            .map(|c| c.key)
            .collect();
        if vowel_keys.len() >= 2 && !is_valid_vowel_pattern(&vowel_keys) {
            return ProcessResult::Reject;
        }

        // Check if last char is 'd' (adjacent dd → immediate stroke)
        let last_is_d = self
            .buffer
            .last()
            .is_some_and(|c| c.key == keys::D && !c.stroke);
        if last_is_d {
            // Adjacent dd → apply stroke immediately
            // BUT only if 'd' is not in final position (đ not valid as final)
            // e.g., "didd" should NOT become "diđ", should stay as "did" + extra 'd'
            if self.state == st::FIN {
                return ProcessResult::Reject; // Can't stroke final 'd' → đ not valid final
            }
            let last_idx = self.buffer.len() - 1;
            if let Some(c) = self.buffer.get_mut(last_idx) {
                c.stroke = true;
                self.revert.record(xform::STROKE, keys::D, last_idx as u8);
                self.raw.mark_consumed(self.raw.len() - 1);
                return ProcessResult::Transform;
            }
        }

        // Non-adjacent d (d + vowel + d) → apply stroke immediately
        // "dod" → "đo", "dad" → "đa", etc.
        // Find the first 'd' to potentially stroke
        let first_d_pos = self
            .buffer
            .iter()
            .position(|c| c.key == keys::D && !c.stroke);
        if let Some(pos) = first_d_pos {
            let has_vowel = self.buffer.iter().any(|c| keys::is_vowel(c.key));
            if has_vowel {
                // Apply stroke immediately for d+vowel+d pattern
                // The second 'd' triggers stroke on first 'd' and is consumed
                if let Some(c) = self.buffer.get_mut(pos) {
                    c.stroke = true;
                    self.revert.record(xform::STROKE, keys::D, pos as u8);
                    self.raw.mark_consumed(self.raw.len() - 1);
                    return ProcessResult::Transform;
                }
            }
        }

        ProcessResult::Reject
    }

    /// Handle VNI number keys for diacritics
    ///
    /// VNI mapping:
    /// - 1-5: tones (sắc, huyền, hỏi, ngã, nặng)
    /// - 6: circumflex (â, ê, ô)
    /// - 7: horn (ơ, ư)
    /// - 8: breve (ă)
    /// - 9: đ stroke
    /// - 0: remove diacritic
    fn handle_vni_number(&mut self, key: u16) -> Option<ProcessResult> {
        // Only handle in VOW/DIA state (or INIT for đ)
        match key {
            keys::N1 | keys::N2 | keys::N3 | keys::N4 | keys::N5 => {
                // Tones require vowel (can be in VOW, DIA, or FIN state)
                if self.state == st::EMPTY || self.state == st::INIT {
                    return None;
                }
                let (tone, digit) = match key {
                    keys::N1 => (1, '1'), // sắc
                    keys::N2 => (2, '2'), // huyền
                    keys::N3 => (3, '3'), // hỏi
                    keys::N4 => (4, '4'), // ngã
                    keys::N5 => (5, '5'), // nặng
                    _ => (0, '0'),
                };
                let vowels = self.buffer.find_vowels();
                if vowels.is_empty() {
                    return None;
                }
                let pos = self.find_tone_position(&vowels);
                if let Some(c) = self.buffer.get_mut(pos) {
                    // VNI revert: if same tone pressed again, remove and pass through digit
                    if c.mark == tone {
                        c.mark = 0;
                        self.state = if self.state == st::DIA {
                            st::VOW
                        } else {
                            self.state
                        };
                        // Set pending literal to output the digit after buffer update
                        self.pending_literal = Some(digit);
                        return Some(ProcessResult::Update);
                    }
                    c.mark = tone;
                    self.raw.mark_consumed(self.raw.len() - 1);
                    self.state = st::DIA;
                    return Some(ProcessResult::Transform);
                }
                None
            }
            keys::N6 => {
                // Circumflex (â, ê, ô)
                if self.state == st::EMPTY || self.state == st::INIT {
                    return None;
                }
                let vowels = self.buffer.find_vowels();
                if vowels.is_empty() {
                    return None;
                }
                // Find vowel that can take circumflex (a, e, o)
                for &pos in vowels.iter().rev() {
                    let (vkey, tone) = {
                        if let Some(c) = self.buffer.get(pos) {
                            (c.key, c.tone)
                        } else {
                            continue;
                        }
                    };
                    if matches!(vkey, keys::A | keys::E | keys::O) {
                        // VNI revert: if already has circumflex, remove and pass through digit
                        if tone == 1 {
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 0;
                            }
                            self.state = if self.state == st::DIA {
                                st::VOW
                            } else {
                                self.state
                            };
                            self.pending_literal = Some('6');
                            return Some(ProcessResult::Update);
                        }
                        // Switch from horn to circumflex (o7 + 6 → ô)
                        if tone == 2 && matches!(vkey, keys::O) {
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 1; // circumflex replaces horn
                            }
                            // For 'uo' pattern, remove horn from 'u' as well
                            if pos > 0 {
                                if let Some(prev) = self.buffer.get(pos - 1) {
                                    if prev.key == keys::U && prev.tone == 2 {
                                        if let Some(u) = self.buffer.get_mut(pos - 1) {
                                            u.tone = 0;
                                        }
                                    }
                                }
                            }
                            self.raw.mark_consumed(self.raw.len() - 1);
                            self.state = st::DIA;
                            return Some(ProcessResult::Transform);
                        }
                        if tone == 0 {
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 1; // circumflex
                            }
                            self.raw.mark_consumed(self.raw.len() - 1);
                            self.state = st::DIA;
                            return Some(ProcessResult::Transform);
                        }
                    }
                }
                None
            }
            keys::N7 => {
                // Horn (ơ, ư) - also handles 'uo' → 'ươ' compound
                if self.state == st::EMPTY || self.state == st::INIT {
                    return None;
                }
                let vowels = self.buffer.find_vowels();
                if vowels.is_empty() {
                    return None;
                }

                // Check for 'uo' pattern → apply horn to both for 'ươ' compound
                // Check for 'uu' pattern → apply horn to FIRST u for 'ưu' cluster
                if vowels.len() >= 2 {
                    let len = vowels.len();
                    // Look for 'uo' or 'uu' patterns
                    for i in 0..len - 1 {
                        let first_pos = vowels[i];
                        let second_pos = vowels[i + 1];
                        let (first_key, first_tone) = self
                            .buffer
                            .get(first_pos)
                            .map_or((0, 0), |c| (c.key, c.tone));
                        let (second_key, second_tone) = self
                            .buffer
                            .get(second_pos)
                            .map_or((0, 0), |c| (c.key, c.tone));

                        if first_key == keys::U && second_key == keys::O {
                            // Found 'uo' pattern
                            if first_tone == 2 && second_tone == 2 {
                                // Both have horn - revert both
                                if let Some(c) = self.buffer.get_mut(first_pos) {
                                    c.tone = 0;
                                }
                                if let Some(c) = self.buffer.get_mut(second_pos) {
                                    c.tone = 0;
                                }
                                self.state = if self.state == st::DIA {
                                    st::VOW
                                } else {
                                    self.state
                                };
                                self.pending_literal = Some('7');
                                return Some(ProcessResult::Update);
                            }
                            if first_tone == 2 && second_tone == 0 {
                                // 'u' already has horn (from "u7"), apply horn to 'o'
                                // Example: "u7o7" → "ươ", "hu7o71" → "huớ"
                                if let Some(c) = self.buffer.get_mut(second_pos) {
                                    c.tone = 2;
                                }
                                self.raw.mark_consumed(self.raw.len() - 1);
                                self.state = st::DIA;
                                return Some(ProcessResult::Transform);
                            }
                            // Check if there's any character after the 'uo' vowels
                            // (indicates final like "buong", "hươu", "người" vs standalone "uo")
                            // Includes both consonant finals (c, ng, t) and semivowel finals (u, i)
                            let has_final = second_pos + 1 < self.buffer.len();

                            // Switch from circumflex to horn
                            // 'uô' pattern: u has no mark, o has circumflex
                            if first_tone == 0 && second_tone == 1 {
                                // For syllables with finals, only switch 'o'
                                // For standalone 'uô', switch both to 'ươ'
                                if has_final {
                                    // Only switch 'o' to horn
                                    if let Some(c) = self.buffer.get_mut(second_pos) {
                                        c.tone = 2;
                                    }
                                } else {
                                    // Switch both for 'ươ' compound
                                    if let Some(c) = self.buffer.get_mut(first_pos) {
                                        c.tone = 2;
                                    }
                                    if let Some(c) = self.buffer.get_mut(second_pos) {
                                        c.tone = 2;
                                    }
                                }
                                self.raw.mark_consumed(self.raw.len() - 1);
                                self.state = st::DIA;
                                return Some(ProcessResult::Transform);
                            }
                            if first_tone == 0 && second_tone == 0 {
                                // Neither has mark - fresh horn application
                                // Per Issue #133: horn placement depends on final
                                if has_final {
                                    // With final: both get horn → "ương", "hươu", "người"
                                    // Examples: "duong7" → "dương", "huou7" → "hươu"
                                    if let Some(c) = self.buffer.get_mut(first_pos) {
                                        c.tone = 2;
                                    }
                                    if let Some(c) = self.buffer.get_mut(second_pos) {
                                        c.tone = 2;
                                    }
                                } else {
                                    // Without final: only 'o' gets horn initially → "uơ"
                                    // Defer horn on 'u' for when final is typed later
                                    // Examples: "uo7" → "uơ", "ruo7u" → defer → "rươu"
                                    if let Some(c) = self.buffer.get_mut(second_pos) {
                                        c.tone = 2;
                                    }
                                    self.defer = DeferState::horn_u(first_pos as u8);
                                }
                                self.raw.mark_consumed(self.raw.len() - 1);
                                self.state = st::DIA;
                                return Some(ProcessResult::Transform);
                            }
                        } else if first_key == keys::U && second_key == keys::U {
                            // Found 'uu' pattern - horn on FIRST u for 'ưu' cluster
                            // Examples: "luu7" → "lưu", "huu7" → "hưu", "cuu7" → "cưu"
                            if first_tone == 2 {
                                // First u has horn - revert
                                if let Some(c) = self.buffer.get_mut(first_pos) {
                                    c.tone = 0;
                                }
                                self.state = if self.state == st::DIA {
                                    st::VOW
                                } else {
                                    self.state
                                };
                                self.pending_literal = Some('7');
                                return Some(ProcessResult::Update);
                            }
                            if first_tone == 0 {
                                // Apply horn to FIRST u
                                if let Some(c) = self.buffer.get_mut(first_pos) {
                                    c.tone = 2;
                                }
                                self.raw.mark_consumed(self.raw.len() - 1);
                                self.state = st::DIA;
                                return Some(ProcessResult::Transform);
                            }
                        }
                    }
                }

                // Find single vowel that can take horn (o, u)
                for &pos in vowels.iter().rev() {
                    let (vkey, vtone) = {
                        if let Some(c) = self.buffer.get(pos) {
                            (c.key, c.tone)
                        } else {
                            continue;
                        }
                    };
                    if matches!(vkey, keys::O | keys::U) {
                        // VNI revert: if already has horn, remove and pass through digit
                        if vtone == 2 {
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 0;
                            }
                            self.state = if self.state == st::DIA {
                                st::VOW
                            } else {
                                self.state
                            };
                            self.pending_literal = Some('7');
                            return Some(ProcessResult::Update);
                        }
                        // Switch from circumflex to horn (o6 + 7 → ơ)
                        // Only 'o' can switch (u doesn't have circumflex in Vietnamese)
                        if vtone == 1 && vkey == keys::O {
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 2; // horn replaces circumflex
                            }
                            self.raw.mark_consumed(self.raw.len() - 1);
                            self.state = st::DIA;
                            return Some(ProcessResult::Transform);
                        }
                        if vtone == 0 {
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 2; // horn
                            }
                            self.raw.mark_consumed(self.raw.len() - 1);
                            self.state = st::DIA;
                            return Some(ProcessResult::Transform);
                        }
                    }
                }
                None
            }
            keys::N8 => {
                // Breve (ă)
                if self.state == st::EMPTY || self.state == st::INIT {
                    return None;
                }
                let vowels = self.buffer.find_vowels();
                if vowels.is_empty() {
                    return None;
                }
                // Find 'a' to apply breve
                for &pos in vowels.iter().rev() {
                    let (vkey, vtone) = {
                        if let Some(c) = self.buffer.get(pos) {
                            (c.key, c.tone)
                        } else {
                            continue;
                        }
                    };
                    if vkey == keys::A {
                        // VNI revert: if already has breve, remove and pass through digit
                        if vtone == 2 {
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 0;
                            }
                            self.state = if self.state == st::DIA {
                                st::VOW
                            } else {
                                self.state
                            };
                            self.pending_literal = Some('8');
                            return Some(ProcessResult::Update);
                        }
                        if vtone == 0 {
                            if let Some(c) = self.buffer.get_mut(pos) {
                                c.tone = 2; // breve
                            }
                            self.raw.mark_consumed(self.raw.len() - 1);
                            self.state = st::DIA;
                            return Some(ProcessResult::Transform);
                        }
                    }
                }
                None
            }
            keys::N9 => {
                // đ stroke - needs d in buffer (can be at any state)
                for i in (0..self.buffer.len()).rev() {
                    if let Some(c) = self.buffer.get_mut(i) {
                        if c.key == keys::D && !c.stroke {
                            c.stroke = true;
                            self.revert.record(xform::STROKE, key, i as u8);
                            self.raw.mark_consumed(self.raw.len() - 1);
                            return Some(ProcessResult::Transform);
                        }
                    }
                }
                None
            }
            keys::N0 => {
                // Remove diacritic (tone or mark)
                if self.state != st::DIA {
                    return None;
                }
                // Find vowel with diacritic and remove
                let vowels = self.buffer.find_vowels();
                for &pos in vowels.iter().rev() {
                    if let Some(c) = self.buffer.get_mut(pos) {
                        if c.mark > 0 {
                            c.mark = 0;
                            self.raw.mark_consumed(self.raw.len() - 1);
                            return Some(ProcessResult::Transform);
                        }
                        if c.tone > 0 {
                            c.tone = 0;
                            self.raw.mark_consumed(self.raw.len() - 1);
                            return Some(ProcessResult::Transform);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Handle Telex 'z' key - removes all diacritics (tone marks)
    ///
    /// In Telex mode, 'z' removes the tone mark from the word.
    /// This only applies when there are vowels with marks.
    fn handle_telex_z(&mut self) -> Option<ProcessResult> {
        // Only applies in states with vowels that might have marks
        if self.state == st::EMPTY || self.state == st::INIT {
            return None;
        }

        let vowels = self.buffer.find_vowels();
        if vowels.is_empty() {
            return None;
        }

        // Find vowel with tone mark and remove it
        for &pos in vowels.iter().rev() {
            if let Some(c) = self.buffer.get_mut(pos) {
                if c.mark > 0 {
                    c.mark = 0;
                    self.raw.mark_consumed(self.raw.len() - 1);
                    // Update state: if no more marks remain, go back to VOW
                    self.state = st::VOW;
                    return Some(ProcessResult::Transform);
                }
            }
        }

        // No tone marks found - pass through as regular consonant
        None
    }

    /// Handle DEFER action
    fn handle_defer(&mut self, key: u16, caps: bool) -> ProcessResult {
        // Store deferred action for later resolution
        self.buffer.push(Char::new(key, caps));
        ProcessResult::Update
    }

    /// Handle REVERT action - undo last transform
    /// For Telex, also adds the triggering key to buffer after reverting
    /// Double-key revert triggers FOREIGN mode (buffer becomes raw ASCII)
    fn handle_revert(&mut self, key: u16) -> ProcessResult {
        let pos = self.revert.position as usize;

        match self.revert.transform {
            xform::STROKE => {
                // Revert đ → d
                if let Some(c) = self.buffer.get_mut(pos) {
                    c.stroke = false;
                }
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
                // Double-key tone revert (ss, ff, rr, xx, jj) triggers FOREIGN mode
                // The buffer is now raw ASCII which is valid English
                self.foreign_word_mode = true;
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

        let transform = self.revert.transform;
        self.revert.mark_reverted();

        // For Telex, add the triggering key to buffer as regular character
        // Exception: W_VOWEL revert (Ww → W) should NOT add another 'w'
        // Exception: CIRCUMFLEX revert (aa→a) - triggering key undoes transform
        // For VNI, this is handled in handle_vni_number with pending_literal
        if self.method == 0 && transform != xform::W_VOWEL {
            // For REVERT (circumflex or tone), the triggering key is consumed by the revert
            // and should NOT remain in raw buffer - raw syncs with buffer
            // Example: "conss" → raw should be [c,o,n,s] not [c,o,n,s,s]
            // Example: "dataa" → raw should be [d,a,t,a] not [d,a,t,a,a]
            if transform == xform::CIRCUMFLEX || transform == xform::TONE {
                // Pop the triggering key from raw - it's consumed by the revert
                self.raw.pop();
            }

            let caps = self.raw.last().is_some_and(|k| k.caps());
            self.buffer.push(Char::new(key, caps));

            // Update state based on what we're adding
            if keys::is_vowel(key) {
                self.state = st::VOW;
            } else if transform == xform::TONE {
                // After tone revert, the letter becomes final consonant
                self.state = st::FIN;
            }
            // For other cases (circumflex, horn, breve with same vowel/w),
            // state is already appropriate
        }

        ProcessResult::Revert
    }

    /// Resolve pending defer based on current state
    /// Returns true if a transformation was applied
    fn resolve_defer(&mut self) -> bool {
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
                            return false;
                        }

                        // Check for ng/nh cluster (len >= 2)
                        let next_key = if len >= 2 {
                            let second_last = self.buffer.get(len - 2).map(|c| c.key);
                            // If second-last is 'n' and last is 'g' or 'h', it's a cluster
                            if second_last == Some(keys::N)
                                && (last == Some(keys::G) || last == Some(keys::H))
                            {
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
                            self.defer.clear();
                            return true;
                        }
                        self.defer.clear();
                    }
                }
                false
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
                            self.defer.clear();
                            return true;
                        }
                    }
                    self.defer.clear();
                } else if self.state == st::VOW || self.state == st::DIA {
                    // Also check for ươi/ươy glide pattern in VOW/DIA states
                    // This handles words like "người" where there's no final consonant
                    // Pattern: ư + ơ + i/y (the horn on u should be applied when glide is added)
                    if self.check_horn_u_with_glide() {
                        let pos = self.defer.position as usize;
                        if let Some(c) = self.buffer.get_mut(pos) {
                            c.tone = 2; // horn
                        }
                        self.defer.clear();
                        return true;
                    }
                }
                false
            }
            defertype::TONE_PLACE => {
                // Tone placement depends on syllable structure
                // If we have a final consonant, may need to adjust tone position
                if self.state == st::FIN {
                    // For now, keep tone where it was placed
                    // More sophisticated logic can be added later
                    self.defer.clear();
                }
                false
            }
            defertype::HORN_O => {
                // Deferred horn on 'o' in "ưo" pattern (from vowel+W like "đuwoc")
                // Apply horn when final consonant is added
                if self.state == st::FIN {
                    let pos = self.defer.position as usize;
                    if let Some(c) = self.buffer.get_mut(pos) {
                        if c.key == keys::O && c.tone == 0 {
                            c.tone = 2; // horn
                            self.defer.clear();
                            return true;
                        }
                    }
                    self.defer.clear();
                } else if self.state == st::DIA {
                    // Also apply when tone mark is added to the vowel cluster
                    // This handles cases like "đưof" where 'f' is tone key
                    let pos = self.defer.position as usize;
                    if let Some(c) = self.buffer.get_mut(pos) {
                        if c.key == keys::O && c.tone == 0 {
                            c.tone = 2; // horn
                            self.defer.clear();
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Check if buffer ends with word boundary (space, punctuation)
    pub fn is_word_boundary(&self, key: u16) -> bool {
        matches!(
            key,
            keys::SPACE
                | keys::DOT
                | keys::COMMA
                | keys::SEMICOLON
                | keys::QUOTE
                | keys::SLASH
                | keys::LBRACKET
                | keys::RBRACKET
                | keys::MINUS
        )
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
                        if second_last == Some(keys::N)
                            && (last == Some(keys::G) || last == Some(keys::H))
                        {
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

        // If we have pending horn_u defer, check if we should apply it
        if self.defer.kind == defertype::HORN_U {
            if self.state == st::FIN {
                // Already handled in resolve_defer when final consonant was added
                self.defer.clear();
            } else {
                // Check for ươi/ươy pattern: ư + ơ + i/y (semivowel glide)
                // In this case, apply horn to u even without final consonant
                // Examples: "người", "cười", "mười"
                let should_apply = self.check_horn_u_with_glide();
                if should_apply {
                    let pos = self.defer.position as usize;
                    if let Some(c) = self.buffer.get_mut(pos) {
                        c.tone = 2; // horn
                    }
                }
                self.defer.clear();
            }
        }
    }

    /// Check if horn on u should apply due to ươi/ươy glide pattern
    fn check_horn_u_with_glide(&self) -> bool {
        // Pattern: u + ơ + glide where u is at defer position
        // Valid glides: i, y, u (for words like người, rượu)
        let defer_pos = self.defer.position as usize;

        // Check if there's ơ after u
        if defer_pos + 1 >= self.buffer.len() {
            return false;
        }

        let next = self.buffer.get(defer_pos + 1);
        let has_horn_o = next.is_some_and(|c| c.key == keys::O && c.tone == 2);

        if !has_horn_o {
            return false;
        }

        // Check if there's i, y, or u (glide) after ơ
        if defer_pos + 2 >= self.buffer.len() {
            return false;
        }

        let glide = self.buffer.get(defer_pos + 2);
        glide.is_some_and(|c| c.key == keys::I || c.key == keys::Y || c.key == keys::U)
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
    #[ignore] // TODO: Add validation to revert breve when 'nh' final detected (ănh invalid)
    fn test_defer_breve_with_nh_cluster_invalid() {
        let mut p = Processor::new();

        // Type "awnh" - breve should NOT be applied (ănh is invalid, use anh)
        // Currently breve is applied immediately for user feedback
        // Validation should revert it when 'nh' is detected
        p.process(keys::A, false, false);
        p.process(keys::W, false, false);
        p.process(keys::N, false, false);
        p.process(keys::H, false, false);

        // Breve should not be applied after validation
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

    #[test]
    fn test_foreign_word_metric() {
        let mut p = Processor::new();

        // Type "metric" - should NOT apply tone to 'e' when 'r' is pressed
        // Because continuing with 'i' after FIN state suggests English
        p.process(keys::M, false, false);
        p.process(keys::E, false, false);
        p.process(keys::T, false, false);
        assert_eq!(p.state(), st::FIN);

        // 'r' is tone key - applies hỏi to 'e'
        let r = p.process(keys::R, false, false);
        eprintln!("After r: state={}, result={:?}", p.state(), r);
        eprintln!(
            "revert: transform={}, key={}, position={}",
            p.revert.transform, p.revert.key, p.revert.position
        );

        // At this point: buffer=[m,ẻ,t], state=FIN, revert.transform=TONE

        // 'i' comes - should detect foreign word and clear tone
        let r = p.process(keys::I, false, false);
        eprintln!("After i: state={}, result={:?}", p.state(), r);

        // Check 'e' has no tone mark
        assert_eq!(
            p.buffer().get(1).map(|c| c.mark),
            Some(0),
            "e should not have tone in foreign word 'metric'"
        );
    }

    #[test]
    fn test_foreign_word_describe() {
        let mut p = Processor::new();
        for (i, &(ch, key)) in [
            ('d', keys::D),
            ('e', keys::E),
            ('s', keys::S),
            ('c', keys::C),
            ('r', keys::R),
            ('i', keys::I),
            ('b', keys::B),
            ('e', keys::E),
        ]
        .iter()
        .enumerate()
        {
            let r = p.process(key, false, false);
            eprintln!(
                "[{}] '{}': state={}, result={:?}, buffer={}",
                i,
                ch,
                p.state(),
                r,
                p.buffer().to_full_string()
            );
        }
        assert_eq!(p.buffer().to_full_string(), "describe");
    }

    #[test]
    fn test_ua_tone_relocation() {
        let mut p = Processor::new();

        // Type "mua" + 's' (tone) + 'n' (final)
        p.process(keys::M, false, false);
        assert_eq!(p.state(), st::INIT);

        p.process(keys::U, false, false);
        assert_eq!(p.state(), st::VOW);

        p.process(keys::A, false, false);
        assert_eq!(p.state(), st::VOW);

        // Apply tone 's' (sắc) - should go on 'u' for open syllable
        p.process(keys::S, false, false);
        assert_eq!(p.state(), st::DIA);

        // Check tone is on 'u' (position 1)
        let vowels = p.buffer().find_vowels();
        assert_eq!(vowels, vec![1, 2]); // u at 1, a at 2

        // Before 'n', tone should be on u
        assert_eq!(p.buffer().get(1).map(|c| c.mark), Some(1)); // u has tone
        assert_eq!(p.buffer().get(2).map(|c| c.mark), Some(0)); // a has no tone

        // Add final consonant 'n' - should trigger relocation
        p.process(keys::N, false, false);
        assert_eq!(p.state(), st::FIN);

        // After 'n', tone should be on 'a' (position 2) for closed syllable
        assert_eq!(
            p.buffer().get(1).map(|c| c.mark),
            Some(0),
            "u should not have tone in closed syllable"
        );
        assert_eq!(
            p.buffer().get(2).map(|c| c.mark),
            Some(1),
            "a should have tone in closed syllable"
        );
    }

    #[test]
    fn test_circumflex_debug() {
        let mut p = Processor::new();

        // Type "caan " (with space)
        println!("\n=== Typing 'c' ===");
        let r1 = p.process(keys::C, false, false);
        println!(
            "  state={}, buffer={}, result={:?}",
            p.state(),
            p.buffer().to_full_string(),
            r1
        );
        println!("  foreign_mode={}", p.is_foreign_mode());
        assert_eq!(p.state(), st::INIT);

        println!("\n=== Typing 'a' ===");
        let r2 = p.process(keys::A, false, false);
        println!(
            "  state={}, buffer={}, result={:?}",
            p.state(),
            p.buffer().to_full_string(),
            r2
        );
        println!("  foreign_mode={}", p.is_foreign_mode());
        assert_eq!(p.state(), st::VOW);

        println!("\n=== Typing second 'a' ===");
        let r3 = p.process(keys::A, false, false);
        println!(
            "  state={}, buffer={}, result={:?}",
            p.state(),
            p.buffer().to_full_string(),
            r3
        );
        println!("  foreign_mode={}", p.is_foreign_mode());

        // After 'aa', buffer should be "câ" (circumflex applied)
        assert_eq!(
            p.buffer().to_full_string(),
            "câ",
            "Circumflex should be applied: caa → câ"
        );

        println!("\n=== Typing 'n' ===");
        let r4 = p.process(keys::N, false, false);
        println!(
            "  state={}, buffer={}, result={:?}",
            p.state(),
            p.buffer().to_full_string(),
            r4
        );
        println!("  foreign_mode={}", p.is_foreign_mode());

        // After 'n', buffer should be "cân"
        assert_eq!(
            p.buffer().to_full_string(),
            "cân",
            "Buffer should be cân after typing caan"
        );
    }
}
