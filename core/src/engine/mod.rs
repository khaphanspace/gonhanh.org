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
    keys,
    vowel::{Phonology, Vowel},
};
use crate::input::{self, ToneType};
use crate::utils;
use buffer::{Buffer, Char, MAX};
use shortcut::{InputMethod, ShortcutTable};
use validation::{is_foreign_word_pattern, is_valid};

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
    /// W as vowel ư (for revert: ww → w)
    WAsVowel,
    /// W shortcut was explicitly skipped (prevent re-transformation)
    WShortcutSkipped,
}

/// Main Vietnamese IME engine
pub struct Engine {
    buf: Buffer,
    method: u8,
    enabled: bool,
    last_transform: Option<Transform>,
    shortcuts: ShortcutTable,
    /// Raw keystroke history for ESC restore (key, caps)
    raw_input: Vec<(u16, bool)>,
    /// Raw mode: skip Vietnamese transforms after prefix chars (@ # $ ^ : > ?)
    raw_mode: bool,
    /// True if current word has non-letter characters before letters
    /// Used to prevent false shortcut matches (e.g., "149k" should not match "k")
    has_non_letter_prefix: bool,
    /// Skip w→ư shortcut in Telex mode (user preference)
    /// When true, typing 'w' at word start stays as 'w' instead of converting to 'ư'
    skip_w_shortcut: bool,
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
            raw_mode: false,
            has_non_letter_prefix: false,
            skip_w_shortcut: false,
        }
    }

    pub fn set_method(&mut self, method: u8) {
        self.method = method;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.buf.clear();
        }
    }

    /// Set whether to skip w→ư shortcut in Telex mode
    pub fn set_skip_w_shortcut(&mut self, skip: bool) {
        self.skip_w_shortcut = skip;
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
            return Result::none();
        }

        // Raw mode prefix detection: when buffer is empty and user types @ # $ ^ : > ?
        // Enable raw mode to skip Vietnamese transforms for subsequent letters
        if self.buf.is_empty() && Self::is_raw_prefix(key, shift) {
            self.raw_mode = true;
            return Result::none();
        }

        // Check for word boundary shortcuts ONLY on SPACE
        if key == keys::SPACE {
            let result = self.try_word_boundary_shortcut();
            self.clear();
            return result;
        }

        // ESC key: restore to raw ASCII (undo all Vietnamese transforms)
        if key == keys::ESC {
            let result = self.restore_to_raw();
            self.clear();
            return result;
        }

        // Other break keys (punctuation, arrows, etc.) just clear buffer
        if keys::is_break(key) {
            self.clear();
            return Result::none();
        }

        if key == keys::DELETE {
            // If buffer is already empty, user is deleting content from previous word
            // that we don't track. Mark this to prevent false shortcut matches.
            // e.g., "đa" + SPACE + backspace×2 + "a" should NOT match shortcut "a"
            if self.buf.is_empty() {
                self.has_non_letter_prefix = true;
            }
            self.buf.pop();
            self.raw_input.pop();
            self.last_transform = None;
            return Result::none();
        }

        // Record raw keystroke for ESC restore (letters and numbers only)
        if keys::is_letter(key) || keys::is_number(key) {
            self.raw_input.push((key, caps));
        }

        self.process(key, caps, shift)
    }

    /// Main processing pipeline - pattern-based
    fn process(&mut self, key: u16, caps: bool, shift: bool) -> Result {
        // Raw mode: skip all Vietnamese transforms, just pass through letters
        // Enabled by typing @ # $ ^ : > ? at start of input (like JOKey)
        if self.raw_mode {
            return self.handle_normal_letter(key, caps);
        }

        let m = input::get(self.method);

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
        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
        if is_valid(&buffer_keys) {
            self.last_transform = Some(Transform::WAsVowel);

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
    fn try_stroke(&mut self, key: u16) -> Option<Result> {
        // Scan buffer for un-stroked 'd'
        let d_pos = self
            .buf
            .iter()
            .enumerate()
            .find(|(_, c)| c.key == keys::D && !c.stroke)
            .map(|(i, _)| i);

        if let Some(pos) = d_pos {
            // Check revert: if last transform was stroke on same key at same position
            if let Some(Transform::Stroke(last_key)) = self.last_transform {
                if last_key == key {
                    return Some(self.revert_stroke(key, pos));
                }
            }

            // Validate buffer before applying stroke
            // Only validate if buffer has vowels (complete syllable)
            // Allow stroke on initial consonant before vowel is typed (e.g., "dd" → "đ" then "đi")
            let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
            let has_vowel = buffer_keys.iter().any(|&k| keys::is_vowel(k));
            if has_vowel && !is_valid(&buffer_keys) {
                return None;
            }

            // Mark as stroked
            if let Some(c) = self.buf.get_mut(pos) {
                c.stroke = true;
            }

            self.last_transform = Some(Transform::Stroke(key));
            return Some(self.rebuild_from(pos));
        }

        None
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

        // Check revert first (same key pressed twice)
        if let Some(Transform::Tone(last_key, _)) = self.last_transform {
            if last_key == key {
                return Some(self.revert_tone(key, caps));
            }
        }

        // Validate buffer
        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
        if !is_valid(&buffer_keys) {
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
                // Non-horn modifiers: use standard target matching
                for (i, c) in self.buf.iter().enumerate().rev() {
                    if targets.contains(&c.key) && c.tone == tone::NONE {
                        target_positions.push(i);
                        break;
                    }
                }
            }
        }

        if target_positions.is_empty() {
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

        // Normalize ưo → ươ compound if horn was applied to 'u'
        if let Some(compound_pos) = self.normalize_uo_compound() {
            earliest_pos = earliest_pos.min(compound_pos);
        }

        self.last_transform = Some(Transform::Tone(key, tone_val));

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

        // Check if buffer has horn transforms - indicates intentional Vietnamese typing
        // (e.g., "rượu" has base keys [R,U,O,U] which looks like "ou" pattern,
        // but with horns applied it's valid "ươu")
        let has_horn_transforms = self.buf.iter().any(|c| c.tone == tone::HORN);

        // Validate buffer (skip if has horn transforms - already intentional Vietnamese)
        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
        if !has_horn_transforms && !is_valid(&buffer_keys) {
            return None;
        }

        // Skip modifier if buffer shows foreign word patterns.
        // Only check when NO horn transforms exist.
        //
        // Detected patterns:
        // - Invalid vowel combinations (ou, yo) that don't exist in Vietnamese
        // - Consonant clusters after finals common in English (T+R, P+R, C+R)
        //
        // Examples:
        // - "met" + 'r' → T+R cluster common in English → skip modifier
        // - "you" + 'r' → "ou" vowel pattern invalid → skip modifier
        // - "rươu" + 'j' → has horn transforms → DON'T skip, apply mark normally
        if !has_horn_transforms && is_foreign_word_pattern(&buffer_keys, key) {
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
        let pos = Phonology::find_tone_position(&vowels, has_final, true, has_qu, has_gi);

        if let Some(c) = self.buf.get_mut(pos) {
            c.mark = mark_val;
            self.last_transform = Some(Transform::Mark(key, mark_val));
            // Rebuild from the earlier position if compound was formed
            let rebuild_pos = rebuild_from_compound.map_or(pos, |cp| cp.min(pos));
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
        Phonology::find_horn_positions(&buffer_keys, &vowels)
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
            let new_pos = Phonology::find_tone_position(&vowels, has_final, true, has_qu, has_gi);

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
    fn revert_mark(&mut self, key: u16, caps: bool) -> Result {
        self.last_transform = None;

        for pos in self.buf.find_vowels().into_iter().rev() {
            if let Some(c) = self.buf.get_mut(pos) {
                if c.mark > mark::NONE {
                    c.mark = mark::NONE;
                    return self.revert_and_rebuild(pos, key, caps);
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

        self.last_transform = None;
        if keys::is_letter(key) {
            // Add the letter to buffer
            self.buf.push(Char::new(key, caps));

            // Normalize ưo → ươ immediately when 'o' is typed after 'ư'
            // This ensures "dduwo" → "đươ" without waiting for a mark
            // Only in Telex mode (0) - VNI uses explicit '7' for horn
            if self.method == 0 && key == keys::O && self.normalize_uo_compound().is_some() {
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
            let is_valid_triphthong_ending =
                self.has_complete_uo_compound() && (key == keys::U || key == keys::I);
            if self.has_w_as_vowel_transform() && !is_valid_triphthong_ending {
                let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
                if is_foreign_word_pattern(&buffer_keys, key) {
                    return self.revert_w_as_vowel_transforms();
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
    pub fn clear(&mut self) {
        self.buf.clear();
        self.raw_input.clear();
        self.last_transform = None;
        self.raw_mode = false;
        self.has_non_letter_prefix = false;
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
            .filter_map(|&(key, caps)| utils::key_to_char(key, caps))
            .collect();

        if raw_chars.is_empty() {
            return Result::none();
        }

        // Backspace count = current buffer length (displayed chars)
        let backspace = self.buf.len() as u8;

        Result::send(backspace, &raw_chars)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{raw_mode, telex, vni};

    const TELEX_BASIC: &[(&str, &str)] = &[
        ("as", "á"),
        ("af", "à"),
        ("ar", "ả"),
        ("ax", "ã"),
        ("aj", "ạ"),
        ("aa", "â"),
        ("aw", "ă"),
        ("ee", "ê"),
        ("oo", "ô"),
        ("ow", "ơ"),
        ("uw", "ư"),
        ("dd", "đ"),
    ];

    const VNI_BASIC: &[(&str, &str)] = &[
        ("a1", "á"),
        ("a2", "à"),
        ("a3", "ả"),
        ("a4", "ã"),
        ("a5", "ạ"),
        ("a6", "â"),
        ("a8", "ă"),
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

    // Raw mode test cases: typing prefix (@, #, :, /) at start skips Vietnamese transforms
    // Like JOKey's feature: @gox → @gox (NOT @gõ)
    const RAW_MODE_PREFIX: &[(&str, &str)] = &[
        ("@gox", "@gox"),         // @ prefix: "gox" stays raw
        ("@text", "@text"),       // @ prefix: "text" stays raw
        ("#hashtag", "#hashtag"), // # prefix
        (":smile:", ":smile:"),   // : prefix (emoji shortcut)
        ("/command", "/command"), // / prefix (slash command)
    ];

    // Normal mode (without prefix): Vietnamese transforms apply
    const RAW_MODE_NORMAL: &[(&str, &str)] = &[
        ("gox", "gõ"),      // Without prefix: "gox" → "gõ"
        ("text", "tẽt"),    // Without prefix: Vietnamese transforms
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
        telex(TELEX_ESC_RESTORE);
    }

    #[test]
    fn test_vni_esc_restore() {
        vni(VNI_ESC_RESTORE);
    }

    #[test]
    fn test_raw_mode_prefix() {
        raw_mode(RAW_MODE_PREFIX);
    }

    #[test]
    fn test_raw_mode_normal() {
        // Without prefix, Vietnamese transforms should still apply
        telex(RAW_MODE_NORMAL);
    }
}
