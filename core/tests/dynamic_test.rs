//! Dynamic Permutation Testing
//!
//! Generates test cases dynamically from Vietnamese syllable components.
//! Tests modifier order flexibility: same output regardless of typing order.

mod common;
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

// =============================================================================
// VIETNAMESE SYLLABLE COMPONENTS
// =============================================================================

const COMMON_INITIALS: &[&str] = &["b", "c", "d", "g", "h", "l", "m", "n", "s", "t"];
const DIPHTHONGS: &[(&str, &str)] = &[("a", "o"), ("a", "i"), ("o", "i"), ("u", "a")];
const TONES: &[&str] = &["s", "f", "r", "x", "j"];

// =============================================================================
// HELPERS
// =============================================================================

fn apply_tone(v: char, t: &str) -> char {
    match (v, t) {
        ('a', "s") => 'á',
        ('a', "f") => 'à',
        ('a', "r") => 'ả',
        ('a', "x") => 'ã',
        ('a', "j") => 'ạ',
        ('e', "s") => 'é',
        ('e', "f") => 'è',
        ('e', "r") => 'ẻ',
        ('e', "x") => 'ẽ',
        ('e', "j") => 'ẹ',
        ('o', "s") => 'ó',
        ('o', "f") => 'ò',
        ('o', "r") => 'ỏ',
        ('o', "x") => 'õ',
        ('o', "j") => 'ọ',
        ('u', "s") => 'ú',
        ('u', "f") => 'ù',
        ('u', "r") => 'ủ',
        ('u', "x") => 'ũ',
        ('u', "j") => 'ụ',
        _ => v,
    }
}

fn test(input: &str, expected: &str) -> Result<(), String> {
    let mut e = Engine::new();
    let result = type_word(&mut e, input);
    if result == expected {
        Ok(())
    } else {
        Err(format!(
            "'{}' → '{}' (expected '{}')",
            input, result, expected
        ))
    }
}

fn test_auto_restore(input: &str, expected: &str) -> Result<(), String> {
    let mut e = Engine::new();
    e.set_english_auto_restore(true);
    let result = type_word(&mut e, input);
    if result == expected {
        Ok(())
    } else {
        Err(format!(
            "'{}' → '{}' (expected '{}')",
            input, result, expected
        ))
    }
}

// =============================================================================
// TEST 1: DIPHTHONG + TONE ORDER
// "naof" and "nafo" → "nào"
// =============================================================================

#[test]
fn dynamic_diphthong_tone_order() {
    let mut errors = vec![];
    for initial in COMMON_INITIALS {
        for (v1, v2) in DIPHTHONGS {
            for tone in TONES {
                let expected = format!(
                    "{}{}{} ",
                    initial,
                    apply_tone(v1.chars().next().unwrap(), tone),
                    v2
                );
                // Standard vs permutation
                if let Err(e) = test(&format!("{}{}{}{} ", initial, v1, v2, tone), &expected) {
                    errors.push(e);
                }
                if let Err(e) = test(&format!("{}{}{}{} ", initial, v1, tone, v2), &expected) {
                    errors.push(e);
                }
            }
        }
    }
    println!(
        "\n=== Diphthong + Tone: {} generated, {} failed ===",
        COMMON_INITIALS.len() * DIPHTHONGS.len() * TONES.len() * 2,
        errors.len()
    );
    assert!(errors.is_empty(), "{} tests failed", errors.len());
}

// =============================================================================
// TEST 2: TONE + FINAL ORDER (sắc tone, valid combinations only)
// "mans" and "masn" → "mán"
// =============================================================================

#[test]
fn dynamic_tone_final_order() {
    let mut errors = vec![];
    // Valid Vietnamese syllable patterns: initial + vowel + final
    let valid_patterns: &[(&str, &str, &str)] = &[
        // a + finals
        ("b", "a", "c"),
        ("b", "a", "m"),
        ("b", "a", "n"),
        ("b", "a", "t"),
        ("c", "a", "m"),
        ("c", "a", "n"),
        ("c", "a", "p"),
        ("c", "a", "t"),
        ("l", "a", "c"),
        ("l", "a", "m"),
        ("l", "a", "n"),
        ("l", "a", "p"),
        ("l", "a", "t"),
        ("m", "a", "c"),
        ("m", "a", "n"),
        ("m", "a", "p"),
        ("m", "a", "t"),
        ("n", "a", "m"),
        ("n", "a", "p"),
        ("t", "a", "c"),
        ("t", "a", "m"),
        ("t", "a", "n"),
        ("t", "a", "p"),
        ("t", "a", "t"),
        // o + finals
        ("b", "o", "c"),
        ("b", "o", "m"),
        ("b", "o", "n"),
        ("b", "o", "p"),
        ("c", "o", "m"),
        ("c", "o", "n"),
        ("c", "o", "p"),
        ("c", "o", "t"),
        ("l", "o", "c"),
        ("l", "o", "m"),
        ("l", "o", "n"),
        ("l", "o", "t"),
        ("m", "o", "c"),
        ("m", "o", "n"),
        ("m", "o", "t"),
        ("n", "o", "m"),
        ("n", "o", "n"),
        ("n", "o", "p"),
        ("t", "o", "c"),
        ("t", "o", "m"),
        ("t", "o", "n"),
        ("t", "o", "p"),
        ("t", "o", "t"),
    ];

    for (initial, vowel, final_c) in valid_patterns {
        let toned = apply_tone(vowel.chars().next().unwrap(), "s");
        let expected = format!("{}{}{} ", initial, toned, final_c);
        // Standard: vowel + final + tone
        if let Err(e) = test(&format!("{}{}{}s ", initial, vowel, final_c), &expected) {
            errors.push(e);
        }
        // Permutation: vowel + tone + final
        if let Err(e) = test(&format!("{}{}s{} ", initial, vowel, final_c), &expected) {
            errors.push(e);
        }
    }
    println!(
        "\n=== Tone + Final: {} generated, {} failed ===",
        valid_patterns.len() * 2,
        errors.len()
    );
    assert!(errors.is_empty(), "{} tests failed", errors.len());
}

// =============================================================================
// TEST 3: HORN ORDER (owi vs oiw)
// =============================================================================

#[test]
fn dynamic_horn_oi_order() {
    let mut errors = vec![];
    for initial in &["", "b", "c", "d", "g", "h", "l", "m", "n", "t"] {
        let expected = format!("{}ơi ", initial);
        if let Err(e) = test(&format!("{}owi ", initial), &expected) {
            errors.push(e);
        }
        if let Err(e) = test(&format!("{}oiw ", initial), &expected) {
            errors.push(e);
        }
    }
    println!("\n=== Horn OI: 20 generated, {} failed ===", errors.len());
    assert!(errors.is_empty(), "{} tests failed", errors.len());
}

// =============================================================================
// TEST 4: CIRCUMFLEX SPLIT (rieeng vs rieneg)
// =============================================================================

#[test]
fn dynamic_circumflex_split() {
    let mut errors = vec![];
    // (input_initial, output_initial) - "dd" → "đ" in output
    let initials: &[(&str, &str)] = &[
        ("b", "b"),
        ("ch", "ch"),
        ("d", "d"),
        ("dd", "đ"),
        ("g", "g"),
        ("k", "k"),
        ("l", "l"),
        ("m", "m"),
        ("n", "n"),
        ("r", "r"),
        ("t", "t"),
    ];

    for (input_i, output_i) in initials {
        let expected = format!("{}iêng ", output_i);
        if let Err(e) = test(&format!("{}ieeng ", input_i), &expected) {
            errors.push(e);
        }
        if let Err(e) = test(&format!("{}ieneg ", input_i), &expected) {
            errors.push(e);
        }
    }
    println!(
        "\n=== Circumflex Split: 22 generated, {} failed ===",
        errors.len()
    );
    assert!(errors.is_empty(), "{} tests failed", errors.len());
}

// =============================================================================
// TEST 5: ALL TONES × AI DIPHTHONG
// =============================================================================

#[test]
fn dynamic_all_tones_ai() {
    let mut errors = vec![];
    for initial in COMMON_INITIALS {
        for tone in TONES {
            let expected = format!("{}{}i ", initial, apply_tone('a', tone));
            if let Err(e) = test(&format!("{}ai{} ", initial, tone), &expected) {
                errors.push(e);
            }
            if let Err(e) = test(&format!("{}a{}i ", initial, tone), &expected) {
                errors.push(e);
            }
        }
    }
    println!(
        "\n=== All Tones AI: 100 generated, {} failed ===",
        errors.len()
    );
    assert!(errors.is_empty(), "{} tests failed", errors.len());
}

// =============================================================================
// TEST 6: ENGLISH AUTO-RESTORE
// =============================================================================

#[test]
fn dynamic_english_restore() {
    let english = &[
        "view", "raw", "law", "saw", "new", "few", "data", "half", "wolf", "golf",
        // SW words
        "sweet", "swim", "switch", "swift", "swing", "swear", "sword", "swipe",
    ];
    let mut errors = vec![];
    for word in english {
        let input = format!("{} ", word);
        if let Err(e) = test_auto_restore(&input, &input) {
            errors.push(e);
        }
    }
    println!(
        "\n=== English Restore: {} generated, {} failed ===",
        english.len(),
        errors.len()
    );
    for e in &errors {
        println!("  FAIL: {}", e);
    }
    assert!(errors.is_empty(), "{} tests failed", errors.len());
}

// =============================================================================
// TEST 7: TONE AFTER COMPLETE WORD
// "nhanaj" → "nhận", "lamf" → "làm", etc.
// =============================================================================

#[test]
fn dynamic_tone_after_word() {
    let mut errors = vec![];

    // Pattern: complete word + tone at end
    // (input, expected) - tone typed AFTER the word is complete
    let patterns: &[(&str, &str, &str, &str)] = &[
        // (initial, vowel, final, expected_vowel_with_tone) for each tone
        // Sắc (s)
        ("nh", "a", "n", "á"), // nhans → nhán
        ("l", "a", "m", "á"),  // lams → lám
        ("c", "a", "n", "á"),  // cans → cán
        ("m", "a", "t", "á"),  // mats → mát
        ("b", "a", "c", "á"),  // bacs → bác
        // Huyền (f)
        ("nh", "a", "n", "à"), // nhanf → nhàn
        ("l", "a", "m", "à"),  // lamf → làm
        ("c", "a", "n", "à"),  // canf → càn
        // Hỏi (r)
        ("nh", "a", "n", "ả"), // nhanr → nhản
        ("c", "a", "m", "ả"),  // camr → cảm
        // Ngã (x)
        ("nh", "a", "n", "ã"), // nhanx → nhãn
        // Nặng (j)
        ("nh", "a", "n", "ạ"), // nhanj → nhạn
        ("nh", "a", "t", "ậ"), // nhatj → nhật (with circumflex!)
    ];

    // Test each pattern with its corresponding tone
    let tone_map: &[(&str, &str)] = &[
        ("á", "s"),
        ("à", "f"),
        ("ả", "r"),
        ("ã", "x"),
        ("ạ", "j"),
        ("ậ", "j"), // Special: â + j = ậ
    ];

    for (initial, vowel, final_c, expected_vowel) in patterns {
        // Find the tone key for this expected vowel
        let tone_key = tone_map
            .iter()
            .find(|(v, _)| v == expected_vowel)
            .map(|(_, t)| *t)
            .unwrap_or("s");

        // Special handling for circumflex vowels
        let (input_vowel, output_vowel) = if *expected_vowel == "ậ" {
            ("aa", "ậ") // nhaat + j → nhật
        } else {
            (*vowel, *expected_vowel)
        };

        let input = format!("{}{}{}{} ", initial, input_vowel, final_c, tone_key);
        let expected = format!("{}{}{} ", initial, output_vowel, final_c);

        if let Err(e) = test(&input, &expected) {
            errors.push(e);
        }
    }

    // Additional common words with tone at end
    let common_words: &[(&str, &str)] = &[
        ("nhanaj ", "nhận "), // nhận (receive) - â + nặng
        ("lamf ", "làm "),    // làm (do)
        ("camr ", "cảm "),    // cảm (feel)
        ("canf ", "càn "),    // càn
        ("bacs ", "bác "),    // bác (uncle)
        ("mats ", "mát "),    // mát (cool)
        ("banf ", "bàn "),    // bàn (table)
        ("sanf ", "sàn "),    // sàn (floor)
        ("tanf ", "tàn "),    // tàn (wither)
        ("lanf ", "làn "),    // làn (lane)
        ("namf ", "nàm "),    // nàm
        ("tamf ", "tàm "),    // tàm
        // With circumflex
        ("taanf ", "tần "), // tần
        ("baans ", "bấn "), // bấn
        ("caanf ", "cần "), // cần (need)
        ("laanf ", "lần "), // lần (time/turn)
        // With double final
        ("langf ", "làng "), // làng (village)
        ("mangf ", "màng "), // màng (membrane)
        ("bangf ", "bàng "), // bàng
    ];

    for (input, expected) in common_words {
        if let Err(e) = test(input, expected) {
            errors.push(e);
        }
    }

    println!(
        "\n=== Tone After Word: {} tests, {} failed ===",
        patterns.len() + common_words.len(),
        errors.len()
    );
    for e in errors.iter().take(10) {
        println!("  FAIL: {}", e);
    }
    assert!(errors.is_empty(), "{} tests failed", errors.len());
}

// =============================================================================
// TEST 8: REGRESSION (Fixed bugs)
// =============================================================================

#[test]
fn dynamic_regression() {
    let cases = &[
        ("oiw ", "ơi "),
        ("owi ", "ơi "),
        ("nafo ", "nào "),
        ("naof ", "nào "),
        ("rieneg ", "riêng "),
        ("rieeng ", "riêng "),
        ("cunxg ", "cũng "),
    ];
    let mut errors = vec![];
    for (input, expected) in cases {
        if let Err(e) = test(input, expected) {
            errors.push(e);
        }
    }
    println!(
        "\n=== Regression: {} tests, {} failed ===",
        cases.len(),
        errors.len()
    );
    assert!(
        errors.is_empty(),
        "{} regression tests failed",
        errors.len()
    );
}

// =============================================================================
// SUMMARY
// =============================================================================

#[test]
fn dynamic_summary() {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║           DYNAMIC TEST: Generated from Arrays                 ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Diphthong+Tone: {} initials × {} diphthongs × {} tones × 2   ║",
        COMMON_INITIALS.len(),
        DIPHTHONGS.len(),
        TONES.len()
    );
    println!("║  Tone+Final:     ~50 valid patterns × 2 orders                ║");
    println!("║  Horn OI:        10 initials × 2 orders                       ║");
    println!("║  Circumflex:     11 initials × 2 patterns                     ║");
    println!("║  All Tones AI:   10 initials × 5 tones × 2 orders             ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
}
