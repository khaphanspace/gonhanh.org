//! Debug test for "uwow" pattern regression

use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_uwow_step_by_step() {
    let mut engine = Engine::new();

    let keys_sequence = [
        (keys::U, "U"),
        (keys::W, "W1"),
        (keys::O, "O"),
        (keys::W, "W2"),
    ];

    for (i, &(key, name)) in keys_sequence.iter().enumerate() {
        let r = engine.on_key_ext(key, false, false, false);
        let buf = engine.get_buffer_string();
        eprintln!(
            "[Step {}] Key: {:?}, action={}, buffer='{}'",
            i + 1,
            name,
            r.action,
            buf
        );
    }

    let final_buf = engine.get_buffer_string();
    eprintln!("Final buffer: '{}'", final_buf);

    // Expected: "ươ" (u+w→ư, o+w→ơ, compound ươ)
    assert_eq!(
        final_buf, "ươ",
        "'uwow' should produce 'ươ', got '{}'",
        final_buf
    );
}
