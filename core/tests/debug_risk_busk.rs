use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::{english_likelihood_keys, is_foreign_pattern_keys, Processor};
use gonhanh_core::engine::Engine;

#[test]
fn debug_risk_busk() {
    // Test "risk" vs "Busk"
    let risk_keys = [keys::R, keys::I, keys::S, keys::K];
    // Use SHIFT+B for capitalized Busk
    let busk_keys = [keys::B, keys::U, keys::S, keys::K];

    // Also test lowercase busk
    let lowercase_busk_keys = [keys::B, keys::U, keys::S, keys::K];

    eprintln!("--- Validation functions ---");
    eprintln!(
        "'risk' is_foreign_pattern: {}",
        is_foreign_pattern_keys(&risk_keys)
    );
    eprintln!(
        "'risk' english_likelihood: {:?}",
        english_likelihood_keys(&risk_keys)
    );
    eprintln!(
        "'busk' is_foreign_pattern: {}",
        is_foreign_pattern_keys(&busk_keys)
    );
    eprintln!(
        "'busk' english_likelihood: {:?}",
        english_likelihood_keys(&busk_keys)
    );

    // Test with processor
    eprintln!("\n--- Processor results ---");
    let mut p1 = Processor::new();
    for &k in &risk_keys {
        p1.process(k, false, false);
    }
    let buf1 = p1.buffer().to_full_string();
    let raw1: String = p1.raw().restore_all().iter().collect();
    eprintln!("'risk': buffer='{}', raw='{}'", buf1, raw1);

    let mut p2 = Processor::new();
    for &k in &busk_keys {
        p2.process(k, false, false);
    }
    let buf2 = p2.buffer().to_full_string();
    let raw2: String = p2.raw().restore_all().iter().collect();
    eprintln!("'busk': buffer='{}', raw='{}'", buf2, raw2);

    // Test with engine
    eprintln!("\n--- Engine results ---");
    let mut engine1 = Engine::new();
    engine1.set_english_auto_restore(true);
    let mut screen1 = String::new();
    for &k in &risk_keys {
        let r = engine1.on_key(k, false, false);
        for _ in 0..r.backspace {
            screen1.pop();
        }
        for i in 0..r.count as usize {
            if let Some(c) = char::from_u32(r.chars[i]) {
                screen1.push(c);
            }
        }
    }
    // Space to commit
    let r = engine1.on_key(keys::SPACE, false, false);
    for _ in 0..r.backspace {
        screen1.pop();
    }
    for i in 0..r.count as usize {
        if let Some(c) = char::from_u32(r.chars[i]) {
            screen1.push(c);
        }
    }
    eprintln!("'risk ': screen='{}'", screen1);

    let mut engine2 = Engine::new();
    engine2.set_english_auto_restore(true);
    let mut screen2 = String::new();
    for &k in &busk_keys {
        let r = engine2.on_key(k, false, false);
        for _ in 0..r.backspace {
            screen2.pop();
        }
        for i in 0..r.count as usize {
            if let Some(c) = char::from_u32(r.chars[i]) {
                screen2.push(c);
            }
        }
    }
    // Space to commit
    let r = engine2.on_key(keys::SPACE, false, false);
    for _ in 0..r.backspace {
        screen2.pop();
    }
    for i in 0..r.count as usize {
        if let Some(c) = char::from_u32(r.chars[i]) {
            screen2.push(c);
        }
    }
    eprintln!("'busk ': screen='{}'", screen2);
}
