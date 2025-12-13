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
use validation::is_valid;

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

    /// Handle key event with extended parameters
    ///
    /// # Arguments
    /// * `key` - macOS virtual keycode
    /// * `caps` - true if Caps Lock is active (for uppercase letters)
    /// * `ctrl` - true if Cmd/Ctrl/Alt is pressed (bypasses IME)
    /// * `shift` - true if Shift key is pressed (for symbols like @, #, $)
    pub fn on_key_ext(&mut self, key: u16, caps: bool, ctrl: bool, shift: bool) -> Result {
        if !self.enabled || ctrl {
            self.buf.clear();
            self.last_transform = None;
            return Result::none();
        }

        // Check for word boundary shortcuts ONLY on SPACE
        if key == keys::SPACE {
            let result = self.try_word_boundary_shortcut();
            self.buf.clear();
            self.last_transform = None;
            return result;
        }

        // Other break keys (punctuation, arrows, etc.) just clear buffer
        if keys::is_break(key) {
            self.buf.clear();
            self.last_transform = None;
            return Result::none();
        }

        if key == keys::DELETE {
            self.buf.pop();
            self.last_transform = None;
            return Result::none();
        }

        self.process(key, caps, shift)
    }

    /// Main processing pipeline - pattern-based
    fn process(&mut self, key: u16, caps: bool, shift: bool) -> Result {
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
        if !skip_vni_modifiers && m.remove(key) {
            self.last_transform = None;
            return self.handle_remove();
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

        let buffer_str = self.buf.to_string_preserve_case();
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
        // If shortcut was previously skipped, don't try again
        if matches!(self.last_transform, Some(Transform::WShortcutSkipped)) {
            return None;
        }

        // Check revert: ww → w (skip shortcut)
        if let Some(Transform::WAsVowel) = self.last_transform {
            self.last_transform = Some(Transform::WShortcutSkipped);
            // Fix buffer: remove the U(horn) that was added, replace with W
            self.buf.pop();
            self.buf.push(Char::new(keys::W, caps));
            // Revert: backspace "ư", output single "w"
            let w = if caps { 'W' } else { 'w' };
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

        // Check revert first
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

        // Scan buffer for eligible target vowels (without existing tone)
        let mut target_positions = Vec::new();

        // Special case: uo compound for horn
        if tone_type == ToneType::Horn && self.has_uo_compound() {
            for (i, c) in self.buf.iter().enumerate() {
                if (c.key == keys::U || c.key == keys::O) && c.tone == tone::NONE {
                    target_positions.push(i);
                }
            }
        }

        // Normal case: find last matching target
        if target_positions.is_empty() {
            // For horn modifier, apply smart vowel selection based on Vietnamese phonology
            if tone_type == ToneType::Horn {
                target_positions = self.find_horn_target(targets);
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

        // Apply tone
        let mut earliest_pos = usize::MAX;
        for &pos in &target_positions {
            if let Some(c) = self.buf.get_mut(pos) {
                c.tone = tone_val;
                earliest_pos = earliest_pos.min(pos);
            }
        }

        self.last_transform = Some(Transform::Tone(key, tone_val));

        // Reposition mark if needed
        let mark_moved_from = self.reposition_mark_if_needed();
        let mut rebuild_pos = earliest_pos;
        if let Some(old_pos) = mark_moved_from {
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

        // Validate buffer
        let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();
        if !is_valid(&buffer_keys) {
            return None;
        }

        let vowels = self.collect_vowels();
        if vowels.is_empty() {
            return None;
        }

        // Find mark position using phonology rules
        let last_vowel_pos = vowels.last().map(|v| v.pos).unwrap_or(0);
        let has_final = self.has_final_consonant(last_vowel_pos);
        let has_qu = self.has_qu_initial();
        let pos = Phonology::find_tone_position(&vowels, has_final, true, has_qu);

        if let Some(c) = self.buf.get_mut(pos) {
            c.mark = mark_val;
            self.last_transform = Some(Transform::Mark(key, mark_val));
            return Some(self.rebuild_from(pos));
        }

        None
    }

    /// Check for uo compound in buffer
    fn has_uo_compound(&self) -> bool {
        let mut prev_key: Option<u16> = None;
        for c in self.buf.iter() {
            if keys::is_vowel(c.key) {
                if let Some(pk) = prev_key {
                    if (pk == keys::U && c.key == keys::O) || (pk == keys::O && c.key == keys::U) {
                        return true;
                    }
                }
                prev_key = Some(c.key);
            } else {
                prev_key = None;
            }
        }
        false
    }

    /// Find target position for horn modifier (w key in Telex, 7/8 in VNI)
    ///
    /// Rules based on Vietnamese phonology:
    /// - 'oa' medial pair: apply breve to 'a' (main vowel) - only if 'a' is in targets
    /// - 'ua' with consonant before (mua, chua): apply horn to 'u'
    /// - 'ua' without consonant or with 'q': apply breve to 'a' - only if 'a' is in targets
    /// - Otherwise: apply to last matching target vowel, prioritizing u/o over a
    fn find_horn_target(&self, targets: &[u16]) -> Vec<usize> {
        let mut result = Vec::new();

        // Find vowel positions that match targets
        let vowels: Vec<(usize, u16)> = self
            .buf
            .iter()
            .enumerate()
            .filter(|(_, c)| targets.contains(&c.key) && c.tone == tone::NONE)
            .map(|(i, c)| (i, c.key))
            .collect();

        if vowels.is_empty() {
            return result;
        }

        // Only apply special pair logic if 'a' is in targets (Telex mode)
        // VNI mode uses 7 for horn (targets=[O,U]) and 8 for breve (targets=[A])
        let has_a_target = targets.contains(&keys::A);

        if has_a_target {
            // Check for adjacent vowel pairs in the full buffer
            for i in 0..self.buf.len().saturating_sub(1) {
                let c1 = self.buf.get(i);
                let c2 = self.buf.get(i + 1);

                if let (Some(ch1), Some(ch2)) = (c1, c2) {
                    if !keys::is_vowel(ch1.key) || !keys::is_vowel(ch2.key) {
                        continue;
                    }

                    // 'oa' medial pair: apply breve to 'a'
                    if ch1.key == keys::O && ch2.key == keys::A && ch2.tone == tone::NONE {
                        result.push(i + 1);
                        return result;
                    }

                    // 'ua' pair: check for preceding consonant
                    if ch1.key == keys::U && ch2.key == keys::A {
                        let preceding = if i > 0 {
                            self.buf.get(i - 1).map(|c| c.key)
                        } else {
                            None
                        };

                        // If preceded by consonant (except 'q'), 'u' is main vowel
                        let has_consonant_not_q = preceding
                            .map(|k| keys::is_consonant(k) && k != keys::Q)
                            .unwrap_or(false);

                        if has_consonant_not_q && ch1.tone == tone::NONE {
                            // 'Cua' pattern (mua, chua): apply horn to 'u'
                            result.push(i);
                        } else if ch2.tone == tone::NONE {
                            // 'qua' or standalone 'ua': apply breve to 'a'
                            result.push(i + 1);
                        }
                        return result;
                    }
                }
            }
        }

        // No special pair found, use standard priority: u/o first, then a
        for &(pos, k) in vowels.iter().rev() {
            if k == keys::U || k == keys::O {
                result.push(pos);
                return result;
            }
        }

        // No u/o found, apply to last matching target (could be 'a' in Telex)
        if let Some(&(pos, _)) = vowels.last() {
            result.push(pos);
        }

        result
    }

    /// Reposition mark after tone change
    fn reposition_mark_if_needed(&mut self) -> Option<usize> {
        let mark_info: Option<(usize, u8)> = self
            .buf
            .iter()
            .enumerate()
            .find(|(_, c)| c.mark > 0)
            .map(|(i, c)| (i, c.mark));

        if let Some((old_pos, mark_value)) = mark_info {
            let vowels = self.collect_vowels();
            if vowels.is_empty() {
                return None;
            }

            let last_vowel_pos = vowels.last().map(|v| v.pos).unwrap_or(0);
            let has_final = self.has_final_consonant(last_vowel_pos);
            let has_qu = self.has_qu_initial();
            let new_pos = Phonology::find_tone_position(&vowels, has_final, true, has_qu);

            if new_pos != old_pos {
                if let Some(c) = self.buf.get_mut(old_pos) {
                    c.mark = 0;
                }
                if let Some(c) = self.buf.get_mut(new_pos) {
                    c.mark = mark_value;
                }
                return Some(old_pos);
            }
        }
        None
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

    /// Handle remove modifier
    fn handle_remove(&mut self) -> Result {
        for pos in self.buf.find_vowels().into_iter().rev() {
            if let Some(c) = self.buf.get_mut(pos) {
                if c.mark > mark::NONE {
                    c.mark = mark::NONE;
                    return self.rebuild_from(pos);
                }
                if c.tone > tone::NONE {
                    c.tone = tone::NONE;
                    return self.rebuild_from(pos);
                }
            }
        }
        Result::none()
    }

    /// Handle normal letter input
    fn handle_normal_letter(&mut self, key: u16, caps: bool) -> Result {
        self.last_transform = None;
        if keys::is_letter(key) {
            self.buf.push(Char::new(key, caps));
        } else {
            self.buf.clear();
        }
        Result::none()
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

    /// Clear buffer
    pub fn clear(&mut self) {
        self.buf.clear();
        self.last_transform = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{telex, vni};

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
}
