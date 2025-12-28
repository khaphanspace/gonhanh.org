//! Debug test for "dedicated" non-adjacent stroke with buffer length check

use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_dedicated_length() {
    let mut engine = Engine::new();

    let keys_sequence = [(keys::D, "D1"), (keys::E, "E1"), (keys::D, "D2")];

    for (i, &(key, name)) in keys_sequence.iter().enumerate() {
        let before_len = engine.get_buffer_string().len();
        let r = engine.on_key_ext(key, false, false, false);
        let buf = engine.get_buffer_string();
        eprintln!(
            "[Step {}] Key: {:?}, before_len={}, action={}, buffer='{}'",
            i + 1,
            name,
            before_len,
            r.action,
            buf
        );
    }
}
