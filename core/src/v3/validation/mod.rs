//! Validation pipelines for Vietnamese and English
//!
//! ## Phase 6: Hybrid Validation Approach
//!
//! Two-phase validation based on V1 production code:
//!
//! ### PHASE 1: PRE-VALIDATE (BEFORE transform)
//! - `should_skip_transform()` - check if transform should be skipped
//!
//! ### PHASE 2: POST-CHECK (AFTER keystroke / on boundary)
//! - `check_buffer_state()` - determine COMPLETE/INCOMPLETE/IMPOSSIBLE
//! - `should_restore_immediate()` - check if immediate restore needed
//! - `should_restore()` - check on word boundary
//!
//! ## Critical Principles
//!
//! 1. **NGUYÊN TẮC 1: TRẢI NGHIỆM GÕ LÀ SỐ 1**
//! 2. **NGUYÊN TẮC 2: KHÔNG FIX CASE-BY-CASE**
//!
//! All decisions MUST go through validation flow, NO hardcoding for specific cases.

pub mod english;
pub mod restore;
pub mod vietnamese;

pub use english::{english_likelihood, is_english_word, EnglishConfidence};
pub use restore::{
    check_buffer_state, should_restore, should_restore_immediate, should_skip_transform,
    BufferState, RestoreDecision, SkipDecision,
};
pub use vietnamese::{is_valid_syllable, validate_vietnamese, ValidationResult};
