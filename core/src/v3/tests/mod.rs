//! V3 Test Suite
//!
//! Comprehensive tests for V3 Matrix Engine.
//! Target: 800+ tests covering all edge cases.
//!
//! ## Test Categories
//!
//! 1. Basic Vietnamese syllables (100)
//! 2. Tone combinations (100)
//! 3. Mark combinations (100)
//! 4. Multi-syllable typing (50)
//! 5. Double-key revert (50)
//! 6. English detection (100)
//! 7. Auto-restore scenarios (100)
//! 8. Edge cases (100)
//! 9. Stress tests (50)
//! 10. Regression tests (50)

mod auto_restore;
mod basic_vietnamese;
mod edge_cases;
mod english_detection;
mod paragraph_tests;
mod tone_mark_tests;

/// Test helper: simulate typing a sequence of keys
#[allow(dead_code)]
pub fn type_sequence(processor: &mut crate::v3::processor::Processor, keys: &str) -> String {
    for c in keys.chars() {
        processor.process(c, false, false);
    }
    processor.buffer_content()
}

/// Test helper: type and commit (add space)
#[allow(dead_code)]
pub fn type_and_commit(processor: &mut crate::v3::processor::Processor, keys: &str) -> String {
    let _ = type_sequence(processor, keys);
    processor.process(' ', false, false);
    let result = processor.buffer_content();
    processor.clear();
    result
}

/// Test helper macro for keystroke tests
#[macro_export]
macro_rules! test_typing {
    ($name:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let mut p = crate::v3::processor::Processor::new();
            let result = crate::v3::tests::type_sequence(&mut p, $input);
            assert_eq!(result, $expected, "Input: {}", $input);
        }
    };
}

/// Test helper macro for restore tests
#[macro_export]
macro_rules! test_restore {
    ($name:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let mut p = crate::v3::processor::Processor::new();
            let result = crate::v3::tests::type_and_commit(&mut p, $input);
            assert_eq!(result, $expected, "Input: {} (with space)", $input);
        }
    };
}
#[cfg(test)]
mod debug_tests {
    use crate::v3::processor::Processor;
    use crate::v3::tests::type_sequence;

    #[test]
    fn debug_banfj() {
        let mut p = Processor::new();

        // Type banfj step by step
        println!("\n=== Typing 'banfj' ===");

        p.process('b', false, false);
        println!(
            "After 'b': buffer='{}', raw='{}'",
            p.buffer_content(),
            p.raw_content()
        );

        p.process('a', false, false);
        println!(
            "After 'a': buffer='{}', raw='{}'",
            p.buffer_content(),
            p.raw_content()
        );

        p.process('n', false, false);
        println!(
            "After 'n': buffer='{}', raw='{}'",
            p.buffer_content(),
            p.raw_content()
        );

        p.process('f', false, false);
        println!(
            "After 'f' (huyền): buffer='{}', raw='{}'",
            p.buffer_content(),
            p.raw_content()
        );

        p.process('j', false, false);
        println!(
            "After 'j' (nặng): buffer='{}', raw='{}'",
            p.buffer_content(),
            p.raw_content()
        );

        // Expected: bạn
        assert_eq!(
            p.buffer_content(),
            "bạn",
            "Expected 'bạn' but got '{}'",
            p.buffer_content()
        );
    }
}
#[cfg(test)]
mod debug_tests2 {
    use crate::v3::constants::placement::{apply_tone, is_vowel, remove_tone, tone};
    use crate::v3::processor::Processor;

    #[test]
    fn debug_tone_application() {
        println!("\n=== Testing apply_tone function ===");

        // Test applying nặng to 'a'
        let result1 = apply_tone('a', tone::NANG);
        println!("apply_tone('a', NANG=5) = '{}'", result1);
        assert_eq!(result1, 'ạ');

        // Test applying nặng to 'à' (already has huyền)
        let result2 = apply_tone('à', tone::NANG);
        println!("apply_tone('à', NANG=5) = '{}' (should be 'ạ')", result2);

        // Test if 'à' is recognized as vowel
        println!("is_vowel('à') = {}", is_vowel('à'));

        // Test remove_tone
        println!("remove_tone('à') = '{}'", remove_tone('à'));
    }

    #[test]
    fn debug_issue() {
        let mut p = Processor::new();

        println!("\n=== Typing 'issue' ===");

        for c in "issue".chars() {
            p.process(c, false, false);
            println!(
                "After '{}': buffer='{}', raw='{}'",
                c,
                p.buffer_content(),
                p.raw_content()
            );
        }

        // Process space to commit
        p.process(' ', false, false);
        println!(
            "After ' ': buffer='{}', raw='{}'",
            p.buffer_content(),
            p.raw_content()
        );
    }
}
