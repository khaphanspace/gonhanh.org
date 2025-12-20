//! Vietnamese IME Engine
//!
//! Core engine for Vietnamese input method processing.
//! Uses pattern-based transformation with validation-first approach.
//!
//! ## Architecture
//!
//! 1. **Validation First**: Check if buffer is valid Vietnamese before transforming
//! 2. **Pattern-Based**: Scan entire buffer for patterns instead of case-by-case
//! 3. **Shortcut Support**: User-defined abbreviations with priority
//! 4. **Longest-Match-First**: For diacritic placement

pub mod buffer;
pub mod shortcut;
pub mod syllable;
pub mod transform;
pub mod validation;

use crate::data::{
    chars::{self, mark, tone},
    constants, keys,
    vowel::{Phonology, Vowel},
};
use crate::input::{self, ToneType};
use crate::utils;
use buffer::{Buffer, Char, MAX};
use shortcut::{InputMethod, ShortcutTable};
use validation::{is_foreign_word_pattern, is_valid, is_valid_for_transform, is_valid_with_tones};

/// Engine action result
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    None = 0,
    Send = 1,
    Restore = 2,
}

/// Result for FFI
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
}

/// Transform type for revert tracking
#[derive(Clone, Copy, Debug, PartialEq)]
enum Transform {
    Mark(u16, u8),
    Tone(u16, u8),
    Stroke(u16),
    /// Short-pattern stroke (d + vowel + d → đ + vowel)
    /// This is revertible if next character creates invalid Vietnamese
    ShortPatternStroke,
    /// W as vowel ư (for revert: ww → w)
    WAsVowel,
    /// W shortcut was explicitly skipped (prevent re-transformation)
    WShortcutSkipped,
}

/// Word history ring buffer capacity (stores last N committed words)
const HISTORY_CAPACITY: usize = 10;

/// Ring buffer for word history (stack-allocated, O(1) push/pop)
///
/// Used for backspace-after-space feature: when user presses backspace
/// immediately after committing a word with space, restore the previous
/// buffer state to allow editing.
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

    /// Push buffer to history (overwrites oldest if full)
    fn push(&mut self, buf: Buffer) {
        self.data[self.head] = buf;
        self.head = (self.head + 1) % HISTORY_CAPACITY;
        if self.len < HISTORY_CAPACITY {
            self.len += 1;
        }
    }

    /// Pop most recent buffer from history
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

/// Main Vietnamese IME engine
pub struct Engine {
    buf: Buffer,
    method: u8,
    enabled: bool,
    last_transform: Option<Transform>,
    shortcuts: ShortcutTable,
    /// Raw keystroke history for ESC restore (key, caps, shift)
    raw_input: Vec<(u16, bool, bool)>,
    /// True if current word has non-letter characters before letters
    /// Used to prevent false shortcut matches (e.g., "149k" should not match "k")
    has_non_letter_prefix: bool,
    /// Skip w→ư shortcut in Telex mode (user preference)
    /// When true, typing 'w' at word start stays as 'w' instead of converting to 'ư'
    skip_w_shortcut: bool,
    /// Enable ESC key to restore raw ASCII (undo Vietnamese transforms)
    /// When false, ESC key is passed through without restoration
    esc_restore_enabled: bool,
    /// Enable free tone placement (skip validation)
    /// When true, allows placing diacritics anywhere without spelling validation
    free_tone_enabled: bool,
    /// Use modern orthography for tone placement (hoà vs hòa)
    /// When true: oà, uý (tone on second vowel)
    /// When false: òa, úy (tone on first vowel - traditional)
    modern_tone: bool,
    /// Enable English auto-restore (experimental)
    /// When true, automatically restores English words that were transformed
    /// e.g., "tẽt" → "text", "ễpct" → "expect"
    english_auto_restore: bool,
    /// Word history for backspace-after-space feature
    word_history: WordHistory,
    /// Number of spaces typed after committing a word (for backspace tracking)
    /// When this reaches 0 on backspace, we restore the committed word
    spaces_after_commit: u8,
    /// Pending breve position: position of 'a' that has deferred breve
    /// Breve on 'a' in open syllables (like "raw") is invalid Vietnamese
    /// We defer applying breve until a valid final consonant is typed
    pending_breve_pos: Option<usize>,
    /// Tracks if stroke was reverted in current word (ddd → dd)
    /// When true, subsequent 'd' keys are treated as normal letters, not stroke triggers
    /// This prevents "ddddd" from oscillating between đ and dd states
    stroke_reverted: bool,
    /// Tracks if a mark was reverted in current word
    /// Used by auto-restore to detect words like "issue", "bass" that need restoration
    had_mark_revert: bool,
    /// Pending pop from raw_input after mark revert
    /// When true, the NEXT consonant key will trigger a pop to remove the consumed modifier
    /// This differentiates: "tesst" → "test" (consonant after) vs "issue" → "issue" (vowel after)
    pending_mark_revert_pop: bool,
    /// Tracks if ANY Vietnamese transform was ever applied during this word
    /// (marks, tones, or stroke). Used to prevent false auto-restore for words
    /// with numbers/symbols that never had Vietnamese transforms applied.
    /// Example: "nhatkha1407@gmail.com" has no transforms, so shouldn't restore.
    had_any_transform: bool,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            buf: Buffer::new(),
            method: 0,
            enabled: true,
            last_transform: None,
            shortcuts: ShortcutTable::with_defaults(),
            raw_input: Vec::with_capacity(64),
            has_non_letter_prefix: false,
            skip_w_shortcut: false,
            esc_restore_enabled: false, // Default: OFF (user request)
            free_tone_enabled: false,
            modern_tone: true,           // Default: modern style (hoà, thuý)
            english_auto_restore: false, // Default: OFF (experimental feature)
            word_history: WordHistory::new(),
            spaces_after_commit: 0,
            pending_breve_pos: None,
            stroke_reverted: false,
            had_mark_revert: false,
            pending_mark_revert_pop: false,
            had_any_transform: false,
        }
    }

    pub fn set_method(&mut self, method: u8) {
        self.method = method;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.buf.clear();
            self.word_history.clear();
            self.spaces_after_commit = 0;
        }
    }

    /// Set whether to skip w→ư shortcut in Telex mode
    pub fn set_skip_w_shortcut(&mut self, skip: bool) {
        self.skip_w_shortcut = skip;
    }

    /// Set whether ESC key restores raw ASCII
    pub fn set_esc_restore(&mut self, enabled: bool) {
        self.esc_restore_enabled = enabled;
    }

    /// Set whether to enable free tone placement (skip validation)
    pub fn set_free_tone(&mut self, enabled: bool) {
        self.free_tone_enabled = enabled;
    }

    /// Set whether to use modern orthography for tone placement
    pub fn set_modern_tone(&mut self, modern: bool) {
        self.modern_tone = modern;
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

    /// Get current input method as InputMethod enum
    fn current_input_method(&self) -> InputMethod {
        match self.method {
            0 => InputMethod::Telex,
            1 => InputMethod::Vni,
            _ => InputMethod::All,
        }
    }

    /// Handle key event - main entry point
    ///
    /// # Arguments
    /// * `key` - macOS virtual keycode
    /// * `caps` - true if Caps Lock is active (for uppercase letters)
    /// * `ctrl` - true if Cmd/Ctrl/Alt is pressed (bypasses IME)
    pub fn on_key(&mut self, key: u16, caps: bool, ctrl: bool) -> Result {
        self.on_key_ext(key, caps, ctrl, false)
    }

    /// Check if key+shift combo is a raw mode prefix character
    /// Raw prefixes: @ # : /
    #[allow(dead_code)] // TEMP DISABLED
    fn is_raw_prefix(key: u16, shift: bool) -> bool {
        // / doesn't need shift
        if key == keys::SLASH && !shift {
            return true;
        }
        // @ # : need shift
        if !shift {
            return false;
        }
        matches!(
            key,
            keys::N2              // @ = Shift+2
                | keys::N3        // # = Shift+3
                | keys::SEMICOLON // : = Shift+;
        )
    }

    /// Handle key event with extended parameters
    ///
    /// # Arguments
    /// * `key` - macOS virtual keycode
    /// * `caps` - true if Caps Lock is active (for uppercase letters)
    /// * `ctrl` - true if Cmd/Ctrl/Alt is pressed (bypasses IME)
    /// * `shift` - true if Shift key is pressed (for symbols like @, #, $)
    pub fn on_key_ext(&mut self, key: u16, caps: bool, ctrl: bool, shift: bool) -> Result {
        if !self.enabled || ctrl {
            self.clear();
            self.word_history.clear();
            self.spaces_after_commit = 0;
            return Result::none();
        }

        // Check for word boundary shortcuts ONLY on SPACE
        // Also auto-restore invalid Vietnamese to raw English
        if key == keys::SPACE {
            // First check for shortcut
            let shortcut_result = self.try_word_boundary_shortcut();
            if shortcut_result.action != 0 {
                self.clear();
                return shortcut_result;
            }

            // Auto-restore: if buffer has transforms but is invalid Vietnamese,
            // restore to raw English (like ESC but triggered by space)
            let restore_result = self.try_auto_restore_on_space();

            // If auto-restore happened, repopulate buffer with plain chars from raw_input
            // This ensures word_history stores the correct restored word (not transformed)
            // Example: "restore" → buffer was "rếtore" (6 chars), raw_input has 7 keys
            // After this, buffer has "restore" (7 chars) for correct history
            if restore_result.action != 0 {
                self.buf.clear();
                for &(key, caps, _) in &self.raw_input {
                    self.buf.push(Char::new(key, caps));
                }
            }

            // Push buffer to history before clearing (for backspace-after-space feature)
            if !self.buf.is_empty() {
                self.word_history.push(self.buf.clone());
                self.spaces_after_commit = 1; // First space after word
            } else if self.spaces_after_commit > 0 {
                // Additional space after commit - increment counter
                self.spaces_after_commit = self.spaces_after_commit.saturating_add(1);
            }
            self.clear();
            return restore_result;
        }

        // ESC key: restore to raw ASCII (undo all Vietnamese transforms)
        // Only if esc_restore is enabled by user
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

        // Other break keys (punctuation, arrows, etc.)
        // Also trigger auto-restore for invalid Vietnamese before clearing
        if keys::is_break(key) {
            let restore_result = self.try_auto_restore_on_break();
            self.clear();
            self.word_history.clear();
            self.spaces_after_commit = 0;
            return restore_result;
        }

        if key == keys::DELETE {
            // Backspace-after-space feature: restore previous word when all spaces deleted
            // Track spaces typed after commit, restore word when counter reaches 0
            if self.spaces_after_commit > 0 && self.buf.is_empty() {
                self.spaces_after_commit -= 1;
                if self.spaces_after_commit == 0 {
                    // All spaces deleted - restore the word buffer
                    if let Some(restored_buf) = self.word_history.pop() {
                        // Restore raw_input from buffer (for ESC restore to work)
                        self.restore_raw_input_from_buffer(&restored_buf);
                        self.buf = restored_buf;
                    }
                }
                // Delete one space
                return Result::send(1, &[]);
            }
            // DON'T reset spaces_after_commit here!
            // User might delete all new input and want to restore previous word.
            // Reset only happens on: break keys, ESC, ctrl, or new commit.

            // If buffer is already empty, user is deleting content from previous word
            // that we don't track. Mark this to prevent false shortcut matches.
            // e.g., "đa" + SPACE + backspace×2 + "a" should NOT match shortcut "a"
            if self.buf.is_empty() {
                self.has_non_letter_prefix = true;
            }
            self.buf.pop();
            self.raw_input.pop();
            self.last_transform = None;
            // Reset stroke_reverted on backspace so user can re-trigger stroke
            // e.g., "ddddd" → "dddd", then backspace×3 → "d", then "d" → "đ"
            self.stroke_reverted = false;
            return Result::none();
        }

        // Record raw keystroke for ESC restore (letters and numbers only)
        if keys::is_letter(key) || keys::is_number(key) {
            self.raw_input.push((key, caps, shift));
        }

        self.process(key, caps, shift)
    }

    /// Main processing pipeline - pattern-based
    fn process(&mut self, key: u16, caps: bool, shift: bool) -> Result {
        let m = input::get(self.method);

        // Handle pending mark revert pop: if previous key was a mark revert (like "ss"),
        // and THIS key is a consonant, pop the consumed modifier from raw_input.
        // This differentiates:
        // - "tesst" → 't' is consonant → pop → raw becomes [t,e,s,t] → "test"
        // - "issue" → 'u' is vowel → don't pop → raw stays [i,s,s,u,e] → "issue"
        if self.pending_mark_revert_pop && keys::is_letter(key) {
            self.pending_mark_revert_pop = false;
            if keys::is_consonant(key) {
                // Pop the consumed modifier key from raw_input
                // raw_input currently has: [..., mark_key, revert_key, current_key]
                // We want: [..., revert_key, current_key]
                // So we pop current, pop revert, pop mark, push revert, push current
                let current = self.raw_input.pop(); // current key (just added)
                let revert = self.raw_input.pop(); // revert key
                self.raw_input.pop(); // mark key (consumed, discard)
                if let Some(r) = revert {
                    self.raw_input.push(r);
                }
                if let Some(c) = current {
                    self.raw_input.push(c);
                }
            }
        }

        // Revert short-pattern stroke when new letter creates invalid Vietnamese
        // This handles: "ded" → "đe" (stroke applied), then 'i' → "dedi" (invalid, revert)
        // IMPORTANT: This check must happen BEFORE any modifiers (tone, mark, etc.)
        // because the modifier key (like 'e' for circumflex) would transform the
        // buffer before we can check validity.
        //
        // We check validity using raw_input (not self.buf) because:
        // - self.buf = [đ, e] after stroke (2 chars)
        // - raw_input = [d, e, d, e] with new 'e' (4 chars - the actual full input)
        // Checking [D, E, D, E] correctly identifies "dede" as invalid.
        //
        // Skip revert for:
        // - Mark keys (s, f, r, x, j) - confirm Vietnamese intent
        // - Tone keys (a, e, o, w) that can apply to buffer - allows fast typing
        //   e.g., "dod" → "đo" + 'o' → "đô" (user typed d-o-d-o fast, intended "ddoo")
        // - Stroke keys ('d') - handled separately in try_stroke for proper revert behavior
        //   e.g., "dadd" → "dad" (d reverts stroke and adds itself, not "dadd")
        let is_mark_key = m.mark(key).is_some();
        let is_tone_key = m.tone(key).is_some();
        let is_stroke_key = m.stroke(key);

        if keys::is_letter(key)
            && !is_mark_key
            && !is_tone_key
            && !is_stroke_key
            && matches!(self.last_transform, Some(Transform::ShortPatternStroke))
        {
            // Build buffer_keys from raw_input (which already includes current key)
            let buffer_keys: Vec<u16> = self.raw_input.iter().map(|&(k, _, _)| k).collect();
            if !is_valid(&buffer_keys) {
                // Invalid pattern - revert stroke and rebuild from raw_input
                if let Some(raw_chars) = self.build_raw_chars() {
                    // Calculate backspace: screen shows buffer content (e.g., "đe")
                    let backspace = self.buf.len() as u8;

                    // Rebuild buffer from raw_input (plain chars, no stroke)
                    self.buf.clear();
                    for &(k, c, _) in &self.raw_input {
                        self.buf.push(Char::new(k, c));
                    }
                    self.last_transform = None;

                    return Result::send(backspace, &raw_chars);
                }
            }
        }

        // In VNI mode, if Shift is pressed with a number key, skip all modifiers
        // User wants the symbol (@ for Shift+2, # for Shift+3, etc.), not VNI marks
        let skip_vni_modifiers = self.method == 1 && shift && keys::is_number(key);

        // Check modifiers by scanning buffer for patterns

        // 1. Stroke modifier (d → đ)
        if !skip_vni_modifiers && m.stroke(key) {
            if let Some(result) = self.try_stroke(key) {
                return result;
            }
        }

        // 2. Tone modifier (circumflex, horn, breve)
        if !skip_vni_modifiers {
            if let Some(tone_type) = m.tone(key) {
                let targets = m.tone_targets(key);
                if let Some(result) = self.try_tone(key, caps, tone_type, targets) {
                    return result;
                }
            }
        }

        // 3. Mark modifier
        if !skip_vni_modifiers {
            if let Some(mark_val) = m.mark(key) {
                if let Some(result) = self.try_mark(key, caps, mark_val) {
                    return result;
                }
            }
        }

        // 4. Remove modifier
        // Only consume key if there's something to remove; otherwise fall through to normal letter
        // This allows shortcuts like "zz" to work when buffer has no marks/tones to remove
        if !skip_vni_modifiers && m.remove(key) {
            if let Some(result) = self.try_remove() {
                return result;
            }
        }

        // 5. In Telex: "w" as vowel "ư" when valid Vietnamese context
        // Examples: "w" → "ư", "nhw" → "như", but "kw" → "kw" (invalid)
        if self.method == 0 && key == keys::W {
            if let Some(result) = self.try_w_as_vowel(caps) {
                return result;
            }
        }

        // Not a modifier - normal letter
        self.handle_normal_letter(key, caps)
    }

    /// Try word boundary shortcuts (triggered by space, punctuation, etc.)
    fn try_word_boundary_shortcut(&mut self) -> Result {
        if self.buf.is_empty() {
            return Result::none();
        }

        // Don't trigger shortcut if word has non-letter prefix
        // e.g., "149k" should NOT match shortcut "k"
        if self.has_non_letter_prefix {
            return Result::none();
        }

        let buffer_str = self.buf.to_full_string();
        let input_method = self.current_input_method();

        // Check for word boundary shortcut match
        if let Some(m) =
            self.shortcuts
                .try_match_for_method(&buffer_str, Some(' '), true, input_method)
        {
            let output: Vec<char> = m.output.chars().collect();
            return Result::send(m.backspace_count as u8, &output);
        }

        Result::none()
    }

    /// Try "w" as vowel "ư" in Telex mode
    ///
    /// Rules:
    /// - "w" alone → "ư"
    /// - "nhw" → "như" (valid consonant + ư)
    /// - "kw" → "kw" (invalid, k cannot precede ư)
    /// - "ww" → revert to "w" (shortcut skipped)
    /// - "www" → "ww" (subsequent w just adds normally)
    fn try_w_as_vowel(&mut self, caps: bool) -> Option<Result> {
        // Issue #44: If breve is pending (deferred due to open syllable),
        // don't convert w→ư. Let w be added as regular letter.
        // Example: "aw" → breve deferred → should stay "aw", not become "aư"
        if self.pending_breve_pos.is_some() {
            return None;
        }

        // If user disabled w→ư shortcut at word start, only skip when buffer is empty
        // This allows "hw" → "hư" even when shortcut is disabled
        if self.skip_w_shortcut && self.buf.is_empty() {
            return None;
        }

        // If shortcut was previously skipped, don't try again
        if matches!(self.last_transform, Some(Transform::WShortcutSkipped)) {
            return None;
        }

        // If we already have a complete ươ compound, swallow the second 'w'
        // This handles "dduwowcj" where the second 'w' should be no-op
        // Use send(0, []) to intercept and consume the key without output
        if self.has_complete_uo_compound() {
            return Some(Result::send(0, &[]));
        }

        // Check revert: ww → w (skip shortcut)
        // Preserve original case: Ww → W, wW → w
        if let Some(Transform::WAsVowel) = self.last_transform {
            self.last_transform = Some(Transform::WShortcutSkipped);
            // Get original case from buffer before popping
            let original_caps = self.buf.last().map(|c| c.caps).unwrap_or(caps);
            self.buf.pop();
            self.buf.push(Char::new(keys::W, original_caps));
            let w = if original_caps { 'W' } else { 'w' };
            return Some(Result::send(1, &[w]));
        }

        // Try adding U (ư base) to buffer and validate
        self.buf.push(Char::new(keys::U, caps));

        // Set horn tone to make it ư
        if let Some(c) = self.buf.get_mut(self.buf.len() - 1) {
            c.tone = tone::HORN;
        }

        // Validate: is this valid Vietnamese?
        // Use is_valid_with_tones to check modifier requirements (e.g., E+U needs circumflex)
        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
        let buffer_tones: Vec<u8> = self.buf.iter().map(|c| c.tone).collect();
        if is_valid_with_tones(&buffer_keys, &buffer_tones) {
            self.last_transform = Some(Transform::WAsVowel);
            self.had_any_transform = true;

            // W shortcut adds ư without replacing anything on screen
            // (the raw 'w' key was never output, so no backspace needed)
            let vowel_char = chars::to_char(keys::U, caps, tone::HORN, 0).unwrap();
            return Some(Result::send(0, &[vowel_char]));
        }

        // Invalid - remove the U we added
        self.buf.pop();
        None
    }

    /// Try to apply stroke transformation by scanning buffer
    ///
    /// Issue #51: In Telex mode, only apply stroke when the new 'd' is ADJACENT to
    /// an existing 'd'. According to Vietnamese Telex docs (Section 9.2.2), "dd" → "đ"
    /// should only work when the two 'd's are consecutive. For words like "deadline",
    /// the 'd's are separated by "ea", so stroke should NOT apply.
    ///
    /// In VNI mode, '9' is always an intentional stroke command (not a letter), so
    /// delayed stroke is allowed (e.g., "duong9" → "đuong").
    fn try_stroke(&mut self, key: u16) -> Option<Result> {
        // If stroke was already reverted in this word (ddd → dd), skip further stroke attempts
        // This prevents "ddddd" from oscillating and ensures subsequent 'd's are just letters
        if self.stroke_reverted && key == keys::D {
            return None;
        }

        // Check for stroke revert first: ddd → dd
        // If last transform was stroke and same key pressed again, revert the stroke
        if let Some(Transform::Stroke(last_key)) = self.last_transform {
            if last_key == key {
                // Find the stroked 'd' to revert
                if let Some(pos) = self.buf.iter().position(|c| c.key == keys::D && c.stroke) {
                    // Revert: un-stroke the 'd'
                    if let Some(c) = self.buf.get_mut(pos) {
                        c.stroke = false;
                    }
                    // Add another 'd' as normal char
                    self.buf.push(Char::new(key, false));
                    self.last_transform = None;
                    // Mark that stroke was reverted - subsequent 'd' keys will be normal letters
                    self.stroke_reverted = true;
                    // Use rebuild_from_after_insert because the new 'd' was just pushed
                    // and hasn't been displayed on screen yet
                    return Some(self.rebuild_from_after_insert(pos));
                }
            }
        }

        // Check for short-pattern stroke revert: dadd → dad
        // If last transform was short-pattern stroke and 'd' is pressed again, revert the stroke
        // This is similar to the ddd → dd revert above, but for delayed stroke patterns
        if let Some(Transform::ShortPatternStroke) = self.last_transform {
            if key == keys::D {
                // Find the stroked 'd' to revert
                if let Some(pos) = self.buf.iter().position(|c| c.key == keys::D && c.stroke) {
                    // Revert: un-stroke the 'd'
                    if let Some(c) = self.buf.get_mut(pos) {
                        c.stroke = false;
                    }
                    // Add another 'd' as normal char
                    self.buf.push(Char::new(key, false));
                    self.last_transform = None;
                    // Mark that stroke was reverted - subsequent 'd' keys will be normal letters
                    self.stroke_reverted = true;
                    // Use rebuild_from_after_insert because the new 'd' was just pushed
                    // and hasn't been displayed on screen yet
                    return Some(self.rebuild_from_after_insert(pos));
                }
            }
        }

        // Collect buffer keys once for all validations
        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
        let has_vowel = buffer_keys.iter().any(|&k| keys::is_vowel(k));

        // Find position of un-stroked 'd' to apply stroke
        // Also track if this is a short pattern stroke (revertible)
        let (pos, is_short_pattern_stroke) = if self.method == 0 {
            // Telex: First try adjacent 'd' (last char is un-stroked d)
            let last_pos = self.buf.len().checked_sub(1)?;
            let last_char = self.buf.get(last_pos)?;

            if last_char.key == keys::D && !last_char.stroke {
                // Adjacent stroke: "dd" → "đ" (not a short pattern)
                (last_pos, false)
            } else {
                // Delayed stroke: check if initial 'd' can be stroked
                // Only allow if: first char is 'd', has vowel, and forms valid Vietnamese
                let first_char = self.buf.get(0)?;
                if first_char.key != keys::D || first_char.stroke {
                    return None;
                }

                // Must have at least one vowel for delayed stroke
                if !has_vowel {
                    return None;
                }

                // Must form valid Vietnamese (including vowel pattern) for delayed stroke
                // Use is_valid() instead of is_valid_for_transform() to check vowel patterns
                // This prevents "dea" + "d" → "đea" (invalid "ea" diphthong)
                if !is_valid(&buffer_keys) {
                    return None;
                }

                // For open syllables (d + vowel only), defer stroke to try_mark
                // UNLESS:
                // - A mark is already applied (confirms Vietnamese intent)
                // - The triggering key is 'd' AND buffer is short (d + single vowel)
                //   This allows "did" → "đi", "dod" → "đo", etc.
                // This prevents "de" + "d" → "đe" while allowing:
                // - "dods" → "đó" (mark key triggers stroke)
                // - "dojd" → "đọ" (mark already present, stroke applies immediately)
                // - "did" → "đi" (d triggers stroke on short open syllable)
                let syllable = syllable::parse(&buffer_keys);
                let has_mark_applied = self.buf.iter().any(|c| c.mark > 0);
                // Only allow 'd' to trigger immediate stroke on short patterns (d + 1 vowel = 2 chars)
                let is_short_d_pattern = key == keys::D && self.buf.len() == 2;
                if syllable.final_c.is_empty() && !has_mark_applied && !is_short_d_pattern {
                    // Open syllable without mark, not short d pattern - defer stroke decision
                    return None;
                }

                // Track if this is a short pattern stroke (can be reverted later)
                // Only revertible if no mark applied - mark confirms Vietnamese intent
                (0, is_short_d_pattern && !has_mark_applied)
            }
        } else {
            // VNI: Allow delayed stroke - find first un-stroked 'd' anywhere in buffer
            // '9' is always intentional stroke command, not a letter
            let pos = self
                .buf
                .iter()
                .enumerate()
                .find(|(_, c)| c.key == keys::D && !c.stroke)
                .map(|(i, _)| i)?;
            (pos, false) // VNI never uses short pattern stroke
        };

        // Check revert: if last transform was stroke on same key at same position
        if let Some(Transform::Stroke(last_key)) = self.last_transform {
            if last_key == key {
                return Some(self.revert_stroke(key, pos));
            }
        }

        // Validate buffer structure before applying stroke
        // Only validate if buffer has vowels (complete syllable)
        // Allow stroke on initial consonant before vowel is typed (e.g., "dd" → "đ" then "đi")
        // Skip validation if free_tone mode is enabled
        if !self.free_tone_enabled && has_vowel && !is_valid_for_transform(&buffer_keys) {
            return None;
        }

        // Mark as stroked
        if let Some(c) = self.buf.get_mut(pos) {
            c.stroke = true;
        }

        // Track transform type for potential revert
        self.last_transform = if is_short_pattern_stroke {
            Some(Transform::ShortPatternStroke)
        } else {
            Some(Transform::Stroke(key))
        };
        self.had_any_transform = true;
        Some(self.rebuild_from(pos))
    }

    /// Try to apply tone transformation by scanning buffer for targets
    fn try_tone(
        &mut self,
        key: u16,
        caps: bool,
        tone_type: ToneType,
        targets: &[u16],
    ) -> Option<Result> {
        if self.buf.is_empty() {
            return None;
        }

        // Issue #44: Cancel pending breve if same modifier pressed again ("aww" → "aw")
        // When breve was deferred and user presses 'w' again, cancel without adding another 'w'
        if self.pending_breve_pos.is_some()
            && (tone_type == ToneType::Horn || tone_type == ToneType::Breve)
        {
            // Cancel the pending breve - user doesn't want Vietnamese
            self.pending_breve_pos = None;
            // Return "consumed but no change" to prevent 'w' from being typed
            // action=Send with 0 backspace and 0 chars effectively consumes the key
            return Some(Result::send(0, &[]));
        }

        // Check revert first (same key pressed twice)
        if let Some(Transform::Tone(last_key, _)) = self.last_transform {
            if last_key == key {
                return Some(self.revert_tone(key, caps));
            }
        }

        // Validate buffer structure (not vowel patterns - those are checked after transform)
        // Skip validation if free_tone mode is enabled
        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
        if !self.free_tone_enabled && !is_valid_for_transform(&buffer_keys) {
            return None;
        }

        let tone_val = tone_type.value();

        // Check if we're switching from one tone to another (e.g., ô → ơ)
        // Find vowels that have a DIFFERENT tone (to switch) or NO tone (to add)
        let is_switching = self
            .buf
            .iter()
            .any(|c| targets.contains(&c.key) && c.tone != tone::NONE && c.tone != tone_val);

        // Scan buffer for eligible target vowels
        let mut target_positions = Vec::new();

        // Special case: uo/ou compound for horn - find adjacent pair only
        // But ONLY apply compound logic when BOTH vowels are plain (not when switching)
        if tone_type == ToneType::Horn && !is_switching {
            if let Some((pos1, pos2)) = self.find_uo_compound_positions() {
                if let (Some(c1), Some(c2)) = (self.buf.get(pos1), self.buf.get(pos2)) {
                    // Only apply compound when BOTH vowels have no tone
                    if c1.tone == tone::NONE && c2.tone == tone::NONE {
                        target_positions.push(pos1);
                        target_positions.push(pos2);
                    }
                }
            }
        }

        // Normal case: find last matching target
        if target_positions.is_empty() {
            if is_switching {
                // When switching, ONLY target vowels that already have a diacritic
                // (don't add diacritics to plain vowels during switch)
                for (i, c) in self.buf.iter().enumerate().rev() {
                    if targets.contains(&c.key) && c.tone != tone::NONE && c.tone != tone_val {
                        target_positions.push(i);
                        break;
                    }
                }
            } else if tone_type == ToneType::Horn {
                // For horn modifier, apply smart vowel selection based on Vietnamese phonology
                target_positions = self.find_horn_target_with_switch(targets, tone_val);
            } else {
                // Non-horn modifiers (circumflex): use standard target matching
                // For Telex circumflex (aa, ee, oo pattern), require either:
                // 1. Target at LAST position (immediate doubling: "oo" → "ô")
                // 2. No consonants between target and end (delayed diphthong: "oio" → "ôi")
                // This prevents transformation in words like "teacher" where consonants
                // (c, h) appear between the two 'e's
                let is_telex_circumflex = self.method == 0
                    && tone_type == ToneType::Circumflex
                    && matches!(key, keys::A | keys::E | keys::O);

                for (i, c) in self.buf.iter().enumerate().rev() {
                    if targets.contains(&c.key) && c.tone == tone::NONE {
                        // For Telex circumflex, check if there are consonants after target
                        if is_telex_circumflex && i != self.buf.len() - 1 {
                            // Check for consonants between target position and end of buffer
                            let consonants_after: Vec<u16> = (i + 1..self.buf.len())
                                .filter_map(|j| {
                                    self.buf.get(j).and_then(|ch| {
                                        if !keys::is_vowel(ch.key) {
                                            Some(ch.key)
                                        } else {
                                            None
                                        }
                                    })
                                })
                                .collect();

                            if !consonants_after.is_empty() {
                                // Check if there's a vowel between target and final consonants
                                // "teacher": e-a-ch has 'a' between first 'e' and 'ch' → block
                                // "hongo": o-ng has no vowel between 'o' and 'ng' → allow
                                let has_vowel_between = (i + 1..self.buf.len()).any(|j| {
                                    self.buf.get(j).is_some_and(|ch| keys::is_vowel(ch.key))
                                });

                                if has_vowel_between {
                                    // Another vowel between target and end → different syllable
                                    // Skip this target (e.g., "teacher" → don't make "têacher")
                                    continue;
                                }

                                // Check if consonants form valid Vietnamese finals
                                // Valid finals: single (c,m,n,p,t) or pairs (ch,ng,nh)
                                // Double consonant finals (ng,nh,ch) are distinctly Vietnamese
                                // - "hongo" → "hông" (ng final, allow circumflex)
                                // - "khongo" → "không" (ng final, allow circumflex)
                                // Single consonant finals need additional context
                                // - "data" → should NOT become "dât" (t final, but English)
                                // - "nhana" → "nhân" (n final, but has nh initial)
                                let (all_are_valid_finals, is_double_final) = match consonants_after
                                    .len()
                                {
                                    1 => (
                                        constants::VALID_FINALS_1.contains(&consonants_after[0]),
                                        false,
                                    ),
                                    2 => {
                                        let pair = [consonants_after[0], consonants_after[1]];
                                        (constants::VALID_FINALS_2.contains(&pair), true)
                                    }
                                    _ => (false, false), // More than 2 consonants is invalid
                                };

                                // Double consonant finals (ng,nh,ch) are distinctly Vietnamese
                                // Always allow circumflex for these patterns
                                if is_double_final && all_are_valid_finals {
                                    // Valid double final like "ng" - allow circumflex
                                    // This handles "hongo" → "hông", "khongo" → "không"
                                } else if !all_are_valid_finals {
                                    // Invalid final consonants → skip
                                    continue;
                                } else {
                                    // Single consonant final - need diphthong or double initial
                                    // Check if there's another vowel adjacent to target (diphthong)
                                    let has_adjacent_vowel = (i > 0
                                        && self
                                            .buf
                                            .get(i - 1)
                                            .is_some_and(|ch| keys::is_vowel(ch.key)))
                                        || (i + 1 < self.buf.len()
                                            && self
                                                .buf
                                                .get(i + 1)
                                                .is_some_and(|ch| keys::is_vowel(ch.key)));

                                    // Check for Vietnamese-specific double initial (nh, ch, th, ph, etc.)
                                    // This allows "nhana" → "nhân" (nh + a + n + a)
                                    // but still blocks "data" → "dât" (d is not a Vietnamese digraph)
                                    let has_vietnamese_double_initial = if i >= 2 {
                                        // Get first two consonants before the target vowel
                                        let initial_keys: Vec<u16> = (0..i)
                                            .filter_map(|j| self.buf.get(j).map(|ch| ch.key))
                                            .take_while(|k| !keys::is_vowel(*k))
                                            .collect();
                                        if initial_keys.len() >= 2 {
                                            let pair = [initial_keys[0], initial_keys[1]];
                                            constants::VALID_INITIALS_2.contains(&pair)
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    };

                                    if !has_adjacent_vowel && !has_vietnamese_double_initial {
                                        // Single final, no diphthong, no double initial → likely English
                                        continue;
                                    }
                                }
                            }
                        }
                        target_positions.push(i);
                        break;
                    }
                }
            }
        }

        if target_positions.is_empty() {
            // Check if any target vowels already have the requested tone
            // If so, absorb the key (no-op) instead of falling through
            // This handles redundant tone keys like "u7o7" → "ươ" (second 7 absorbed)
            //
            // EXCEPTION: Don't absorb 'w' if last_transform was WAsVowel
            // because try_w_as_vowel needs to handle the revert (ww → w)
            let is_w_revert_pending =
                key == keys::W && matches!(self.last_transform, Some(Transform::WAsVowel));

            let has_tone_already = self
                .buf
                .iter()
                .any(|c| targets.contains(&c.key) && c.tone == tone_val);
            if has_tone_already && !is_w_revert_pending {
                // Return empty Send to absorb key without passthrough
                return Some(Result::send(0, &[]));
            }
            return None;
        }

        // Track earliest position modified for rebuild
        let mut earliest_pos = usize::MAX;

        // If switching, clear old tones first for proper rebuild
        if is_switching {
            for &pos in &target_positions {
                if let Some(c) = self.buf.get_mut(pos) {
                    c.tone = tone::NONE;
                    earliest_pos = earliest_pos.min(pos);
                }
            }

            // Special case: switching from horn compound (ươ) to circumflex (uô)
            // When switching to circumflex on 'o', also clear horn from adjacent 'u'
            if tone_type == ToneType::Circumflex {
                for &pos in &target_positions {
                    if let Some(c) = self.buf.get(pos) {
                        if c.key == keys::O {
                            // Check for adjacent 'u' with horn and clear it
                            if pos > 0 {
                                if let Some(prev) = self.buf.get_mut(pos - 1) {
                                    if prev.key == keys::U && prev.tone == tone::HORN {
                                        prev.tone = tone::NONE;
                                        earliest_pos = earliest_pos.min(pos - 1);
                                    }
                                }
                            }
                            if pos + 1 < self.buf.len() {
                                if let Some(next) = self.buf.get_mut(pos + 1) {
                                    if next.key == keys::U && next.tone == tone::HORN {
                                        next.tone = tone::NONE;
                                        earliest_pos = earliest_pos.min(pos + 1);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Special case: switching from circumflex (uô) to horn compound (ươ)
            // For standalone uo compound (no final consonant), add horn to adjacent 'u'
            if tone_type == ToneType::Horn && self.has_uo_compound() {
                // Check if this is a standalone compound (o is last vowel, no final consonant)
                let has_final = target_positions.iter().any(|&pos| {
                    pos + 1 < self.buf.len()
                        && self
                            .buf
                            .get(pos + 1)
                            .is_some_and(|c| !keys::is_vowel(c.key))
                });

                if !has_final {
                    for &pos in &target_positions {
                        if let Some(c) = self.buf.get(pos) {
                            if c.key == keys::O {
                                // Add horn to adjacent 'u' for compound
                                if pos > 0 {
                                    if let Some(prev) = self.buf.get_mut(pos - 1) {
                                        if prev.key == keys::U && prev.tone == tone::NONE {
                                            prev.tone = tone::HORN;
                                            earliest_pos = earliest_pos.min(pos - 1);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Apply new tone
        for &pos in &target_positions {
            if let Some(c) = self.buf.get_mut(pos) {
                c.tone = tone_val;
                earliest_pos = earliest_pos.min(pos);
            }
        }

        // Validate result: check for breve (ă) followed by vowel - NEVER valid in Vietnamese
        // Issue #44: "tai" + 'w' → "tăi" is INVALID (ăi, ăo, ău, ăy don't exist)
        // Only check this specific pattern, not all vowel patterns, to allow Telex shortcuts
        // like "eie" → "êi" which may not be standard but are expected Telex behavior
        // Note: ToneType::Horn (Telex 'w') and ToneType::Breve (VNI '8') both create breve on 'a'
        if tone_type == ToneType::Horn || tone_type == ToneType::Breve {
            // Early check: "W at end after vowel (not U)" with earlier Vietnamese transforms
            // suggests English word like "seesaw" where:
            // - Earlier chars were transformed (sê, sế)
            // - But "aw" ending makes it look like English
            // Only restore if buffer has EARLIER transforms (tone or mark)
            // Don't restore for simple "aw" or "raw" - let breve deferral handle those
            // Only run if english_auto_restore is enabled (experimental feature)
            if self.english_auto_restore && key == keys::W && self.raw_input.len() >= 2 {
                let (prev_key, _, _) = self.raw_input[self.raw_input.len() - 2];
                if prev_key == keys::A {
                    // Check if there are earlier Vietnamese transforms in buffer
                    // (tone marks on OTHER vowels, or circumflex/horn on non-A vowels)
                    // IMPORTANT: Exclude positions we just modified in this call
                    let has_earlier_transforms = self.buf.iter().enumerate().any(|(i, c)| {
                        // Skip positions we just applied horn to - those aren't "earlier" transforms
                        if target_positions.contains(&i) {
                            return false;
                        }
                        // Check for any tone (circumflex, horn) or mark on NON-A vowels
                        // A itself might just be plain "a" waiting for breve
                        c.key != keys::A && (c.tone > 0 || c.mark > 0)
                    });

                    if has_earlier_transforms {
                        // "aw" ending is English (like "seesaw") - restore immediately
                        let raw_chars: Vec<char> = self
                            .raw_input
                            .iter()
                            .filter_map(|&(k, c, s)| utils::key_to_char_ext(k, c, s))
                            .collect();
                        let backspace = self.buf.len() as u8;
                        self.buf.clear();
                        self.raw_input.clear();
                        self.last_transform = None;
                        return Some(Result::send(backspace, &raw_chars));
                    }
                }
            }
            let has_breve_vowel_pattern = target_positions.iter().any(|&pos| {
                if let Some(c) = self.buf.get(pos) {
                    // Check if this is 'a' with horn (breve) followed by another vowel
                    if c.key == keys::A {
                        // Look for any vowel after this position
                        return (pos + 1..self.buf.len()).any(|i| {
                            self.buf
                                .get(i)
                                .map(|next| keys::is_vowel(next.key))
                                .unwrap_or(false)
                        });
                    }
                }
                false
            });

            if has_breve_vowel_pattern {
                // Revert: clear applied tones
                for &pos in &target_positions {
                    if let Some(c) = self.buf.get_mut(pos) {
                        c.tone = tone::NONE;
                    }
                }
                return None;
            }

            // Issue #44 (part 2): Always apply breve for "aw" pattern immediately
            // "aw" → "ă", "taw" → "tă", "raw" → "ră"
            // The breve is always applied - English auto-restore handles English words separately
            let has_breve_open_syllable = false;

            if has_breve_open_syllable {
                // Revert: clear applied tones, defer breve until final consonant
                for &pos in &target_positions {
                    if let Some(c) = self.buf.get_mut(pos) {
                        if c.key == keys::A {
                            c.tone = tone::NONE;
                            // Store position for deferred breve
                            self.pending_breve_pos = Some(pos);
                        }
                    }
                }
                // Return None to let 'w' fall through:
                // - try_w_as_vowel will fail (invalid vowel pattern)
                // - handle_normal_letter will add 'w' as regular letter
                // - When final consonant is typed, breve is applied
                return None;
            }
        }

        // Normalize ưo → ươ compound if horn was applied to 'u'
        if let Some(compound_pos) = self.normalize_uo_compound() {
            earliest_pos = earliest_pos.min(compound_pos);
        }

        self.last_transform = Some(Transform::Tone(key, tone_val));
        self.had_any_transform = true;

        // Reposition tone mark if vowel pattern changed
        let mut rebuild_pos = earliest_pos;
        if let Some((old_pos, _)) = self.reposition_tone_if_needed() {
            rebuild_pos = rebuild_pos.min(old_pos);
        }

        Some(self.rebuild_from(rebuild_pos))
    }

    /// Try to apply mark transformation
    fn try_mark(&mut self, key: u16, caps: bool, mark_val: u8) -> Option<Result> {
        if self.buf.is_empty() {
            return None;
        }

        // Check revert first
        if let Some(Transform::Mark(last_key, _)) = self.last_transform {
            if last_key == key {
                return Some(self.revert_mark(key, caps));
            }
        }

        // Telex: Check for delayed stroke pattern (d + vowels + d)
        // When buffer is "dod" and mark key is typed, apply stroke to initial 'd'
        // This enables "dods" → "đó" while preventing "de" + "d" → "đe"
        let had_delayed_stroke = self.method == 0
            && self.buf.len() >= 2
            && self
                .buf
                .get(0)
                .is_some_and(|c| c.key == keys::D && !c.stroke)
            && self.buf.last().is_some_and(|c| c.key == keys::D)
            && {
                // Check vowels and validity in one pass
                let buf_len = self.buf.len();
                let has_vowel = self
                    .buf
                    .iter()
                    .take(buf_len - 1)
                    .any(|c| keys::is_vowel(c.key));
                has_vowel && {
                    let buffer_without_last: Vec<u16> =
                        self.buf.iter().take(buf_len - 1).map(|c| c.key).collect();
                    is_valid(&buffer_without_last) && {
                        // Apply delayed stroke: stroke initial 'd', remove trigger 'd'
                        if let Some(c) = self.buf.get_mut(0) {
                            c.stroke = true;
                        }
                        self.buf.pop();
                        true
                    }
                }
            };

        // Issue #44: Apply pending breve before adding mark
        // When user types "aws" (Telex) or "a81" (VNI), they want "ắ" (breve + sắc)
        // Breve was deferred due to open syllable, but adding mark confirms Vietnamese input
        let mut had_pending_breve = false;
        if let Some(breve_pos) = self.pending_breve_pos {
            had_pending_breve = true;
            // Try to find and remove the breve modifier from buffer
            // Both Telex 'w' and VNI '8' are stored in buffer (handle_normal_letter adds them)
            let modifier_pos = breve_pos + 1;
            if modifier_pos < self.buf.len() {
                if let Some(c) = self.buf.get(modifier_pos) {
                    // Remove 'w' (Telex) or '8' (VNI) breve modifier from buffer
                    if c.key == keys::W || c.key == keys::N8 {
                        self.buf.remove(modifier_pos);
                    }
                }
            }
            // Apply breve to 'a'
            if let Some(c) = self.buf.get_mut(breve_pos) {
                if c.key == keys::A {
                    c.tone = tone::HORN; // HORN on A = breve (ă)
                    self.had_any_transform = true;
                }
            }
            self.pending_breve_pos = None;
        }

        // Check if buffer has horn transforms - indicates intentional Vietnamese typing
        // (e.g., "rượu" has base keys [R,U,O,U] which looks like "ou" pattern,
        // but with horns applied it's valid "ươu")
        let has_horn_transforms = self.buf.iter().any(|c| c.tone == tone::HORN);

        // Check if buffer has stroke transforms (đ) - indicates intentional Vietnamese typing
        // Issue #48: "ddeso" → "đéo" (d was stroked to đ, so this is Vietnamese, not English)
        let has_stroke_transforms = self.buf.iter().any(|c| c.stroke);

        // Validate buffer structure (skip if has horn/stroke transforms - already intentional Vietnamese)
        // Also skip validation if free_tone mode is enabled
        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
        let buffer_tones: Vec<u8> = self.buf.iter().map(|c| c.tone).collect();
        if !self.free_tone_enabled
            && !has_horn_transforms
            && !has_stroke_transforms
            && !is_valid_for_transform(&buffer_keys)
        {
            return None;
        }

        // Skip modifier if buffer shows foreign word patterns.
        // Only check when NO horn/stroke transforms exist.
        //
        // Detected patterns:
        // - Invalid vowel combinations (ou, yo) that don't exist in Vietnamese
        // - Consonant clusters after finals common in English (T+R, P+R, C+R)
        //
        // Examples:
        // - "met" + 'r' → T+R cluster common in English → skip modifier
        // - "you" + 'r' → "ou" vowel pattern invalid → skip modifier
        // - "rươu" + 'j' → has horn transforms → DON'T skip, apply mark normally
        // - "đe" + 's' → has stroke transform → DON'T skip, apply mark normally (Issue #48)
        // Skip foreign word detection if free_tone mode is enabled
        if !self.free_tone_enabled
            && !has_horn_transforms
            && !has_stroke_transforms
            && is_foreign_word_pattern(&buffer_keys, &buffer_tones, key)
        {
            return None;
        }

        // Issue #29: Normalize ưo → ươ compound before placing mark
        // In Vietnamese, "ưo" is never valid - it's always "ươ"
        let rebuild_from_compound = self.normalize_uo_compound();

        let vowels = self.collect_vowels();
        if vowels.is_empty() {
            return None;
        }

        // Find mark position using phonology rules
        let last_vowel_pos = vowels.last().map(|v| v.pos).unwrap_or(0);
        let has_final = self.has_final_consonant(last_vowel_pos);
        let has_qu = self.has_qu_initial();
        let has_gi = self.has_gi_initial();
        let pos =
            Phonology::find_tone_position(&vowels, has_final, self.modern_tone, has_qu, has_gi);

        if let Some(c) = self.buf.get_mut(pos) {
            c.mark = mark_val;
            self.last_transform = Some(Transform::Mark(key, mark_val));
            self.had_any_transform = true;
            // Rebuild from the earlier position if compound was formed
            let mut rebuild_pos = rebuild_from_compound.map_or(pos, |cp| cp.min(pos));

            // If delayed stroke was applied, rebuild from position 0
            // and add extra backspace for the trigger 'd' that was on screen
            if had_delayed_stroke {
                rebuild_pos = 0;
                let result = self.rebuild_from(rebuild_pos);
                let chars: Vec<char> = result.chars[..result.count as usize]
                    .iter()
                    .filter_map(|&c| char::from_u32(c))
                    .collect();
                // Add 1 to backspace for the trigger 'd' that was on screen but removed from buffer
                return Some(Result::send(result.backspace + 1, &chars));
            }

            // If there was pending breve, we need extra backspace
            // Screen has 'w' (Telex) or '8' (VNI) that needs to be deleted
            // Note: Telex 'w' was in buffer and removed, VNI '8' was never in buffer
            if had_pending_breve {
                let result = self.rebuild_from(rebuild_pos);
                // Convert u32 chars to char vec
                let chars: Vec<char> = result.chars[..result.count as usize]
                    .iter()
                    .filter_map(|&c| char::from_u32(c))
                    .collect();
                // Add 1 to backspace to account for modifier on screen
                return Some(Result::send(result.backspace + 1, &chars));
            }
            return Some(self.rebuild_from(rebuild_pos));
        }

        None
    }

    /// Normalize ưo → ươ compound
    ///
    /// In Vietnamese, "ưo" (u with horn + plain o) is NEVER valid.
    /// It should always be "ươ" (both with horn).
    /// This function finds and fixes this pattern anywhere in the buffer.
    ///
    /// Returns Some(position) of the 'o' that was modified, None if no change.
    fn normalize_uo_compound(&mut self) -> Option<usize> {
        // Look for pattern: U with horn + O without horn (anywhere in buffer)
        for i in 0..self.buf.len().saturating_sub(1) {
            let c1 = self.buf.get(i)?;
            let c2 = self.buf.get(i + 1)?;

            // Check: U with horn + O plain → always normalize to ươ
            let is_u_with_horn = c1.key == keys::U && c1.tone == tone::HORN;
            let is_o_plain = c2.key == keys::O && c2.tone == tone::NONE;

            if is_u_with_horn && is_o_plain {
                // Apply horn to O to form the ươ compound
                if let Some(c) = self.buf.get_mut(i + 1) {
                    c.tone = tone::HORN;
                    return Some(i + 1);
                }
            }
        }
        None
    }

    /// Find positions of U+O or O+U compound (adjacent vowels)
    /// Returns Some((first_pos, second_pos)) if found, None otherwise
    fn find_uo_compound_positions(&self) -> Option<(usize, usize)> {
        for i in 0..self.buf.len().saturating_sub(1) {
            if let (Some(c1), Some(c2)) = (self.buf.get(i), self.buf.get(i + 1)) {
                let is_uo = c1.key == keys::U && c2.key == keys::O;
                let is_ou = c1.key == keys::O && c2.key == keys::U;
                if is_uo || is_ou {
                    return Some((i, i + 1));
                }
            }
        }
        None
    }

    /// Check for uo compound in buffer (any tone state)
    fn has_uo_compound(&self) -> bool {
        self.find_uo_compound_positions().is_some()
    }

    /// Check for complete ươ compound (both u and o have horn)
    fn has_complete_uo_compound(&self) -> bool {
        if let Some((pos1, pos2)) = self.find_uo_compound_positions() {
            if let (Some(c1), Some(c2)) = (self.buf.get(pos1), self.buf.get(pos2)) {
                // Check ư + ơ pattern (both with horn)
                let is_u_horn = c1.key == keys::U && c1.tone == tone::HORN;
                let is_o_horn = c2.key == keys::O && c2.tone == tone::HORN;
                return is_u_horn && is_o_horn;
            }
        }
        false
    }

    /// Find target position for horn modifier with switching support
    /// Allows selecting vowels that have a different tone (for switching circumflex ↔ horn)
    fn find_horn_target_with_switch(&self, targets: &[u16], new_tone: u8) -> Vec<usize> {
        // Find vowel positions that match targets and either:
        // - have no tone (normal case)
        // - have a different tone (switching case)
        let vowels: Vec<usize> = self
            .buf
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                targets.contains(&c.key) && (c.tone == tone::NONE || c.tone != new_tone)
            })
            .map(|(i, _)| i)
            .collect();

        if vowels.is_empty() {
            return vec![];
        }

        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();

        // Use centralized phonology rules (context inferred from buffer)
        let mut result = Phonology::find_horn_positions(&buffer_keys, &vowels);

        // Special case: standalone "ua" pattern where U already has a mark
        // If user typed "uaf" → "ùa", then 'w' should go to U (making "ừa"), not A
        // This ensures consistent behavior: mark placement indicates user's intent
        if result.len() == 1 {
            if let Some(&pos) = result.first() {
                if let Some(c) = self.buf.get(pos) {
                    // If horn target is A, check if U exists before it with a mark
                    if c.key == keys::A && pos > 0 {
                        if let Some(prev) = self.buf.get(pos - 1) {
                            // Adjacent U with a mark → user wants horn on U, not breve on A
                            if prev.key == keys::U && prev.mark > 0 {
                                result = vec![pos - 1]; // Return U position instead
                            }
                        }
                    }
                }
            }
        }

        result
            .into_iter()
            .filter(|&pos| {
                self.buf
                    .get(pos)
                    .map(|c| {
                        targets.contains(&c.key) && (c.tone == tone::NONE || c.tone != new_tone)
                    })
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Reposition tone (sắc/huyền/hỏi/ngã/nặng) after vowel pattern changes
    ///
    /// When user types out-of-order (e.g., "osa" instead of "oas"), the tone may be
    /// placed on wrong vowel. This function moves it to the correct position based
    /// on Vietnamese phonology rules.
    ///
    /// Returns Some((old_pos, new_pos)) if tone was moved, None otherwise.
    fn reposition_tone_if_needed(&mut self) -> Option<(usize, usize)> {
        // Find vowel with tone mark (sắc/huyền/hỏi/ngã/nặng)
        let tone_info: Option<(usize, u8)> = self
            .buf
            .iter()
            .enumerate()
            .find(|(_, c)| c.mark > mark::NONE && keys::is_vowel(c.key))
            .map(|(i, c)| (i, c.mark));

        if let Some((old_pos, tone_value)) = tone_info {
            let vowels = self.collect_vowels();
            if vowels.is_empty() {
                return None;
            }

            // Check for syllable boundary: if there's a consonant between the toned vowel
            // and any later vowel, the toned vowel is in a closed syllable - don't reposition.
            // Example: "bủn" + "o" → 'n' closes "bủn", so 'o' starts new syllable.
            let has_consonant_after_tone = (old_pos + 1..self.buf.len()).any(|i| {
                self.buf
                    .get(i)
                    .is_some_and(|c| !keys::is_vowel(c.key) && c.key != keys::W)
            });
            let has_vowel_after_consonant = has_consonant_after_tone
                && vowels
                    .iter()
                    .any(|v| v.pos > old_pos && self.has_consonant_between(old_pos, v.pos));

            if has_vowel_after_consonant {
                // Syllable boundary detected - tone is in previous syllable, don't move it
                return None;
            }

            let last_vowel_pos = vowels.last().map(|v| v.pos).unwrap_or(0);
            let has_final = self.has_final_consonant(last_vowel_pos);
            let has_qu = self.has_qu_initial();
            let has_gi = self.has_gi_initial();
            let new_pos =
                Phonology::find_tone_position(&vowels, has_final, self.modern_tone, has_qu, has_gi);

            if new_pos != old_pos {
                // Move tone from old position to new position
                if let Some(c) = self.buf.get_mut(old_pos) {
                    c.mark = mark::NONE;
                }
                if let Some(c) = self.buf.get_mut(new_pos) {
                    c.mark = tone_value;
                }
                return Some((old_pos, new_pos));
            }
        }
        None
    }

    /// Check if there's a consonant between two positions
    fn has_consonant_between(&self, start: usize, end: usize) -> bool {
        (start + 1..end).any(|i| {
            self.buf
                .get(i)
                .is_some_and(|c| !keys::is_vowel(c.key) && c.key != keys::W)
        })
    }

    /// Common revert logic: clear modifier, add key to buffer, rebuild output
    fn revert_and_rebuild(&mut self, pos: usize, key: u16, caps: bool) -> Result {
        // Calculate backspace BEFORE adding key (based on old buffer state)
        let backspace = (self.buf.len() - pos) as u8;

        // Add the reverted key to buffer so validation sees the full sequence
        self.buf.push(Char::new(key, caps));

        // Build output from position (includes new key)
        let output: Vec<char> = (pos..self.buf.len())
            .filter_map(|i| self.buf.get(i))
            .filter_map(|c| utils::key_to_char(c.key, c.caps))
            .collect();

        Result::send(backspace, &output)
    }

    /// Revert tone transformation
    fn revert_tone(&mut self, key: u16, caps: bool) -> Result {
        self.last_transform = None;

        for pos in self.buf.find_vowels().into_iter().rev() {
            if let Some(c) = self.buf.get_mut(pos) {
                if c.tone > tone::NONE {
                    c.tone = tone::NONE;
                    return self.revert_and_rebuild(pos, key, caps);
                }
            }
        }
        Result::none()
    }

    /// Revert mark transformation
    /// When mark is reverted, only the reverting key appears as a letter.
    /// Standard behavior: "ass" → "as" (first 's' was modifier, second 's' reverts + outputs one 's')
    /// This matches standard Vietnamese IME behavior (UniKey, ibus-unikey, etc.)
    fn revert_mark(&mut self, key: u16, caps: bool) -> Result {
        self.last_transform = None;
        self.had_mark_revert = true; // Track for auto-restore

        for pos in self.buf.find_vowels().into_iter().rev() {
            if let Some(c) = self.buf.get_mut(pos) {
                if c.mark > mark::NONE {
                    c.mark = mark::NONE;

                    // Set flag to defer raw_input pop until next key
                    // If next key is CONSONANT: pop the mark key (user intended revert)
                    //   Example: "tesst" → next is 't' (consonant) → pop → "test"
                    // If next key is VOWEL: don't pop (user typing English word like "issue")
                    //   Example: "issue" → next is 'u' (vowel) → keep → "issue"
                    self.pending_mark_revert_pop = true;

                    // Add only the reverting key (current key being pressed)
                    // The original mark key was consumed as a modifier and doesn't produce output
                    self.buf.push(Char::new(key, caps));

                    // Calculate backspace and output
                    let backspace = (self.buf.len() - pos - 1) as u8; // -1 because we added 1 char
                    let output: Vec<char> = (pos..self.buf.len())
                        .filter_map(|i| self.buf.get(i))
                        .filter_map(|c| utils::key_to_char(c.key, c.caps))
                        .collect();

                    return Result::send(backspace, &output);
                }
            }
        }
        Result::none()
    }

    /// Revert stroke transformation at specific position
    fn revert_stroke(&mut self, key: u16, pos: usize) -> Result {
        self.last_transform = None;

        if let Some(c) = self.buf.get_mut(pos) {
            if c.key == keys::D && !c.stroke {
                // Un-stroked d found at pos - this means we need to add another d
                let caps = c.caps;
                self.buf.push(Char::new(key, caps));
                return self.rebuild_from(pos);
            }
        }
        Result::none()
    }

    /// Try to apply remove modifier
    /// Returns Some(Result) if a mark/tone was removed, None if nothing to remove
    /// When None is returned, the key falls through to handle_normal_letter()
    fn try_remove(&mut self) -> Option<Result> {
        self.last_transform = None;
        for pos in self.buf.find_vowels().into_iter().rev() {
            if let Some(c) = self.buf.get_mut(pos) {
                if c.mark > mark::NONE {
                    c.mark = mark::NONE;
                    return Some(self.rebuild_from(pos));
                }
                if c.tone > tone::NONE {
                    c.tone = tone::NONE;
                    return Some(self.rebuild_from(pos));
                }
            }
        }
        // Nothing to remove - return None so key can be processed as normal letter
        // This allows shortcuts like "zz" to work
        None
    }

    /// Handle normal letter input
    fn handle_normal_letter(&mut self, key: u16, caps: bool) -> Result {
        // Special case: "o" after "w→ư" should form "ươ" compound
        // This only handles the WAsVowel case (typing "w" alone creates ư)
        // For "uw" pattern, the compound is normalized in try_mark via normalize_uo_compound
        if key == keys::O && matches!(self.last_transform, Some(Transform::WAsVowel)) {
            // Add O with horn to form ươ compound
            let mut c = Char::new(key, caps);
            c.tone = tone::HORN;
            self.buf.push(c);
            self.last_transform = None;

            // Return the ơ character (o with horn)
            let vowel_char = chars::to_char(keys::O, caps, tone::HORN, 0).unwrap();
            return Result::send(0, &[vowel_char]);
        }

        // Note: ShortPatternStroke revert is now handled at the beginning of process()
        // before any modifiers are applied, so we don't need to check it here.

        self.last_transform = None;
        // Add letters to buffer, and numbers in VNI mode (for pass-through after revert)
        // This ensures buffer.len() stays in sync with screen chars for correct backspace count
        if keys::is_letter(key) || (self.method == 1 && keys::is_number(key)) {
            // Add the letter/number to buffer
            self.buf.push(Char::new(key, caps));

            // Issue #44 (part 2): Apply deferred breve when valid final consonant is typed
            // "trawm" → after "traw" (pending breve on 'a'), typing 'm' applies breve → "trăm"
            if let Some(breve_pos) = self.pending_breve_pos {
                // Valid final consonants that make breve valid: c, m, n, p, t
                if matches!(key, keys::C | keys::M | keys::N | keys::P | keys::T) {
                    // Find and remove the breve modifier from buffer
                    // Telex uses 'w', VNI uses '8' - it should be right after 'a' at breve_pos
                    let modifier_pos = breve_pos + 1;
                    if modifier_pos < self.buf.len() {
                        if let Some(c) = self.buf.get(modifier_pos) {
                            // Remove 'w' (Telex) or '8' (VNI)
                            if c.key == keys::W || c.key == keys::N8 {
                                self.buf.remove(modifier_pos);
                            }
                        }
                    }

                    // Apply breve to the 'a' at pending position
                    let a_caps = self.buf.get(breve_pos).map(|c| c.caps).unwrap_or(false);
                    if let Some(c) = self.buf.get_mut(breve_pos) {
                        if c.key == keys::A {
                            c.tone = tone::HORN; // HORN on A = breve (ă)
                            self.had_any_transform = true;
                        }
                    }
                    self.pending_breve_pos = None;

                    // Rebuild from breve position: delete "aw" (or "awX"), output "ăX"
                    // Buffer now has: ...ă (at breve_pos) + consonant (just added)
                    // Screen has: ...aw (need to delete "aw", output "ă" + consonant)
                    let vowel_char = chars::to_char(keys::A, a_caps, tone::HORN, 0).unwrap_or('ă');
                    let cons_char = crate::utils::key_to_char(key, caps).unwrap_or('?');
                    return Result::send(2, &[vowel_char, cons_char]); // backspace 2 ("aw"), output "ăm"
                } else if key == keys::W {
                    // 'w' is the breve modifier - don't clear pending_breve_pos
                    // It will be added as a regular letter and removed later
                } else if keys::is_vowel(key) {
                    // Vowel after "aw" pattern - breve not valid, clear pending
                    self.pending_breve_pos = None;
                }
                // For other consonants (not finals, not W), keep pending_breve_pos
                // They might be followed by more letters that complete the syllable
            }

            // Normalize ưo → ươ immediately when 'o' is typed after 'ư'
            // This ensures "dduwo" → "đươ" (Telex) and "u7o" → "ươ" (VNI)
            // Works for both methods since "ưo" alone is not valid Vietnamese
            if key == keys::O && self.normalize_uo_compound().is_some() {
                // ươ compound formed - reposition tone if needed (ư→ơ)
                if let Some((old_pos, _)) = self.reposition_tone_if_needed() {
                    return self.rebuild_from_after_insert(old_pos);
                }

                // No tone to reposition - just output ơ
                let vowel_char = chars::to_char(keys::O, caps, tone::HORN, 0).unwrap();
                return Result::send(0, &[vowel_char]);
            }

            // Auto-correct tone position when new character changes the correct placement
            //
            // Two scenarios:
            // 1. New vowel changes diphthong pattern:
            //    "osa" → tone on 'o', then 'a' added → "oa" needs tone on 'a'
            // 2. New consonant creates final, which changes tone position:
            //    "muas" → tone on 'u' (ua open), then 'n' added → "uan" needs tone on 'a'
            //
            // Both cases need to reposition the tone mark based on Vietnamese phonology.
            if let Some((old_pos, _new_pos)) = self.reposition_tone_if_needed() {
                // Tone was moved - rebuild output from the old position
                // Note: the new char was just added to buffer but NOT yet displayed
                // So backspace = (chars from old_pos to BEFORE new char)
                // And output = (chars from old_pos to end INCLUDING new char)
                return self.rebuild_from_after_insert(old_pos);
            }

            // Check if adding this letter creates invalid vowel pattern (foreign word detection)
            // Only revert if the horn transforms are from w-as-vowel (standalone w→ư),
            // not from w-as-tone (adding horn to existing vowels like in "rượu")
            //
            // w-as-vowel: first horn is U at position 0 (was standalone 'w')
            // w-as-tone: horns are on vowels after initial consonant
            //
            // Exception: complete ươ compound + vowel = valid Vietnamese triphthong
            // (like "rượu" = ươu, "mười" = ươi) - don't revert in these cases
            // Only skip for vowels that form valid triphthongs (u, i), not for consonants
            // Only run foreign word detection if english_auto_restore is enabled
            if self.english_auto_restore {
                let is_valid_triphthong_ending =
                    self.has_complete_uo_compound() && (key == keys::U || key == keys::I);
                if self.has_w_as_vowel_transform() && !is_valid_triphthong_ending {
                    let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
                    let buffer_tones: Vec<u8> = self.buf.iter().map(|c| c.tone).collect();
                    if is_foreign_word_pattern(&buffer_keys, &buffer_tones, key) {
                        return self.revert_w_as_vowel_transforms();
                    }
                }
            }

            // Auto-restore when consonant after mark creates clear English pattern
            // Example: "tex" → "tẽ", then 't' typed → "tẽt" has English modifier pattern → restore "text"
            //
            // IMPORTANT: Mid-word, only restore for clear English PATTERNS (modifier+consonant clusters),
            // NOT just structural invalidity. Words like "dọd" are invalid but user might still be typing.
            // Full structural validation happens at word boundary (space/break).
            //
            // This catches: "tex" + 't' where 'x' modifier before 't' creates English cluster
            // But preserves: "dọ" + 'd' where 'j' modifier before 'd' doesn't indicate English
            //
            // Only run if english_auto_restore is enabled (experimental feature)
            if self.english_auto_restore && keys::is_consonant(key) && self.buf.len() >= 2 {
                // Check if consonant immediately follows a marked character
                if let Some(prev_char) = self.buf.get(self.buf.len() - 2) {
                    let prev_has_mark = prev_char.mark > 0 || prev_char.tone > 0;
                    if prev_has_mark && self.has_english_modifier_pattern(false) {
                        // Clear English pattern detected - restore to raw
                        if let Some(raw_chars) = self.build_raw_chars() {
                            let backspace = (self.buf.len() - 1) as u8;

                            // Repopulate buffer with restored content (plain chars, no marks)
                            self.buf.clear();
                            for &(key, caps, _) in &self.raw_input {
                                self.buf.push(Char::new(key, caps));
                            }

                            self.last_transform = None;
                            return Result::send(backspace, &raw_chars);
                        }
                    }
                }
            }
        } else {
            // Non-letter character (number, symbol, etc.)
            // Mark that this word has non-letter prefix to prevent false shortcut matches
            // e.g., "149k" should NOT trigger shortcut "k" → "không"
            // e.g., "@abc" should NOT trigger shortcut "abc"
            self.has_non_letter_prefix = true;
        }
        Result::none()
    }

    /// Check if buffer has w-as-vowel transform (standalone w→ư at start)
    /// This is different from w-as-tone which adds horn to existing vowels
    fn has_w_as_vowel_transform(&self) -> bool {
        // w-as-vowel creates U with horn at position 0 or after consonants
        // The key distinguishing feature: the U with horn was created from 'w',
        // meaning there was no preceding vowel at that position
        //
        // Simple heuristic: if first char is U with horn, it's w-as-vowel
        // (words like "rượu" start with consonant R, not U)
        self.buf
            .get(0)
            .map(|c| c.key == keys::U && c.tone == tone::HORN)
            .unwrap_or(false)
    }

    /// Revert w-as-vowel transforms and rebuild output
    /// Used when foreign word pattern is detected after w→ư transformation
    fn revert_w_as_vowel_transforms(&mut self) -> Result {
        // Only revert if first char is U with horn (w-as-vowel pattern)
        if !self.has_w_as_vowel_transform() {
            return Result::none();
        }

        // Find all horn transforms to revert
        let horn_positions: Vec<usize> = self
            .buf
            .iter()
            .enumerate()
            .filter(|(_, c)| c.tone == tone::HORN)
            .map(|(i, _)| i)
            .collect();

        if horn_positions.is_empty() {
            return Result::none();
        }

        let first_pos = horn_positions[0];

        // Clear horn tones and change U back to W (for w-as-vowel positions)
        for &pos in &horn_positions {
            if let Some(c) = self.buf.get_mut(pos) {
                // U with horn was from 'w' → change key to W
                if c.key == keys::U {
                    c.key = keys::W;
                }
                c.tone = tone::NONE;
            }
        }

        self.rebuild_from(first_pos)
    }

    /// Collect vowels from buffer
    fn collect_vowels(&self) -> Vec<Vowel> {
        utils::collect_vowels(&self.buf)
    }

    /// Check for final consonant after position
    fn has_final_consonant(&self, after_pos: usize) -> bool {
        utils::has_final_consonant(&self.buf, after_pos)
    }

    /// Check for qu initial
    fn has_qu_initial(&self) -> bool {
        utils::has_qu_initial(&self.buf)
    }

    /// Check for gi initial (gi + vowel)
    fn has_gi_initial(&self) -> bool {
        utils::has_gi_initial(&self.buf)
    }

    /// Rebuild output from position
    fn rebuild_from(&self, from: usize) -> Result {
        let mut output = Vec::with_capacity(self.buf.len() - from);
        let mut backspace = 0u8;

        for i in from..self.buf.len() {
            if let Some(c) = self.buf.get(i) {
                backspace += 1;

                if c.key == keys::D && c.stroke {
                    output.push(chars::get_d(c.caps));
                } else if let Some(ch) = chars::to_char(c.key, c.caps, c.tone, c.mark) {
                    output.push(ch);
                } else if let Some(ch) = utils::key_to_char(c.key, c.caps) {
                    output.push(ch);
                }
            }
        }

        if output.is_empty() {
            Result::none()
        } else {
            Result::send(backspace, &output)
        }
    }

    /// Rebuild output from position after a new character was inserted
    ///
    /// Unlike rebuild_from, this accounts for the fact that the last character
    /// in the buffer was just added but NOT yet displayed on screen.
    /// So backspace count = (chars from `from` to end - 1) because last char isn't on screen.
    fn rebuild_from_after_insert(&self, from: usize) -> Result {
        if self.buf.is_empty() {
            return Result::none();
        }

        let mut output = Vec::with_capacity(self.buf.len() - from);
        // Backspace = number of chars from `from` to BEFORE the new char
        // The new char (last in buffer) hasn't been displayed yet
        let backspace = (self.buf.len().saturating_sub(1).saturating_sub(from)) as u8;

        for i in from..self.buf.len() {
            if let Some(c) = self.buf.get(i) {
                if c.key == keys::D && c.stroke {
                    output.push(chars::get_d(c.caps));
                } else if let Some(ch) = chars::to_char(c.key, c.caps, c.tone, c.mark) {
                    output.push(ch);
                } else if let Some(ch) = utils::key_to_char(c.key, c.caps) {
                    output.push(ch);
                }
            }
        }

        if output.is_empty() {
            Result::none()
        } else {
            Result::send(backspace, &output)
        }
    }

    /// Clear buffer and raw input history
    /// Note: Does NOT clear word_history to preserve backspace-after-space feature
    pub fn clear(&mut self) {
        self.buf.clear();
        self.raw_input.clear();
        self.last_transform = None;
        self.has_non_letter_prefix = false;
        self.pending_breve_pos = None;
        self.stroke_reverted = false;
        self.had_mark_revert = false;
        self.pending_mark_revert_pop = false;
        self.had_any_transform = false;
    }

    /// Clear everything including word history
    /// Used when cursor position changes (mouse click, arrow keys, etc.)
    /// to prevent accidental restore from stale history
    pub fn clear_all(&mut self) {
        self.clear();
        self.word_history.clear();
        self.spaces_after_commit = 0;
    }

    /// Get the full composed buffer as a Vietnamese string with diacritics.
    ///
    /// Used for "Select All + Replace" injection method.
    pub fn get_buffer_string(&self) -> String {
        self.buf.to_full_string()
    }

    /// Restore buffer from a Vietnamese word string
    ///
    /// Used when native app detects cursor at word boundary and wants to edit.
    /// Parses Vietnamese characters back to buffer components.
    pub fn restore_word(&mut self, word: &str) {
        self.clear();
        for c in word.chars() {
            if let Some(parsed) = chars::parse_char(c) {
                let mut ch = Char::new(parsed.key, parsed.caps);
                ch.tone = parsed.tone;
                ch.mark = parsed.mark;
                ch.stroke = parsed.stroke;
                self.buf.push(ch);
                self.raw_input.push((parsed.key, parsed.caps, false));
            }
        }
    }

    /// Check if buffer has transforms and is invalid Vietnamese
    /// Returns the raw chars if restore is needed, None otherwise
    ///
    /// `is_word_complete`: true when called on space/break (word is complete)
    ///                     false when called mid-word (during typing)
    fn should_auto_restore(&self, is_word_complete: bool) -> Option<Vec<char>> {
        // Only run auto-restore if the feature is enabled
        if !self.english_auto_restore {
            return None;
        }

        if self.raw_input.is_empty() || self.buf.is_empty() {
            return None;
        }

        // If no Vietnamese transforms were ever applied this word, nothing to restore
        // This prevents false restore for words with numbers/symbols like "nhatkha1407@gmail.com"
        // where the buffer is invalid Vietnamese but no transforms were ever attempted
        if !self.had_any_transform {
            return None;
        }

        // Check if any transforms remain in buffer
        // - Marks (sắc, huyền, hỏi, ngã, nặng): indicate Vietnamese typing intent
        // - Vowel tones (â, ê, ô, ư, ă): indicate Vietnamese typing intent
        // - Stroke (đ): included for longer words that are structurally invalid
        let has_marks_or_tones = self.buf.iter().any(|c| c.tone > 0 || c.mark > 0);
        let has_stroke = self.buf.iter().any(|c| c.stroke);

        // If no transforms remain in buffer AND user reverted at END of word,
        // keep the result (user intentionally reverted)
        // Examples: "ass" → "as", "maxx" → "max" (double modifier at end)
        // But "issue" → "isue" should still check validity (more letters typed after revert)
        if !has_marks_or_tones && !has_stroke && self.ends_with_double_modifier() {
            return None;
        }

        // Check 1: If buffer_keys is structurally invalid Vietnamese → RESTORE
        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
        let is_structurally_valid = is_valid(&buffer_keys);

        if !is_structurally_valid {
            // For stroke-only transforms (no marks/tones), only restore if word is long enough
            // Short words like "đd" from "ddd" should stay; long invalid words like "đealine" should restore
            if has_stroke && !has_marks_or_tones {
                // Stroke-only: restore if word has 4+ chars (likely English like "deadline")
                // Keep short words (e.g., "đd" from "ddd")
                if self.buf.len() < 4 {
                    return None;
                }
            }
            return self.build_raw_chars();
        }

        // Check 2: English patterns in raw_input
        // Even if buffer is valid, certain patterns suggest English
        if self.has_english_modifier_pattern(is_word_complete) {
            return self.build_raw_chars();
        }

        // Buffer is valid Vietnamese AND no English patterns → KEEP
        None
    }

    /// Check if this is an intentional revert at end of word that should be kept.
    /// Returns true when double modifier is at end AND it's likely intentional (not English word).
    ///
    /// Heuristics:
    /// - Very short words (raw_input <= 3 chars): likely intentional revert → keep
    /// - Double vowel tone keys (a, e, o, w): always intentional → keep
    /// - Double 'x' or 'j': not common in English → keep
    /// - Double 's', 'f', 'r' in longer words (4+ chars): common English pattern → restore
    ///
    /// Examples:
    /// - "ass" (3 chars, ss) → keep "as"
    /// - "aaa" (3 chars, aa) → keep "aa" (circumflex revert)
    /// - "maxx" (4 chars, xx) → keep "max" (xx not common in English)
    /// - "bass" (4 chars, ss) → restore to "bass" (ss very common in English)
    fn ends_with_double_modifier(&self) -> bool {
        if self.raw_input.len() < 2 {
            return false;
        }

        let (last_key, _, _) = self.raw_input[self.raw_input.len() - 1];
        let (second_last_key, _, _) = self.raw_input[self.raw_input.len() - 2];

        // Must be same key pressed twice
        if last_key != second_last_key {
            return false;
        }

        // Check if it's a vowel tone key (Telex: a, e, o for circumflex; w for horn/breve)
        // These are always intentional reverts - no English words use double vowels like this
        if self.method == 0 {
            if matches!(last_key, keys::A | keys::E | keys::O | keys::W) {
                return true;
            }
        } else {
            // VNI: 6, 7, 8 for vowel tones
            if matches!(last_key, keys::N6 | keys::N7 | keys::N8) {
                return true;
            }
        }

        // Check if it's a mark key
        let is_mark_key = if self.method == 0 {
            // Telex tone modifiers: s, f, r, x, j
            matches!(last_key, keys::S | keys::F | keys::R | keys::X | keys::J)
        } else {
            // VNI tone modifiers: 1, 2, 3, 4, 5
            matches!(
                last_key,
                keys::N1 | keys::N2 | keys::N3 | keys::N4 | keys::N5
            )
        };

        if !is_mark_key {
            return false;
        }

        // Very short words (3 chars or less) → likely intentional revert
        if self.raw_input.len() <= 3 {
            return true;
        }

        // For longer words (4+ chars), check modifier type:
        // - 'x', 'j' (Telex) or VNI numbers: not common doubles in English → keep
        // - 's', 'f', 'r' (Telex): very common doubles in English (bass, staff, error) → restore
        if self.method == 0 {
            // Telex: only keep for uncommon double letters (x, j)
            matches!(last_key, keys::X | keys::J)
        } else {
            // VNI: number modifiers are always intentional → keep
            true
        }
    }

    /// Build raw chars from raw_input for restore
    fn build_raw_chars(&self) -> Option<Vec<char>> {
        let raw_chars: Vec<char> = self
            .raw_input
            .iter()
            .filter_map(|&(key, caps, shift)| utils::key_to_char_ext(key, caps, shift))
            .collect();

        if raw_chars.is_empty() {
            None
        } else {
            Some(raw_chars)
        }
    }

    /// Check for English patterns in raw_input that suggest non-Vietnamese
    ///
    /// Patterns detected:
    /// 1. Modifier (s/f/r/x/j in Telex) followed by consonant: "text" (x before t)
    /// 2. Modifier at end of long word (>2 chars): "their" (r at end)
    /// 3. Modifier after first vowel then another vowel: "use" (s between u and e)
    /// 4. Consonant + W + vowel without tone modifiers (only on word complete): "swim"
    fn has_english_modifier_pattern(&self, is_word_complete: bool) -> bool {
        // Check for W at start - W is not a valid Vietnamese initial consonant
        // Words like "wow", "window", "water" start with W
        // Exception: standalone "w" → "ư" is valid Vietnamese
        if self.raw_input.len() >= 2 {
            let (first, _, _) = self.raw_input[0];
            if first == keys::W {
                // Check if there's another W later (non-adjacent) → English pattern like "wow"
                let has_later_w = self.raw_input[2..].iter().any(|(k, _, _)| *k == keys::W);
                if has_later_w {
                    return true;
                }

                // Analyze pattern: W + vowels + consonants
                let vowels_after: Vec<u16> = self.raw_input[1..]
                    .iter()
                    .filter(|(k, _, _)| keys::is_vowel(*k) && *k != keys::W)
                    .map(|(k, _, _)| *k)
                    .collect();

                let consonants_after: Vec<u16> = self.raw_input[1..]
                    .iter()
                    .filter(|(k, _, _)| keys::is_consonant(*k) && *k != keys::W)
                    .map(|(k, _, _)| *k)
                    .collect();

                // W + vowel + consonant → likely English like "win", "water"
                // But W + vowel only → valid Vietnamese (ưa, ưe, ưi, ươ)
                // And W + consonant only → valid Vietnamese (ưng, ưn, ưm)
                if !vowels_after.is_empty() && !consonants_after.is_empty() {
                    // Both vowels and consonants after W → likely English
                    return true;
                }

                // W + consonants only → check if valid Vietnamese final
                if !consonants_after.is_empty() && vowels_after.is_empty() {
                    let is_valid_final = match consonants_after.len() {
                        1 => constants::VALID_FINALS_1.contains(&consonants_after[0]),
                        2 => {
                            let pair = [consonants_after[0], consonants_after[1]];
                            constants::VALID_FINALS_2.contains(&pair)
                        }
                        _ => false, // 3+ consonants is invalid
                    };

                    if !is_valid_final {
                        return true;
                    }
                }
            }

            // Check for consonant + W + vowel pattern without tone modifiers
            // Only check when word is complete (on space/break), not mid-word
            // Mid-word we can't tell if user will add tone modifiers later
            // - "nwoc" during typing → might become "nwocj" → "nược" (Vietnamese)
            // - "swim" on space → no tone modifiers → restore to English
            if is_word_complete {
                let (second, _, _) = self.raw_input[1];
                if second == keys::W && keys::is_consonant(first) && first != keys::Q {
                    // Q+W is valid Vietnamese (qu-), but other consonant+W may be English
                    if self.raw_input.len() >= 3 {
                        let (third, _, _) = self.raw_input[2];
                        // Check if third char is a vowel (not a tone modifier like j)
                        if keys::is_vowel(third) {
                            // Check if there's ANY tone modifier (j/s/f/r/x) in the rest of the word
                            let tone_modifiers = [keys::S, keys::F, keys::R, keys::X, keys::J];
                            let has_tone_modifier = self.raw_input[2..]
                                .iter()
                                .any(|(k, _, _)| tone_modifiers.contains(k));

                            // No tone modifier + consonant+W+vowel → likely English like "swim"
                            if !has_tone_modifier {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        // Telex modifiers that add tone marks
        let tone_modifiers = [keys::S, keys::F, keys::R, keys::X, keys::J];

        // Find positions of modifiers in raw_input
        for i in 0..self.raw_input.len() {
            let (key, _, _) = self.raw_input[i];

            if !tone_modifiers.contains(&key) {
                continue;
            }

            // Found a modifier at position i

            // Pattern 1: Modifier followed by consonant → English
            // Example: "text" has X followed by T, "expect" has X followed by P
            // Counter-example: "muwowjt" has J followed by T (Vietnamese - multiple vowels)
            // Counter-example: "dojdc" = D+O+J+D+C (Vietnamese "đọc" - j + consonants is valid)
            if i + 1 < self.raw_input.len() {
                let (next_key, _, _) = self.raw_input[i + 1];
                // W is a vowel modifier in Telex, not a true consonant for this check
                let is_true_consonant = keys::is_consonant(next_key) && next_key != keys::W;
                if is_true_consonant {
                    // Heuristic: In Vietnamese, tone modifiers + consonant is common:
                    // - nặng (j) + consonant: học, bọc, bật, cặp, đọc, etc.
                    // - sắc (s) + consonant: bức, đất, ất, etc.
                    // - huyền (f) + consonant: làm, hàng, dùng, vàng, etc.
                    // - hỏi (r) + consonant: tỉnh, đỉnh, nhỉnh, mỉnh, etc.
                    // - ngã (x) + consonant: mãnh, hãnh, etc.
                    //
                    // Skip restore for ALL tone modifiers followed by consonant
                    // This handles:
                    // - "dojc" → "dọc" (j + final c)
                    // - "lafm" → "làm" (f + final m)
                    // - "tirnh" → "tỉnh" (r + final nh)
                    // - "maxnh" → "mãnh" (x + final nh)
                    // Vietnamese tone modifiers have different likelihood with consonants:
                    // - nặng (j) + any consonant: COMMON (học, bọc, bật, làm, etc.)
                    // - sắc (s) + any consonant: COMMON (bức, đất, sắm, etc.)
                    // - huyền (f) + sonorant (m,n,ng,nh): COMMON (làm, hàng, dùng, cũng)
                    // - hỏi (r) + sonorant (m,n,ng,nh): COMMON (tỉnh, đỉnh, nhỉnh, cửng)
                    // - ngã (x) + sonorant (m,n,ng,nh): COMMON (mãnh, hãnh, cũng)
                    // - huyền/hỏi/ngã + stop (c,p,t): RARE in Vietnamese
                    let is_common_viet_mark = key == keys::J || key == keys::S;
                    let is_rare_with_stop = key == keys::F || key == keys::R || key == keys::X;
                    // Sonorants: M, N, or G/H when following N (part of ng, nh finals)
                    let is_sonorant_or_part_of_final = next_key == keys::M
                        || next_key == keys::N
                        || (next_key == keys::G && i >= 1 && self.raw_input[i - 1].0 == keys::N)
                        || (next_key == keys::H && i >= 1 && self.raw_input[i - 1].0 == keys::N);

                    // Always skip for J and S - these are very common in Vietnamese
                    if is_common_viet_mark {
                        continue;
                    }

                    // For F, R, X: skip only if followed by sonorant (m, n, ng, nh)
                    // This allows "text" to restore but keeps "tỉnh", "làm", "mãnh", "cũng"
                    if is_rare_with_stop && is_sonorant_or_part_of_final {
                        continue;
                    }

                    // Case 1a: More letters after the consonant → definitely English
                    // Example: "expect" = E+X+P+E+C+T (X followed by P, then more)
                    if i + 2 < self.raw_input.len() {
                        return true;
                    }

                    // Case 1b: Final consonant but only 1 vowel before modifier → likely English
                    // Example: "text" = T+E+X+T (only 1 vowel E before X)
                    let vowels_before: usize = (0..i)
                        .filter(|&j| keys::is_vowel(self.raw_input[j].0))
                        .count();
                    if vowels_before == 1 {
                        return true;
                    }
                }
            }

            // Pattern 2: Modifier at end AND suspicious vowel pair before → English
            // Example: "their" → t-h-e-i-r, "ei" before r → suspicious English pattern
            // Example: "pair" → p-a-i-r, "ai" before r (only 2 vowels) → suspicious English pattern
            // Counter-example: "booj" → b-o-o-j, "oo" (same vowel) → Telex doubling, Vietnamese
            // Counter-example: "chiuj" → c-h-i-u-j, "iu" → valid Vietnamese diphthong
            // Counter-example: "hoaij" → h-o-a-i-j, "oai" (3 vowels) → valid Vietnamese
            if i + 1 == self.raw_input.len() && i >= 2 {
                let (v1, _, _) = self.raw_input[i - 2];
                let (v2, _, _) = self.raw_input[i - 1];
                // Check for suspicious English vowel patterns before modifier
                // Same vowel doubling (oo, aa, ee) is Telex pattern, not suspicious
                if keys::is_vowel(v1) && keys::is_vowel(v2) && v1 != v2 {
                    // Count total vowels before modifier
                    let total_vowels: usize = (0..i)
                        .filter(|&j| keys::is_vowel(self.raw_input[j].0))
                        .count();

                    // EI before modifier is very English (their, weird, vein)
                    if v1 == keys::E && v2 == keys::I {
                        return true;
                    }
                    // AI before modifier is English ONLY if:
                    // 1. Exactly 2 vowels (not "oai" in "hoại")
                    // 2. AND initial is P alone (not PH) - P is rare in native Vietnamese
                    // This catches "pair" but not "mái", "cái", "xài" (common Vietnamese)
                    if v1 == keys::A && v2 == keys::I && total_vowels == 2 {
                        // Check if initial is just P (rare in native Vietnamese)
                        if !self.raw_input.is_empty() && self.raw_input[0].0 == keys::P {
                            // Make sure it's not PH (PH is common Vietnamese)
                            let is_ph = self.raw_input.len() >= 2 && self.raw_input[1].0 == keys::H;
                            if !is_ph {
                                return true;
                            }
                        }
                    }
                }
            }

            // Pattern 3: Modifier immediately after single vowel, then another vowel
            // AND no initial consonant before the vowel
            // Example: "use" → U (vowel) + S (modifier) + E (vowel) = starts with vowel → English
            // Counter-example: "cura" → C + U + R + A = starts with consonant → Vietnamese "của"
            let vowels_before: usize = (0..i)
                .filter(|&j| keys::is_vowel(self.raw_input[j].0))
                .count();

            // If only 1 vowel before modifier AND vowel after AND no initial consonant → English
            if vowels_before == 1 && i + 1 < self.raw_input.len() {
                let (next_key, _, _) = self.raw_input[i + 1];
                if keys::is_vowel(next_key) {
                    // Find first vowel position
                    let first_vowel_pos = (0..i)
                        .find(|&j| keys::is_vowel(self.raw_input[j].0))
                        .unwrap_or(0);
                    // Check if there's a consonant before the first vowel
                    let has_initial_consonant = first_vowel_pos > 0
                        && keys::is_consonant(self.raw_input[first_vowel_pos - 1].0);
                    // Only restore if NO initial consonant (pure vowel-start like "use")
                    if !has_initial_consonant {
                        return true;
                    }

                    // Pattern 4: vowel + modifier + DIFFERENT vowel → English
                    // EXCEPT for Vietnamese diphthong patterns with tone in middle:
                    // - U + modifier + A/O: ưa, ươ (của, được)
                    // - A + modifier + I/Y/O: ai, ay, ao (gái, máy, nào)
                    // - O + modifier + I/A: oi, oa (bói, hói, hoá)
                    // Example: "core" = c + o + r + e → o+r+e is NOT Vietnamese pattern
                    // Example: "cura" = c + u + r + a → u+r+a IS Vietnamese (cửa)
                    // Example: "gasi" = g + a + s + i → a+s+i IS Vietnamese (gái)
                    // Example: "nafo" = n + a + f + o → a+f+o IS Vietnamese (nào)
                    if has_initial_consonant {
                        let (prev_vowel, _, _) = self.raw_input[i - 1];
                        // Vietnamese exceptions: diphthongs with tone modifier in middle
                        let is_vietnamese_pattern = match prev_vowel {
                            k if k == keys::U => next_key == keys::A || next_key == keys::O,
                            k if k == keys::A => {
                                next_key == keys::I || next_key == keys::Y || next_key == keys::O
                            }
                            k if k == keys::O => next_key == keys::I || next_key == keys::A,
                            _ => false,
                        };
                        if !is_vietnamese_pattern {
                            return true;
                        }
                    }
                }
            }
        }

        // Pattern 5: W at end after vowel → English (like "raw", "law", "saw", "view")
        // W as final is not valid Vietnamese, it's an English pattern
        // Exception: "uw" ending is Vietnamese (tuw → tư)
        // Exception: W modified a diphthong (oiw → ơi where OI is diphthong, W adds horn to O)
        if self.raw_input.len() >= 2 {
            let (last, _, _) = self.raw_input[self.raw_input.len() - 1];
            if last == keys::W {
                let (second_last, _, _) = self.raw_input[self.raw_input.len() - 2];
                // W after vowel (not U) at end is English: raw, law, saw
                // W after U is Vietnamese: tuw → tư
                if keys::is_vowel(second_last) && second_last != keys::U {
                    // Check if W was absorbed (modified existing vowel vs created new ư)
                    // "oiw" → "ơi": 3 chars → 2 chars (absorbed)
                    // "view" → "vieư": 4 chars → 4 chars (not absorbed)
                    let w_was_absorbed = self.buf.len() < self.raw_input.len();

                    // Count vowels before W in raw_input
                    let vowel_count = self.raw_input[..self.raw_input.len() - 1]
                        .iter()
                        .filter(|(k, _, _)| keys::is_vowel(*k))
                        .count();

                    // Only skip restore if BOTH conditions are true:
                    // 1. W was absorbed (actually modified an existing vowel)
                    // 2. There are 2+ vowels before W (diphthong like OI in "oiw")
                    // Otherwise, this is likely English (bow, view) - restore
                    if !(w_was_absorbed && vowel_count >= 2) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Auto-restore invalid Vietnamese to raw English on space
    ///
    /// Called when SPACE is pressed. If buffer has transforms but result is not
    /// valid Vietnamese, restore to original English + space.
    /// Example: "tẽt" (from typing "text") → "text " (restored + space)
    /// Example: "ễpct" (from typing "expect") → "expect " (restored + space)
    fn try_auto_restore_on_space(&self) -> Result {
        if let Some(mut raw_chars) = self.should_auto_restore(true) {
            // Add space at the end
            raw_chars.push(' ');
            // Backspace count = current buffer length (displayed chars)
            let backspace = self.buf.len() as u8;
            Result::send(backspace, &raw_chars)
        } else {
            Result::none()
        }
    }

    /// Auto-restore invalid Vietnamese to raw English on break key
    ///
    /// Called when punctuation/break key is pressed. If buffer has transforms
    /// but result is not valid Vietnamese, restore to original English.
    /// Does NOT include the break key (it's passed through by the app).
    /// Example: "ễpct" + comma → "expect" (comma added by app)
    fn try_auto_restore_on_break(&self) -> Result {
        if let Some(raw_chars) = self.should_auto_restore(true) {
            // Backspace count = current buffer length (displayed chars)
            let backspace = self.buf.len() as u8;
            Result::send(backspace, &raw_chars)
        } else {
            Result::none()
        }
    }

    /// Restore buffer to raw ASCII (undo all Vietnamese transforms)
    ///
    /// Called when ESC is pressed. Replaces transformed output with original keystrokes.
    /// Example: "tẽt" (from typing "text" in Telex) → "text"
    fn restore_to_raw(&self) -> Result {
        if self.raw_input.is_empty() || self.buf.is_empty() {
            return Result::none();
        }

        // Check if any transforms were applied
        let has_transforms = self
            .buf
            .iter()
            .any(|c| c.tone > 0 || c.mark > 0 || c.stroke);
        if !has_transforms {
            return Result::none();
        }

        // Build raw ASCII output from raw_input history
        let raw_chars: Vec<char> = self
            .raw_input
            .iter()
            .filter_map(|&(key, caps, shift)| utils::key_to_char_ext(key, caps, shift))
            .collect();

        if raw_chars.is_empty() {
            return Result::none();
        }

        // Backspace count = current buffer length (displayed chars)
        let backspace = self.buf.len() as u8;

        Result::send(backspace, &raw_chars)
    }

    /// Restore raw_input from buffer (for ESC restore to work after backspace-restore)
    fn restore_raw_input_from_buffer(&mut self, buf: &Buffer) {
        self.raw_input.clear();
        for c in buf.iter() {
            self.raw_input.push((c.key, c.caps, false));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Engine;
    use crate::utils::{telex, type_word, vni};

    const TELEX_BASIC: &[(&str, &str)] = &[
        ("as", "á"),
        ("af", "à"),
        ("ar", "ả"),
        ("ax", "ã"),
        ("aj", "ạ"),
        ("aa", "â"),
        // Issue #44: Breve deferred in open syllable until final consonant or mark
        ("aw", "ă"),   // standalone aw → ă
        ("awm", "ăm"), // breve applied when final consonant typed
        ("aws", "ắ"),  // breve applied when mark typed
        ("ee", "ê"),
        ("oo", "ô"),
        ("ow", "ơ"),
        ("uw", "ư"),
        ("dd", "đ"),
        // Mark after consonant
        ("tex", "tẽ"), // t + e + x(ngã) → tẽ
    ];

    const VNI_BASIC: &[(&str, &str)] = &[
        ("a1", "á"),
        ("a2", "à"),
        ("a3", "ả"),
        ("a4", "ã"),
        ("a5", "ạ"),
        ("a6", "â"),
        // Issue #44: Breve deferred in open syllable until final consonant or mark
        ("a8", "ă"),   // standalone a8 → ă
        ("a8m", "ăm"), // breve applied when final consonant typed
        ("a81", "ắ"),  // breve applied when mark typed
        ("e6", "ê"),
        ("o6", "ô"),
        ("o7", "ơ"),
        ("u7", "ư"),
        ("d9", "đ"),
    ];

    const TELEX_COMPOUND: &[(&str, &str)] =
        &[("duocw", "dươc"), ("nguoiw", "ngươi"), ("tuoiws", "tưới")];

    // ESC restore test cases: input with ESC (\x1b) → expected raw ASCII
    // ESC restores to exactly what user typed (including modifier keys)
    const TELEX_ESC_RESTORE: &[(&str, &str)] = &[
        ("text\x1b", "text"),     // tẽt → text
        ("user\x1b", "user"),     // úẻ → user
        ("esc\x1b", "esc"),       // éc → esc
        ("dd\x1b", "dd"),         // đ → dd (stroke restore)
        ("vieejt\x1b", "vieejt"), // việt → vieejt (all typed keys)
        ("Vieejt\x1b", "Vieejt"), // Việt → Vieejt (preserve case)
    ];

    const VNI_ESC_RESTORE: &[(&str, &str)] = &[
        ("a1\x1b", "a1"),         // á → a1
        ("vie65t\x1b", "vie65t"), // việt → vie65t
        ("d9\x1b", "d9"),         // đ → d9
    ];

    // Normal Vietnamese transforms apply
    const TELEX_NORMAL: &[(&str, &str)] = &[
        ("gox", "gõ"),      // Without prefix: "gox" → "gõ"
        ("tas", "tá"),      // Without prefix: Vietnamese transforms (s adds sắc)
        ("vieejt", "việt"), // Normal Vietnamese typing
    ];

    #[test]
    fn test_telex_basic() {
        telex(TELEX_BASIC);
    }

    #[test]
    fn test_vni_basic() {
        vni(VNI_BASIC);
    }

    #[test]
    fn test_telex_compound() {
        telex(TELEX_COMPOUND);
    }

    #[test]
    fn test_telex_esc_restore() {
        // ESC restore is disabled by default, enable it for this test
        for (input, expected) in TELEX_ESC_RESTORE {
            let mut e = Engine::new();
            e.set_esc_restore(true);
            let result = type_word(&mut e, input);
            assert_eq!(result, *expected, "[Telex] '{}' → '{}'", input, result);
        }
    }

    #[test]
    fn test_vni_esc_restore() {
        // ESC restore is disabled by default, enable it for this test
        for (input, expected) in VNI_ESC_RESTORE {
            let mut e = Engine::new();
            e.set_method(1);
            e.set_esc_restore(true);
            let result = type_word(&mut e, input);
            assert_eq!(result, *expected, "[VNI] '{}' → '{}'", input, result);
        }
    }

    #[test]
    fn test_telex_normal() {
        telex(TELEX_NORMAL);
    }
}
