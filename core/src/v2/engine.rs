//! V2 Engine - Main entry point
//!
//! Implements the 7-step keystroke pipeline as specified in engine-v2-specs.md.
//! Matches V1's public API for seamless FFI integration.

use super::buffer::Buffer;
use super::classify::{classify_key, Method};
use super::dict::Dict;
use super::output::generate_output;
use super::precheck::{pre_check, Mode};
use super::restore::{should_restore, Decision};
use super::state::{BufferState, VnState};
use super::types::{KeyType, MarkType};
use super::validate::validate_vn;

use crate::data::keys;
use crate::engine::shortcut::ShortcutTable;
use crate::utils;

/// Maximum buffer size (matches V1)
pub const MAX: usize = 256;

/// Engine action result
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    None = 0,
    Send = 1,
    Restore = 2,
}

/// Result for FFI (matches V1's Result struct exactly)
#[repr(C)]
pub struct Result {
    pub chars: [u32; MAX],
    pub action: u8,
    pub backspace: u8,
    pub count: u8,
    /// Flags byte:
    /// - bit 0 (0x01): key_consumed - if set, the trigger key should NOT be passed through
    pub flags: u8,
}

/// Flag: key was consumed by shortcut, don't pass through
pub const FLAG_KEY_CONSUMED: u8 = 0x01;

impl Result {
    pub fn none() -> Self {
        Self {
            chars: [0; MAX],
            action: Action::None as u8,
            backspace: 0,
            count: 0,
            flags: 0,
        }
    }

    pub fn send(backspace: u8, chars: &[char]) -> Self {
        let mut result = Self {
            chars: [0; MAX],
            action: Action::Send as u8,
            backspace,
            count: chars.len().min(MAX) as u8,
            flags: 0,
        };
        for (i, &c) in chars.iter().take(MAX).enumerate() {
            result.chars[i] = c as u32;
        }
        result
    }

    /// Send with key_consumed flag set (shortcut consumed the trigger key)
    pub fn send_consumed(backspace: u8, chars: &[char]) -> Self {
        let mut result = Self::send(backspace, chars);
        result.flags = FLAG_KEY_CONSUMED;
        result
    }

    /// Check if key was consumed (should not be passed through)
    pub fn key_consumed(&self) -> bool {
        self.flags & FLAG_KEY_CONSUMED != 0
    }
}

/// V2 Vietnamese IME Engine
///
/// Implements the 7-step keystroke pipeline:
/// 0. Key Classification
/// 1. Pre-check (foreign mode)
/// 2. Dispatch & Execute
/// 3. Tone/Mark Placement
/// 4. Buffer Update
/// 5. 9-Layer Validation
/// 6. Restore Decision
/// 7. Output Generation
pub struct Engine {
    /// Dual-string buffer (raw + transformed)
    buffer: Buffer,
    /// State flags bitmask
    state: BufferState,
    /// Input method (Telex/VNI)
    method: Method,
    /// Whether engine is enabled
    enabled: bool,
    /// Previous key (for double-key patterns like dd, aa)
    prev_key: Option<u8>,
    /// Previous output (for diff calculation)
    prev_output: String,

    // Settings (match V1)
    skip_w_shortcut: bool,
    esc_restore_enabled: bool,
    free_tone_enabled: bool,
    modern_tone: bool,
    english_auto_restore: bool,
    auto_capitalize: bool,

    // Shortcuts (reuse V1's ShortcutTable)
    shortcuts: ShortcutTable,

    // Dictionary for auto-restore (optional)
    #[allow(dead_code)]
    dict: Option<Dict>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Create new engine with default settings
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            state: BufferState::new(),
            method: Method::Telex,
            enabled: true,
            prev_key: None,
            prev_output: String::new(),

            skip_w_shortcut: false,
            esc_restore_enabled: false,
            free_tone_enabled: false,
            modern_tone: true,
            english_auto_restore: false,
            auto_capitalize: false,

            shortcuts: ShortcutTable::with_defaults(),
            dict: None,
        }
    }

    /// Main key event handler (matches V1's on_key)
    pub fn on_key(&mut self, key: u16, caps: bool, ctrl: bool) -> Result {
        self.on_key_ext(key, caps, ctrl, false)
    }

    /// Extended key event handler (matches V1's on_key_ext)
    pub fn on_key_ext(&mut self, key: u16, caps: bool, ctrl: bool, shift: bool) -> Result {
        // Bypass if disabled or Ctrl/Cmd pressed
        if !self.enabled || ctrl {
            return Result::none();
        }

        // Step 0: Key Classification
        let ascii = key_to_ascii(key, caps, shift);
        let ascii_byte = match ascii {
            Some(c) => c as u8,
            None => {
                // Handle special keys that don't map to ASCII
                return self.handle_special_key(key);
            }
        };

        let key_type = classify_key(ascii_byte, self.prev_key, self.method);

        // Update prev_key for next keystroke
        self.prev_key = Some(ascii_byte);

        // Step 1: Pre-check (early foreign detection)
        if self.buffer.raw_len() < 3 {
            let mode = pre_check(self.buffer.raw());
            if mode == Mode::Foreign {
                // Foreign mode: append without transform
                if let Some(c) = ascii {
                    self.buffer.push_raw(c);
                    self.buffer.push_transformed(c);
                }
                return self.rebuild_output();
            }
        }

        // Step 2-7: Process based on key type
        match key_type {
            KeyType::Letter(c) => self.handle_letter(c as char),
            KeyType::Tone(t) => self.handle_tone(t),
            KeyType::Mark(m) => self.handle_mark(m),
            KeyType::Terminator => self.handle_terminator(ascii),
            KeyType::Special => self.handle_special_key(key),
            KeyType::Passthrough => {
                // For passthrough, still append to buffer if it's a character
                if let Some(c) = ascii {
                    self.buffer.push_raw(c);
                    self.buffer.push_transformed(c);
                }
                Result::none()
            }
        }
    }

    /// Handle regular letter input
    fn handle_letter(&mut self, c: char) -> Result {
        self.buffer.push_raw(c);
        self.buffer.push_transformed(c);

        // Step 5: Validate Vietnamese
        let vn_state = validate_vn(self.buffer.transformed());
        self.state.set_vn_state(vn_state);
        self.state.set_had_transform(true);

        // Immediate restore check if Impossible and auto-restore enabled
        if vn_state == VnState::Impossible && self.english_auto_restore {
            return self.restore_to_raw();
        }

        self.rebuild_output()
    }

    /// Handle tone key (Telex: s,f,r,x,j or VNI: 1-5)
    fn handle_tone(&mut self, tone: u8) -> Result {
        // For now, append raw key and mark state
        let c = tone as char;
        self.buffer.push_raw(c);

        // TODO: Implement proper tone placement using placement.rs
        // For now, just pass through
        self.buffer.push_transformed(c);

        self.state.set_has_tone(true);
        self.state.set_had_transform(true);

        self.rebuild_output()
    }

    /// Handle mark key (circumflex, horn, breve, stroke)
    fn handle_mark(&mut self, mark: MarkType) -> Result {
        // Append raw key
        let raw_char = match mark {
            MarkType::Stroke => 'd',
            MarkType::Circumflex => {
                // Get the doubled vowel
                self.prev_key.map(|k| k as char).unwrap_or('a')
            }
            MarkType::HornOrBreve => 'w',
        };
        self.buffer.push_raw(raw_char);

        // TODO: Implement proper mark placement
        // For now, just pass through
        self.buffer.push_transformed(raw_char);

        if mark == MarkType::Stroke {
            self.state.set_has_stroke(true);
        }
        self.state.set_had_transform(true);

        self.rebuild_output()
    }

    /// Handle terminator (word boundary)
    fn handle_terminator(&mut self, ascii: Option<char>) -> Result {
        // Step 6: Restore decision
        let decision = should_restore(
            &self.state,
            self.buffer.raw(),
            self.buffer.transformed(),
            self.dict.as_ref(),
        );

        let result = match decision {
            Decision::Keep => self.rebuild_output(),
            Decision::Restore => self.restore_to_raw(),
            Decision::Skip => Result::none(),
        };

        // Clear buffer for next word (preserve the result)
        self.clear();

        // Add terminator character to result if needed
        if let Some(term_char) = ascii {
            let mut chars: Vec<char> = (0..result.count as usize)
                .filter_map(|i| char::from_u32(result.chars[i]))
                .collect();
            chars.push(term_char);
            return Result::send(result.backspace, &chars);
        }

        result
    }

    /// Handle special keys (backspace/delete, esc)
    fn handle_special_key(&mut self, key: u16) -> Result {
        match key {
            keys::DELETE => {
                // Pop from both buffers
                self.buffer.pop_raw();
                self.buffer.pop_transformed();

                // Recalculate state
                if self.buffer.is_empty() {
                    self.state.clear();
                } else {
                    let vn_state = validate_vn(self.buffer.transformed());
                    self.state.set_vn_state(vn_state);
                }

                Result::none()
            }
            keys::ESC if self.esc_restore_enabled => self.restore_to_raw(),
            _ => Result::none(),
        }
    }

    /// Generate output diff from previous state
    fn rebuild_output(&mut self) -> Result {
        let current = self.buffer.transformed().to_string();
        let (backspaces, commit) = generate_output(&self.prev_output, &current);
        self.prev_output = current;

        if backspaces == 0 && commit.is_empty() {
            return Result::none();
        }

        let chars: Vec<char> = commit.chars().collect();
        Result::send(backspaces, &chars)
    }

    /// Restore buffer to raw input
    fn restore_to_raw(&mut self) -> Result {
        let raw = self.buffer.raw().to_string();
        let (backspaces, commit) = generate_output(&self.prev_output, &raw);
        self.prev_output = raw.clone();
        self.buffer.set_transformed(raw);

        let chars: Vec<char> = commit.chars().collect();
        Result::send(backspaces, &chars)
    }

    // ===== Public API matching V1 =====

    /// Clear current buffer (preserves history)
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.state.clear();
        self.prev_key = None;
        self.prev_output.clear();
    }

    /// Clear everything including history
    pub fn clear_all(&mut self) {
        self.clear();
        // V2 doesn't have word history yet, but matches V1 API
    }

    /// Set input method (0=Telex, 1=VNI)
    pub fn set_method(&mut self, method: u8) {
        self.method = if method == 0 {
            Method::Telex
        } else {
            Method::Vni
        };
    }

    /// Check if engine is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable/disable engine
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.clear();
        }
    }

    /// Set skip w→ư shortcut
    pub fn set_skip_w_shortcut(&mut self, skip: bool) {
        self.skip_w_shortcut = skip;
    }

    /// Set ESC restore behavior
    pub fn set_esc_restore(&mut self, enabled: bool) {
        self.esc_restore_enabled = enabled;
    }

    /// Set free tone placement (skip validation)
    pub fn set_free_tone(&mut self, enabled: bool) {
        self.free_tone_enabled = enabled;
    }

    /// Set modern tone placement style
    pub fn set_modern_tone(&mut self, modern: bool) {
        self.modern_tone = modern;
    }

    /// Set English auto-restore
    pub fn set_english_auto_restore(&mut self, enabled: bool) {
        self.english_auto_restore = enabled;
    }

    /// Set auto-capitalize
    pub fn set_auto_capitalize(&mut self, enabled: bool) {
        self.auto_capitalize = enabled;
    }

    /// Get shortcuts table (immutable)
    pub fn shortcuts(&self) -> &ShortcutTable {
        &self.shortcuts
    }

    /// Get shortcuts table (mutable)
    pub fn shortcuts_mut(&mut self) -> &mut ShortcutTable {
        &mut self.shortcuts
    }

    /// Get current buffer string
    pub fn get_buffer_string(&self) -> String {
        self.buffer.transformed().to_string()
    }

    /// Restore word from Vietnamese string
    pub fn restore_word(&mut self, word: &str) {
        // Parse Vietnamese word back to buffer
        // For V2, we just set the buffer directly
        self.buffer.set_transformed(word.to_string());
        for c in word.chars() {
            self.buffer.push_raw(c);
        }
        self.prev_output = word.to_string();

        // Validate
        let vn_state = validate_vn(word);
        self.state.set_vn_state(vn_state);
        self.state.set_had_transform(true);
    }
}

/// Convert keycode to ASCII character
fn key_to_ascii(key: u16, caps: bool, shift: bool) -> Option<char> {
    // Use existing utils function with caps XOR shift for case
    utils::key_to_char(key, caps ^ shift)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let e = Engine::new();
        assert!(e.enabled);
        assert_eq!(e.method, Method::Telex);
        assert!(e.buffer.is_empty());
    }

    #[test]
    fn test_engine_disabled() {
        let mut e = Engine::new();
        e.set_enabled(false);
        let r = e.on_key(keys::A, false, false);
        assert_eq!(r.action, Action::None as u8);
    }

    #[test]
    fn test_engine_ctrl_bypass() {
        let mut e = Engine::new();
        let r = e.on_key(keys::A, false, true);
        assert_eq!(r.action, Action::None as u8);
    }

    #[test]
    fn test_simple_letter() {
        let mut e = Engine::new();
        let r = e.on_key(keys::A, false, false);
        assert_eq!(r.action, Action::Send as u8);
        assert_eq!(r.count, 1);
        assert_eq!(char::from_u32(r.chars[0]), Some('a'));
    }

    #[test]
    fn test_uppercase_letter() {
        let mut e = Engine::new();
        let r = e.on_key(keys::A, true, false);
        assert_eq!(r.action, Action::Send as u8);
        assert_eq!(r.count, 1);
        assert_eq!(char::from_u32(r.chars[0]), Some('A'));
    }

    #[test]
    fn test_clear() {
        let mut e = Engine::new();
        e.on_key(keys::A, false, false);
        e.clear();
        assert!(e.buffer.is_empty());
        assert!(e.prev_output.is_empty());
    }

    #[test]
    fn test_set_method() {
        let mut e = Engine::new();
        e.set_method(1);
        assert_eq!(e.method, Method::Vni);
        e.set_method(0);
        assert_eq!(e.method, Method::Telex);
    }

    #[test]
    fn test_get_buffer_string() {
        let mut e = Engine::new();
        e.on_key(keys::A, false, false);
        e.on_key(keys::B, false, false);
        assert_eq!(e.get_buffer_string(), "ab");
    }

    #[test]
    fn test_result_none() {
        let r = Result::none();
        assert_eq!(r.action, 0);
        assert_eq!(r.backspace, 0);
        assert_eq!(r.count, 0);
    }

    #[test]
    fn test_result_send() {
        let chars = vec!['a', 'b', 'c'];
        let r = Result::send(2, &chars);
        assert_eq!(r.action, 1);
        assert_eq!(r.backspace, 2);
        assert_eq!(r.count, 3);
        assert_eq!(char::from_u32(r.chars[0]), Some('a'));
        assert_eq!(char::from_u32(r.chars[1]), Some('b'));
        assert_eq!(char::from_u32(r.chars[2]), Some('c'));
    }

    #[test]
    fn test_result_key_consumed() {
        let r = Result::send_consumed(0, &['a']);
        assert!(r.key_consumed());

        let r2 = Result::send(0, &['a']);
        assert!(!r2.key_consumed());
    }

    #[test]
    fn test_restore_word() {
        let mut e = Engine::new();
        e.restore_word("việt");
        assert_eq!(e.get_buffer_string(), "việt");
    }

    #[test]
    fn test_foreign_mode() {
        let mut e = Engine::new();
        // 'f' is foreign initial
        let r = e.on_key(keys::F, false, false);
        assert_eq!(r.action, Action::Send as u8);
        assert_eq!(char::from_u32(r.chars[0]), Some('f'));
    }
}
