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
    constants, english_dict, keys, telex_doubles,
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
    /// Flags byte:
    /// - bit 0 (0x01): key_consumed - if set, the trigger key should NOT be passed through
    ///   Used for shortcuts where the trigger key is part of the replacement
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
    /// Bracket as vowel: ] → ư, [ → ơ (Issue #159)
    BracketAsVowel,
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

/// Check if key is sentence-ending punctuation (. ! ?) but NOT Enter
/// Issue #185: Only set pending_capitalize after punctuation + space
#[inline]
fn is_sentence_ending_punctuation(key: u16, shift: bool) -> bool {
    key == keys::DOT
        || (shift && key == keys::N1) // !
        || (shift && key == keys::SLASH) // ?
}

/// Check if a break key should reset pending_capitalize
/// Neutral keys like quotes, parentheses, arrows should NOT reset (preserve pending)
/// Word-breaking keys like comma should reset
#[inline]
fn should_reset_pending_capitalize(key: u16, shift: bool) -> bool {
    // These neutral characters/keys should NOT reset pending_capitalize:
    // - Quotes: ' " (QUOTE with/without shift)
    // - Parentheses: ( ) (Shift+9, Shift+0)
    // - Brackets: [ ] { } (LBRACKET, RBRACKET with/without shift)
    // - Arrow keys: navigation shouldn't reset pending state
    // - Tab, ESC: navigation/cancel shouldn't reset pending state
    let is_neutral = key == keys::QUOTE
        || key == keys::LBRACKET
        || key == keys::RBRACKET
        || (shift && key == keys::N9)  // (
        || (shift && key == keys::N0)  // )
        || key == keys::LEFT
        || key == keys::RIGHT
        || key == keys::UP
        || key == keys::DOWN
        || key == keys::TAB
        || key == keys::ESC;

    // Reset for all other break keys (comma, semicolon, etc.)
    !is_neutral
}

/// Convert break key to its character representation
/// Handles both shifted and unshifted break characters for shortcut matching.
/// Examples: MINUS → '-', Shift+DOT → '>', Shift+MINUS → '_'
fn break_key_to_char(key: u16, shift: bool) -> Option<char> {
    if shift {
        // Shifted break characters
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
            keys::MINUS => Some('_'),
            keys::EQUAL => Some('+'),
            keys::SEMICOLON => Some(':'),
            keys::QUOTE => Some('"'),
            keys::COMMA => Some('<'),
            keys::DOT => Some('>'),
            keys::SLASH => Some('?'),
            keys::BACKSLASH => Some('|'),
            keys::LBRACKET => Some('{'),
            keys::RBRACKET => Some('}'),
            keys::BACKQUOTE => Some('~'),
            _ => None,
        }
    } else {
        // Unshifted break characters
        match key {
            keys::MINUS => Some('-'),
            keys::EQUAL => Some('='),
            keys::SEMICOLON => Some(';'),
            keys::QUOTE => Some('\''),
            keys::COMMA => Some(','),
            keys::DOT => Some('.'),
            keys::SLASH => Some('/'),
            keys::BACKSLASH => Some('\\'),
            keys::LBRACKET => Some('['),
            keys::RBRACKET => Some(']'),
            keys::BACKQUOTE => Some('`'),
            _ => None,
        }
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
    /// Enable bracket shortcuts: ] → ư, [ → ơ (Issue #159)
    bracket_shortcut: bool,
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
    /// Issue #133: Pending horn position on 'u' in "uơ" pattern
    /// When "uo" + 'w' is typed at end of syllable, only 'o' gets horn initially.
    /// If a final consonant/vowel is added, also apply horn to 'u'.
    /// Examples: "huow" → "huơ" (stays), "duow" + "c" → "dược" (u gets horn)
    pending_u_horn_pos: Option<usize>,
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
    /// Tracks if circumflex was applied from V+C+V pattern by vowel trigger (not mark key)
    /// Example: "toto" → "tôt" (second 'o' triggers circumflex on first 'o')
    /// Used for auto-restore: if no mark follows, restore on space (e.g., "toto " → "toto ")
    had_vowel_triggered_circumflex: bool,
    /// Tracks if circumflex was REVERTED by third vowel (aa→â, aaa→aa)
    /// Example: "dataa" → "dât" (after 4th key), typing 5th 'a' reverts to "data"
    /// Used in build_raw_chars to collapse double vowel at end for restore
    had_circumflex_revert: bool,
    /// Tracks if ANY Telex transform was applied (tone, mark, or stroke)
    /// Used for whitelist-based auto-restore to English words
    had_telex_transform: bool,
    /// Stores raw_input string when telex double pattern is detected (BEFORE modification)
    /// For stroke revert (ddd→dd), raw_input is modified to remove one 'd', but we need
    /// the original for whitelist lookup (e.g., "daddy" not "dady")
    telex_double_raw: Option<String>,
    /// Stores length of raw_input at time telex_double_raw was stored
    /// Used to append subsequent chars typed after revert
    telex_double_raw_len: usize,
    /// Issue #107: Special character prefix for shortcut matching
    /// When a shifted symbol (like #, @, $) is typed first, store it here
    /// so shortcuts like "#fne" can match even though # is normally a break char
    /// Extended: Now accumulates multiple break chars for shortcuts like "->" → "→"
    shortcut_prefix: String,
    /// Buffer was just restored from DELETE - clear on next letter input
    /// This prevents typing after restore from appending to old buffer
    restored_pending_clear: bool,
    /// Auto-capitalize first letter after sentence-ending punctuation
    /// Triggers: . ! ? Enter → next letter becomes uppercase
    auto_capitalize: bool,
    /// Pending capitalize state: set after sentence-ending punctuation + space
    pending_capitalize: bool,
    /// Tracks if auto-capitalize was just used on the current word
    /// Used to restore pending_capitalize when user deletes the capitalized letter
    auto_capitalize_used: bool,
    /// Tracks if we just saw sentence-ending punctuation (. ! ?)
    /// Only set pending_capitalize when space/Enter follows
    /// Issue #185: don't capitalize immediately after punctuation (e.g., google.com)
    saw_sentence_ending: bool,
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
            bracket_shortcut: false,    // Default: OFF (Issue #159)
            esc_restore_enabled: false, // Default: OFF (user request)
            free_tone_enabled: false,
            modern_tone: true,           // Default: modern style (hoà, thuý)
            english_auto_restore: false, // Default: OFF (experimental feature)
            word_history: WordHistory::new(),
            spaces_after_commit: 0,
            pending_breve_pos: None,
            pending_u_horn_pos: None,
            stroke_reverted: false,
            had_mark_revert: false,
            pending_mark_revert_pop: false,
            had_any_transform: false,
            had_vowel_triggered_circumflex: false,
            had_circumflex_revert: false,
            had_telex_transform: false,
            telex_double_raw: None,
            telex_double_raw_len: 0,
            shortcut_prefix: String::new(),
            restored_pending_clear: false,
            auto_capitalize: false, // Default: OFF
            pending_capitalize: false,
            auto_capitalize_used: false,
            saw_sentence_ending: false,
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

    /// Set whether bracket shortcuts are enabled: ] → ư, [ → ơ (Issue #159)
    pub fn set_bracket_shortcut(&mut self, enabled: bool) {
        self.bracket_shortcut = enabled;
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

    /// Set whether to enable auto-capitalize after sentence-ending punctuation
    pub fn set_auto_capitalize(&mut self, enabled: bool) {
        self.auto_capitalize = enabled;
        if !enabled {
            self.pending_capitalize = false;
            self.saw_sentence_ending = false;
        }
    }

    pub fn shortcuts(&self) -> &ShortcutTable {
        &self.shortcuts
    }

    pub fn shortcuts_mut(&mut self) -> &mut ShortcutTable {
        &mut self.shortcuts
    }

    /// Debug: get buffer length
    pub fn debug_buffer_len(&self) -> usize {
        self.buf.len()
    }

    /// Debug: get raw_input length (alias for raw_input_len)
    pub fn debug_raw_input_len(&self) -> usize {
        self.raw_input.len()
    }

    /// Debug: check had_any_transform flag
    pub fn debug_had_any_transform(&self) -> bool {
        self.had_any_transform
    }

    /// Debug: get buffer content as string
    pub fn debug_buffer_string(&self) -> String {
        self.buf.to_full_string()
    }

    /// Debug: dump full buffer state
    pub fn debug_buffer_state(&self) -> String {
        let mut result = String::new();
        for (i, c) in self.buf.iter().enumerate() {
            result.push_str(&format!(
                "[{}] key={} tone={} mark={} stroke={}\n",
                i, c.key, c.tone, c.mark, c.stroke
            ));
        }
        result
    }

    /// Debug: check had_mark_revert flag
    pub fn debug_had_mark_revert(&self) -> bool {
        self.had_mark_revert
    }

    /// Debug: dump raw_input
    pub fn debug_raw_input(&self) -> String {
        self.raw_input
            .iter()
            .map(|(k, c, s)| format!("({},{},{})", k, c, s))
            .collect::<Vec<_>>()
            .join(" ")
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
        // Issue #129: Process shortcuts even when IME is disabled
        // Only bypass completely for Ctrl/Cmd modifier keys
        if ctrl {
            self.clear();
            self.word_history.clear();
            self.spaces_after_commit = 0;
            return Result::none();
        }

        // When IME is disabled, process shortcuts but skip Vietnamese transforms
        // This allows both word shortcuts (btw → by the way) and symbol shortcuts (-> → →)
        if !self.enabled {
            // Clear Vietnamese state
            self.buf.clear();
            self.raw_input.clear();
            self.word_history.clear();
            self.spaces_after_commit = 0;

            // Word boundary keys (Space, Enter): check for word shortcuts
            if key == keys::SPACE || key == keys::RETURN || key == keys::ENTER {
                if !self.shortcut_prefix.is_empty() {
                    let input_method = self.current_input_method();
                    if let Some(m) = self.shortcuts.try_match_for_method(
                        &self.shortcut_prefix,
                        None,
                        true, // is_word_boundary = true for word shortcuts
                        input_method,
                    ) {
                        let output: Vec<char> = m.output.chars().collect();
                        let backspace_count = m.backspace_count as u8;
                        self.shortcut_prefix.clear();
                        // For Space, include space in output; for Enter, don't
                        if key == keys::SPACE {
                            let mut output_with_space = output;
                            output_with_space.push(' ');
                            return Result::send(backspace_count, &output_with_space);
                        } else {
                            return Result::send(backspace_count, &output);
                        }
                    }
                }
                self.shortcut_prefix.clear();
                return Result::none();
            }

            // Break keys (punctuation): check for immediate shortcuts like "->"
            if keys::is_break_ext(key, shift) {
                if let Some(ch) = break_key_to_char(key, shift) {
                    self.shortcut_prefix.push(ch);

                    let input_method = self.current_input_method();
                    if let Some(m) = self.shortcuts.try_match_for_method(
                        &self.shortcut_prefix,
                        None,
                        false,
                        input_method,
                    ) {
                        let output: Vec<char> = m.output.chars().collect();
                        let backspace_count = (m.backspace_count as u8).saturating_sub(1);
                        self.shortcut_prefix.clear();
                        return Result::send_consumed(backspace_count, &output);
                    }
                    return Result::none();
                }
                // Break key without char mapping (Tab, arrows, etc.) - clear and pass through
                self.shortcut_prefix.clear();
                return Result::none();
            }

            // Letter and number keys: accumulate for word shortcuts (e.g., "btw", "f1", "a1")
            if let Some(ch) = utils::key_to_char(key, caps) {
                self.shortcut_prefix.push(ch);
                return Result::none();
            }

            // Unknown keys: clear shortcut prefix and pass through
            self.shortcut_prefix.clear();
            return Result::none();
        }

        // Check for word boundary shortcuts ONLY on SPACE
        // Also auto-restore invalid Vietnamese to raw English
        if key == keys::SPACE {
            // Handle pending mark revert pop on space (end of word)
            // When user types "simss" → mark reverted → raw should be "sims" not "simss"
            // This is deferred from the revert action to support "issue" pattern
            //
            // EXCEPTION: Double 'f' should NOT pop because 'ff' is very common in English
            // (off, offer, office, coffee, effect, etc.). Keeping both 'f's in raw_input
            // ensures auto-restore produces "off" not "of".
            if self.pending_mark_revert_pop {
                self.pending_mark_revert_pop = false;
                // Pop the consumed mark key from raw_input
                // raw_input: [..., mark_key, revert_key] → [..., revert_key]
                // SKIP for double 'f' or double 's' to preserve 'ff'/'ss' for auto-restore
                // because these are very common in English (off, offer, guess, miss, class, etc.)
                if self.raw_input.len() >= 2 {
                    let len = self.raw_input.len();
                    let (last_key, _, _) = self.raw_input[len - 1];
                    let (second_last_key, _, _) = self.raw_input[len - 2];
                    let is_double_f = last_key == keys::F && second_last_key == keys::F;
                    let is_double_s = last_key == keys::S && second_last_key == keys::S;

                    if !is_double_f && !is_double_s {
                        let revert_key = self.raw_input.pop();
                        self.raw_input.pop(); // mark_key (consumed)
                        if let Some(k) = revert_key {
                            self.raw_input.push(k);
                        }
                    }
                }
            }

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
            self.auto_capitalize_used = false; // Reset on word commit

            // Issue #185: Set pending_capitalize on space AFTER sentence-ending punctuation
            // This ensures "google.com" doesn't capitalize, but "ok. ban" does
            if self.auto_capitalize && self.saw_sentence_ending {
                self.pending_capitalize = true;
                // Keep saw_sentence_ending for multiple spaces (e.g., "ok.  ban")
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

        // Issue #159: In Telex mode, `]` → ư and `[` → ơ
        // caps affects revert: ]] → ], uppercase (Shift/CapsLock) → }
        if self.method == 0 && (key == keys::RBRACKET || key == keys::LBRACKET) {
            if let Some(result) = self.try_bracket_as_vowel(key, caps) {
                return result;
            }
        }

        // Other break keys (punctuation, arrows, etc.)
        // Also trigger auto-restore for invalid Vietnamese before clearing
        // Use is_break_ext to handle shifted symbols like @, !, #, etc.
        if keys::is_break_ext(key, shift) {
            // Issue #107 + Bug #11: When buffer is empty AND we're at true start of input
            // (no word history), accumulate break chars for shortcuts.
            // This allows shortcuts like "#fne", "->", "=>" to work.
            // BUT: if there's word history (user just typed "du "), break chars should
            // clear history as before, not accumulate.
            let at_true_start =
                self.buf.is_empty() && self.word_history.len == 0 && self.spaces_after_commit == 0;

            // Also continue accumulating if we already started a prefix
            let continuing_prefix = self.buf.is_empty() && !self.shortcut_prefix.is_empty();

            if at_true_start || continuing_prefix {
                // Reset has_non_letter_prefix when starting a new shortcut at true start
                // This ensures shortcuts like "->" work after DELETE cleared the buffer
                if at_true_start {
                    self.has_non_letter_prefix = false;
                }

                // Try to get the character for this break key
                if let Some(ch) = break_key_to_char(key, shift) {
                    self.shortcut_prefix.push(ch);

                    // Check for immediate shortcut match
                    let input_method = self.current_input_method();
                    if let Some(m) = self.shortcuts.try_match_for_method(
                        &self.shortcut_prefix,
                        None,
                        false,
                        input_method,
                    ) {
                        // Found a match! Send the replacement with key_consumed flag
                        // Note: backspace_count - 1 because current key hasn't been typed yet
                        // Example: "->" trigger has backspace_count=2, but only '-' is on screen
                        let output: Vec<char> = m.output.chars().collect();
                        let backspace_count = (m.backspace_count as u8).saturating_sub(1);
                        self.shortcut_prefix.clear();
                        return Result::send_consumed(backspace_count, &output);
                    }

                    // Issue #185: Only set saw_sentence_ending for punctuation (not Enter)
                    // pending_capitalize will be set when space follows
                    if self.auto_capitalize && is_sentence_ending_punctuation(key, shift) {
                        self.saw_sentence_ending = true;
                    } else if self.auto_capitalize && (key == keys::RETURN || key == keys::ENTER) {
                        // Enter = newline = immediate capitalize (no space needed)
                        self.pending_capitalize = true;
                        self.saw_sentence_ending = false;
                    }
                    return Result::none(); // Let the char pass through, keep accumulating
                }
            }

            // Issue #185: Only set saw_sentence_ending for punctuation (not Enter)
            // pending_capitalize will be set when space follows
            if self.auto_capitalize && is_sentence_ending_punctuation(key, shift) {
                self.saw_sentence_ending = true;
            } else if self.auto_capitalize && (key == keys::RETURN || key == keys::ENTER) {
                // Enter = newline = immediate capitalize (no space needed)
                self.pending_capitalize = true;
                self.saw_sentence_ending = false;
            } else if self.auto_capitalize && should_reset_pending_capitalize(key, shift) {
                // Reset pending for word-breaking keys (comma, semicolon, etc.)
                // But preserve pending for neutral keys (quotes, parentheses, brackets)
                self.pending_capitalize = false;
                self.saw_sentence_ending = false;
            }
            self.auto_capitalize_used = false; // Reset on word boundary

            // Issue #167: Check for word boundary shortcuts on punctuation and ENTER
            // Example: "ko." → "không." or "ko<Enter>" → "không<Enter>"
            // ENTER doesn't have a printable char, so check it separately
            let trigger_char = if key == keys::RETURN || key == keys::ENTER {
                Some('\n') // ENTER: use newline as trigger (won't be appended)
            } else {
                break_key_to_char(key, shift)
            };
            if let Some(ch) = trigger_char {
                let shortcut_result = self.try_word_boundary_shortcut_with_char(ch);
                if shortcut_result.action != 0 {
                    self.clear();
                    self.word_history.clear();
                    self.spaces_after_commit = 0;
                    return shortcut_result;
                }
            }

            let restore_result = self.try_auto_restore_on_break();
            self.clear();
            self.word_history.clear();
            self.spaces_after_commit = 0;

            // Issue #130: After clearing buffer, store break char as potential shortcut prefix
            // This allows shortcuts like "->" to work after "abc->" (where "-" clears "abc")
            // Example: type "→abc->" should produce "→abc→"
            if let Some(ch) = break_key_to_char(key, shift) {
                self.shortcut_prefix.push(ch);
            }

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
                        // Mark that buffer was restored - if user types new letter,
                        // clear buffer first (they want fresh word, not append)
                        self.restored_pending_clear = true;
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
            // Only reset restored_pending_clear when buffer is empty
            // (user finished deleting restored word completely)
            // If buffer still has chars, user might think they cleared everything
            // but actually didn't - let them start fresh on next letter input
            if self.buf.is_empty() {
                self.restored_pending_clear = false;
                // Restore pending_capitalize if user deleted the auto-capitalized letter
                // This allows: ". B" → delete B → ". " → type again → auto-capitalizes
                if self.auto_capitalize_used {
                    self.pending_capitalize = true;
                    self.auto_capitalize_used = false;
                }
            }
            return Result::none();
        }

        // After DELETE restore, determine if user wants to:
        // 1. Continue editing restored word (add tone/mark) - vowels, mark keys, tone keys
        // 2. Start fresh word - regular consonants (not mark/tone keys)
        // This allows "cha" + restore + "f" → "chà" (f is mark key)
        // But "cha" + restore + "m" → "m..." (m is consonant, start fresh)
        if self.restored_pending_clear && keys::is_letter(key) {
            let m = input::get(self.method);
            let is_mark_or_tone = m.mark(key).is_some() || m.tone(key).is_some();
            if keys::is_consonant(key) && !is_mark_or_tone {
                // Regular consonant (not mark/tone key) = user starting new word
                self.clear();
            }
            // Reset flag regardless - user is now actively typing
            self.restored_pending_clear = false;
        }

        // Auto-capitalize: force uppercase for first letter after sentence-ending punctuation
        let was_auto_capitalized = self.pending_capitalize && keys::is_letter(key) && !caps;
        let effective_caps = if self.pending_capitalize && keys::is_letter(key) {
            self.pending_capitalize = false;
            self.saw_sentence_ending = false; // Reset after capitalizing
            self.auto_capitalize_used = true; // Track that we used auto-capitalize
            true // Force uppercase
        } else {
            // Reset pending on number (e.g., "1.5" should not capitalize "5")
            if self.pending_capitalize && keys::is_number(key) {
                self.pending_capitalize = false;
                self.saw_sentence_ending = false;
                self.auto_capitalize_used = false; // Number after punctuation, reset
            }
            // Issue #185: Reset saw_sentence_ending when letter is typed without space
            // e.g., "google.com" - 'c' typed after '.' without space, don't capitalize
            if self.saw_sentence_ending && keys::is_letter(key) {
                self.saw_sentence_ending = false;
            }
            caps
        };

        // Record raw keystroke for ESC restore (letters and numbers only)
        if keys::is_letter(key) || keys::is_number(key) {
            self.raw_input.push((key, effective_caps, shift));
        }

        let result = self.process(key, effective_caps, shift);

        // If auto-capitalize triggered for first letter of a new word and process returned none,
        // we need to send the uppercase character since the original key was lowercase
        if was_auto_capitalized && result.action == Action::None as u8 && self.buf.len() == 1 {
            if let Some(ch) = crate::utils::key_to_char(key, true) {
                return Result::send(0, &[ch]);
            }
        }

        result
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
                //
                // EXCEPTION: Double tone modifiers should NOT pop because they're common in English:
                // - 'ff': offline, offer, office, coffee, effect, staff, stuff, etc.
                // - 'ss': harassment, mississippi, class, miss, pass, boss, etc.
                // - 'rr': diarrhea, arrhythmia, error, mirror, horror, terror, etc.
                // Auto-restore at word-end will handle these correctly by checking whitelist.
                if self.raw_input.len() >= 3 {
                    let len = self.raw_input.len();
                    let (revert_key, _, _) = self.raw_input[len - 2];
                    let (mark_key, _, _) = self.raw_input[len - 3];
                    let is_double_f = revert_key == keys::F && mark_key == keys::F;
                    let is_double_s = revert_key == keys::S && mark_key == keys::S;
                    let is_double_r = revert_key == keys::R && mark_key == keys::R;

                    if !is_double_f && !is_double_s && !is_double_r {
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
            let raw_keys: Vec<u16> = self.raw_input.iter().map(|&(k, _, _)| k).collect();

            // Also check if the buffer (with stroke) + new key would be valid Vietnamese
            // This handles delayed stroke patterns like "dadu" → "đau":
            // - raw_input = [d, a, d, u] (invalid as "dadu")
            // - But buffer + key = [đ, a] + [u] = "đau" (valid)
            // If buffer + key is valid, don't revert the stroke
            let mut buf_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
            buf_keys.push(key);

            if !is_valid(&raw_keys) && !is_valid(&buf_keys) {
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
            if let Some(result) = self.try_stroke(key, caps) {
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
    /// The `trigger_char` is appended to the output (space for space, punctuation for punctuation)
    fn try_word_boundary_shortcut_with_char(&mut self, trigger_char: char) -> Result {
        // Issue #107: Allow shortcuts with special char prefix (like "#fne")
        // If shortcut_prefix is set, we still try to match even with empty buffer
        if self.buf.is_empty() && self.shortcut_prefix.is_empty() {
            return Result::none();
        }

        // Don't trigger shortcut if word has non-letter prefix (like "149k")
        // But DO allow shortcut_prefix (like "#fne") - that's intentional
        if self.has_non_letter_prefix {
            return Result::none();
        }

        // Build full trigger string including shortcut_prefix if present
        let full_trigger = if self.shortcut_prefix.is_empty() {
            self.buf.to_full_string()
        } else {
            format!("{}{}", self.shortcut_prefix, self.buf.to_full_string())
        };

        let input_method = self.current_input_method();

        // Check for word boundary shortcut match
        // For SPACE: append to output (space is "consumed" via Result::forward later)
        // For punctuation: pass None - don't append, platform layer types it normally
        // (This matches auto-restore behavior which also doesn't append break char)
        let key_char = if trigger_char == ' ' {
            Some(' ')
        } else {
            None // Punctuation: don't append, let platform type it
        };
        if let Some(m) =
            self.shortcuts
                .try_match_for_method(&full_trigger, key_char, true, input_method)
        {
            let output: Vec<char> = m.output.chars().collect();
            // backspace_count = trigger.len() which already includes prefix (e.g., "#fne" = 4)
            return Result::send(m.backspace_count as u8, &output);
        }

        Result::none()
    }

    /// Try word boundary shortcuts (triggered by space)
    fn try_word_boundary_shortcut(&mut self) -> Result {
        self.try_word_boundary_shortcut_with_char(' ')
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
            // Track ww pattern for whitelist-based restore
            self.had_telex_transform = true;
            // Store raw_input BEFORE modification for whitelist lookup
            self.telex_double_raw = Some(self.get_raw_input_string_preserve_case());
            // Get original case from buffer before popping
            let original_caps = self.buf.last().map(|c| c.caps).unwrap_or(caps);
            self.buf.pop();
            self.buf.push(Char::new(keys::W, original_caps));
            // Fix raw_input: "ww" typed → raw has [w,w] but buffer is "w"
            // Remove the shortcut-triggering 'w' from raw_input so restore works correctly
            // raw_input: [a, w, w] → [a, w] (remove first 'w' that triggered shortcut)
            // This ensures "awwait" → "await" not "awwait" on auto-restore
            if self.raw_input.len() >= 2 {
                let current = self.raw_input.pop(); // current 'w' (just added)
                self.raw_input.pop(); // shortcut-trigger 'w' (consumed, discard)
                if let Some(c) = current {
                    self.raw_input.push(c);
                }
            }
            // Store length AFTER modification
            self.telex_double_raw_len = self.raw_input.len();
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
    fn try_stroke(&mut self, key: u16, caps: bool) -> Option<Result> {
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
                    // Add another 'd' as normal char (preserve caps state)
                    self.buf.push(Char::new(key, caps));
                    self.last_transform = None;
                    // Mark that stroke was reverted - subsequent 'd' keys will be normal letters
                    self.stroke_reverted = true;
                    // Track dd pattern for whitelist-based restore
                    self.had_telex_transform = true;
                    // Store raw_input BEFORE modification for whitelist lookup
                    // For "daddy": raw_input = [d,a,d,d] → store "dadd"
                    self.telex_double_raw = Some(self.get_raw_input_string_preserve_case());
                    // Fix raw_input: "ddd" typed → raw has [d,d,d] but buffer is "dd"
                    // Remove the stroke-triggering 'd' from raw_input so restore works correctly
                    // raw_input: [d, d, d] → [d, d] (remove middle 'd' that triggered stroke)
                    // This ensures "didd" → "did" not "didd" on auto-restore
                    if self.raw_input.len() >= 2 {
                        let current = self.raw_input.pop(); // current 'd' (just added)
                        self.raw_input.pop(); // stroke-trigger 'd' (consumed, discard)
                        if let Some(c) = current {
                            self.raw_input.push(c);
                        }
                    }
                    // Store length AFTER modification - for "daddy": [d,a,d] → len=3
                    // Subsequent chars (y) start at position 3
                    self.telex_double_raw_len = self.raw_input.len();
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
                    // Add another 'd' as normal char (preserve caps state)
                    self.buf.push(Char::new(key, caps));
                    self.last_transform = None;
                    // Mark that stroke was reverted - subsequent 'd' keys will be normal letters
                    self.stroke_reverted = true;
                    // Track dd pattern for whitelist-based restore
                    self.had_telex_transform = true;
                    // Store raw_input BEFORE modification for whitelist lookup
                    self.telex_double_raw = Some(self.get_raw_input_string_preserve_case());
                    // Fix raw_input same as above
                    if self.raw_input.len() >= 2 {
                        let current = self.raw_input.pop();
                        self.raw_input.pop();
                        if let Some(c) = current {
                            self.raw_input.push(c);
                        }
                    }
                    // Store length AFTER modification
                    self.telex_double_raw_len = self.raw_input.len();
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
                // - The triggering key is 'd' AND buffer is vowels-only after initial 'd'
                //   This allows "did" → "đi", "dod" → "đo", "duod" → "đuo", etc.
                // This prevents "de" + "d" → "đe" while allowing:
                // - "dods" → "đó" (mark key triggers stroke)
                // - "dojd" → "đọ" (mark already present, stroke applies immediately)
                // - "did" → "đi" (d triggers stroke on short open syllable)
                // - "duod" → "đuo" (d triggers stroke on diphthong open syllable)
                let syllable = syllable::parse(&buffer_keys);
                let has_mark_applied = self.buf.iter().any(|c| c.mark > 0);
                // Allow 'd' to trigger immediate stroke on open syllables with d + vowels only
                // Examples: "di" (len 2), "duo" (len 3), "dua" (len 3), "duoi" (len 4)
                let is_d_vowels_only_pattern = key == keys::D
                    && self.buf.len() >= 2
                    && self.buf.iter().skip(1).all(|c| keys::is_vowel(c.key));
                if syllable.final_c.is_empty() && !has_mark_applied && !is_d_vowels_only_pattern {
                    // Open syllable without mark, not d+vowels pattern - defer stroke decision
                    return None;
                }

                // Track if this is a short pattern stroke (can be reverted later)
                // Only revertible if no mark applied - mark confirms Vietnamese intent
                (0, is_d_vowels_only_pattern && !has_mark_applied)
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
        self.had_telex_transform = true; // dd pattern detected
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

        // Check for invalid "-ing" rhyme: Vietnamese uses "-inh", NOT "-ing" with tone
        // Examples: "thíng" is invalid (things), but "tính" is valid
        // If vowel is 'i' and final is 'ng', reject tone marks
        if !self.free_tone_enabled {
            let syllable = syllable::parse(&buffer_keys);
            if syllable.vowel.len() == 1 && syllable.final_c.len() == 2 {
                let vowel_key = buffer_keys[syllable.vowel[0]];
                let final_keys = [
                    buffer_keys[syllable.final_c[0]],
                    buffer_keys[syllable.final_c[1]],
                ];
                // i + ng = invalid Vietnamese rhyme for tone marks
                if vowel_key == keys::I && final_keys == [keys::N, keys::G] {
                    return None;
                }
            }
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
                        // Issue #133: Check if "uo" pattern is at end of syllable (no final)
                        // If no final consonant/vowel after "uo", only apply horn to 'o'
                        // Examples: "huow" → "huơ", "khuow" → "khuơ"
                        // But: "duowc" → "dược", "muowif" → "mười" (both get horn)
                        let is_uo_pattern = c1.key == keys::U && c2.key == keys::O;
                        let has_final = self.buf.get(pos2 + 1).is_some();

                        if is_uo_pattern && !has_final {
                            // "uơ" pattern - only 'o' gets horn initially
                            // Set pending so 'u' gets horn if final consonant/vowel is added
                            target_positions.push(pos2);
                            self.pending_u_horn_pos = Some(pos1);
                        } else {
                            // "ươ" pattern (or has final) - both get horn
                            target_positions.push(pos1);
                            target_positions.push(pos2);
                            self.pending_u_horn_pos = None;
                        }
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

                // Issue #312: If any vowel already has a tone (horn/circumflex/breve),
                // don't trigger same-vowel circumflex. The typed vowel should append raw.
                // Example: "chưa" + "a" → "chưaa" (NOT "chưâ")
                if is_telex_circumflex {
                    let any_vowel_has_tone = self
                        .buf
                        .iter()
                        .filter(|c| keys::is_vowel(c.key))
                        .any(|c| c.has_tone());

                    if any_vowel_has_tone {
                        // Skip circumflex, let the vowel append as raw letter
                        return None;
                    }

                    // Check if buffer has multiple vowel types and any has a mark
                    // Skip circumflex if it would create invalid diphthong (like ôà, âo)
                    // But allow if circumflex creates valid pattern (like uê, iê, yê)
                    // Examples:
                    // - "toà" + "a" → [O,A], âo invalid → skip → "toàa"
                    // - "ué" + "e" → [U,E], uê valid → allow → "uế"
                    let vowel_chars: Vec<_> =
                        self.buf.iter().filter(|c| keys::is_vowel(c.key)).collect();

                    let has_any_mark = vowel_chars.iter().any(|c| c.has_mark());
                    let unique_vowel_types: std::collections::HashSet<u16> =
                        vowel_chars.iter().map(|c| c.key).collect();
                    let has_multiple_vowel_types = unique_vowel_types.len() > 1;

                    if has_any_mark && has_multiple_vowel_types {
                        // Check if circumflex on V2 (the key) creates a valid pattern
                        // Valid V2 circumflex patterns: iê, uê, yê, uô
                        // Invalid: oa→oâ, ao→âo, ae→âe, etc.
                        let other_vowel = unique_vowel_types.iter().find(|&&v| v != key).copied();

                        // Check if this is a same-vowel trigger for V1 circumflex
                        // Example: "dausa" (d-á-u + a) → circumflex on first 'a' → "dấu"
                        // The trigger 'a' matches existing 'a' in buffer
                        let is_same_vowel_trigger = unique_vowel_types.contains(&key);

                        // V1 circumflex patterns: circumflex on FIRST vowel of diphthong
                        // These patterns have the trigger vowel + another vowel forming valid diphthong
                        // âu, ây (A with circumflex + U/Y)
                        // êu (E with circumflex + U) - already in V1_CIRCUMFLEX_REQUIRED
                        // ôi (O with circumflex + I)
                        let v1_circumflex_diphthongs: &[[u16; 2]] = &[
                            [keys::A, keys::U], // âu - "dấu"
                            [keys::A, keys::Y], // ây - "dây"
                            [keys::E, keys::U], // êu - "nếu"
                            [keys::O, keys::I], // ôi - "tối"
                        ];

                        let is_valid_v1_circumflex = is_same_vowel_trigger
                            && other_vowel
                                .is_some_and(|v| v1_circumflex_diphthongs.contains(&[key, v]));

                        // Patterns where circumflex on V2 is valid
                        let v2_circumflex_valid: &[[u16; 2]] = &[
                            [keys::I, keys::E], // iê
                            [keys::U, keys::E], // uê
                            [keys::Y, keys::E], // yê
                            [keys::U, keys::O], // uô
                        ];

                        let is_valid_v2_circumflex =
                            other_vowel.is_some_and(|v| v2_circumflex_valid.contains(&[v, key]));

                        if !is_valid_v2_circumflex && !is_valid_v1_circumflex {
                            // Invalid pattern → skip circumflex
                            return None;
                        }
                    }

                    // Check if adding this vowel would create a valid triphthong
                    // If so, skip circumflex and let the vowel append raw
                    // Example: "oe" + "o" → [O, E, O] = "oeo" triphthong → skip circumflex
                    // BUT: Only check this if the last char in buffer is a vowel
                    // If there's a consonant at the end (e.g., "boem"), then same-vowel
                    // trigger applies instead of triphthong building
                    let last_is_vowel = self.buf.last().is_some_and(|c| keys::is_vowel(c.key));

                    if last_is_vowel {
                        let vowels: Vec<u16> = self
                            .buf
                            .iter()
                            .filter(|c| keys::is_vowel(c.key))
                            .map(|c| c.key)
                            .collect();

                        if vowels.len() == 2 {
                            let potential_triphthong = [vowels[0], vowels[1], key];
                            if constants::VALID_TRIPHTHONGS.contains(&potential_triphthong) {
                                // This would create a valid triphthong, skip circumflex
                                return None;
                            }
                        }

                        // Check for V1-V2-V1 pattern in last 2 vowels + new key
                        // Example: "queue" has buffer vowels [U, E, U], new key = E
                        // Last 2 vowels = [E, U], new key = E → pattern is E-U-E (V1-V2-V1)
                        // ONLY block when:
                        // 1. NO Vietnamese indicators present (mark/stroke)
                        // 2. There's a consonant initial (foreign word pattern)
                        // 3. NOT a valid Vietnamese triphthong pattern
                        // This allows: "oio" → "ôi" (no initial, valid VN interjection)
                        // This allows: "hieu" + e → "hiêu" (iêu is valid VN triphthong)
                        // But blocks: "queue" → "quêu" (has "qu" initial, foreign word)
                        let has_vn_indicator = self.buf.iter().any(|c| c.mark > 0 || c.stroke);
                        let has_initial =
                            self.buf.get(0).is_some_and(|c| keys::is_consonant(c.key));

                        if !has_vn_indicator && has_initial && vowels.len() >= 2 {
                            let last_two = &vowels[vowels.len() - 2..];
                            let v1 = last_two[0]; // second-to-last vowel
                            let v2 = last_two[1]; // last vowel
                                                  // V1-V2-V1 pattern: new key matches v1 but not v2
                            if key == v1 && key != v2 {
                                // Exception: Allow circumflex for valid Vietnamese triphthongs
                                // e.g., [i, e, u] = iêu (hiểu), [y, e, u] = yêu, [u, e, u] = uêu (nguều)
                                // These require circumflex on E (middle vowel)
                                // The trigger 'e' is the same as v1, which triggers circumflex
                                //
                                // BUT: Exclude Q + U pattern (like "queue")
                                // In Vietnamese, Q only appears as part of "qu" initial cluster
                                // If initial is Q and first vowel is U, it's English (queue, quest)
                                let initial_q = self.buf.get(0).is_some_and(|c| c.key == keys::Q);
                                let first_vowel_u = vowels.first().is_some_and(|&v| v == keys::U);
                                let is_english_qu_pattern = initial_q && first_vowel_u;

                                let is_valid_vn_triphthong = vowels.len() == 3
                                    && !is_english_qu_pattern
                                    && constants::VALID_TRIPHTHONGS
                                        .contains(&[vowels[0], vowels[1], vowels[2]]);

                                // Issue #183: Also allow V1-V2 diphthongs requiring circumflex on V1
                                // e.g., "neue" → [e, u] = êu (nếu), "xaua" → [a, u] = âu (xấu)
                                // When typing the second V1, it should trigger circumflex on first V1
                                // BUT: Exclude English "qu" patterns (like "queue")
                                let v1_circumflex_diphthongs: &[[u16; 2]] = &[
                                    [keys::A, keys::U], // âu - "dấu", "xấu"
                                    [keys::A, keys::Y], // ây - "dây"
                                    [keys::E, keys::U], // êu - "nếu", "kêu"
                                    [keys::O, keys::I], // ôi - "tối"
                                ];
                                let is_valid_v1_circumflex_diphthong = !is_english_qu_pattern
                                    && v1_circumflex_diphthongs.contains(&[v1, v2]);

                                if !is_valid_vn_triphthong && !is_valid_v1_circumflex_diphthong {
                                    return None;
                                }
                            }
                        }
                    }
                }

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
                                // Check if there's a NON-ADJACENT vowel between target and final
                                // "teacher": e-a-ch has 'a' between first 'e' and 'ch' → block
                                // "hongo": o-ng has no vowel between 'o' and 'ng' → allow
                                // "dau": a-u is a diphthong (adjacent vowels) → allow
                                // Adjacent vowels (position i+1) form diphthongs, not separate syllables
                                let has_non_adjacent_vowel = (i + 2..self.buf.len()).any(|j| {
                                    self.buf.get(j).is_some_and(|ch| keys::is_vowel(ch.key))
                                });

                                if has_non_adjacent_vowel {
                                    // A vowel exists after the adjacent position → different syllable
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
                                // But still need to check: if there's an adjacent vowel, it must
                                // form a valid diphthong with the target. Otherwise skip.
                                // Example: "teacher" has 'e' at i=1 with adjacent 'a' at i+1,
                                // but "ea" is NOT a valid Vietnamese diphthong → skip
                                if is_double_final && all_are_valid_finals {
                                    // Check for adjacent vowel that doesn't form valid diphthong
                                    let adjacent_vowel_key = (i + 1 < self.buf.len())
                                        .then(|| self.buf.get(i + 1))
                                        .flatten()
                                        .filter(|ch| keys::is_vowel(ch.key))
                                        .map(|ch| ch.key);

                                    if let Some(adj_key) = adjacent_vowel_key {
                                        // Check if [target, adjacent] forms valid diphthong
                                        let diphthong =
                                            [self.buf.get(i).map(|c| c.key).unwrap_or(0), adj_key];
                                        if !constants::VALID_DIPHTHONGS.contains(&diphthong) {
                                            // Invalid diphthong like "ea" → skip this target
                                            continue;
                                        }
                                    }
                                    // Valid double final with valid diphthong (or no adjacent vowel)
                                    // This handles "hongo" → "hông", "khongo" → "không"
                                } else if !all_are_valid_finals {
                                    // Invalid final consonants → skip
                                    continue;
                                } else {
                                    // Single consonant final - need VALID diphthong or double initial
                                    // Check if there's another vowel adjacent to target that forms
                                    // a VALID Vietnamese diphthong (in correct order)
                                    // Example: "coup" + "o" → "ou" is NOT valid diphthong → block
                                    // Example: "daup" + "a" → "au" IS valid diphthong → allow
                                    // Note: diphthong order matters: [V1, V2] not [V2, V1]
                                    let target_key = self.buf.get(i).map(|c| c.key).unwrap_or(0);
                                    // Adjacent BEFORE: [adjacent, target] order
                                    let adjacent_before = i > 0
                                        && self.buf.get(i - 1).is_some_and(|ch| {
                                            keys::is_vowel(ch.key)
                                                && constants::VALID_DIPHTHONGS
                                                    .contains(&[ch.key, target_key])
                                        });
                                    // Adjacent AFTER: [target, adjacent] order
                                    let adjacent_after = i + 1 < self.buf.len()
                                        && self.buf.get(i + 1).is_some_and(|ch| {
                                            keys::is_vowel(ch.key)
                                                && constants::VALID_DIPHTHONGS
                                                    .contains(&[target_key, ch.key])
                                        });
                                    let has_valid_adjacent_diphthong =
                                        adjacent_before || adjacent_after;

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

                                    // Same-vowel trigger: typing the same vowel after consonant
                                    // Example: "nanag" → second 'a' triggers circumflex on first 'a'
                                    // Pattern: initial + vowel + consonant + SAME_VOWEL
                                    // Only allow immediate circumflex for middle consonants that
                                    // can form double finals (n→ng/nh, c→ch). These are clearly
                                    // Vietnamese patterns.
                                    // For other single finals (t,m,p), delay circumflex until
                                    // a mark key is typed to avoid false positives like "data"→"dât"
                                    let is_same_vowel_trigger =
                                        self.buf.get(i).is_some_and(|c| c.key == key);
                                    // Consonants that can form double finals: n→ng/nh, c→ch
                                    let middle_can_extend = consonants_after.len() == 1
                                        && matches!(consonants_after[0], keys::N | keys::C);

                                    // Check if initial consonant already has stroke (đ/Đ)
                                    // If so, it's clearly Vietnamese (from delayed stroke pattern)
                                    let initial_has_stroke = (0..i)
                                        .filter_map(|j| self.buf.get(j))
                                        .take_while(|c| !keys::is_vowel(c.key))
                                        .any(|c| c.stroke);

                                    // Check for non-extending middle consonant (t, m, p)
                                    // These require special handling for delayed circumflex
                                    let is_non_extending_final = consonants_after.len() == 1
                                        && matches!(
                                            consonants_after[0],
                                            keys::T | keys::M | keys::P
                                        );

                                    // Allow circumflex if any of these conditions are true:
                                    // 1. Has adjacent vowel forming VALID diphthong (au, oi, etc.)
                                    //    BUT NOT if final is non-extending (t,m,p) - diphthong+t/m/p rarely valid
                                    //    EXCEPTION: V2_CIRCUMFLEX_REQUIRED diphthongs (iê, uê, yê, uô) ARE
                                    //    valid with non-extending finals (viết, thiết, miếng, etc.)
                                    // 2. Has Vietnamese double initial (nh, th, ph, etc.)
                                    // 3. Same-vowel trigger with middle consonant that can extend (n,c)
                                    // 4. Initial has stroke (đ) - clearly Vietnamese
                                    let is_v2_circumflex_diphthong = adjacent_before && {
                                        let v1 = self.buf.get(i - 1).map(|c| c.key).unwrap_or(0);
                                        constants::V2_CIRCUMFLEX_REQUIRED
                                            .contains(&[v1, target_key])
                                    };
                                    let diphthong_allows = has_valid_adjacent_diphthong
                                        && (!is_non_extending_final || is_v2_circumflex_diphthong);
                                    let allow_circumflex = diphthong_allows
                                        || has_vietnamese_double_initial
                                        || (is_same_vowel_trigger && middle_can_extend)
                                        || initial_has_stroke;

                                    // Special case: same-vowel trigger with non-extending middle consonant
                                    // Apply circumflex immediately when typing second matching vowel
                                    // Example: "toto" → "tôt" (second 'o' triggers circumflex on first 'o')
                                    // Auto-restore on space will revert if invalid (e.g., "data " → "data ")
                                    // Only apply if target has NO mark - if it has a mark (like ngã from 'x'),
                                    // the user is building a different pattern (like "expect" → ẽ-p-e-c-t)
                                    // Also block if adjacent vowel forms INVALID diphthong
                                    // Example: "coupo" → [O, U] invalid → don't apply circumflex
                                    let target_has_no_mark =
                                        self.buf.get(i).is_some_and(|c| c.mark == 0);
                                    // Check if target has ANY adjacent vowel
                                    // Diphthong + non-extending final (t,m,p) is rarely valid Vietnamese
                                    // Examples: "âup", "oem", "aum" are all invalid syllables
                                    let has_adjacent_vowel_before = i > 0
                                        && self
                                            .buf
                                            .get(i - 1)
                                            .is_some_and(|ch| keys::is_vowel(ch.key));
                                    let has_adjacent_vowel_after = i + 1 < self.buf.len()
                                        && self
                                            .buf
                                            .get(i + 1)
                                            .is_some_and(|ch| keys::is_vowel(ch.key));
                                    let has_any_adjacent_vowel =
                                        has_adjacent_vowel_before || has_adjacent_vowel_after;
                                    // Block if: has adjacent vowel (diphthong pattern) with non-extending final
                                    if is_same_vowel_trigger
                                        && is_non_extending_final
                                        && target_has_no_mark
                                        && !has_any_adjacent_vowel
                                    {
                                        // Apply circumflex to first vowel
                                        if let Some(c) = self.buf.get_mut(i) {
                                            c.tone = tone::CIRCUMFLEX;
                                            self.had_any_transform = true;
                                            self.had_vowel_triggered_circumflex = true;
                                        }
                                        // Don't add the trigger vowel - return result immediately
                                        // Need extra backspace because we're replacing displayed char
                                        let result = self.rebuild_from(i);
                                        let chars: Vec<char> = result.chars
                                            [..result.count as usize]
                                            .iter()
                                            .filter_map(|&c| char::from_u32(c))
                                            .collect();
                                        return Some(Result::send(result.backspace, &chars));
                                    }

                                    if !allow_circumflex {
                                        // Single final, no diphthong, no double initial, not valid same-vowel → likely English
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
                // Absorb the key (no-op)
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
        self.had_telex_transform = true; // Track for whitelist-based auto-restore

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

        // Telex: Check for delayed circumflex pattern (V + C + V where both V are same)
        // When buffer is "toto" (t-o-t-o) and mark key is typed, apply circumflex + remove trigger
        // This enables "totos" → "tốt" while preventing "data" → "dât"
        // Pattern: C₁ + V + C₂ + V where V is same vowel (a, e, o)
        let mut had_delayed_circumflex = false;
        if self.method == 0 && self.buf.len() >= 3 {
            // Get vowel positions
            let vowel_positions: Vec<(usize, u16)> = self
                .buf
                .iter()
                .enumerate()
                .filter(|(_, c)| keys::is_vowel(c.key))
                .map(|(i, c)| (i, c.key))
                .collect();

            // Check for exactly 2 vowels that are the same (a, e, or o for circumflex)
            if vowel_positions.len() == 2 {
                let (pos1, key1) = vowel_positions[0];
                let (pos2, key2) = vowel_positions[1];
                let is_circumflex_vowel = matches!(key1, keys::A | keys::E | keys::O);

                // Check if first vowel already has circumflex - skip delayed circumflex if so
                // This prevents "deeper" from being corrupted: after "dee" → "dê", then "deepe"
                // should NOT trigger delayed circumflex since first 'e' already has circumflex
                let first_vowel_already_has_circumflex = self
                    .buf
                    .get(pos1)
                    .is_some_and(|c| c.tone == tone::CIRCUMFLEX);

                // Must be same vowel, must have consonant(s) between them, first vowel must not already have circumflex
                if key1 == key2
                    && is_circumflex_vowel
                    && pos2 > pos1 + 1
                    && !first_vowel_already_has_circumflex
                {
                    // Check for consonants between the two vowels
                    let consonants_between: Vec<u16> = (pos1 + 1..pos2)
                        .filter_map(|j| {
                            self.buf.get(j).and_then(|c| {
                                if !keys::is_vowel(c.key) {
                                    Some(c.key)
                                } else {
                                    None
                                }
                            })
                        })
                        .collect();

                    // Must have exactly one consonant between and it must be a non-extending
                    // final (t, m, p). Consonants that can extend (n→ng/nh, c→ch) are
                    // handled immediately in try_tone.
                    let is_non_extending_final = consonants_between.len() == 1
                        && matches!(consonants_between[0], keys::T | keys::M | keys::P);

                    // Check if second vowel is at end of buffer (typical trigger position)
                    let second_vowel_at_end = pos2 == self.buf.len() - 1;

                    // Check initial consonants for Vietnamese validity
                    // Skip delayed circumflex if initial looks English (e.g., "pr" in "proposal")
                    let initial_keys: Vec<u16> = (0..pos1)
                        .filter_map(|j| self.buf.get(j).map(|ch| ch.key))
                        .take_while(|k| !keys::is_vowel(*k))
                        .collect();

                    // Validate initial consonants:
                    // - 0 initials: valid (vowel-only start)
                    // - 1 initial: valid (single consonant)
                    // - 2 initials: must be in VALID_INITIALS_2 (nh, th, ph, etc.)
                    // - 3+ initials: skip for delayed circumflex
                    //   (words like "proposal" with "pr" will be rejected here)
                    let has_valid_vietnamese_initial = match initial_keys.len() {
                        0 | 1 => true,
                        2 => {
                            let pair = [initial_keys[0], initial_keys[1]];
                            constants::VALID_INITIALS_2.contains(&pair)
                        }
                        _ => false,
                    };

                    // Check for double initial specifically (for immediate vs delayed handling)
                    let has_vietnamese_double_initial =
                        initial_keys.len() >= 2 && has_valid_vietnamese_initial;

                    // Only apply delayed circumflex if:
                    // - Has non-extending middle consonant (t, m, p)
                    // - Second vowel is at end (trigger position)
                    // - Has valid Vietnamese initial (skip English like "proposal")
                    // - No double initial (those work immediately without delay)
                    if is_non_extending_final
                        && second_vowel_at_end
                        && has_valid_vietnamese_initial
                        && !has_vietnamese_double_initial
                    {
                        // Skip delayed circumflex if raw_input is an English word
                        // This prevents "pasta" → "pất", "costa" → "côt", etc.
                        // The raw_input check works because English words like "pasta"
                        // are in our dictionary, while Vietnamese typing patterns are not.
                        let raw_str: String = self
                            .raw_input
                            .iter()
                            .filter_map(|&(k, caps, _)| utils::key_to_char(k, caps))
                            .collect::<String>()
                            .to_lowercase();
                        if english_dict::is_english_word(&raw_str) {
                            // Raw input is English - don't apply delayed circumflex
                            // Let the letter be added normally, auto-restore will handle it
                        } else {
                            // IMPORTANT: Check foreign word pattern BEFORE modifying buffer
                            // to avoid leaving buffer in inconsistent state if we need to return None.
                            // Example: "cete" + 'r' → "cêt" (delayed circumflex) + T+R check → foreign
                            // Without this check, buffer would be left as "cêt" even though we return None.
                            let temp_buffer_keys: Vec<u16> =
                                self.buf.iter().map(|c| c.key).collect();
                            let temp_buffer_tones: Vec<u8> =
                                self.buf.iter().map(|c| c.tone).collect();
                            // Check what buffer would look like after circumflex (keys without trigger)
                            let mut post_circumflex_keys = temp_buffer_keys.clone();
                            post_circumflex_keys.remove(pos2); // simulate removing trigger vowel
                            let post_circumflex_tones: Vec<u8> = post_circumflex_keys
                                .iter()
                                .enumerate()
                                .map(|(i, _)| {
                                    if i == pos1 {
                                        tone::CIRCUMFLEX
                                    } else {
                                        temp_buffer_tones.get(i).copied().unwrap_or(0)
                                    }
                                })
                                .collect();

                            // Skip delayed circumflex if the resulting buffer would trigger foreign pattern
                            if is_foreign_word_pattern(
                                &post_circumflex_keys,
                                &post_circumflex_tones,
                                key,
                            ) {
                                // Don't apply delayed circumflex - let the letter be added normally
                            } else {
                                had_delayed_circumflex = true;
                                // Apply circumflex to first vowel
                                if let Some(c) = self.buf.get_mut(pos1) {
                                    c.tone = tone::CIRCUMFLEX;
                                    self.had_any_transform = true;
                                }
                                // Remove second vowel (it was just a trigger)
                                self.buf.remove(pos2);
                            }
                        }
                    }
                }
            }
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

        // Check for invalid "-ing" rhyme: Vietnamese uses "-inh", NOT "-ing" with tone marks
        // Examples: "thíng" is invalid (things), but "tính" is valid
        // If vowel is 'i' and final is 'ng', reject marks
        if !self.free_tone_enabled && !has_horn_transforms && !has_stroke_transforms {
            let syllable = syllable::parse(&buffer_keys);
            if syllable.vowel.len() == 1 && syllable.final_c.len() == 2 {
                let vowel_key = buffer_keys[syllable.vowel[0]];
                let final_keys = [
                    buffer_keys[syllable.final_c[0]],
                    buffer_keys[syllable.final_c[1]],
                ];
                // i + ng = invalid Vietnamese rhyme for tone/mark
                if vowel_key == keys::I && final_keys == [keys::N, keys::G] {
                    return None;
                }
            }
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

        // Check if target vowel already has the same mark
        // This handles two cases:
        //
        // 1. "lists" pattern: After mark, user typed CONSONANT then same mark key
        //    - Buffer: [L, í, T] → has consonant after marked vowel
        //    - User wants to REVERT the mark → "lits"
        //
        // 2. "roofif" pattern: After mark, user typed VOWEL then same mark key
        //    - Buffer: [R, ồ, I] → only vowels after marked vowel (diphthong)
        //    - User is still in same syllable, second 'f' is likely accidental
        //    - Absorb → "rồi"
        //
        // EXCEPTION: Words starting with W are English - don't apply revert logic.
        // W is not a valid Vietnamese initial, so "writer", "wrong", "wrap" etc.
        // should NOT trigger delayed revert even if same mark key is pressed.
        // Auto-restore will handle these at word boundary.
        let starts_with_w = self
            .raw_input
            .first()
            .map(|(k, _, _)| *k == keys::W)
            .unwrap_or(false);

        if let Some(c) = self.buf.get(pos) {
            if c.mark == mark_val && !starts_with_w {
                // Check if there's a consonant after the marked vowel position
                let has_consonant_after = self
                    .buf
                    .iter()
                    .skip(pos + 1)
                    .any(|ch| !keys::is_vowel(ch.key));

                if has_consonant_after {
                    // Consonant after: REVERT the mark (remove dấu)
                    // "lists" → "lits", user typed s twice to undo the mark
                    return Some(self.revert_mark(key, caps));
                } else {
                    // Only vowels after: absorb (user double-tapped in same syllable)
                    // "roofif" → "rồi"
                    return Some(Result::send(0, &[]));
                }
            }
        }

        if let Some(c) = self.buf.get_mut(pos) {
            c.mark = mark_val;
            self.last_transform = Some(Transform::Mark(key, mark_val));
            self.had_any_transform = true;
            self.had_telex_transform = true; // Track for whitelist-based auto-restore
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

            // If delayed circumflex was applied, rebuild from earliest vowel position
            // and add extra backspace for the trigger vowel that was on screen but removed
            if had_delayed_circumflex {
                rebuild_pos = rebuild_pos.min(1); // Start from first vowel position
                let result = self.rebuild_from(rebuild_pos);
                let chars: Vec<char> = result.chars[..result.count as usize]
                    .iter()
                    .filter_map(|&c| char::from_u32(c))
                    .collect();
                // Add 1 to backspace for the removed trigger vowel still on screen
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
        // Check if raw_input is an English word (used later with diphthong check)
        let raw_str: String = self
            .raw_input
            .iter()
            .filter_map(|&(k, caps, _)| utils::key_to_char(k, caps))
            .collect::<String>()
            .to_lowercase();
        let is_english_word = english_dict::is_english_word(&raw_str);

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

            // Skip tone repositioning if raw_input is an English word AND vowels don't form
            // a valid Vietnamese diphthong pattern.
            // This prevents "costa" → "cotá" (O→A tone move with consonants between)
            // while allowing "usee" → "uế" (valid UE diphthong pattern).
            if is_english_word && !self.vowels_form_valid_diphthong(&vowels) {
                return None;
            }

            // Check for syllable boundary: if there's a consonant between the toned vowel
            // and any later vowel, the toned vowel is in a closed syllable - don't reposition.
            // Example: "bủn" + "o" → 'n' closes "bủn", so 'o' starts new syllable.
            //
            // EXCEPTION: If vowels form a valid Vietnamese diphthong pattern, allow repositioning.
            // This handles interleaved typing like "kisna" where tone on 'i' should move to 'a'
            // because "ia" is a valid diphthong with tone on first vowel, but the user typed
            // the consonant 'n' before 'a'.
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
                // Check if vowels form a valid diphthong pattern
                // If yes, this is NOT a syllable boundary - user just typed out of order
                if !self.vowels_form_valid_diphthong(&vowels) {
                    // Syllable boundary detected - tone is in previous syllable, don't move it
                    return None;
                }
            }

            // Issue #162 fix: Don't reposition if vowels are identical (doubled vowels like "oo", "aa", "ee").
            // These are NOT valid Vietnamese diphthongs and should keep mark on first vowel.
            // This prevents VNI "o2o" from incorrectly producing "oò" instead of "òo".
            if vowels.len() == 2 && vowels[0].key == vowels[1].key {
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

    /// Check if vowels form a valid Vietnamese diphthong pattern
    ///
    /// This allows tone repositioning even when consonants are between vowels
    /// (typed out of order). Valid diphthongs: ia, ua, oa, ai, ao, oi, etc.
    fn vowels_form_valid_diphthong(&self, vowels: &[Vowel]) -> bool {
        use crate::data::vowel::{TONE_FIRST_PATTERNS, TONE_SECOND_PATTERNS};

        if vowels.len() < 2 {
            return false;
        }

        // Check if first two vowels form a valid diphthong pattern
        let pair = [vowels[0].key, vowels[1].key];

        // Check against all known diphthong patterns
        TONE_FIRST_PATTERNS
            .iter()
            .any(|p| p[0] == pair[0] && p[1] == pair[1])
            || TONE_SECOND_PATTERNS
                .iter()
                .any(|p| p[0] == pair[0] && p[1] == pair[1])
    }

    /// Reorder buffer when a vowel completes a diphthong with earlier vowel,
    /// and there are consonants between that should be final consonants.
    ///
    /// Example: "kisna" → buffer is k-í-n-a, but Vietnamese order is k-í-a-n
    /// because "ia" is a diphthong and 'n' is the final consonant.
    ///
    /// Returns Some(reorder_start_pos) if reordering happened, None otherwise.
    fn reorder_diphthong_with_final(&mut self) -> Option<usize> {
        use crate::data::constants::VALID_FINALS_1;

        let len = self.buf.len();
        if len < 3 {
            return None; // Need at least: vowel + consonant + vowel
        }

        // Only reorder if buffer has Vietnamese transforms (tone marks or diacritics)
        // This prevents reordering for English words like "final" → "fianl"
        let has_vn_transforms = self.buf.iter().any(|c| c.mark > 0 || c.tone > 0);
        if !has_vn_transforms {
            return None;
        }

        // Skip reordering if raw_input is an English word
        // This prevents corrupting English words like "vista" → "víat"
        // The auto-restore will handle restoring "vísta" to "vista" since it's invalid VN
        // But if we reorder, "víat" looks like valid VN structure and won't restore
        let raw_str: String = self
            .raw_input
            .iter()
            .filter_map(|&(key, caps, _)| utils::key_to_char(key, caps))
            .collect::<String>()
            .to_lowercase();
        if english_dict::is_english_word(&raw_str) {
            return None;
        }

        // The new vowel is the last character in buffer
        let new_vowel_pos = len - 1;
        let new_vowel_key = self.buf.get(new_vowel_pos)?.key;

        // Find the previous vowel (before any consonants)
        let mut prev_vowel_pos = None;
        let mut consonants_between = Vec::new();

        for i in (0..new_vowel_pos).rev() {
            let c = self.buf.get(i)?;
            if keys::is_vowel(c.key) {
                prev_vowel_pos = Some(i);
                break;
            } else if c.key != keys::W {
                // Collect consonants between vowels
                consonants_between.push(i);
            }
        }

        let prev_vowel_pos = prev_vowel_pos?;
        if consonants_between.is_empty() {
            return None; // No consonants between - no reordering needed
        }

        // Check if there are other vowels before prev_vowel_pos
        // If yes, don't reorder - the consonant might belong to an earlier vowel cluster
        // Example: "coupo" - don't reorder because "ou" vowel cluster exists before 'p'
        let has_earlier_vowels =
            (0..prev_vowel_pos).any(|i| self.buf.get(i).is_some_and(|c| keys::is_vowel(c.key)));
        if has_earlier_vowels {
            return None;
        }

        // Check if the two vowels form a valid diphthong
        let prev_vowel = self.buf.get(prev_vowel_pos)?;
        let prev_vowel_key = prev_vowel.key;

        // If previous vowel has a vowel modifier (circumflex/breve/horn), it can't form
        // a diphthong with the new vowel. Example: ô+a is NOT valid, only o+a is valid.
        // Circumflex/breve/horn marks make a vowel "complete" and non-combinable.
        // Note: `tone` field = vowel modifier (^, ư, ơ, ă), `mark` field = accent (sắc, huyền, etc.)
        if prev_vowel.tone > 0 {
            return None;
        }

        let pair = [prev_vowel_key, new_vowel_key];

        // Only reorder for specific diphthongs that are commonly typed "out of order"
        // (tone modifier between vowels). Other diphthongs like OE, OA, EO are typically
        // typed in natural order and shouldn't trigger reordering.
        //
        // Patterns that support interleaved typing (V + tone + C + V → V + V + C):
        // - IA: "misa" → "mía", "kisna" → "kían"
        // - UA: "musa" → "mùa", "kusna" → "kùan"
        //
        // This prevents false positives like "copses" → "coeps" (OE pattern)
        let is_reorderable_diphthong = matches!(pair, [keys::I, keys::A] | [keys::U, keys::A]);

        if !is_reorderable_diphthong {
            return None;
        }

        // Check raw_input pattern to distinguish valid interleaved typing from foreign words
        //
        // Valid: "misa" = m + i + s + a → only tone mod (S) between I and A
        // Valid: "kisna" = k + i + s + n + a → tone mod (S) + final consonant (N) between I and A
        //        In buffer: [K, Í, N, A] - the N between I and A will become the final
        // Invalid: "gusta" = g + u + s + t + a → tone mod (S) + T between U and A
        //          BUT in buffer: [G, Ú, T, A] - T is also between U and A, same as kisna!
        //
        // The difference: "kisna" is meant to be Vietnamese, "gusta" is foreign.
        // We can't distinguish purely from structure, so use heuristics:
        // 1. Check if the pattern is common Vietnamese interleaved typing
        // 2. For IA/UA patterns with single consonant between, allow reordering
        //    (auto-restore will fix English words on space)
        //
        // This allows "kisna" → "kían" while relying on auto-restore to fix "gusta" → "gusta"

        // Check if consonants could be valid final consonants
        // For simplicity, only handle single consonant final (most common case)
        if consonants_between.len() > 2 {
            return None; // Too many consonants - probably not a reorder case
        }

        // Check if consonants form valid final (ng, nh, ch, or single consonant)
        let consonant_keys: Vec<u16> = consonants_between
            .iter()
            .rev()
            .filter_map(|&i| self.buf.get(i).map(|c| c.key))
            .collect();

        let is_valid_final = match consonant_keys.len() {
            1 => VALID_FINALS_1.contains(&consonant_keys[0]),
            2 => {
                // Check for valid 2-char finals: ng, nh, ch
                matches!(
                    (consonant_keys[0], consonant_keys[1]),
                    (keys::N, keys::G) | (keys::N, keys::H) | (keys::C, keys::H)
                )
            }
            _ => false,
        };

        if !is_valid_final {
            return None;
        }

        // Reorder: move new vowel to right after previous vowel
        // Buffer: [... prev_vowel ... consonants ... new_vowel]
        // Target: [... prev_vowel new_vowel ... consonants ...]
        //
        // Since new_vowel is already at end, we need to:
        // 1. Save the new vowel
        // 2. Move consonants one position forward (toward end)
        // 3. Insert new vowel right after prev_vowel

        let new_vowel = *self.buf.get(new_vowel_pos)?;

        // Shift consonants one position forward
        // consonants_between is in reverse order (highest pos first)
        for &pos in &consonants_between {
            if let Some(c) = self.buf.get(pos) {
                let c_copy = *c;
                if let Some(next) = self.buf.get_mut(pos + 1) {
                    *next = c_copy;
                }
            }
        }

        // Place new vowel right after prev_vowel
        let insert_pos = prev_vowel_pos + 1;
        if let Some(slot) = self.buf.get_mut(insert_pos) {
            *slot = new_vowel;
        }

        // Return position to rebuild from
        Some(insert_pos)
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
                    // Track for auto-restore logic (double ss/ff detection)
                    self.had_mark_revert = true;
                    // Track ww pattern for whitelist-based restore
                    self.had_telex_transform = true;
                    // Store raw_input BEFORE modification for whitelist lookup
                    self.telex_double_raw = Some(self.get_raw_input_string_preserve_case());
                    // Fix raw_input: "ww" typed → raw has [w,w] but buffer is "w"
                    // Remove the tone-triggering key from raw_input so restore works correctly
                    // raw_input: [a, w, w] → [a, w] (remove first 'w' that triggered tone)
                    // This ensures "awwait" → "await" not "awwait" on auto-restore
                    if self.raw_input.len() >= 2 {
                        let current = self.raw_input.pop(); // current key (just added)
                        self.raw_input.pop(); // tone-trigger key (consumed, discard)
                        if let Some(c) = current {
                            self.raw_input.push(c);
                        }
                    }
                    // Store length AFTER modification
                    self.telex_double_raw_len = self.raw_input.len();
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
                                     // Set had_telex_transform for whitelist-based auto-restore
                                     // This allows "taxxi" → "taxi" (not in whitelist → keep buffer)
        self.had_telex_transform = true;
        // Store raw_input for whitelist lookup
        self.telex_double_raw = Some(self.get_raw_input_string_preserve_case());
        self.telex_double_raw_len = self.raw_input.len();

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

        // Telex: Revert delayed circumflex when same vowel is typed again
        // Pattern: After "data" → "dât" (delayed circumflex), typing 'a' again should revert to "data"
        // Buffer ends with: vowel-with-circumflex + non-extending-final (t, m, p)
        // Typed key matches the base of the circumflex vowel (a→â, e→ê, o→ô)
        // IMPORTANT: Only apply this revert for DELAYED circumflex (V+C+V pattern), not for
        // immediate circumflex (VV pattern like "deep" → "dêp"). For immediate circumflex,
        // typing another vowel should NOT revert (allows words like "deeper").
        if self.method == 0
            && self.had_vowel_triggered_circumflex
            && matches!(key, keys::A | keys::E | keys::O)
            && self.buf.len() >= 2
        {
            let last_idx = self.buf.len() - 1;
            let vowel_idx = self.buf.len() - 2;

            // Check if last char is a non-extending final consonant
            let last_is_non_extending = self
                .buf
                .get(last_idx)
                .is_some_and(|c| matches!(c.key, keys::T | keys::M | keys::P));

            // Check if second-to-last has circumflex and matches typed vowel
            let should_revert = last_is_non_extending
                && self.buf.get(vowel_idx).is_some_and(|c| {
                    c.tone == tone::CIRCUMFLEX
                        && c.key == key
                        && matches!(c.key, keys::A | keys::E | keys::O)
                });

            if should_revert {
                // Remove circumflex from the vowel
                if let Some(c) = self.buf.get_mut(vowel_idx) {
                    c.tone = tone::NONE;
                }
                // Reset vowel-triggered circumflex flag since we're reverting
                self.had_vowel_triggered_circumflex = false;
                // Track circumflex revert for auto-restore (used to collapse double vowel at end)
                self.had_circumflex_revert = true;

                // Add the typed vowel to buffer (the one that triggered revert)
                // "dataa" flow: "dât" (3 chars) → revert â → "dat" → add 'a' → "data" (4 chars)
                self.buf.push(Char::new(key, caps));

                // Rebuild from vowel position using after_insert (new char not yet on screen)
                // Screen has: "dât" (3 chars), buffer now has: "data" (4 chars)
                // Need to delete "ât" (2 chars) and output "ata" (3 chars) → screen becomes "data"
                return self.rebuild_from_after_insert(vowel_idx);
            }
        }

        // Telex: Post-tone delayed circumflex (xepse → xếp)
        // Pattern: initial-consonant + vowel-with-mark + non-extending-final (t, m, p) + same vowel
        // When user types tone BEFORE circumflex modifier: "xeps" → "xép", then 'e' → "xếp"
        // The second vowel triggers circumflex on the first vowel (keeping existing mark)
        // IMPORTANT: Must have initial consonant to form valid Vietnamese syllable
        // "expect" (e-x-p-e) should NOT trigger because no initial consonant
        if self.method == 0 && matches!(key, keys::A | keys::E | keys::O) && self.buf.len() >= 3 {
            let last_idx = self.buf.len() - 1;
            let vowel_idx = self.buf.len() - 2;

            // Check if there's at least one initial consonant before the vowel
            let has_initial_consonant =
                vowel_idx > 0 && self.buf.get(0).is_some_and(|c| keys::is_consonant(c.key));

            // Check if last char is a non-extending final consonant
            let last_is_non_extending = self
                .buf
                .get(last_idx)
                .is_some_and(|c| matches!(c.key, keys::T | keys::M | keys::P));

            // Check if second-to-last has mark but NO circumflex, and matches typed vowel
            let should_add_circumflex = has_initial_consonant
                && last_is_non_extending
                && self.buf.get(vowel_idx).is_some_and(|c| {
                    c.mark > 0 // has tone mark (sắc, huyền, etc.)
                        && c.tone == tone::NONE // but no circumflex yet
                        && c.key == key // matches typed vowel
                        && matches!(c.key, keys::A | keys::E | keys::O)
                });

            if should_add_circumflex {
                // Skip circumflex if raw_input is an English word
                // This prevents "pasta" → "pất", "costa" → "côt", etc.
                // raw_input includes the current key (pushed before process() is called)
                let raw_str: String = self
                    .raw_input
                    .iter()
                    .filter_map(|&(k, caps, _)| utils::key_to_char(k, caps))
                    .collect::<String>()
                    .to_lowercase();
                if english_dict::is_english_word(&raw_str) {
                    // Raw input is English - skip circumflex, add vowel normally
                    // The auto-restore will handle restoring the English word
                } else {
                    // Add circumflex to the vowel (keeping existing mark)
                    if let Some(c) = self.buf.get_mut(vowel_idx) {
                        c.tone = tone::CIRCUMFLEX;
                        self.had_any_transform = true;
                    }

                    // Note: raw_input already has the key (pushed at on_key_ext before process)

                    // Rebuild from vowel position (second vowel is NOT added to buffer - it's modifier)
                    // Screen has: "xép" (3 chars), buffer stays: "xếp" (3 chars, vowel updated)
                    // Need to delete "ép" (2 chars) and output "ếp" (2 chars)
                    return self.rebuild_from(vowel_idx);
                }
            }
        }

        self.last_transform = None;
        // Add letters to buffer, and numbers in both Telex and VNI modes
        // This ensures buffer.len() stays in sync with screen chars for correct backspace count
        // Issue #162: Numbers must be added to buffer in Telex mode too, otherwise patterns
        // like "o2o" have buffer = [O] (missing '2') causing the second 'o' to incorrectly
        // trigger circumflex (thinking it's "oo" → "ô")
        if keys::is_letter(key) || keys::is_number(key) {
            // Add the letter/number to buffer
            self.buf.push(Char::new(key, caps));

            // Issue #44 (part 2): Apply deferred breve when valid final consonant is typed
            // "trawm" → after "traw" (pending breve on 'a'), typing 'm' applies breve → "trăm"
            if let Some(breve_pos) = self.pending_breve_pos {
                // Valid final consonants that make breve valid: c, k, m, n, p, t
                // Note: k is included for ethnic minority words (Đắk Lắk)
                if matches!(
                    key,
                    keys::C | keys::K | keys::M | keys::N | keys::P | keys::T
                ) {
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

            // Issue #133: Apply deferred horn to 'u' when final consonant/vowel is typed
            // "duow" → "duơ" (pending on u), then "c" → apply horn to u → "dược"
            if let Some(u_pos) = self.pending_u_horn_pos {
                // Apply horn to 'u' at pending position
                if let Some(c) = self.buf.get_mut(u_pos) {
                    if c.key == keys::U && c.tone == tone::NONE {
                        c.tone = tone::HORN;
                        self.had_any_transform = true;
                    }
                }
                self.pending_u_horn_pos = None;

                // Rebuild from u position: screen has "...uơ...", buffer has "...ươ...+new_char"
                // The new char was already pushed at line 1799 but not yet on screen
                // Use rebuild_from_after_insert which accounts for this
                return self.rebuild_from_after_insert(u_pos);
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

            // Reorder buffer when a vowel completes a diphthong with earlier vowel
            // and there are consonants between that should be final consonants.
            // Example: "kisna" → buffer is k-í-n, adding 'a' should produce k-í-a-n (kían)
            // because "ia" is a diphthong and 'n' is a valid final consonant.
            if keys::is_vowel(key) {
                if let Some(reorder_pos) = self.reorder_diphthong_with_final() {
                    // After reordering, also reposition tone if needed
                    // Example: "musno" → buffer reordered to m-ú-o-n, but tone should be on 'o'
                    // because "uo" diphthong has tone on second vowel.
                    let tone_reposition = self.reposition_tone_if_needed();
                    let rebuild_pos = tone_reposition.map(|(old, _)| old).unwrap_or(reorder_pos);
                    return self.rebuild_from_after_insert(rebuild_pos);
                }
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
            // IMPORTANT: Skip mark keys (s, f, r, x, j in Telex) because they're tone modifiers,
            // not true consonants. User typing "đườ" + 's' wants to add sắc mark, not restore.
            //
            // Only run if english_auto_restore is enabled (experimental feature)
            let im = input::get(self.method);
            let is_mark_key = im.mark(key).is_some();
            if self.english_auto_restore
                && keys::is_consonant(key)
                && !is_mark_key
                && self.buf.len() >= 2
            {
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
    /// Also restores pending_capitalize if auto_capitalize was used (for selection-delete)
    pub fn clear(&mut self) {
        // Restore pending_capitalize if auto_capitalize was used
        // This handles selection-delete: user selects and deletes text,
        // we should restore pending state so next letter is capitalized
        if self.auto_capitalize_used {
            self.pending_capitalize = true;
            self.auto_capitalize_used = false;
        }
        self.buf.clear();
        self.raw_input.clear();
        self.last_transform = None;
        self.has_non_letter_prefix = false;
        self.pending_breve_pos = None;
        self.pending_u_horn_pos = None;
        self.stroke_reverted = false;
        self.had_mark_revert = false;
        self.pending_mark_revert_pop = false;
        self.had_any_transform = false;
        self.had_vowel_triggered_circumflex = false;
        self.had_circumflex_revert = false;
        self.had_telex_transform = false;
        self.telex_double_raw = None;
        self.telex_double_raw_len = 0;
        self.restored_pending_clear = false;
        self.shortcut_prefix.clear();
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

    /// Debug: Check if vowel-triggered circumflex flag is set
    pub fn had_vowel_circumflex(&self) -> bool {
        self.had_vowel_triggered_circumflex
    }

    /// Debug: Get raw_input length
    pub fn raw_input_len(&self) -> usize {
        self.raw_input.len()
    }

    /// Debug: Check if raw_input is valid English
    pub fn is_raw_english(&self) -> bool {
        self.is_raw_input_valid_english()
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
        // Also handles words with invalid initials like "forr" - since 'f' is not valid,
        // no mark was ever applied, so the result stays "forr" (not collapsed to "for")
        if !self.had_any_transform {
            return None;
        }

        // VIETNAMESE PRIORITY: Only keep Vietnamese when buffer has Vietnamese-SPECIFIC marks
        // Vietnamese-specific: circumflex (ô,â,ê), horn (ơ,ư), breve (ă), stroke (đ)
        // These marks indicate intentional Vietnamese typing
        // Plain tone marks (sắc/huyền/hỏi/ngã/nặng) alone are NOT enough - could be English typo
        // EXCEPTIONS that skip Vietnamese priority:
        //   - W patterns: produce HORN but clearly English (law, saw, west)
        //   - Telex doubles (oo, ee, aa, dd): let whitelist logic handle them (poor, bees, add)
        // Examples:
        //   "boos" → "bố" (oo = circumflex, in whitelist) → let whitelist handle
        //   "bore" → "boẻ" (only hỏi tone, no VN-specific mark) → check other logic
        //   "law" → "lă" (W produces breve/HORN) → skip priority, let W restore handle it
        let has_w_in_raw = self.raw_input.iter().any(|(key, _, _)| *key == keys::W);
        // Check for telex double patterns:
        // 1. Consecutive same vowels (oo, ee, aa) or dd
        // 2. VCV patterns with same vowel (oto→ôt, ata→ât, ete→êt) - delayed circumflex
        let has_telex_double = self.raw_input.windows(2).any(|pair| {
            let (k1, _, _) = pair[0];
            let (k2, _, _) = pair[1];
            k1 == k2 && (k1 == keys::O || k1 == keys::E || k1 == keys::A || k1 == keys::D)
        }) || self.raw_input.windows(3).any(|triple| {
            let (k1, _, _) = triple[0];
            let (k2, _, _) = triple[1];
            let (k3, _, _) = triple[2];
            // VCV pattern: same vowel with consonant in between (delayed circumflex)
            k1 == k3 && keys::is_vowel(k1) && !keys::is_vowel(k2)
        });
        let has_vn_specific_mark = self.buf.iter().any(|c| {
            c.tone == tone::CIRCUMFLEX  // ô, â, ê
                || c.tone == tone::HORN // ơ, ư, ă (breve uses HORN value)
                || c.stroke // đ
        });
        // Only apply Vietnamese priority if NO W pattern AND NO telex double
        if has_vn_specific_mark
            && !has_w_in_raw
            && !has_telex_double
            && !self.is_buffer_invalid_vietnamese()
        {
            return None;
        }

        // TELEX DOUBLES WHITELIST CHECK
        // Check whitelist for words with telex patterns (s/f/r/x/j tones, aa/ee/oo marks, dd stroke)
        if self.had_telex_transform {
            // Build raw string for whitelist lookup
            let raw_str = if let Some(ref stored) = self.telex_double_raw {
                // Double revert pattern occurred (xx, ss, dd, etc.)
                // Build full raw string including subsequent chars typed after revert
                let subsequent_start = if self.raw_input.len() < self.telex_double_raw_len {
                    self.telex_double_raw_len.saturating_sub(1)
                } else {
                    self.telex_double_raw_len
                };
                let subsequent: String = self
                    .raw_input
                    .iter()
                    .skip(subsequent_start)
                    .filter_map(|&(key, caps, shift)| utils::key_to_char_ext(key, caps, shift))
                    .collect();
                format!("{}{}", stored.to_lowercase(), subsequent.to_lowercase())
            } else {
                self.get_raw_input_string()
            };

            if telex_doubles::contains(&raw_str) {
                // Word is in English telex doubles whitelist
                // Decision logic with Vietnamese-first principle:
                //
                // 1. Double vowel patterns (oo, ee, aa) + in English dict → RESTORE to English
                //    These are distinctive English patterns (poor, beer, teen)
                // 2. Stroke (đ) + NOT in English dict → keep Vietnamese (abbreviation)
                // 3. Buffer is VALID Vietnamese + NOT double vowel → keep Vietnamese
                // 4. Buffer is INVALID Vietnamese → restore to English
                //
                // Examples:
                //   "poor" → has "oo" pattern + in dict → restore "poor"
                //   "bits" → no double vowel, buffer "bít" valid VN → keep "bít"
                //   "chir" → no double vowel, buffer "chỉ" valid VN → keep "chỉ"
                //   "daddy" → buffer "đady" invalid VN, in dict → restore "daddy"
                //   "ddc" → has stroke, not in dict → keep "đc"

                let has_stroke = self.buf.iter().any(|c| c.stroke);
                let buffer_invalid_vn = self.is_buffer_invalid_vietnamese();
                let raw_in_english_dict = english_dict::is_english_word(&raw_str);

                // W at end pattern: foreign words like moscow, warsaw, saw, law
                let w_at_end = self
                    .raw_input
                    .last()
                    .map(|(k, _, _)| *k == keys::W)
                    .unwrap_or(false);

                // Simple logic: buffer invalid VN + raw in English dict → restore
                // Special case: W at end + in dict → restore (foreign word pattern)
                if has_stroke && !raw_in_english_dict {
                    // Skip restore - Vietnamese abbreviation like đc, đt
                } else if w_at_end && raw_in_english_dict {
                    // W at end + in dict → restore foreign words (moscow, warsaw, saw)
                    return self.build_raw_chars_exact();
                } else if buffer_invalid_vn && raw_in_english_dict {
                    // Check if collapsed buffer is also a valid English word
                    // If buffer is a known English word, keep it (e.g., "lissa" → "lisa")
                    // If buffer is NOT a known word, restore original (e.g., "larissa" → "larissa")
                    let buffer_str = self.get_buffer_string().to_lowercase();
                    if !english_dict::is_english_word(&buffer_str) {
                        // Buffer not in dict → restore to original English
                        return self.build_raw_chars_exact();
                    }
                    // Buffer IS in dict → keep buffer (collapsed form is valid word)
                }
                // Otherwise keep buffer (valid VN or not in dict)
            }

            // DOUBLE SS/FF DICTIONARY CHECK (before "keep clean buffer" logic)
            // Words ending with ss/ff that are in dictionary should restore to English.
            // Examples: mass, bass, pass, buff, cuff → restore to English
            // This check must come BEFORE "keep clean buffer" logic because buffer "mas"
            // looks clean but should restore to "mass" if "mass" is in dictionary.
            // IMPORTANT: Only apply when ss/ff is at END of complete word (no subsequent chars)
            // For "masson" (ss in middle), let normal collapse logic handle it → "mason"
            if let Some(ref stored) = self.telex_double_raw {
                // Check if any chars were typed AFTER the double pattern
                let has_subsequent_chars = self.raw_input.len() > self.telex_double_raw_len;

                // Only apply this check when ss/ff is at the END of the word
                if !has_subsequent_chars {
                    let chars: Vec<char> = stored.chars().collect();
                    if chars.len() >= 2 {
                        let last = chars[chars.len() - 1].to_ascii_lowercase();
                        let second_last = chars[chars.len() - 2].to_ascii_lowercase();
                        let is_double_ss = last == 's' && second_last == 's';
                        let is_double_ff = last == 'f' && second_last == 'f';

                        if is_double_ss || is_double_ff {
                            let original_lower = stored.to_lowercase();
                            if english_dict::is_english_word(&original_lower) {
                                // EXCEPTIONS: "off", "iff", "ass" should keep reverted form
                                let is_exception = if chars.len() == 3 {
                                    let first = chars[0].to_ascii_lowercase();
                                    let is_off = first == 'o' && is_double_ff;
                                    let is_iff = first == 'i' && is_double_ff;
                                    let is_ass = first == 'a' && is_double_ss;
                                    is_off || is_iff || is_ass
                                } else {
                                    false
                                };

                                if !is_exception {
                                    return self.build_raw_chars_exact();
                                }
                            }
                        }
                    }
                }
            }

            // Word NOT in whitelist:
            // If double revert pattern occurred AND buffer has NO Vietnamese marks,
            // AND buffer doesn't have repeated consonants (indicating incomplete collapse),
            // keep buffer (user intentionally typed double to get clean result).
            // Example: "taxxi" → buffer "taxi" (no marks, no repeats) → keep "taxi"
            // Example: "reff" → buffer "ref" (no marks, no repeats) → keep "ref"
            // But: "assssess" → buffer "asssess" (has repeated 's') → continue to collapse
            // But: "prooff" → buffer "prôf" (has mark ô) → continue to other logic
            if self.telex_double_raw.is_some() {
                let has_marks = self.buf.iter().any(|c| c.tone > 0 || c.mark > 0);
                let has_stroke = self.buf.iter().any(|c| c.stroke);
                let buffer_str = self.get_buffer_string();
                // Check for repeated consonants (ss, ff, rr, etc.) in buffer
                let has_repeated_consonant = buffer_str
                    .as_bytes()
                    .windows(2)
                    .any(|w| w[0] == w[1] && matches!(w[0], b's' | b'f' | b'r' | b'x' | b'j'));
                if !has_marks && !has_stroke && !has_repeated_consonant {
                    return None; // Keep buffer (clean, no Vietnamese transforms)
                }
            }
        }

        // No telex double pattern → continue with existing logic for other cases

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
        // EXCEPTION: If buffer is INVALID Vietnamese (like "cofee" with F), still restore
        // This handles cases like "coffee" where 'ee' is part of English word, not Telex revert
        if !has_marks_or_tones && !has_stroke && self.ends_with_double_modifier() {
            // Only skip restore if buffer is actually valid Vietnamese
            // Invalid buffers (containing F, W at wrong positions, etc.) should still restore
            let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
            let buffer_tones: Vec<u8> = self.buf.iter().map(|c| c.tone).collect();
            if validation::is_valid_with_tones(&buffer_keys, &buffer_tones) {
                return None;
            }
        }

        // UNIFIED LOGIC: Restore ONLY when BOTH conditions are met:
        // 1. buffer != valid Vietnamese (is_buffer_invalid_vietnamese)
        // 2. raw_input == valid English (is_raw_input_valid_english)
        //
        // This replaces the previous multi-check pattern-based approach.
        // Benefits:
        // - Simpler, more predictable logic
        // - Fewer false positives for valid Vietnamese words
        // - Works correctly with "sims", "homo", and other edge cases

        // First check: Is buffer invalid Vietnamese?
        let buffer_invalid_vn = self.is_buffer_invalid_vietnamese();

        // For stroke-only transforms (no marks/tones), only restore if word is long enough
        // Short words like "đd" from "ddd" should stay; long invalid words like "đealine" should restore
        if buffer_invalid_vn && has_stroke && !has_marks_or_tones && self.buf.len() < 4 {
            return None;
        }

        // If user typed double TONE modifier (rr) at END of SHORT word, keep reverted form
        if self.had_mark_revert && self.raw_input.len() >= 2 && self.raw_input.len() <= 4 {
            let (last_key, _, _) = self.raw_input[self.raw_input.len() - 1];
            let (second_last_key, _, _) = self.raw_input[self.raw_input.len() - 2];
            // Double 'rr' at end of short word → keep reverted form
            if last_key == second_last_key && last_key == keys::R {
                return None;
            }
        }

        // Second check: Is raw_input valid English?
        let raw_input_valid_en = self.is_raw_input_valid_english();

        // NOTE: Double ss/ff dictionary check is handled earlier (lines 3388-3421)
        // in the telex_doubles section, BEFORE the "keep clean buffer" logic.

        // SPECIAL CASE: Doubled modifier pattern handling
        // Distinguish between:
        // - V + doubled_modifier (issue, offer) → restore to raw (common English)
        // - C + V + doubled_modifier (carre) → keep buffer (Telex revert pattern)
        if self.had_mark_revert && buffer_invalid_vn && raw_input_valid_en {
            let tone_mods = [keys::S, keys::F, keys::R, keys::X, keys::J];

            // Find position of doubled modifier in raw_input
            let mut doubled_pos = None;
            for i in 0..self.raw_input.len().saturating_sub(1) {
                let (k1, _, _) = self.raw_input[i];
                let (k2, _, _) = self.raw_input[i + 1];
                if tone_mods.contains(&k1) && k1 == k2 {
                    doubled_pos = Some(i);
                    break;
                }
            }

            if let Some(pos) = doubled_pos {
                // Check if doubled modifier is at END of word (like "bass", "varr")
                let is_at_end = pos + 2 >= self.raw_input.len();

                // Check if doubled modifier is RIGHT AFTER initial vowel (like i-ss, o-ff)
                // Pattern: V + doubled_modifier (position 1)
                let is_after_initial_vowel = pos == 1 && {
                    let (first_key, _, _) = self.raw_input[0];
                    keys::is_vowel(first_key)
                };

                // Check how many chars follow the doubled modifier
                // - "carre": rr + 1 char (e) → likely Telex pattern
                // - "mirror": rr + 2 chars (or) → likely English word
                // - "sorry": rr + 1 char (y) → English word (not Telex)
                let chars_after = self.raw_input.len() - pos - 2;

                // Only consider Telex pattern if:
                // 1. Exactly 1 char after doubled modifier
                // 2. That char is 'e' (common Telex ending: carre→care, barre→bare)
                let ends_with_e = self
                    .raw_input
                    .last()
                    .map(|(k, _, _)| *k == keys::E)
                    .unwrap_or(false);
                let is_telex_pattern = chars_after == 1 && ends_with_e;

                // Check if 'w' at start was converted to 'ư' (Telex w-vowel)
                // Words like "worry" start with 'w' in raw but 'ư' in buffer
                let w_converted_to_horn = !self.raw_input.is_empty() && {
                    let (first_key, _, _) = self.raw_input[0];
                    first_key == keys::W && self.buf.get(0).map(|c| c.key) != Some(keys::W)
                };

                if !is_at_end && !is_after_initial_vowel && is_telex_pattern && !w_converted_to_horn
                {
                    // Pattern like "carre" (C + V + rr + single_char) → keep buffer "care"
                    // Buffer already has the collapsed result from Telex revert
                    // But NOT if raw is English word and buffer is not (like "giraffe" → "giafe")
                    return None;
                }
            }
        }

        // UNIFIED: Restore only when buffer is invalid Vietnamese AND raw_input is valid English
        if buffer_invalid_vn && raw_input_valid_en {
            return self.build_raw_chars();
        }

        // OW PATTERN CHECK: raw_input has 'ow' but buffer has ơ/ở/ờ/ớ/ỡ/ợ (horn-o)
        // English words: power, tower, down, town, etc.
        // Telex 'w' converts 'o' to 'ơ' (horn mark), which is wrong for English
        // Example: "power" → buffer "pởe" (wrong), should restore to "power"
        // BUT: "bow" → buffer "bơ" (valid Vietnamese), keep it
        // Only restore when buffer is INVALID Vietnamese
        if is_word_complete && buffer_invalid_vn && raw_input_valid_en {
            let has_ow_in_raw = self.raw_input.windows(2).any(|w| {
                let (k1, _, _) = w[0];
                let (k2, _, _) = w[1];
                k1 == keys::O && k2 == keys::W
            });
            let has_horn_o_in_buffer = self
                .buf
                .iter()
                .any(|c| c.key == keys::O && c.tone == tone::HORN);
            if has_ow_in_raw && has_horn_o_in_buffer {
                return self.build_raw_chars();
            }
        }

        // W-START CHECK: If raw input starts with 'w', restore in specific cases
        // Vietnamese doesn't have 'w', so words starting with 'w' are likely English
        if is_word_complete && !self.raw_input.is_empty() {
            let (first_key, _, _) = self.raw_input[0];
            if first_key == keys::W {
                // Case 1: English consonant cluster at start (wr, wh) - ALWAYS restore
                // These are English-only clusters that don't exist in Vietnamese
                // Examples: wra, wri, wro (wr-), whi, who (wh-)
                // But NOT: wng, wn, wm (these are w→ư + final consonant = valid Vietnamese)
                if self.raw_input.len() >= 2 {
                    let (second_key, _, _) = self.raw_input[1];
                    // Only restore for English consonant clusters: wr, wh
                    // (r and h after w form English onset clusters)
                    if second_key == keys::R || second_key == keys::H {
                        return self.build_raw_chars();
                    }
                }
                // Case 2: W+vowel with invalid VN buffer - restore
                // Examples: "wmd", "wtf" with invalid structure
                if buffer_invalid_vn {
                    return self.build_raw_chars();
                }
            }
        }

        // Additional check: English patterns in raw_input even when buffer appears valid
        // This catches patterns like "text", "their", "law", "saw", etc.
        // EXCEPTION: If buffer has stroke (đ), this is intentional Vietnamese
        // Example: "derde" → "để" has stroke, keep it (valid VN word)
        // Example: "law" → "lă" has no stroke, restore to "law" (English)
        if is_word_complete && self.has_english_modifier_pattern(true) && raw_input_valid_en {
            // Skip restore if buffer has stroke - user intentionally typed Vietnamese đ
            if !has_stroke {
                return self.build_raw_chars();
            }
        }

        // Check 3: Significant character consumption with circumflex
        // If raw_input is 2+ chars longer than buffer AND buffer has circumflex without mark,
        // this suggests transforms consumed chars that shouldn't have been consumed.
        // Example: "await" (5 chars) → "âit" (3 chars) - diff of 2
        // - "aw" triggers breve on 'a'
        // - second 'a' triggers circumflex (double-vowel), consuming 'w' and second 'a'
        // - Result: buffer is valid but user typed English word
        // EXCEPTION: If buffer has stroke (đ), it's intentional Vietnamese
        if is_word_complete
            && self.raw_input.len() >= self.buf.len() + 2
            && !has_stroke
            && raw_input_valid_en
        {
            let has_circumflex = self.buf.iter().any(|c| c.tone == tone::CIRCUMFLEX);
            let has_marks = self.buf.iter().any(|c| c.mark > 0);
            if has_circumflex && !has_marks {
                return self.build_raw_chars();
            }
        }

        // Check 4: V+C+V circumflex with stop consonant final
        // Pattern: "data" → "dât", "tata" → "tât", "papa" → "pâp"
        // V+C+V triggers circumflex, consuming 1 char (raw_input.len = buf.len + 1)
        // If buffer ends with circumflex + stop consonant (t/c/p) without mark,
        // these are rarely valid Vietnamese words → restore to English
        // Compare: "hôm" (circumflex + m) and "sân" (circumflex + n) are valid Vietnamese
        // NOTE: Use `had_vowel_triggered_circumflex` flag for accurate detection
        if is_word_complete
            && self.had_vowel_triggered_circumflex
            && !has_stroke
            && raw_input_valid_en
        {
            let has_marks = self.buf.iter().any(|c| c.mark > 0);
            if !has_marks {
                let buf_str = self.buf.to_full_string().to_lowercase();
                // Stop consonants after circumflex without mark → likely English
                // Examples: dât, tât, pât, sêt, bôc, etc.
                if buf_str.ends_with("ât")
                    || buf_str.ends_with("êt")
                    || buf_str.ends_with("ôt")
                    || buf_str.ends_with("âc")
                    || buf_str.ends_with("êc")
                    || buf_str.ends_with("ôc")
                    || buf_str.ends_with("âp")
                    || buf_str.ends_with("êp")
                    || buf_str.ends_with("ôp")
                {
                    return self.build_raw_chars();
                }
            }
        }

        // Check 5: Same modifier doubled + vowel = Telex revert pattern
        // When user typed double modifier to revert unwanted Vietnamese transform,
        // the resulting buffer might be valid VN but user intended English.
        // Example: "arro" → user wanted "aro", typed 'rr' to cancel hỏi
        // Only apply to short buffers (<=3 chars) to avoid false positives on words
        // like "issue" (buffer "isue" = 4 chars) or "worry" (buffer "wory" = 4 chars)
        // For no-initial patterns: V + modifier + modifier + V → buf = 3 chars
        if is_word_complete
            && self.had_mark_revert
            && self.buf.len() <= 3
            && raw_input_valid_en
            && !has_stroke
        {
            let tone_modifiers = [keys::S, keys::F, keys::R, keys::X, keys::J];
            let has_same_modifier_doubled_vowel =
                (0..self.raw_input.len().saturating_sub(2)).any(|i| {
                    let (key, _, _) = self.raw_input[i];
                    let (next_key, _, _) = self.raw_input[i + 1];
                    let (after_key, _, _) = self.raw_input[i + 2];
                    tone_modifiers.contains(&key)
                        && key == next_key // Same modifier doubled (rr, ss, ff)
                        && keys::is_vowel(after_key)
                });
            if has_same_modifier_doubled_vowel {
                return self.build_raw_chars();
            }
        }

        // Check 6: V1-V2-V1 vowel pattern that collapsed via circumflex
        // Pattern: raw input has 3+ consecutive vowels ending with same vowel that started
        // Example: "queue" raw=[q,u,e,u,e] → consecutive vowels "eue" → buffer "quêu"
        // The third vowel triggers circumflex on first vowel and gets consumed
        // EXCEPTION: If buffer has stroke (đ), it's intentional Vietnamese
        // EXCEPTION: If buffer has valid Vietnamese triphthong (iêu, yêu, uôi, etc.)
        if is_word_complete && !has_stroke && raw_input_valid_en {
            // Extract consecutive vowel sequence from end of raw_input
            let raw_vowels: Vec<u16> = self
                .raw_input
                .iter()
                .map(|(k, _, _)| *k)
                .filter(|k| keys::is_vowel(*k))
                .collect();

            // Check for V1-V2-V1 pattern (3+ vowels where first and last are same, middle is different)
            if raw_vowels.len() >= 3 {
                let last_three = &raw_vowels[raw_vowels.len() - 3..];
                let v1 = last_three[0];
                let v2 = last_three[1];
                let v3 = last_three[2];

                // V1-V2-V1 pattern: first and last are same vowel, middle is different
                if v1 == v3 && v1 != v2 {
                    // EXCEPTION: Check if buffer contains a valid Vietnamese triphthong
                    // Valid triphthongs: iêu, yêu, uôi, oai, etc. (defined in constants)
                    // If the first 3 raw vowels form a triphthong that matches buffer, it's valid VN
                    // Example: "yeue" → raw_vowels=[Y,E,U,E] → first3=[Y,E,U] → yêu is valid VN
                    // Example: "queue" → raw_vowels=[U,E,U,E] → first3=[U,E,U] → not a VN triphthong
                    let first_three = [raw_vowels[0], raw_vowels[1], raw_vowels[2]];
                    if constants::VALID_TRIPHTHONGS.contains(&first_three) {
                        // Check if buffer actually has this triphthong with proper circumflex
                        let buf_vowels: Vec<(u16, u8)> = self
                            .buf
                            .iter()
                            .filter(|c| keys::is_vowel(c.key))
                            .map(|c| (c.key, c.tone))
                            .collect();
                        if buf_vowels.len() == 3 {
                            let (bv0, _) = buf_vowels[0];
                            let (bv1, bv1_tone) = buf_vowels[1];
                            let (bv2, _) = buf_vowels[2];
                            // Check if buffer matches the triphthong pattern with circumflex on middle vowel
                            // iêu/yêu: circumflex on E (middle)
                            // uôi: circumflex on O (middle)
                            if bv0 == first_three[0]
                                && bv1 == first_three[1]
                                && bv2 == first_three[2]
                                && bv1_tone == tone::CIRCUMFLEX
                            {
                                // Valid Vietnamese triphthong - don't restore
                                return None;
                            }
                        }
                    }

                    // Check if buffer has circumflex on v1 type followed by v2
                    let buf_vowels: Vec<(u16, u8)> = self
                        .buf
                        .iter()
                        .filter(|c| keys::is_vowel(c.key))
                        .map(|c| (c.key, c.tone))
                        .collect();

                    // Buffer should have 2 vowels (V1' with circumflex, V2)
                    if buf_vowels.len() >= 2 {
                        let buf_last_two = &buf_vowels[buf_vowels.len() - 2..];
                        let (buf_v1, buf_v1_tone) = buf_last_two[0];
                        let (buf_v2, _) = buf_last_two[1];

                        // V1 in buffer has circumflex and matches raw V1, V2 matches
                        if buf_v1 == v1
                            && buf_v1_tone == tone::CIRCUMFLEX
                            && buf_v2 == v2
                            && !self.buf.iter().any(|c| c.mark > 0)
                        {
                            return self.build_raw_chars();
                        }
                    }
                }
            }
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

        // Very short words (3 chars or less raw input) → likely intentional revert
        // EXCEPTION: Double 'f' should NOT be treated as intentional revert because
        // 'ff' is extremely common in English (off, iff, aff-, eff-, etc.)
        // These short words should still trigger auto-restore to preserve 'ff'
        // For 'ff', continue to more checks below - don't return true
        if self.raw_input.len() <= 3 && last_key != keys::F {
            return true;
        }

        // For 4-char raw input producing 3-char result (e.g., "SOSS" → "SOS", "varr" → "var"),
        // keep the reverted result. The user explicitly typed double modifier to revert.
        // EXCEPTION: Double 'ss' with buffer ending in 's' - this is invalid VN final
        // Words like "bass", "pass", "boss", "less" should restore to English.
        if self.raw_input.len() == 4 && self.buf.len() == 3 {
            // Double 'ss' at end → buffer ends with 's' → invalid VN final → restore
            if last_key == keys::S {
                if let Some(last_char) = self.buf.last() {
                    if last_char.key == keys::S {
                        // Buffer ends with 's' = invalid Vietnamese final
                        // Return false to allow restore
                        return false;
                    }
                }
            }
            return true;
        }

        // For longer words (5+ chars), check modifier type:
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

    /// Get raw_input as lowercase ASCII string
    fn get_raw_input_string(&self) -> String {
        self.raw_input
            .iter()
            .filter_map(|&(key, caps, _)| utils::key_to_char(key, caps))
            .collect::<String>()
            .to_lowercase()
    }

    /// Get raw_input as ASCII string preserving original case
    fn get_raw_input_string_preserve_case(&self) -> String {
        self.raw_input
            .iter()
            .filter_map(|&(key, caps, shift)| utils::key_to_char_ext(key, caps, shift))
            .collect()
    }

    /// Check if buffer is NOT valid Vietnamese (for unified auto-restore logic)
    ///
    /// Uses full validation including tone requirements (circumflex for êu, etc.)
    /// Also checks for patterns that are structurally valid but not real Vietnamese words.
    /// Returns true if buffer is structurally or phonetically invalid Vietnamese.
    fn is_buffer_invalid_vietnamese(&self) -> bool {
        if self.buf.is_empty() {
            return false;
        }

        // Get keys and tones from buffer
        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
        let buffer_tones: Vec<u8> = self.buf.iter().map(|c| c.tone).collect();
        let buffer_marks: Vec<u8> = self.buf.iter().map(|c| c.mark).collect();

        // Check 1: Basic structural validation
        if !validation::is_valid_with_tones(&buffer_keys, &buffer_tones) {
            return true;
        }

        // Check 2: -ing + tone mark is NOT valid Vietnamese
        // Vietnamese uses -inh (tính, kính), not -ing with tone marks
        // Pattern: [vowel I with tone] + [N] + [G] at the end
        if buffer_keys.len() >= 3 {
            let len = buffer_keys.len();
            if buffer_keys[len - 2] == keys::N
                && buffer_keys[len - 1] == keys::G
                && buffer_keys[len - 3] == keys::I
                && buffer_marks[len - 3] > 0
            {
                // 'i' has a tone mark + ends with 'ng' = invalid (thíng, kíng)
                return true;
            }
        }

        // Check 3: Single vowel validation
        // ALL single vowels with tone marks are valid Vietnamese words
        // Vietnamese-first logic: valid VN → keep VN
        // Examples: á, à, ả, ã, ạ, é, è, ẻ, ẽ, ẹ, í, ì, ỉ, ĩ, ị, ...
        // Also: ồ, ố, ổ, ỗ, ộ (circumflex), ừ, ứ, ử, ữ, ự (horn)
        if buffer_keys.len() == 1 && keys::is_vowel(buffer_keys[0]) && buffer_marks[0] > 0 {
            // Single vowel + any mark → valid Vietnamese, skip restore
            return false;
        }

        // Check 4: C + circumflex vowel (from double vowel) + NO MARK + no final = uncommon
        // "sê", "tê", "pê" are not real Vietnamese words (no mark)
        // But "số" (number), "tế" (cell), etc. with marks ARE valid Vietnamese
        // And "bê" (calf), "mê" (obsessed), "lê" (pear) are valid even without marks
        if buffer_keys.len() == 2 {
            let initial = buffer_keys[0];
            let vowel = buffer_keys[1];
            let vowel_tone = buffer_tones[1];
            let vowel_mark = buffer_marks[1];
            if keys::is_consonant(initial)
                && keys::is_vowel(vowel)
                && vowel_tone == tone::CIRCUMFLEX
                && vowel_mark == 0
            // Only invalid when there's NO mark
            {
                // Check if this is an uncommon pattern
                if constants::UNCOMMON_CIRCUMFLEX_NO_FINAL.contains(&initial) {
                    return true;
                }
            }
        }

        // Check 5: Open diphthong + consonant final = INVALID
        // Open diphthongs (ai, ao, au, ay, eo, iu, oi, ui, ưu) cannot take consonant finals.
        // Example: "mason" → "máon" has diphthong "ao" + final "n" → invalid
        // This catches English words like mason, reason, poison, etc.
        let syllable = syllable::parse(&buffer_keys);
        if syllable.vowel.len() == 2 && !syllable.final_c.is_empty() {
            let vowel_pair = [
                buffer_keys[syllable.vowel[0]],
                buffer_keys[syllable.vowel[1]],
            ];
            // Check if final is a consonant (not semi-vowel that's part of diphthong)
            let final_key = buffer_keys[syllable.final_c[0]];
            let is_consonant_final = matches!(
                final_key,
                keys::C | keys::K | keys::M | keys::N | keys::P | keys::T
            ) || (syllable.final_c.len() == 2); // CH, NG, NH are always consonant finals

            if is_consonant_final && constants::OPEN_DIPHTHONGS.contains(&vowel_pair) {
                return true;
            }
        }

        // Check 6: HORN-O + E pattern is INVALID Vietnamese
        // "oe" is valid diphthong (xoe, hoe), but "ơe" / "ởe" doesn't exist
        // This catches English words like "power" → "pởe", "tower" → "tởe"
        for i in 0..buffer_keys.len().saturating_sub(1) {
            if buffer_keys[i] == keys::O
                && buffer_tones[i] == tone::HORN
                && buffer_keys[i + 1] == keys::E
            {
                return true; // ơe is invalid Vietnamese
            }
        }

        false
    }

    /// Check if raw_input is valid English (for unified auto-restore logic)
    ///
    /// Checks that raw_input contains only basic ASCII letters (A-Z, a-z)
    /// and doesn't have patterns that would indicate Vietnamese typing intent.
    /// Returns true if raw_input looks like an English word.
    fn is_raw_input_valid_english(&self) -> bool {
        if self.raw_input.is_empty() {
            return false;
        }

        // All keys must be ASCII letters (A-Z)
        let all_ascii_letters = self.raw_input.iter().all(|(k, _, _)| {
            // Keys are in range A-Z (from keys.rs)
            // Consonants and vowels are valid English letters
            keys::is_consonant(*k) || keys::is_vowel(*k)
        });

        if !all_ascii_letters {
            return false;
        }

        // Check raw_input is structurally valid (can be parsed as English word)
        // Simplified check: must have at least one vowel (except for short abbreviations)
        let has_vowel = self.raw_input.iter().any(|(k, _, _)| keys::is_vowel(*k));

        // Short words (1-2 chars) without vowels might be abbreviations
        if self.raw_input.len() <= 2 {
            return true;
        }

        has_vowel
    }

    /// Build raw chars from raw_input EXACTLY as typed (no collapsing)
    /// Used for whitelist-based restore where we want the exact English word.
    fn build_raw_chars_exact(&self) -> Option<Vec<char>> {
        // If telex_double_raw is stored (original input before modification), use it
        // plus any subsequent chars typed after the revert
        // Example: "daddy" → telex_double_raw="dadd", subsequent="y" → "daddy"
        if let Some(ref raw_str) = self.telex_double_raw {
            if !raw_str.is_empty() && self.telex_double_raw_len > 0 {
                let mut result: Vec<char> = raw_str.chars().collect();
                // Append subsequent chars from raw_input
                // For stroke revert (dd): raw_input was modified (1 char removed)
                //   → subsequent_start = stored_len - 1
                // For mark revert (ss): raw_input was NOT modified (deferred pop)
                //   → subsequent_start = stored_len
                // Detect by comparing current length with stored length
                let subsequent_start = if self.raw_input.len() < self.telex_double_raw_len {
                    // raw_input was modified (1 char removed)
                    self.telex_double_raw_len.saturating_sub(1)
                } else {
                    // raw_input was NOT modified
                    self.telex_double_raw_len
                };
                for i in subsequent_start..self.raw_input.len() {
                    if let Some(&(key, caps, shift)) = self.raw_input.get(i) {
                        if let Some(ch) = utils::key_to_char_ext(key, caps, shift) {
                            result.push(ch);
                        }
                    }
                }
                return Some(result);
            }
        }
        // Fallback to current raw_input
        let chars: Vec<char> = self
            .raw_input
            .iter()
            .filter_map(|&(key, caps, shift)| utils::key_to_char_ext(key, caps, shift))
            .collect();
        if chars.is_empty() {
            None
        } else {
            Some(chars)
        }
    }

    /// Build raw chars from raw_input for restore
    ///
    /// When a mark was reverted (e.g., "ss" → "s"), decide between buffer and raw_input:
    /// - If after revert there's vowel + consonant pattern → use buffer ("dissable" → "disable")
    /// - If after revert there's only vowels → use raw_input ("issue" → "issue")
    ///
    /// Also handles triple vowel collapse (e.g., "saaas" → "saas"):
    /// - Triple vowel (aaa, eee, ooo) is collapsed to double vowel
    /// - This handles circumflex revert in Telex (aa=â, aaa=aa)
    fn build_raw_chars(&self) -> Option<Vec<char>> {
        let raw_chars: Vec<char> = if self.had_mark_revert && self.should_use_buffer_for_revert() {
            // Use buffer content which already has the correct reverted form
            // e.g., "dissable" → "disable", "usser" → "user"
            self.buf.to_string_preserve_case().chars().collect()
        } else {
            let mut chars: Vec<char> = self
                .raw_input
                .iter()
                .filter_map(|&(key, caps, shift)| utils::key_to_char_ext(key, caps, shift))
                .collect();

            // Collapse vowel patterns for English restore (Telex circumflex patterns)
            // Only collapse when double/triple vowel is IMMEDIATELY followed by tone modifier at END
            // This distinguishes Telex patterns (saax → sax) from real English doubles (wheel, looks)

            // Check for SaaS pattern: same consonant at start and end
            // SaaS, FaaS, etc. should keep the double vowel
            let is_saas_pattern = chars.len() >= 3
                && chars.first().map(|c| c.to_ascii_lowercase())
                    == chars.last().map(|c| c.to_ascii_lowercase())
                && chars
                    .first()
                    .map(|c| !matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y'))
                    .unwrap_or(false);

            // Check if double vowel is immediately followed by tone modifier at end
            // Example: "saax" (s-aa-x) → double 'a' at index 1-2, 'x' at index 3 (end)
            // Counter-example: "looks" (l-oo-k-s) → double 'o' at index 1-2, 'k' at index 3 (NOT modifier)
            // Counter-example: "career" (c-a-r-ee-r) → double 'e' but 'r' is part of English word
            // IMPORTANT: Only collapse for SHORT words (<=4 chars) which are clearly Telex patterns
            // Longer words like "career", "beer", "peer" should keep their double vowels
            let tone_modifiers = ['s', 'f', 'r', 'x', 'j'];
            let has_double_vowel_at_end = chars.len() >= 3 && chars.len() <= 4 && {
                let last = chars[chars.len() - 1].to_ascii_lowercase();
                let second_last = chars[chars.len() - 2].to_ascii_lowercase();
                let third_last = chars[chars.len() - 3].to_ascii_lowercase();
                // Check: double vowel (same letter) + tone modifier at end
                matches!(second_last, 'a' | 'e' | 'o')
                    && second_last == third_last
                    && tone_modifiers.contains(&last)
            };

            // 1. Triple vowel → always collapse to double: "saaas" → "saas"
            let mut i = 0;
            while i + 2 < chars.len() {
                let c = chars[i].to_ascii_lowercase();
                if matches!(c, 'a' | 'e' | 'o')
                    && chars[i].eq_ignore_ascii_case(&chars[i + 1])
                    && chars[i + 1].eq_ignore_ascii_case(&chars[i + 2])
                {
                    chars.remove(i + 1);
                    continue;
                }
                i += 1;
            }

            // 1b. Triple consonant → collapse to double: "bufffer" → "buffer", "afffair" → "affair"
            // English never has 3 consecutive same consonants, so this is always a revert artifact
            let mut i = 0;
            while i + 2 < chars.len() {
                let c = chars[i].to_ascii_lowercase();
                // Check for triple same consonant (common modifiers: s, f, r, x, j)
                if chars[i].eq_ignore_ascii_case(&chars[i + 1])
                    && chars[i + 1].eq_ignore_ascii_case(&chars[i + 2])
                    && !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
                {
                    chars.remove(i + 1);
                    continue;
                }
                i += 1;
            }

            // 2. Double vowel → single ONLY if:
            //    - Double vowel immediately precedes tone modifier at end (Telex pattern)
            //    - NOT SaaS pattern (same consonant at start/end)
            // Example: "saax" → "sax" (aa + x at end)
            // Counter-example: "looks" → "looks" (oo + k, not tone modifier)
            // Counter-example: "saas" → "saas" (SaaS pattern)
            if has_double_vowel_at_end && !is_saas_pattern {
                // Collapse the double vowel (remove one of the paired letters)
                // Position: third_last and second_last are the double vowel
                let pos = chars.len() - 3;
                chars.remove(pos);
            }

            // 3. Double vowel at VERY END → collapse when circumflex was applied then reverted
            // Example: "dataa" → "data" (aa at end, circumflex was applied then reverted)
            // This happens when user types a-a-a and third 'a' reverts the circumflex
            // IMPORTANT: Use had_circumflex_revert, NOT had_mark_revert
            // had_mark_revert is set for tone marks (ff in coffee), which should NOT collapse
            if self.had_circumflex_revert && chars.len() >= 2 {
                let last = chars[chars.len() - 1].to_ascii_lowercase();
                let second_last = chars[chars.len() - 2].to_ascii_lowercase();
                // Double vowel at very end (a/e/o)
                if matches!(last, 'a' | 'e' | 'o') && last == second_last {
                    chars.pop();
                }
            }

            // Collapse double 'w' at start to single 'w' only if there's more after
            // Example: "wwax" → "wax" (double 'w' is Telex revert pattern)
            // But "ww" or "www" alone → keep "ww" (user typed this intentionally)
            if chars.len() > 2
                && chars[0].eq_ignore_ascii_case(&'w')
                && chars[1].eq_ignore_ascii_case(&'w')
            {
                chars.remove(0);
            }

            // Collapse consecutive double tone modifiers when mark was reverted
            // AND one of these conditions:
            // 1. Short buffer (<=3 chars) - user just wanted a diphthong
            //    Example: "arro" → "aro" (buffer="aro" = 3 chars, collapse double 'r')
            // 2. Word starts with "u + doubled_modifier" - rare pattern in English
            //    English words rarely start with u+ss, u+ff, u+rr, etc.
            //    Example: "ussers" → "users" (u+ss is revert artifact)
            //    Counter-example: "issue" (i+ss is common: issue, issuer)
            //    Counter-example: "offers" (o+ff is common: offer, office)
            //
            // EXCEPTION: Never collapse 'ff' because it's very common in English:
            // - off, offer, office, coffee, effect, effort, afford, differ, etc.
            // - Collapsing 'ff' → 'f' would break many common English words
            let tone_modifiers_char = ['s', 'r', 'x', 'j']; // Exclude 'f'
            let starts_with_u_doubled_modifier = chars.len() >= 3
                && chars[0].eq_ignore_ascii_case(&'u')
                && tone_modifiers_char.contains(&chars[1].to_ascii_lowercase())
                && chars[1].eq_ignore_ascii_case(&chars[2]);

            if self.had_mark_revert && (self.buf.len() <= 3 || starts_with_u_doubled_modifier) {
                // Collapse consecutive double modifiers, but skip 'ss'/'ff' at the VERY END
                // Examples:
                // - "usser" → "user" (ss in middle, collapse)
                // - "bass" → "bass" (ss at end, keep)
                // - "buff" → "buff" (ff at end, keep)
                let tone_modifiers = ['s', 'f', 'r', 'x', 'j'];
                let mut i = 0;
                while i + 1 < chars.len() {
                    let c = chars[i].to_ascii_lowercase();
                    let next = chars[i + 1].to_ascii_lowercase();

                    // Skip 'ss' or 'ff' at the VERY END of the word
                    let is_at_end = i + 2 == chars.len();
                    let is_ss_or_ff = (c == 's' && next == 's') || (c == 'f' && next == 'f');
                    if is_at_end && is_ss_or_ff {
                        i += 1;
                        continue;
                    }

                    // Same tone modifier doubled → collapse to single
                    if tone_modifiers.contains(&c) && c == next {
                        chars.remove(i);
                        continue; // Check again at same position for triple+
                    }
                    i += 1;
                }
            }

            // Partial restore: tone + double vowel at end
            // Pattern: C + V + tone_modifier + V + V (same vowel)
            // Example: "tafoo" = t + a + f + o + o → restore to "tàoo"
            // - Keep the tone on first vowel (from 'f' = huyền)
            // - Keep double vowel at end (not collapsed to circumflex)
            if chars.len() == 5 && self.method == 0 {
                // Telex only
                let c0 = chars[0].to_ascii_lowercase();
                let c1 = chars[1].to_ascii_lowercase();
                let c2 = chars[2].to_ascii_lowercase();
                let c3 = chars[3].to_ascii_lowercase();
                let c4 = chars[4].to_ascii_lowercase();

                // Check pattern: consonant + vowel + tone_modifier + vowel + vowel (same)
                let is_consonant_0 = !matches!(c0, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
                let is_vowel_1 = matches!(c1, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
                let is_tone_2 = matches!(c2, 's' | 'f' | 'r' | 'x' | 'j');
                let is_circumflex_vowel_34 = matches!(c3, 'a' | 'e' | 'o') && c3 == c4;

                if is_consonant_0 && is_vowel_1 && is_tone_2 && is_circumflex_vowel_34 {
                    // Build: C + (V with tone) + V + V
                    let toned_vowel = match (c1, c2) {
                        ('a', 's') => 'á',
                        ('a', 'f') => 'à',
                        ('a', 'r') => 'ả',
                        ('a', 'x') => 'ã',
                        ('a', 'j') => 'ạ',
                        ('e', 's') => 'é',
                        ('e', 'f') => 'è',
                        ('e', 'r') => 'ẻ',
                        ('e', 'x') => 'ẽ',
                        ('e', 'j') => 'ẹ',
                        ('i', 's') => 'í',
                        ('i', 'f') => 'ì',
                        ('i', 'r') => 'ỉ',
                        ('i', 'x') => 'ĩ',
                        ('i', 'j') => 'ị',
                        ('o', 's') => 'ó',
                        ('o', 'f') => 'ò',
                        ('o', 'r') => 'ỏ',
                        ('o', 'x') => 'õ',
                        ('o', 'j') => 'ọ',
                        ('u', 's') => 'ú',
                        ('u', 'f') => 'ù',
                        ('u', 'r') => 'ủ',
                        ('u', 'x') => 'ũ',
                        ('u', 'j') => 'ụ',
                        ('y', 's') => 'ý',
                        ('y', 'f') => 'ỳ',
                        ('y', 'r') => 'ỷ',
                        ('y', 'x') => 'ỹ',
                        ('y', 'j') => 'ỵ',
                        _ => c1,
                    };
                    // Preserve case
                    let toned_vowel = if chars[1].is_uppercase() {
                        toned_vowel.to_uppercase().next().unwrap_or(toned_vowel)
                    } else {
                        toned_vowel
                    };
                    return Some(vec![chars[0], toned_vowel, chars[3], chars[4]]);
                }
            }

            chars
        };

        if raw_chars.is_empty() {
            return None;
        }

        // Optimization: If raw_chars equals current buffer, no restore needed
        // This happens when user manually reverted (e.g., "usser" → "user")
        // Avoids unnecessary backspace + retype of the same content
        let buffer_str: String = self.buf.to_string_preserve_case();
        let raw_str: String = raw_chars.iter().collect();
        if buffer_str == raw_str {
            return None;
        }

        Some(raw_chars)
    }

    /// Determine if buffer should be used for restore after a mark revert
    ///
    /// Heuristic: Use buffer when it forms a recognizable English word pattern,
    /// OR when raw_input looks like a typo (double letter + single vowel at end).
    ///
    /// Examples:
    /// - "dissable" → buffer "disable" has dis- prefix → use buffer
    /// - "soffa" → double ff + single vowel 'a' at end → use buffer "sofa"
    /// - "issue" → iss + ue pattern (double + multiple chars) → use raw_input "issue"
    /// - "error" → err + or pattern (double + multiple chars) → use raw_input "error"
    fn should_use_buffer_for_revert(&self) -> bool {
        let buf_str = self.buf.to_lowercase_string();

        // Common English prefixes that suggest intentional revert
        const PREFIXES: &[&str] = &[
            "dis", "mis", "un", "re", "de", "pre", "anti", "non", "sub", "trans", "con",
        ];

        // Common English suffixes
        const SUFFIXES: &[&str] = &[
            "able", "ible", "tion", "sion", "ment", "ness", "less", "ful", "ing", "ive", "ified",
            "ous", "ory",
        ];

        // Short suffixes for common words (need minimum buffer length check)
        // Examples: "user" (ends with -er), "color" (ends with -or)
        const SHORT_SUFFIXES: &[&str] = &["er", "or"];

        // Check if buffer matches common English word patterns
        // Use >= to include short words like "transit" (7 chars) with "trans" (5 chars)
        for prefix in PREFIXES {
            if buf_str.starts_with(prefix) && buf_str.len() >= prefix.len() + 2 {
                return true;
            }
        }

        // Check if raw_input contains double 'ss' or 'ff' with MULTIPLE chars after
        // The rule: double letter + multiple chars after → use raw (English word)
        //           double letter + single char after → use buffer (revert pattern)
        // Examples:
        // - "massive" (ss at pos 2-3, then ive) → use raw "massive"
        // - "soffa" (ff at pos 2-3, then just 'a') → use buffer "sofa"
        // - "masson" (ss at pos 2-3, then on) → use buffer "mason" (open diphthong case)
        let raw_len = self.raw_input.len();
        for i in 0..raw_len.saturating_sub(1) {
            let (k1, _, _) = self.raw_input[i];
            let (k2, _, _) = self.raw_input[i + 1];
            if (k1 == keys::S && k2 == keys::S) || (k1 == keys::F && k2 == keys::F) {
                // Found double at position i, i+1
                // Check how many chars follow after the double
                let chars_after_double = raw_len - (i + 2);

                // Special case: buffer ends with common single-consonant patterns
                // like "-son", "-ton", "-ron" (mason, reason, person, etc.)
                // These are much more common than double-consonant versions
                // so prefer buffer when this pattern is detected
                let common_single_consonant_endings = ["son", "ton", "ron", "non", "mon"];
                let use_buffer_for_ending = common_single_consonant_endings
                    .iter()
                    .any(|ending| buf_str.ends_with(ending));

                if chars_after_double >= 2 && !use_buffer_for_ending {
                    // Multiple chars after double → likely English word, use raw
                    return false;
                }
                // Only 0-1 char after double, or common ending → likely revert pattern
                if use_buffer_for_ending {
                    return true;
                }
                break;
            }
        }

        // Suffix check: only use buffer if raw_input is exactly 1 char longer
        // This indicates user typed double modifier to revert, and buffer has the collapsed form.
        // Example: "verrified" (9 chars) → buffer "verified" (8 chars) → use buffer
        // Counter-example: "massive" (7 chars) → buffer "masive" (6 chars) → raw has 7, buf has 6
        //   But raw_input for "massive" should be 7 chars... let me check
        // Actually for double 's' at end, we skip the pop, so raw stays at 7 chars.
        // For double 'r' in "verrified", we don't skip the pop, so raw becomes 8 chars? No wait...
        // The issue is that suffix check runs AFTER the pop logic on space.
        // For "verrified": pop removes one 'r', so raw becomes 8 chars = buffer.len
        // For "massive": we skip pop for double 's', so raw stays 7 chars > buffer 6
        // So the condition should be: raw_input.len() == buf_str.len() + 1 means double was NOT at end
        // and raw_input.len() == buf_str.len() means pop happened (double was at end of pattern)
        for suffix in SUFFIXES {
            if buf_str.ends_with(suffix) && buf_str.len() >= suffix.len() + 2 {
                // Only use buffer if lengths match (pop happened) or diff is 1 (expected revert pattern)
                // This filters out cases like "massive" where raw has legitimate double letter
                if self.raw_input.len() <= buf_str.len() + 1 {
                    return true;
                }
            }
        }

        // Check short suffixes with stricter conditions:
        // - Buffer must be exactly 4 chars (short words like "user", not longer like "userer")
        // - Must end with -er or -or
        // - Raw input must have exactly 5 chars (one more than buffer due to double modifier)
        // - The double must be 'ss' only (not 'ff', 'rr', etc.) because:
        //   - "usser" → "user" is a common typing pattern when reverting sắc mark
        //   - "offer", "differ", "suffer" are legitimate English words with double 'f'
        //   - "error", "mirror" have double 'r' as legitimate English
        // - The double 's' must appear exactly twice (not "assessor")
        if buf_str.len() == 4 && self.raw_input.len() == 5 {
            for suffix in SHORT_SUFFIXES {
                if buf_str.ends_with(suffix) {
                    // Only check for double 's' at position 1,2 (0-indexed)
                    // Pattern: V-SS-V-C like "usser" → "user"
                    let (key_1, _, _) = self.raw_input[1];
                    let (key_2, _, _) = self.raw_input[2];
                    if key_1 == keys::S && key_2 == keys::S {
                        // Check 's' appears exactly twice
                        let s_count = self
                            .raw_input
                            .iter()
                            .filter(|(k, _, _)| *k == keys::S)
                            .count();
                        if s_count == 2 {
                            return true;
                        }
                    }
                }
            }
        }

        // Check if raw_input has double 'f' followed by single vowel at end
        // Pattern: "soffa" → double 'f' + single 'a' → likely typo, use buffer "sofa"
        // Only apply for 'f' because:
        // - Double 'f' + vowel at end is rare in English (no common words like "staffa")
        // - Double 's'/'r' + vowel has many valid words (worry, sorry, carry, etc.)
        if self.raw_input.len() >= 4 {
            let len = self.raw_input.len();
            let (last_key, _, _) = self.raw_input[len - 1];
            let (second_last_key, _, _) = self.raw_input[len - 2];
            let (third_last_key, _, _) = self.raw_input[len - 3];

            // Only for double 'f' + single vowel at end
            if keys::is_vowel(last_key) && second_last_key == keys::F && third_last_key == keys::F {
                return true;
            }

            // Double 's' + single vowel at end (but not 'y' to avoid "sorry" → "sory")
            // Pattern: "raisse" → buffer "raise" (double 's' + single 'e' → use buffer)
            // This handles cases where user typed extra 's' for sắc mark then reverted
            // Exclude 'y' because words like "sorry", "carry" are common English
            let is_core_vowel = matches!(
                last_key,
                k if k == keys::A || k == keys::E || k == keys::I || k == keys::O || k == keys::U
            );
            if is_core_vowel && second_last_key == keys::S && third_last_key == keys::S {
                return true;
            }

            // Double 's' at very end: distinguish revert pattern from English words
            // "thiss" → buffer "this" (revert pattern, buffer is valid word)
            // "guess" → raw "guess" (valid English word with double 's')
            // Heuristics for when buffer is a valid English word:
            // 1. Buffer starts with common English digraph (th, wh, ch, sh)
            // 2. Buffer ends with consonant + 's' (common plural: sims, gaps, maps)
            //    vs buffer ends with vowel + 's' (less common: gues, mues)
            if last_key == keys::S && second_last_key == keys::S && len == buf_str.len() + 1 {
                let starts_with_digraph = buf_str.starts_with("th")
                    || buf_str.starts_with("wh")
                    || buf_str.starts_with("ch")
                    || buf_str.starts_with("sh");
                if starts_with_digraph {
                    return true;
                }

                // Check if buffer ends with consonant + 's' (common English plural pattern)
                // "sims" = m + s (consonant + s) → use buffer
                // "gues" = e + s (vowel + s) → use raw "guess"
                if buf_str.len() >= 2 {
                    let chars: Vec<char> = buf_str.chars().collect();
                    let second_last_char = chars[chars.len() - 2];
                    let last_char = chars[chars.len() - 1];
                    // Check consonant + 's' pattern (plural)
                    let is_plural_pattern = last_char == 's'
                        && !matches!(second_last_char, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
                    if is_plural_pattern {
                        return true;
                    }
                }
            }
        }

        // Generic check: double Telex modifier in middle with EXACTLY 2 chars after
        // Pattern: raw has double modifier (ss/ff/rr/xx/jj) followed by V+C (vowel+consonant)
        // Examples:
        // - "sarrah" → "sarah" (double 'r' + "ah" = V+C)
        // - "usser" → "user" (double 's' + "er" = V+C) [also handled by specific check above]
        //
        // IMPORTANT constraints to avoid false positives on real English words:
        // 1. Buffer must be plain ASCII (no Vietnamese transforms)
        // 2. Raw must end with consonant (not vowel like "issue")
        // 3. Suffix after double must be short: exactly 2 chars (V+C pattern)
        //    This excludes "current" (suffix "ent" = 3 chars), "effect" (suffix "ect" = 3 chars)
        // 4. Only apply if exactly 2 occurrences of the modifier (not "assess")
        // 5. For safety, only apply to double 'r', 'x', 'j' (not 's' or 'f' which are more
        //    common in legitimate English doubles like "professor", "different")
        //    Double 's' is already handled by specific check above.
        // 6. Exclude common English suffixes after double consonant:
        //    - "ow" (borrow, sorrow, tomorrow), "or" (error, mirror, horror)
        //    - "ry"/"y" (carry, sorry, worry), "ed" (occurred, referred)
        //    These are legitimate English words, not typing mistakes.
        const RARE_DOUBLE_MODIFIERS: &[u16] = &[keys::R, keys::X, keys::J];

        if self.raw_input.len() >= 4 && self.raw_input.len() == buf_str.len() + 1 {
            // Constraint 1: Buffer must be plain ASCII (no Vietnamese transforms)
            let has_transforms = self
                .buf
                .iter()
                .any(|c| c.tone > 0 || c.mark > 0 || c.stroke);
            if has_transforms {
                return false;
            }

            // Constraint 2: Raw must end with consonant
            let (last_key, _, _) = self.raw_input[self.raw_input.len() - 1];
            if !keys::is_consonant(last_key) {
                return false;
            }

            // Constraint 6: Exclude common English suffixes after double consonant
            // Get last 2 key codes
            let (last_key_1, _, _) = self.raw_input[self.raw_input.len() - 1];
            let (last_key_2, _, _) = self.raw_input[self.raw_input.len() - 2];

            // Common English suffixes that appear after double consonants:
            // - "ow" (borrow, sorrow), "or" (error, mirror), "ry" (carry, sorry, worry)
            // - "ed" (occurred, referred), "ly" (hurriedly)
            // Note: "er" is NOT excluded here because the SHORT_SUFFIXES check above
            // handles 4-char words ending with "er", and longer words like "error"
            // have 3+ occurrences which is excluded by occurrence count check.
            let is_common_suffix = matches!(
                (last_key_2, last_key_1),
                (keys::O, keys::W)   // ow: borrow, sorrow
                    | (keys::O, keys::R) // or: error, mirror, horror
                    | (keys::R, keys::Y) // ry: carry, sorry, worry
                    | (keys::E, keys::D) // ed: occurred, referred
                    | (keys::L, keys::Y) // ly: hurriedly
            );
            if is_common_suffix {
                return false;
            }

            // Find double modifier with exactly 2 chars after (V+C or C+C pattern)
            for i in 0..self.raw_input.len().saturating_sub(2) {
                let (key_i, _, _) = self.raw_input[i];
                let (key_next, _, _) = self.raw_input[i + 1];

                if RARE_DOUBLE_MODIFIERS.contains(&key_i) && key_i == key_next {
                    // Double modifier found at position i, i+1
                    let chars_after_double = self.raw_input.len() - (i + 2);

                    // Constraint 3: Exactly 2 chars after double
                    // This excludes longer suffixes like "ent" (current), "ect" (effect)
                    if chars_after_double == 2 {
                        // Count total occurrences of this modifier
                        let occurrence_count = self
                            .raw_input
                            .iter()
                            .filter(|(k, _, _)| *k == key_i)
                            .count();

                        // Constraint 4: Only 2 occurrences
                        if occurrence_count == 2 {
                            return true;
                        }
                    }
                }
            }
        }

        // Check for short words with double modifier at end that reverted
        // Pattern: "thiss" → buffer "this"
        // Raw input ends with double modifier (ss, rr, ff, xx, jj)
        // Buffer has 4+ chars ending with that consonant
        // Only apply if double modifier at end is the ONLY occurrence of that char
        // This preserves "assess" (multiple 's') while converting "thiss" → "this"
        // IMPORTANT: Only use buffer if it's VALID Vietnamese structure.
        // If buffer is invalid (like "gues" ending with 's'), use raw input "guess" instead.
        if self.raw_input.len() >= 4 && buf_str.len() >= 4 && buf_str.len() <= 6 {
            let len = self.raw_input.len();
            let (last_key, _, _) = self.raw_input[len - 1];
            let (second_last_key, _, _) = self.raw_input[len - 2];

            // Check for double modifier at end (ss, rr, ff, xx, jj)
            let tone_modifiers = [keys::S, keys::F, keys::R, keys::X, keys::J];
            if tone_modifiers.contains(&last_key) && last_key == second_last_key {
                // Count occurrences of this modifier key in raw_input
                let occurrence_count = self
                    .raw_input
                    .iter()
                    .filter(|(k, _, _)| *k == last_key)
                    .count();
                // Only apply if the double at end is the only occurrence (exactly 2)
                if occurrence_count == 2 {
                    // Buffer should end with that consonant (after revert)
                    let expected_char = match last_key {
                        k if k == keys::S => 's',
                        k if k == keys::F => 'f',
                        k if k == keys::R => 'r',
                        k if k == keys::X => 'x',
                        k if k == keys::J => 'j',
                        _ => '\0',
                    };
                    if expected_char != '\0' && buf_str.ends_with(expected_char) {
                        // Check if buffer is valid Vietnamese structure
                        // If not (like "gues" ending with invalid final 's'), don't use buffer
                        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
                        let buffer_tones: Vec<u8> = self.buf.iter().map(|c| c.tone).collect();
                        if validation::is_valid_with_tones(&buffer_keys, &buffer_tones) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Check for English patterns in raw_input that suggest non-Vietnamese
    ///
    /// Patterns detected:
    /// 1. Modifier (s/f/r/x/j in Telex) followed by consonant: "text" (x before t)
    /// 2. Modifier at end of long word (>2 chars): "their" (r at end)
    /// 3. Modifier after first vowel then another vowel: "use" (s between u and e)
    /// 4. Consonant + W + vowel without tone modifiers (only on word complete): "swim"
    fn has_english_modifier_pattern(&self, is_word_complete: bool) -> bool {
        let tone_modifiers = [keys::S, keys::F, keys::R, keys::X, keys::J];

        // CRITICAL: Detect tone override pattern - vowel + mod1 + mod2 + vowel
        // Example: "chajfo" = ch + [A + J + F + O] → A is first vowel, J→F tone override, O completes diphthong
        // Pattern must be: first_vowel + modifier1 + modifier2 + second_vowel (all consecutive)
        // BUT: if raw_input is in English dictionary (like "cursor"), restore to English
        if is_word_complete && !self.buf.is_empty() {
            // Find position of first vowel
            let first_vowel_pos = self
                .raw_input
                .iter()
                .position(|(k, _, _)| keys::is_vowel(*k));

            if let Some(vowel_pos) = first_vowel_pos {
                // Check for pattern: vowel + mod1 + mod2 + vowel at positions vowel_pos..vowel_pos+4
                if vowel_pos + 3 < self.raw_input.len() {
                    let (k1, _, _) = self.raw_input[vowel_pos + 1];
                    let (k2, _, _) = self.raw_input[vowel_pos + 2];
                    let (k3, _, _) = self.raw_input[vowel_pos + 3];

                    let has_tone_override = tone_modifiers.contains(&k1)
                        && tone_modifiers.contains(&k2)
                        && k1 != k2  // Different modifiers (j→f, not jj)
                        && keys::is_vowel(k3); // Followed by vowel

                    if has_tone_override {
                        // Check if raw is in English dictionary
                        let raw_str: String = self
                            .raw_input
                            .iter()
                            .filter_map(|&(k, c, s)| utils::key_to_char_ext(k, c, s))
                            .collect();
                        let raw_in_dict = english_dict::is_english_word(&raw_str);

                        // If raw is NOT in English dict AND buffer is valid Vietnamese, keep it
                        if !raw_in_dict && !self.is_buffer_invalid_vietnamese() {
                            return false;
                        }
                    }
                }
            }
        }

        // Single vowel + modifiers only → valid Vietnamese (á, é, í, ó, ú, ý, etc.)
        // ALL single vowels with tone marks are valid Vietnamese words
        // Examples: "as" → "á", "es" → "é", "is" → "í", "or" → "ỏ", "us" → "ú"
        // Vietnamese-first logic: valid VN → keep VN (don't check if raw looks English)
        if self.raw_input.len() >= 2 {
            let (first, _, _) = self.raw_input[0];
            if keys::is_vowel(first) && first != keys::W {
                let all_after_are_modifiers = self.raw_input[1..]
                    .iter()
                    .all(|(k, _, _)| tone_modifiers.contains(k));
                if all_after_are_modifiers {
                    // Vowel + mark modifiers only → valid Vietnamese, not English
                    return false;
                }
            }
        }

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

                // W-as-vowel pattern: When W is converted to ư, treat it as a vowel position
                // This means mark modifiers (s, f, r, x, j) immediately after W are tone marks
                // for the ư vowel, not consonants.
                // Examples: "wf" → "ừ", "ws" → "ứ", "wmf" → "ừm"

                // Check for "W + only mark modifiers" pattern → valid Vietnamese (ừ, ứ, ử, ữ, ự)
                // This handles standalone W with tone marks like "wf " → "ừ "
                let all_are_modifiers = self.raw_input[1..]
                    .iter()
                    .all(|(k, _, _)| tone_modifiers.contains(k));
                if all_are_modifiers && !self.raw_input[1..].is_empty() {
                    // W + mark modifiers only → valid Vietnamese, not English
                    return false;
                }

                // Check for "W + consonant + mark modifier" pattern → valid Vietnamese
                // Examples: "wmf" → "ừm", "wms" → "ứm", "wng" → "ưng"
                // Pattern: W (→ư) + valid_final_consonant + optional_mark (NO other vowels!)
                // "west" has vowel E, so it should NOT match this pattern
                if self.raw_input.len() >= 2 {
                    // First check if there are any other vowels after W
                    let has_other_vowels = self.raw_input[1..]
                        .iter()
                        .any(|(k, _, _)| keys::is_vowel(*k) && *k != keys::W);

                    // Only apply W+consonant+mark pattern if there are NO other vowels
                    if !has_other_vowels {
                        let non_modifier_consonants: Vec<u16> = self.raw_input[1..]
                            .iter()
                            .filter(|(k, _, _)| {
                                keys::is_consonant(*k) && !tone_modifiers.contains(k)
                            })
                            .map(|(k, _, _)| *k)
                            .collect();

                        let has_mark_modifier = self.raw_input[1..]
                            .iter()
                            .any(|(k, _, _)| tone_modifiers.contains(k));

                        // W + valid_final + mark → valid Vietnamese (ừm, ứng, etc.)
                        if !non_modifier_consonants.is_empty() && has_mark_modifier {
                            let is_valid_final = match non_modifier_consonants.len() {
                                1 => {
                                    constants::VALID_FINALS_1.contains(&non_modifier_consonants[0])
                                }
                                2 => {
                                    let pair =
                                        [non_modifier_consonants[0], non_modifier_consonants[1]];
                                    constants::VALID_FINALS_2.contains(&pair)
                                }
                                _ => false,
                            };
                            if is_valid_final {
                                return false; // Valid Vietnamese pattern
                            }
                        }
                    }
                }

                // Analyze pattern: W + vowels + consonants
                // Find position of first vowel to distinguish consonants from modifiers
                let first_vowel_pos = self.raw_input[1..]
                    .iter()
                    .position(|(k, _, _)| keys::is_vowel(*k) && *k != keys::W);

                let vowels_after: Vec<u16> = self.raw_input[1..]
                    .iter()
                    .filter(|(k, _, _)| keys::is_vowel(*k) && *k != keys::W)
                    .map(|(k, _, _)| *k)
                    .collect();

                // Only exclude Telex mark modifiers (s, f, r, x, j) when they come AFTER a vowel
                // If they come BEFORE any vowel, they're consonants (e.g., "wra" has 'r' as consonant)
                // EXCEPTION: When W is at start (w-as-vowel) and NO other vowels, modifiers are marks
                let consonants_after: Vec<u16> = self.raw_input[1..]
                    .iter()
                    .enumerate()
                    .filter(|(i, (k, _, _))| {
                        if !keys::is_consonant(*k) || *k == keys::W {
                            return false;
                        }
                        // For W-as-vowel WITHOUT other vowels, treat modifiers as marks
                        // e.g., "wf" → "ừ", "wmf" → "ừm" (no other vowels, so f is mark)
                        // But "wra" has vowel A, so R should be treated as consonant
                        if vowels_after.is_empty() && tone_modifiers.contains(k) {
                            return false;
                        }
                        // Modifier keys AFTER first vowel are tone modifiers, not consonants
                        if let Some(vowel_pos) = first_vowel_pos {
                            if *i > vowel_pos && tone_modifiers.contains(k) {
                                return false;
                            }
                        }
                        true
                    })
                    .map(|(_, (k, _, _))| *k)
                    .collect();

                // W + vowel + consonant → likely English like "win", "water"
                // W + consonant only → valid Vietnamese (ưng, ưn, ưm)
                // EXCEPTION: W+O+final is valid Vietnamese "ươ+final" (ương, ươn, ươm, ươc, ươt, ươp)
                if !vowels_after.is_empty() && !consonants_after.is_empty() {
                    // Check for W+O+valid_final pattern (ương, ươn, ươm, etc.)
                    // raw_input: [W, O, N, G] → valid Vietnamese ương
                    // raw_input: [W, O, M] → valid Vietnamese ươm
                    let is_wo_final_pattern = vowels_after.len() == 1
                        && vowels_after[0] == keys::O
                        && match consonants_after.len() {
                            1 => {
                                // Single consonant finals: n, m, c, t, p
                                matches!(
                                    consonants_after[0],
                                    keys::N | keys::M | keys::C | keys::T | keys::P
                                )
                            }
                            2 => {
                                // Double consonant finals: ng, nh
                                let pair = [consonants_after[0], consonants_after[1]];
                                pair == [keys::N, keys::G] || pair == [keys::N, keys::H]
                            }
                            _ => false,
                        };
                    if is_wo_final_pattern {
                        // Valid Vietnamese "ươ+final", don't restore
                        return false;
                    }
                    // Both vowels and consonants after W → likely English
                    return true;
                }

                // W + vowel only → check if valid Vietnamese pattern
                // Valid: ưa (W+A), ươ (W+O), ưu (W+U)
                // Invalid: ưe (W+E), ưi (W+I), ưy (W+Y) → restore as English
                if !vowels_after.is_empty() && consonants_after.is_empty() {
                    let valid_vowels_after_w = [keys::A, keys::O, keys::U];
                    let has_invalid_vowel = vowels_after
                        .iter()
                        .any(|v| !valid_vowels_after_w.contains(v));
                    if has_invalid_vowel {
                        return true;
                    }
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
                            // Exception: C+W+O+NG pattern is Vietnamese "ương" (tương, sương, etc.)
                            // Pattern: consonant + W + O + N + G → valid Vietnamese diphthong
                            if third == keys::O && self.raw_input.len() >= 5 {
                                let (fourth, _, _) = self.raw_input[3];
                                let (fifth, _, _) = self.raw_input[4];
                                if fourth == keys::N && fifth == keys::G {
                                    // This is Vietnamese "ương" pattern, don't restore
                                    return false;
                                }
                            }

                            // Issue #151: C+W+A pattern is Vietnamese "ưa" (mưa, cưa, lưa, etc.)
                            // Pattern: consonant + W + A → valid Vietnamese diphthong
                            // When raw_input is exactly 3 chars (C+W+A), this is Vietnamese
                            // Examples: mwa → mưa, cwa → cưa, lwa → lưa, twa → tưa
                            if third == keys::A && self.raw_input.len() == 3 {
                                return false;
                            }

                            // C+W+U pattern is Vietnamese "ưu" (lưu, mưu, cưu, etc.)
                            // Pattern: consonant + W + U → valid Vietnamese diphthong
                            // Examples: lwu → lưu, mwu → mưu, cwu → cưu
                            if third == keys::U && self.raw_input.len() == 3 {
                                return false;
                            }

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

        // Pattern: Consecutive tone modifiers followed by VOWEL (English pattern)
        // Example: "cursor" = c-u-r-s-o-r → "rs" followed by vowel 'o' → English
        // Counter-example: "đướng" typed as dduowfsng → "fs" followed by consonant 'n' → Vietnamese
        // Vietnamese allows consecutive modifiers for tone adjustment (f→s changes huyền to sắc)
        for i in 0..self.raw_input.len().saturating_sub(2) {
            let (key, _, _) = self.raw_input[i];
            let (next_key, _, _) = self.raw_input[i + 1];
            let (after_key, _, _) = self.raw_input[i + 2];
            // Two DIFFERENT consecutive modifiers followed by vowel → English
            // Example: "cursor" = c-u-r-s-o-r → "rs" (r≠s) followed by vowel 'o' → English
            // Same modifier doubled (rr, ss, ff) is Telex revert pattern, NOT English
            // Example: "arro" = a-r-r-o → "rr" (r=r) is revert pattern → skip
            if tone_modifiers.contains(&key)
                && tone_modifiers.contains(&next_key)
                && key != next_key // Only different modifiers indicate English
                && keys::is_vowel(after_key)
            {
                return true;
            }
        }

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
                // Also exclude tone modifier keys (S, F, R, X, J) - these are mark keys, not consonants
                // when they appear after a vowel. Example: "dduowfs" has 'f' then 's', both are modifiers.
                let is_true_consonant = keys::is_consonant(next_key)
                    && next_key != keys::W
                    && !tone_modifiers.contains(&next_key);
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
                    // OE before S modifier at end is English "-oes" pattern
                    // Examples: "goes", "does", "toes", "woes", "foes", "hoes"
                    // Vietnamese "oe" diphthong (hoè, xoè) typically has different tone placement
                    // or final consonant, not bare "oe + sắc at end"
                    // IMPORTANT: Only check for S (sắc) modifier, NOT F/R/X/J
                    // This allows "moef" → "moè", "boef" → "boè" (Vietnamese huyền)
                    // EXCEPTION: "oes" without initial consonant is Vietnamese exclamation "oé"
                    // EXCEPTION: Vietnamese-specific initials (kh, gh, ngh, tr, ph, etc.) + oe + modifier
                    //            Example: "khoer" → "khoẻ" (healthy), "nhoer" → "nhoẻ"
                    if v1 == keys::O && v2 == keys::E && total_vowels == 2 && key == keys::S {
                        // Check for Vietnamese-specific initials (both digraphs and single consonants)
                        // Vietnamese OE words from dictionary: hòe, loè, tóe, xòe, khoẻ, ngoé...
                        let is_vietnamese_oe_initial = if self.raw_input.len() >= 2 {
                            let (c1, _, _) = self.raw_input[0];
                            let (c2, _, _) = self.raw_input[1];

                            // Vietnamese digraphs: kh, gh, ph, ch, th, nh (end with H), tr, ng
                            let ends_with_h = c2 == keys::H
                                && matches!(
                                    c1,
                                    k if k == keys::K
                                        || k == keys::G
                                        || k == keys::P
                                        || k == keys::C
                                        || k == keys::T
                                        || k == keys::N
                                );
                            let is_tr = c1 == keys::T && c2 == keys::R;
                            let is_ng = c1 == keys::N && c2 == keys::G;

                            // Single consonant + OE: check if c2 is O (meaning c1 is single initial)
                            // Valid Vietnamese single initials for OE: H, L, T, X, B, S
                            // Dictionary: hòe, loè, tóe, xòe, boe (boẻ), soé
                            let is_single_initial_oe = c2 == keys::O
                                && matches!(
                                    c1,
                                    keys::H | keys::L | keys::T | keys::X | keys::B | keys::S
                                );

                            ends_with_h || is_tr || is_ng || is_single_initial_oe
                        } else {
                            false
                        };

                        if is_vietnamese_oe_initial {
                            continue; // Vietnamese word, don't restore
                        }

                        // Only return true if there's an initial consonant (goes, does, foes, woes)
                        // Words without initial like "oes" → "oé" should stay Vietnamese
                        let has_initial =
                            !self.raw_input.is_empty() && keys::is_consonant(self.raw_input[0].0);
                        if has_initial {
                            return true;
                        }
                    }
                }

                // Pattern 2b: P + single vowel + modifier at end → English
                // P alone (not PH) is rare in native Vietnamese
                // Example: "per" = P + E + R → pẻ (but "per" is English preposition)
                if self.raw_input.len() >= 2 && self.raw_input[0].0 == keys::P {
                    let is_ph = self.raw_input.len() >= 2 && self.raw_input[1].0 == keys::H;
                    if !is_ph {
                        // Count vowels before modifier
                        let vowels_before: usize = (0..i)
                            .filter(|&j| keys::is_vowel(self.raw_input[j].0))
                            .count();
                        // P + single vowel + modifier at end (no more chars after modifier)
                        if vowels_before == 1 && i + 1 == self.raw_input.len() {
                            return true;
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
                    // EXCEPT: Vietnamese diphthongs without initial consonant
                    // U + modifier + A: ủa, ùa, úa, ũa, ụa (interjections)
                    //
                    // LINGUISTIC RULE: Vietnamese syllables have consonant BETWEEN vowel and modifier
                    // - "onro" = O + N + R + O → N separates first O from R → Vietnamese "ổn"
                    // - "use"  = U + S + E     → S directly after U → English
                    // This distinguishes intentional Vietnamese (vowel-consonant-modifier-vowel)
                    // from accidental English (vowel-modifier-vowel without consonant)
                    let has_consonant_between = (first_vowel_pos + 1 < i)
                        && keys::is_consonant(self.raw_input[first_vowel_pos + 1].0);
                    if !has_initial_consonant && !has_consonant_between {
                        let first_vowel = self.raw_input[first_vowel_pos].0;
                        // Vietnamese no-initial patterns:
                        // - Same vowel doubling: OFO → ồ, EFE → ề, AFA → ầ (circumflex + tone)
                        // - U + modifier + A: ủa, ùa, úa (interjections)
                        // - U + modifier + I: ủi, ùi, úi (ủi quần, úi chà)
                        // - U + modifier + Y: ủy, ùy, úy, uỵ (uy tín, uỵch - valid Vietnamese)
                        // - A + modifier + O: ảo, ào, áo (ảo giác, ảo tưởng)
                        // - A + modifier + I: ải, ái, ài (interjections)
                        // - A + modifier + Y: ảy, áy, ày (cảy, máy when no initial)
                        // - O + modifier + I: ỏi, ói, òi (interjections)
                        // - O + modifier + E: oẻ, oẹ, oé (interjections)
                        // - E + modifier + O: ẻo, ẹo, éo (interjections)
                        let is_vietnamese_no_initial = first_vowel == next_key // Same vowel = Telex circumflex
                            || (first_vowel == keys::U && (next_key == keys::A || next_key == keys::I || next_key == keys::Y))
                            || (first_vowel == keys::A && (next_key == keys::O || next_key == keys::I || next_key == keys::Y))
                            || (first_vowel == keys::O && (next_key == keys::I || next_key == keys::E))
                            || (first_vowel == keys::E && next_key == keys::O);

                        // Special case: no-initial O + modifier + E
                        // "ore" is common English word, should restore
                        // "oer", "oje" are not English, keep Vietnamese (oẻ, oẹ)
                        if first_vowel == keys::O && next_key == keys::E {
                            let raw_str: String = self
                                .raw_input
                                .iter()
                                .filter_map(|&(k, c, s)| utils::key_to_char_ext(k, c, s))
                                .collect();
                            if english_dict::is_english_word(&raw_str) {
                                return true; // Restore to English
                            }
                            // Not English word, keep Vietnamese
                            continue;
                        }

                        if !is_vietnamese_no_initial {
                            return true;
                        }
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
                        let (prev_char, _, _) = self.raw_input[i - 1];
                        // Skip if prev char is not a vowel (e.g., "ddense" has n before s)
                        // Pattern requires vowel + modifier + vowel
                        if !keys::is_vowel(prev_char) {
                            continue;
                        }
                        let prev_vowel = prev_char;
                        // Same vowel is Telex circumflex doubling (aa, ee, oo)
                        // Example: "loxoi" = l+o+x+O+i → O after X is same vowel doubling
                        // EXCEPTION: If a CONSONANT follows the doubled vowel, it's likely English
                        // Example: "param" = p+a+r+A+m → 'A' after 'r' same as before, 'm' (consonant) follows
                        // Counter-example: "loxoi" has 'i' (vowel) after → Vietnamese diphthong
                        if prev_vowel == next_key {
                            // Check if there are more chars after the second vowel
                            if i + 2 < self.raw_input.len() {
                                let (char_after, _, _) = self.raw_input[i + 2];
                                // Only English if followed by CONSONANT (param has 'm')
                                // If followed by vowel (loxoi has 'i'), it's Vietnamese diphthong
                                //
                                // EXCEPTION: Vietnamese delayed circumflex + final consonant
                                // "vajan" = v + a + j + a + n → "vận" (valid Vietnamese)
                                // "hajan" = h + a + j + a + n → "hận"
                                // Pattern: initial + vowel + mark + same_vowel + valid_final
                                // This creates circumflex+mark on vowel (ậ, ẫ, ầ, ẩ, ặ, etc.)
                                // BUT: "param" has same pattern but IS in English dict → restore
                                if keys::is_consonant(char_after) {
                                    // Check if this forms valid Vietnamese syllable with circumflex
                                    // Circumflex vowels (â, ê, ô) + valid finals
                                    let is_circumflex_vowel =
                                        matches!(prev_vowel, keys::A | keys::E | keys::O);
                                    let is_valid_final =
                                        constants::VALID_FINALS_1.contains(&char_after);

                                    if is_circumflex_vowel && is_valid_final {
                                        // Could be Vietnamese delayed circumflex (vận, hận)
                                        // But if raw input is in English dict → restore to English
                                        let raw_str: String = self
                                            .raw_input
                                            .iter()
                                            .filter_map(|&(k, c, s)| {
                                                utils::key_to_char_ext(k, c, s)
                                            })
                                            .collect();
                                        if english_dict::is_english_word(&raw_str) {
                                            return true; // English word (param, etc.)
                                        }
                                        // Not in English dict → keep Vietnamese (vận, hận, etc.)
                                    } else {
                                        return true; // Not Vietnamese pattern → English
                                    }
                                }
                            }
                            continue; // Same vowel without consonant after is Telex pattern
                        }
                        // Vietnamese exceptions: diphthongs with tone modifier in middle
                        let is_vietnamese_pattern = match prev_vowel {
                            k if k == keys::U => {
                                // ua: của, mủa; uo: được; uy: thuỷ, quỷ; ui: tụi, mủi, cúi, núi
                                // ue: handled separately below (like OE) to check English dict
                                next_key == keys::A
                                    || next_key == keys::O
                                    || next_key == keys::Y
                                    || next_key == keys::I
                            }
                            k if k == keys::A => {
                                // au: màu, náu, cau, lau, etc.
                                next_key == keys::I
                                    || next_key == keys::Y
                                    || next_key == keys::O
                                    || next_key == keys::U
                            }
                            k if k == keys::O => {
                                // oi: bói, hói; oa: hoá, toá
                                // oe: NOT included here - handled separately via English dict check
                                // Words like "lore", "bore", "core" are common English
                                next_key == keys::I || next_key == keys::A
                            }
                            k if k == keys::E => {
                                // eo: đeo, kẹo, mèo
                                // eu: nếu, kêu (êu diphthong with tone on ê)
                                next_key == keys::O || next_key == keys::U
                            }
                            k if k == keys::I => {
                                // iu: chịu, nịu, lịu; ia: mía, kia, chia, tía
                                next_key == keys::U || next_key == keys::A
                            }
                            _ => false,
                        };
                        if !is_vietnamese_pattern {
                            // Special case: O + modifier + E
                            // Vietnamese-first for specific patterns, English otherwise
                            // "chose" → "choé" (CH is Vietnamese digraph, keep VN)
                            // "lore", "bore" → English (single initial + raw in EN dict)
                            // "xofe", "hofe" → Vietnamese (raw not in EN dict)
                            if prev_vowel == keys::O && next_key == keys::E {
                                // Check for Vietnamese digraph initials (CH, KH, GH, TH, PH, NH, NG, TR)
                                // These are Vietnamese-specific, so keep Vietnamese for OE words
                                let has_vn_digraph = if self.raw_input.len() >= 2 {
                                    let (c1, _, _) = self.raw_input[0];
                                    let (c2, _, _) = self.raw_input[1];
                                    // Digraphs ending with H: CH, KH, GH, TH, PH, NH
                                    let ends_with_h = c2 == keys::H
                                        && matches!(
                                            c1,
                                            keys::C
                                                | keys::K
                                                | keys::G
                                                | keys::T
                                                | keys::P
                                                | keys::N
                                        );
                                    // Other digraphs: NG, TR
                                    let is_ng = c1 == keys::N && c2 == keys::G;
                                    let is_tr = c1 == keys::T && c2 == keys::R;
                                    ends_with_h || is_ng || is_tr
                                } else {
                                    false
                                };

                                if has_vn_digraph {
                                    // Vietnamese digraph + OE → keep Vietnamese
                                    continue;
                                }

                                // For single initial + OE: only restore if raw is English word
                                let raw_str: String = self
                                    .raw_input
                                    .iter()
                                    .filter_map(|&(k, c, s)| utils::key_to_char_ext(k, c, s))
                                    .collect();
                                if !english_dict::is_english_word(&raw_str) {
                                    // Not a common English word, keep Vietnamese
                                    continue;
                                }
                            }

                            // Special case: U + modifier + E (similar to OE handling)
                            // Vietnamese-first for specific patterns, English otherwise
                            // "huse" → "hué" (H + UE is valid Vietnamese pattern)
                            // "cure", "pure", "sure" → English (common English words)
                            // "xuse", "zuse" → Vietnamese (raw not in EN dict)
                            if prev_vowel == keys::U && next_key == keys::E {
                                // Check for Vietnamese digraph initials (QU is special for UE)
                                let has_vn_digraph = if self.raw_input.len() >= 2 {
                                    let (c1, _, _) = self.raw_input[0];
                                    let (c2, _, _) = self.raw_input[1];
                                    // QU + E pattern: quế, qué (valid Vietnamese)
                                    let is_qu = c1 == keys::Q && c2 == keys::U;
                                    // Digraphs ending with H: CH, KH, GH, TH, PH, NH
                                    let ends_with_h = c2 == keys::H
                                        && matches!(
                                            c1,
                                            keys::C
                                                | keys::K
                                                | keys::G
                                                | keys::T
                                                | keys::P
                                                | keys::N
                                        );
                                    // Other digraphs: NG, TR
                                    let is_ng = c1 == keys::N && c2 == keys::G;
                                    let is_tr = c1 == keys::T && c2 == keys::R;
                                    is_qu || ends_with_h || is_ng || is_tr
                                } else {
                                    false
                                };

                                if has_vn_digraph {
                                    // Vietnamese digraph + UE → keep Vietnamese
                                    continue;
                                }

                                // For single initial + UE: only restore if raw is English word
                                let raw_str: String = self
                                    .raw_input
                                    .iter()
                                    .filter_map(|&(k, c, s)| utils::key_to_char_ext(k, c, s))
                                    .collect();
                                if !english_dict::is_english_word(&raw_str) {
                                    // Not a common English word, keep Vietnamese
                                    continue;
                                }
                            }
                            return true;
                        }
                    }
                }
            }
        }

        // Pattern 5: W at end after vowel → English (like "raw", "law", "saw", "view")
        // W as final is not valid Vietnamese, it's an English pattern
        // Exception: "uw" ending is Vietnamese (tuw → tư)
        // Exception: "ow" ending is Vietnamese (cow → cơ)
        // Exception: W modified a diphthong (oiw → ơi where OI is diphthong, W adds horn to O)
        if self.raw_input.len() >= 2 {
            let (last, _, _) = self.raw_input[self.raw_input.len() - 1];
            if last == keys::W {
                let (second_last, _, _) = self.raw_input[self.raw_input.len() - 2];
                // W after vowel (not U or O) at end is English: raw, law, saw
                // W after U is Vietnamese: tuw → tư
                // W after O is Vietnamese: cow → cơ
                if keys::is_vowel(second_last) && second_last != keys::U && second_last != keys::O {
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

        // Pattern 6: Double vowel (oo, aa, ee) followed by K → English
        // Vietnamese uses single vowel + breve + K (đắk = aw+k)
        // English uses double vowel + K (looks, took, book)
        // This distinguishes "looks" (English) from "đắk" (Vietnamese)
        if self.raw_input.len() >= 3 {
            for i in 0..self.raw_input.len() - 2 {
                let (v1, _, _) = self.raw_input[i];
                let (v2, _, _) = self.raw_input[i + 1];
                let (next, _, _) = self.raw_input[i + 2];

                // Check for double vowel (same vowel twice) followed by K
                if keys::is_vowel(v1) && v1 == v2 && next == keys::K {
                    return true;
                }
            }
        }

        // Pattern 6a: Double E (ee) followed by P at END → English (keep, deep, sleep, seep)
        // Only EE+P, not AA+P or OO+P which can be valid Vietnamese (cấp = caaps)
        // ONLY check at word boundary - mid-word "kêp" could still become valid Vietnamese
        // Exceptions:
        //   - I+EE+P is Vietnamese "iệp" pattern (nghiệp, hiệp, kiệp, v.v.)
        //   - X+EE+P is Vietnamese "xếp" pattern (xếp = to arrange)
        if is_word_complete && self.raw_input.len() >= 3 {
            let len = self.raw_input.len();
            let (last, _, _) = self.raw_input[len - 1];
            if last == keys::P {
                let (v1, _, _) = self.raw_input[len - 3];
                let (v2, _, _) = self.raw_input[len - 2];
                // Only match EE (not AA or OO)
                if v1 == keys::E && v2 == keys::E {
                    // Exception: I+EE+P or X+EE+P are Vietnamese patterns
                    // Check if there's an I or X before the double E
                    if len >= 4 {
                        let (before_ee, _, _) = self.raw_input[len - 4];
                        if before_ee == keys::I || before_ee == keys::X {
                            // This is Vietnamese "iêp" or "xêp" pattern, don't restore
                            // Continue to check other patterns
                        } else {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
            }
        }

        // Pattern 6b: Double vowel (aa, ee, oo) followed by tone modifier at end → English
        // ONLY when initial is rare in Vietnamese (S alone, F alone)
        // Example: "saas" = s + aa + s → S initial + double 'a' + tone modifier 's' → SaaS pattern
        // Example: "saax" = s + aa + x → S initial + double 'a' + tone modifier 'x' → English
        // Counter-example: "leex" = l + ee + x → L is common Vietnamese initial → keep "lễ"
        // Counter-example: "meex" = m + ee + x → M is common Vietnamese initial → keep "mễ"
        // Counter-example: "soos" = s + oo + s → "số" (Vietnamese for "number")
        // Counter-example: "seef" = s + ee + f → "sề" (valid Vietnamese word)
        let tone_modifiers = [keys::S, keys::F, keys::R, keys::X, keys::J];
        if self.raw_input.len() >= 4 {
            let (first, _, _) = self.raw_input[0];
            let (last, _, _) = self.raw_input[self.raw_input.len() - 1];
            // Only match if initial is S or F (rare alone in Vietnamese)
            // S alone (not SH) and F are English patterns
            if (first == keys::S || first == keys::F) && tone_modifiers.contains(&last) {
                // Check for double vowel just before the last key
                let (v1, _, _) = self.raw_input[self.raw_input.len() - 3];
                let (v2, _, _) = self.raw_input[self.raw_input.len() - 2];
                if keys::is_vowel(v1) && v1 == v2 {
                    // Exception: S/F + OO/EE + modifier → Vietnamese
                    // - số, sở, sỗ, sổ (number-related words)
                    // - sề, sể, sễ, sệ (valid Vietnamese words)
                    // S/F + AA + modifier → English (SaaS, FaaS patterns)
                    if v1 != keys::O && v1 != keys::E {
                        return true;
                    }
                }
            }
        }

        // Pattern 6c: S + A + X pattern → English "sax" (saxophone)
        // Only match "sax" specifically, not "six" (which is Vietnamese "sĩ")
        // "sax" = s + a + x → "sã" but should restore to "sax"
        // "six" = s + i + x → "sĩ" (valid Vietnamese: soldier, scholar)
        if self.raw_input.len() == 3 {
            let (first, _, _) = self.raw_input[0];
            let (second, _, _) = self.raw_input[1];
            let (third, _, _) = self.raw_input[2];
            // Only S + A + X (not other vowels)
            if first == keys::S && second == keys::A && third == keys::X {
                return true;
            }
        }

        // Pattern 7: C + V + tone_modifier + double_vowel → partial English restore
        // Example: "tafoo" = t + a + f + o + o → restore to "tàoo"
        // Example: "mufaa" = m + u + f + a + a → restore to "mùaa"
        // This pattern detects when someone types like "tattoo" with Vietnamese tone
        if self.raw_input.len() == 5 {
            let (c0, _, _) = self.raw_input[0];
            let (c1, _, _) = self.raw_input[1];
            let (c2, _, _) = self.raw_input[2];
            let (c3, _, _) = self.raw_input[3];
            let (c4, _, _) = self.raw_input[4];

            let is_consonant_0 = keys::is_consonant(c0);
            let is_vowel_1 = keys::is_vowel(c1);
            let is_tone_2 = matches!(c2, keys::S | keys::F | keys::R | keys::X | keys::J);
            let is_circumflex_vowel_34 = matches!(c3, keys::A | keys::E | keys::O) && c3 == c4;

            if is_consonant_0 && is_vowel_1 && is_tone_2 && is_circumflex_vowel_34 {
                return true;
            }
        }

        // Pattern 8: tone_modifier + K at end → English (risk, disk, task, mask)
        // K as final is only valid in Vietnamese with breve vowels (Đắk Lắk ethnic minority words)
        // or other ethnic minority patterns like "Búk"
        // Example: "risk" = r + i + s + k → should restore to "risk" (s NOT consumed)
        // Counter-example: "đắk" = dd + aw + k → "đắk" (breve 'ắ', valid Vietnamese)
        // Counter-example: "Busk" = B + u + s + k → "Búk" (s consumed as sắc, valid Vietnamese)
        if self.raw_input.len() >= 4 {
            let (last, _, _) = self.raw_input[self.raw_input.len() - 1];
            if last == keys::K {
                let (second_last, _, _) = self.raw_input[self.raw_input.len() - 2];
                let tone_modifiers = [keys::S, keys::F, keys::R, keys::X, keys::J];
                // Check if second_last is a tone modifier (s, f, r, x, j)
                if tone_modifiers.contains(&second_last) {
                    // Key insight: if modifier was consumed (applied to vowel),
                    // buf.len() < raw_input.len() → Vietnamese
                    // If modifier was NOT consumed (stayed as letter),
                    // buf.len() == raw_input.len() → English
                    // Example: "Busk" → "Búk" (4 chars → 3 chars, s consumed)
                    // Example: "risk" → "rík" (4 chars → 3 chars, s consumed)
                    // Both have s consumed, so we need another check...

                    // Vietnamese ethnic minority words have breve: ắ, ẳ, ẵ (from 'aw')
                    // Check if there's a 'w' in raw_input before the modifier (indicating breve)
                    let has_breve_marker = self.raw_input[..self.raw_input.len() - 2]
                        .iter()
                        .any(|(k, _, _)| *k == keys::W);

                    // Also check for common English -Vsk patterns where V is i, a, e, o, u
                    // but NOT ethnic minority patterns
                    // The key difference: ethnic minority words are usually short (3-4 letters)
                    // and have specific structures. English -sk words often have more consonants.
                    let (third_last, _, _) = self.raw_input[self.raw_input.len() - 3];
                    let is_isk_ask_pattern = keys::is_vowel(third_last)
                        && second_last == keys::S
                        && !has_breve_marker
                        && self.raw_input.len() >= 4;

                    // Only restore if it's a common English -Vsk pattern (V+s+k)
                    // AND there's no breve marker (aw pattern)
                    // AND word has at least one consonant before the vowel (like r-i-s-k, d-i-s-k)
                    if is_isk_ask_pattern {
                        // Check if there's a consonant initial before the vowel
                        let has_consonant_before_vowel =
                            self.raw_input.len() >= 4 && keys::is_consonant(self.raw_input[0].0);

                        // For short words (4 chars like "risk", "disk", "task"),
                        // only restore if initial is a common English consonant pattern
                        if has_consonant_before_vowel {
                            // Skip restore for ethnic minority initials that commonly use K final
                            // B, L are common in Vietnamese ethnic minority words (Búk, Lắk)
                            // Note: Đắk uses DD (double D) for Đ, not single D
                            // So D initial (disk, desk, dusk) should restore as English
                            let (first, _, _) = self.raw_input[0];
                            let is_ethnic_initial = first == keys::B || first == keys::L;

                            if !is_ethnic_initial {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        // Pattern 9: C + V + M + S at end → English plural pattern (-ms)
        // Example: "sims" = s + i + m + s → English (The Sims, rims)
        // Example: "gems" = g + e + m + s → English plural
        // Counter-example: "dims" = d + i + m + s → "dím" is valid Vietnamese (to press/push down)
        // Counter-example: "sems" = s + e + m + s → "sém" is valid Vietnamese (burnt/scorched)
        // Counter-example: "hems" = h + e + m + s → "hẻm" is valid Vietnamese (alley)
        //
        // KEY INSIGHT: If buffer has a TONE MARK applied, the 's' was consumed as Vietnamese tone
        // modifier (sắc), not as English plural suffix. Keep Vietnamese in this case.
        // Words that DON'T get tone mark applied are true English plurals.
        if self.raw_input.len() == 4 {
            let (c0, _, _) = self.raw_input[0];
            let (c1, _, _) = self.raw_input[1];
            let (c2, _, _) = self.raw_input[2];
            let (c3, _, _) = self.raw_input[3];

            // Pattern: single consonant + i/e + m + s (tone modifier)
            if keys::is_consonant(c0)
                && (c1 == keys::I || c1 == keys::E)
                && c2 == keys::M
                && c3 == keys::S
            {
                // Check if this C+ÍM/ÉM combination is a real Vietnamese word.
                // Vietnamese dictionary 22k contains these C+ÍM/ÉM words:
                // - *ím: bím, dím, mím, tím (initials: B, D, M, T)
                // - *ém: kém, lém, ném, sém, tém (initials: K, L, N, S, T)
                // English plurals: sims, rims, gems, hems (NOT Vietnamese words)
                let is_vietnamese_cim_word = (c1 == keys::I
                    && matches!(c0, keys::B | keys::D | keys::M | keys::T))
                    || (c1 == keys::E
                        && matches!(c0, keys::K | keys::L | keys::N | keys::S | keys::T));

                if !is_vietnamese_cim_word {
                    // Not a known Vietnamese word → English plural pattern
                    return true;
                }
                // Vietnamese word → don't trigger English pattern
            }
        }

        false
    }

    /// Try to convert bracket key to vowel: ] → ư, [ → ơ (Issue #159)
    ///
    /// Returns Some(Result) if bracket was converted, None otherwise.
    /// Handles:
    /// - ] at word start or after consonant → ư
    /// - [ at word start or after consonant → ơ
    /// - Double bracket reverts: ]] → ], [[ → [, uppercase revert → } or {
    /// - Valid Vietnamese vowel combinations: ươ (from ][)
    fn try_bracket_as_vowel(&mut self, key: u16, caps: bool) -> Option<Result> {
        // Check if bracket shortcut is enabled
        if !self.bracket_shortcut {
            return None;
        }

        // Check for revert: if last transform was BracketAsVowel with same bracket
        if self.last_transform == Some(Transform::BracketAsVowel) && !self.buf.is_empty() {
            if let Some(last_char) = self.buf.last() {
                // Check if last char matches the bracket we're typing
                let should_revert = match key {
                    keys::RBRACKET => last_char.key == keys::U && last_char.tone == tone::HORN,
                    keys::LBRACKET => last_char.key == keys::O && last_char.tone == tone::HORN,
                    _ => false,
                };

                if should_revert {
                    // Remove the vowel we added
                    self.buf.pop();
                    // Also remove from raw_input
                    self.raw_input.pop();
                    // Clear transform
                    self.last_transform = None;

                    // Return the original bracket character
                    // Use caps (Shift or CapsLock) to decide: uppercase → {/}, lowercase → [/]
                    let bracket_char = match (key, caps) {
                        (keys::RBRACKET, true) => '}',
                        (keys::RBRACKET, false) => ']',
                        (keys::LBRACKET, true) => '{',
                        (keys::LBRACKET, false) => '[',
                        (_, true) => '{',  // fallback (shouldn't happen)
                        (_, false) => '[', // fallback (shouldn't happen)
                    };
                    return Some(Result::send_consumed(1, &[bracket_char]));
                }
            }
        }

        // Determine target vowel based on bracket key
        let base_key = if key == keys::RBRACKET {
            keys::U // ] → ư (U with horn)
        } else {
            keys::O // [ → ơ (O with horn)
        };

        // Add vowel to buffer (similar to W shortcut pattern)
        self.buf.push(Char::new(base_key, caps));

        // Set horn tone to make ư or ơ
        if let Some(c) = self.buf.get_mut(self.buf.len() - 1) {
            c.tone = tone::HORN;
        }

        // Validate: is this valid Vietnamese?
        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
        let buffer_tones: Vec<u8> = self.buf.iter().map(|c| c.tone).collect();
        if !is_valid_with_tones(&buffer_keys, &buffer_tones) {
            // Invalid - remove the vowel we added
            self.buf.pop();
            return None;
        }

        // Track raw input for ESC restore
        self.raw_input.push((key, caps, false));

        // Mark transform
        self.last_transform = Some(Transform::BracketAsVowel);
        self.had_any_transform = true;

        // Return result with key consumed (don't pass through bracket)
        let vowel_char = chars::to_char(base_key, caps, tone::HORN, 0).unwrap();
        Some(Result::send_consumed(0, &[vowel_char]))
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
        ("ver", "vẻ"), // v + e + r(hỏi) → vẻ (test for #issue)
        // Post-tone delayed circumflex: o + n + r(hỏi) + o(circumflex) → ổn
        ("onro", "ổn"),
        // ===== Invalid diphthong blocking =====
        // When vowel combination is NOT valid Vietnamese diphthong,
        // same-vowel circumflex should NOT be triggered
        // Pattern: initial + V1 + V2(invalid) + consonant + V1
        ("coupo", "coupo"), // [O,U] invalid diphthong → stays raw
        ("soupo", "soupo"), // [O,U] invalid → stays raw
        ("beapa", "beapa"), // [E,A] invalid → stays raw
        ("beipi", "beipi"), // [E,I] invalid → stays raw
        ("daupa", "daupa"), // [A,U] valid diphthong but "aup" invalid syllable → stays raw
        ("boemo", "boemo"), // [O,E] valid diphthong but "oem" invalid syllable → stays raw
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

    // =========================================================================
    // AUTO-RESTORE TESTS
    // Test space-triggered auto-restore for all Telex modifiers (s/f/r/x/j)
    // When user types double modifier to revert, then continues typing,
    // pressing space should restore to the buffer form (with revert applied)
    // =========================================================================

    // =========================================================================
    // AUTO-RESTORE TESTS for double modifier patterns
    //
    // Generic check handles: double 'r', 'x', 'j' with EXACTLY 2 chars after
    // Double 's' is handled by existing specific check (5 chars raw, 4 chars buf)
    // Double 'f' has too many legitimate English words (effect, different, etc.)
    //
    // Constraint: suffix after double must be exactly 2 chars (V+C pattern)
    // This avoids false positives like "current" (suffix "ent" = 3 chars)
    // =========================================================================

    // Auto-restore with double 'r' (hỏi mark)
    // Pattern: double 'r' + exactly 2 chars (V+C)
    const TELEX_AUTO_RESTORE_R: &[(&str, &str)] = &[
        ("sarrah ", "sarah "), // s-a-rr-a-h: suffix "ah" = 2 chars ✓
        ("barrut ", "barut "), // b-a-rr-u-t: suffix "ut" = 2 chars ✓
        ("tarrep ", "tarep "), // t-a-rr-e-p: suffix "ep" = 2 chars ✓
    ];

    // Auto-restore with double 'x' (ngã mark)
    // Pattern: double 'x' + exactly 2 chars
    const TELEX_AUTO_RESTORE_X: &[(&str, &str)] = &[
        ("maxxat ", "maxat "), // m-a-xx-a-t: suffix "at" = 2 chars ✓
        ("texxup ", "texup "), // t-e-xx-u-p: suffix "up" = 2 chars ✓
    ];

    // Auto-restore with double 'j' (nặng mark)
    // Pattern: double 'j' + exactly 2 chars
    const TELEX_AUTO_RESTORE_J: &[(&str, &str)] = &[
        ("majjam ", "majam "), // m-a-jj-a-m: suffix "am" = 2 chars ✓
        ("bajjut ", "bajut "), // b-a-jj-u-t: suffix "ut" = 2 chars ✓
    ];

    #[test]
    fn test_auto_restore_double_r() {
        for (input, expected) in TELEX_AUTO_RESTORE_R {
            let mut e = Engine::new();
            e.set_english_auto_restore(true);
            let result = type_word(&mut e, input);
            assert_eq!(
                result, *expected,
                "[Auto-restore R] '{}' → '{}', expected '{}'",
                input, result, expected
            );
        }
    }

    #[test]
    fn test_auto_restore_double_x() {
        for (input, expected) in TELEX_AUTO_RESTORE_X {
            let mut e = Engine::new();
            e.set_english_auto_restore(true);
            let result = type_word(&mut e, input);
            assert_eq!(
                result, *expected,
                "[Auto-restore X] '{}' → '{}', expected '{}'",
                input, result, expected
            );
        }
    }

    #[test]
    fn test_auto_restore_double_j() {
        for (input, expected) in TELEX_AUTO_RESTORE_J {
            let mut e = Engine::new();
            e.set_english_auto_restore(true);
            let result = type_word(&mut e, input);
            assert_eq!(
                result, *expected,
                "[Auto-restore J] '{}' → '{}', expected '{}'",
                input, result, expected
            );
        }
    }

    /// Issue: Typing "DDD" with shift/capslock held → should produce "DD", not "Dd"
    /// When stroke is reverted (ddd → dd), the added 'd' must preserve caps state
    #[test]
    fn test_stroke_revert_preserves_caps() {
        // Uppercase: DDD → Đ → DD (both chars uppercase)
        let cases: &[(&str, &str)] = &[
            ("DDD", "DD"),   // caps: D→D, DD→Đ, DDD→DD (both uppercase)
            ("ddd", "dd"),   // no caps: d→d, dd→đ, ddd→dd
            ("DDd", "Dd"),   // mixed: first two caps, third lowercase → Dd
            ("ddD", "dD"),   // mixed: first two lowercase, third caps → dD
            ("DDDD", "DDD"), // 4 D's: after revert, stroke_reverted=true, 4th D added
            ("dddd", "ddd"), // 4 d's: after revert, stroke_reverted=true, 4th d added
        ];

        for (input, expected) in cases {
            let mut e = Engine::new();
            let result = type_word(&mut e, input);
            assert_eq!(
                result, *expected,
                "[Stroke caps] '{}' → '{}', expected '{}'",
                input, result, expected
            );
        }
    }

    /// Comprehensive tests for Vietnamese diphthong patterns with interleaved tone marks.
    /// Pattern: C + V1 + tone + V2 (e.g., "misa" = m + i + s + a → mía)
    /// These patterns should NOT trigger auto-restore because they are valid Vietnamese.
    const TELEX_INTERLEAVED_DIPHTHONG: &[(&str, &str)] = &[
        // IA diphthong: mía, kia, chia, tía
        ("misa ", "mía "), // m + i + s(sắc) + a → mía
        ("kisa ", "kía "), // k + i + s + a → kía
        ("lisa ", "lía "), // l + i + s + a → lía
        ("tifa ", "tìa "), // t + i + f(huyền) + a → tìa
        ("nira ", "nỉa "), // n + i + r(hỏi) + a → nỉa
        // UA diphthong: múa, lúa, cúa
        ("musa ", "múa "), // m + u + s + a → múa
        ("lusa ", "lúa "), // l + u + s + a → lúa
        ("cufa ", "cùa "), // c + u + f + a → cùa
        // UE diphthong: huế, tuệ, quế (valid Vietnamese, tone on E)
        ("huse ", "hué "), // h + u + s + e → hué (tone moves to E)
        ("tuse ", "tué "), // t + u + s + e → tué (tone on E)
        // OA diphthong: hoá, toá (tone on A, second vowel)
        ("hosa ", "hoá "), // h + o + s + a → hoá (tone on A)
        ("tosa ", "toá "), // t + o + s + a → toá (tone on A)
        ("lora ", "loả "), // l + o + r + a → loả (tone on A)
        // AI diphthong: mái, lái, tái
        ("masi ", "mái "), // m + a + s + i → mái
        ("lasi ", "lái "), // l + a + s + i → lái
        ("tafi ", "tài "), // t + a + f + i → tài
        // AO diphthong: cáo, náo, báo
        ("caso ", "cáo "), // c + a + s + o → cáo
        ("naso ", "náo "), // n + a + s + o → náo
        ("bafo ", "bào "), // b + a + f + o → bào
        // AU diphthong: máu, láu, càu
        ("masu ", "máu "), // m + a + s + u → máu
        ("lasu ", "láu "), // l + a + s + u → láu
        ("cafu ", "càu "), // c + a + f + u → càu (huyền, not circumflex)
        // AY diphthong: máy, cáy, lấy
        ("masy ", "máy "), // m + a + s + y → máy
        ("casy ", "cáy "), // c + a + s + y → cáy
        ("lafy ", "lày "), // l + a + f + y → lày
        // OI diphthong: bói, hói, đói
        ("bosi ", "bói "), // b + o + s + i → bói
        ("hofi ", "hòi "), // h + o + f + i → hòi
        ("lori ", "lỏi "), // l + o + r + i → lỏi
        // UI diphthong: núi, cúi, tủi
        ("nusi ", "núi "), // n + u + s + i → núi
        ("cufi ", "cùi "), // c + u + f + i → cùi
        ("turi ", "tủi "), // t + u + r + i → tủi
        // IU diphthong: chịu, nịu, lịu
        ("chiju ", "chịu "), // ch + i + j(nặng) + u → chịu
        ("niju ", "nịu "),   // n + i + j + u → nịu
        ("lisu ", "líu "),   // l + i + s + u → líu
        // EO diphthong: méo, kẹo, bèo
        ("meso ", "méo "), // m + e + s + o → méo
        ("kejo ", "kẹo "), // k + e + j + o → kẹo
        ("befo ", "bèo "), // b + e + f + o → bèo
    ];

    #[test]
    fn test_interleaved_diphthong_auto_restore() {
        for (input, expected) in TELEX_INTERLEAVED_DIPHTHONG {
            let mut e = Engine::new();
            e.set_english_auto_restore(true);
            let result = type_word(&mut e, input);
            assert_eq!(
                result, *expected,
                "[Interleaved diphthong] '{}' → '{}', expected '{}'",
                input, result, expected
            );
        }
    }
}
