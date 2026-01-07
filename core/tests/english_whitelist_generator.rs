//! English Whitelist Generator
//!
//! Script để tìm ra những từ tiếng Anh cần whitelist cho auto-restore.
//! Logic: Với mỗi từ trong danh sách 100k English words:
//! 1. Gõ word + space với auto_restore enabled
//! 2. Nếu output != "word " → từ đó bị transform sai → cần whitelist
//!
//! Usage: cargo test --test english_whitelist_generator -- --nocapture

use gonhanh_core::engine::Engine;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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

/// Simulate typing word + space, return actual output
fn type_word_space(engine: &mut Engine, word: &str) -> String {
    engine.clear();
    let mut output = String::new();

    // Type each character
    for ch in word.chars() {
        let key = char_to_key(ch);
        if key == 255 {
            // Non-letter character (hyphen, apostrophe, etc.) - skip word
            return String::new();
        }

        let result = engine.on_key(key, ch.is_uppercase(), false);

        if result.action == 1 {
            // Action::Send
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

/// Check if word needs whitelist (output != expected)
fn needs_whitelist(engine: &mut Engine, word: &str) -> Option<(String, String)> {
    let output = type_word_space(engine, word);
    if output.is_empty() {
        return None; // Skip words with special chars
    }

    let expected = format!("{} ", word);
    if output != expected {
        Some((word.to_string(), output))
    } else {
        None
    }
}

#[test]
#[ignore] // Run manually: cargo test --test english_whitelist_generator generate_whitelist -- --ignored --nocapture
fn generate_whitelist() {
    let wordlist_path = "tests/data/english_100k.txt";

    // Check if wordlist exists
    let file = match File::open(wordlist_path) {
        Ok(f) => f,
        Err(_) => {
            println!("ERROR: Wordlist not found at {}", wordlist_path);
            println!("Download from: https://raw.githubusercontent.com/david47k/top-english-wordlists/master/top_english_words_lower_100000.txt");
            println!("Save to: core/tests/data/english_100k.txt");
            return;
        }
    };

    let reader = BufReader::new(file);
    let words: Vec<String> = reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|w| w.len() >= 2 && w.chars().all(|c| c.is_ascii_lowercase()))
        .collect();

    println!("Loaded {} words from wordlist", words.len());

    let mut engine = Engine::new();
    engine.set_method(0); // Telex
    engine.set_english_auto_restore(true);

    let mut whitelist: Vec<(String, String)> = Vec::new();
    let mut correct_count = 0;

    for (i, word) in words.iter().enumerate() {
        if let Some((w, output)) = needs_whitelist(&mut engine, word) {
            whitelist.push((w, output));
        } else {
            correct_count += 1;
        }

        // Progress
        if (i + 1) % 10000 == 0 {
            println!(
                "Progress: {}/{} ({} need whitelist)",
                i + 1,
                words.len(),
                whitelist.len()
            );
        }
    }

    println!("\n========== RESULTS ==========");
    println!("Total words tested: {}", words.len());
    println!("Correctly restored: {}", correct_count);
    println!("Need whitelist: {}", whitelist.len());
    println!(
        "Accuracy: {:.2}%",
        (correct_count as f64 / words.len() as f64) * 100.0
    );

    // Write whitelist to file
    let output_path = "tests/data/english_whitelist.txt";
    let mut output_file = File::create(output_path).expect("Failed to create output file");

    writeln!(
        output_file,
        "// English words that need whitelist for auto-restore"
    )
    .unwrap();
    writeln!(
        output_file,
        "// Format: word → actual_output (what engine produced)"
    )
    .unwrap();
    writeln!(output_file, "// Total: {} words\n", whitelist.len()).unwrap();

    for (word, output) in &whitelist {
        writeln!(output_file, "{} → {}", word, output.trim()).unwrap();
    }

    println!("\nWhitelist saved to: {}", output_path);

    // Also create a simple word-only list
    let simple_path = "tests/data/english_whitelist_words.txt";
    let mut simple_file = File::create(simple_path).expect("Failed to create simple list");

    for (word, _) in &whitelist {
        writeln!(simple_file, "{}", word).unwrap();
    }

    println!("Simple word list saved to: {}", simple_path);

    // Print sample of problematic words
    println!("\n========== SAMPLE (first 50) ==========");
    for (word, output) in whitelist.iter().take(50) {
        println!("  {} → {}", word, output.trim());
    }
}

#[test]
#[ignore]
fn analyze_whitelist_patterns() {
    // After generating whitelist, analyze patterns to find common issues
    let whitelist_path = "tests/data/english_whitelist.txt";

    let file = match File::open(whitelist_path) {
        Ok(f) => f,
        Err(_) => {
            println!("Run generate_whitelist first!");
            return;
        }
    };

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| !l.starts_with("//") && !l.is_empty())
        .collect();

    println!("Analyzing {} problematic words...\n", lines.len());

    // Categorize by pattern
    let mut pattern_counts: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for line in &lines {
        if let Some((word, _)) = line.split_once(" → ") {
            let word = word.trim();

            // Detect patterns
            let pattern = if word.contains("ss") {
                "double-s"
            } else if word.contains("ff") {
                "double-f"
            } else if word.contains("rr") {
                "double-r"
            } else if word.contains("xx") {
                "double-x"
            } else if word.contains("aw") {
                "aw-pattern"
            } else if word.contains("ew") {
                "ew-pattern"
            } else if word.contains("ow") {
                "ow-pattern"
            } else if word.contains("aa") {
                "double-a"
            } else if word.contains("ee") {
                "double-e"
            } else if word.contains("oo") {
                "double-o"
            } else if word.ends_with("er") {
                "ends-er"
            } else if word.ends_with("es") {
                "ends-es"
            } else if word.ends_with("ed") {
                "ends-ed"
            } else {
                "other"
            };

            pattern_counts
                .entry(pattern.to_string())
                .or_default()
                .push(word.to_string());
        }
    }

    println!("========== PATTERN ANALYSIS ==========");
    let mut patterns: Vec<_> = pattern_counts.iter().collect();
    patterns.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    for (pattern, words) in patterns {
        println!("\n{} ({} words):", pattern, words.len());
        for w in words.iter().take(10) {
            println!("  - {}", w);
        }
        if words.len() > 10 {
            println!("  ... and {} more", words.len() - 10);
        }
    }
}
