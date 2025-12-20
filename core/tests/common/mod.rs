//! Common test utilities - Re-exports from core + integration test helpers
//!
//! Core functions are shared from `gonhanh_core::test_utils`.
//! This module adds additional helpers for integration tests.

#![allow(dead_code)]
#![allow(unused_imports)]

// Re-export core test utilities
pub use gonhanh_core::utils::{
    telex, telex_auto_restore, telex_traditional, type_word, vni, vni_traditional,
};

use gonhanh_core::engine::{Action, Engine};

// ============================================================
// TEST RUNNERS - Extended helpers for integration tests
// ============================================================

/// Input method type
#[derive(Clone, Copy, Debug)]
pub enum Method {
    Telex,
    Vni,
}

/// Run test cases with method
pub fn run(method: Method, cases: &[(&str, &str)]) {
    match method {
        Method::Telex => telex(cases),
        Method::Vni => vni(cases),
    }
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
