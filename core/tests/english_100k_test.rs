//! English 100k Words Test
//!
//! Tests all 100k English words to find which ones don't output correctly
//! when typed with Vietnamese IME (Telex) + space.

use gonhanh_core::engine::Engine;
use std::fs;

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
        '1' => 18,
        '2' => 19,
        '3' => 20,
        '4' => 21,
        '6' => 22,
        '5' => 23,
        '9' => 25,
        '7' => 26,
        '8' => 28,
        '0' => 29,
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
    engine.clear();
    let mut output = String::new();

    // Type the word
    for ch in word.chars() {
        let key = char_to_key(ch);
        if key == 255 {
            // Unknown char, just add it
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
    }

    // Type space to trigger auto-restore
    let result = engine.on_key(49, false, false); // 49 = SPACE key
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

    output
}

#[test]
fn english_100k_failures() {
    let content =
        fs::read_to_string("tests/data/english_100k.txt").expect("Failed to read english_100k.txt");

    let words: Vec<&str> = content
        .lines()
        .filter(|line| {
            let w = line.trim();
            !w.is_empty() && w.chars().all(|c| c.is_ascii_alphabetic())
        })
        .collect();

    let mut engine = Engine::new();
    engine.set_method(0); // Telex
    engine.set_english_auto_restore(true);

    let mut failures: Vec<(String, String)> = Vec::new();

    for word in &words {
        let expected = format!("{} ", word);
        let actual = type_word_with_space(&mut engine, word);

        if actual != expected {
            failures.push((word.to_string(), actual));
        }
    }

    println!("\n=== ENGLISH 100K TEST RESULTS ===\n");
    println!("Total words tested: {}", words.len());
    println!("Failed words: {}", failures.len());
    println!(
        "Success rate: {:.2}%",
        (words.len() - failures.len()) as f64 / words.len() as f64 * 100.0
    );

    if !failures.is_empty() {
        println!("\n=== FAILED WORDS ({}) ===\n", failures.len());

        // Group by transformation type
        let mut by_output: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (word, output) in &failures {
            by_output
                .entry(output.clone())
                .or_default()
                .push(word.clone());
        }

        // Show first 100 failures with their outputs
        println!("{:<20} {:<20}", "WORD", "OUTPUT");
        println!("{}", "-".repeat(40));
        for (word, output) in failures.iter().take(100) {
            let display_output = output.replace('\n', "\\n").replace('\r', "\\r");
            println!("{:<20} {:<20}", word, display_output);
        }

        if failures.len() > 100 {
            println!("\n... and {} more failures", failures.len() - 100);
        }

        // Write full list to file
        let failure_list: Vec<String> = failures
            .iter()
            .map(|(word, output)| format!("{}\t{}", word, output.trim()))
            .collect();
        fs::write(
            "tests/data/english_100k_failures.txt",
            failure_list.join("\n"),
        )
        .expect("Failed to write failures file");
        println!("\nFull list written to: tests/data/english_100k_failures.txt");
    }

    // CI threshold: fail if pass rate drops below 97%
    let pass_rate = (words.len() - failures.len()) as f64 / words.len() as f64 * 100.0;
    const MIN_PASS_RATE: f64 = 97.0;
    assert!(
        pass_rate >= MIN_PASS_RATE,
        "English 100k pass rate {:.2}% is below threshold {:.1}%",
        pass_rate,
        MIN_PASS_RATE
    );
}
