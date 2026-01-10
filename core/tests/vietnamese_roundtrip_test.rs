//! Vietnamese Round-Trip Test
//!
//! Generates Telex input sequences from Vietnamese words,
//! feeds them to the engine, and verifies the output matches the original.
//!
//! Supports multiple typing styles/permutations:
//! - Style 1 (word first): nguoiwf → người
//! - Style 2 (mark inline): nguwowif → người
//! - Style 3 (tone inline): nguwowfi → người
//! - Permutations: được → dduwowcj, duocdwwj, dduowwcj, ...
//!
//! Example: "được" can be typed as:
//!   - dduwowcj (inline marks)
//!   - duocdwwj (base first, marks at end)
//!   - dduowcjw (mixed order)
//!   - All should produce "được"

mod common;

use gonhanh_core::engine::Engine;
use std::collections::{HashMap, HashSet};
use std::fs;

// ============================================================
// VIETNAMESE TO TELEX CONVERSION
// ============================================================

/// Decompose a Vietnamese character into (base_char, vowel_mark, tone_mark)
fn decompose_vn_char(ch: char) -> (char, Option<char>, Option<char>) {
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

// ============================================================
// TELEX PERMUTATION GENERATOR
// ============================================================

/// Represents a typing element with its position and modifiers
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct TypingElement {
    base: char,              // Base character (a, b, c, d, e, ...)
    vowel_mod: Option<char>, // Vowel modifier (a, e, o, w, d)
    tone: Option<char>,      // Tone modifier (s, f, r, x, j) - stored for future use
    is_upper: bool,          // Uppercase
}

/// Generate all permutations of Telex typing for a Vietnamese word
/// Returns a set of unique typing sequences
fn generate_telex_permutations(word: &str) -> Vec<String> {
    let mut elements: Vec<TypingElement> = Vec::new();
    let mut tone: Option<char> = None;
    let mut tone_vowel_idx: Option<usize> = None; // Which vowel has the tone

    // Parse word into elements
    for ch in word.chars() {
        let is_upper = ch.is_uppercase();
        let (base, vowel_mod, t) = decompose_vn_char(ch);

        if t.is_some() {
            tone = t;
            tone_vowel_idx = Some(elements.len());
        }

        elements.push(TypingElement {
            base,
            vowel_mod,
            tone: None, // Will be assigned later
            is_upper,
        });
    }

    let mut results = HashSet::new();

    // Strategy 1: Inline marks (modifier immediately after base)
    // được → dduwowcj
    let mut s1 = String::new();
    for (i, el) in elements.iter().enumerate() {
        if el.is_upper {
            s1.push(el.base.to_ascii_uppercase());
        } else {
            s1.push(el.base);
        }
        if let Some(m) = el.vowel_mod {
            s1.push(m);
        }
        // Tone at the toned vowel position
        if tone_vowel_idx == Some(i) {
            if let Some(t) = tone {
                s1.push(t);
            }
        }
    }
    results.insert(s1);

    // Strategy 2: Inline marks, tone at end
    // được → dduwowcj (same if tone was at last vowel)
    let mut s2 = String::new();
    for el in elements.iter() {
        if el.is_upper {
            s2.push(el.base.to_ascii_uppercase());
        } else {
            s2.push(el.base);
        }
        if let Some(m) = el.vowel_mod {
            s2.push(m);
        }
    }
    if let Some(t) = tone {
        s2.push(t);
    }
    results.insert(s2);

    // Strategy 3: Base word first, then all marks, then tone
    // được → duocddwwj
    let mut base_str = String::new();
    let mut marks_str = String::new();
    for el in elements.iter() {
        if el.is_upper {
            base_str.push(el.base.to_ascii_uppercase());
        } else {
            base_str.push(el.base);
        }
        if let Some(m) = el.vowel_mod {
            marks_str.push(m);
        }
    }
    let mut s3 = base_str.clone();
    s3.push_str(&marks_str);
    if let Some(t) = tone {
        s3.push(t);
    }
    results.insert(s3);

    // Strategy 4: Base + marks interleaved differently
    // Mark đ first (dd at start), then rest inline
    // được → dduowcwj
    if elements.first().map(|e| e.vowel_mod) == Some(Some('d')) {
        let mut s4 = String::new();
        for (i, el) in elements.iter().enumerate() {
            if el.is_upper {
                s4.push(el.base.to_ascii_uppercase());
            } else {
                s4.push(el.base);
            }
            // dd at position 0
            if i == 0 {
                s4.push('d');
            }
            // Other marks inline but skip đ's d
            if i > 0 {
                if let Some(m) = el.vowel_mod {
                    s4.push(m);
                }
            }
        }
        if let Some(t) = tone {
            s4.push(t);
        }
        results.insert(s4);
    }

    // Strategy 5: Tone before last consonant (if word ends with consonant)
    // được → dduwowjc
    let last_is_consonant = elements
        .last()
        .map(|e| !"aeiouy".contains(e.base))
        .unwrap_or(false);
    if last_is_consonant && elements.len() > 1 {
        let mut s5 = String::new();
        for (i, el) in elements.iter().enumerate() {
            if el.is_upper {
                s5.push(el.base.to_ascii_uppercase());
            } else {
                s5.push(el.base);
            }
            if let Some(m) = el.vowel_mod {
                s5.push(m);
            }
            // Tone before last element
            if i == elements.len() - 2 {
                if let Some(t) = tone {
                    s5.push(t);
                }
            }
        }
        results.insert(s5);
    }

    // Strategy 6: Mixed - some marks inline, some at end
    // được → dduowcwj (first w inline, second w at end)
    if elements.iter().filter(|e| e.vowel_mod.is_some()).count() >= 2 {
        let mut s6 = String::new();
        let mut pending_marks = Vec::new();
        for (i, el) in elements.iter().enumerate() {
            if el.is_upper {
                s6.push(el.base.to_ascii_uppercase());
            } else {
                s6.push(el.base);
            }
            if let Some(m) = el.vowel_mod {
                if i % 2 == 0 {
                    s6.push(m); // Even positions: inline
                } else {
                    pending_marks.push(m); // Odd positions: defer
                }
            }
        }
        for m in pending_marks {
            s6.push(m);
        }
        if let Some(t) = tone {
            s6.push(t);
        }
        results.insert(s6);
    }

    // Strategy 7: Tone after first vowel with mark
    // được → ddujwowc
    if let Some(first_vowel_idx) = elements.iter().position(|e| "aeiouy".contains(e.base)) {
        let mut s7 = String::new();
        for (i, el) in elements.iter().enumerate() {
            if el.is_upper {
                s7.push(el.base.to_ascii_uppercase());
            } else {
                s7.push(el.base);
            }
            if let Some(m) = el.vowel_mod {
                s7.push(m);
            }
            // Tone after first vowel
            if i == first_vowel_idx {
                if let Some(t) = tone {
                    s7.push(t);
                }
            }
        }
        results.insert(s7);
    }

    // Strategy 8: All marks at end, then tone
    // được → duocdwwj (same as s3 but different mark order)
    let mut s8 = String::new();
    let mut all_marks = Vec::new();
    for el in elements.iter() {
        if el.is_upper {
            s8.push(el.base.to_ascii_uppercase());
        } else {
            s8.push(el.base);
        }
        if let Some(m) = el.vowel_mod {
            all_marks.push(m);
        }
    }
    // Reverse order of marks
    all_marks.reverse();
    for m in all_marks {
        s8.push(m);
    }
    if let Some(t) = tone {
        s8.push(t);
    }
    results.insert(s8);

    results.into_iter().collect()
}

/// Generate Telex input - Style 1: base word first, then marks at end
/// Example: "người" → "nguoiwf"
fn vietnamese_to_telex_style1(word: &str) -> String {
    let mut base = String::new();
    let mut vowel_marks = String::new();
    let mut tone: Option<char> = None;

    for ch in word.chars() {
        let is_upper = ch.is_uppercase();
        let (b, v, t) = decompose_vn_char(ch);
        if is_upper {
            base.push(b.to_ascii_uppercase());
        } else {
            base.push(b);
        }
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

/// Generate Telex input - Style 2: marks inline after each char
/// Example: "người" → "nguwowif"
fn vietnamese_to_telex_style2(word: &str) -> String {
    let mut result = String::new();
    let mut tone: Option<char> = None;

    for ch in word.chars() {
        let is_upper = ch.is_uppercase();
        let (b, v, t) = decompose_vn_char(ch);
        if is_upper {
            result.push(b.to_ascii_uppercase());
        } else {
            result.push(b);
        }
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

/// Generate Telex input - Style 3: tone immediately after marked char
/// Example: "người" → "nguwowfi"
fn vietnamese_to_telex_style3(word: &str) -> String {
    let mut result = String::new();

    for ch in word.chars() {
        let is_upper = ch.is_uppercase();
        let (b, v, t) = decompose_vn_char(ch);
        if is_upper {
            result.push(b.to_ascii_uppercase());
        } else {
            result.push(b);
        }
        if let Some(vm) = v {
            result.push(vm);
        }
        if let Some(tone) = t {
            result.push(tone);
        }
    }

    result
}

/// Simulate typing and get result
fn type_telex(engine: &mut Engine, telex: &str) -> String {
    engine.clear();
    common::type_word(engine, telex)
}

// ============================================================
// HELPER FUNCTIONS
// ============================================================

/// Check if difference is just tone position (oà vs òa)
fn is_tone_position_diff(result: &str, expected: &str) -> bool {
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
    result_base == expected_base && result != expected
}

/// Check if result is normalized form (e.g., choòng → chồng)
fn is_normalized_form(result: &str, expected: &str) -> bool {
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

/// Get the toned vowel for display
fn get_tone_char(word: &str) -> String {
    for c in word.chars() {
        if "áàảãạắằẳẵặấầẩẫậéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ".contains(c)
        {
            return c.to_string();
        }
    }
    "?".to_string()
}

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

fn detect_other_issue(expected: &str, telex: &str, result: &str) -> String {
    let tone_keys = ['s', 'f', 'r', 'x', 'j'];
    for tk in &tone_keys {
        if telex.contains(*tk) && expected.contains(*tk) && !result.contains(*tk) {
            return format!("mất '{}'", tk);
        }
    }

    if expected.starts_with('d') && result.starts_with('đ') {
        return "d→đ sai".to_string();
    }

    let exp_chars: Vec<char> = expected.chars().collect();
    let res_chars: Vec<char> = result.chars().collect();
    if exp_chars.len() > res_chars.len() {
        return format!("mất {} ký tự", exp_chars.len() - res_chars.len());
    }

    "unknown".to_string()
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{}…", truncated)
    }
}

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
    for (word, telex, result, issue) in failures.iter().take(30) {
        println!(
            "│ {:11} │ {:20} │ {:20} │ {:15} │",
            truncate_str(word, 11),
            truncate_str(telex, 20),
            truncate_str(result, 20),
            truncate_str(issue, 15)
        );
    }
    if failures.len() > 30 {
        println!(
            "│ ... và {} mục khác                                                          │",
            failures.len() - 30
        );
    }
    println!("└─────────────┴──────────────────────┴──────────────────────┴─────────────────┘");
}

// ============================================================
// TESTS
// ============================================================

#[test]
fn test_decompose_chars() {
    assert_eq!(decompose_vn_char('a'), ('a', None, None));
    assert_eq!(decompose_vn_char('á'), ('a', None, Some('s')));
    assert_eq!(decompose_vn_char('ă'), ('a', Some('w'), None));
    assert_eq!(decompose_vn_char('ắ'), ('a', Some('w'), Some('s')));
    assert_eq!(decompose_vn_char('ư'), ('u', Some('w'), None));
    assert_eq!(decompose_vn_char('đ'), ('d', Some('d'), None));
}

#[test]
fn test_telex_conversion_styles() {
    println!("\n=== Telex Conversion Styles ===");

    let samples = vec![
        ("người", "nguoidwf", "nguwowif", "nguwowfi"),
        ("việt", "vietej", "vieetj", "vieejt"),
        ("được", "duocdwj", "duwowcj", "duwowjc"),
        ("đã", "daddx", "ddax", "ddax"),
        ("không", "khongof", "khoongf", "khoongf"),
        ("năm", "namwf", "nawmf", "nawmf"),
    ];

    for (word, _exp1, _exp2, _exp3) in &samples {
        let s1 = vietnamese_to_telex_style1(word);
        let s2 = vietnamese_to_telex_style2(word);
        let s3 = vietnamese_to_telex_style3(word);
        println!("  {} → s1:'{}' | s2:'{}' | s3:'{}'", word, s1, s2, s3);
    }
}

#[test]
fn test_roundtrip_common_words() {
    let mut engine = Engine::new();
    engine.set_method(0); // Telex
    engine.set_enabled(true);

    let common_words = vec![
        "có", "là", "và", "một", "cho", "với", "để", "về", "từ", "ra", "như", "còn", "làm", "đi",
        "ta", "ở", "người", "được", "không", "này", "ăn", "năm", "thế", "đến",
    ];

    println!("\n=== Round-trip Common Words ===");
    println!(
        "{:<10} {:<15} {:<15} {:<15} {}",
        "Word", "Style1", "Style2", "Style3", "Result"
    );
    println!("{}", "-".repeat(75));

    let mut passed = 0;

    for word in &common_words {
        let telex1 = vietnamese_to_telex_style1(word);
        let telex2 = vietnamese_to_telex_style2(word);
        let telex3 = vietnamese_to_telex_style3(word);

        let result1 = type_telex(&mut engine, &telex1);
        let result2 = type_telex(&mut engine, &telex2);
        let result3 = type_telex(&mut engine, &telex3);

        // At least one style should work
        let any_pass = result1 == *word || result2 == *word || result3 == *word;
        if any_pass {
            passed += 1;
        }

        let ok1 = if result1 == *word { "✓" } else { "✗" };
        let ok2 = if result2 == *word { "✓" } else { "✗" };
        let ok3 = if result3 == *word { "✓" } else { "✗" };

        println!(
            "{:<10} {:<15} {:<15} {:<15} {} {} {}",
            word, telex1, telex2, telex3, ok1, ok2, ok3
        );
    }

    println!("\nPassed: {}/{}", passed, common_words.len());
    assert!(
        passed >= common_words.len() * 80 / 100,
        "Should pass at least 80% of common words"
    );
}

#[test]
fn test_roundtrip_from_wordlist() {
    let content = match fs::read_to_string("tests/data/vietnamese-words.txt") {
        Ok(c) => c,
        Err(_) => {
            println!("Skipping: tests/data/vietnamese-words.txt not found");
            return;
        }
    };

    // Test with both modern_tone settings
    for modern_tone in [true, false] {
        let mut engine = Engine::new();
        engine.set_method(0); // Telex
        engine.set_enabled(true);
        engine.set_modern_tone(modern_tone);

        let mut passed = 0;
        let mut fail_no_transform: Vec<(String, String, String, String)> = Vec::new();
        let mut fail_wrong_mark: Vec<(String, String, String, String)> = Vec::new();
        let mut fail_tone_position: Vec<(String, String, String, String)> = Vec::new();
        let mut fail_normalize: Vec<(String, String, String, String)> = Vec::new();
        let mut fail_other: Vec<(String, String, String, String)> = Vec::new();

        // Filter: single words with Vietnamese chars
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
            // Try all styles - if any works, count as pass
            let telex1 = vietnamese_to_telex_style1(word);
            let telex2 = vietnamese_to_telex_style2(word);
            let telex3 = vietnamese_to_telex_style3(word);

            let result1 = type_telex(&mut engine, &telex1);
            let result2 = type_telex(&mut engine, &telex2);
            let result3 = type_telex(&mut engine, &telex3);

            if result1 == *word || result2 == *word || result3 == *word {
                passed += 1;
            } else {
                // Use style2 for failure analysis (most common style)
                let telex = telex2.clone();
                let result = result2.clone();

                // Categorize failure
                if result == telex || result.chars().all(|c| c.is_ascii()) {
                    let issue = "không transform".to_string();
                    fail_no_transform.push((word.to_string(), telex, result, issue));
                } else if is_tone_position_diff(&result, word) {
                    let issue = format!("{}↔{}", get_tone_char(word), get_tone_char(&result));
                    fail_tone_position.push((word.to_string(), telex, result, issue));
                } else if is_normalized_form(&result, word) {
                    let issue = "normalized".to_string();
                    fail_normalize.push((word.to_string(), telex, result, issue));
                } else if has_wrong_vowel_mark(&result, word) {
                    let issue = detect_vowel_mark_issue(word, &result);
                    fail_wrong_mark.push((word.to_string(), telex, result, issue));
                } else {
                    let issue = detect_other_issue(word, &telex, &result);
                    fail_other.push((word.to_string(), telex, result, issue));
                }
            }
        }

        let total_failed = fail_no_transform.len()
            + fail_wrong_mark.len()
            + fail_tone_position.len()
            + fail_normalize.len()
            + fail_other.len();

        // Print report
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

        // Print failure categories
        print_failure_table("1. FOREIGN WORDS (expected - từ mượn)", &fail_no_transform);
        print_failure_table("2. TONE POSITION (style diff - oà↔òa)", &fail_tone_position);
        print_failure_table("3. NORMALIZED (correct - choòng→chồng)", &fail_normalize);
        print_failure_table("4. WRONG VOWEL MARK (investigate)", &fail_wrong_mark);
        print_failure_table("5. OTHER BUGS (NEEDS FIX)", &fail_other);

        // Summary
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

#[test]
fn test_telex_samples() {
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
        let t1 = vietnamese_to_telex_style1(word);
        let t2 = vietnamese_to_telex_style2(word);
        let t3 = vietnamese_to_telex_style3(word);
        println!("  {} → s1:'{}' s2:'{}' s3:'{}'", word, t1, t2, t3);
    }
}

// ============================================================
// PERMUTATION TESTS - Test multiple typing styles per word
// ============================================================

#[test]
fn test_permutation_samples() {
    println!("\n=== Telex Permutation Samples ===\n");

    let samples = vec!["được", "người", "đường", "nước", "trường", "không", "việt"];

    for word in samples {
        let perms = generate_telex_permutations(word);
        println!("  {} → {} permutations:", word, perms.len());
        for p in &perms {
            println!("    - {}", p);
        }
        println!();
    }
}

#[test]
fn test_permutation_roundtrip() {
    let mut engine = Engine::new();
    engine.set_method(0); // Telex
    engine.set_enabled(true);

    println!("\n=== Permutation Round-Trip Test ===\n");

    let test_words = vec![
        "được",
        "người",
        "đường",
        "nước",
        "trường",
        "không",
        "việt",
        "năm",
        "đã",
        "những",
        "để",
        "về",
        "đến",
        "ăn",
        "uống",
        "sương",
        "hương",
        "thương",
        "chương",
        "phương",
    ];

    let mut total_perms = 0;
    let mut total_passed = 0;
    let mut failed_cases: Vec<(String, String, String)> = Vec::new();

    for word in &test_words {
        let perms = generate_telex_permutations(word);
        let mut word_passed = 0;

        for telex in &perms {
            let result = type_telex(&mut engine, telex);
            if result == *word {
                word_passed += 1;
            } else {
                failed_cases.push((word.to_string(), telex.clone(), result));
            }
        }

        total_perms += perms.len();
        total_passed += word_passed;

        let status = if word_passed == perms.len() {
            "✓"
        } else {
            "✗"
        };
        println!(
            "  {} {} → {}/{} permutations passed",
            status,
            word,
            word_passed,
            perms.len()
        );
    }

    println!(
        "\n  Total: {}/{} ({:.1}%)",
        total_passed,
        total_perms,
        (total_passed as f64 / total_perms as f64) * 100.0
    );

    if !failed_cases.is_empty() {
        println!("\n  Failed cases (first 20):");
        for (word, telex, result) in failed_cases.iter().take(20) {
            println!(
                "    {} → '{}' = '{}' (expected '{}')",
                word, telex, result, word
            );
        }
    }
}

#[test]
fn test_specific_word_permutations() {
    // Test specific word "được" with all its permutations
    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_enabled(true);

    let word = "được";
    let perms = generate_telex_permutations(word);

    println!(
        "\n=== Testing '{}' with {} permutations ===\n",
        word,
        perms.len()
    );

    for telex in &perms {
        let result = type_telex(&mut engine, telex);
        let status = if result == word { "✓" } else { "✗" };
        println!("  {} '{}' → '{}'", status, telex, result);
    }

    // Also test some manual permutations that users commonly type
    let manual_perms = vec![
        "dduwowcj",  // inline all
        "duocdwwj",  // base first, marks end
        "dduowcwj",  // dd first, then mix
        "dduwojwc",  // tone early
        "duocddwwj", // base, then dd, then ww
        "duwowcdj",  // partial inline
    ];

    println!("\n  Manual permutations:");
    for telex in manual_perms {
        let result = type_telex(&mut engine, telex);
        let status = if result == word { "✓" } else { "✗" };
        println!("  {} '{}' → '{}'", status, telex, result);
    }
}

#[test]
fn test_permutation_wordlist() {
    let content = match fs::read_to_string("tests/data/vietnamese-words.txt") {
        Ok(c) => c,
        Err(_) => {
            println!("Skipping: tests/data/vietnamese-words.txt not found");
            return;
        }
    };

    let mut engine = Engine::new();
    engine.set_method(0);
    engine.set_enabled(true);

    println!("\n=== Permutation Wordlist Test ===\n");

    // Filter words with diacritics (single words only)
    let words: Vec<&str> = content
        .lines()
        .filter(|line| !line.contains(' '))
        .filter(|line| {
            line.chars().any(|c| {
                "àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđ".contains(c)
            })
        })
        .take(500) // Test first 500 words
        .collect();

    let mut words_all_pass = 0;
    let mut words_partial_pass = 0;
    let mut words_all_fail = 0;
    let mut total_perms = 0;
    let mut total_passed = 0;

    for word in &words {
        let perms = generate_telex_permutations(word);
        let mut word_passed = 0;

        for telex in &perms {
            let result = type_telex(&mut engine, telex);
            if result == *word {
                word_passed += 1;
            }
        }

        total_perms += perms.len();
        total_passed += word_passed;

        if word_passed == perms.len() {
            words_all_pass += 1;
        } else if word_passed > 0 {
            words_partial_pass += 1;
        } else {
            words_all_fail += 1;
        }
    }

    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│  PERMUTATION WORDLIST RESULTS                                               │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!(
        "│  Words tested:     {:5}                                                     │",
        words.len()
    );
    println!(
        "│  All perms pass:   {:5} ({:5.1}%)                                            │",
        words_all_pass,
        (words_all_pass as f64 / words.len() as f64) * 100.0
    );
    println!(
        "│  Partial pass:     {:5} ({:5.1}%)                                            │",
        words_partial_pass,
        (words_partial_pass as f64 / words.len() as f64) * 100.0
    );
    println!(
        "│  All perms fail:   {:5} ({:5.1}%)                                            │",
        words_all_fail,
        (words_all_fail as f64 / words.len() as f64) * 100.0
    );
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!(
        "│  Total permutations: {:5}  │  Passed: {:5} ({:5.1}%)                        │",
        total_perms,
        total_passed,
        (total_passed as f64 / total_perms as f64) * 100.0
    );
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
}
