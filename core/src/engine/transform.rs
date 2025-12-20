//! Vietnamese Transformation
//!
//! Pattern-based transformation for Vietnamese diacritics.
//! Scans entire buffer instead of case-by-case processing.

use super::buffer::Buffer;
use crate::data::{
    chars::{mark, tone},
    keys,
    vowel::Phonology,
};
use crate::utils;

/// Modifier type detected from key
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModifierType {
    /// Tone diacritic: circumflex (^), horn, breve
    Tone(u8),
    /// Tone mark: sắc, huyền, hỏi, ngã, nặng
    Mark(u8),
    /// Stroke: d → đ
    Stroke,
    /// Remove last diacritic
    Remove,
}

/// Transformation result
#[derive(Debug)]
pub struct TransformResult {
    /// Positions that were modified
    pub modified_positions: Vec<usize>,
    /// Whether transformation was applied
    pub applied: bool,
}

impl TransformResult {
    pub fn none() -> Self {
        Self {
            modified_positions: vec![],
            applied: false,
        }
    }

    pub fn success(positions: Vec<usize>) -> Self {
        Self {
            modified_positions: positions,
            applied: true,
        }
    }

    pub fn earliest_position(&self) -> Option<usize> {
        self.modified_positions.iter().copied().min()
    }
}

/// Apply tone diacritic transformation (^, ơ, ư, ă)
///
/// Pattern-based: scans buffer for matching vowels
pub fn apply_tone(buf: &mut Buffer, key: u16, tone_value: u8, method: u8) -> TransformResult {
    let buffer_keys: Vec<u16> = buf.iter().map(|c| c.key).collect();

    // Find target vowels based on key and method
    let targets = find_tone_targets(&buffer_keys, key, tone_value, method);

    if targets.is_empty() {
        return TransformResult::none();
    }

    // Apply tone to targets
    let mut positions = vec![];
    for pos in &targets {
        if let Some(c) = buf.get_mut(*pos) {
            if c.tone == tone::NONE {
                c.tone = tone_value;
                positions.push(*pos);
            }
        }
    }

    if positions.is_empty() {
        TransformResult::none()
    } else {
        // After adding tone, reposition mark if needed
        reposition_mark_if_needed(buf);
        TransformResult::success(positions)
    }
}

/// Find which vowel positions should receive the tone modifier
fn find_tone_targets(buffer_keys: &[u16], key: u16, tone_value: u8, method: u8) -> Vec<usize> {
    let mut targets = vec![];

    // Find all vowel positions
    let vowel_positions: Vec<usize> = buffer_keys
        .iter()
        .enumerate()
        .filter(|(_, &k)| keys::is_vowel(k))
        .map(|(i, _)| i)
        .collect();

    if vowel_positions.is_empty() {
        return targets;
    }

    // Telex patterns
    if method == 0 {
        // aa, ee, oo → circumflex (immediate doubling only)
        // The target vowel must be at the LAST position in the buffer
        // This ensures "ee" doubling only works for consecutive presses,
        // not for words like "teacher" where 'e' appears twice non-adjacently
        if tone_value == tone::CIRCUMFLEX && matches!(key, keys::A | keys::E | keys::O) {
            // Find matching vowel (same key) - must be at last position
            let last_pos = buffer_keys.len().saturating_sub(1);
            for &pos in vowel_positions.iter().rev() {
                if buffer_keys[pos] == key && pos == last_pos {
                    targets.push(pos);
                    break;
                }
            }
        }
        // w → horn/breve
        else if tone_value == tone::HORN && key == keys::W {
            targets = Phonology::find_horn_positions(buffer_keys, &vowel_positions);
        }
    }
    // VNI patterns
    else {
        // 6 → circumflex for a, e, o
        if tone_value == tone::CIRCUMFLEX && key == keys::N6 {
            for &pos in vowel_positions.iter().rev() {
                if matches!(buffer_keys[pos], keys::A | keys::E | keys::O) {
                    targets.push(pos);
                    break;
                }
            }
        }
        // 7 → horn for o, u
        else if tone_value == tone::HORN && key == keys::N7 {
            targets = Phonology::find_horn_positions(buffer_keys, &vowel_positions);
        }
        // 8 → breve for a only
        else if tone_value == tone::HORN && key == keys::N8 {
            for &pos in vowel_positions.iter().rev() {
                if buffer_keys[pos] == keys::A {
                    targets.push(pos);
                    break;
                }
            }
        }
    }

    targets
}

/// Apply mark transformation (sắc, huyền, hỏi, ngã, nặng)
pub fn apply_mark(buf: &mut Buffer, mark_value: u8, modern: bool) -> TransformResult {
    let vowels = utils::collect_vowels(buf);
    if vowels.is_empty() {
        return TransformResult::none();
    }

    // Find position using phonology rules
    let last_vowel_pos = vowels.last().map(|v| v.pos).unwrap_or(0);
    let has_final = utils::has_final_consonant(buf, last_vowel_pos);
    let has_qu = utils::has_qu_initial(buf);
    let has_gi = utils::has_gi_initial(buf);
    let pos = Phonology::find_tone_position(&vowels, has_final, modern, has_qu, has_gi);

    // Clear any existing mark first
    for v in &vowels {
        if let Some(c) = buf.get_mut(v.pos) {
            c.mark = mark::NONE;
        }
    }

    // Apply new mark
    if let Some(c) = buf.get_mut(pos) {
        c.mark = mark_value;
        return TransformResult::success(vec![pos]);
    }

    TransformResult::none()
}

/// Apply stroke transformation (d → đ)
///
/// Scans buffer for 'd' at any position
pub fn apply_stroke(buf: &mut Buffer) -> TransformResult {
    // Find first 'd' that hasn't been stroked
    for i in 0..buf.len() {
        if let Some(c) = buf.get_mut(i) {
            if c.key == keys::D && !c.stroke {
                c.stroke = true;
                return TransformResult::success(vec![i]);
            }
        }
    }
    TransformResult::none()
}

/// Remove last diacritic (mark first, then tone)
pub fn apply_remove(buf: &mut Buffer) -> TransformResult {
    let vowel_positions = buf.find_vowels();

    // Try to remove mark first
    for pos in vowel_positions.iter().rev() {
        if let Some(c) = buf.get_mut(*pos) {
            if c.mark > mark::NONE {
                c.mark = mark::NONE;
                return TransformResult::success(vec![*pos]);
            }
        }
    }

    // Then try to remove tone
    for pos in vowel_positions.iter().rev() {
        if let Some(c) = buf.get_mut(*pos) {
            if c.tone > tone::NONE {
                c.tone = tone::NONE;
                return TransformResult::success(vec![*pos]);
            }
        }
    }

    TransformResult::none()
}

/// Revert tone transformation
pub fn revert_tone(buf: &mut Buffer, target_key: u16) -> TransformResult {
    let vowel_positions = buf.find_vowels();

    for pos in vowel_positions.iter().rev() {
        if let Some(c) = buf.get_mut(*pos) {
            if c.key == target_key && c.tone > tone::NONE {
                c.tone = tone::NONE;
                return TransformResult::success(vec![*pos]);
            }
        }
    }

    TransformResult::none()
}

/// Revert mark transformation
pub fn revert_mark(buf: &mut Buffer) -> TransformResult {
    let vowel_positions = buf.find_vowels();

    for pos in vowel_positions.iter().rev() {
        if let Some(c) = buf.get_mut(*pos) {
            if c.mark > mark::NONE {
                c.mark = mark::NONE;
                return TransformResult::success(vec![*pos]);
            }
        }
    }

    TransformResult::none()
}

/// Revert stroke transformation
pub fn revert_stroke(buf: &mut Buffer) -> TransformResult {
    // Find stroked 'd' and un-stroke it
    for i in 0..buf.len() {
        if let Some(c) = buf.get_mut(i) {
            if c.key == keys::D && c.stroke {
                c.stroke = false;
                return TransformResult::success(vec![i]);
            }
        }
    }
    TransformResult::none()
}

/// Reposition mark after tone change if needed
fn reposition_mark_if_needed(buf: &mut Buffer) {
    // Find current mark
    let mark_info: Option<(usize, u8)> = buf
        .iter()
        .enumerate()
        .find(|(_, c)| c.mark > 0)
        .map(|(i, c)| (i, c.mark));

    if let Some((old_pos, mark_value)) = mark_info {
        let vowels = utils::collect_vowels(buf);
        if vowels.is_empty() {
            return;
        }

        let last_vowel_pos = vowels.last().map(|v| v.pos).unwrap_or(0);
        let has_final = utils::has_final_consonant(buf, last_vowel_pos);
        let has_qu = utils::has_qu_initial(buf);
        let has_gi = utils::has_gi_initial(buf);
        let new_pos = Phonology::find_tone_position(&vowels, has_final, true, has_qu, has_gi);

        if new_pos != old_pos {
            // Clear old mark
            if let Some(c) = buf.get_mut(old_pos) {
                c.mark = 0;
            }
            // Set new mark
            if let Some(c) = buf.get_mut(new_pos) {
                c.mark = mark_value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::buffer::Char;
    use super::*;

    fn setup_buffer(s: &str) -> Buffer {
        let mut buf = Buffer::new();
        for ch in s.chars() {
            let key = match ch.to_ascii_lowercase() {
                'a' => keys::A,
                'b' => keys::B,
                'c' => keys::C,
                'd' => keys::D,
                'e' => keys::E,
                'g' => keys::G,
                'h' => keys::H,
                'i' => keys::I,
                'n' => keys::N,
                'o' => keys::O,
                'u' => keys::U,
                _ => continue,
            };
            buf.push(Char::new(key, ch.is_uppercase()));
        }
        buf
    }

    #[test]
    fn test_apply_stroke() {
        let mut buf = setup_buffer("do");
        let result = apply_stroke(&mut buf);
        assert!(result.applied);
        assert!(buf.get(0).unwrap().stroke);
    }

    #[test]
    fn test_apply_stroke_anywhere() {
        // "dod" should stroke the first 'd'
        let mut buf = setup_buffer("dod");
        let result = apply_stroke(&mut buf);
        assert!(result.applied);
        assert!(buf.get(0).unwrap().stroke); // First d is stroked
    }

    #[test]
    fn test_apply_mark() {
        let mut buf = setup_buffer("an");
        let result = apply_mark(&mut buf, mark::SAC, true);
        assert!(result.applied);
        assert_eq!(buf.get(0).unwrap().mark, mark::SAC);
    }

    #[test]
    fn test_uo_compound() {
        let mut buf = setup_buffer("duoc");
        let result = apply_tone(&mut buf, keys::W, tone::HORN, 0);
        assert!(result.applied);
        // Both u and o should have horn
        assert_eq!(buf.get(1).unwrap().tone, tone::HORN); // u
        assert_eq!(buf.get(2).unwrap().tone, tone::HORN); // o
    }
}
