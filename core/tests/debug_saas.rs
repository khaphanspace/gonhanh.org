use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::{
    english_likelihood_keys, is_buffer_invalid_vietnamese, is_foreign_pattern_keys, Processor,
};
use gonhanh_core::engine::Engine;

#[test]
fn debug_saas() {
    // Test "saas" - s+a+a+s → "sâ" (circumflex) + s (sắc) = "sấ"
    let keys_seq = [keys::S, keys::A, keys::A, keys::S];

    eprintln!("--- Validation ---");
    eprintln!("is_foreign_pattern: {}", is_foreign_pattern_keys(&keys_seq));
    eprintln!(
        "english_likelihood: {:?}",
        english_likelihood_keys(&keys_seq)
    );

    // Test with processor
    let mut processor = Processor::new();
    for &k in &keys_seq {
        processor.process(k, false, false);
    }
    let buf = processor.buffer().to_full_string();
    let raw: String = processor.raw().restore_all().iter().collect();
    eprintln!("\nProcessor: buffer='{}', raw='{}'", buf, raw);
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
        eprintln!(
            "Key {}: backspace={}, count={}, action={}",
            k, r.backspace, r.count, r.action
        );
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
    let r = engine.on_key(keys::SPACE, false, false);
    for _ in 0..r.backspace {
        screen.pop();
    }
    for i in 0..r.count as usize {
        if let Some(c) = char::from_u32(r.chars[i]) {
            screen.push(c);
        }
    }
    eprintln!("\nFinal: screen='{}'", screen);
    eprintln!("Expected: 'saas '");
}
