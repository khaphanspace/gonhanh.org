use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_azure() {
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    // Type "azure"
    let word_keys = [keys::A, keys::Z, keys::U, keys::R, keys::E];

    for &key in &word_keys {
        let r = engine.on_key_ext(key, false, false, false);
        let buf = engine.get_buffer_string();
        let raw = engine.get_raw_string();
        eprintln!(
            "Key {:?}: action={}, buffer='{}', raw='{}'",
            key, r.action, buf, raw
        );
    }

    let buf = engine.get_buffer_string();
    let raw = engine.get_raw_string();
    eprintln!("Before Space: buffer='{}', raw='{}'", buf, raw);

    // Space
    let r = engine.on_key_ext(keys::SPACE, false, false, false);
    eprintln!(
        "Space: action={}, backspace={}, count={}",
        r.action, r.backspace, r.count
    );

    let output: String = (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect();
    eprintln!("Space output: '{}'", output);
}
