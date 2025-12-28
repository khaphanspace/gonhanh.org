use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

/// Simulate typing and collect final output like the test utility does
fn type_and_get_output(key_seq: &[u16]) -> String {
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    let mut screen = String::new();

    for &key in key_seq {
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

        // Add output
        screen.push_str(&output);

        // If action is NORMAL (0), add the key character directly (for non-modifier keys)
        if result.action == 0 && result.count == 0 && result.backspace == 0 {
            // This is a buffered key, need to add it to screen
            if let Some(c) = char::from_u32(key as u32 + 'a' as u32) {
                // This isn't quite right but let's see what happens
            }
        }

        eprintln!(
            "Key {}: bs={}, action={}, count={}, output='{}' → screen='{}'",
            key, result.backspace, result.action, result.count, output, screen
        );
    }

    screen
}

#[test]
fn debug_caaps_full() {
    let key_seq = [keys::C, keys::A, keys::A, keys::P, keys::S, keys::SPACE];

    let result = type_and_get_output(&key_seq);
    eprintln!("\nFinal screen: '{}'", result);
    eprintln!("Expected: 'cấp '");
}
