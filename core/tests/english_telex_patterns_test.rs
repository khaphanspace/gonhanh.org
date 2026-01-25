//! English Telex Patterns Test
//!
//! Tests English words that contain patterns triggering Vietnamese transforms.
//! Classifies words into:
//! - RESTORABLE: transformed buffer is invalid VN → auto-restore works
//! - NON-RESTORABLE: transformed buffer is valid VN → kept as VN (expected)
//!
//! Patterns that trigger VN transforms:
//! - aa → â (circumflex)
//! - ee → ê (circumflex)
//! - oo → ô (circumflex)
//! - aw → ă (breve)
//! - ow → ơ (horn)
//! - uw → ư (horn)
//! - dd → đ (stroke)
//!
//! Run with: cargo test --test english_telex_patterns_test -- --nocapture

use gonhanh_core::engine::Engine;
use std::fs;

// =============================================================================
// TELEX PATTERN DETECTION
// =============================================================================

/// Telex patterns that trigger Vietnamese transforms
const TELEX_PATTERNS: &[(&str, &str)] = &[
    ("aa", "â"),
    ("ee", "ê"),
    ("oo", "ô"),
    ("aw", "ă"),
    ("ow", "ơ"),
    ("uw", "ư"),
    ("dd", "đ"),
];

/// Check if word contains any telex trigger patterns
fn has_telex_patterns(word: &str) -> bool {
    let lower = word.to_lowercase();
    TELEX_PATTERNS
        .iter()
        .any(|(pattern, _)| lower.contains(pattern))
}

/// Get all telex patterns present in a word
fn get_telex_patterns(word: &str) -> Vec<(&'static str, &'static str)> {
    let lower = word.to_lowercase();
    TELEX_PATTERNS
        .iter()
        .filter(|(pattern, _)| lower.contains(pattern))
        .copied()
        .collect()
}

/// Generate the transformed buffer state (what Vietnamese IME would produce)
fn generate_transformed_buffer(word: &str) -> String {
    let mut result = word.to_string();

    // Apply transforms in order (case-insensitive replacement preserving case)
    for (pattern, replacement) in TELEX_PATTERNS {
        result = replace_pattern_preserve_case(&result, pattern, replacement);
    }

    result
}

/// Check if word starts with valid Vietnamese initial consonant
fn has_valid_vn_initial(word: &str) -> bool {
    let lower = word.to_lowercase();

    // Invalid English consonant clusters (not valid in Vietnamese)
    let invalid_clusters = [
        // 3-char clusters
        "thr", "str", "spr", "scr", "spl", "shr", "sch", // 2-char clusters
        "bl", "br", "cl", "cr", "dr", "fl", "fr", "gl", "gr", "pl", "pr", "sl", "sc", "sk", "sm",
        "sn", "sp", "st", "sw", "tw", "wr", "wh", // Invalid single initials
        "f", "j", "w", "z",
    ];

    // Check invalid clusters first (longer patterns first)
    for cluster in invalid_clusters {
        if lower.starts_with(cluster) {
            return false;
        }
    }

    // Valid Vietnamese initials
    let valid_initials = [
        "ngh", "ch", "gh", "gi", "kh", "ng", "nh", "ph", "qu", "th", "tr", "b", "c", "d", "g", "h",
        "k", "l", "m", "n", "p", "q", "r", "s", "t", "v", "x", "a", "e", "i", "o", "u", "y",
    ];

    for initial in valid_initials {
        if lower.starts_with(initial) {
            // Check what follows the initial
            let rest = &lower[initial.len()..];
            if rest.is_empty() {
                return true;
            }
            let next = rest.chars().next().unwrap();
            // Must be followed by vowel (or 'h'/'r' for valid digraphs already checked)
            if is_vowel_char(next) {
                return true;
            }
            // Digraphs/trigraphs are already complete, anything after should be vowel
            if initial.len() > 1 {
                return false;
            }
            // Single consonant followed by consonant - only valid for digraphs
            // th, tr, ch, gh, kh, ng, nh, ph, qu are already in valid_initials
            return false;
        }
    }

    false
}

fn is_vowel_char(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

/// Generate typing variants for English word with telex patterns
/// - Words with valid VN initial: test both direct + self-cancel variants
/// - Words with invalid VN initial: test direct only (rely on auto-restore)
fn generate_english_typing_variants(word: &str) -> Vec<String> {
    // Words with invalid Vietnamese initial → direct only
    if !has_valid_vn_initial(word) {
        return vec![word.to_string()];
    }

    let lower = word.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();

    // Self-cancel patterns: extra char cancels the transform
    // Only apply for patterns that are the PRIMARY vowel cluster
    // Skip if there's another vowel BEFORE the pattern (indicates compound/longer structure)
    let self_cancel_patterns = [
        ("aa", 'a'),
        ("ee", 'e'),
        ("oo", 'o'),
        ("aw", 'w'),
        ("ow", 'w'),
        ("uw", 'w'),
        ("dd", 'd'),
    ];

    let is_vowel = |c: char| "aeiou".contains(c);

    let mut self_cancel = word.to_string();
    for (pattern, cancel_char) in self_cancel_patterns {
        if let Some(pos) = lower.find(pattern) {
            // Check if there's a vowel BEFORE the pattern
            // If yes, skip cancel (pattern is not the primary vowel)
            // Examples:
            //   "goon" - no vowel before 'oo' → generate cancel ✓
            //   "career" - has 'a' before 'ee' → skip cancel ✓
            //   "boom" - no vowel before 'oo' → generate cancel ✓
            //   "doreen" - has 'o' before 'ee' → skip cancel ✓
            //   "casebook" - has 'a','e' before 'oo' → skip cancel ✓
            let has_vowel_before = chars[..pos].iter().any(|&c| is_vowel(c));

            // The has_vowel_before check handles all cases:
            // - "however" (h-ow-ever) → no vowel before 'ow' → generate cancel ✓
            // - "below" (b-e-l-ow) → has 'e' before 'ow' → skip cancel ✓
            // - "narrow" (n-a-rr-ow) → has 'a' before 'ow' → skip cancel ✓
            // - "middle" (m-i-dd-le) → has 'i' before 'dd' → skip cancel ✓

            if !has_vowel_before {
                self_cancel = insert_cancel_char(&self_cancel, pattern, cancel_char);
            }
        }
    }

    if self_cancel != word {
        vec![self_cancel, word.to_string()]
    } else {
        vec![word.to_string()]
    }
}

/// Insert cancel character after pattern occurrences
/// Skip if double vowel (aa/ee/oo) is preceded by 'w' - horn/breve takes precedence
fn insert_cancel_char(word: &str, pattern: &str, cancel_char: char) -> String {
    let mut result = String::new();
    let chars: Vec<char> = word.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let remaining: String = chars[i..].iter().collect();
        let remaining_lower = remaining.to_lowercase();

        if remaining_lower.starts_with(pattern) {
            // Check if preceded by 'w' for double vowel patterns (aa/ee/oo)
            // In this case, 'w' creates horn/breve, not circumflex, so no cancel needed
            // Examples: harwood (w+oo), biweekly (w+ee), sapwood (w+oo)
            let preceded_by_w = i > 0
                && chars[i - 1].to_ascii_lowercase() == 'w'
                && matches!(pattern, "aa" | "ee" | "oo");

            if preceded_by_w {
                // Skip cancel, just add the pattern as-is
                for j in 0..pattern.len() {
                    result.push(chars[i + j]);
                }
            } else {
                for j in 0..pattern.len() {
                    result.push(chars[i + j]);
                }
                // Preserve case of last pattern char for cancel char
                if chars[i + pattern.len() - 1].is_uppercase() {
                    result.push(cancel_char.to_ascii_uppercase());
                } else {
                    result.push(cancel_char);
                }
            }
            i += pattern.len();
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Replace pattern while preserving case of first character
fn replace_pattern_preserve_case(text: &str, pattern: &str, replacement: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = text.chars().collect();

    while i < chars.len() {
        // Check if pattern matches at current position (case-insensitive)
        let remaining: String = chars[i..].iter().collect();
        let remaining_lower = remaining.to_lowercase();

        if remaining_lower.starts_with(pattern) {
            // Preserve case of first char
            let rep_chars: Vec<char> = replacement.chars().collect();
            if chars[i].is_uppercase() {
                result.extend(rep_chars.iter().flat_map(|c| c.to_uppercase()));
            } else {
                result.push_str(replacement);
            }
            i += pattern.len();
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
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

fn type_word_with_space(engine: &mut Engine, word: &str) -> String {
    engine.clear();
    let mut output = String::new();

    // Type the word
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

// =============================================================================
// GENERATOR
// =============================================================================

/// Generate typing orders file for English words with telex patterns
/// Format: word TAB variant1,variant2,... (same as vietnamese_22k_typing_variants.txt)
#[test]
#[ignore] // Run with: cargo test generate_english_telex_patterns -- --ignored --nocapture
fn generate_english_telex_patterns() {
    use std::io::Write;

    let content =
        fs::read_to_string("tests/data/english_100k.txt").expect("Failed to read english_100k.txt");

    let words: Vec<&str> = content
        .lines()
        .filter(|line| {
            let w = line.trim();
            !w.is_empty() && w.chars().all(|c| c.is_ascii_alphabetic())
        })
        .collect();

    let mut output = std::fs::File::create("tests/data/english_100k_typing_variants.txt")
        .expect("Failed to create");

    writeln!(output, "# English 100k Typing Variants").unwrap();
    writeln!(output, "# Format: word TAB variant1,variant2,...").unwrap();
    writeln!(output, "# Generated by english_telex_patterns_test.rs").unwrap();
    writeln!(output).unwrap();

    let mut count = 0;
    let mut by_pattern: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for word in &words {
        if has_telex_patterns(word) {
            let patterns = get_telex_patterns(word);
            let pattern_strs: Vec<&str> = patterns.iter().map(|(p, _)| *p).collect();

            // Generate variants: direct + self-cancel for each pattern
            let variants = generate_english_typing_variants(word);
            writeln!(output, "{}\t{}", word, variants.join(",")).unwrap();

            for p in &pattern_strs {
                by_pattern
                    .entry(p.to_string())
                    .or_default()
                    .push(word.to_string());
            }

            count += 1;
        }
    }

    println!("\n=== ENGLISH TELEX PATTERNS SUMMARY ===\n");
    println!("Total words tested: {}", words.len());
    println!("Words with telex patterns: {}", count);

    println!("\nBy pattern:");
    let mut patterns: Vec<_> = by_pattern.iter().collect();
    patterns.sort_by_key(|(k, _)| *k);
    for (pattern, words) in patterns {
        println!("  {}: {} words", pattern, words.len());
    }

    println!("\nOutput: tests/data/english_100k_typing_variants.txt");
}

// =============================================================================
// TESTS
// =============================================================================

/// Test all English words with telex patterns
/// Classifies results:
/// - RESTORABLE: auto-restore works (transformed buffer is invalid VN)
/// - NON-RESTORABLE: kept as VN (transformed buffer is valid VN - expected)
#[test]
fn test_english_telex_patterns_restore() {
    let content = match fs::read_to_string("tests/data/english_100k_typing_variants.txt") {
        Ok(c) => c,
        Err(_) => {
            println!("Run 'cargo test generate_english_telex_patterns -- --ignored' first");
            return;
        }
    };

    let mut engine = Engine::new();
    engine.set_method(0); // Telex
    engine.set_english_auto_restore(true);

    let mut total = 0;
    let mut restored = 0;
    // (input_typed, expected_word, actual_output, vn_buffer)
    let mut failures: Vec<(String, String, String, String)> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let word = parts[0];
        let variants: Vec<&str> = parts[1].split(',').collect();

        // Test each variant
        for variant in &variants {
            total += 1;

            let expected = format!("{} ", word);
            let actual = type_word_with_space(&mut engine, variant);

            if actual == expected {
                restored += 1;
            } else {
                let buffer = generate_transformed_buffer(variant);
                failures.push((
                    variant.to_string(),
                    word.to_string(),
                    actual.trim().to_string(),
                    buffer,
                ));
            }
        }
    }

    println!("\n=== ENGLISH TELEX PATTERNS TEST ===\n");
    println!("Total variants tested: {}", total);
    println!(
        "Auto-restored to English: {} ({:.1}%)",
        restored,
        restored as f64 / total as f64 * 100.0
    );
    println!(
        "Failed (kept as VN): {} ({:.1}%)",
        failures.len(),
        failures.len() as f64 / total as f64 * 100.0
    );

    if !failures.is_empty() {
        println!("\n=== FAILURES (first 30) ===\n");
        println!(
            "{:<12} {:<15} {:<12} {:<12} {:<12}",
            "WORD", "VARIANT", "EXPECTED", "ACTUAL", "BUFFER"
        );
        println!("{}", "-".repeat(65));
        for (variant, word, actual, buffer) in failures.iter().take(30) {
            println!(
                "{:<12} {:<15} {:<12} {:<12} {:<12}",
                word, variant, word, actual, buffer
            );
        }

        // Write failures to file (same format as vietnamese_22k_failures.txt)
        use std::io::Write;
        if let Ok(mut f) = std::fs::File::create("tests/data/english_100k_failures.txt") {
            writeln!(f, "# English 100k Typing Variants Failures").ok();
            writeln!(
                f,
                "# Format: WORD \\t VARIANT \\t EXPECTED \\t ACTUAL \\t BUFFER"
            )
            .ok();
            writeln!(f, "# Total failures: {}", failures.len()).ok();
            writeln!(f).ok();
            for (variant, word, actual, buffer) in &failures {
                writeln!(f, "{}\t{}\t{}\t{}\t{}", word, variant, word, actual, buffer).ok();
            }
            println!("\nWritten to: tests/data/english_100k_failures.txt");
        }
    }

    // Count by pattern
    let mut by_pattern_restored: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut by_pattern_kept: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let word = parts[0];
        let patterns = get_telex_patterns(word);
        let expected = format!("{} ", word);
        let actual = type_word_with_space(&mut engine, word);

        for (p, _) in &patterns {
            if actual == expected {
                *by_pattern_restored.entry(p.to_string()).or_default() += 1;
            } else {
                *by_pattern_kept.entry(p.to_string()).or_default() += 1;
            }
        }
    }

    println!("\n=== BY PATTERN ===\n");
    println!("{:<10} {:<15} {:<15}", "PATTERN", "RESTORED", "KEPT_AS_VN");
    println!("{}", "-".repeat(40));
    for pattern in ["aa", "ee", "oo", "aw", "ow", "uw", "dd"] {
        let r = by_pattern_restored.get(pattern).unwrap_or(&0);
        let k = by_pattern_kept.get(pattern).unwrap_or(&0);
        println!("{:<10} {:<15} {:<15}", pattern, r, k);
    }

    // This test is informational - we don't fail because keeping valid VN is expected
    println!("\n✓ Test completed (informational - no assertions)");
}

/// Quick sanity test for specific words - shows which restore vs kept as VN
#[test]
fn test_specific_english_telex_words() {
    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_english_auto_restore(true);

    // Words with oo pattern (most should restore - invalid VN ending)
    let oo_words = [
        "book", "good", "food", "look", "took", "cool", "pool", "tool",
    ];

    // Words with ee pattern (some valid VN like sê, bê, etc.)
    let ee_words = ["see", "bee", "fee", "tree", "free", "keep", "deep", "seek"];

    // Words with aw pattern (many valid VN like lắ, etc.)
    let aw_words = [
        "law", "saw", "raw", "draw", "straw", "crawl", "dawn", "lawn",
    ];

    // Words with ow pattern (many valid VN like lơ, bơ, etc.)
    let ow_words = ["low", "bow", "row", "show", "know", "flow", "grow", "slow"];

    println!("\n=== SPECIFIC ENGLISH TELEX WORDS ===\n");
    println!("✓ = restored to English | ✗ = kept as Vietnamese (valid VN syllable)\n");

    let mut restored = 0;
    let mut kept_vn = 0;

    for (label, words) in [
        ("oo pattern", oo_words.as_slice()),
        ("ee pattern", ee_words.as_slice()),
        ("aw pattern", aw_words.as_slice()),
        ("ow pattern", ow_words.as_slice()),
    ] {
        println!("--- {} ---", label);
        for word in words {
            let expected = format!("{} ", word);
            let actual = type_word_with_space(&mut engine, word);

            if actual == expected {
                restored += 1;
                println!("✓ {}", word);
            } else {
                kept_vn += 1;
                println!("✗ {} → {}", word, actual.trim());
            }
        }
        println!();
    }

    println!("Summary: {} restored, {} kept as VN", restored, kept_vn);
    // No assertion - this is informational
}
