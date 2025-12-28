use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::{is_buffer_invalid_vietnamese, Processor};
use gonhanh_core::engine::Engine;

#[test]
fn debug_saax() {
    // Test "saax" - s+a+a+x → should become "sax" (buffer after revert)
    let keys_seq = [keys::S, keys::A, keys::A, keys::X];

    // Test with processor
    let mut processor = Processor::new();
    for (i, &k) in keys_seq.iter().enumerate() {
        processor.process(k, false, false);
        let buf = processor.buffer().to_full_string();
        let raw: String = processor.raw().restore_all().iter().collect();
        eprintln!("[{}] Key {}: buffer='{}', raw='{}'", i, k, buf, raw);

        // Show unconsumed keys at each step
        let unconsumed: Vec<u16> = processor.raw().unconsumed_keys().collect();
        let unconsumed_chars: String = unconsumed
            .iter()
            .filter_map(|&k| {
                if k >= b'A' as u16 && k <= b'Z' as u16 {
                    Some((k as u8 + 32) as char)
                } else if k >= b'a' as u16 && k <= b'z' as u16 {
                    Some(k as u8 as char)
                } else {
                    None
                }
            })
            .collect();
        eprintln!(
            "    unconsumed_keys: {:?} → '{}'",
            unconsumed, unconsumed_chars
        );

        // Show consumed status for each key
        let consumed_status: Vec<(u16, bool)> = processor.raw().iter_with_consumed().collect();
        eprintln!("    consumed_status: {:?}", consumed_status);
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
        for _ in 0..r.backspace {
            screen.pop();
        }
        for i in 0..r.count as usize {
            if let Some(c) = char::from_u32(r.chars[i]) {
                screen.push(c);
            }
        }
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
    eprintln!("Expected: 'sax '");
}
