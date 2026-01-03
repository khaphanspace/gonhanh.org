//! Engine V2 - Vietnamese IME (Spec v3.7)
//!
//! Implements the 7-step keystroke pipeline:
//! 0. Key Classification
//! 1. Pre-check (foreign mode)
//! 2. Dispatch & Execute
//! 3. Tone/Mark Placement
//! 4. Buffer Update
//! 5. 9-Layer Validation
//! 6. Restore Decision
//! 7. Output Generation

pub mod buffer;
pub mod state;
pub mod types;

// Re-exports
pub use buffer::Buffer;
pub use state::{BufferState, VnState};
pub use types::*;
