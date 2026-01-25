//! Test Vietnamese dictionary coverage.
//! Target: 100% pass rate for Vietnamese words across all input methods.

use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
use std::fs::File;
use std::io::Write;

/// Convert Vietnamese output to VNI input.
/// This converts the expected Vietnamese output back to VNI typing input.
/// For example: "hòa" → "hoa2", "được" → "duoc75"
fn vietnamese_to_vni(vn: &str) -> String {
    let mut result = String::new();
    let mut tone_marker: Option<char> = None;

    for c in vn.chars() {
        match c {
            // Đ/đ → d9
            'đ' => result.push_str("d9"),
            'Đ' => result.push_str("D9"),

            // Vowels with circumflex (â, ê, ô)
            'â' => result.push_str("a6"),
            'Â' => result.push_str("A6"),
            'ê' => result.push_str("e6"),
            'Ê' => result.push_str("E6"),
            'ô' => result.push_str("o6"),
            'Ô' => result.push_str("O6"),

            // Vowels with horn (ơ, ư)
            'ơ' => result.push_str("o7"),
            'Ơ' => result.push_str("O7"),
            'ư' => result.push_str("u7"),
            'Ư' => result.push_str("U7"),

            // Vowels with breve (ă)
            'ă' => result.push_str("a8"),
            'Ă' => result.push_str("A8"),

            // Vowels with tone marks - extract base + modifier + tone
            // Sắc (1): á, ấ, ắ, é, ế, í, ó, ố, ớ, ú, ứ, ý
            'á' => {
                result.push('a');
                tone_marker = Some('1');
            }
            'Á' => {
                result.push('A');
                tone_marker = Some('1');
            }
            'ấ' => {
                result.push_str("a6");
                tone_marker = Some('1');
            }
            'Ấ' => {
                result.push_str("A6");
                tone_marker = Some('1');
            }
            'ắ' => {
                result.push_str("a8");
                tone_marker = Some('1');
            }
            'Ắ' => {
                result.push_str("A8");
                tone_marker = Some('1');
            }
            'é' => {
                result.push('e');
                tone_marker = Some('1');
            }
            'É' => {
                result.push('E');
                tone_marker = Some('1');
            }
            'ế' => {
                result.push_str("e6");
                tone_marker = Some('1');
            }
            'Ế' => {
                result.push_str("E6");
                tone_marker = Some('1');
            }
            'í' => {
                result.push('i');
                tone_marker = Some('1');
            }
            'Í' => {
                result.push('I');
                tone_marker = Some('1');
            }
            'ó' => {
                result.push('o');
                tone_marker = Some('1');
            }
            'Ó' => {
                result.push('O');
                tone_marker = Some('1');
            }
            'ố' => {
                result.push_str("o6");
                tone_marker = Some('1');
            }
            'Ố' => {
                result.push_str("O6");
                tone_marker = Some('1');
            }
            'ớ' => {
                result.push_str("o7");
                tone_marker = Some('1');
            }
            'Ớ' => {
                result.push_str("O7");
                tone_marker = Some('1');
            }
            'ú' => {
                result.push('u');
                tone_marker = Some('1');
            }
            'Ú' => {
                result.push('U');
                tone_marker = Some('1');
            }
            'ứ' => {
                result.push_str("u7");
                tone_marker = Some('1');
            }
            'Ứ' => {
                result.push_str("U7");
                tone_marker = Some('1');
            }
            'ý' => {
                result.push('y');
                tone_marker = Some('1');
            }
            'Ý' => {
                result.push('Y');
                tone_marker = Some('1');
            }

            // Huyền (2): à, ầ, ằ, è, ề, ì, ò, ồ, ờ, ù, ừ, ỳ
            'à' => {
                result.push('a');
                tone_marker = Some('2');
            }
            'À' => {
                result.push('A');
                tone_marker = Some('2');
            }
            'ầ' => {
                result.push_str("a6");
                tone_marker = Some('2');
            }
            'Ầ' => {
                result.push_str("A6");
                tone_marker = Some('2');
            }
            'ằ' => {
                result.push_str("a8");
                tone_marker = Some('2');
            }
            'Ằ' => {
                result.push_str("A8");
                tone_marker = Some('2');
            }
            'è' => {
                result.push('e');
                tone_marker = Some('2');
            }
            'È' => {
                result.push('E');
                tone_marker = Some('2');
            }
            'ề' => {
                result.push_str("e6");
                tone_marker = Some('2');
            }
            'Ề' => {
                result.push_str("E6");
                tone_marker = Some('2');
            }
            'ì' => {
                result.push('i');
                tone_marker = Some('2');
            }
            'Ì' => {
                result.push('I');
                tone_marker = Some('2');
            }
            'ò' => {
                result.push('o');
                tone_marker = Some('2');
            }
            'Ò' => {
                result.push('O');
                tone_marker = Some('2');
            }
            'ồ' => {
                result.push_str("o6");
                tone_marker = Some('2');
            }
            'Ồ' => {
                result.push_str("O6");
                tone_marker = Some('2');
            }
            'ờ' => {
                result.push_str("o7");
                tone_marker = Some('2');
            }
            'Ờ' => {
                result.push_str("O7");
                tone_marker = Some('2');
            }
            'ù' => {
                result.push('u');
                tone_marker = Some('2');
            }
            'Ù' => {
                result.push('U');
                tone_marker = Some('2');
            }
            'ừ' => {
                result.push_str("u7");
                tone_marker = Some('2');
            }
            'Ừ' => {
                result.push_str("U7");
                tone_marker = Some('2');
            }
            'ỳ' => {
                result.push('y');
                tone_marker = Some('2');
            }
            'Ỳ' => {
                result.push('Y');
                tone_marker = Some('2');
            }

            // Hỏi (3): ả, ẩ, ẳ, ẻ, ể, ỉ, ỏ, ổ, ở, ủ, ử, ỷ
            'ả' => {
                result.push('a');
                tone_marker = Some('3');
            }
            'Ả' => {
                result.push('A');
                tone_marker = Some('3');
            }
            'ẩ' => {
                result.push_str("a6");
                tone_marker = Some('3');
            }
            'Ẩ' => {
                result.push_str("A6");
                tone_marker = Some('3');
            }
            'ẳ' => {
                result.push_str("a8");
                tone_marker = Some('3');
            }
            'Ẳ' => {
                result.push_str("A8");
                tone_marker = Some('3');
            }
            'ẻ' => {
                result.push('e');
                tone_marker = Some('3');
            }
            'Ẻ' => {
                result.push('E');
                tone_marker = Some('3');
            }
            'ể' => {
                result.push_str("e6");
                tone_marker = Some('3');
            }
            'Ể' => {
                result.push_str("E6");
                tone_marker = Some('3');
            }
            'ỉ' => {
                result.push('i');
                tone_marker = Some('3');
            }
            'Ỉ' => {
                result.push('I');
                tone_marker = Some('3');
            }
            'ỏ' => {
                result.push('o');
                tone_marker = Some('3');
            }
            'Ỏ' => {
                result.push('O');
                tone_marker = Some('3');
            }
            'ổ' => {
                result.push_str("o6");
                tone_marker = Some('3');
            }
            'Ổ' => {
                result.push_str("O6");
                tone_marker = Some('3');
            }
            'ở' => {
                result.push_str("o7");
                tone_marker = Some('3');
            }
            'Ở' => {
                result.push_str("O7");
                tone_marker = Some('3');
            }
            'ủ' => {
                result.push('u');
                tone_marker = Some('3');
            }
            'Ủ' => {
                result.push('U');
                tone_marker = Some('3');
            }
            'ử' => {
                result.push_str("u7");
                tone_marker = Some('3');
            }
            'Ử' => {
                result.push_str("U7");
                tone_marker = Some('3');
            }
            'ỷ' => {
                result.push('y');
                tone_marker = Some('3');
            }
            'Ỷ' => {
                result.push('Y');
                tone_marker = Some('3');
            }

            // Ngã (4): ã, ẫ, ẵ, ẽ, ễ, ĩ, õ, ỗ, ỡ, ũ, ữ, ỹ
            'ã' => {
                result.push('a');
                tone_marker = Some('4');
            }
            'Ã' => {
                result.push('A');
                tone_marker = Some('4');
            }
            'ẫ' => {
                result.push_str("a6");
                tone_marker = Some('4');
            }
            'Ẫ' => {
                result.push_str("A6");
                tone_marker = Some('4');
            }
            'ẵ' => {
                result.push_str("a8");
                tone_marker = Some('4');
            }
            'Ẵ' => {
                result.push_str("A8");
                tone_marker = Some('4');
            }
            'ẽ' => {
                result.push('e');
                tone_marker = Some('4');
            }
            'Ẽ' => {
                result.push('E');
                tone_marker = Some('4');
            }
            'ễ' => {
                result.push_str("e6");
                tone_marker = Some('4');
            }
            'Ễ' => {
                result.push_str("E6");
                tone_marker = Some('4');
            }
            'ĩ' => {
                result.push('i');
                tone_marker = Some('4');
            }
            'Ĩ' => {
                result.push('I');
                tone_marker = Some('4');
            }
            'õ' => {
                result.push('o');
                tone_marker = Some('4');
            }
            'Õ' => {
                result.push('O');
                tone_marker = Some('4');
            }
            'ỗ' => {
                result.push_str("o6");
                tone_marker = Some('4');
            }
            'Ỗ' => {
                result.push_str("O6");
                tone_marker = Some('4');
            }
            'ỡ' => {
                result.push_str("o7");
                tone_marker = Some('4');
            }
            'Ỡ' => {
                result.push_str("O7");
                tone_marker = Some('4');
            }
            'ũ' => {
                result.push('u');
                tone_marker = Some('4');
            }
            'Ũ' => {
                result.push('U');
                tone_marker = Some('4');
            }
            'ữ' => {
                result.push_str("u7");
                tone_marker = Some('4');
            }
            'Ữ' => {
                result.push_str("U7");
                tone_marker = Some('4');
            }
            'ỹ' => {
                result.push('y');
                tone_marker = Some('4');
            }
            'Ỹ' => {
                result.push('Y');
                tone_marker = Some('4');
            }

            // Nặng (5): ạ, ậ, ặ, ẹ, ệ, ị, ọ, ộ, ợ, ụ, ự, ỵ
            'ạ' => {
                result.push('a');
                tone_marker = Some('5');
            }
            'Ạ' => {
                result.push('A');
                tone_marker = Some('5');
            }
            'ậ' => {
                result.push_str("a6");
                tone_marker = Some('5');
            }
            'Ậ' => {
                result.push_str("A6");
                tone_marker = Some('5');
            }
            'ặ' => {
                result.push_str("a8");
                tone_marker = Some('5');
            }
            'Ặ' => {
                result.push_str("A8");
                tone_marker = Some('5');
            }
            'ẹ' => {
                result.push('e');
                tone_marker = Some('5');
            }
            'Ẹ' => {
                result.push('E');
                tone_marker = Some('5');
            }
            'ệ' => {
                result.push_str("e6");
                tone_marker = Some('5');
            }
            'Ệ' => {
                result.push_str("E6");
                tone_marker = Some('5');
            }
            'ị' => {
                result.push('i');
                tone_marker = Some('5');
            }
            'Ị' => {
                result.push('I');
                tone_marker = Some('5');
            }
            'ọ' => {
                result.push('o');
                tone_marker = Some('5');
            }
            'Ọ' => {
                result.push('O');
                tone_marker = Some('5');
            }
            'ộ' => {
                result.push_str("o6");
                tone_marker = Some('5');
            }
            'Ộ' => {
                result.push_str("O6");
                tone_marker = Some('5');
            }
            'ợ' => {
                result.push_str("o7");
                tone_marker = Some('5');
            }
            'Ợ' => {
                result.push_str("O7");
                tone_marker = Some('5');
            }
            'ụ' => {
                result.push('u');
                tone_marker = Some('5');
            }
            'Ụ' => {
                result.push('U');
                tone_marker = Some('5');
            }
            'ự' => {
                result.push_str("u7");
                tone_marker = Some('5');
            }
            'Ự' => {
                result.push_str("U7");
                tone_marker = Some('5');
            }
            'ỵ' => {
                result.push('y');
                tone_marker = Some('5');
            }
            'Ỵ' => {
                result.push('Y');
                tone_marker = Some('5');
            }

            // Regular character - just push (tone goes at END)
            _ => {
                result.push(c);
            }
        }
    }

    // Flush tone marker at END of word
    if let Some(t) = tone_marker {
        result.push(t);
    }

    result
}

/// Test result structure
struct TestResult {
    passed: usize,
    failed: usize,
    failures: Vec<(String, String, String)>,
}

impl TestResult {
    fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            failures: Vec::new(),
        }
    }

    fn pass_rate(&self) -> f64 {
        let total = self.passed + self.failed;
        if total > 0 {
            (self.passed as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    fn total(&self) -> usize {
        self.passed + self.failed
    }
}

/// Run dictionary test with given configuration
fn run_dict_test<F>(content: &str, convert_input: F, setup_engine: fn(&mut Engine)) -> TestResult
where
    F: Fn(&str) -> String,
{
    let mut result = TestResult::new();

    for line in content.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 2 {
            continue;
        }

        let raw_input = convert_input(parts[0]);
        let input = raw_input.clone() + " ";
        let expected = parts[1].to_string() + " ";

        let mut e = Engine::new();
        setup_engine(&mut e);
        let actual = type_word(&mut e, &input);

        if actual == expected {
            result.passed += 1;
        } else {
            result.failed += 1;
            result.failures.push((
                raw_input,
                expected.trim().to_string(),
                actual.trim().to_string(),
            ));
        }
    }
    result
}

/// Print test result table
fn print_result(name: &str, result: &TestResult) {
    println!("\n┌─────────────────────────────────────────┐");
    println!("│ {:^39} │", name);
    println!("├─────────────────────────────────────────┤");
    println!("│ Total words     │ {:>20} │", result.total());
    println!("│ Passed          │ {:>20} │", result.passed);
    println!("│ Failed          │ {:>20} │", result.failed);
    println!("│ Pass rate       │ {:>19.2}% │", result.pass_rate());
    println!("└─────────────────────────────────────────┘");

    if !result.failures.is_empty() {
        println!("\n=== First {} Failures ===", result.failures.len().min(30));
        for (input, expected, actual) in result.failures.iter().take(30) {
            println!("  '{}' → expected '{}', got '{}'", input, expected, actual);
        }
    }
}

/// Write failures to file
fn write_failures(filename: &str, result: &TestResult, header: &str) {
    if let Ok(mut f) = File::create(format!("tests/data/{}", filename)) {
        writeln!(f, "# {}", header).ok();
        writeln!(f, "# Format: INPUT \\t EXPECTED \\t ACTUAL").ok();
        writeln!(f, "# Total failures: {}", result.failures.len()).ok();
        writeln!(f).ok();
        for (input, expected, actual) in &result.failures {
            writeln!(f, "{}\t{}\t{}", input, expected, actual).ok();
        }
    }
}

// ============================================================
// VNI TEST
// ============================================================

/// Run VNI dictionary test - converts Vietnamese output to VNI input
fn run_vni_dict_test(content: &str) -> TestResult {
    let mut result = TestResult::new();

    for line in content.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 2 {
            continue;
        }

        // Convert Vietnamese output → VNI input
        let expected = parts[1];
        let vni_input = vietnamese_to_vni(expected);
        let input = vni_input.clone() + " ";
        let expected_with_space = expected.to_string() + " ";

        let mut e = Engine::new();
        e.set_method(1); // VNI mode
        e.set_modern_tone(false);
        let actual = type_word(&mut e, &input);

        if actual == expected_with_space {
            result.passed += 1;
        } else {
            result.failed += 1;
            result
                .failures
                .push((vni_input, expected.to_string(), actual.trim().to_string()));
        }
    }
    result
}

#[test]
fn vietnamese_dict_vni() {
    let content = include_str!("data/vietnamese_telex_pairs.txt");

    let result = run_vni_dict_test(content);

    print_result("VNI", &result);
    write_failures(
        "vietnamese_vni_failures.txt",
        &result,
        "Vietnamese VNI Failures",
    );

    const MIN_PASS_RATE: f64 = 100.0;
    assert!(
        result.pass_rate() >= MIN_PASS_RATE,
        "VNI pass rate {:.2}% is below {:.0}% target. Failed {} words.",
        result.pass_rate(),
        MIN_PASS_RATE,
        result.failed
    );
}

// ============================================================
// TELEX NORMAL TEST
// ============================================================

#[test]
fn vietnamese_dict_telex() {
    let content = include_str!("data/vietnamese_telex_pairs.txt");

    let result = run_dict_test(
        content,
        |s| s.to_string(), // No conversion
        |e| {
            e.set_modern_tone(false);
        },
    );

    print_result("TELEX", &result);
    write_failures(
        "vietnamese_telex_failures.txt",
        &result,
        "Vietnamese Telex Failures",
    );

    const MIN_PASS_RATE: f64 = 100.0;
    assert!(
        result.pass_rate() >= MIN_PASS_RATE,
        "Telex pass rate {:.2}% is below {:.0}% target. Failed {} words.",
        result.pass_rate(),
        MIN_PASS_RATE,
        result.failed
    );
}

// ============================================================
// TELEX AUTO-RESTORE TEST
// ============================================================

#[test]
fn vietnamese_dict_telex_auto_restore() {
    let content = include_str!("data/vietnamese_telex_pairs.txt");

    let result = run_dict_test(
        content,
        |s| s.to_string(), // No conversion
        |e| {
            e.set_modern_tone(false);
            e.set_english_auto_restore(true);
        },
    );

    print_result("TELEX + AUTO-RESTORE", &result);
    write_failures(
        "vietnamese_telex_auto_restore_failures.txt",
        &result,
        "Vietnamese Telex Auto-Restore Failures (should not be restored)",
    );

    const MIN_PASS_RATE: f64 = 100.0;
    assert!(
        result.pass_rate() >= MIN_PASS_RATE,
        "Telex+AutoRestore pass rate {:.2}% is below {:.0}% target. {} words incorrectly restored.",
        result.pass_rate(),
        MIN_PASS_RATE,
        result.failed
    );
}

// ============================================================
// TYPING VARIANTS TEST (22K)
// ============================================================

/// Detect tone style from Vietnamese word
/// Returns true if word uses modern style (tone on main vowel in oa/oe/uy)
fn detect_tone_style_is_modern(word: &str) -> bool {
    // Check for oa/oe/uy patterns with tone marks
    // Modern: dấu trên nguyên âm thứ 2 (hoà, khoẻ, thuỷ) - 'à', 'ẻ', 'ỷ'
    // Traditional: dấu trên nguyên âm đầu (hòa, khỏe, thủy) - 'ò', 'ỏ', 'ủ'
    let chars: Vec<char> = word.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        // Check for 'o' followed by toned 'a' or 'e' (modern)
        if *c == 'o' || *c == 'O' {
            if let Some(next) = chars.get(i + 1) {
                // Modern: tone on 'a' or 'e' after 'o'
                if matches!(
                    next,
                    'à' | 'á'
                        | 'ả'
                        | 'ã'
                        | 'ạ'
                        | 'è'
                        | 'é'
                        | 'ẻ'
                        | 'ẽ'
                        | 'ẹ'
                        | 'À'
                        | 'Á'
                        | 'Ả'
                        | 'Ã'
                        | 'Ạ'
                        | 'È'
                        | 'É'
                        | 'Ẻ'
                        | 'Ẽ'
                        | 'Ẹ'
                ) {
                    return true;
                }
            }
        }
        // Check for toned 'o' followed by 'a' or 'e' (traditional)
        if matches!(c, 'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ') {
            if let Some(next) = chars.get(i + 1) {
                if matches!(next, 'a' | 'e' | 'A' | 'E') {
                    return false; // Traditional
                }
            }
        }
        // Check for 'u' followed by toned 'y' (modern uy pattern)
        if *c == 'u' || *c == 'U' {
            if let Some(next) = chars.get(i + 1) {
                if matches!(
                    next,
                    'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' | 'Ỳ' | 'Ý' | 'Ỷ' | 'Ỹ' | 'Ỵ'
                ) {
                    return true;
                }
            }
        }
        // Check for toned 'u' followed by 'y' (traditional)
        if matches!(c, 'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'Ù' | 'Ú' | 'Ủ' | 'Ũ' | 'Ụ') {
            if let Some(next) = chars.get(i + 1) {
                if matches!(next, 'y' | 'Y') {
                    return false; // Traditional
                }
            }
        }
    }
    // Default to modern (most common in modern Vietnamese)
    true
}

/// Test Vietnamese typing variants from vietnamese_22k_typing_variants.txt
/// Format: word TAB variant1,variant2,...
/// Tests BOTH modern and traditional tone modes - passes if either produces correct output
#[test]
fn vietnamese_dict_typing_variants() {
    let content = include_str!("data/vietnamese_22k_typing_variants.txt");

    let mut total_words = 0;
    let mut total_variants = 0;
    let mut passed_variants = 0;
    let mut failed_variants = 0;
    let mut failures: Vec<(String, String, String, String)> = Vec::new(); // (word, variant, expected, actual)

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 2 {
            continue;
        }

        let expected_word = parts[0];
        let variants: Vec<&str> = parts[1].split(',').collect();
        total_words += 1;

        // Detect tone style from expected word
        let is_modern = detect_tone_style_is_modern(expected_word);

        for variant in &variants {
            total_variants += 1;
            let input = format!("{} ", variant);
            let expected = format!("{} ", expected_word);

            // Test with detected style
            let mut e = Engine::new();
            e.set_method(0); // Telex
            e.set_modern_tone(is_modern);
            let actual = type_word(&mut e, &input);

            // If first attempt fails, try opposite style
            let (passed, final_actual) = if actual == expected {
                (true, actual)
            } else {
                let mut e2 = Engine::new();
                e2.set_method(0);
                e2.set_modern_tone(!is_modern);
                let actual2 = type_word(&mut e2, &input);
                if actual2 == expected {
                    (true, actual2)
                } else {
                    (false, actual)
                }
            };

            if passed {
                passed_variants += 1;
            } else {
                failed_variants += 1;
                if failures.len() < 1000 {
                    failures.push((
                        expected_word.to_string(),
                        variant.to_string(),
                        expected_word.to_string(),
                        final_actual.trim().to_string(),
                    ));
                }
            }
        }
    }

    // Print summary
    let pass_rate = if total_variants > 0 {
        (passed_variants as f64 / total_variants as f64) * 100.0
    } else {
        0.0
    };

    println!("\n┌─────────────────────────────────────────┐");
    println!("│ {:^39} │", "TYPING VARIANTS TEST");
    println!("├─────────────────────────────────────────┤");
    println!("│ Total words     │ {:>20} │", total_words);
    println!("│ Total variants  │ {:>20} │", total_variants);
    println!("│ Passed          │ {:>20} │", passed_variants);
    println!("│ Failed          │ {:>20} │", failed_variants);
    println!("│ Pass rate       │ {:>19.2}% │", pass_rate);
    println!("└─────────────────────────────────────────┘");

    // Write failures to file
    if let Ok(mut f) = File::create("tests/data/vietnamese_22k_failures.txt") {
        writeln!(f, "# Vietnamese 22k Typing Variants Failures").ok();
        writeln!(f, "# Format: WORD \\t VARIANT \\t EXPECTED \\t ACTUAL").ok();
        writeln!(f, "# Total failures: {}", failures.len()).ok();
        writeln!(f).ok();
        for (word, variant, expected, actual) in &failures {
            writeln!(f, "{}\t{}\t{}\t{}", word, variant, expected, actual).ok();
        }
        println!("\nFailures written to: tests/data/vietnamese_22k_failures.txt");
    }

    // Print sample failures
    if !failures.is_empty() {
        println!("\n=== First {} Failures ===", failures.len().min(30));
        for (word, variant, expected, actual) in failures.iter().take(30) {
            println!(
                "  '{}' typed as '{}' → expected '{}', got '{}'",
                word, variant, expected, actual
            );
        }
    }
}

// ============================================================
// UNIT TESTS
// ============================================================

#[test]
fn test_vietnamese_to_vni_conversion() {
    // Plain words (no transformation)
    assert_eq!(vietnamese_to_vni("xin"), "xin");
    assert_eq!(vietnamese_to_vni("abaddon"), "abaddon");

    // Tone marks only (tone at END of syllable)
    assert_eq!(vietnamese_to_vni("hòa"), "hoa2");
    assert_eq!(vietnamese_to_vni("chào"), "chao2");

    // Modifiers only
    assert_eq!(vietnamese_to_vni("đi"), "d9i");
    assert_eq!(vietnamese_to_vni("ăn"), "a8n");
    assert_eq!(vietnamese_to_vni("ơi"), "o7i");
    assert_eq!(vietnamese_to_vni("ưa"), "u7a");
    assert_eq!(vietnamese_to_vni("âm"), "a6m");

    // Combined: modifier + tone (tone at END of syllable)
    assert_eq!(vietnamese_to_vni("được"), "d9u7o7c5");
    assert_eq!(vietnamese_to_vni("việt"), "vie6t5");
    assert_eq!(vietnamese_to_vni("ấn"), "a6n1");
}

#[test]
fn test_oe_oa_tone_placement() {
    let cases = [
        ("choes ", "chóe "),
        ("hoas ", "hóa "),
        ("doas ", "dóa "),
        ("doaj ", "dọa "),
        ("loas ", "lóa "),
        ("toats ", "toát "),
    ];

    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_modern_tone(false);
        let result = type_word(&mut e, input);
        assert_eq!(result, expected, "Failed for input '{}'", input);
    }
}
