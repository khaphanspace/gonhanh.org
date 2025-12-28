//! Debug trace for "user" typing
use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_user_trace() {
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    let keys_sequence = [
        (keys::U, "U"),
        (keys::S, "S"),
        (keys::E, "E"),
        (keys::R, "R"),
        (keys::DOT, "."),
    ];

    for (key, name) in keys_sequence {
        let r = engine.on_key_ext(key, false, false, false);
        let chars: String = (0..r.count as usize)
            .filter_map(|i| char::from_u32(r.chars[i]))
            .collect();
        eprintln!(
            "[{}] action={}, bs={}, count={}, chars='{}', buffer='{}'",
            name,
            r.action,
            r.backspace,
            r.count,
            chars,
            engine.get_buffer_string()
        );
    }

    let final_buf = engine.get_buffer_string();
    eprintln!("Final buffer: '{}'", final_buf);
}
