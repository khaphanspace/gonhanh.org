//! Core keystroke processing
//!
//! Stateful processor that transforms keystrokes into Vietnamese text.
//! Uses matrix dispatch for all decisions.
//!
//! ## Architecture
//!
//! ```text
//! Input → Classify (matrix) → Dispatch (matrix) → Execute → Validate → Output
//! ```
//!
//! ## State Machine
//!
//! 5 states: EMPTY → INIT → VOW → DIA → FIN
//! Transitions are matrix-driven, zero conditionals in hot path.

pub mod buffer;
pub mod keystroke;
pub mod result;
pub mod state;

pub use buffer::{DualBuffer, PendingPop, RawBuffer, RawKeystroke, TransformTrack, XformType};
pub use keystroke::process_keystroke;
pub use result::ProcessResult;
pub use state::{EngineState, Transition};

use crate::v3::constants::dispatch::State;
use keystroke::{execute_action_with_context, ActionContext};

/// Main V3 processor struct
pub struct Processor {
    /// Current state machine state
    state: State,
    /// Dual buffer (raw + transformed)
    buffer: DualBuffer,
    /// Input method (0=Telex)
    method: u8,
    /// Foreign mode flag (after revert)
    foreign_mode: bool,
    /// Modern tone placement (used in Phase 05)
    #[allow(dead_code)]
    modern_tone: bool,
}

impl Processor {
    /// Create new processor with default settings
    pub fn new() -> Self {
        Self {
            state: State::Empty,
            buffer: DualBuffer::new(),
            method: 0, // Telex
            foreign_mode: false,
            modern_tone: true,
        }
    }

    /// Process a single keystroke
    ///
    /// Pipeline: Input → Classify → Dispatch → Execute → Update State → Output
    pub fn process(&mut self, key: char, _caps: bool, ctrl: bool) -> ProcessResult {
        // Ctrl bypass - pass through without processing
        if ctrl {
            return ProcessResult::Pass(key);
        }

        // Step 1: Get action and next state from dispatch matrix
        let (action, next_state) = process_keystroke(self.state, key, self.foreign_mode);

        // Step 2: Execute action with buffer context
        let mut ctx = ActionContext {
            buffer: &mut self.buffer,
            key,
            method: self.method,
        };
        let result = execute_action_with_context(action, &mut ctx);

        // Step 3: Update state
        self.state = next_state;

        // Step 4: Handle special results
        match &result {
            ProcessResult::ForeignMode => {
                self.foreign_mode = true;
            }
            ProcessResult::Commit => {
                // Clear buffer and reset state after commit
                self.state = State::Empty;
                self.foreign_mode = false;
            }
            _ => {}
        }

        result
    }

    /// Clear buffer and reset state
    pub fn clear(&mut self) {
        self.state = State::Empty;
        self.buffer.clear();
        self.foreign_mode = false;
    }

    /// Set input method
    pub fn set_method(&mut self, method: u8) {
        self.method = method;
    }

    /// Get current buffer content
    pub fn buffer_content(&self) -> String {
        self.buffer.transformed()
    }

    /// Get raw buffer content (for restore)
    pub fn raw_content(&self) -> String {
        self.buffer.raw_all()
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_processor() {
        let p = Processor::new();
        assert_eq!(p.state, State::Empty);
        assert_eq!(p.method, 0);
        assert!(!p.foreign_mode);
    }

    #[test]
    fn test_clear() {
        let mut p = Processor::new();
        p.foreign_mode = true;
        p.clear();
        assert!(!p.foreign_mode);
        assert_eq!(p.state, State::Empty);
    }
}
