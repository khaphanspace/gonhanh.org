//! Core keystroke processing
//!
//! Stateful processor that transforms keystrokes into Vietnamese text.
//! Uses matrix dispatch for all decisions.
//!
//! ## Architecture (Phase 6: Hybrid Validation)
//!
//! ```text
//! Input → Classify → Dispatch → PRE-VALIDATE → Execute → POST-CHECK → Output
//!                               (skip if EN)              (restore if IMPOSSIBLE)
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
use crate::v3::validation::{should_restore, should_restore_immediate, RestoreDecision};
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
    /// Pipeline: Input → Classify → Dispatch → PRE-VALIDATE → Execute → POST-CHECK → Output
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

        // Step 4: POST-CHECK - Check if immediate restore needed (IMPOSSIBLE state)
        if self.buffer.has_transform && !self.foreign_mode {
            let buffer_str = self.buffer.transformed();
            let raw_str = self.buffer.raw_all();

            if should_restore_immediate(&buffer_str, &raw_str, true, self.buffer.has_stroke) {
                // Execute restore
                let backspaces = buffer_str.chars().count() as u8;
                self.buffer.set_transformed(&raw_str);
                self.buffer.has_transform = false;
                self.foreign_mode = true; // Enter foreign mode after restore

                return ProcessResult::Restore {
                    backspaces,
                    output: raw_str,
                };
            }
        }

        // Step 5: Handle special results
        match &result {
            ProcessResult::ForeignMode => {
                self.foreign_mode = true;
            }
            ProcessResult::Commit => {
                // Check boundary restore before clearing
                // Restore when: has_transform OR buffer differs from raw (revert case)
                let has_difference = self.buffer.has_difference();
                if self.buffer.has_transform || has_difference {
                    let buffer_str = self.buffer.transformed();
                    let raw_str = self.buffer.raw_all();
                    let had_revert = self.buffer.track.xform_type == XformType::None
                        && has_difference;

                    let decision = should_restore(
                        self.buffer.has_transform || has_difference,
                        self.buffer.has_stroke,
                        &buffer_str,
                        &raw_str,
                        had_revert,
                    );

                    if decision == RestoreDecision::RestoreEnglish {
                        let backspaces = buffer_str.chars().count() as u8;
                        // Set buffer to raw (restored) value instead of clearing
                        // This ensures buffer_content() returns the restored word
                        self.buffer.set_transformed(&raw_str);
                        self.buffer.has_transform = false;
                        self.state = State::Empty;
                        self.foreign_mode = true; // Enter foreign mode after restore
                        return ProcessResult::Restore {
                            backspaces,
                            output: raw_str,
                        };
                    }
                }

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
