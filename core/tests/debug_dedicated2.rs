//! Debug test for "dedicated" non-adjacent stroke issue

use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

#[test]
fn debug_dedicated_step_by_step() {
    let mut engine = Engine::new();

    let keys_sequence = [
        (keys::D, "D1"),
        (keys::E, "E1"),
        (keys::D, "D2"),
        (keys::I, "I"),
        (keys::C, "C"),
        (keys::A, "A"),
        (keys::T, "T"),
        (keys::E, "E2"),
        (keys::D, "D3"),
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

    // Expected: "dedicated" (non-adjacent stroke should NOT trigger)
    assert_eq!(
        final_buf, "dedicated",
        "'dedicated' should stay as 'dedicated', got '{}'",
        final_buf
    );
}

#[test]
fn debug_dedicated_with_space() {
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    let result = type_word(&mut engine, "dedicated ");
    eprintln!("Result: '{}'", result);

    // Expected: "dedicated " (English word preserved)
    assert_eq!(
        result, "dedicated ",
        "'dedicated ' should stay as 'dedicated ', got '{}'",
        result
    );
}
