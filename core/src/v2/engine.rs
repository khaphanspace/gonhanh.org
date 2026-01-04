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
    bracket_shortcut: bool,
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
            bracket_shortcut: false,
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
                // Passthrough: don't push to buffer, let system handle it
                // Clear buffer on passthrough to reset word context
                self.clear();
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

        // NOTE: Do NOT restore immediately on Impossible during letter typing!
        // The user might be in the middle of typing a valid word.
        // Example: "đưo" is temporarily Impossible, but becomes "đươ" after next 'w'.
        // Restore only happens on terminators (space, punctuation) in handle_terminator.

        self.rebuild_output()
    }

    /// Handle tone key (Telex: s,f,r,x,j or VNI: 1-5)
    fn handle_tone(&mut self, tone_key: u8) -> Result {
        use super::bitmask::get_tone;
        use super::placement::{
            apply_tone_to_vowel, extract_vowels, find_tone_position, get_base_vowel,
            has_final_consonant, replace_char_at, telex_key_to_tone,
        };

        // Convert tone key to tone value (1-5)
        let tone_value = match self.method {
            Method::Telex => telex_key_to_tone(tone_key),
            Method::Vni => tone_key, // VNI already uses 1-5
        };

        let raw_char = tone_key as char;

        // Get current transformed text (BEFORE pushing raw) - clone to avoid borrow issues
        let transformed = self.buffer.transformed().to_string();

        // Find vowels in transformed text
        let vowels = extract_vowels(&transformed);

        if vowels.is_empty() {
            // No vowels - just pass through the key
            self.buffer.push_raw(raw_char);
            self.buffer.push_transformed(raw_char);
            return self.rebuild_output();
        }

        // Check if buffer already has a toned vowel
        let existing_toned: Option<(usize, char, u8)> =
            transformed.chars().enumerate().find_map(|(i, c)| {
                let tone = get_tone(c);
                if tone > 0 {
                    Some((i, c, tone))
                } else {
                    None
                }
            });

        if let Some((toned_pos, toned_char, existing_tone)) = existing_toned {
            // Already has a tone - check if we should revert or ignore
            let chars_after_tone = transformed.chars().count() - toned_pos - 1;

            if existing_tone == tone_value && chars_after_tone == 0 {
                // Same tone key pressed immediately after toned vowel → REVERT
                self.buffer.push_raw(raw_char);
                let base = get_base_vowel(toned_char);
                let new_transformed = replace_char_at(&transformed, toned_pos, base);
                self.buffer.replace_transformed(&new_transformed);
                self.state.set_has_tone(false);

                // Update vn_state
                let vn_state = validate_vn(self.buffer.transformed());
                self.state.set_vn_state(vn_state);

                return self.rebuild_output();
            }

            // Otherwise: ignore tone key - consume without effect
            // Don't push to raw, don't change transformed
            // Return consumed result (key handled, no output)
            return Result::send(0, &[]);
        }

        // No existing tone - apply new tone using normal rules
        self.buffer.push_raw(raw_char);

        let has_final = has_final_consonant(&transformed);
        let pos = find_tone_position(&vowels, has_final);

        if let Some(target_pos) = pos {
            // Get the vowel at target position
            if let Some(vowel) = transformed.chars().nth(target_pos) {
                // Apply tone to the vowel
                let toned_vowel = apply_tone_to_vowel(vowel, tone_value);

                // Replace the vowel in transformed buffer
                let new_transformed = replace_char_at(&transformed, target_pos, toned_vowel);
                self.buffer.replace_transformed(&new_transformed);

                self.state.set_has_tone(true);
                self.state.set_had_transform(true);

                // Update vn_state after transform
                let vn_state = validate_vn(self.buffer.transformed());
                self.state.set_vn_state(vn_state);
            }
        } else {
            // No valid position - pass through the key
            self.buffer.push_transformed(raw_char);
        }

        self.rebuild_output()
    }

    /// Handle mark key (circumflex, horn, breve, stroke)
    fn handle_mark(&mut self, mark: MarkType) -> Result {
        use super::placement::{
            apply_breve, apply_circumflex, apply_horn, apply_stroke, replace_char_at,
        };

        // Determine raw key for buffer
        let raw_char = match mark {
            MarkType::Stroke => 'd',
            MarkType::Circumflex => {
                // Get the doubled vowel
                self.prev_key.map(|k| k as char).unwrap_or('a')
            }
            MarkType::HornOrBreve => 'w',
        };
        self.buffer.push_raw(raw_char);

        // Get current transformed text
        let transformed = self.buffer.transformed();

        // Apply the appropriate mark
        let result = match mark {
            MarkType::Stroke => apply_stroke(transformed),
            MarkType::Circumflex => apply_circumflex(transformed),
            MarkType::HornOrBreve => {
                // Try horn first (o→ơ, u→ư), then breve (a→ă)
                apply_horn(transformed).or_else(|| apply_breve(transformed))
            }
        };

        if let Some((pos, new_char)) = result {
            // Replace the character at position
            let new_transformed = replace_char_at(transformed, pos, new_char);
            self.buffer.replace_transformed(&new_transformed);

            if mark == MarkType::Stroke {
                self.state.set_has_stroke(true);
            } else {
                self.state.set_has_mark(true);
            }
            self.state.set_had_transform(true);

            // Update vn_state after transform
            let vn_state = validate_vn(self.buffer.transformed());
            self.state.set_vn_state(vn_state);
        } else {
            // No valid position for mark - just pass through
            self.buffer.push_transformed(raw_char);
        }

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

        // Clear buffer for next word
        self.clear();

        // V1 compatibility: Only append space if restore happened (has chars to send)
        // For other terminators (punctuation), never append - they pass through
        // This matches V1's try_auto_restore_on_space/break behavior
        if let Some(' ') = ascii {
            // Only include space if we're doing a restore (result has content)
            if result.count > 0 {
                let mut chars: Vec<char> = (0..result.count as usize)
                    .filter_map(|i| char::from_u32(result.chars[i]))
                    .collect();
                chars.push(' ');
                return Result::send(result.backspace, &chars);
            }
        }
        // For non-space terminators: don't append, let them pass through

        result
    }

    /// Handle special keys (backspace/delete, esc)
    fn handle_special_key(&mut self, key: u16) -> Result {
        match key {
            keys::DELETE => {
                // Pop from both buffers
                self.buffer.pop_raw();
                self.buffer.pop_transformed();

                // CRITICAL: Also update prev_output to stay in sync!
                // Otherwise, subsequent rebuild_output() will compute wrong diff
                // because prev_output still has the deleted character.
                self.prev_output.pop();

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

        // Return none if no diff (raw == transformed already)
        if backspaces == 0 && commit.is_empty() {
            return Result::none();
        }

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

    /// Set bracket shortcuts: ] → ư, [ → ơ
    pub fn set_bracket_shortcut(&mut self, enabled: bool) {
        self.bracket_shortcut = enabled;
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
fn key_to_ascii(key: u16, caps: bool, _shift: bool) -> Option<char> {
    // Match V1 behavior: Swift includes shift in caps (caps = shift || alphaShift)
    // So just use caps directly for letter case, like V1's key_to_char_ext does
    utils::key_to_char(key, caps)
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

    #[test]
    fn test_chafo_step_by_step() {
        let mut e = Engine::new();
        let mut screen = String::new();

        // Helper to apply result to screen
        // For passthrough (action=None), the calling code adds the raw character
        fn apply(screen: &mut String, r: &Result, raw_char: char) -> String {
            if r.action != Action::Send as u8 {
                // Passthrough: caller adds the raw character
                screen.push(raw_char);
                return format!("(none) → added '{}'", raw_char);
            }
            for _ in 0..r.backspace {
                screen.pop();
            }
            let chars: String = (0..r.count as usize)
                .filter_map(|i| char::from_u32(r.chars[i]))
                .collect();
            screen.push_str(&chars);
            format!("bs={}, chars='{}', screen='{}'", r.backspace, chars, screen)
        }

        // Type "chafo "
        let r = e.on_key(keys::C, false, false);
        println!("c: {}", apply(&mut screen, &r, 'c'));

        let r = e.on_key(keys::H, false, false);
        println!("h: {}", apply(&mut screen, &r, 'h'));

        let r = e.on_key(keys::A, false, false);
        println!("a: {}", apply(&mut screen, &r, 'a'));

        let r = e.on_key(keys::F, false, false);
        println!("f: {}", apply(&mut screen, &r, 'f'));

        let r = e.on_key(keys::O, false, false);
        println!("o: {}", apply(&mut screen, &r, 'o'));

        let r = e.on_key(keys::SPACE, false, false);
        println!("space: {}", apply(&mut screen, &r, ' '));

        assert_eq!(screen, "chào ");
    }

    #[test]
    fn test_tieengs_step_by_step() {
        let mut e = Engine::new();
        let mut screen = String::new();

        fn apply(screen: &mut String, r: &Result) -> String {
            if r.action != Action::Send as u8 {
                return "(none)".to_string();
            }
            for _ in 0..r.backspace {
                screen.pop();
            }
            let chars: String = (0..r.count as usize)
                .filter_map(|i| char::from_u32(r.chars[i]))
                .collect();
            screen.push_str(&chars);
            format!("bs={}, chars='{}', screen='{}'", r.backspace, chars, screen)
        }

        // Type "tieengs" (tiếng)
        for &(key, label) in &[
            (keys::T, "t"),
            (keys::I, "i"),
            (keys::E, "e"),
            (keys::E, "e2"),
            (keys::N, "n"),
            (keys::G, "g"),
            (keys::S, "s"),
        ] {
            let r = e.on_key(key, false, false);
            println!("{}: {}", label, apply(&mut screen, &r));
        }

        assert_eq!(screen, "tiếng");
    }

    #[test]
    fn test_debug_tieng() {
        use crate::v2::placement::{
            apply_tone_to_vowel, extract_vowels, find_tone_position, has_final_consonant, is_vowel,
        };

        let s = "tiêng";
        println!("Input: '{}'", s);

        for (i, c) in s.chars().enumerate() {
            println!(
                "  char[{}] = '{}' (U+{:04X}) is_vowel={}",
                i,
                c,
                c as u32,
                is_vowel(c)
            );
        }

        let vowels = extract_vowels(s);
        println!("Vowels: {:?}", vowels);

        let has_final = has_final_consonant(s);
        println!("has_final: {}", has_final);

        let pos = find_tone_position(&vowels, has_final);
        println!("Tone position: {:?}", pos);

        if let Some(target_pos) = pos {
            if let Some(vowel) = s.chars().nth(target_pos) {
                println!(
                    "Vowel at position {}: '{}' (U+{:04X})",
                    target_pos, vowel, vowel as u32
                );
                let toned = apply_tone_to_vowel(vowel, 1);
                println!(
                    "apply_tone_to_vowel('{}', 1) = '{}' (U+{:04X})",
                    vowel, toned, toned as u32
                );
            }
        }
    }

    #[test]
    fn test_debug_char_to_key() {
        // Debug: trace what char_to_key returns vs direct key constants
        let input = "tieengs ";
        let mut e = Engine::new();
        let mut screen = String::new();

        for c in input.chars() {
            let key = crate::utils::char_to_key(c);
            let caps = c.is_uppercase();

            // Debug: compare with expected key constant
            let expected_key = match c {
                't' => keys::T,
                'i' => keys::I,
                'e' => keys::E,
                'n' => keys::N,
                'g' => keys::G,
                's' => keys::S,
                ' ' => keys::SPACE,
                _ => 255,
            };

            println!(
                "char='{}': char_to_key={}, expected={}, match={}",
                c,
                key,
                expected_key,
                key == expected_key
            );

            let r = e.on_key(key, caps, false);
            println!(
                "  -> action={}, backspace={}, count={}, chars={:?}",
                r.action,
                r.backspace,
                r.count,
                (0..r.count as usize)
                    .filter_map(|i| char::from_u32(r.chars[i]))
                    .collect::<String>()
            );

            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        screen.push(ch);
                    }
                }
            } else {
                println!("  -> action != Send, adding raw char");
                screen.push(c);
            }
            println!("  -> screen='{}'", screen);
        }

        println!("Final screen: '{}'", screen);
        assert_eq!(screen, "tiếng ");
    }

    #[test]
    fn test_v2_comprehensive() {
        // Helper to simulate typing and get screen result
        fn type_and_get(input: &str) -> String {
            let mut e = Engine::new();
            let mut screen = String::new();

            for c in input.chars() {
                let key = crate::utils::char_to_key(c);
                let caps = c.is_uppercase();
                let r = e.on_key(key, caps, false);

                if r.action == Action::Send as u8 {
                    for _ in 0..r.backspace {
                        screen.pop();
                    }
                    for i in 0..r.count as usize {
                        if let Some(ch) = char::from_u32(r.chars[i]) {
                            screen.push(ch);
                        }
                    }
                } else {
                    // Pass through if not handled
                    screen.push(c);
                }
            }
            screen
        }

        // Test cases from debug script
        assert_eq!(type_and_get("xin chafo "), "xin chào ");
        assert_eq!(type_and_get("tieengs "), "tiếng ");
        assert_eq!(type_and_get("Vieetj "), "Việt ");
        assert_eq!(type_and_get("tooi ddi work "), "tôi đi work ");
        // ddeef: dd→đ, ee→ê, f→huyền = đề (not để which needs r=hỏi)
        assert_eq!(type_and_get("ddeef "), "đề ");
        assert_eq!(type_and_get("ddeer "), "để "); // r = hỏi tone
        assert_eq!(type_and_get("dduwowcj "), "được ");
    }

    #[test]
    fn test_debug_duoc() {
        // Step-by-step debug for "dduwowcj "
        let input = "dduwowcj ";
        let mut e = Engine::new();
        let mut screen = String::new();

        for c in input.chars() {
            let key = crate::utils::char_to_key(c);
            let caps = c.is_uppercase();
            let prev = screen.clone();
            let r = e.on_key(key, caps, false);

            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                let mut chars_str = String::new();
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        chars_str.push(ch);
                        screen.push(ch);
                    }
                }
                println!(
                    "'{}'(key={}) → backspace={}, chars='{}' → '{}'→'{}'",
                    c, key, r.backspace, chars_str, prev, screen
                );
            } else {
                screen.push(c);
                println!(
                    "'{}'(key={}) → passthrough → '{}'→'{}'",
                    c, key, prev, screen
                );
            }
        }

        println!("Final: '{}'", screen);
        assert_eq!(screen, "được ");
    }

    #[test]
    fn test_caps_first_letter_dd() {
        // Simulate TextEdit auto-capitalize: first D is uppercase (caps=true)
        use crate::data::keys;
        let mut e = Engine::new();
        let mut screen = String::new();

        // Helper to apply result
        fn apply(screen: &mut String, r: &Result, raw: char) {
            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(c) = char::from_u32(r.chars[i]) {
                        screen.push(c);
                    }
                }
            } else {
                screen.push(raw);
            }
        }

        // First 'D' with caps=true (TextEdit auto-capitalize)
        let r1 = e.on_key(keys::D, true, false);
        println!(
            "'D' (caps=true): action={}, bs={}, count={}",
            r1.action, r1.backspace, r1.count
        );
        apply(&mut screen, &r1, 'D');
        println!("  Screen after D: '{}'", screen);

        // Second 'd' with caps=false
        let r2 = e.on_key(keys::D, false, false);
        println!(
            "'d' (caps=false): action={}, bs={}, count={}",
            r2.action, r2.backspace, r2.count
        );
        apply(&mut screen, &r2, 'd');
        println!("  Screen after Dd: '{}'", screen);

        // Expected: 'Đ' (uppercase đ because first letter was caps)
        assert_eq!(screen, "Đ", "Expected 'Dd' -> 'Đ' (uppercase)");
    }
}

#[test]
fn test_doubled_keys() {
    // Simulate doubled keystrokes like the test script produces
    fn type_with_doubles(input: &str) -> String {
        let mut e = Engine::new();
        let mut screen = String::new();

        for c in input.chars() {
            // Each key is sent twice (like CGEventPost issue)
            for _ in 0..2 {
                let key = crate::utils::char_to_key(c);
                let caps = c.is_uppercase();
                let r = e.on_key(key, caps, false);

                if r.action == Action::Send as u8 {
                    for _ in 0..r.backspace {
                        screen.pop();
                    }
                    for i in 0..r.count as usize {
                        if let Some(ch) = char::from_u32(r.chars[i]) {
                            screen.push(ch);
                        }
                    }
                } else {
                    screen.push(c);
                }
            }
        }
        screen
    }

    println!("Testing doubled keys:");
    let result = type_with_doubles("dduwowcj ");
    println!("'dduwowcj ' (doubled) → '{}'", result);

    let result2 = type_with_doubles("tieengs ");
    println!("'tieengs ' (doubled) → '{}'", result2);
}

#[test]
fn test_debug_roofif() {
    // Debug case 8: " roofif" → " rồi"
    let input = " roofif";
    let mut e = Engine::new();
    let mut screen = String::new();

    for c in input.chars() {
        let key = crate::utils::char_to_key(c);
        let caps = c.is_uppercase();
        let prev = screen.clone();
        let r = e.on_key(key, caps, false);

        if r.action == Action::Send as u8 {
            for _ in 0..r.backspace {
                screen.pop();
            }
            let mut chars_str = String::new();
            for i in 0..r.count as usize {
                if let Some(ch) = char::from_u32(r.chars[i]) {
                    chars_str.push(ch);
                    screen.push(ch);
                }
            }
            println!(
                "'{}'(key={}) → bs={}, chars='{}' | '{}'→'{}'",
                c, key, r.backspace, chars_str, prev, screen
            );
        } else {
            screen.push(c);
            println!(
                "'{}'(key={}) → passthrough | '{}'→'{}'",
                c, key, prev, screen
            );
        }
    }

    println!("Final: '{}'", screen);
    assert_eq!(screen, " rồi");
}

#[test]
fn test_debug_test_word() {
    // Debug: trace "Test " typing step by step
    let mut e = Engine::new();
    let mut screen = String::new();

    let input = "Test ";
    for c in input.chars() {
        let key = crate::utils::char_to_key(c);
        let caps = c.is_uppercase();
        let prev = screen.clone();
        let r = e.on_key(key, caps, false);

        if r.action == Action::Send as u8 {
            for _ in 0..r.backspace {
                screen.pop();
            }
            let mut chars_str = String::new();
            for i in 0..r.count as usize {
                if let Some(ch) = char::from_u32(r.chars[i]) {
                    chars_str.push(ch);
                    screen.push(ch);
                }
            }
            println!(
                "'{}'(key={}, caps={}) → bs={}, chars='{}' | '{}'→'{}' | raw='{}' transformed='{}'",
                c,
                key,
                caps,
                r.backspace,
                chars_str,
                prev,
                screen,
                e.buffer.raw(),
                e.buffer.transformed()
            );
        } else {
            screen.push(c);
            println!(
                "'{}'(key={}, caps={}) → passthrough | '{}'→'{}' | raw='{}' transformed='{}'",
                c,
                key,
                caps,
                prev,
                screen,
                e.buffer.raw(),
                e.buffer.transformed()
            );
        }
    }

    println!("Final screen: '{}'", screen);
    assert_eq!(screen, "Test ");
}

#[test]
fn test_e2e_all_cases() {
    // Test all E2E cases with single keystrokes
    fn type_input(input: &str) -> String {
        let mut e = Engine::new();
        let mut screen = String::new();

        for c in input.chars() {
            let key = crate::utils::char_to_key(c);
            let caps = c.is_uppercase();
            let r = e.on_key(key, caps, false);

            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        screen.push(ch);
                    }
                }
            } else {
                screen.push(c);
            }
        }
        screen
    }

    let cases = [
        ("xin chafo ", "xin chào "),
        ("tieengs Vieetj ", "tiếng Việt "),
        ("text ", "text "),
        ("window ", "window "),
        ("view ", "view "),
        ("tooi ddi work ", "tôi đi work "),
        ("saii", "saii"), // No Telex transform for 'ii', outputs as-is
        (" roofif", " rồi"),
        ("ddeer ", "để "), // 'r' = hỏi tone, not 'f'
        ("dduwowcj ", "được "),
        ("law ", "law "),
        ("saw ", "saw "),
        ("raw ", "raw "),
    ];

    let mut all_pass = true;
    for (input, expected) in cases {
        let result = type_input(input);
        let pass = result == expected;
        println!(
            "[{}] '{}' → '{}' (expected '{}')",
            if pass { "✓" } else { "✗" },
            input,
            result,
            expected
        );
        if !pass {
            all_pass = false;
        }
    }

    assert!(all_pass, "Some E2E cases failed");
}
