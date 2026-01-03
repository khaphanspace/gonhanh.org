//! Validation pipelines for Vietnamese and English
//!
//! Implements the validation flow:
//! ```text
//! VN(R) → VN(B) → EN(R) → Decision
//! ```
//!
//! ## Critical Principles
//!
//! 1. **NGUYÊN TẮC 1: TRẢI NGHIỆM GÕ LÀ SỐ 1**
//! 2. **NGUYÊN TẮC 2: KHÔNG FIX CASE-BY-CASE**
//!
//! All decisions MUST go through validation flow, NO hardcoding for specific cases.

pub mod vietnamese;
pub mod english;
pub mod restore;

pub use vietnamese::{validate_vietnamese, is_valid_syllable};
pub use english::{english_likelihood, EnglishConfidence};
pub use restore::{should_restore, RestoreDecision};
