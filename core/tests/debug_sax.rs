use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::{
    is_buffer_invalid_vietnamese, is_foreign_pattern_keys, Processor,
};
use gonhanh_core::engine::Engine;

#[test]
fn debug_sax() {
    // Test "sax" - s+a+x → 'a' gets ngã, should restore to "sax"
    let keys_seq = [keys::S, keys::A, keys::X];

    eprintln!("--- Validation ---");
    eprintln!(
        "is_foreign_pattern_keys: {}",
        is_foreign_pattern_keys(&keys_seq)
    );

    // Test with processor
    let mut processor = Processor::new();
    for (i, &k) in keys_seq.iter().enumerate() {
        processor.process(k, false, false);
        let buf = processor.buffer().to_full_string();
        let raw: String = processor.raw().restore_all().iter().collect();
        eprintln!("[{}] Key {}: buffer='{}', raw='{}'", i, k, buf, raw);
    }
    let buf = processor.buffer().to_full_string();
    let raw: String = processor.raw().restore_all().iter().collect();
    eprintln!("\nFinal: buffer='{}', raw='{}'", buf, raw);
    eprintln!(
        "is_buffer_invalid_vietnamese: {}",
        is_buffer_invalid_vietnamese(&buf)
    );

    // Test with engine
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);
    let mut screen = String::new();
    for &k in &keys_seq {
        let r = engine.on_key(k, false, false);
        eprintln!("Key {}: backspace={}, count={}", k, r.backspace, r.count);
        for _ in 0..r.backspace {
            screen.pop();
        }
        for i in 0..r.count as usize {
            if let Some(c) = char::from_u32(r.chars[i]) {
                screen.push(c);
            }
        }
        eprintln!("  → screen='{}'", screen);
    }
    // Space to commit
    eprintln!("\n--- Space commit ---");
    let r = engine.on_key(keys::SPACE, false, false);
    eprintln!("Space: backspace={}, count={}", r.backspace, r.count);
    let space_output: String = (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect();
    eprintln!("Space output chars: '{}'", space_output);
    for _ in 0..r.backspace {
        screen.pop();
    }
    for i in 0..r.count as usize {
        if let Some(c) = char::from_u32(r.chars[i]) {
            screen.push(c);
        }
    }
    eprintln!("\nEngine Final: screen='{}'", screen);
    eprintln!("Expected: 'sax '");
}
