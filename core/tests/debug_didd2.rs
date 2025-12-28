//! Debug test for "didd" double 'd' revert issue

use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

#[test]
fn debug_didd_restore() {
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    // "didd" should auto-restore to "did" (double consonant revert)
    let result = type_word(&mut engine, "didd ");
    eprintln!("Result: '{}'", result);

    // Expected: "did " (double 'd' reverted)
    assert_eq!(
        result, "did ",
        "'didd ' should auto-restore to 'did ', got '{}'",
        result
    );
}
