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
    vowel::{Modifier, Phonology, Vowel},
};
use crate::input::{self, ToneType};
use buffer::{Buffer, Char, MAX};
use shortcut::ShortcutTable;
use validation::is_valid;

/// Convert key code to character
fn key_to_char(key: u16, caps: bool) -> Option<char> {
    let ch = match key {
        keys::A => 'a',
        keys::B => 'b',
        keys::C => 'c',
        keys::D => 'd',
        keys::E => 'e',
        keys::F => 'f',
        keys::G => 'g',
        keys::H => 'h',
        keys::I => 'i',
        keys::J => 'j',
        keys::K => 'k',
        keys::L => 'l',
        keys::M => 'm',
        keys::N => 'n',
        keys::O => 'o',
        keys::P => 'p',
        keys::Q => 'q',
        keys::R => 'r',
        keys::S => 's',
        keys::T => 't',
        keys::U => 'u',
        keys::V => 'v',
        keys::W => 'w',
        keys::X => 'x',
        keys::Y => 'y',
        keys::Z => 'z',
        keys::N0 => return Some('0'),
        keys::N1 => return Some('1'),
        keys::N2 => return Some('2'),
        keys::N3 => return Some('3'),
        keys::N4 => return Some('4'),
        keys::N5 => return Some('5'),
        keys::N6 => return Some('6'),
        keys::N7 => return Some('7'),
        keys::N8 => return Some('8'),
        keys::N9 => return Some('9'),
        _ => return None,
    };
    Some(if caps { ch.to_ascii_uppercase() } else { ch })
}

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
            shortcuts: ShortcutTable::new(),
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

    pub fn shortcuts_mut(&mut self) -> &mut ShortcutTable {
        &mut self.shortcuts
    }

    /// Handle key event - main entry point
    pub fn on_key(&mut self, key: u16, caps: bool, ctrl: bool) -> Result {
        if !self.enabled || ctrl {
            self.buf.clear();
            return Result::none();
        }

        if keys::is_break(key) {
            self.buf.clear();
            return Result::none();
        }

        if key == keys::DELETE {
            self.buf.pop();
            return Result::none();
        }

        self.process(key, caps)
    }

    /// Main processing pipeline - pattern-based
    fn process(&mut self, key: u16, caps: bool) -> Result {
        let m = input::get(self.method);

        // Check modifiers by scanning buffer for patterns

        // 1. Stroke modifier (d → đ)
        if m.stroke(key) {
            if let Some(result) = self.try_stroke(key) {
                return result;
            }
        }

        // 2. Tone modifier (circumflex, horn, breve)
        if let Some(tone_type) = m.tone(key) {
            let targets = m.tone_targets(key);
            if let Some(result) = self.try_tone(key, caps, tone_type, targets) {
                return result;
            }
        }

        // 3. Mark modifier
        if let Some(mark_val) = m.mark(key) {
            if let Some(result) = self.try_mark(key, caps, mark_val) {
                return result;
            }
        }

        // 4. Remove modifier
        if m.remove(key) {
            self.last_transform = None;
            return self.handle_remove();
        }

        // Not a modifier - normal letter
        self.handle_normal_letter(key, caps)
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
            for (i, c) in self.buf.iter().enumerate().rev() {
                if targets.contains(&c.key) && c.tone == tone::NONE {
                    target_positions.push(i);
                    break;
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
            .filter_map(|c| key_to_char(c.key, c.caps))
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
        self.buf
            .iter()
            .enumerate()
            .filter(|(_, c)| keys::is_vowel(c.key))
            .map(|(pos, c)| {
                let modifier = match c.tone {
                    tone::CIRCUMFLEX => Modifier::Circumflex,
                    tone::HORN => Modifier::Horn,
                    _ => Modifier::None,
                };
                Vowel::new(c.key, modifier, pos)
            })
            .collect()
    }

    /// Check for final consonant after position
    fn has_final_consonant(&self, after_pos: usize) -> bool {
        (after_pos + 1..self.buf.len()).any(|i| {
            self.buf
                .get(i)
                .map(|c| keys::is_consonant(c.key))
                .unwrap_or(false)
        })
    }

    /// Check for qu initial
    fn has_qu_initial(&self) -> bool {
        for (i, c) in self.buf.iter().enumerate() {
            if c.key == keys::U && i > 0 {
                if let Some(prev) = self.buf.get(i - 1) {
                    return prev.key == keys::Q;
                }
            }
        }
        false
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
                } else if let Some(ch) = key_to_char(c.key, c.caps) {
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
    use crate::test_utils::{telex, vni};

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
