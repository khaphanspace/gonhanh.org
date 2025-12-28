//! Debug test for "datdas" → "đất"

use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_datdas_step_by_step() {
    let mut engine = Engine::new();

    let keys_sequence = [
        (keys::D, "D1"),
        (keys::A, "A1"),
        (keys::T, "T"),
        (keys::D, "D2"),
        (keys::A, "A2"),
        (keys::S, "S"),
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
    eprintln!("Final: '{}'", final_buf);

    // Expected: "đất" (d+a+t+d → stroke+circumflex, then s adds tone)
    assert_eq!(final_buf, "đất", "Expected 'đất', got '{}'", final_buf);
}
