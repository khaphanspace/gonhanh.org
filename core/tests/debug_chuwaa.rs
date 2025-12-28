//! Debug test for "chuwaa" → "chưaa" issue

use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_chuwaa_step_by_step() {
    let mut engine = Engine::new();

    let keys_sequence = [
        (keys::C, "C"),
        (keys::H, "H"),
        (keys::U, "U"),
        (keys::W, "W"),
        (keys::A, "A1"),
        (keys::A, "A2"),
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

    // Expected: "chưaa" (second 'a' should be literal since horn is on 'ư')
    assert_eq!(
        final_buf, "chưaa",
        "'chuwaa' should produce 'chưaa', got '{}'",
        final_buf
    );
}
