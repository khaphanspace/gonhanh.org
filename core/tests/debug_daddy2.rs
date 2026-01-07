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

fn type_word_debug(engine: &mut Engine, word: &str) -> String {
    engine.clear();
    let mut output = String::new();
    println!("=== Typing '{}' ===", word);
    for (idx, ch) in word.chars().enumerate() {
        let key = char_to_key(ch);
        if key == 255 {
            output.push(ch);
            continue;
        }
        let result = engine.on_key(key, false, false);
        println!(
            "  [{}] '{}' key={} -> action={} bs={} count={}",
            idx, ch, key, result.action, result.backspace, result.count
        );
        if result.action == 1 {
            for _ in 0..(result.backspace as usize).min(output.len()) {
                output.pop();
            }
            for i in 0..result.count as usize {
                if let Some(c) = char::from_u32(result.chars[i]) {
                    output.push(c);
                }
            }
        } else {
            output.push(ch);
        }
        println!("      output so far: '{}'", output);
    }
    // space
    println!("  [SPACE]");
    let result = engine.on_key(49, false, false);
    println!(
        "    -> action={} bs={} count={}",
        result.action, result.backspace, result.count
    );
    if result.action == 1 {
        for _ in 0..(result.backspace as usize).min(output.len()) {
            output.pop();
        }
        for i in 0..result.count as usize {
            if let Some(c) = char::from_u32(result.chars[i]) {
                output.push(c);
            }
        }
    } else {
        output.push(' ');
    }
    println!("  FINAL: '{}'", output);
    output
}

#[test]
fn debug_daddy_step() {
    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_english_auto_restore(true);

    type_word_debug(&mut engine, "daddy");
    println!("");
    type_word_debug(&mut engine, "poor");
}
