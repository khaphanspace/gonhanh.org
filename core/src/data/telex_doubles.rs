//! English words containing Telex double patterns that should auto-restore.
//! Data lives in telex_doubles.txt (packed by build.rs, see lexicon.rs).

use super::lexicon::DOUBLES;

/// Check if word contains Telex patterns that should auto-restore
pub fn contains(word: &str) -> bool {
    DOUBLES.contains(word)
}
