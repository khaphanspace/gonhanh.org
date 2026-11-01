//! Regression tests for Telex `uo + w` horn placement.

mod common;

use common::type_word;
use gonhanh_core::engine::Engine;

#[test]
fn telex_uo_circumflex_switch_to_horn_updates_both_vowels() {
    let mut engine = Engine::new();

    // `buoco` first forms `buôc`; the following `w` switches `ô` to `ơ`.
    // The adjacent `u` must also receive horn, producing `bươc`.
    assert_eq!(type_word(&mut engine, "buocow"), "bươc");
}

#[test]
fn telex_uo_horn_with_final_consonant_updates_both_vowels() {
    let mut engine = Engine::new();

    assert_eq!(type_word(&mut engine, "buowc"), "bươc");
}
