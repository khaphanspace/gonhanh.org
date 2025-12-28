use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_looks() {
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    // Type "looks"
    let word_keys = [keys::L, keys::O, keys::O, keys::K, keys::S];

    for &key in &word_keys {
        let r = engine.on_key_ext(key, false, false, false);
        let buf = engine.get_buffer_string();
        let raw = engine.get_raw_string();
        eprintln!(
            "Key {:?}: action={}, buffer='{}', raw='{}'",
            key, r.action, buf, raw
        );
    }

    eprintln!("--- Before Space ---");
    let buf_before = engine.get_buffer_string();
    let raw_before = engine.get_raw_string();
    eprintln!("Buffer: '{}', Raw: '{}'", buf_before, raw_before);

    // Space
    let r = engine.on_key_ext(keys::SPACE, false, false, false);
    eprintln!(
        "Space: action={}, backspace={}, count={}",
        r.action, r.backspace, r.count
    );

    // Check output chars
    let output: String = (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect();
    eprintln!("Space output: '{}'", output);

    // What should happen: "looks" detected as English, restore to raw
    assert_eq!(r.action, 2, "should trigger restore");
}
