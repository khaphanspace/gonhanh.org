use gonhanh_core::engine::Engine;

fn char_to_key(c: char) -> u16 {
    match c.to_ascii_lowercase() {
        'a' => 0,
        's' => 1,
        'd' => 2,
        'f' => 3,
        'h' => 4,
        'g' => 5,
        'z' => 6,
        'x' => 7,
        'c' => 8,
        'v' => 9,
        'b' => 11,
        'q' => 12,
        'w' => 13,
        'e' => 14,
        'r' => 15,
        'y' => 16,
        't' => 17,
        'o' => 31,
        'u' => 32,
        'i' => 34,
        'p' => 35,
        'l' => 37,
        'j' => 38,
        'k' => 40,
        'n' => 45,
        'm' => 46,
        _ => 255,
    }
}

#[test]
fn trace_deeper_keystroke() {
    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_english_auto_restore(true);

    let word = "deeper";
    let mut output = String::new();

    eprintln!("\n=== Tracing '{}' keystroke by keystroke ===\n", word);
    for (i, ch) in word.chars().enumerate() {
        let key = char_to_key(ch);
        let result = engine.on_key(key, false, false);

        eprintln!("Step {}: Key '{}' (code={})", i + 1, ch, key);
        eprintln!("  Action: {} (0=None, 1=Send)", result.action);
        eprintln!("  Backspace: {}", result.backspace);
        eprintln!("  Count: {}", result.count);

        if result.action == 1 {
            let chars_str: String = (0..result.count as usize)
                .filter_map(|j| char::from_u32(result.chars[j]))
                .collect();
            eprintln!("  Chars: '{}'", chars_str);

            for _ in 0..result.backspace.min(output.len() as u8) {
                output.pop();
            }
            for j in 0..result.count as usize {
                if let Some(c) = char::from_u32(result.chars[j]) {
                    output.push(c);
                }
            }
        } else {
            output.push(ch);
        }
        eprintln!("  Screen after: '{}' (len={})\n", output, output.len());
    }

    eprintln!("\n=== Typing SPACE ===\n");
    let result = engine.on_key(49, false, false); // SPACE
    eprintln!("Space result:");
    eprintln!("  Action: {}", result.action);
    eprintln!("  Backspace: {}", result.backspace);
    eprintln!("  Count: {}", result.count);

    if result.action == 1 {
        let chars_str: String = (0..result.count as usize)
            .filter_map(|j| char::from_u32(result.chars[j]))
            .collect();
        eprintln!("  Chars: '{}'", chars_str);

        for _ in 0..result.backspace.min(output.len() as u8) {
            output.pop();
        }
        for j in 0..result.count as usize {
            if let Some(c) = char::from_u32(result.chars[j]) {
                output.push(c);
            }
        }
    } else {
        output.push(' ');
    }

    eprintln!("\nFinal output: '{}'", output);
    eprintln!("Expected:     'deeper '");

    assert_eq!(output, "deeper ", "deeper should restore to 'deeper '");
}
