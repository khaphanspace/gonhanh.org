//! Debug test for "loxoi" → "lỗi" issue

use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::validation::is_foreign_pattern_keys;
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

#[test]
fn debug_loxoi_pattern() {
    // Check if "loxoi" is incorrectly detected as foreign
    let keys_seq = [keys::L, keys::O, keys::X, keys::O, keys::I];
    let is_foreign = is_foreign_pattern_keys(&keys_seq);
    eprintln!("is_foreign_pattern_keys([L,O,X,O,I]) = {}", is_foreign);
    assert!(
        !is_foreign,
        "loxoi should NOT be detected as foreign pattern"
    );
}

#[test]
fn debug_loxoi() {
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    // Use type_word helper which properly collects output
    let result = type_word(&mut engine, "loxoi ");

    eprintln!("Result: '{}'", result);

    // Expected: "lỗi " (Vietnamese word, should NOT be auto-restored)
    assert_eq!(
        result, "lỗi ",
        "Vietnamese 'loxoi ' should produce 'lỗi ', got '{}'",
        result
    );
}
