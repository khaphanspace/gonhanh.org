//! Step 7: Output Generation
//!
//! Calculates the diff between previous output and current output.
//! Returns (backspaces, commit) where:
//! - backspaces: number of characters to delete from previous
//! - commit: new string to output

/// Calculate diff between previous and current output
///
/// # Arguments
/// * `prev` - Previous output string
/// * `current` - Current output string
///
/// # Returns
/// * `(backspaces, commit)` - Number of backspaces and string to commit
pub fn generate_output(prev: &str, current: &str) -> (u8, String) {
    let common_len = common_prefix_length(prev, current);
    let prev_chars = prev.chars().count();
    let backspaces = (prev_chars - common_len) as u8;
    let commit: String = current.chars().skip(common_len).collect();

    (backspaces, commit)
}

/// Find length of common prefix in characters
fn common_prefix_length(a: &str, b: &str) -> usize {
    a.chars()
        .zip(b.chars())
        .take_while(|(ca, cb)| ca == cb)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_prefix() {
        assert_eq!(common_prefix_length("abc", "abd"), 2);
        assert_eq!(common_prefix_length("abc", "abc"), 3);
        assert_eq!(common_prefix_length("abc", "xyz"), 0);
        assert_eq!(common_prefix_length("", "abc"), 0);
        assert_eq!(common_prefix_length("abc", ""), 0);
    }

    #[test]
    fn test_add_letter() {
        // "ba" → "ban"
        let (bs, commit) = generate_output("ba", "ban");
        assert_eq!(bs, 0);
        assert_eq!(commit, "n");
    }

    #[test]
    fn test_add_tone() {
        // "ba" → "bá" (replace a with á)
        let (bs, commit) = generate_output("ba", "bá");
        assert_eq!(bs, 1);
        assert_eq!(commit, "á");
    }

    #[test]
    fn test_revert_to_raw() {
        // "bá" → "bas" (revert tone, add s)
        let (bs, commit) = generate_output("bá", "bas");
        assert_eq!(bs, 1); // delete á
        assert_eq!(commit, "as");
    }

    #[test]
    fn test_complex_transform() {
        // "viê" → "việt"
        let (bs, commit) = generate_output("viê", "việt");
        assert_eq!(bs, 1); // delete ê
        assert_eq!(commit, "ệt");
    }

    #[test]
    fn test_complete_replace() {
        // "abc" → "xyz"
        let (bs, commit) = generate_output("abc", "xyz");
        assert_eq!(bs, 3);
        assert_eq!(commit, "xyz");
    }

    #[test]
    fn test_empty_prev() {
        // "" → "abc"
        let (bs, commit) = generate_output("", "abc");
        assert_eq!(bs, 0);
        assert_eq!(commit, "abc");
    }

    #[test]
    fn test_empty_current() {
        // "abc" → ""
        let (bs, commit) = generate_output("abc", "");
        assert_eq!(bs, 3);
        assert_eq!(commit, "");
    }

    #[test]
    fn test_no_change() {
        // "abc" → "abc"
        let (bs, commit) = generate_output("abc", "abc");
        assert_eq!(bs, 0);
        assert_eq!(commit, "");
    }

    #[test]
    fn test_vietnamese_diacritics() {
        // "viêt" → "việt" (add tone to ê)
        let (bs, commit) = generate_output("viêt", "việt");
        assert_eq!(bs, 2); // delete ê, t
        assert_eq!(commit, "ệt");
    }

    #[test]
    fn test_stroke_d() {
        // "d" → "đ"
        let (bs, commit) = generate_output("d", "đ");
        assert_eq!(bs, 1);
        assert_eq!(commit, "đ");
    }

    #[test]
    fn test_circumflex() {
        // "a" → "â"
        let (bs, commit) = generate_output("a", "â");
        assert_eq!(bs, 1);
        assert_eq!(commit, "â");
    }
}
