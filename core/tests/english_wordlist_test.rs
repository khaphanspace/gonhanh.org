//! Test English word auto-restore using Google 20k word list.
//!
//! This test suite validates that common English words are correctly
//! restored when typed using Telex input method with auto-restore enabled.
//!
//! Word list source: https://github.com/first20hours/google-10000-english (20k.txt)

mod common;
use common::type_word;
use gonhanh_core::engine::Engine;
use std::fs;
use std::io::{BufRead, BufReader};

// =============================================================================
// TEST UTILITIES
// =============================================================================

/// Check if a word restores correctly with auto-restore enabled
fn check_restore(word: &str) -> Option<(String, String)> {
    let input = format!("{} ", word);
    let mut eng = Engine::new();
    eng.set_english_auto_restore(true);
    let result = type_word(&mut eng, &input);
    let expected = format!("{} ", word);
    if result != expected {
        Some((result.trim().to_string(), expected.trim().to_string()))
    } else {
        None
    }
}

/// Check multiple words and return failures
fn check_words(words: &[&str]) -> Vec<(String, String, String)> {
    words
        .iter()
        .filter_map(|word| {
            check_restore(word).map(|(got, expected)| (word.to_string(), got, expected))
        })
        .collect()
}

/// Load words from file
fn load_words_from_file(path: &str) -> Vec<String> {
    let file = fs::File::open(path).expect("Failed to open word list file");
    BufReader::new(file)
        .lines()
        .filter_map(|line| line.ok())
        .filter(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_alphabetic()))
        .collect()
}

// =============================================================================
// KNOWN FAILURE PATTERNS - Words that currently fail auto-restore
// These are tracked for fixing
// =============================================================================

/// Words with V+r+V pattern (vowel + r + vowel) that trigger circumflex
const PATTERN_VRV: &[&str] = &[
    "param",
    "there",
    "here",
    "where",
    "were",
    "parent",
    "area",
    "camera",
    "separate",
    "preparation",
    "opera",
    "elaborate",
    "moderate",
    "generate",
];

/// Words with oe/ue diphthong + s (sắc mark)
const PATTERN_OES: &[&str] = &["goes", "does", "shoes", "toes", "hoes", "foes"];

/// Words with double modifier that reverts mark
const PATTERN_DOUBLE_MOD: &[&str] = &[
    "guess", "less", "pass", "class", "glass", "grass", "mass", "brass", "cross", "loss", "boss",
    "moss", "toss", "fuss", "buss", "muss",
];

/// Words with vowel+s+consonant pattern
const PATTERN_VSC: &[&str] = &[
    "list", "last", "most", "post", "best", "test", "rest", "cost", "lost", "just", "must", "dust",
    "rust", "bust", "fast", "past", "cast", "vast", "east", "west", "nest", "pest", "vest", "fest",
    "zest", "mist", "fist",
];

// =============================================================================
// TESTS FOR KNOWN PATTERNS
// =============================================================================

#[test]
fn test_pattern_vrv_circumflex() {
    let failures = check_words(PATTERN_VRV);
    println!("\n=== V+r+V Pattern (circumflex) ===");
    println!("Failures: {}/{}", failures.len(), PATTERN_VRV.len());
    for (word, got, _) in &failures {
        println!("  '{}' → '{}'", word, got);
    }
    // Track but don't fail - these are known issues
}

#[test]
fn test_pattern_oes_diphthong() {
    let failures = check_words(PATTERN_OES);
    println!("\n=== OE diphthong + s Pattern ===");
    println!("Failures: {}/{}", failures.len(), PATTERN_OES.len());
    for (word, got, _) in &failures {
        println!("  '{}' → '{}'", word, got);
    }
}

#[test]
fn test_pattern_double_modifier() {
    let failures = check_words(PATTERN_DOUBLE_MOD);
    println!("\n=== Double Modifier Pattern ===");
    println!("Failures: {}/{}", failures.len(), PATTERN_DOUBLE_MOD.len());
    for (word, got, _) in &failures {
        println!("  '{}' → '{}'", word, got);
    }
}

#[test]
fn test_pattern_vsc() {
    let failures = check_words(PATTERN_VSC);
    println!("\n=== Vowel+S+Consonant Pattern ===");
    println!("Failures: {}/{}", failures.len(), PATTERN_VSC.len());
    for (word, got, _) in &failures {
        println!("  '{}' → '{}'", word, got);
    }
}

// =============================================================================
// WORDS THAT SHOULD PASS (already working)
// =============================================================================

#[test]
fn test_working_patterns() {
    let working_words = [
        // Modifier + consonant (x+consonant pattern works)
        "text",
        "next",
        "context",
        "expect",
        "export",
        "express",
        "extend",
        // Invalid Vietnamese patterns
        "their",
        "weird",
        "view",
        "review",
        // W-initial patterns
        "window",
        "water",
        "winter",
        "wonder",
        // Long words with clear English patterns
        "information",
        "different",
        "experience",
        "beautiful",
    ];

    let failures = check_words(&working_words);
    println!("\n=== Should-Pass Words ===");
    println!(
        "Passed: {}/{}",
        working_words.len() - failures.len(),
        working_words.len()
    );

    if !failures.is_empty() {
        println!("Unexpected failures:");
        for (word, got, expected) in &failures {
            println!("  '{}' → '{}' (expected: '{}')", word, got, expected);
        }
    }

    // These should all pass
    assert!(
        failures.is_empty(),
        "Expected all working words to pass, but {} failed",
        failures.len()
    );
}

// =============================================================================
// FULL WORD LIST TESTS
// =============================================================================

#[test]
fn test_english_wordlist() {
    let path = "tests/data/english-words.txt";
    if !std::path::Path::new(path).exists() {
        println!("Skipping: {} not found", path);
        return;
    }

    let words = load_words_from_file(path);
    let total = words.len();

    let mut failures: Vec<(String, String, String)> = Vec::new();

    for word in &words {
        if let Some((got, expected)) = check_restore(word) {
            failures.push((word.clone(), got, expected));
        }
    }

    let passed = total - failures.len();
    let pass_rate = (passed as f64 / total as f64) * 100.0;

    println!("\n=== English Word List Test (20k) ===");
    println!("Total words: {}", total);
    println!("Passed: {} ({:.1}%)", passed, pass_rate);
    println!("Failed: {} ({:.1}%)", failures.len(), 100.0 - pass_rate);

    // Show sample failures by pattern
    println!("\n--- Sample Failures (first 30) ---");
    for (word, got, _) in failures.iter().take(30) {
        println!("  '{}' → '{}'", word, got);
    }

    // Report but don't fail - this is informational
    println!("\n[INFO] This test tracks auto-restore coverage, not pass/fail");
}

#[test]
fn test_telex_conflict_words() {
    let path = "tests/data/telex-conflict-words.txt";
    if !std::path::Path::new(path).exists() {
        println!("Skipping: {} not found", path);
        return;
    }

    let words = load_words_from_file(path);
    let total = words.len();

    let mut failures: Vec<(String, String, String)> = Vec::new();

    for word in &words {
        if let Some((got, expected)) = check_restore(word) {
            failures.push((word.clone(), got, expected));
        }
    }

    let passed = total - failures.len();
    let pass_rate = (passed as f64 / total as f64) * 100.0;

    println!("\n=== Telex Conflict Words Test ===");
    println!("Total conflict words: {}", total);
    println!("Correctly restored: {} ({:.1}%)", passed, pass_rate);
    println!(
        "Failed to restore: {} ({:.1}%)",
        failures.len(),
        100.0 - pass_rate
    );

    // Categorize failures by pattern
    let vrv_fails: Vec<_> = failures
        .iter()
        .filter(|(w, _, _)| {
            let chars: Vec<char> = w.chars().collect();
            chars
                .windows(3)
                .any(|win| "aeiou".contains(win[0]) && win[1] == 'r' && "aeiou".contains(win[2]))
        })
        .collect();

    let oes_fails: Vec<_> = failures
        .iter()
        .filter(|(w, _, _)| w.ends_with("oes"))
        .collect();
    let ss_fails: Vec<_> = failures
        .iter()
        .filter(|(w, _, _)| w.ends_with("ss"))
        .collect();
    let st_fails: Vec<_> = failures
        .iter()
        .filter(|(w, _, _)| w.ends_with("st"))
        .collect();

    println!("\n--- Failure Categories ---");
    println!("  V+r+V pattern: {}", vrv_fails.len());
    println!("  *oes pattern: {}", oes_fails.len());
    println!("  *ss pattern: {}", ss_fails.len());
    println!("  *st pattern: {}", st_fails.len());
}

// =============================================================================
// PROGRAMMING TERMS
// =============================================================================

#[test]
fn test_programming_terms() {
    let programming_words = [
        // Common programming keywords
        "async",
        "await",
        "const",
        "class",
        "export",
        "import",
        "extends",
        "return",
        "function",
        "interface",
        "struct",
        "enum",
        "match",
        // Variable naming patterns
        "param",
        "args",
        "props",
        "state",
        "context",
        "result",
        "response",
        "request",
        "handler",
        "callback",
        "promise",
        "resolve",
        "reject",
        // Common method/function names
        "parse",
        "process",
        "create",
        "update",
        "delete",
        "insert",
        "select",
        "fetch",
        "post",
        "push",
        "merge",
        "filter",
        "reduce",
        "transform",
        // File/path related
        "path",
        "file",
        "buffer",
        "stream",
        "reader",
        "writer",
    ];

    let failures = check_words(&programming_words);

    println!("\n=== Programming Terms ===");
    println!(
        "Passed: {}/{}",
        programming_words.len() - failures.len(),
        programming_words.len()
    );

    if !failures.is_empty() {
        println!("\nFailures:");
        for (word, got, _) in &failures {
            println!("  '{}' → '{}'", word, got);
        }
    }
}

// =============================================================================
// EDGE CASES
// =============================================================================

#[test]
fn test_short_words() {
    // Short words are tricky - many form valid Vietnamese
    let short_words = [
        "a", "i", "is", "as", "or", "of", "if", "us", "we", "be", "he", "me", "so", "no", "do",
        "go", "to", "up", "an", "at", "by", "in", "on",
    ];

    let failures = check_words(&short_words);

    println!("\n=== Short Words (1-2 chars) ===");
    println!(
        "Passed: {}/{}",
        short_words.len() - failures.len(),
        short_words.len()
    );

    // Many short words can't be distinguished from Vietnamese
    // This is informational, not a test failure
    if !failures.is_empty() {
        println!("Cannot auto-restore (valid VN structure):");
        for (word, got, _) in &failures {
            println!("  '{}' → '{}'", word, got);
        }
    }
}

#[test]
fn test_mixed_case() {
    // Test that capitalization is preserved
    let cases = [
        ("Text", "Text"),
        ("TEXT", "TEXT"),
        ("Next", "Next"),
        ("NEXT", "NEXT"),
    ];

    println!("\n=== Mixed Case Words ===");
    for (input, expected) in cases {
        let input_with_space = format!("{} ", input);
        let mut eng = Engine::new();
        eng.set_english_auto_restore(true);
        let result = type_word(&mut eng, &input_with_space);
        let expected_with_space = format!("{} ", expected);

        if result == expected_with_space {
            println!("  '{}' → '{}' OK", input, result.trim());
        } else {
            println!(
                "  '{}' → '{}' FAIL (expected: '{}')",
                input,
                result.trim(),
                expected
            );
        }
    }
}
