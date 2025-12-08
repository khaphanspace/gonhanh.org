//! Common test utilities - DRY helpers for all test modules

#![allow(dead_code)]

use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine};

// ============================================================
// KEY MAPPING
// ============================================================

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
// TEST RUNNERS - Dynamic test execution from data arrays
// ============================================================

/// Input method type
#[derive(Clone, Copy, Debug)]
pub enum Method {
    Telex,
    Vni,
}

/// Orthography mode
#[derive(Clone, Copy, Debug)]
pub enum Ortho {
    Modern,
    Classic,
}

/// Test case definition
pub struct Case<'a> {
    pub input: &'a str,
    pub expected: &'a str,
}

impl<'a> From<(&'a str, &'a str)> for Case<'a> {
    fn from((input, expected): (&'a str, &'a str)) -> Self {
        Self { input, expected }
    }
}

/// Run test cases with method
pub fn run(method: Method, cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        if matches!(method, Method::Vni) {
            e.set_method(1);
        }
        let result = type_word(&mut e, input);
        let method_name = match method {
            Method::Telex => "Telex",
            Method::Vni => "VNI",
        };
        assert_eq!(
            result, *expected,
            "\n[{}] '{}' → '{}' (expected '{}')",
            method_name, input, result, expected
        );
    }
}

/// Run with orthography setting
pub fn run_with_ortho(method: Method, ortho: Ortho, cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        if matches!(method, Method::Vni) {
            e.set_method(1);
        }
        e.set_modern(matches!(ortho, Ortho::Modern));
        let result = type_word(&mut e, input);
        assert_eq!(
            result, *expected,
            "\n[{:?}/{:?}] '{}' → '{}' (expected '{}')",
            method, ortho, input, result, expected
        );
    }
}

/// Shorthand: Telex tests
pub fn telex(cases: &[(&str, &str)]) {
    run(Method::Telex, cases);
}

/// Shorthand: VNI tests
pub fn vni(cases: &[(&str, &str)]) {
    run(Method::Vni, cases);
}

/// Run same cases for both methods (with different inputs)
pub fn both(telex_cases: &[(&str, &str)], vni_cases: &[(&str, &str)]) {
    telex(telex_cases);
    vni(vni_cases);
}

// ============================================================
// ENGINE STATE HELPERS
// ============================================================

pub fn engine_telex() -> Engine {
    Engine::new()
}

pub fn engine_vni() -> Engine {
    let mut e = Engine::new();
    e.set_method(1);
    e
}

pub fn engine_modern() -> Engine {
    let mut e = Engine::new();
    e.set_modern(true);
    e
}

pub fn engine_classic() -> Engine {
    let mut e = Engine::new();
    e.set_modern(false);
    e
}

// ============================================================
// ASSERTION HELPERS
// ============================================================

/// Assert engine action
pub fn assert_action(e: &mut Engine, key: u16, caps: bool, ctrl: bool, expected: Action) {
    let r = e.on_key(key, caps, ctrl);
    assert_eq!(
        r.action, expected as u8,
        "Expected {:?} for key {}",
        expected, key
    );
}

/// Assert pass-through (no transformation)
pub fn assert_passthrough(e: &mut Engine, key: u16) {
    assert_action(e, key, false, false, Action::None);
}

/// Assert transformation happens
pub fn assert_transforms(e: &mut Engine, key: u16) {
    assert_action(e, key, false, false, Action::Send);
}
