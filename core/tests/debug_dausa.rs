use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_dausa() {
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    // Type "dausa"
    let word_keys = [keys::D, keys::A, keys::U, keys::S, keys::A];

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

    // Type SPACE
    let r = engine.on_key_ext(keys::SPACE, false, false, false);
    eprintln!(
        "Space: action={}, backspace={}, count={}",
        r.action, r.backspace, r.count
    );

    let output: String = (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect();
    eprintln!("Output: '{}'", output);

    // When VN buffer is kept (no restore), action=0 and output is empty
    // The buffer "dấu" is retained
    assert_eq!(r.action, 0, "dausa should NOT restore - keep VN buffer");
    let final_buf = engine.get_buffer_string();
    // Buffer is cleared after space, but we verified action=0 (keep buffer)
    eprintln!("Final buffer: '{}'", final_buf);
}
