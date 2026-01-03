//! Matrix-based Vietnamese IME Engine V3
//!
//! Complete rewrite with:
//! - 100% matrix-based decisions (zero if-else in hot path)
//! - Comprehensive 20K word coverage
//! - Full keystroke processing documentation
//! - 800+ test cases
//!
//! ## Architecture
//!
//! ```text
//! Input → Classify (LETTER_CLASS) → Dispatch (DISPATCH) → Execute → Validate → Output
//!          26 bytes                  40 bytes              O(1)     ~2KB
//! ```
//!
//! ## Validation Pipeline
//!
//! ```text
//! Step 1: VN(R) - Raw buffer validation
//! Step 2: VN(B) - Buffer validation (8 phonotactic rules)
//! Step 3: EN(R) - English pattern detection (7-tier)
//! Step 4: Decision - KEEP VN | RESTORE EN | KEEP AS-IS
//! ```
//!
//! ## Critical Principles
//!
//! **NGUYÊN TẮC 1: TRẢI NGHIỆM GÕ LÀ SỐ 1**
//! **NGUYÊN TẮC 2: KHÔNG FIX CASE-BY-CASE**

pub mod constants;
pub mod processor;
pub mod validation;
pub mod adapter;

#[cfg(test)]
pub mod tests;

// Re-exports
pub use constants::{
    dispatch::{Action, State},
    key_category::KeyCategory,
    letter_class::LetterClass,
};

pub use processor::{DualBuffer, Processor, ProcessResult, RawBuffer, RawKeystroke};

pub use validation::{
    english_likelihood, is_valid_syllable, should_restore, validate_vietnamese,
    EnglishConfidence, RestoreDecision,
};

// V1-compatible Engine adapter
pub use adapter::Engine;
