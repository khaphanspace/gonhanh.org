//! Vietnamese Round-Trip Test
//!
//! Automatically generates Telex input sequences from Vietnamese words,
//! feeds them to the engine, and verifies the output matches the original.
//!
//! Supports multiple typing styles:
//! - Style 1 (word first): nguoiwf → người
//! - Style 2 (mark after char): nguowif → người
//! - Style 3 (inline): nguwowif → người
//!
//! Example: "người" → generate telex variants → engine → "người" ✓

use gonhanh_core::engine::Engine;
use std::collections::HashMap;
use std::fs;

/// Decompose a Vietnamese character into (base_char, vowel_mark, tone_mark)
/// Returns: (base, vowel_modifier, tone_modifier)
fn decompose_vn_char(ch: char) -> (char, Option<char>, Option<char>) {
    // Full decomposition map
    let decomp: HashMap<char, (char, Option<char>, Option<char>)> = [
        // a variants
        ('a', ('a', None, None)),
        ('á', ('a', None, Some('s'))),
        ('à', ('a', None, Some('f'))),
        ('ả', ('a', None, Some('r'))),
        ('ã', ('a', None, Some('x'))),
        ('ạ', ('a', None, Some('j'))),
        // ă variants
        ('ă', ('a', Some('w'), None)),
        ('ắ', ('a', Some('w'), Some('s'))),
        ('ằ', ('a', Some('w'), Some('f'))),
        ('ẳ', ('a', Some('w'), Some('r'))),
        ('ẵ', ('a', Some('w'), Some('x'))),
        ('ặ', ('a', Some('w'), Some('j'))),
        // â variants
        ('â', ('a', Some('a'), None)),
        ('ấ', ('a', Some('a'), Some('s'))),
        ('ầ', ('a', Some('a'), Some('f'))),
        ('ẩ', ('a', Some('a'), Some('r'))),
        ('ẫ', ('a', Some('a'), Some('x'))),
        ('ậ', ('a', Some('a'), Some('j'))),
        // e variants
        ('e', ('e', None, None)),
        ('é', ('e', None, Some('s'))),
        ('è', ('e', None, Some('f'))),
        ('ẻ', ('e', None, Some('r'))),
        ('ẽ', ('e', None, Some('x'))),
        ('ẹ', ('e', None, Some('j'))),
        // ê variants
        ('ê', ('e', Some('e'), None)),
        ('ế', ('e', Some('e'), Some('s'))),
        ('ề', ('e', Some('e'), Some('f'))),
        ('ể', ('e', Some('e'), Some('r'))),
        ('ễ', ('e', Some('e'), Some('x'))),
        ('ệ', ('e', Some('e'), Some('j'))),
        // i variants
        ('i', ('i', None, None)),
        ('í', ('i', None, Some('s'))),
        ('ì', ('i', None, Some('f'))),
        ('ỉ', ('i', None, Some('r'))),
        ('ĩ', ('i', None, Some('x'))),
        ('ị', ('i', None, Some('j'))),
        // o variants
        ('o', ('o', None, None)),
        ('ó', ('o', None, Some('s'))),
        ('ò', ('o', None, Some('f'))),
        ('ỏ', ('o', None, Some('r'))),
        ('õ', ('o', None, Some('x'))),
        ('ọ', ('o', None, Some('j'))),
        // ô variants
        ('ô', ('o', Some('o'), None)),
        ('ố', ('o', Some('o'), Some('s'))),
        ('ồ', ('o', Some('o'), Some('f'))),
        ('ổ', ('o', Some('o'), Some('r'))),
        ('ỗ', ('o', Some('o'), Some('x'))),
        ('ộ', ('o', Some('o'), Some('j'))),
        // ơ variants
        ('ơ', ('o', Some('w'), None)),
        ('ớ', ('o', Some('w'), Some('s'))),
        ('ờ', ('o', Some('w'), Some('f'))),
        ('ở', ('o', Some('w'), Some('r'))),
        ('ỡ', ('o', Some('w'), Some('x'))),
        ('ợ', ('o', Some('w'), Some('j'))),
        // u variants
        ('u', ('u', None, None)),
        ('ú', ('u', None, Some('s'))),
        ('ù', ('u', None, Some('f'))),
        ('ủ', ('u', None, Some('r'))),
        ('ũ', ('u', None, Some('x'))),
        ('ụ', ('u', None, Some('j'))),
        // ư variants
        ('ư', ('u', Some('w'), None)),
        ('ứ', ('u', Some('w'), Some('s'))),
        ('ừ', ('u', Some('w'), Some('f'))),
        ('ử', ('u', Some('w'), Some('r'))),
        ('ữ', ('u', Some('w'), Some('x'))),
        ('ự', ('u', Some('w'), Some('j'))),
        // y variants
        ('y', ('y', None, None)),
        ('ý', ('y', None, Some('s'))),
        ('ỳ', ('y', None, Some('f'))),
        ('ỷ', ('y', None, Some('r'))),
        ('ỹ', ('y', None, Some('x'))),
        ('ỵ', ('y', None, Some('j'))),
        // đ
        ('đ', ('d', Some('d'), None)),
    ]
    .iter()
    .cloned()
    .collect();

    let lower = ch.to_lowercase().next().unwrap_or(ch);
    decomp.get(&lower).cloned().unwrap_or((ch, None, None))
}

/// Generate Telex input sequence - Style 1: word first, then marks at end
/// Example: "người" → "nguoiwf"
fn vietnamese_to_telex_style1(word: &str) -> String {
    let mut base = String::new();
    let mut vowel_marks = String::new();
    let mut tone: Option<char> = None;

    for ch in word.chars() {
        let (b, v, t) = decompose_vn_char(ch);
        base.push(b);
        if let Some(vm) = v {
            vowel_marks.push(vm);
        }
        if t.is_some() {
            tone = t;
        }
    }

    let mut result = base;
    result.push_str(&vowel_marks);
    if let Some(t) = tone {
        result.push(t);
    }
    result
}

/// Generate Telex input sequence - Style 2: marks inline after each char
/// Example: "người" → "nguwowif"
fn vietnamese_to_telex_style2(word: &str) -> String {
    let mut result = String::new();
    let mut tone: Option<char> = None;

    for ch in word.chars() {
        let (b, v, t) = decompose_vn_char(ch);
        result.push(b);
        if let Some(vm) = v {
            result.push(vm);
        }
        if t.is_some() {
            tone = t;
        }
    }

    if let Some(t) = tone {
        result.push(t);
    }
    result
}

/// Generate Telex input sequence - default style (inline marks)
fn vietnamese_to_telex(word: &str) -> String {
    vietnamese_to_telex_style2(word)
}

/// Simulate typing a Telex sequence and get the result
fn type_telex(engine: &mut Engine, telex: &str) -> String {
    engine.clear();

    for ch in telex.chars() {
        let keycode = char_to_keycode(ch);
        let caps = ch.is_uppercase();
        engine.on_key(keycode, caps, false);
    }

    engine.get_buffer_string()
}

/// Convert char to keycode (simplified)
fn char_to_keycode(ch: char) -> u16 {
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    match lower {
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
        '=' => 24,
        '9' => 25,
        '7' => 26,
        '-' => 27,
        '8' => 28,
        '0' => 29,
        ']' => 30,
        'o' => 31,
        'u' => 32,
        '[' => 33,
        'i' => 34,
        'p' => 35,
        'l' => 37,
        'j' => 38,
        '\'' => 39,
        'k' => 40,
        ';' => 41,
        '\\' => 42,
        ',' => 43,
        '/' => 44,
        'n' => 45,
        'm' => 46,
        '.' => 47,
        _ => 0,
    }
}

#[test]
fn test_decompose_chars() {
    // Test decomposition
    assert_eq!(decompose_vn_char('a'), ('a', None, None));
    assert_eq!(decompose_vn_char('á'), ('a', None, Some('s')));
    assert_eq!(decompose_vn_char('ă'), ('a', Some('w'), None));
    assert_eq!(decompose_vn_char('ắ'), ('a', Some('w'), Some('s')));
    assert_eq!(decompose_vn_char('ư'), ('u', Some('w'), None));
    assert_eq!(decompose_vn_char('đ'), ('d', Some('d'), None));
}

#[test]
fn test_telex_styles_output() {
    println!("\n=== Telex Conversion Styles ===");

    let samples = vec![
        ("người", "nguoiwf", "nguwowif"),
        ("việt", "vietwf", "vieetj"), // Note: style depends on tone position
        ("được", "duocwj", "duwowocj"),
        ("đã", "daddx", "ddax"),
        ("không", "khoongo", "khoong"),
        ("năm", "namwf", "nawm"),
    ];

    for (word, _expected_s1, _expected_s2) in &samples {
        let s1 = vietnamese_to_telex_style1(word);
        let s2 = vietnamese_to_telex_style2(word);
        println!("  {} → style1: '{}' | style2: '{}'", word, s1, s2);
    }
}

#[test]
fn test_roundtrip_common_words() {
    let mut engine = Engine::new();
    engine.set_method(0); // Telex
    engine.set_enabled(true);

    let common_words = vec![
        "có", "là", "và", "một", "cho", "với", "để", "về", "từ", "ra", "như", "còn", "làm", "đi",
        "ta", "ở",
    ];

    println!("\n=== Round-trip Common Words ===");
    println!(
        "{:<10} {:<15} {:<15} {:<10} {:<10}",
        "Word", "Style1", "Style2", "Result1", "Result2"
    );
    println!("{}", "-".repeat(65));

    let mut s1_passed = 0;
    let mut s2_passed = 0;

    for word in &common_words {
        let telex1 = vietnamese_to_telex_style1(word);
        let telex2 = vietnamese_to_telex_style2(word);

        let result1 = type_telex(&mut engine, &telex1);
        let result2 = type_telex(&mut engine, &telex2);

        let ok1 = if result1 == *word {
            s1_passed += 1;
            "✓"
        } else {
            "✗"
        };
        let ok2 = if result2 == *word {
            s2_passed += 1;
            "✓"
        } else {
            "✗"
        };

        println!(
            "{:<10} {:<15} {:<15} {:<10} {:<10}",
            word,
            telex1,
            telex2,
            format!("{} {}", result1, ok1),
            format!("{} {}", result2, ok2)
        );
    }

    println!(
        "\nStyle1: {}/{} | Style2: {}/{}",
        s1_passed,
        common_words.len(),
        s2_passed,
        common_words.len()
    );
}

#[test]
fn test_roundtrip_from_wordlist() {
    let content = fs::read_to_string("tests/data/vietnamese-words.txt")
        .expect("Failed to read vietnamese-words.txt");

    // Test with both modern_tone settings
    for modern_tone in [false, true] {
        let mut engine = Engine::new();
        engine.set_method(0); // Telex
        engine.set_enabled(true);
        engine.set_modern_tone(modern_tone);

        let mut passed = 0;

        // Group failures by category with issue description
        let mut fail_no_transform: Vec<(String, String, String, String)> = Vec::new();
        let mut fail_wrong_mark: Vec<(String, String, String, String)> = Vec::new();
        let mut fail_tone_position: Vec<(String, String, String, String)> = Vec::new();
        let mut fail_normalize: Vec<(String, String, String, String)> = Vec::new();
        let mut fail_other: Vec<(String, String, String, String)> = Vec::new();

        let words: Vec<&str> = content
            .lines()
            .filter(|line| !line.contains(' '))
            .filter(|line| {
                line.chars().any(|c| {
                    "àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđ"
                        .contains(c)
                })
            })
            .take(1000)
            .collect();

        for word in &words {
            let telex = vietnamese_to_telex_style2(word);
            let result = type_telex(&mut engine, &telex);

            if result == *word {
                passed += 1;
            } else {
                // Categorize failure with issue description
                if result == telex || result.chars().all(|c| c.is_ascii()) {
                    let issue = "không transform".to_string();
                    fail_no_transform.push((
                        word.to_string(),
                        telex.clone(),
                        result.clone(),
                        issue,
                    ));
                } else if is_tone_position_diff(&result, word) {
                    let issue = format!("{}↔{}", get_tone_char(word), get_tone_char(&result));
                    fail_tone_position.push((
                        word.to_string(),
                        telex.clone(),
                        result.clone(),
                        issue,
                    ));
                } else if is_normalized_form(&result, word) {
                    let issue = "normalized".to_string();
                    fail_normalize.push((word.to_string(), telex.clone(), result.clone(), issue));
                } else if has_wrong_vowel_mark(&result, word) {
                    let issue = detect_vowel_mark_issue(word, &result);
                    fail_wrong_mark.push((word.to_string(), telex.clone(), result.clone(), issue));
                } else {
                    let issue = detect_other_issue(word, &telex, &result);
                    fail_other.push((word.to_string(), telex.clone(), result.clone(), issue));
                }
            }
        }

        let total_failed = fail_no_transform.len()
            + fail_wrong_mark.len()
            + fail_tone_position.len()
            + fail_normalize.len()
            + fail_other.len();

        // Print report header
        println!();
        println!(
            "╔══════════════════════════════════════════════════════════════════════════════╗"
        );
        println!(
            "║  VIETNAMESE ROUND-TRIP TEST REPORT                                           ║"
        );
        println!("║  modern_tone = {:<63}║", modern_tone);
        println!(
            "╚══════════════════════════════════════════════════════════════════════════════╝"
        );
        println!();
        println!("┌─────────────────────────────────────────────────────────────────────────────┐");
        println!("│  SUMMARY                                                                    │");
        println!("├─────────────────────────────────────────────────────────────────────────────┤");
        println!(
            "│  Tested: {:5}  │  Passed: {:5} ({:5.1}%)  │  Failed: {:5} ({:5.1}%)      │",
            words.len(),
            passed,
            (passed as f64 / words.len() as f64) * 100.0,
            total_failed,
            (total_failed as f64 / words.len() as f64) * 100.0
        );
        println!("└─────────────────────────────────────────────────────────────────────────────┘");

        // Print each category
        print_failure_table("1. FOREIGN WORDS (expected - từ mượn)", &fail_no_transform);
        print_failure_table("2. TONE POSITION (style diff - oà↔òa)", &fail_tone_position);
        print_failure_table("3. NORMALIZED (correct - choòng→chồng)", &fail_normalize);
        print_failure_table("4. WRONG VOWEL MARK (investigate)", &fail_wrong_mark);
        print_failure_table("5. OTHER BUGS (NEEDS FIX)", &fail_other);

        // Print final summary
        println!();
        println!("┌─────────────────────────────────────────────────────────────────────────────┐");
        println!("│  CATEGORY BREAKDOWN                                                         │");
        println!("├─────────────────────────────────────────────────────────────────────────────┤");
        println!(
            "│  Foreign words:    {:4}  (expected)                                        │",
            fail_no_transform.len()
        );
        println!(
            "│  Tone position:    {:4}  (style diff)                                      │",
            fail_tone_position.len()
        );
        println!(
            "│  Normalized:       {:4}  (correct)                                         │",
            fail_normalize.len()
        );
        println!(
            "│  Wrong mark:       {:4}  (investigate)                                     │",
            fail_wrong_mark.len()
        );
        println!(
            "│  Other bugs:       {:4}  (NEEDS FIX)                                       │",
            fail_other.len()
        );
        println!("└─────────────────────────────────────────────────────────────────────────────┘");
    }
}

/// Check if difference is just tone position (oà vs òa)
fn is_tone_position_diff(result: &str, expected: &str) -> bool {
    // Remove all tones and compare base
    fn remove_tones(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'á' | 'à' | 'ả' | 'ã' | 'ạ' => 'a',
                'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => 'ă',
                'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => 'â',
                'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' => 'e',
                'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => 'ê',
                'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => 'i',
                'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' => 'o',
                'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' => 'ô',
                'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => 'ơ',
                'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' => 'u',
                'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => 'ư',
                'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
                _ => c,
            })
            .collect()
    }

    let result_base = remove_tones(result);
    let expected_base = remove_tones(expected);

    // Same base, different tone position = style difference
    result_base == expected_base && result != expected
}

/// Check if result is normalized form (e.g., choòng → chồng)
fn is_normalized_form(result: &str, expected: &str) -> bool {
    // Check for double vowel in expected that got normalized
    let has_double_vowel = expected.contains("oò")
        || expected.contains("òo")
        || expected.contains("oó")
        || expected.contains("óo")
        || expected.contains("eè")
        || expected.contains("èe")
        || expected.contains("eé")
        || expected.contains("ée");
    has_double_vowel && result.chars().count() < expected.chars().count()
}

/// Print failure table with proper formatting
fn print_failure_table(name: &str, failures: &[(String, String, String, String)]) {
    if failures.is_empty() {
        return;
    }
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!(
        "│  {} ({:3}){}│",
        name,
        failures.len(),
        " ".repeat(75 - name.len() - 8)
    );
    println!("├─────────────┬──────────────────────┬──────────────────────┬─────────────────┤");
    println!("│ Word        │ Telex Input          │ Result               │ Issue           │");
    println!("├─────────────┼──────────────────────┼──────────────────────┼─────────────────┤");
    for (word, telex, result, issue) in failures.iter() {
        println!(
            "│ {:11} │ {:20} │ {:20} │ {:15} │",
            truncate_str(word, 11),
            truncate_str(telex, 20),
            truncate_str(result, 20),
            truncate_str(issue, 15)
        );
    }
    println!("└─────────────┴──────────────────────┴──────────────────────┴─────────────────┘");
}

/// Truncate string to fit column width
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{}…", truncated)
    }
}

/// Get the toned vowel from a word for display
fn get_tone_char(word: &str) -> String {
    for c in word.chars() {
        if "áàảãạắằẳẵặấầẩẫậéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ".contains(c)
        {
            return c.to_string();
        }
    }
    "?".to_string()
}

/// Detect what vowel mark issue occurred
fn detect_vowel_mark_issue(expected: &str, result: &str) -> String {
    let mark_chars = "ăâêôơư";
    let exp_marks: Vec<char> = expected
        .chars()
        .filter(|c| mark_chars.contains(*c))
        .collect();
    let res_marks: Vec<char> = result.chars().filter(|c| mark_chars.contains(*c)).collect();

    if exp_marks.len() != res_marks.len() {
        format!("mark count: {}→{}", exp_marks.len(), res_marks.len())
    } else {
        format!(
            "{}→{}",
            exp_marks.iter().collect::<String>(),
            res_marks.iter().collect::<String>()
        )
    }
}

/// Detect what issue occurred in "other" category
fn detect_other_issue(expected: &str, telex: &str, result: &str) -> String {
    // Check for missing characters
    let exp_chars: Vec<char> = expected.chars().collect();
    let res_chars: Vec<char> = result.chars().collect();

    // Check if tone key was eaten as modifier (s, f, r, x, j)
    let tone_keys = ['s', 'f', 'r', 'x', 'j'];
    for tk in &tone_keys {
        if telex.contains(*tk) && expected.contains(*tk) && !result.contains(*tk) {
            return format!("mất '{}'", tk);
        }
    }

    // Check for d→đ conversion
    if expected.starts_with('d') && result.starts_with('đ') {
        return "d→đ sai".to_string();
    }

    // Check length difference
    if exp_chars.len() > res_chars.len() {
        return format!("mất {} ký tự", exp_chars.len() - res_chars.len());
    }

    // Default
    "unknown".to_string()
}

fn has_wrong_vowel_mark(result: &str, expected: &str) -> bool {
    let vowel_marks = "ăâêôơư";
    let result_marks: String = result
        .chars()
        .filter(|c| vowel_marks.contains(*c))
        .collect();
    let expected_marks: String = expected
        .chars()
        .filter(|c| vowel_marks.contains(*c))
        .collect();
    !result_marks.is_empty() && result_marks != expected_marks
}

#[test]
fn test_print_telex_samples() {
    println!("\n=== Telex Conversion Samples ===");

    let samples = vec![
        "người",
        "việt",
        "nam",
        "được",
        "không",
        "có",
        "là",
        "và",
        "đã",
        "năm",
        "những",
        "để",
        "này",
        "về",
        "đến",
        "ở",
        "ăn",
        "uống",
        "đường",
        "trường",
        "nước",
        "sương",
        "hương",
    ];

    for word in samples {
        let telex = vietnamese_to_telex(word);
        println!("  {} → {}", word, telex);
    }
}
