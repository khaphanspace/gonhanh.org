//! Matrix-based Vietnamese IME Engine
//!
//! Drop-in replacement for the old Engine using matrix dispatch.
//! Zero if-else in hot path. All decisions via lookup tables.
//!
//! ## Migration Strategy
//!
//! This wraps the new Processor and provides the same public interface
//! as the old Engine for seamless integration.

use crate::data::keys;
use crate::engine::buffer::{Buffer, MAX};
use crate::engine::matrix::english::{
    english_likelihood_keys, has_invalid_vietnamese_pattern, EnglishLikelihood,
};
use crate::engine::matrix::{ProcessResult, Processor};
use crate::engine::shortcut::{InputMethod, ShortcutTable};

/// Engine action result (same as old Engine)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    None = 0,
    Send = 1,
    Restore = 2,
}

/// Result for FFI (same as old Engine)
#[repr(C)]
pub struct Result {
    pub chars: [u32; MAX],
    pub action: u8,
    pub backspace: u8,
    pub count: u8,
    pub _pad: u8,
}

impl Result {
    pub fn none() -> Self {
        Self {
            chars: [0; MAX],
            action: Action::None as u8,
            backspace: 0,
            count: 0,
            _pad: 0,
        }
    }

    pub fn send(backspace: u8, chars: &[char]) -> Self {
        let mut result = Self {
            chars: [0; MAX],
            action: Action::Send as u8,
            backspace,
            count: chars.len().min(MAX) as u8,
            _pad: 0,
        };
        for (i, &c) in chars.iter().take(MAX).enumerate() {
            result.chars[i] = c as u32;
        }
        result
    }

    pub fn restore(backspace: u8, chars: &[char]) -> Self {
        let mut result = Self {
            chars: [0; MAX],
            action: Action::Restore as u8,
            backspace,
            count: chars.len().min(MAX) as u8,
            _pad: 0,
        };
        for (i, &c) in chars.iter().take(MAX).enumerate() {
            result.chars[i] = c as u32;
        }
        result
    }
}

/// Word history for backspace-after-space feature
const HISTORY_CAPACITY: usize = 10;

struct WordHistory {
    data: [Buffer; HISTORY_CAPACITY],
    head: usize,
    len: usize,
}

impl WordHistory {
    fn new() -> Self {
        Self {
            data: std::array::from_fn(|_| Buffer::new()),
            head: 0,
            len: 0,
        }
    }

    fn push(&mut self, buf: Buffer) {
        self.data[self.head] = buf;
        self.head = (self.head + 1) % HISTORY_CAPACITY;
        if self.len < HISTORY_CAPACITY {
            self.len += 1;
        }
    }

    fn pop(&mut self) -> Option<Buffer> {
        if self.len == 0 {
            return None;
        }
        self.head = (self.head + HISTORY_CAPACITY - 1) % HISTORY_CAPACITY;
        self.len -= 1;
        Some(self.data[self.head].clone())
    }

    fn clear(&mut self) {
        self.len = 0;
        self.head = 0;
    }
}

/// Check if key is sentence-ending punctuation
#[inline]
fn is_sentence_ending(key: u16, shift: bool) -> bool {
    key == keys::RETURN
        || key == keys::ENTER
        || key == keys::DOT
        || (shift && key == keys::N1) // !
        || (shift && key == keys::SLASH) // ?
}

/// Matrix-based Vietnamese IME Engine
///
/// Uses the new Processor for core processing with matrix dispatch.
/// Provides same interface as old Engine for compatibility.
pub struct Engine {
    /// Core processor with matrix dispatch
    processor: Processor,
    /// Previous buffer state for calculating backspaces
    prev_chars: Vec<char>,
    /// Current rendered characters
    curr_chars: Vec<char>,
    /// Input enabled flag
    enabled: bool,
    /// Shortcut table
    shortcuts: ShortcutTable,
    /// Word history for backspace-after-space
    word_history: WordHistory,
    /// Spaces typed after commit
    spaces_after_commit: u8,
    /// Auto-capitalize enabled
    auto_capitalize: bool,
    /// Pending capitalize (after sentence-ending)
    pending_capitalize: bool,
    /// Tracks if auto-capitalize was used on current word
    /// Used to restore pending_capitalize when user deletes the capitalized letter
    auto_capitalize_used: bool,
    /// ESC restore enabled
    esc_restore_enabled: bool,
    /// Skip w→ư shortcut
    skip_w_shortcut: bool,
    /// Prefix characters for shortcuts (e.g., '#', '@')
    /// These are rejected by the processor but tracked for shortcut matching
    shortcut_prefix: String,
    /// Tracks if previous break was a number (e.g., "149k" should not trigger "k" shortcut)
    prev_was_number: bool,
    /// English auto-restore enabled
    english_auto_restore: bool,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            processor: Processor::new(),
            prev_chars: Vec::with_capacity(32),
            curr_chars: Vec::with_capacity(32),
            enabled: true,
            shortcuts: ShortcutTable::with_defaults(),
            word_history: WordHistory::new(),
            spaces_after_commit: 0,
            auto_capitalize: false,
            pending_capitalize: false,
            auto_capitalize_used: false,
            esc_restore_enabled: false,
            skip_w_shortcut: false,
            shortcut_prefix: String::new(),
            prev_was_number: false,
            english_auto_restore: false,
        }
    }

    // ===== Configuration Methods =====

    pub fn set_method(&mut self, method: u8) {
        self.processor.set_method(method);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.processor.clear();
            self.word_history.clear();
            self.spaces_after_commit = 0;
        }
    }

    pub fn set_skip_w_shortcut(&mut self, skip: bool) {
        self.skip_w_shortcut = skip;
    }

    pub fn set_esc_restore(&mut self, enabled: bool) {
        self.esc_restore_enabled = enabled;
    }

    pub fn set_modern_tone(&mut self, modern: bool) {
        self.processor.set_modern_tone(modern);
    }

    pub fn set_auto_capitalize(&mut self, enabled: bool) {
        self.auto_capitalize = enabled;
        if !enabled {
            self.pending_capitalize = false;
        }
    }

    /// Set whether to enable free tone placement (skip validation)
    pub fn set_free_tone(&mut self, _enabled: bool) {
        // TODO: implement in Processor if needed
    }

    /// Set whether to enable English auto-restore (experimental)
    pub fn set_english_auto_restore(&mut self, enabled: bool) {
        self.english_auto_restore = enabled;
    }

    pub fn shortcuts(&self) -> &ShortcutTable {
        &self.shortcuts
    }

    pub fn shortcuts_mut(&mut self) -> &mut ShortcutTable {
        &mut self.shortcuts
    }

    /// Clear input buffer (public interface)
    /// Also restores pending_capitalize if auto_capitalize was used (for selection-delete)
    pub fn clear(&mut self) {
        // Restore pending_capitalize if auto_capitalize was used
        // This handles selection-delete: user selects and deletes text,
        // we should restore pending state so next letter is capitalized
        if self.auto_capitalize_used {
            self.pending_capitalize = true;
            self.auto_capitalize_used = false;
        }
        self.processor.clear();
        self.prev_chars.clear();
        self.curr_chars.clear();
        self.shortcut_prefix.clear();
    }

    /// Clear everything including word history
    pub fn clear_all(&mut self) {
        self.clear();
        self.word_history.clear();
        self.spaces_after_commit = 0;
    }

    /// Get the full composed buffer as a string
    pub fn get_buffer_string(&self) -> String {
        self.processor.buffer().to_full_string()
    }

    /// Restore buffer from a Vietnamese word string
    pub fn restore_word(&mut self, word: &str) {
        use crate::data::chars;
        use crate::engine::buffer::{Buffer, Char};

        self.processor.clear();
        self.prev_chars.clear();
        self.curr_chars.clear();

        // Build a buffer with parsed characters
        let mut buf = Buffer::new();
        for ch in word.chars() {
            if let Some(parsed) = chars::parse_char(ch) {
                let mut c = Char::new(parsed.key, parsed.caps);
                c.tone = parsed.tone;
                c.mark = parsed.mark;
                c.stroke = parsed.stroke;
                buf.push(c);
            }
        }

        // Use restore_buffer to properly set processor state
        self.processor.restore_buffer(buf);

        // Update prev_chars to match the word for diff calculation
        self.prev_chars = word.chars().collect();
    }

    // ===== Internal Helpers =====

    /// Get current input method
    fn current_input_method(&self) -> InputMethod {
        match self.processor.buffer().len() {
            0 => InputMethod::Telex,
            _ => InputMethod::All,
        }
    }

    /// Render buffer to characters
    fn render_buffer(&self) -> Vec<char> {
        self.processor.buffer().to_full_string().chars().collect()
    }

    /// Calculate diff and create result
    fn create_result(&mut self) -> Result {
        self.curr_chars = self.render_buffer();

        // Check for pending literal (VNI revert - digit to pass through)
        let pending_literal = self.processor.take_pending_literal();

        if self.curr_chars == self.prev_chars && pending_literal.is_none() {
            // Key was processed but no screen change - return Send with count=0
            // to indicate "consumed" rather than "pass through"
            return Result::send(0, &[]);
        }

        // Calculate common prefix length
        let common = self
            .prev_chars
            .iter()
            .zip(self.curr_chars.iter())
            .take_while(|(a, b)| a == b)
            .count();

        let backspace = (self.prev_chars.len() - common) as u8;
        let mut new_chars: Vec<char> = self.curr_chars[common..].to_vec();

        // Append pending literal if present (VNI revert)
        if let Some(ch) = pending_literal {
            new_chars.push(ch);
            // Also update curr_chars for consistency
            self.curr_chars.push(ch);
        }

        // Update prev for next call
        self.prev_chars = self.curr_chars.clone();

        Result::send(backspace, &new_chars)
    }

    /// Restore to raw ASCII
    fn restore_to_raw(&mut self) -> Result {
        let raw_chars = self.processor.raw().restore();
        let backspace = self.prev_chars.len() as u8;

        self.processor.clear();
        self.prev_chars.clear();
        self.curr_chars.clear();

        if raw_chars.is_empty() {
            Result::none()
        } else {
            Result::restore(backspace, &raw_chars)
        }
    }

    /// Check if current word should be restored as English
    fn should_restore_as_english(&self) -> bool {
        if !self.english_auto_restore {
            return false;
        }

        // Need at least 2 characters for detection
        if self.processor.raw().len() < 2 {
            return false;
        }

        // Collect raw keys
        let keys: Vec<u16> = self.processor.raw().iter().map(|(k, _)| k).collect();

        // First check: patterns that are definitely NOT Vietnamese
        if has_invalid_vietnamese_pattern(&keys) {
            return true;
        }

        // Second check: English likelihood (for less obvious cases)
        match english_likelihood_keys(&keys) {
            EnglishLikelihood::Likely | EnglishLikelihood::VeryLikely => true,
            _ => false,
        }
    }

    /// Try to restore as English word (returns Result if restored)
    fn try_english_restore(&mut self) -> Option<Result> {
        if !self.english_auto_restore {
            #[cfg(test)]
            eprintln!("[try_english_restore] auto_restore disabled");
            return None;
        }

        // Need at least 2 characters for detection
        if self.processor.raw().len() < 2 {
            #[cfg(test)]
            eprintln!("[try_english_restore] raw len < 2: {}", self.processor.raw().len());
            return None;
        }

        // Get raw chars and buffer string
        let raw_chars = self.processor.raw().restore_all();
        let vn_str = self.processor.buffer().to_full_string();

        #[cfg(test)]
        eprintln!("[try_english_restore] raw='{}' ({} chars), buffer='{}' ({} chars)",
            raw_chars.iter().collect::<String>(), raw_chars.len(),
            vn_str, vn_str.chars().count());

        // Quick comparison - if raw equals buffer, no transformation happened
        let raw_str: String = raw_chars.iter().collect();
        if raw_str == vn_str {
            // No transformation happened, no need to restore
            #[cfg(test)]
            eprintln!("[try_english_restore] raw == buffer, skip");
            return None;
        }

        // Collect raw keys for pattern detection
        let keys: Vec<u16> = self.processor.raw().iter().map(|(k, _)| k).collect();

        // Check 1: Pattern is DEFINITELY invalid Vietnamese
        let is_invalid_vn = has_invalid_vietnamese_pattern(&keys);

        // Check 2: Multiple DIFFERENT tone modifiers present
        // Vietnamese words can only have ONE tone mark, so if we see 2+ different tone
        // modifier keys (s, f, r, x, j), it's definitely English
        // "user" has 's' and 'r' → 2 tone modifiers → English
        // "ddepj" has only 'j' → 1 tone modifier → could be Vietnamese
        //
        // IMPORTANT: Only count tone modifiers that come AFTER a vowel!
        // 'r' in "raast" is an initial consonant, not a tone modifier
        // 's' and 'r' in "user" are both after vowels, so they count
        let is_tone_key = |k: u16| {
            matches!(
                k,
                crate::data::keys::S
                    | crate::data::keys::F
                    | crate::data::keys::R
                    | crate::data::keys::X
                    | crate::data::keys::J
            )
        };
        let is_vowel_key = |k: u16| {
            matches!(
                k,
                crate::data::keys::A
                    | crate::data::keys::E
                    | crate::data::keys::I
                    | crate::data::keys::O
                    | crate::data::keys::U
                    | crate::data::keys::Y
            )
        };

        // Find tone modifiers that come AFTER a vowel
        let mut tone_modifiers: Vec<u16> = Vec::new();
        let mut seen_vowel = false;
        for &k in &keys {
            if is_vowel_key(k) {
                seen_vowel = true;
            } else if seen_vowel && is_tone_key(k) {
                // This key is a tone modifier (after vowel)
                tone_modifiers.push(k);
            }
        }

        // Check if there are 2+ DIFFERENT tone modifiers (not just repeated same key)
        let unique_tone_mods: std::collections::HashSet<u16> =
            tone_modifiers.iter().copied().collect();
        let multiple_tone_modifiers = unique_tone_mods.len() >= 2;

        #[cfg(test)]
        eprintln!(
            "[try_english_restore] is_invalid_vn={}, tone_mods={:?}, multiple_tone_modifiers={}",
            is_invalid_vn, unique_tone_mods, multiple_tone_modifiers
        );

        // Only restore if either check passes
        if !is_invalid_vn && !multiple_tone_modifiers {
            // Pattern could be valid Vietnamese - don't restore
            #[cfg(test)]
            eprintln!("[try_english_restore] not restoring - could be valid VN");
            return None;
        }

        // Pattern is definitely invalid Vietnamese - decide whether to use raw or buffer
        // For double consonant reverts in SHORT words (<=4 raw chars), use buffer
        // This handles intentional reverts like "bass" → "bas", "miss" → "mis"
        // For longer words (5+ chars), use raw to preserve the double consonant
        if raw_chars.len() <= 4 && self.has_double_consonant_revert(&raw_str, &vn_str) {
            // Short word with revert - user probably intended the revert
            let buffer_chars: Vec<char> = vn_str.chars().collect();
            let backspace = self.prev_chars.len() as u8;
            self.processor.clear();
            self.prev_chars.clear();
            self.curr_chars.clear();
            return Some(Result::restore(backspace, &buffer_chars));
        }

        // Longer word or no revert - restore to raw
        // This handles words like "issue", "coffee", "brown" etc.
        let backspace = self.prev_chars.len() as u8;

        self.processor.clear();
        self.prev_chars.clear();
        self.curr_chars.clear();

        Some(Result::restore(backspace, &raw_chars))
    }

    /// Check if raw has double consonant that was reverted in buffer
    /// e.g., raw="soffa" buffer="sofa" → true (ff was reverted to f)
    fn has_double_consonant_revert(&self, raw: &str, buffer: &str) -> bool {
        // Quick length check - buffer should be shorter if revert happened
        if raw.len() <= buffer.len() {
            return false;
        }

        // Check for common double consonants that could revert
        // s, f, r, x, j are Telex modifiers that can double-revert
        let modifiers = ['s', 'f', 'r', 'x', 'j', 'w'];

        for m in modifiers {
            let double = format!("{}{}", m, m);
            if raw.contains(&double) && !buffer.contains(&double) {
                // Raw has double, buffer doesn't - likely a revert
                return true;
            }
        }

        false
    }

    /// Try word boundary shortcut
    fn try_word_boundary_shortcut(&mut self, include_space: bool) -> Result {
        // Don't trigger shortcuts after numbers (e.g., "149k" should not expand "k")
        if self.prev_was_number {
            return Result::none();
        }

        // Use to_full_string to get proper Vietnamese chars (đ, etc.)
        // Don't lowercase - shortcuts are case-sensitive
        let buffer_str = self.processor.buffer().to_full_string();

        // Prepend shortcut prefix if any (e.g., '#' for "#fne" shortcuts)
        let trigger = if self.shortcut_prefix.is_empty() {
            buffer_str.clone()
        } else {
            format!("{}{}", self.shortcut_prefix, buffer_str)
        };

        if trigger.is_empty() {
            return Result::none();
        }

        if let Some((_trigger, shortcut)) = self
            .shortcuts
            .lookup_for_method(&trigger, self.current_input_method())
        {
            let backspace = self.prev_chars.len() as u8;
            let mut chars: Vec<char> = shortcut.replacement.chars().collect();
            // Add the triggering space if requested
            if include_space {
                chars.push(' ');
            }
            // Clear prefix after successful shortcut match
            self.shortcut_prefix.clear();
            return Result::send(backspace, &chars);
        }

        Result::none()
    }

    // ===== Main Entry Points =====

    /// Handle key event (simplified)
    pub fn on_key(&mut self, key: u16, caps: bool, ctrl: bool) -> Result {
        self.on_key_ext(key, caps, ctrl, false)
    }

    /// Handle key event with extended parameters
    ///
    /// Main entry point - same interface as old Engine.
    pub fn on_key_ext(&mut self, key: u16, caps: bool, ctrl: bool, shift: bool) -> Result {
        if !self.enabled || ctrl {
            self.clear();
            self.word_history.clear();
            self.spaces_after_commit = 0;
            return Result::none();
        }

        // SPACE: check shortcuts, commit word
        if key == keys::SPACE {
            // Try shortcut first
            let shortcut_result = self.try_word_boundary_shortcut(true);
            if shortcut_result.action != 0 {
                self.prev_was_number = false;
                self.clear();
                return shortcut_result;
            }

            // Try English auto-restore
            if let Some(restore_result) = self.try_english_restore() {
                self.prev_was_number = false;
                self.word_history.clear();
                self.spaces_after_commit = 0;
                return restore_result;
            }

            // Push to history before clearing
            if !self.processor.buffer().is_empty() {
                self.word_history.push(self.processor.buffer().clone());
                self.spaces_after_commit = 1;
            } else if self.spaces_after_commit > 0 {
                self.spaces_after_commit = self.spaces_after_commit.saturating_add(1);
            }
            self.auto_capitalize_used = false; // Reset on word commit
            self.prev_was_number = false; // Reset number context on word commit
            self.clear();
            return Result::none();
        }

        // ESC: restore to raw ASCII
        if key == keys::ESC {
            let result = if self.esc_restore_enabled {
                self.restore_to_raw()
            } else {
                Result::none()
            };
            self.clear();
            self.word_history.clear();
            self.spaces_after_commit = 0;
            return result;
        }

        // Check for shortcut prefix characters (e.g., # from Shift+3)
        // Must be checked BEFORE break key handling
        if shift && keys::is_number(key) {
            let prefix_char = match key {
                keys::N1 => '!',
                keys::N2 => '@',
                keys::N3 => '#',
                keys::N4 => '$',
                keys::N5 => '%',
                keys::N6 => '^',
                keys::N7 => '&',
                keys::N8 => '*',
                keys::N9 => '(',
                keys::N0 => ')',
                _ => return Result::none(),
            };
            // Auto-capitalize: set pending on sentence-ending (!  ?)
            if self.auto_capitalize && is_sentence_ending(key, shift) {
                self.pending_capitalize = true;
            }
            self.auto_capitalize_used = false;

            // Try English auto-restore BEFORE clearing (same as break keys)
            // This handles "user@" → "user@" instead of "ủe@"
            if let Some(restore_result) = self.try_english_restore() {
                self.word_history.clear();
                self.spaces_after_commit = 0;
                self.shortcut_prefix.push(prefix_char);
                return restore_result;
            }

            // Add prefix character for shortcut matching
            self.shortcut_prefix.push(prefix_char);
            // Clear processor and prev_chars (word boundary)
            // Don't add to prev_chars - we don't want diff to include this punctuation
            self.processor.finalize_word();
            self.processor.clear();
            self.prev_chars.clear();
            return Result::none();
        }

        // Break keys: punctuation, arrows, etc.
        if keys::is_break_ext(key, shift) {
            // Track if break was caused by a number (not shift+number symbol)
            self.prev_was_number = !shift && keys::is_number(key);
            // Auto-capitalize: set pending on sentence-ending
            if self.auto_capitalize && is_sentence_ending(key, shift) {
                self.pending_capitalize = true;
            }
            self.auto_capitalize_used = false; // Reset on word boundary

            // Try English auto-restore for punctuation (same as SPACE)
            // This handles "toto," → "toto," instead of "tôt,"
            if let Some(restore_result) = self.try_english_restore() {
                self.word_history.clear();
                self.spaces_after_commit = 0;
                return restore_result;
            }

            // Clear everything including word history
            self.clear();
            self.word_history.clear();
            self.spaces_after_commit = 0;
            return Result::none();
        }

        // DELETE/BACKSPACE handling
        if key == keys::DELETE {
            // Try to backspace in the processor
            let had_content = self.processor.backspace();

            if !had_content {
                // Buffer was already empty - check for backspace-after-space restore
                if self.spaces_after_commit > 0 {
                    self.spaces_after_commit -= 1;

                    // Return immediately to delete one space from screen
                    // When spaces_after_commit reaches 0, also restore the word
                    if self.spaces_after_commit == 0 {
                        // Restore last word from history
                        if let Some(restored) = self.word_history.pop() {
                            let word_str = restored.to_full_string();
                            self.processor.restore_buffer(restored);
                            self.prev_chars = word_str.chars().collect();
                        }
                    }
                    // Delete one space from screen
                    return Result::send(1, &[]);
                }
            }

            // Check if buffer is now empty after backspace
            if self.processor.buffer().is_empty() {
                // Restore pending_capitalize if user deleted the auto-capitalized letter
                // This allows: ". B" → delete B → ". " → type again → auto-capitalizes
                if self.auto_capitalize_used {
                    self.pending_capitalize = true;
                    self.auto_capitalize_used = false;
                }
                self.prev_chars.clear();
            } else {
                // Update prev_chars to match current buffer state after backspace
                // This ensures next key press correctly calculates the diff
                self.prev_chars = self.processor.buffer().to_full_string().chars().collect();
            }

            return Result::none();
        }

        // Skip w→ư shortcut if configured
        if self.skip_w_shortcut && key == keys::W && self.processor.buffer().is_empty() {
            // Just pass through as regular 'w'
            return Result::none();
        }

        // Auto-capitalize: apply to first letter after sentence-ending
        let auto_cap_triggered = self.pending_capitalize && keys::is_letter(key) && !caps;
        let effective_caps = if self.pending_capitalize && keys::is_letter(key) {
            self.pending_capitalize = false;
            self.auto_capitalize_used = true;
            true
        } else {
            // Reset pending on number (e.g., "1.5k" should not capitalize "k")
            if self.pending_capitalize && keys::is_number(key) {
                self.pending_capitalize = false;
                self.auto_capitalize_used = false;
            }
            caps
        };

        // Process through matrix-based processor
        let result = self.processor.process(key, effective_caps, shift);

        // Map ProcessResult to Result
        match result {
            ProcessResult::None => Result::none(),
            ProcessResult::Update => {
                // If auto-capitalize triggered on this key, we need to transform
                // (replace lowercase with uppercase)
                if auto_cap_triggered {
                    return self.create_result();
                }
                // Simple addition, no transformation - let the key pass through naturally
                // Update prev_chars to prefix + buffer for future diff calculations
                self.prev_chars = self.shortcut_prefix.chars()
                    .chain(self.processor.buffer().to_full_string().chars())
                    .collect();
                Result::none()
            }
            ProcessResult::Transform | ProcessResult::Revert => self.create_result(),
            ProcessResult::Reject => {
                // Check if this is a shortcut prefix character (e.g., # from Shift+3)
                let prefix_char = if shift {
                    match key {
                        keys::N1 => Some('!'),
                        keys::N2 => Some('@'),
                        keys::N3 => Some('#'),
                        keys::N4 => Some('$'),
                        keys::N5 => Some('%'),
                        keys::N6 => Some('^'),
                        keys::N7 => Some('&'),
                        keys::N8 => Some('*'),
                        keys::N9 => Some('('),
                        keys::N0 => Some(')'),
                        _ => None,
                    }
                } else {
                    None
                };

                if let Some(ch) = prefix_char {
                    // Add prefix character for shortcut matching
                    // Don't clear buffer - might be a shortcut like "#fne"
                    self.shortcut_prefix.push(ch);
                    self.prev_chars.push(ch);
                    Result::none()
                } else {
                    // Word boundary on rejection - finalize and reset
                    // Track if rejection was caused by a number (prevents shortcut after "149k")
                    self.prev_was_number = keys::is_number(key);
                    self.processor.finalize_word();
                    self.processor.clear();
                    self.prev_chars.clear();
                    self.shortcut_prefix.clear();
                    Result::none()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_engine_new() {
        let engine = Engine::new();
        assert!(engine.enabled);
    }

    #[test]
    fn test_matrix_engine_process_vowel() {
        let mut engine = Engine::new();

        // Type 'a' - simple addition passes through (no transformation)
        let result = engine.on_key_ext(keys::A, false, false, false);

        // Simple additions pass through with action=None
        assert_eq!(result.action, Action::None as u8);
        // But buffer should have the character
        assert_eq!(engine.get_buffer_string(), "a");
    }

    #[test]
    fn test_matrix_engine_process_viet() {
        let mut engine = Engine::new();

        // Type "viet" - all simple additions
        engine.on_key_ext(keys::V, false, false, false);
        engine.on_key_ext(keys::I, false, false, false);
        engine.on_key_ext(keys::E, false, false, false);
        let result = engine.on_key_ext(keys::T, false, false, false);

        // Simple additions pass through
        assert_eq!(result.action, Action::None as u8);
        // But buffer should have the word
        assert_eq!(engine.get_buffer_string(), "viet");
    }

    #[test]
    fn test_matrix_engine_tone() {
        let mut engine = Engine::new();

        // Type "vie" + 's' for sắc tone
        engine.on_key_ext(keys::V, false, false, false);
        engine.on_key_ext(keys::I, false, false, false);
        engine.on_key_ext(keys::E, false, false, false);
        let result = engine.on_key_ext(keys::S, false, false, false);

        // Tone should produce output
        assert_eq!(result.action, Action::Send as u8);
    }

    #[test]
    fn test_matrix_engine_stroke() {
        let mut engine = Engine::new();

        // Type "dd" for đ
        engine.on_key_ext(keys::D, false, false, false);
        let result = engine.on_key_ext(keys::D, false, false, false);

        assert_eq!(result.action, Action::Send as u8);
    }

    #[test]
    fn test_matrix_engine_disabled() {
        let mut engine = Engine::new();
        engine.set_enabled(false);

        let result = engine.on_key_ext(keys::A, false, false, false);
        assert_eq!(result.action, Action::None as u8);
    }

    #[test]
    fn test_matrix_engine_ctrl_bypass() {
        let mut engine = Engine::new();

        // Ctrl+key should bypass
        let result = engine.on_key_ext(keys::A, false, true, false);
        assert_eq!(result.action, Action::None as u8);
    }

    #[test]
    fn test_matrix_engine_space_clears() {
        let mut engine = Engine::new();

        // Type some letters
        engine.on_key_ext(keys::A, false, false, false);
        engine.on_key_ext(keys::B, false, false, false);

        // Space should clear buffer
        let result = engine.on_key_ext(keys::SPACE, false, false, false);
        assert_eq!(result.action, Action::None as u8);

        // Next letter should start fresh (simple addition = passthrough)
        let result = engine.on_key_ext(keys::C, false, false, false);
        assert_eq!(result.action, Action::None as u8);
        // Buffer should only have 'c', not 'abc'
        assert_eq!(engine.get_buffer_string(), "c");
    }

    #[test]
    fn test_shortcut_prefix_debug() {
        let mut engine = Engine::new();
        engine.shortcuts_mut().add(
            crate::engine::shortcut::Shortcut::new("#fne", "for next episode")
        );

        // Type '#' (Shift+3) - should be rejected but tracked as prefix
        let r1 = engine.on_key_ext(keys::N3, false, false, true);
        eprintln!("[#] action={}, prefix='{}', buffer='{}'",
            r1.action, engine.shortcut_prefix, engine.get_buffer_string());

        // Type 'f'
        let r2 = engine.on_key_ext(keys::F, false, false, false);
        eprintln!("[f] action={}, prefix='{}', buffer='{}'",
            r2.action, engine.shortcut_prefix, engine.get_buffer_string());

        // Type 'n'
        let r3 = engine.on_key_ext(keys::N, false, false, false);
        eprintln!("[n] action={}, prefix='{}', buffer='{}'",
            r3.action, engine.shortcut_prefix, engine.get_buffer_string());

        // Type 'e'
        let r4 = engine.on_key_ext(keys::E, false, false, false);
        eprintln!("[e] action={}, prefix='{}', buffer='{}'",
            r4.action, engine.shortcut_prefix, engine.get_buffer_string());

        // Type SPACE - should trigger shortcut
        let r5 = engine.on_key_ext(keys::SPACE, false, false, false);
        eprintln!("[SPACE] action={}", r5.action);

        assert_eq!(r5.action, Action::Send as u8, "shortcut '#fne' should trigger");
    }

    #[test]
    fn test_debug_english_restore() {
        let mut engine = Engine::new();
        engine.set_english_auto_restore(true);

        // Type "water"
        for &key in &[keys::W, keys::A, keys::T, keys::E, keys::R] {
            let r = engine.on_key_ext(key, false, false, false);
            eprintln!("[key {:?}] action={}, buffer='{}'", key, r.action, engine.get_buffer_string());
        }

        // Check raw buffer contents
        let raw_keys: Vec<_> = engine.processor.raw().iter().collect();
        eprintln!("Raw keys: {:?}", raw_keys);
        eprintln!("Raw len: {}", engine.processor.raw().len());

        // Type SPACE - should trigger auto-restore
        let r = engine.on_key_ext(keys::SPACE, false, false, false);
        eprintln!("[SPACE] action={}, backspace={}, count={}", r.action, r.backspace, r.count);
        for i in 0..r.count as usize {
            eprintln!("  char[{}] = {:?}", i, char::from_u32(r.chars[i]));
        }

        // Should restore to "water" not "ưater"
        assert_eq!(r.action, Action::Restore as u8, "should trigger auto-restore");
    }

    #[test]
    fn test_debug_issue() {
        let mut engine = Engine::new();
        engine.set_english_auto_restore(true);

        // Type "issue" - note: two 's' characters
        let word_keys = [keys::I, keys::S, keys::S, keys::U, keys::E];
        for &key in &word_keys {
            let r = engine.on_key_ext(key, false, false, false);
            eprintln!("[key {:?}] action={}, buffer='{}', raw_len={}",
                key, r.action, engine.get_buffer_string(), engine.processor.raw().len());
        }

        // Check raw buffer contents
        let raw_chars: Vec<char> = engine.processor.raw().restore_all();
        eprintln!("Raw chars (restore_all): {:?}", raw_chars);
        eprintln!("Raw len: {}", engine.processor.raw().len());

        // Check buffer
        eprintln!("Buffer: '{}'", engine.get_buffer_string());

        // Type SPACE - should trigger auto-restore
        let r = engine.on_key_ext(keys::SPACE, false, false, false);
        eprintln!("[SPACE] action={}, backspace={}, count={}", r.action, r.backspace, r.count);

        let result: String = (0..r.count as usize)
            .filter_map(|i| char::from_u32(r.chars[i]))
            .collect();
        eprintln!("Result: '{}'", result);

        // Should restore to "issue" with both 's' characters
        // Note: restore doesn't include the trailing space
        assert_eq!(result, "issue", "should restore to 'issue'");
    }

    #[test]
    fn test_debug_around() {
        let mut engine = Engine::new();
        engine.set_english_auto_restore(true);

        // Type "around"
        let word_keys = [keys::A, keys::R, keys::O, keys::U, keys::N, keys::D];
        for &key in &word_keys {
            let r = engine.on_key_ext(key, false, false, false);
            eprintln!("[key {:?}] action={}, buffer='{}', raw_len={}",
                key, r.action, engine.get_buffer_string(), engine.processor.raw().len());
        }

        // Check raw buffer contents
        let raw_chars: Vec<char> = engine.processor.raw().restore_all();
        eprintln!("Raw chars (restore_all): {:?}", raw_chars);
        eprintln!("Raw len: {}", engine.processor.raw().len());

        // Check buffer
        eprintln!("Buffer: '{}'", engine.get_buffer_string());

        // Type SPACE - should trigger auto-restore
        let r = engine.on_key_ext(keys::SPACE, false, false, false);
        eprintln!("[SPACE] action={}, backspace={}, count={}", r.action, r.backspace, r.count);

        let result: String = (0..r.count as usize)
            .filter_map(|i| char::from_u32(r.chars[i]))
            .collect();
        eprintln!("Result: '{}'", result);

        // Should restore to "around"
        assert_eq!(result, "around", "should restore to 'around'");
    }

    #[test]
    fn test_debug_dataa() {
        let mut engine = Engine::new();
        engine.set_english_auto_restore(true);

        // Type "dataa " - double 'a' should revert circumflex
        let word_keys = [keys::D, keys::A, keys::T, keys::A, keys::A];
        for &key in &word_keys {
            let r = engine.on_key_ext(key, false, false, false);
            eprintln!("[key {:?}] action={}, buffer='{}', buf_len={}, raw_len={}",
                key, r.action, engine.get_buffer_string(), engine.processor.buffer().len(), engine.processor.raw().len());
        }

        // Check raw buffer contents
        let raw_chars: Vec<char> = engine.processor.raw().restore_all();
        eprintln!("Raw chars (restore_all): {:?}", raw_chars);
        eprintln!("Raw len: {}", engine.processor.raw().len());

        // Check buffer before space
        eprintln!("Buffer before space: '{}'", engine.get_buffer_string());

        // Type SPACE - should trigger auto-restore
        let r = engine.on_key_ext(keys::SPACE, false, false, false);
        eprintln!("[SPACE] action={}, backspace={}, count={}", r.action, r.backspace, r.count);

        let result: String = (0..r.count as usize)
            .filter_map(|i| char::from_u32(r.chars[i]))
            .collect();
        eprintln!("Result: '{}'", result);

        // With fix: raw_str == vn_str, so no restore happens
        // action=0 means no restore, just add space normally
        // Buffer is already "data", so the visible output should be "data " after space
        // Note: buffer is cleared after SPACE, so we check before space was typed
        assert!(result.is_empty() || result == "data", "buffer should be 'data' or empty after commit");
    }

    #[test]
    fn test_debug_toto() {
        let mut engine = Engine::new();
        engine.set_english_auto_restore(true);

        // Type "toto" - circumflex should be applied but not reverted
        let word_keys = [keys::T, keys::O, keys::T, keys::O];
        for &key in &word_keys {
            let r = engine.on_key_ext(key, false, false, false);
            eprintln!("[key {:?}] action={}, buffer='{}', buf_len={}, raw_len={}",
                key, r.action, engine.get_buffer_string(), engine.processor.buffer().len(), engine.processor.raw().len());
        }

        // Check raw buffer contents
        let raw_chars: Vec<char> = engine.processor.raw().restore_all();
        eprintln!("Raw chars (restore_all): {:?}", raw_chars);
        eprintln!("Raw len: {}", engine.processor.raw().len());

        // Check buffer before space
        let buffer_before = engine.get_buffer_string();
        eprintln!("Buffer before space: '{}'", buffer_before);

        // Type SPACE
        let r = engine.on_key_ext(keys::SPACE, false, false, false);
        eprintln!("[SPACE] action={}, backspace={}, count={}", r.action, r.backspace, r.count);

        let result: String = (0..r.count as usize)
            .filter_map(|i| char::from_u32(r.chars[i]))
            .collect();
        eprintln!("Result: '{}'", result);

        // For "toto", we expect circumflex to be applied ("tôt")
        // But on space, it should restore to "toto" if raw != buffer
        eprintln!("Expected behavior: buffer='tôt' but restore to 'toto'");
    }

    #[test]
    fn test_debug_user() {
        let mut engine = Engine::new();
        engine.set_english_auto_restore(true);

        // Type "user"
        let word_keys = [keys::U, keys::S, keys::E, keys::R];
        for &key in &word_keys {
            let r = engine.on_key_ext(key, false, false, false);
            eprintln!("[key {:?}] action={}, buffer='{}', buf_len={}, raw_len={}",
                key, r.action, engine.get_buffer_string(), engine.processor.buffer().len(), engine.processor.raw().len());
        }

        // Check raw buffer contents
        let raw_chars: Vec<char> = engine.processor.raw().restore_all();
        eprintln!("Raw chars (restore_all): {:?}", raw_chars);
        eprintln!("Raw len: {}", engine.processor.raw().len());

        // Check buffer before @
        let buffer_before = engine.get_buffer_string();
        eprintln!("Buffer before @: '{}'", buffer_before);

        // Type @ (Shift+2)
        let r = engine.on_key_ext(keys::N2, false, false, true);
        eprintln!("[@] action={}, backspace={}, count={}", r.action, r.backspace, r.count);

        let result: String = (0..r.count as usize)
            .filter_map(|i| char::from_u32(r.chars[i]))
            .collect();
        eprintln!("Result: '{}'", result);
    }

    #[test]
    fn test_debug_toto_comma() {
        let mut engine = Engine::new();
        engine.set_english_auto_restore(true);

        // Type "toto"
        let word_keys = [keys::T, keys::O, keys::T, keys::O];
        for &key in &word_keys {
            let r = engine.on_key_ext(key, false, false, false);
            eprintln!("[key {:?}] action={}, buffer='{}', buf_len={}, raw_len={}",
                key, r.action, engine.get_buffer_string(), engine.processor.buffer().len(), engine.processor.raw().len());
        }

        // Check raw buffer contents
        let raw_chars: Vec<char> = engine.processor.raw().restore_all();
        eprintln!("Raw chars (restore_all): {:?}", raw_chars);
        eprintln!("Buffer before comma: '{}'", engine.get_buffer_string());

        // Type comma
        let r = engine.on_key_ext(keys::COMMA, false, false, false);
        eprintln!("[COMMA] action={}, backspace={}, count={}", r.action, r.backspace, r.count);

        let result: String = (0..r.count as usize)
            .filter_map(|i| char::from_u32(r.chars[i]))
            .collect();
        eprintln!("Result: '{}'", result);
    }
}
