//! Process result types
//!
//! Defines the possible outcomes of keystroke processing.

/// Result of processing a keystroke
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessResult {
    /// Pass key through unchanged
    Pass(char),

    /// Transform: backspace N characters, output new text
    Transform {
        /// Number of backspaces to send
        backspaces: u8,
        /// New text to output
        output: String,
    },

    /// Restore to raw (English detected)
    Restore {
        /// Number of backspaces to send
        backspaces: u8,
        /// Raw text to output
        output: String,
    },

    /// Commit current word (word boundary)
    Commit,

    /// Nothing to do (absorbed key)
    None,

    /// Switch to foreign mode (English detected)
    ForeignMode,
}

impl ProcessResult {
    /// Create transform result
    pub fn transform(backspaces: u8, output: impl Into<String>) -> Self {
        Self::Transform {
            backspaces,
            output: output.into(),
        }
    }

    /// Create restore result
    pub fn restore(backspaces: u8, output: impl Into<String>) -> Self {
        Self::Restore {
            backspaces,
            output: output.into(),
        }
    }

    /// Check if result requires output
    pub fn has_output(&self) -> bool {
        matches!(self, Self::Transform { .. } | Self::Restore { .. } | Self::Pass(_))
    }

    /// Get backspace count
    pub fn backspaces(&self) -> u8 {
        match self {
            Self::Transform { backspaces, .. } => *backspaces,
            Self::Restore { backspaces, .. } => *backspaces,
            _ => 0,
        }
    }

    /// Get output text
    pub fn output(&self) -> Option<&str> {
        match self {
            Self::Transform { output, .. } => Some(output),
            Self::Restore { output, .. } => Some(output),
            Self::Pass(_) => None, // Would need allocation
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass() {
        let r = ProcessResult::Pass('a');
        assert!(r.has_output());
        assert_eq!(r.backspaces(), 0);
    }

    #[test]
    fn test_transform() {
        let r = ProcessResult::transform(1, "á");
        assert!(r.has_output());
        assert_eq!(r.backspaces(), 1);
        assert_eq!(r.output(), Some("á"));
    }

    #[test]
    fn test_restore() {
        let r = ProcessResult::restore(3, "test");
        assert!(r.has_output());
        assert_eq!(r.backspaces(), 3);
        assert_eq!(r.output(), Some("test"));
    }

    #[test]
    fn test_commit() {
        let r = ProcessResult::Commit;
        assert!(!r.has_output());
        assert_eq!(r.backspaces(), 0);
    }
}
