use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::{english_likelihood_keys, is_foreign_pattern_keys, Processor};
use gonhanh_core::engine::Engine;

#[test]
fn debug_effect() {
    // Test what happens when typing "effect "
    // Expected: "effect " (English word with double F)
    let key_seq = [keys::E, keys::F, keys::F, keys::E, keys::C, keys::T];

    // Test with raw processor first
    let mut processor = Processor::new();
    for &key in &key_seq {
        processor.process(key, false, false);
    }
    let raw = processor.raw().restore_all();
    let raw_str: String = raw.iter().collect();
    let buffer = processor.buffer().to_full_string();
    eprintln!(
        "[Processor] raw='{}' ({} chars), buffer='{}' ({} chars)",
        raw_str,
        raw_str.len(),
        buffer,
        buffer.len()
    );

    // Now test with engine
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    let mut screen = String::new();

    for (i, &key) in key_seq.iter().enumerate() {
        let result = engine.on_key(key, false, false);
        let output: String = result
            .chars
            .iter()
            .take(result.count as usize)
            .filter_map(|&c| char::from_u32(c))
            .collect();

        // Apply backspaces
        for _ in 0..result.backspace {
            screen.pop();
        }
        screen.push_str(&output);

        eprintln!(
            "[{}] Key {}: backspace={}, action={}, output='{}' → screen='{}'",
            i, key, result.backspace, result.action, output, screen
        );
    }

    // Now SPACE
    let result = engine.on_key(keys::SPACE, false, false);
    let output: String = result
        .chars
        .iter()
        .take(result.count as usize)
        .filter_map(|&c| char::from_u32(c))
        .collect();
    for _ in 0..result.backspace {
        screen.pop();
    }
    screen.push_str(&output);
    eprintln!(
        "[SPACE] backspace={}, action={}, output='{}' → screen='{}'",
        result.backspace, result.action, output, screen
    );

    eprintln!("\nFinal screen: '{}'", screen);
    eprintln!("Expected: 'effect '");

    // Also check the validation functions directly
    let all_keys = vec![keys::E, keys::F, keys::F, keys::E, keys::C, keys::T];
    let is_foreign = is_foreign_pattern_keys(&all_keys);
    let en = english_likelihood_keys(&all_keys);
    eprintln!(
        "\n'effect' validation: foreign={}, EN(R)={:?}",
        is_foreign, en
    );

    assert_eq!(screen, "effect ", "Effect should preserve double F");
}
