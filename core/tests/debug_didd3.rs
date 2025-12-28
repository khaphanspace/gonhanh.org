//! Debug test for "didd" step by step

use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_didd_step_by_step() {
    let mut engine = Engine::new();

    let keys_sequence = [
        (keys::D, "D1"),
        (keys::I, "I"),
        (keys::D, "D2"),
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
}
