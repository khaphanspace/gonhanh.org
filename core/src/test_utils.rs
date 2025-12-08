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
        '<' => keys::DELETE,
        ' ' => keys::SPACE,
        _ => 255,
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
            screen.push(' ');
            e.on_key(key, false, false);
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
        } else if keys::is_letter(key) {
            screen.push(if is_caps {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            });
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
