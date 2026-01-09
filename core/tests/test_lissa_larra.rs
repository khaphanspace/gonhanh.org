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

fn type_word_with_space(engine: &mut Engine, word: &str) -> String {
    type_word_with_space_debug(engine, word, false)
}

fn type_word_with_space_debug(engine: &mut Engine, word: &str, debug: bool) -> String {
    engine.clear();
    let mut output = String::new();

    for ch in word.chars() {
        let key = char_to_key(ch);
        if key == 255 {
            output.push(ch);
            continue;
        }
        let result = engine.on_key(key, ch.is_uppercase(), false);
        if result.action == 1 {
            let bs = result.backspace as usize;
            for _ in 0..bs.min(output.len()) {
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
        if debug {
            println!("  after '{}': output = '{}'", ch, output);
        }
    }

    // Type space
    let result = engine.on_key(49, false, false);
    if result.action == 1 {
        let bs = result.backspace as usize;
        for _ in 0..bs.min(output.len()) {
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
    if debug {
        println!("  after space: output = '{}'", output);
    }
    output
}

#[test]
fn test_lissa_larra() {
    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_enabled(true);
    engine.set_english_auto_restore(true);

    let result1 = type_word_with_space(&mut engine, "larra");
    println!("larra + space → '{}'", result1);

    let result2 = type_word_with_space(&mut engine, "lissa");
    println!("lissa + space → '{}'", result2);

    let result3 = type_word_with_space(&mut engine, "melissa");
    println!("melissa + space → '{}'", result3);

    println!("\n=== DEBUG larissa ===");
    let result4 = type_word_with_space_debug(&mut engine, "larissa", true);
    println!("larissa + space → '{}'", result4);

    // Test words NOT in dict to verify collapse behavior
    let result5 = type_word_with_space(&mut engine, "xyzss");
    println!("xyzss + space → '{}'", result5);

    let result6 = type_word_with_space(&mut engine, "abcrr");
    println!("abcrr + space → '{}'", result6);

    println!("\n=== DEBUG grass ===");
    let result7 = type_word_with_space_debug(&mut engine, "grass", true);
    println!("grass + space → '{}'", result7);

    // Assertions - New behavior: if buffer collapsed form is in dict, keep it
    // larra NOT in dict → collapse to lara
    assert_eq!(result1.trim(), "lara", "larra should collapse to lara");
    // lissa → lisa: buffer "lisa" IN dict → keep "lisa"
    assert_eq!(
        result2.trim(),
        "lisa",
        "lissa should collapse to lisa (in dict)"
    );
    assert_eq!(result3.trim(), "melissa", "melissa should restore");
    assert_eq!(result4.trim(), "larissa", "larissa should restore");
}

#[test]
fn test_lissa_debug() {
    use gonhanh_core::data::english_dict;
    use gonhanh_core::data::keys;
    use gonhanh_core::engine::validation;

    // Check dictionary status
    println!("=== Dictionary Status ===");
    println!(
        "lissa in english_dict: {}",
        english_dict::is_english_word("lissa")
    );
    println!(
        "grass in english_dict: {}",
        english_dict::is_english_word("grass")
    );
    println!(
        "gras in english_dict: {}",
        english_dict::is_english_word("gras")
    );

    // Check validation for buffer strings
    println!("\n=== Validation Status ===");
    // gras = G R A S
    let gras_keys = vec![keys::G, keys::R, keys::A, keys::S];
    let gras_tones = vec![0u8, 0, 0, 0];
    println!(
        "gras is valid VN: {}",
        validation::is_valid_with_tones(&gras_keys, &gras_tones)
    );
}
