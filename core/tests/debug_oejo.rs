use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::{is_buffer_invalid_vietnamese, Processor};
use gonhanh_core::engine::Engine;

#[test]
fn debug_oejo() {
    // Test "oejo" - o+e+j+o
    let keys_seq = [keys::O, keys::E, keys::J, keys::O];

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
        "is_buffer_invalid_vietnamese('{}') = {}",
        buf,
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
    eprintln!("screen before space='{}'", screen);
    // Space to commit
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
}
