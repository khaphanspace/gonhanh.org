use gonhanh_core::engine::Engine;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

fn type_word_space(engine: &mut Engine, word: &str) -> String {
    engine.clear();
    let mut output = String::new();
    for ch in word.chars() {
        let key = char_to_key(ch);
        if key == 255 {
            return String::new();
        }
        let result = engine.on_key(key, ch.is_uppercase(), false);
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
    }
    let result = engine.on_key(49, false, false);
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
    output
}

#[test]
#[ignore]
fn find_whitelist_diff() {
    let file = File::open("tests/data/english_100k.txt").unwrap();
    let reader = BufReader::new(file);
    let words: Vec<String> = reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|w| w.len() >= 2 && w.chars().all(|c| c.is_ascii_lowercase()))
        .collect();

    let mut diff_words = Vec::new();

    for word in &words {
        let mut e1 = Engine::new();
        e1.set_english_auto_restore(false);
        let out1 = type_word_space(&mut e1, word);

        let mut e2 = Engine::new();
        e2.set_english_auto_restore(true);
        let out2 = type_word_space(&mut e2, word);

        let expected = format!("{} ", word);

        if out1 != expected && out2 == expected {
            diff_words.push((word.clone(), out1.trim().to_string()));
        }
    }

    println!(
        "\n========== WHITELIST DIFF ({} words) ==========\n",
        diff_words.len()
    );
    for (word, wrong_output) in &diff_words {
        println!("  {} → {} (restored to {})", word, wrong_output, word);
    }
}
