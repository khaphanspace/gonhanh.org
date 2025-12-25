//! Vietnamese IME Engine
//!
//! Matrix-based engine with zero if-else in hot path.
//! All decisions use lookup tables for O(1) performance.
//!
//! ## Architecture
//!
//! ```text
//! Input → Classify (matrix) → Dispatch (matrix) → Execute → Done
//! ```
//!
//! - 5-State Machine: EMPTY → INIT → VOW → DIA → FIN
//! - Matrix Dispatch: State × Category → Action | NextState
//! - Deferred Actions: Handle context-dependent transformations

pub mod buffer;
pub mod matrix;
pub mod shortcut;
pub mod syllable;
pub mod transform;

mod engine;
pub use engine::{Action, Engine, Result};
