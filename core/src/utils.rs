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
            _ => 255, // Unknown/Other
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
            let key = char_to_key(c);
            let is_caps = c.is_uppercase();

            if key == keys::DELETE {
                screen.pop();
                e.on_key(key, false, false);
                continue;
            }

            if key == keys::SPACE {
                // Space can trigger shortcuts - process result
                let r = e.on_key(key, false, false);
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

            let r = e.on_key(key, is_caps, false);
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

    /// Run VNI test cases
    pub fn vni(cases: &[(&str, &str)]) {
        for (input, expected) in cases {
            let mut e = Engine::new();
            e.set_method(1);
            let result = type_word(&mut e, input);
            assert_eq!(result, *expected, "[VNI] '{}' → '{}'", input, result);
        }
    }
}

// Re-export test utilities for use in other test modules
pub use test_utils::*;
