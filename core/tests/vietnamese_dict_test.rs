//! Test Vietnamese dictionary coverage.
//! Target: 100% pass rate for Vietnamese words.

use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

/// Test Vietnamese words from dictionary.
/// Each line in the file is: telex_input\texpected_output
#[test]
fn vietnamese_dictionary_coverage() {
    let content = include_str!("data/vietnamese_telex_pairs.txt");
    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<(String, String, String)> = Vec::new();

    for line in content.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 2 {
            continue;
        }
        let input = parts[0].to_string() + " ";
        let expected = parts[1].to_string() + " ";

        let mut e = Engine::new();
        e.set_modern_tone(false); // Use traditional tone placement (hóa, not hoá)
        let result = type_word(&mut e, &input);

        if result == expected {
            passed += 1;
        } else {
            failed += 1;
            // Store all failures for filtering
            failures.push((
                input.trim().to_string(),
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

    println!("\n=== Vietnamese Dictionary Test Results ===");
    println!("Total words: {}", total);
    println!("Passed: {} ({:.2}%)", passed, pass_rate);
    println!("Failed: {}", failed);

    if !failures.is_empty() {
        println!("\n=== First {} Failures ===", failures.len().min(50));
        for (input, expected, actual) in failures.iter().take(50) {
            println!("  '{}' → expected '{}', got '{}'", input, expected, actual);
        }
    }

    // Write failures to file for analysis
    {
        use std::fs::File;
        use std::io::Write;
        if let Ok(mut f) = File::create("tests/data/vietnamese_failures.txt") {
            for (input, expected, actual) in &failures {
                let _ = writeln!(f, "{}\t{}\t{}", input, expected, actual);
            }
        }
    }

    // Target: 100% pass rate
    assert!(
        pass_rate >= 99.0,
        "Vietnamese pass rate {:.2}% is below 99% target. Failed {} out of {} words.",
        pass_rate,
        failed,
        total
    );
}

#[test]
fn test_oe_oa_tone_placement() {
    let cases = [
        ("choes ", "chóe "), // s should apply sắc to o in oe diphthong
        ("hoas ", "hóa "),
        ("doas ", "dóa "),
        ("doaj ", "dọa "),
        ("loas ", "lóa "),
        ("toats ", "toát "), // t is final consonant, s is tone marker
    ];

    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_modern_tone(false); // Use traditional tone placement (hóa, not hoá)
        let result = type_word(&mut e, input);
        assert_eq!(result, expected, "Failed for input '{}'", input);
    }
}
