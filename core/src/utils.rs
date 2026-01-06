//! Shared utilities for Vietnamese IME processing
//!
//! Contains common functions used across engine modules to avoid duplication.
//! Also includes test utilities under #[cfg(test)].

use crate::data::{
    chars::tone,
    keys,
    vowel::{Modifier, Vowel},
};
use crate::engine::buffer::Buffer;

/// Convert key code to character
pub fn key_to_char(key: u16, caps: bool) -> Option<char> {
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

/// Convert key code to character with shift state support
/// Handles shifted symbols like @ (Shift+2), # (Shift+3), etc.
pub fn key_to_char_ext(key: u16, caps: bool, shift: bool) -> Option<char> {
    // If shift is pressed, check for shifted symbols first
    if shift {
        return match key {
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
            _ => key_to_char(key, caps),
        };
    }
    key_to_char(key, caps)
}

/// Collect vowels from buffer with phonological info
pub fn collect_vowels(buf: &Buffer) -> Vec<Vowel> {
    buf.iter()
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

/// Check if there's a consonant after position
pub fn has_final_consonant(buf: &Buffer, after_pos: usize) -> bool {
    (after_pos + 1..buf.len()).any(|i| {
        buf.get(i)
            .map(|c| keys::is_consonant(c.key))
            .unwrap_or(false)
    })
}

/// Check if 'q' precedes 'u' in buffer
pub fn has_qu_initial(buf: &Buffer) -> bool {
    for (i, c) in buf.iter().enumerate() {
        if c.key == keys::U && i > 0 {
            if let Some(prev) = buf.get(i - 1) {
                return prev.key == keys::Q;
            }
        }
    }
    false
}

/// Check if 'gi' is initial followed by another vowel
/// e.g., "gia", "giau" → gi is initial, 'i' is NOT a vowel
pub fn has_gi_initial(buf: &Buffer) -> bool {
    if buf.len() < 3 {
        return false;
    }
    // Check for g + i + vowel pattern
    let first = buf.get(0).map(|c| c.key);
    let second = buf.get(1).map(|c| c.key);
    let third = buf.get(2).map(|c| c.key);

    matches!((first, second), (Some(keys::G), Some(keys::I)))
        && third.map(keys::is_vowel).unwrap_or(false)
}

mod test_utils {
    //! Shared test utilities for inline tests
    //!
    //! Provides common helpers for testing Vietnamese IME engine.
    //! Used by `#[cfg(test)]` modules throughout the crate.

    use crate::data::keys;
    use crate::engine::{Action, Engine};

    // ============================================================
    // KEY MAPPING
    // ============================================================

    /// Convert character to key code
    pub fn char_to_key(c: char) -> u16 {
        match c.to_ascii_lowercase() {
            'a' => keys::A,
            'b' => keys::B,
            'c' => keys::C,
            'd' => keys::D,
            'e' => keys::E,
            'f' => keys::F,
            'g' => keys::G,
            'h' => keys::H,
            'i' => keys::I,
            'j' => keys::J,
            'k' => keys::K,
            'l' => keys::L,
            'm' => keys::M,
            'n' => keys::N,
            'o' => keys::O,
            'p' => keys::P,
            'q' => keys::Q,
            'r' => keys::R,
            's' => keys::S,
            't' => keys::T,
            'u' => keys::U,
            'v' => keys::V,
            'w' => keys::W,
            'x' => keys::X,
            'y' => keys::Y,
            'z' => keys::Z,
            '0' => keys::N0,
            '1' => keys::N1,
            '2' => keys::N2,
            '3' => keys::N3,
            '4' => keys::N4,
            '5' => keys::N5,
            '6' => keys::N6,
            '7' => keys::N7,
            '8' => keys::N8,
            '9' => keys::N9,
            '.' => keys::DOT,
            ',' => keys::COMMA,
            ';' => keys::SEMICOLON,
            ':' => keys::SEMICOLON, // Approximate
            '\'' => keys::QUOTE,
            '"' => keys::QUOTE,
            '-' => keys::MINUS,
            '=' => keys::EQUAL,
            '[' => keys::LBRACKET,
            ']' => keys::RBRACKET,
            '\\' => keys::BACKSLASH,
            '/' => keys::SLASH,
            '`' => keys::BACKQUOTE,
            '<' => keys::DELETE,
            ' ' => keys::SPACE,
            '\x1b' => keys::ESC, // ESC character
            // Common symbols - map to base key (handler checks shift state)
            '@' => keys::N2,    // Shift+2
            '!' => keys::N1,    // Shift+1
            '#' => keys::N3,    // Shift+3
            '$' => keys::N4,    // Shift+4
            '%' => keys::N5,    // Shift+5
            '^' => keys::N6,    // Shift+6
            '&' => keys::N7,    // Shift+7
            '*' => keys::N8,    // Shift+8
            '(' => keys::N9,    // Shift+9
            ')' => keys::N0,    // Shift+0
            '_' => keys::MINUS, // Shift+-
            '+' => keys::EQUAL, // Shift+=
            _ => 255,           // Unknown/Other
        }
    }

    /// Convert string to key codes
    pub fn keys_from_str(s: &str) -> Vec<u16> {
        s.chars().map(char_to_key).filter(|&k| k != 255).collect()
    }

    // ============================================================
    // TYPING SIMULATION
    // ============================================================

    /// Simulate typing, returns screen output
    pub fn type_word(e: &mut Engine, input: &str) -> String {
        let mut screen = String::new();
        for c in input.chars() {
            // Detect shifted symbols and get proper (key, shift) pair
            // NOTE: '<' is NOT included here - it maps to DELETE in test utilities
            let (key, shift) = match c {
                '@' => (keys::N2, true),
                '!' => (keys::N1, true),
                '#' => (keys::N3, true),
                '$' => (keys::N4, true),
                '%' => (keys::N5, true),
                '^' => (keys::N6, true),
                '&' => (keys::N7, true),
                '*' => (keys::N8, true),
                '(' => (keys::N9, true),
                ')' => (keys::N0, true),
                '_' => (keys::MINUS, true),
                '+' => (keys::EQUAL, true),
                ':' => (keys::SEMICOLON, true),
                '"' => (keys::QUOTE, true),
                '>' => (keys::DOT, true),
                '?' => (keys::SLASH, true),
                '|' => (keys::BACKSLASH, true),
                '{' => (keys::LBRACKET, true),
                '}' => (keys::RBRACKET, true),
                '~' => (keys::BACKQUOTE, true),
                _ => (char_to_key(c), false),
            };
            let is_caps = c.is_uppercase();

            if key == keys::DELETE {
                let r = e.on_key_ext(key, false, false, false);
                if r.action == Action::Send as u8 {
                    // Restore from history - apply backspaces and replacement
                    for _ in 0..r.backspace {
                        screen.pop();
                    }
                    for i in 0..r.count as usize {
                        if let Some(ch) = char::from_u32(r.chars[i]) {
                            screen.push(ch);
                        }
                    }
                } else {
                    // Normal backspace - just remove last char
                    screen.pop();
                }
                continue;
            }

            // ESC key: restore to raw ASCII
            if key == keys::ESC {
                let r = e.on_key_ext(key, false, false, false);
                if r.action == Action::Send as u8 {
                    for _ in 0..r.backspace {
                        screen.pop();
                    }
                    for i in 0..r.count as usize {
                        if let Some(ch) = char::from_u32(r.chars[i]) {
                            screen.push(ch);
                        }
                    }
                }
                continue;
            }

            if key == keys::SPACE {
                // Space can trigger shortcuts - process result
                let r = e.on_key_ext(key, false, false, false);
                if r.action == Action::Send as u8 {
                    // Shortcut triggered - apply backspaces and replacement
                    for _ in 0..r.backspace {
                        screen.pop();
                    }
                    for i in 0..r.count as usize {
                        if let Some(ch) = char::from_u32(r.chars[i]) {
                            screen.push(ch);
                        }
                    }
                } else {
                    // No shortcut - just add space
                    screen.push(' ');
                }
                continue;
            }

            let r = e.on_key_ext(key, is_caps, false, shift);
            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        screen.push(ch);
                    }
                }
                // For break keys (punctuation), add the character after auto-restore
                // The restored text doesn't include the break character
                // Use is_break_ext to handle shifted symbols like @, !, #, etc.
                // BUT: if key_consumed flag is set (shortcut match), don't add the char
                if keys::is_break_ext(key, shift) && !r.key_consumed() {
                    screen.push(c);
                }
            } else {
                // Pass through if not handled (mimic editor receiving char)
                screen.push(c);
            }
        }
        screen
    }

    // ============================================================
    // TEST RUNNERS
    // ============================================================

    /// Run Telex test cases
    pub fn telex(cases: &[(&str, &str)]) {
        for (input, expected) in cases {
            let mut e = Engine::new();
            let result = type_word(&mut e, input);
            assert_eq!(result, *expected, "[Telex] '{}' → '{}'", input, result);
        }
    }

    /// Run Telex test cases with English auto-restore enabled
    pub fn telex_auto_restore(cases: &[(&str, &str)]) {
        for (input, expected) in cases {
            let mut e = Engine::new();
            e.set_english_auto_restore(true);
            let result = type_word(&mut e, input);
            assert_eq!(
                result, *expected,
                "[Telex AutoRestore] '{}' → '{}'",
                input, result
            );
        }
    }

    /// Run Telex test cases with auto-capitalize enabled
    pub fn telex_auto_capitalize(cases: &[(&str, &str)]) {
        for (input, expected) in cases {
            let mut e = Engine::new();
            e.set_auto_capitalize(true);
            let result = type_word(&mut e, input);
            assert_eq!(
                result, *expected,
                "[Telex AutoCapitalize] '{}' → '{}'",
                input, result
            );
        }
    }

    /// Run VNI test cases
    pub fn vni(cases: &[(&str, &str)]) {
        for (input, expected) in cases {
            let mut e = Engine::new();
            e.set_method(1);
            let result = type_word(&mut e, input);
            assert_eq!(result, *expected, "[VNI] '{}' → '{}'", input, result);
        }
    }

    /// Run Telex test cases with traditional tone placement (hòa, thúy style)
    pub fn telex_traditional(cases: &[(&str, &str)]) {
        for (input, expected) in cases {
            let mut e = Engine::new();
            e.set_modern_tone(false);
            let result = type_word(&mut e, input);
            assert_eq!(
                result, *expected,
                "[Telex Traditional] '{}' → '{}'",
                input, result
            );
        }
    }

    /// Run VNI test cases with traditional tone placement (hòa, thúy style)
    pub fn vni_traditional(cases: &[(&str, &str)]) {
        for (input, expected) in cases {
            let mut e = Engine::new();
            e.set_method(1);
            e.set_modern_tone(false);
            let result = type_word(&mut e, input);
            assert_eq!(
                result, *expected,
                "[VNI Traditional] '{}' → '{}'",
                input, result
            );
        }
    }

    /// Simulate typing with extended parameters (supports raw mode prefix)
    /// Input format: use special prefixes to trigger shift+key:
    /// - "@" triggers Shift+2
    /// - "#" triggers Shift+3
    /// - ":" triggers Shift+;
    /// - "/" triggers SLASH (no shift)
    pub fn type_word_ext(e: &mut Engine, input: &str) -> String {
        let mut screen = String::new();
        for c in input.chars() {
            // Handle raw mode prefix characters
            let (key, shift) = match c {
                '@' => (keys::N2, true),
                '#' => (keys::N3, true),
                ':' => (keys::SEMICOLON, true),
                '/' => (keys::SLASH, false), // / doesn't need shift
                _ => (char_to_key(c), false),
            };

            let is_caps = c.is_uppercase();

            if key == keys::DELETE {
                let r = e.on_key_ext(key, false, false, false);
                if r.action == Action::Send as u8 {
                    // Restore from history - apply backspaces and replacement
                    for _ in 0..r.backspace {
                        screen.pop();
                    }
                    for i in 0..r.count as usize {
                        if let Some(ch) = char::from_u32(r.chars[i]) {
                            screen.push(ch);
                        }
                    }
                } else {
                    // Normal backspace - just remove last char
                    screen.pop();
                }
                continue;
            }

            if key == keys::ESC {
                let r = e.on_key_ext(key, false, false, false);
                if r.action == Action::Send as u8 {
                    for _ in 0..r.backspace {
                        screen.pop();
                    }
                    for i in 0..r.count as usize {
                        if let Some(ch) = char::from_u32(r.chars[i]) {
                            screen.push(ch);
                        }
                    }
                }
                continue;
            }

            if key == keys::SPACE {
                let r = e.on_key_ext(key, false, false, false);
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
                    screen.push(' ');
                }
                continue;
            }

            let r = e.on_key_ext(key, is_caps, false, shift);
            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        screen.push(ch);
                    }
                }
                // For break keys (punctuation), add the character after auto-restore
                if keys::is_break(key) {
                    screen.push(c);
                }
            } else {
                // Pass through if not handled
                screen.push(c);
            }
        }
        screen
    }
}

// Re-export test utilities for use in other test modules
pub use test_utils::*;
