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
