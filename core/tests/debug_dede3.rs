//! Debug test for "dede" → "đê" - checking defer mechanism

use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_dede_defer() {
    let mut engine = Engine::new();

    // Step 1: 'd'
    let r1 = engine.on_key_ext(keys::D, false, false, false);
    eprintln!(
        "[Step 1] D1: action={}, buffer='{}'",
        r1.action,
        engine.get_buffer_string()
    );

    // Step 2: 'e'
    let r2 = engine.on_key_ext(keys::E, false, false, false);
    eprintln!(
        "[Step 2] E1: action={}, buffer='{}'",
        r2.action,
        engine.get_buffer_string()
    );

    // Step 3: 'd' - this should set up defer
    let r3 = engine.on_key_ext(keys::D, false, false, false);
    eprintln!(
        "[Step 3] D2: action={}, buffer='{}'",
        r3.action,
        engine.get_buffer_string()
    );

    // Step 4: 'e' - this should trigger circumflex AND resolve defer
    let r4 = engine.on_key_ext(keys::E, false, false, false);
    eprintln!(
        "[Step 4] E2: action={}, buffer='{}'",
        r4.action,
        engine.get_buffer_string()
    );

    let final_buf = engine.get_buffer_string();
    eprintln!("Final: '{}'", final_buf);

    // Check intermediate states
    // If defer worked: "d" → "de" → "ded" → "đê"
    // Current bug: "d" → "de" → "ded" → "dede"

    assert_eq!(final_buf, "đê", "Expected 'đê', got '{}'", final_buf);
}
