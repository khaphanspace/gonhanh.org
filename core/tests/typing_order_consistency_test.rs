//! Dynamic Typing Order Consistency Test
//!
//! Verifies that ALL valid Telex typing orders for a Vietnamese word
//! produce the SAME result. This is a dynamic test - no hardcoded expected values.
//!
//! Test logic:
//! 1. Given target Vietnamese word (e.g., "mủi")
//! 2. Generate ALL valid Telex typing orders (e.g., "muri", "muir")
//! 3. Type each order and collect results
//! 4. FAIL if any order produces different result
//!
//! This catches bugs where one typing order works but another doesn't.

use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
use std::collections::HashMap;

// =============================================================================
// CHARACTER DECOMPOSITION
// =============================================================================

/// Decompose Vietnamese character into (base, mark, tone)
fn decompose_char(c: char) -> (char, Option<char>, Option<char>) {
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
        // Breve ă
        'ă' => ('a', Some('w'), None),
        'ằ' => ('a', Some('w'), Some('f')),
        'ắ' => ('a', Some('w'), Some('s')),
        'ẳ' => ('a', Some('w'), Some('r')),
        'ẵ' => ('a', Some('w'), Some('x')),
        'ặ' => ('a', Some('w'), Some('j')),
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
        // Plain characters
        _ => (c, None, None),
    }
}

// =============================================================================
// TYPING ORDER GENERATOR
// =============================================================================

/// Generate all valid Telex typing orders for a Vietnamese word
fn generate_typing_orders(word: &str) -> Vec<String> {
    let chars: Vec<char> = word.chars().collect();

    // Decompose each character
    let mut parts: Vec<(char, Option<char>, Option<char>)> = Vec::new();
    for c in &chars {
        parts.push(decompose_char(c.to_ascii_lowercase()));
    }

    // Find vowel indices and tone
    let mut vowel_indices: Vec<usize> = Vec::new();
    let mut tone: Option<char> = None;
    let mut marks: Vec<(usize, char)> = Vec::new(); // (index, mark_char)

    for (i, (base, mark, t)) in parts.iter().enumerate() {
        if "aeiou".contains(*base) {
            vowel_indices.push(i);
        }
        if let Some(m) = mark {
            marks.push((i, *m));
        }
        if t.is_some() && tone.is_none() {
            tone = *t;
        }
    }

    // Build base word (without tone and marks)
    let base_word: String = parts.iter().map(|(b, _, _)| *b).collect();

    // Find final consonant position (after last vowel)
    let final_start = vowel_indices.last().map(|&i| i + 1);
    let has_final = final_start.map(|s| s < parts.len()).unwrap_or(false);

    let mut orders: Vec<String> = Vec::new();

    // Generate typing orders
    if let Some(t) = tone {
        // Tone positions: only after the main vowel and after final consonant
        // The main vowel is determined by Vietnamese phonology rules
        let mut tone_positions: Vec<usize> = Vec::new();

        // Find the main vowel (nucleus) - where tone should be placed
        // Rules:
        // 1. For single vowel: that's the main vowel
        // 2. For diphthong ai/ao/au/ay/oi/ui/uy/ei/eo: first vowel is main (tone can go there)
        // 3. For diphthong ia/ie/iu/ua/ue/uo/oa/oe/ye: second vowel is main
        // 4. For triphthong iêu/ươi/uôi/etc: middle vowel is main

        if vowel_indices.len() == 1 {
            // Single vowel - tone goes after it
            tone_positions.push(vowel_indices[0] + 1);
        } else if vowel_indices.len() == 2 {
            let first = parts[vowel_indices[0]].0;
            let second = parts[vowel_indices[1]].0;
            let second_has_mark = parts[vowel_indices[1]].1.is_some();

            // Diphthongs where FIRST vowel is main (can put tone after first):
            // ai, ay, ao, au, oi, oy, oe, ui, uy, ei, eo, eu
            let first_is_main = matches!(
                (first, second),
                ('a', 'i')
                    | ('a', 'y')
                    | ('a', 'o')
                    | ('a', 'u')
                    | ('o', 'i')
                    | ('o', 'y')
                    | ('o', 'e') // xòe, hòe, lòe, tóe - tone on first vowel
                    | ('u', 'i')
                    | ('u', 'y')
                    | ('e', 'i')
                    | ('e', 'o')
                    | ('e', 'u')
            ) && !second_has_mark; // If second has circumflex (ie→iê), it's the main vowel

            if first_is_main {
                // Can put tone after first or second vowel
                tone_positions.push(vowel_indices[0] + 1);
                tone_positions.push(vowel_indices[1] + 1);
            } else {
                // Second vowel is main - only tone after second
                tone_positions.push(vowel_indices[1] + 1);
            }
        } else if vowel_indices.len() >= 3 {
            // Triphthong - tone goes on middle vowel
            // But for simplicity, just after last vowel for now
            tone_positions.push(vowel_indices[vowel_indices.len() - 1] + 1);
        }

        // Also allow tone after final consonant
        if has_final {
            tone_positions.push(parts.len());
        }

        // Deduplicate
        tone_positions.sort();
        tone_positions.dedup();

        for tone_pos in tone_positions {
            // Insert marks first, then tone
            let mut result = String::new();

            for (i, (base, mark, _)) in parts.iter().enumerate() {
                result.push(*base);

                // Insert mark if this char has one (for circumflex/horn/breve/stroke)
                if let Some(m) = mark {
                    result.push(*m);
                }

                // Insert tone at this position
                if i + 1 == tone_pos {
                    result.push(t);
                }
            }

            // If tone_pos is at the end
            if tone_pos == parts.len() && !result.ends_with(t) {
                result.push(t);
            }

            orders.push(result);
        }
    } else {
        // No tone, just marks
        let mut result = String::new();
        for (base, mark, _) in &parts {
            result.push(*base);
            if let Some(m) = mark {
                result.push(*m);
            }
        }
        orders.push(result);
    }

    // Remove duplicates
    orders.sort();
    orders.dedup();
    orders
}

/// Test a word: all typing orders must produce consistent results
/// Returns (passed, failures) where failures lists inconsistent orders
fn test_word_consistency(word: &str, with_auto_restore: bool) -> (bool, Vec<(String, String)>) {
    let orders = generate_typing_orders(word);

    if orders.is_empty() {
        return (true, vec![]);
    }

    let mut results: HashMap<String, Vec<String>> = HashMap::new();

    for order in &orders {
        let mut engine = Engine::new();
        if with_auto_restore {
            engine.set_english_auto_restore(true);
        }

        let input = if with_auto_restore {
            format!("{} ", order)
        } else {
            order.clone()
        };

        let result = type_word(&mut engine, &input);
        results.entry(result).or_default().push(order.clone());
    }

    // If all orders produce same result, pass
    if results.len() == 1 {
        return (true, vec![]);
    }

    // Find the most common result (likely correct)
    let (expected_result, _) = results
        .iter()
        .max_by_key(|(_, orders)| orders.len())
        .unwrap();

    // Collect failures
    let mut failures: Vec<(String, String)> = Vec::new();
    for (result, orders) in &results {
        if result != expected_result {
            for order in orders {
                failures.push((order.clone(), result.clone()));
            }
        }
    }

    (false, failures)
}

// =============================================================================
// DYNAMIC TESTS
// =============================================================================

#[test]
fn ui_diphthong_consistency() {
    // Test UI diphthong words - all typing orders must be consistent
    let words = ["mủi", "tụi", "núi", "cúi", "đuổi", "tuổi", "muỗi"];

    println!("\n=== UI Diphthong Consistency Test ===\n");

    let mut all_passed = true;

    for word in &words {
        // Test without auto-restore
        let (passed, failures) = test_word_consistency(word, false);
        if !passed {
            all_passed = false;
            println!(
                "FAIL [no restore]: '{}' has inconsistent typing orders:",
                word
            );
            for (order, result) in &failures {
                println!("  '{}' → '{}'", order, result);
            }
        }

        // Test with auto-restore
        let (passed, failures) = test_word_consistency(word, true);
        if !passed {
            all_passed = false;
            println!(
                "FAIL [auto-restore]: '{}' has inconsistent typing orders:",
                word
            );
            for (order, result) in &failures {
                println!("  '{}' → '{}'", order, result);
            }
        }
    }

    if all_passed {
        println!("All UI diphthong words passed!");
    }

    assert!(
        all_passed,
        "Some typing orders produced inconsistent results"
    );
}

#[test]
fn common_words_consistency() {
    // Test common Vietnamese words
    let words = [
        // Simple tones
        "là", "và", "có", "để", "từ", "về", // Diphthongs with tones
        "của", "được", "người", "những", // UI diphthong
        "tụi", "mủi", "núi", "cúi", // Complex
        "không", "cũng", "như", "này",
    ];

    println!("\n=== Common Words Consistency Test ===\n");

    let mut total = 0;
    let mut failed = 0;

    for word in &words {
        total += 1;

        // Test with auto-restore (more strict)
        let (passed, failures) = test_word_consistency(word, true);
        if !passed {
            failed += 1;
            println!("FAIL: '{}' has inconsistent typing orders:", word);
            for (order, result) in &failures {
                println!("  '{}' → '{}'", order, result);
            }
        }
    }

    println!("\nResults: {}/{} passed", total - failed, total);

    assert_eq!(
        failed, 0,
        "{} words have inconsistent typing orders",
        failed
    );
}

#[test]
fn tone_position_consistency() {
    // Test that tone can be placed at different valid positions
    let words = [
        // Tone after first vowel vs after second vowel vs after final
        "toán", // to-a-n with s tone
        "hoàn", // ho-a-n with f tone
        "được", // du-o-c with j tone
    ];

    println!("\n=== Tone Position Consistency Test ===\n");

    let mut all_passed = true;

    for word in &words {
        let orders = generate_typing_orders(word);
        println!("'{}' typing orders: {:?}", word, orders);

        let (passed, failures) = test_word_consistency(word, true);
        if !passed {
            all_passed = false;
            println!("FAIL: '{}' has inconsistent results:", word);
            for (order, result) in &failures {
                println!("  '{}' → '{}'", order, result);
            }
        }
    }

    assert!(
        all_passed,
        "Some tone positions produced inconsistent results"
    );
}

// =============================================================================
// 22K DICTIONARY TEST
// =============================================================================

#[test]
#[ignore] // Run with: cargo test test_22k_consistency -- --ignored --nocapture
fn test_22k_consistency() {
    let dict_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/vietnamese_22k.txt");
    let content = std::fs::read_to_string(dict_path).expect("Failed to read dictionary");

    // Only test single-syllable words (no spaces)
    // Compound words need different handling (each syllable separately)
    let words: Vec<&str> = content
        .lines()
        .filter(|w| !w.is_empty() && !w.contains(' '))
        .collect();

    println!("\n=== 22K Dictionary Consistency Test (Single Syllables) ===");
    println!("Testing {} single-syllable words...\n", words.len());

    let mut total = 0;
    let mut failed = 0;
    let mut failures_detail: Vec<(String, Vec<(String, String)>)> = Vec::new();

    for word in &words {
        total += 1;

        let (passed, failures) = test_word_consistency(word, true);
        if !passed {
            failed += 1;
            if failures_detail.len() < 50 {
                failures_detail.push((word.to_string(), failures));
            }
        }
    }

    println!("\n=== Results ===");
    println!("Total: {}", total);
    println!("Passed: {}", total - failed);
    println!("Failed: {}", failed);
    println!(
        "Pass rate: {:.2}%",
        (total - failed) as f64 / total as f64 * 100.0
    );

    if !failures_detail.is_empty() {
        println!("\n=== Sample Failures ===");
        for (word, failures) in &failures_detail {
            println!("\n'{}' inconsistent orders:", word);
            for (order, result) in failures {
                println!("  '{}' → '{}'", order, result);
            }
        }
    }

    // Require at least 95% consistency
    let pass_rate = (total - failed) as f64 / total as f64;
    assert!(
        pass_rate >= 0.95,
        "Pass rate {:.2}% is below 95% threshold",
        pass_rate * 100.0
    );
}
