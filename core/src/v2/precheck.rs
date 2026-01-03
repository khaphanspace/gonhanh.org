//! Step 1: Pre-check for Foreign Mode
//!
//! Early detection of English/foreign words to skip VN processing.
//! Only checks first 2-3 characters for maximum efficiency.
//!
//! Detection tiers:
//! - Tier 1: Invalid VN initials (f, j, w, z)
//! - Tier 2: English-only onset clusters (bl, cl, st, etc.)

/// Processing mode
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Mode {
    #[default]
    Vietnamese,
    Foreign,
}

/// English-only onset clusters (Tier 2)
/// These NEVER appear in Vietnamese word beginnings.
/// Based on spec Section 4: 21 clusters
const EN_ONSET_CLUSTERS: &[[u8; 2]] = &[
    *b"bl", *b"br", *b"cl", *b"cr", *b"dr", *b"fl", *b"fr", *b"gl", *b"gr", *b"pl", *b"pr",
    *b"sc", *b"sk", *b"sl", *b"sm", *b"sn", *b"sp", *b"st", *b"sw", *b"tw", *b"wr",
];

/// Pre-check for foreign mode (runs on first 2-3 chars only)
///
/// Returns `Mode::Foreign` if:
/// - Tier 1: First char is f, j, w, or z
/// - Tier 2: First two chars form English-only onset cluster
///
/// Returns `Mode::Vietnamese` otherwise (default).
#[inline]
pub fn pre_check(raw: &str) -> Mode {
    // Only check early characters for efficiency
    if raw.len() > 3 {
        return Mode::Vietnamese;
    }

    let bytes = raw.as_bytes();

    // Tier 1: Invalid VN initials (f, j, w, z)
    if let Some(&first) = bytes.first() {
        let lower = first.to_ascii_lowercase();
        if matches!(lower, b'f' | b'j' | b'w' | b'z') {
            return Mode::Foreign;
        }
    }

    // Tier 2: English-only onset clusters
    if bytes.len() >= 2 {
        let pair = [
            bytes[0].to_ascii_lowercase(),
            bytes[1].to_ascii_lowercase(),
        ];
        if is_en_onset_cluster(&pair) {
            return Mode::Foreign;
        }
    }

    Mode::Vietnamese
}

/// Check if byte pair is English-only onset cluster
#[inline]
fn is_en_onset_cluster(pair: &[u8; 2]) -> bool {
    EN_ONSET_CLUSTERS.iter().any(|p| p == pair)
}

/// Check if first char is shortcut prefix (@, #, :, /)
#[inline]
pub fn is_shortcut_prefix(c: char) -> bool {
    matches!(c, '@' | '#' | ':' | '/')
}

/// Check if buffer starts with digit (149k, 2024)
#[inline]
pub fn has_digit_prefix(raw: &str) -> bool {
    raw.chars().next().is_some_and(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Tier 1: Invalid VN Initials =====

    #[test]
    fn test_tier1_invalid_initials() {
        // f, j, w, z are not valid VN initials
        assert_eq!(pre_check("f"), Mode::Foreign);
        assert_eq!(pre_check("j"), Mode::Foreign);
        assert_eq!(pre_check("w"), Mode::Foreign);
        assert_eq!(pre_check("z"), Mode::Foreign);

        // Uppercase should also work
        assert_eq!(pre_check("F"), Mode::Foreign);
        assert_eq!(pre_check("J"), Mode::Foreign);
        assert_eq!(pre_check("W"), Mode::Foreign);
        assert_eq!(pre_check("Z"), Mode::Foreign);
    }

    #[test]
    fn test_tier1_with_suffix() {
        // Foreign initial + more chars
        assert_eq!(pre_check("fo"), Mode::Foreign);
        assert_eq!(pre_check("jav"), Mode::Foreign);
        assert_eq!(pre_check("web"), Mode::Foreign);
        assert_eq!(pre_check("zip"), Mode::Foreign);
    }

    // ===== Tier 2: English-Only Onset Clusters =====

    #[test]
    fn test_tier2_onset_clusters() {
        // Common English clusters
        assert_eq!(pre_check("bl"), Mode::Foreign); // block
        assert_eq!(pre_check("br"), Mode::Foreign); // break
        assert_eq!(pre_check("cl"), Mode::Foreign); // class
        assert_eq!(pre_check("cr"), Mode::Foreign); // create
        assert_eq!(pre_check("dr"), Mode::Foreign); // draw
        assert_eq!(pre_check("fl"), Mode::Foreign); // flow
        assert_eq!(pre_check("fr"), Mode::Foreign); // from
        assert_eq!(pre_check("gl"), Mode::Foreign); // global
        assert_eq!(pre_check("gr"), Mode::Foreign); // great
        assert_eq!(pre_check("pl"), Mode::Foreign); // play
        assert_eq!(pre_check("pr"), Mode::Foreign); // program
        assert_eq!(pre_check("sc"), Mode::Foreign); // scope
        assert_eq!(pre_check("sk"), Mode::Foreign); // skip
        assert_eq!(pre_check("sl"), Mode::Foreign); // sleep
        assert_eq!(pre_check("sm"), Mode::Foreign); // small
        assert_eq!(pre_check("sn"), Mode::Foreign); // snake
        assert_eq!(pre_check("sp"), Mode::Foreign); // space
        assert_eq!(pre_check("st"), Mode::Foreign); // string
        assert_eq!(pre_check("sw"), Mode::Foreign); // switch
        assert_eq!(pre_check("tw"), Mode::Foreign); // two
        assert_eq!(pre_check("wr"), Mode::Foreign); // write
    }

    #[test]
    fn test_tier2_uppercase() {
        assert_eq!(pre_check("BL"), Mode::Foreign);
        assert_eq!(pre_check("St"), Mode::Foreign);
        assert_eq!(pre_check("Pr"), Mode::Foreign);
    }

    // ===== Valid Vietnamese =====

    #[test]
    fn test_valid_vietnamese_initials() {
        // Single valid VN initials
        assert_eq!(pre_check("a"), Mode::Vietnamese);
        assert_eq!(pre_check("b"), Mode::Vietnamese);
        assert_eq!(pre_check("c"), Mode::Vietnamese);
        assert_eq!(pre_check("d"), Mode::Vietnamese);
        assert_eq!(pre_check("e"), Mode::Vietnamese);
        assert_eq!(pre_check("g"), Mode::Vietnamese);
        assert_eq!(pre_check("h"), Mode::Vietnamese);
        assert_eq!(pre_check("i"), Mode::Vietnamese);
        assert_eq!(pre_check("k"), Mode::Vietnamese);
        assert_eq!(pre_check("l"), Mode::Vietnamese);
        assert_eq!(pre_check("m"), Mode::Vietnamese);
        assert_eq!(pre_check("n"), Mode::Vietnamese);
        assert_eq!(pre_check("o"), Mode::Vietnamese);
        assert_eq!(pre_check("p"), Mode::Vietnamese);
        assert_eq!(pre_check("q"), Mode::Vietnamese);
        assert_eq!(pre_check("r"), Mode::Vietnamese);
        assert_eq!(pre_check("s"), Mode::Vietnamese);
        assert_eq!(pre_check("t"), Mode::Vietnamese);
        assert_eq!(pre_check("u"), Mode::Vietnamese);
        assert_eq!(pre_check("v"), Mode::Vietnamese);
        assert_eq!(pre_check("x"), Mode::Vietnamese);
        assert_eq!(pre_check("y"), Mode::Vietnamese);
    }

    #[test]
    fn test_valid_vietnamese_clusters() {
        // Vietnamese consonant clusters
        assert_eq!(pre_check("ch"), Mode::Vietnamese);
        assert_eq!(pre_check("gh"), Mode::Vietnamese);
        assert_eq!(pre_check("gi"), Mode::Vietnamese);
        assert_eq!(pre_check("kh"), Mode::Vietnamese);
        assert_eq!(pre_check("ng"), Mode::Vietnamese);
        assert_eq!(pre_check("nh"), Mode::Vietnamese);
        assert_eq!(pre_check("ph"), Mode::Vietnamese);
        assert_eq!(pre_check("qu"), Mode::Vietnamese);
        assert_eq!(pre_check("th"), Mode::Vietnamese);
        assert_eq!(pre_check("tr"), Mode::Vietnamese);
    }

    #[test]
    fn test_longer_strings_default_vietnamese() {
        // After 3 chars, default to Vietnamese mode
        assert_eq!(pre_check("abcd"), Mode::Vietnamese);
        assert_eq!(pre_check("string"), Mode::Vietnamese); // even though "st" is foreign cluster
    }

    // ===== Shortcut Prefixes =====

    #[test]
    fn test_shortcut_prefixes() {
        assert!(is_shortcut_prefix('@'));
        assert!(is_shortcut_prefix('#'));
        assert!(is_shortcut_prefix(':'));
        assert!(is_shortcut_prefix('/'));

        // Not shortcut prefixes
        assert!(!is_shortcut_prefix('a'));
        assert!(!is_shortcut_prefix('1'));
        assert!(!is_shortcut_prefix('!'));
    }

    // ===== Digit Prefixes =====

    #[test]
    fn test_digit_prefix() {
        assert!(has_digit_prefix("149k"));
        assert!(has_digit_prefix("2024"));
        assert!(has_digit_prefix("0"));

        assert!(!has_digit_prefix("abc"));
        assert!(!has_digit_prefix(""));
        assert!(!has_digit_prefix("a123"));
    }

    // ===== Edge Cases =====

    #[test]
    fn test_empty_string() {
        assert_eq!(pre_check(""), Mode::Vietnamese);
    }

    #[test]
    fn test_mode_default() {
        assert_eq!(Mode::default(), Mode::Vietnamese);
    }
}
