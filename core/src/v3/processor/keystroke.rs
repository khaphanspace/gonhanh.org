//! Single keystroke processing
//!
//! Implements the matrix-based dispatch for each keystroke.
//! All decisions are table lookups - zero conditionals in hot path.

use super::buffer::{DualBuffer, XformType};
use super::result::ProcessResult;
use crate::v3::constants::{
    dispatch::{dispatch, Action, State},
    key_category::get_category,
    placement::{
        apply_breve, apply_circumflex, apply_horn, apply_stroke, apply_tone, find_tone_position,
        is_vowel, key_to_tone_telex, mark, remove_breve, remove_circumflex, remove_horn,
        remove_stroke, remove_tone, tone,
    },
};

/// Process a single keystroke
///
/// # Arguments
/// * `state` - Current state machine state
/// * `key` - The key pressed
/// * `foreign_mode` - Whether foreign mode is active
///
/// # Returns
/// * Action to perform and next state
pub fn process_keystroke(state: State, key: char, foreign_mode: bool) -> (Action, State) {
    // In foreign mode, all keys pass through (no transforms)
    if foreign_mode {
        let cat = get_category(key);
        // Only boundary exits foreign mode
        if matches!(
            cat,
            crate::v3::constants::key_category::KeyCategory::Boundary
        ) {
            return (Action::Commit, State::Empty);
        }
        return (Action::Pass, state);
    }

    // Get key category and dispatch
    let category = get_category(key);
    dispatch(state, category)
}

/// Action handler context
pub struct ActionContext<'a> {
    pub buffer: &'a mut DualBuffer,
    pub key: char,
    pub method: u8, // 0=Telex, 1=VNI
}

/// Execute action with buffer context
pub fn execute_action_with_context(action: Action, ctx: &mut ActionContext) -> ProcessResult {
    match action {
        Action::Pass => handle_pass(ctx),
        Action::AddInitial => handle_add_initial(ctx),
        Action::AddVowel => handle_add_vowel(ctx),
        Action::ApplyTone => handle_tone(ctx),
        Action::ApplyCircumflex => handle_circumflex(ctx),
        Action::ApplyHorn => handle_horn(ctx),
        Action::ApplyBreve => handle_breve(ctx),
        Action::ApplyStroke => handle_stroke(ctx),
        Action::AddFinal => handle_add_final(ctx),
        Action::Revert => handle_revert(ctx),
        Action::Commit => handle_commit(ctx),
        Action::Defer => handle_defer(ctx),
        Action::Invalid => handle_pass(ctx),
    }
}

/// Simple execute action (for backward compatibility)
pub fn execute_action(action: Action, key: char) -> ProcessResult {
    match action {
        Action::Pass => ProcessResult::Pass(key),
        Action::AddInitial | Action::AddVowel | Action::AddFinal => ProcessResult::Transform {
            backspaces: 0,
            output: key.to_string(),
        },
        Action::Commit => ProcessResult::Commit,
        _ => ProcessResult::Pass(key),
    }
}

// ============================================
// Action Handlers
// ============================================

/// Handle PASS - key passes through unchanged
fn handle_pass(ctx: &mut ActionContext) -> ProcessResult {
    ctx.buffer.push_raw(ctx.key);
    ctx.buffer.push_transformed(ctx.key);
    ProcessResult::Pass(ctx.key)
}

/// Handle ADD_INITIAL - add initial consonant
fn handle_add_initial(ctx: &mut ActionContext) -> ProcessResult {
    ctx.buffer.push_raw(ctx.key);
    ctx.buffer.push_transformed(ctx.key);
    ProcessResult::Transform {
        backspaces: 0,
        output: ctx.key.to_string(),
    }
}

/// Handle ADD_VOWEL - add vowel to buffer
fn handle_add_vowel(ctx: &mut ActionContext) -> ProcessResult {
    ctx.buffer.push_raw(ctx.key);
    ctx.buffer.push_transformed(ctx.key);
    ProcessResult::Transform {
        backspaces: 0,
        output: ctx.key.to_string(),
    }
}

/// Handle ADD_FINAL - add final consonant
fn handle_add_final(ctx: &mut ActionContext) -> ProcessResult {
    ctx.buffer.push_raw(ctx.key);
    ctx.buffer.push_transformed(ctx.key);
    ProcessResult::Transform {
        backspaces: 0,
        output: ctx.key.to_string(),
    }
}

/// Handle TONE - apply tone mark to vowel
fn handle_tone(ctx: &mut ActionContext) -> ProcessResult {
    let tone_value = key_to_tone_telex(ctx.key);
    if tone_value == tone::NONE {
        return handle_pass(ctx);
    }

    // Check for double-key revert
    if ctx.buffer.track.should_revert(ctx.key) && ctx.buffer.track.xform_type == XformType::Tone {
        return handle_revert(ctx);
    }

    // Find vowel to apply tone
    let transformed = ctx.buffer.transformed();
    let vowels: Vec<(usize, char)> = transformed
        .char_indices()
        .filter(|(_, c)| is_vowel(*c))
        .collect();

    if vowels.is_empty() {
        return handle_pass(ctx);
    }

    // Find tone position using placement rules
    let vowel_chars: Vec<char> = vowels.iter().map(|(_, c)| *c).collect();
    let tone_idx = find_tone_position(&vowel_chars);
    let (char_pos, vowel_char) = vowels[tone_idx];

    // Remove existing tone first, then apply new tone
    let base_char = remove_tone(vowel_char);
    let new_char = apply_tone(base_char, tone_value);

    // Build new transformed string
    let mut new_transformed = String::new();
    for (i, c) in transformed.char_indices() {
        if i == char_pos {
            new_transformed.push(new_char);
        } else {
            new_transformed.push(c);
        }
    }

    // Update buffer
    ctx.buffer.push_raw(ctx.key);
    if let Some(raw_key) = ctx.buffer.raw_buffer_mut().last_mut() {
        raw_key.consume();
    }
    ctx.buffer.set_transformed(&new_transformed);
    ctx.buffer.has_transform = true;
    ctx.buffer.track.record_tone(ctx.key, char_pos, tone_value);

    // Calculate backspaces (from changed position to end)
    let chars_after = transformed.chars().count() - char_pos;
    let new_suffix: String = new_transformed.chars().skip(char_pos).collect();

    ProcessResult::Transform {
        backspaces: chars_after as u8,
        output: new_suffix,
    }
}

/// Handle CIRCUMFLEX - apply â, ê, ô
fn handle_circumflex(ctx: &mut ActionContext) -> ProcessResult {
    // Check for double-key revert
    if ctx.buffer.track.should_revert(ctx.key) && ctx.buffer.track.xform_type == XformType::Mark {
        return handle_revert(ctx);
    }

    // Find vowel to apply circumflex (a, e, o)
    let transformed = ctx.buffer.transformed();
    let target_vowel = ctx.key.to_ascii_lowercase();

    // Find matching vowel from end (most recent)
    let mut found_pos = None;
    let mut found_char = '\0';
    for (i, c) in transformed.char_indices().rev() {
        let lower = c.to_ascii_lowercase();
        if lower == target_vowel {
            found_pos = Some(i);
            found_char = c;
            break;
        }
    }

    let char_pos = match found_pos {
        Some(pos) => pos,
        None => return handle_pass(ctx),
    };

    // Apply circumflex
    let new_char = apply_circumflex(found_char);
    if new_char == found_char {
        return handle_pass(ctx);
    }

    // Build new transformed string
    let mut new_transformed = String::new();
    for (i, c) in transformed.char_indices() {
        if i == char_pos {
            new_transformed.push(new_char);
        } else {
            new_transformed.push(c);
        }
    }

    // Update buffer
    ctx.buffer.push_raw(ctx.key);
    if let Some(raw_key) = ctx.buffer.raw_buffer_mut().last_mut() {
        raw_key.consume();
    }
    ctx.buffer.set_transformed(&new_transformed);
    ctx.buffer.has_transform = true;
    ctx.buffer
        .track
        .record_mark(ctx.key, char_pos, mark::CIRCUM);

    let chars_after = transformed.chars().count() - char_pos;
    let new_suffix: String = new_transformed.chars().skip(char_pos).collect();

    ProcessResult::Transform {
        backspaces: chars_after as u8,
        output: new_suffix,
    }
}

/// Handle HORN - apply ơ, ư (or delegate to breve for ă)
///
/// 'w' is overloaded in Telex:
/// - After 'a' → breve (ă)
/// - After 'o' → horn (ơ)
/// - After 'u' → horn (ư)
/// - After 'uo' cluster → horn on BOTH (ươ) - Vietnamese dipthong pattern
fn handle_horn(ctx: &mut ActionContext) -> ProcessResult {
    // Check for double-key revert
    if ctx.buffer.track.should_revert(ctx.key) && ctx.buffer.track.xform_type == XformType::Mark {
        return handle_revert(ctx);
    }

    let transformed = ctx.buffer.transformed();
    let chars: Vec<char> = transformed.chars().collect();

    // Find last vowel to decide: horn or breve?
    if let Some(&last_char) = chars.last() {
        let lower = last_char.to_ascii_lowercase();
        // If last char is 'a', delegate to breve
        if lower == 'a' {
            return handle_breve(ctx);
        }
    }

    // Check for "uo" cluster pattern (Vietnamese diphthong: uo → ươ)
    // Pattern: ...u...o... where both are adjacent vowels
    let vowel_indices: Vec<(usize, usize, char)> = chars
        .iter()
        .enumerate()
        .filter(|(_, c)| is_vowel(**c))
        .map(|(byte_idx, c)| {
            // Calculate byte position for char_indices
            let byte_pos = transformed
                .char_indices()
                .find(|(_, ch)| std::ptr::eq(ch, c) || *ch == *c)
                .map(|(i, _)| i)
                .unwrap_or(byte_idx);
            (byte_idx, byte_pos, *c)
        })
        .collect();

    // Find consecutive u-o pattern (applies horn to both → ươ)
    let mut u_pos = None;
    let mut o_pos = None;
    for i in 0..vowel_indices.len().saturating_sub(1) {
        let (_, u_byte, v1) = vowel_indices[i];
        let (_, o_byte, v2) = vowel_indices[i + 1];
        let v1_lower = v1.to_ascii_lowercase();
        let v2_lower = v2.to_ascii_lowercase();
        if v1_lower == 'u' && v2_lower == 'o' {
            u_pos = Some((u_byte, v1));
            o_pos = Some((o_byte, v2));
            break;
        }
    }

    // If "uo" pattern found, apply horn to both
    if let (Some((u_byte_pos, u_char)), Some((o_byte_pos, o_char))) = (u_pos, o_pos) {
        let new_u = apply_horn(u_char);
        let new_o = apply_horn(o_char);

        // Build new transformed string with both horns applied
        let mut new_transformed = String::new();
        for (i, c) in transformed.char_indices() {
            if i == u_byte_pos {
                new_transformed.push(new_u);
            } else if i == o_byte_pos {
                new_transformed.push(new_o);
            } else {
                new_transformed.push(c);
            }
        }

        // Update buffer
        ctx.buffer.push_raw(ctx.key);
        if let Some(raw_key) = ctx.buffer.raw_buffer_mut().last_mut() {
            raw_key.consume();
        }
        ctx.buffer.set_transformed(&new_transformed);
        ctx.buffer.has_transform = true;
        ctx.buffer
            .track
            .record_mark(ctx.key, o_byte_pos, mark::HORN);

        let chars_after =
            transformed.chars().count() - chars.iter().position(|&c| c == u_char).unwrap_or(0);
        let u_char_idx = chars.iter().position(|&c| c == u_char).unwrap_or(0);
        let new_suffix: String = new_transformed.chars().skip(u_char_idx).collect();

        return ProcessResult::Transform {
            backspaces: chars_after as u8,
            output: new_suffix,
        };
    }

    // Single vowel horn: find o or u from end
    let mut found_pos = None;
    let mut found_char = '\0';
    for (i, c) in transformed.char_indices().rev() {
        let lower = c.to_ascii_lowercase();
        if lower == 'o' || lower == 'u' {
            found_pos = Some(i);
            found_char = c;
            break;
        }
    }

    let char_pos = match found_pos {
        Some(pos) => pos,
        None => return handle_pass(ctx),
    };

    // Apply horn
    let new_char = apply_horn(found_char);
    if new_char == found_char {
        return handle_pass(ctx);
    }

    // Build new transformed string
    let mut new_transformed = String::new();
    for (i, c) in transformed.char_indices() {
        if i == char_pos {
            new_transformed.push(new_char);
        } else {
            new_transformed.push(c);
        }
    }

    // Update buffer
    ctx.buffer.push_raw(ctx.key);
    if let Some(raw_key) = ctx.buffer.raw_buffer_mut().last_mut() {
        raw_key.consume();
    }
    ctx.buffer.set_transformed(&new_transformed);
    ctx.buffer.has_transform = true;
    ctx.buffer.track.record_mark(ctx.key, char_pos, mark::HORN);

    let chars_after = transformed.chars().count() - char_pos;
    let new_suffix: String = new_transformed.chars().skip(char_pos).collect();

    ProcessResult::Transform {
        backspaces: chars_after as u8,
        output: new_suffix,
    }
}

/// Handle BREVE - apply ă
fn handle_breve(ctx: &mut ActionContext) -> ProcessResult {
    // Check for double-key revert
    if ctx.buffer.track.should_revert(ctx.key) && ctx.buffer.track.xform_type == XformType::Mark {
        return handle_revert(ctx);
    }

    // Find 'a' to apply breve
    let transformed = ctx.buffer.transformed();

    let mut found_pos = None;
    let mut found_char = '\0';
    for (i, c) in transformed.char_indices().rev() {
        let lower = c.to_ascii_lowercase();
        if lower == 'a' {
            found_pos = Some(i);
            found_char = c;
            break;
        }
    }

    let char_pos = match found_pos {
        Some(pos) => pos,
        None => return handle_pass(ctx),
    };

    // Apply breve
    let new_char = apply_breve(found_char);
    if new_char == found_char {
        return handle_pass(ctx);
    }

    // Build new transformed string
    let mut new_transformed = String::new();
    for (i, c) in transformed.char_indices() {
        if i == char_pos {
            new_transformed.push(new_char);
        } else {
            new_transformed.push(c);
        }
    }

    // Update buffer
    ctx.buffer.push_raw(ctx.key);
    if let Some(raw_key) = ctx.buffer.raw_buffer_mut().last_mut() {
        raw_key.consume();
    }
    ctx.buffer.set_transformed(&new_transformed);
    ctx.buffer.has_transform = true;
    ctx.buffer.track.record_mark(ctx.key, char_pos, mark::BREVE);

    let chars_after = transformed.chars().count() - char_pos;
    let new_suffix: String = new_transformed.chars().skip(char_pos).collect();

    ProcessResult::Transform {
        backspaces: chars_after as u8,
        output: new_suffix,
    }
}

/// Handle STROKE - apply đ
fn handle_stroke(ctx: &mut ActionContext) -> ProcessResult {
    // Check for double-key revert (ddd → dd)
    if ctx.buffer.has_stroke {
        return handle_revert(ctx);
    }

    // Find 'd' to apply stroke
    let transformed = ctx.buffer.transformed();

    let mut found_pos = None;
    let mut found_char = '\0';
    for (i, c) in transformed.char_indices() {
        let lower = c.to_ascii_lowercase();
        if lower == 'd' {
            found_pos = Some(i);
            found_char = c;
            break; // Take first 'd' (initial position)
        }
    }

    let char_pos = match found_pos {
        Some(pos) => pos,
        None => return handle_pass(ctx),
    };

    // Apply stroke
    let new_char = apply_stroke(found_char);

    // Build new transformed string
    let mut new_transformed = String::new();
    for (i, c) in transformed.char_indices() {
        if i == char_pos {
            new_transformed.push(new_char);
        } else {
            new_transformed.push(c);
        }
    }

    // Update buffer
    ctx.buffer.push_raw(ctx.key);
    if let Some(raw_key) = ctx.buffer.raw_buffer_mut().last_mut() {
        raw_key.consume();
    }
    ctx.buffer.set_transformed(&new_transformed);
    ctx.buffer.has_transform = true;
    ctx.buffer.has_stroke = true;
    ctx.buffer.track.record_stroke(ctx.key, char_pos);

    let chars_after = transformed.chars().count() - char_pos;
    let new_suffix: String = new_transformed.chars().skip(char_pos).collect();

    ProcessResult::Transform {
        backspaces: chars_after as u8,
        output: new_suffix,
    }
}

/// Handle REVERT - undo last transform on double-key
fn handle_revert(ctx: &mut ActionContext) -> ProcessResult {
    let transformed = ctx.buffer.transformed();
    let track = &ctx.buffer.track;

    match track.xform_type {
        XformType::Tone => {
            // Remove tone from vowel
            let pos = track.position;
            let mut new_transformed = String::new();
            for (i, c) in transformed.char_indices() {
                if i == pos {
                    new_transformed.push(remove_tone(c));
                } else {
                    new_transformed.push(c);
                }
            }
            // Append the revert key
            new_transformed.push(ctx.key);

            // Set pending pop for consumed modifier
            let consumed = track.consumed_key;
            ctx.buffer
                .raw_buffer_mut()
                .set_pending_pop(consumed, ctx.key);

            // Update buffer
            ctx.buffer.push_raw(ctx.key);
            ctx.buffer.set_transformed(&new_transformed);
            ctx.buffer.track.clear();
            ctx.buffer.has_transform = check_has_transform(&new_transformed);

            let chars_after = transformed.chars().count() - pos;
            let new_suffix: String = new_transformed.chars().skip(pos).collect();

            ProcessResult::Transform {
                backspaces: chars_after as u8,
                output: new_suffix,
            }
        }
        XformType::Mark => {
            // Remove mark from vowel
            let pos = track.position;
            let mark_value = track.mark_value;
            let mut new_transformed = String::new();
            for (i, c) in transformed.char_indices() {
                if i == pos {
                    let base = match mark_value {
                        1 => remove_circumflex(c),
                        2 => remove_horn(c),
                        3 => remove_breve(c),
                        _ => c,
                    };
                    new_transformed.push(base);
                } else {
                    new_transformed.push(c);
                }
            }
            // Append the revert key
            new_transformed.push(ctx.key);

            // Set pending pop
            let consumed = track.consumed_key;
            ctx.buffer
                .raw_buffer_mut()
                .set_pending_pop(consumed, ctx.key);

            // Update buffer
            ctx.buffer.push_raw(ctx.key);
            ctx.buffer.set_transformed(&new_transformed);
            ctx.buffer.track.clear();
            ctx.buffer.has_transform = check_has_transform(&new_transformed);

            let chars_after = transformed.chars().count() - pos;
            let new_suffix: String = new_transformed.chars().skip(pos).collect();

            ProcessResult::Transform {
                backspaces: chars_after as u8,
                output: new_suffix,
            }
        }
        XformType::Stroke => {
            // Remove stroke from đ
            let pos = track.position;
            let mut new_transformed = String::new();
            for (i, c) in transformed.char_indices() {
                if i == pos {
                    new_transformed.push(remove_stroke(c));
                } else {
                    new_transformed.push(c);
                }
            }
            // Append the revert key
            new_transformed.push(ctx.key);

            // Pop consumed 'd' immediately for stroke revert
            ctx.buffer.raw_buffer_mut().pop();

            // Update buffer
            ctx.buffer.push_raw(ctx.key);
            ctx.buffer.set_transformed(&new_transformed);
            ctx.buffer.track.clear();
            ctx.buffer.has_stroke = false;
            ctx.buffer.has_transform = check_has_transform(&new_transformed);

            let chars_after = transformed.chars().count() - pos;
            let new_suffix: String = new_transformed.chars().skip(pos).collect();

            ProcessResult::Transform {
                backspaces: chars_after as u8,
                output: new_suffix,
            }
        }
        XformType::None => handle_pass(ctx),
    }
}

/// Handle COMMIT - word boundary
fn handle_commit(ctx: &mut ActionContext) -> ProcessResult {
    // Handle pending pop on space
    ctx.buffer.raw_buffer_mut().handle_pending_pop_space();
    ProcessResult::Commit
}

/// Handle DEFER - resolve ambiguous cases based on context
///
/// Defer is dispatched when the action depends on previous buffer state:
/// - Same key as trigger → Revert (double-key undo)
/// - Tone keys (s/f/r/x/j) → Apply or change tone
/// - aa/ee/oo → Circumflex (same vowel repeated)
/// - aw → Breve (a + w)
/// - ow/uw → Horn (o/u + w)
/// - Other → Pass through
fn handle_defer(ctx: &mut ActionContext) -> ProcessResult {
    let key_lower = ctx.key.to_ascii_lowercase();

    // Check for revert first (works for all transform types: tone, mark, etc.)
    if ctx.buffer.track.should_revert(ctx.key) {
        return handle_revert(ctx);
    }

    let transformed = ctx.buffer.transformed();
    let last_char = transformed.chars().last();

    match (last_char, key_lower) {
        // TONE: apply or change tone (s/f/r/x/j)
        (Some(_), k) if matches!(k, 's' | 'f' | 'r' | 'x' | 'j') => handle_tone(ctx),
        // CIRCUMFLEX: aa→â, ee→ê, oo→ô (same vowel repeated)
        (Some(c), k) if c.to_ascii_lowercase() == k && matches!(k, 'a' | 'e' | 'o') => {
            handle_circumflex(ctx)
        }
        // BREVE: a+w → ă
        (Some(c), 'w') if c.to_ascii_lowercase() == 'a' => handle_breve(ctx),
        // HORN: o+w→ơ, u+w→ư
        (Some(c), 'w') if matches!(c.to_ascii_lowercase(), 'o' | 'u') => handle_horn(ctx),
        // Default: pass through as regular character
        _ => handle_pass(ctx),
    }
}

/// Check if transformed string has any Vietnamese marks
fn check_has_transform(s: &str) -> bool {
    for c in s.chars() {
        match c {
            'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư' | 'đ' => return true,
            'Ă' | 'Â' | 'Ê' | 'Ô' | 'Ơ' | 'Ư' | 'Đ' => return true,
            'á' | 'à' | 'ả' | 'ã' | 'ạ' => return true,
            'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => return true,
            'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => return true,
            'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' => return true,
            'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => return true,
            'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => return true,
            'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' => return true,
            'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' => return true,
            'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => return true,
            'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' => return true,
            'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => return true,
            'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => return true,
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foreign_mode_pass() {
        let (action, state) = process_keystroke(State::Vow, 'a', true);
        assert_eq!(action, Action::Pass);
        assert_eq!(state, State::Vow);
    }

    #[test]
    fn test_foreign_mode_boundary_exits() {
        let (action, state) = process_keystroke(State::Vow, ' ', true);
        assert_eq!(action, Action::Commit);
        assert_eq!(state, State::Empty);
    }

    #[test]
    fn test_normal_dispatch() {
        // Vowel from empty state
        let (action, state) = process_keystroke(State::Empty, 'a', false);
        assert_eq!(action, Action::AddVowel);
        assert_eq!(state, State::Vow);
    }

    #[test]
    fn test_tone_on_vowel() {
        let (action, state) = process_keystroke(State::Vow, 's', false);
        assert_eq!(action, Action::ApplyTone);
        assert_eq!(state, State::Dia);
    }

    #[test]
    fn test_handle_tone() {
        let mut buffer = DualBuffer::new();
        buffer.push_raw('b');
        buffer.push_raw('a');
        buffer.set_transformed("ba");

        let mut ctx = ActionContext {
            buffer: &mut buffer,
            key: 's',
            method: 0,
        };

        let result = handle_tone(&mut ctx);
        assert!(matches!(result, ProcessResult::Transform { .. }));
        assert_eq!(ctx.buffer.transformed(), "bá");
    }

    #[test]
    fn test_handle_circumflex() {
        let mut buffer = DualBuffer::new();
        buffer.push_raw('a');
        buffer.set_transformed("a");

        let mut ctx = ActionContext {
            buffer: &mut buffer,
            key: 'a',
            method: 0,
        };

        let result = handle_circumflex(&mut ctx);
        assert!(matches!(result, ProcessResult::Transform { .. }));
        assert_eq!(ctx.buffer.transformed(), "â");
    }

    #[test]
    fn test_handle_horn() {
        let mut buffer = DualBuffer::new();
        buffer.push_raw('u');
        buffer.set_transformed("u");

        let mut ctx = ActionContext {
            buffer: &mut buffer,
            key: 'w',
            method: 0,
        };

        let result = handle_horn(&mut ctx);
        assert!(matches!(result, ProcessResult::Transform { .. }));
        assert_eq!(ctx.buffer.transformed(), "ư");
    }

    #[test]
    fn test_handle_stroke() {
        let mut buffer = DualBuffer::new();
        buffer.push_raw('d');
        buffer.set_transformed("d");

        let mut ctx = ActionContext {
            buffer: &mut buffer,
            key: 'd',
            method: 0,
        };

        let result = handle_stroke(&mut ctx);
        assert!(matches!(result, ProcessResult::Transform { .. }));
        assert_eq!(ctx.buffer.transformed(), "đ");
    }

    #[test]
    fn test_tone_revert() {
        let mut buffer = DualBuffer::new();
        buffer.push_raw('b');
        buffer.push_raw('a');
        buffer.set_transformed("ba");

        // Apply tone
        let mut ctx = ActionContext {
            buffer: &mut buffer,
            key: 's',
            method: 0,
        };
        handle_tone(&mut ctx);
        assert_eq!(ctx.buffer.transformed(), "bá");

        // Revert with same key
        ctx.key = 's';
        let result = handle_revert(&mut ctx);
        assert!(matches!(result, ProcessResult::Transform { .. }));
        assert_eq!(ctx.buffer.transformed(), "bas");
    }

    #[test]
    fn test_check_has_transform() {
        assert!(check_has_transform("bá"));
        assert!(check_has_transform("đi"));
        assert!(check_has_transform("ăn"));
        assert!(!check_has_transform("ba"));
        assert!(!check_has_transform("test"));
    }
}
