//! Regression test for Telex `w` in shell-like paths.
//!
//! Reported sequence: type `~/ww/` as `~, w, w, /` and the editor receives
//! `~/ww/`, while the expected result is `~/w/`.

mod common;

use common::type_word;
use gonhanh_core::engine::Engine;

#[test]
fn telex_double_w_before_slash_should_leave_one_w() {
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);
    let actual = type_word(&mut engine, "~/ww/");

    assert_eq!(
        actual, "~/w/",
        "Typing `~, w, w, /` should produce `~/w/`, not duplicate `w`"
    );
}

#[test]
fn telex_w_modifier_and_path_behavior() {
    let cases = [("w", "ư"), ("ww", "w"), ("ww ", "w "), ("~/ww/", "~/w/")];

    for (input, expected) in cases {
        let mut engine = Engine::new();
        engine.set_english_auto_restore(true);
        assert_eq!(
            type_word(&mut engine, input),
            expected,
            "Unexpected output for Telex input {input:?}"
        );
    }
}

#[test]
fn english_words_with_w_still_auto_restore() {
    for input in ["with ", "wow ", "window "] {
        let mut engine = Engine::new();
        engine.set_english_auto_restore(true);
        assert_eq!(
            type_word(&mut engine, input),
            input,
            "Valid English input {input:?} should still auto-restore"
        );
    }
}
