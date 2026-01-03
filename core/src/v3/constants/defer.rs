//! Defer resolution for context-dependent decisions
//!
//! Some keystrokes require context to determine their meaning:
//! - 'a' after 'a' could be circumflex (â) or new syllable
//! - 'd' could be stroke (đ) or regular consonant
//! - Letter after vowel could be final or new word

/// Defer types requiring context resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeferType {
    /// No deferral needed
    None = 0,
    /// Possible circumflex (aa, ee, oo)
    Circumflex = 1,
    /// Possible stroke (dd)
    Stroke = 2,
    /// Possible final consonant
    Final = 3,
    /// Possible consonant cluster (ng, nh, ch, tr, etc.)
    Cluster = 4,
    /// Possible tone change or revert
    ToneChange = 5,
}

/// Defer resolution table (8 bytes)
/// Maps DeferType to resolution strategy
pub const DEFER: [u8; 8] = [
    0, // None
    1, // Circumflex: check if same vowel
    2, // Stroke: check if 'd' follows 'd'
    3, // Final: check if valid final consonant
    4, // Cluster: check if valid cluster
    5, // ToneChange: check current tone
    0, // Reserved
    0, // Reserved
];

/// Resolution result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeferResolution {
    /// Apply the deferred action
    Apply,
    /// Pass through as regular letter
    Pass,
    /// Start new syllable
    NewSyllable,
    /// Revert previous transformation
    Revert,
}

/// Resolve circumflex defer (aa -> â, ee -> ê, oo -> ô)
#[inline]
pub fn resolve_circumflex(last_char: char, new_char: char) -> DeferResolution {
    let last = last_char.to_ascii_lowercase();
    let new = new_char.to_ascii_lowercase();

    if last == new && matches!(last, 'a' | 'e' | 'o') {
        DeferResolution::Apply
    } else {
        DeferResolution::Pass
    }
}

/// Resolve stroke defer (dd -> đ)
#[inline]
pub fn resolve_stroke(last_char: char, new_char: char) -> DeferResolution {
    let last = last_char.to_ascii_lowercase();
    let new = new_char.to_ascii_lowercase();

    if last == 'd' && new == 'd' {
        DeferResolution::Apply
    } else {
        DeferResolution::Pass
    }
}

/// Check if character sequence forms valid final cluster
#[inline]
pub fn is_valid_final_cluster(c1: char, c2: char) -> bool {
    let s = [c1.to_ascii_lowercase(), c2.to_ascii_lowercase()];
    matches!(s, ['n', 'g'] | ['n', 'h'] | ['c', 'h'])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circumflex_resolution() {
        assert_eq!(resolve_circumflex('a', 'a'), DeferResolution::Apply);
        assert_eq!(resolve_circumflex('e', 'e'), DeferResolution::Apply);
        assert_eq!(resolve_circumflex('o', 'o'), DeferResolution::Apply);
        assert_eq!(resolve_circumflex('a', 'e'), DeferResolution::Pass);
    }

    #[test]
    fn test_stroke_resolution() {
        assert_eq!(resolve_stroke('d', 'd'), DeferResolution::Apply);
        assert_eq!(resolve_stroke('d', 'a'), DeferResolution::Pass);
    }

    #[test]
    fn test_final_clusters() {
        assert!(is_valid_final_cluster('n', 'g'));
        assert!(is_valid_final_cluster('n', 'h'));
        assert!(is_valid_final_cluster('c', 'h'));
        assert!(!is_valid_final_cluster('n', 't'));
    }
}
