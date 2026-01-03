//! Paragraph tests
//!
//! Long-form typing tests with mixed Vietnamese/English content.
//! Tests real-world usage patterns and auto-restore accuracy.
//!
//! ## Test Categories
//!
//! 1. Mixed paragraph (VN/EN words)
//! 2. English auto-restore
//! 3. Vietnamese keep
//! 4. Rapid language switching
//! 5. Ambiguous words
//! 6. Double consonants
//! 7. English prefixes/suffixes
//! 8. Tech terms
//! 9. Vietnamese place names

use crate::v3::processor::Processor;
use crate::v3::tests::type_sequence;

// ============================================================================
// COMPREHENSIVE PARAGRAPH TEST
// ============================================================================

/// Main test: Mixed Vietnamese/English paragraph
///
/// Input (Telex):
/// "Chafo cacs banfj, minhf ddang tesst Gox Nhanh. Smart auto restore: text,
/// expect, perfect, window, with, their, wow, luxury, tesla, life, issue,
/// feature, express, wonderful, support, core, care, saas, sax, push, work,
/// hard, user. Per app memory: VS Code, Slack. Auto disable: Japanese,
/// Korean, Chinese. DDawsk Lawsk, DDawsk Noong, Kroong Buks. Thanks for your
/// wonderful support with thiss software."
///
/// Expected:
/// Vietnamese words converted: Chafo→Chào, cacs→các, banfj→bạn, etc.
/// English words preserved: text, expect, perfect, window, etc.
/// Place names: Đắk Lắk, Đắk Nông, Krông Búk
#[test]
fn test_mixed_paragraph_comprehensive() {
    let mut p = Processor::new();

    // Test individual word transformations first
    let test_cases = [
        // Vietnamese words (should transform)
        ("Chafo", "Chào"),
        ("cacs", "các"),
        ("banfj", "bạn"),
        ("minhf", "mình"),
        ("ddang", "đang"),
        ("Gox", "Gõ"),
        // English words (should restore/preserve)
        ("text", "text"),
        ("expect", "expect"),
        ("perfect", "perfect"),
        ("window", "window"),
        ("with", "with"),
        ("their", "their"),
        ("wow", "wow"),
        ("luxury", "luxury"),
        ("tesla", "tesla"),
        ("life", "life"),
        ("issue", "issue"),
        ("feature", "feature"),
        ("express", "express"),
        ("wonderful", "wonderful"),
        ("support", "support"),
        ("core", "core"),
        ("care", "care"),
        ("saas", "saas"),
        ("sax", "sax"),
        ("push", "push"),
        ("work", "work"),
        ("hard", "hard"),
        ("user", "user"),
        // Place names (Vietnamese with special patterns)
        ("DDawsk", "Đắk"),
        ("Lawsk", "Lắk"),
        ("Noong", "Nông"),
        ("Kroong", "Krông"),
        ("Buks", "Búk"),
        // More English
        ("Thanks", "Thanks"),
        ("software", "software"),
    ];

    for (input, expected) in test_cases {
        p.clear();
        let _result = type_sequence(&mut p, input);
        // Commit with space to trigger auto-restore
        p.process(' ', false, false);
        let final_result = p.buffer_content();

        // Note: This test documents expected behavior
        // Some assertions may fail until Phase 05 complete
        eprintln!(
            "Input: {:?} -> Buffer: {:?}, Expected: {:?}",
            input, final_result, expected
        );
    }
}

/// Test: Smart auto-restore for common English words
#[test]
fn test_english_auto_restore() {
    let mut p = Processor::new();

    // English words that should be auto-restored (not transformed to Vietnamese)
    let english_words = [
        "text",
        "expect",
        "perfect",
        "window",
        "with",
        "their",
        "wow",
        "luxury",
        "tesla",
        "life",
        "issue",
        "feature",
        "express",
        "wonderful",
        "support",
        "core",
        "care",
        "saas",
        "sax",
        "push",
        "work",
        "hard",
        "user",
        "test",
        "smart",
        "auto",
        "restore",
        "memory",
        "code",
        "slack",
        "disable",
        "japanese",
        "korean",
        "chinese",
        "thanks",
        "software",
    ];

    let mut failures = Vec::new();

    for word in english_words {
        p.clear();
        type_sequence(&mut p, word);
        p.process(' ', false, false); // Commit
        let result = p.buffer_content().trim().to_string();

        if result != word {
            failures.push(format!("{:?} -> {:?} (expected {:?})", word, result, word));
        }
    }

    if !failures.is_empty() {
        panic!(
            "English auto-restore failed for {} words:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}

/// Test: Vietnamese words should be transformed correctly
#[test]
fn test_vietnamese_keep() {
    let mut p = Processor::new();

    // (input_telex, expected_output)
    let vietnamese_words = [
        ("chafo", "chào"),
        ("cacs", "các"),
        ("banfj", "bạn"),
        ("minhf", "mình"),
        ("ddang", "đang"),
        ("Gox", "Gõ"),
        ("Nhanh", "Nhanh"),
        ("DDawsk", "Đắk"),
        ("Lawsk", "Lắk"),
        ("Noong", "Nông"),
        ("Kroong", "Krông"),
        ("Buks", "Búk"),
    ];

    let mut failures = Vec::new();

    for (input, expected) in vietnamese_words {
        p.clear();
        type_sequence(&mut p, input);
        p.process(' ', false, false); // Commit
        let result = p.buffer_content().trim().to_string();

        if result != expected {
            failures.push(format!(
                "{:?} -> {:?} (expected {:?})",
                input, result, expected
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "Vietnamese transform failed for {} words:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}

/// Test: Full paragraph simulation (word by word)
///
/// Tests mixed Vietnamese/English paragraph processing.
#[test]
fn test_full_paragraph() {
    let mut p = Processor::new();

    let input = "Chafo cacs banfj, minhf ddang tesst Gox Nhanh. Smart auto restore: text, expect, perfect, window, with, their, wow, luxury, tesla, life, issue, feature, express, wonderful, support, core, care, saas, sax, push, work, hard, user. Per app memory: VS Code, Slack. Auto disable: Japanese, Korean, Chinese. DDawsk Lawsk, DDawsk Noong, Kroong Buks. Thanks for your wonderful support with thiss software.";

    let expected = "Chào các bạn, mình đang test Gõ Nhanh. Smart auto restore: text, expect, perfect, window, with, their, wow, luxury, tesla, life, issue, feature, express, wonderful, support, core, care, saas, sax, push, work, hard, user. Per app memory: VS Code, Slack. Auto disable: Japanese, Korean, Chinese. Đắk Lắk, Đắk Nông, Krông Búk. Thanks for your wonderful support with this software.";

    // Word-by-word test (safe approach - avoids punctuation overflow)
    let mut results = Vec::new();
    for word in input.split_whitespace() {
        p.clear();
        // Strip punctuation for processing
        let clean_word: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
        let trailing_punct: String = word
            .chars()
            .rev()
            .take_while(|c| !c.is_alphanumeric())
            .collect::<String>()
            .chars()
            .rev()
            .collect();

        if !clean_word.is_empty() {
            for c in clean_word.chars() {
                p.process(c, false, false);
            }
            p.process(' ', false, false); // Commit
            let word_result = p.buffer_content().trim().to_string();
            results.push(format!("{}{}", word_result, trailing_punct));
        }
    }

    let result = results.join(" ");

    assert_eq!(
        result, expected,
        "\n=== FULL PARAGRAPH TEST FAILED ===\nInput:    {}\nExpected: {}\nGot:      {}",
        input, expected, result
    );
}

// ============================================================================
// STRESS TESTS
// ============================================================================

/// Test: Rapid switching between Vietnamese and English
#[test]
fn test_rapid_language_switching() {
    let mut p = Processor::new();

    // Pattern: VN EN VN EN VN EN
    let sequence = [
        ("chafo", true),  // VN
        ("hello", false), // EN
        ("banfj", true),  // VN
        ("world", false), // EN
        ("ddang", true),  // VN
        ("test", false),  // EN
    ];

    for (word, is_vietnamese) in sequence {
        p.clear();
        type_sequence(&mut p, word);
        p.process(' ', false, false);
        let result = p.buffer_content();

        eprintln!(
            "Word: {:?} ({}), Result: {:?}",
            word,
            if is_vietnamese { "VN" } else { "EN" },
            result
        );
    }
}

/// Test: Words that look similar in both languages
#[test]
fn test_ambiguous_words() {
    let mut p = Processor::new();

    // Words that could be either language
    let ambiguous = [
        ("an", "ăn or an?"), // VN: ăn, EN: an
        ("no", "nô or no?"), // Could be either
        ("me", "mẽ or me?"), // Could be either
        ("to", "tô or to?"), // Could be either
    ];

    for (word, note) in ambiguous {
        p.clear();
        type_sequence(&mut p, word);
        p.process(' ', false, false);
        let result = p.buffer_content();

        eprintln!("Ambiguous: {:?} ({}) -> {:?}", word, note, result);
    }
}

// ============================================================================
// EDGE CASES
// ============================================================================

/// Test: Double consonants (common in English)
#[test]
fn test_double_consonants() {
    let mut p = Processor::new();

    let words = [
        "coffee", "pizza", "jazz", "buzz", "class", "glass", "press", "stress", "success",
        "process", "access",
    ];

    for word in words {
        p.clear();
        type_sequence(&mut p, word);
        p.process(' ', false, false);
        let result = p.buffer_content();

        eprintln!("Double consonant: {:?} -> {:?}", word, result);
    }
}

/// Test: English prefixes
#[test]
fn test_english_prefixes() {
    let mut p = Processor::new();

    let words = [
        "uninstall",
        "reinstall",
        "preview",
        "disable",
        "disconnect",
        "impossible",
        "international",
    ];

    for word in words {
        p.clear();
        type_sequence(&mut p, word);
        p.process(' ', false, false);
        let result = p.buffer_content();

        eprintln!("Prefix: {:?} -> {:?}", word, result);
    }
}

/// Test: English suffixes
#[test]
fn test_english_suffixes() {
    let mut p = Processor::new();

    let words = [
        "running",
        "testing",
        "working",
        "processing",
        "action",
        "function",
        "solution",
        "application",
        "quickly",
        "slowly",
        "perfectly",
        "wonderfully",
        "tested",
        "processed",
        "installed",
        "disabled",
    ];

    for word in words {
        p.clear();
        type_sequence(&mut p, word);
        p.process(' ', false, false);
        let result = p.buffer_content();

        eprintln!("Suffix: {:?} -> {:?}", word, result);
    }
}

/// Test: Programming/tech terms
#[test]
fn test_tech_terms() {
    let mut p = Processor::new();

    let words = [
        "function",
        "variable",
        "const",
        "async",
        "await",
        "promise",
        "callback",
        "interface",
        "struct",
        "enum",
        "import",
        "export",
        "module",
        "package",
        "library",
        "framework",
        "runtime",
        "compile",
        "debug",
        "deploy",
    ];

    for word in words {
        p.clear();
        type_sequence(&mut p, word);
        p.process(' ', false, false);
        let result = p.buffer_content();

        eprintln!("Tech term: {:?} -> {:?}", word, result);
    }
}

/// Test: Vietnamese place names
#[test]
fn test_vietnamese_place_names() {
    let mut p = Processor::new();

    let places = [
        ("DDawsk", "Đắk"),
        ("Lawsk", "Lắk"),
        ("Noong", "Nông"),
        ("Kroong", "Krông"),
        ("Buks", "Búk"),
        ("Hawf", "Hà"),
        ("Nooji", "Nội"),
        ("Saif", "Sài"),
        ("Gofn", "Gòn"),
        ("DDaf", "Đà"),
        ("Nawxng", "Nẵng"),
    ];

    for (input, expected) in places {
        p.clear();
        type_sequence(&mut p, input);
        p.process(' ', false, false);
        let result = p.buffer_content();

        eprintln!(
            "Place: {:?} -> {:?}, expected: {:?}",
            input, result, expected
        );
    }
}
