//! Test Vietnamese 22k word list.
//! Converts Vietnamese words to Telex input and verifies engine output.

use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
use std::fs::File;
use std::io::Write;

/// Get base character and modifiers (mark, tone) for Vietnamese character
fn decompose_vn_char(c: char) -> (char, Option<char>, Option<char>) {
    // Returns (base_char, mark_char, tone_char)
    // mark_char: 'a' for â, 'w' for ă/ơ/ư, 'e' for ê, 'o' for ô
    // tone_char: 's' sắc, 'f' huyền, 'r' hỏi, 'x' ngã, 'j' nặng
    match c {
        // Plain vowels with tones
        'à' => ('a', None, Some('f')),
        'á' => ('a', None, Some('s')),
        'ả' => ('a', None, Some('r')),
        'ã' => ('a', None, Some('x')),
        'ạ' => ('a', None, Some('j')),
        'è' => ('e', None, Some('f')),
        'é' => ('e', None, Some('s')),
        'ẻ' => ('e', None, Some('r')),
        'ẽ' => ('e', None, Some('x')),
        'ẹ' => ('e', None, Some('j')),
        'ì' => ('i', None, Some('f')),
        'í' => ('i', None, Some('s')),
        'ỉ' => ('i', None, Some('r')),
        'ĩ' => ('i', None, Some('x')),
        'ị' => ('i', None, Some('j')),
        'ò' => ('o', None, Some('f')),
        'ó' => ('o', None, Some('s')),
        'ỏ' => ('o', None, Some('r')),
        'õ' => ('o', None, Some('x')),
        'ọ' => ('o', None, Some('j')),
        'ù' => ('u', None, Some('f')),
        'ú' => ('u', None, Some('s')),
        'ủ' => ('u', None, Some('r')),
        'ũ' => ('u', None, Some('x')),
        'ụ' => ('u', None, Some('j')),
        'ỳ' => ('y', None, Some('f')),
        'ý' => ('y', None, Some('s')),
        'ỷ' => ('y', None, Some('r')),
        'ỹ' => ('y', None, Some('x')),
        'ỵ' => ('y', None, Some('j')),
        // Circumflex â
        'â' => ('a', Some('a'), None),
        'ầ' => ('a', Some('a'), Some('f')),
        'ấ' => ('a', Some('a'), Some('s')),
        'ẩ' => ('a', Some('a'), Some('r')),
        'ẫ' => ('a', Some('a'), Some('x')),
        'ậ' => ('a', Some('a'), Some('j')),
        // Breve ă
        'ă' => ('a', Some('w'), None),
        'ằ' => ('a', Some('w'), Some('f')),
        'ắ' => ('a', Some('w'), Some('s')),
        'ẳ' => ('a', Some('w'), Some('r')),
        'ẵ' => ('a', Some('w'), Some('x')),
        'ặ' => ('a', Some('w'), Some('j')),
        // Circumflex ê
        'ê' => ('e', Some('e'), None),
        'ề' => ('e', Some('e'), Some('f')),
        'ế' => ('e', Some('e'), Some('s')),
        'ể' => ('e', Some('e'), Some('r')),
        'ễ' => ('e', Some('e'), Some('x')),
        'ệ' => ('e', Some('e'), Some('j')),
        // Circumflex ô
        'ô' => ('o', Some('o'), None),
        'ồ' => ('o', Some('o'), Some('f')),
        'ố' => ('o', Some('o'), Some('s')),
        'ổ' => ('o', Some('o'), Some('r')),
        'ỗ' => ('o', Some('o'), Some('x')),
        'ộ' => ('o', Some('o'), Some('j')),
        // Horn ơ
        'ơ' => ('o', Some('w'), None),
        'ờ' => ('o', Some('w'), Some('f')),
        'ớ' => ('o', Some('w'), Some('s')),
        'ở' => ('o', Some('w'), Some('r')),
        'ỡ' => ('o', Some('w'), Some('x')),
        'ợ' => ('o', Some('w'), Some('j')),
        // Horn ư
        'ư' => ('u', Some('w'), None),
        'ừ' => ('u', Some('w'), Some('f')),
        'ứ' => ('u', Some('w'), Some('s')),
        'ử' => ('u', Some('w'), Some('r')),
        'ữ' => ('u', Some('w'), Some('x')),
        'ự' => ('u', Some('w'), Some('j')),
        // Stroke đ
        'đ' => ('d', Some('d'), None),
        // Uppercase
        'À' => ('A', None, Some('f')),
        'Á' => ('A', None, Some('s')),
        'Ả' => ('A', None, Some('r')),
        'Ã' => ('A', None, Some('x')),
        'Ạ' => ('A', None, Some('j')),
        'Â' => ('A', Some('a'), None),
        'Ầ' => ('A', Some('a'), Some('f')),
        'Ấ' => ('A', Some('a'), Some('s')),
        'Ẩ' => ('A', Some('a'), Some('r')),
        'Ẫ' => ('A', Some('a'), Some('x')),
        'Ậ' => ('A', Some('a'), Some('j')),
        'Ă' => ('A', Some('w'), None),
        'Ằ' => ('A', Some('w'), Some('f')),
        'Ắ' => ('A', Some('w'), Some('s')),
        'Ẳ' => ('A', Some('w'), Some('r')),
        'Ẵ' => ('A', Some('w'), Some('x')),
        'Ặ' => ('A', Some('w'), Some('j')),
        'È' => ('E', None, Some('f')),
        'É' => ('E', None, Some('s')),
        'Ẻ' => ('E', None, Some('r')),
        'Ẽ' => ('E', None, Some('x')),
        'Ẹ' => ('E', None, Some('j')),
        'Ê' => ('E', Some('e'), None),
        'Ề' => ('E', Some('e'), Some('f')),
        'Ế' => ('E', Some('e'), Some('s')),
        'Ể' => ('E', Some('e'), Some('r')),
        'Ễ' => ('E', Some('e'), Some('x')),
        'Ệ' => ('E', Some('e'), Some('j')),
        'Ì' => ('I', None, Some('f')),
        'Í' => ('I', None, Some('s')),
        'Ỉ' => ('I', None, Some('r')),
        'Ĩ' => ('I', None, Some('x')),
        'Ị' => ('I', None, Some('j')),
        'Ò' => ('O', None, Some('f')),
        'Ó' => ('O', None, Some('s')),
        'Ỏ' => ('O', None, Some('r')),
        'Õ' => ('O', None, Some('x')),
        'Ọ' => ('O', None, Some('j')),
        'Ô' => ('O', Some('o'), None),
        'Ồ' => ('O', Some('o'), Some('f')),
        'Ố' => ('O', Some('o'), Some('s')),
        'Ổ' => ('O', Some('o'), Some('r')),
        'Ỗ' => ('O', Some('o'), Some('x')),
        'Ộ' => ('O', Some('o'), Some('j')),
        'Ơ' => ('O', Some('w'), None),
        'Ờ' => ('O', Some('w'), Some('f')),
        'Ớ' => ('O', Some('w'), Some('s')),
        'Ở' => ('O', Some('w'), Some('r')),
        'Ỡ' => ('O', Some('w'), Some('x')),
        'Ợ' => ('O', Some('w'), Some('j')),
        'Ù' => ('U', None, Some('f')),
        'Ú' => ('U', None, Some('s')),
        'Ủ' => ('U', None, Some('r')),
        'Ũ' => ('U', None, Some('x')),
        'Ụ' => ('U', None, Some('j')),
        'Ư' => ('U', Some('w'), None),
        'Ừ' => ('U', Some('w'), Some('f')),
        'Ứ' => ('U', Some('w'), Some('s')),
        'Ử' => ('U', Some('w'), Some('r')),
        'Ữ' => ('U', Some('w'), Some('x')),
        'Ự' => ('U', Some('w'), Some('j')),
        'Ỳ' => ('Y', None, Some('f')),
        'Ý' => ('Y', None, Some('s')),
        'Ỷ' => ('Y', None, Some('r')),
        'Ỹ' => ('Y', None, Some('x')),
        'Ỵ' => ('Y', None, Some('j')),
        'Đ' => ('D', Some('d'), None),
        // No transformation needed
        _ => (c, None, None),
    }
}

/// Convert Vietnamese word to Telex input (tone at end of word)
fn vn_to_telex(word: &str) -> String {
    let mut base = String::new();
    let mut tone: Option<char> = None;

    for c in word.chars() {
        let (base_char, mark, char_tone) = decompose_vn_char(c);
        base.push(base_char);
        if let Some(m) = mark {
            base.push(m);
        }
        // Keep the last tone marker (words should only have one tone)
        if char_tone.is_some() {
            tone = char_tone;
        }
    }

    // Append tone at the end
    if let Some(t) = tone {
        base.push(t);
    }

    base
}

/// Convert between traditional and modern oa/oe tone placement
/// Traditional: hoá (tone on 'a'), Modern: hóa (tone on 'o')
fn to_modern_tone(word: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = word.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        // Check for oa/oe with tone on second vowel (traditional)
        if i + 1 < chars.len() && (c == 'o' || c == 'O') {
            let next = chars[i + 1];
            // Map: traditional a/e with tone → modern o with tone + plain a/e
            let (new_o, new_next) = match next {
                'à' => ('ò', 'a'),
                'á' => ('ó', 'a'),
                'ả' => ('ỏ', 'a'),
                'ã' => ('õ', 'a'),
                'ạ' => ('ọ', 'a'),
                'è' => ('ò', 'e'),
                'é' => ('ó', 'e'),
                'ẻ' => ('ỏ', 'e'),
                'ẽ' => ('õ', 'e'),
                'ẹ' => ('ọ', 'e'),
                'À' => ('Ò', 'A'),
                'Á' => ('Ó', 'A'),
                'Ả' => ('Ỏ', 'A'),
                'Ã' => ('Õ', 'A'),
                'Ạ' => ('Ọ', 'A'),
                'È' => ('Ò', 'E'),
                'É' => ('Ó', 'E'),
                'Ẻ' => ('Ỏ', 'E'),
                'Ẽ' => ('Õ', 'E'),
                'Ẹ' => ('Ọ', 'E'),
                _ => {
                    result.push(c);
                    i += 1;
                    continue;
                }
            };
            // Handle uppercase O
            let final_o = if c == 'O' {
                new_o.to_uppercase().next().unwrap_or(new_o)
            } else {
                new_o
            };
            result.push(final_o);
            result.push(new_next);
            i += 2;
        } else {
            result.push(c);
            i += 1;
        }
    }
    result
}

/// Check if two words match (considering both traditional and modern tone styles)
fn matches_either_style(expected: &str, actual: &str) -> bool {
    if expected == actual {
        return true;
    }
    // Convert expected to modern style and compare
    let modern_expected = to_modern_tone(expected);
    modern_expected == actual
}

#[test]
fn vietnamese_22k_coverage() {
    let content = include_str!("data/vietnamese_22k.txt");
    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<(String, String, String, String)> = Vec::new(); // (word, telex, expected, actual)

    for line in content.lines() {
        let word = line.trim();
        if word.is_empty() {
            continue;
        }

        // Skip compound words (with spaces) for now - test single syllables
        if word.contains(' ') {
            continue;
        }

        // Skip loan words with double 'o' pattern (boong, soong, chòong, etc.)
        // These require typing 'ooo' to get 'oo' output, dict format is incorrect
        // Check for oo, òo, óo, ỏo, õo, ọo patterns
        let has_double_o = word.contains("oo")
            || word.contains("òo")
            || word.contains("óo")
            || word.contains("ỏo")
            || word.contains("õo")
            || word.contains("ọo")
            || word.contains("ồo")
            || word.contains("ốo")
            || word.contains("ổo")
            || word.contains("ỗo")
            || word.contains("ộo");
        if has_double_o {
            continue;
        }

        let telex_input = vn_to_telex(word);
        let input_with_space = format!("{} ", telex_input);
        let expected = format!("{} ", word);

        let mut e = Engine::new();
        e.set_modern_tone(false);
        e.set_english_auto_restore(true);
        let result = type_word(&mut e, &input_with_space);

        // Check if result matches expected (either traditional or modern style)
        if matches_either_style(expected.trim(), result.trim()) {
            passed += 1;
        } else {
            failed += 1;
            failures.push((
                word.to_string(),
                telex_input,
                expected.trim().to_string(),
                result.trim().to_string(),
            ));
        }
    }

    let total = passed + failed;
    let pass_rate = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    println!("\n=== Vietnamese 22k Test Results ===");
    println!("Total single syllables: {}", total);
    println!("Passed: {} ({:.2}%)", passed, pass_rate);
    println!("Failed: {}", failed);

    if !failures.is_empty() {
        println!("\n=== First {} Failures ===", failures.len().min(100));
        println!(
            "{:<15} {:<20} {:<15} {:<15}",
            "WORD", "TELEX", "EXPECTED", "ACTUAL"
        );
        for (word, telex, expected, actual) in failures.iter().take(100) {
            println!("{:<15} {:<20} {:<15} {:<15}", word, telex, expected, actual);
        }
    }

    // Write failures to file
    if let Ok(mut f) = File::create("tests/data/vietnamese_failures.txt") {
        for (word, telex, expected, actual) in &failures {
            let _ = writeln!(f, "{}\t{}\t{}\t{}", word, telex, expected, actual);
        }
        println!("\nFailures written to tests/data/vietnamese_failures.txt");
    }

    // CI threshold: fail if pass rate drops below 99.5%
    const MIN_PASS_RATE: f64 = 99.5;
    assert!(
        pass_rate >= MIN_PASS_RATE,
        "Vietnamese single syllable pass rate {:.2}% is below threshold {:.1}%",
        pass_rate,
        MIN_PASS_RATE
    );
}

#[test]
fn vietnamese_22k_compound() {
    let content = include_str!("data/vietnamese_22k.txt");
    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<(String, String, String, String)> = Vec::new();

    for line in content.lines() {
        let word = line.trim();
        if word.is_empty() || !word.contains(' ') {
            continue;
        }

        // For compound words, test each syllable separately
        let syllables: Vec<&str> = word.split_whitespace().collect();
        let mut all_passed = true;
        let mut telex_parts = Vec::new();
        let mut actual_parts = Vec::new();

        for syllable in &syllables {
            let telex_input = vn_to_telex(syllable);
            telex_parts.push(telex_input.clone());
            let input_with_space = format!("{} ", telex_input);
            let expected = format!("{} ", syllable);

            let mut e = Engine::new();
            e.set_modern_tone(false);
            e.set_english_auto_restore(true);
            let result = type_word(&mut e, &input_with_space);

            actual_parts.push(result.trim().to_string());
            if result != expected {
                all_passed = false;
            }
        }

        if all_passed {
            passed += 1;
        } else {
            failed += 1;
            failures.push((
                word.to_string(),
                telex_parts.join(" "),
                word.to_string(),
                actual_parts.join(" "),
            ));
        }
    }

    let total = passed + failed;
    let pass_rate = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    println!("\n=== Vietnamese 22k Compound Words Test ===");
    println!("Total compound words: {}", total);
    println!("Passed: {} ({:.2}%)", passed, pass_rate);
    println!("Failed: {}", failed);

    if !failures.is_empty() {
        println!("\n=== First {} Failures ===", failures.len().min(100));
        for (word, telex, expected, actual) in failures.iter().take(100) {
            println!(
                "'{}' (telex: '{}') → expected '{}', got '{}'",
                word, telex, expected, actual
            );
        }
    }

    // CI threshold: fail if pass rate drops below 99.0%
    const MIN_PASS_RATE: f64 = 99.0;
    assert!(
        pass_rate >= MIN_PASS_RATE,
        "Vietnamese compound words pass rate {:.2}% is below threshold {:.1}%",
        pass_rate,
        MIN_PASS_RATE
    );
}
