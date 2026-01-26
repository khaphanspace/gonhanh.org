//! English 100k Words Test
//!
//! Tests all 100k English words to find which ones don't output correctly
//! when typed with Vietnamese IME (Telex) + space.
//!
//! Failures are categorized by cause:
//! - Tone: words with tone markers (s/f/r/x/j) at end
//! - Vowel: words with vowel patterns (aa/ee/oo/aw/ow/uw/dd)

use gonhanh_core::engine::Engine;
use std::fs;

// =============================================================================
// PATTERN DETECTION
// =============================================================================

/// Tone markers in Telex (at end of word triggers tone)
const TONE_MARKERS: [char; 5] = ['s', 'f', 'r', 'x', 'j'];

/// Vowel/consonant patterns that trigger transforms
const VOWEL_PATTERNS: [&str; 7] = ["aa", "ee", "oo", "aw", "ow", "uw", "dd"];

/// Check if word has tone marker pattern (ends with s/f/r/x/j after vowel)
fn has_tone_pattern(word: &str) -> bool {
    let lower = word.to_lowercase();
    if let Some(last) = lower.chars().last() {
        TONE_MARKERS.contains(&last)
    } else {
        false
    }
}

/// Check if word has vowel/double pattern
fn has_vowel_pattern(word: &str) -> bool {
    let lower = word.to_lowercase();
    VOWEL_PATTERNS.iter().any(|p| lower.contains(p))
}

/// Categorize failure cause
#[derive(Debug, Clone, PartialEq)]
enum FailureCause {
    Tone,    // Tone marker caused transform
    Vowel,   // Vowel pattern caused transform
    Both,    // Both tone and vowel patterns
    Unknown, // Neither (shouldn't happen often)
}

fn categorize_failure(word: &str) -> FailureCause {
    let has_tone = has_tone_pattern(word);
    let has_vowel = has_vowel_pattern(word);

    match (has_tone, has_vowel) {
        (true, true) => FailureCause::Both,
        (true, false) => FailureCause::Tone,
        (false, true) => FailureCause::Vowel,
        (false, false) => FailureCause::Unknown,
    }
}

// =============================================================================
// KEY SIMULATION
// =============================================================================

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

/// Returns (final_output, buffer_before_space)
fn type_word_with_space(engine: &mut Engine, word: &str) -> (String, String) {
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
    }

    // Save buffer state before space (Vietnamese transform state)
    let buffer = output.clone();

    // Type space to trigger auto-restore
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

    (output, buffer)
}

// =============================================================================
// TEST
// =============================================================================

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

    // Categorized failures: (word, actual_output, buffer)
    let mut tone_failures: Vec<(String, String, String)> = Vec::new();
    let mut vowel_failures: Vec<(String, String, String)> = Vec::new();
    let mut both_failures: Vec<(String, String, String)> = Vec::new();
    let mut unknown_failures: Vec<(String, String, String)> = Vec::new();

    for word in &words {
        let expected = format!("{} ", word);
        let (actual, buffer) = type_word_with_space(&mut engine, word);

        if actual != expected {
            let failure = (word.to_string(), actual.trim().to_string(), buffer);
            match categorize_failure(word) {
                FailureCause::Tone => tone_failures.push(failure),
                FailureCause::Vowel => vowel_failures.push(failure),
                FailureCause::Both => both_failures.push(failure),
                FailureCause::Unknown => unknown_failures.push(failure),
            }
        }
    }

    let total_failures =
        tone_failures.len() + vowel_failures.len() + both_failures.len() + unknown_failures.len();

    // Print summary table
    let pass_rate = (words.len() - total_failures) as f64 / words.len() as f64 * 100.0;
    println!("\n┌─────────────────────────────────────────┐");
    println!("│         ENGLISH 100K RESULTS            │");
    println!("├─────────────────────────────────────────┤");
    println!("│ Total words     │ {:>20} │", words.len());
    println!("│ Passed          │ {:>20} │", words.len() - total_failures);
    println!("│ Failed          │ {:>20} │", total_failures);
    println!("│ Pass rate       │ {:>19.2}% │", pass_rate);
    println!("├─────────────────────────────────────────┤");
    println!("│ BY CAUSE                                │");
    println!("├─────────────────────────────────────────┤");
    println!("│ Tone (s/f/r/x/j)│ {:>20} │", tone_failures.len());
    println!("│ Vowel (aa/ee..) │ {:>20} │", vowel_failures.len());
    println!("│ Both            │ {:>20} │", both_failures.len());
    println!("│ Unknown         │ {:>20} │", unknown_failures.len());
    println!("└─────────────────────────────────────────┘");

    // Show samples
    if !tone_failures.is_empty() {
        println!("\n--- Tone Failures (first 20) ---");
        println!("{:<15} {:<15} {:<15}", "WORD", "ACTUAL", "BUFFER");
        for (word, actual, buffer) in tone_failures.iter().take(20) {
            println!("{:<15} {:<15} {:<15}", word, actual, buffer);
        }
    }
    if !vowel_failures.is_empty() {
        println!("\n--- Vowel Failures (first 20) ---");
        println!("{:<15} {:<15} {:<15}", "WORD", "ACTUAL", "BUFFER");
        for (word, actual, buffer) in vowel_failures.iter().take(20) {
            println!("{:<15} {:<15} {:<15}", word, actual, buffer);
        }
    }

    // Write to separate files
    use std::io::Write;

    // Tone failures file
    if let Ok(mut f) = std::fs::File::create("tests/data/english_100k_failures_tone.txt") {
        writeln!(f, "# English 100k Failures - Tone Markers").ok();
        writeln!(f, "# Cause: words ending with s/f/r/x/j trigger tone marks").ok();
        writeln!(f, "# Format: WORD \\t ACTUAL \\t BUFFER").ok();
        writeln!(
            f,
            "# Total: {} (+ {} both)",
            tone_failures.len(),
            both_failures.len()
        )
        .ok();
        writeln!(f, "#").ok();
        writeln!(f, "# WORD: English word typed").ok();
        writeln!(f, "# ACTUAL: engine output after space").ok();
        writeln!(f, "# BUFFER: Vietnamese transform state before space").ok();
        writeln!(f).ok();
        for (word, actual, buffer) in &tone_failures {
            writeln!(f, "{}\t{}\t{}", word, actual, buffer).ok();
        }
        for (word, actual, buffer) in &both_failures {
            writeln!(f, "{}\t{}\t{}", word, actual, buffer).ok();
        }
    }

    // Vowel failures file
    if let Ok(mut f) = std::fs::File::create("tests/data/english_100k_failures_vowel.txt") {
        writeln!(f, "# English 100k Failures - Vowel Patterns").ok();
        writeln!(f, "# Cause: aa/ee/oo/aw/ow/uw/dd trigger vowel transforms").ok();
        writeln!(f, "# Format: WORD \\t ACTUAL \\t BUFFER").ok();
        writeln!(
            f,
            "# Total: {} (+ {} both)",
            vowel_failures.len(),
            both_failures.len()
        )
        .ok();
        writeln!(f, "#").ok();
        writeln!(f, "# WORD: English word typed").ok();
        writeln!(f, "# ACTUAL: engine output after space").ok();
        writeln!(f, "# BUFFER: Vietnamese transform state before space").ok();
        writeln!(f).ok();
        for (word, actual, buffer) in &vowel_failures {
            writeln!(f, "{}\t{}\t{}", word, actual, buffer).ok();
        }
        for (word, actual, buffer) in &both_failures {
            writeln!(f, "{}\t{}\t{}", word, actual, buffer).ok();
        }
    }

    println!("\nWritten to:");
    println!("  - tests/data/english_100k_failures_tone.txt");
    println!("  - tests/data/english_100k_failures_vowel.txt");

    // CI threshold
    let pass_rate = (words.len() - total_failures) as f64 / words.len() as f64 * 100.0;
    const MIN_PASS_RATE: f64 = 97.0;
    assert!(
        pass_rate >= MIN_PASS_RATE,
        "English 100k pass rate {:.2}% is below threshold {:.0}%",
        pass_rate,
        MIN_PASS_RATE
    );
}
