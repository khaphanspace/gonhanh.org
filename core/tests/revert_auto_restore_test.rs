//! Test cases for revert + auto-restore interaction
//!
//! When user types a word with double modifier keys (revert), the revert
//! consumes the original modifier key from raw_input. This means auto-restore
//! produces the post-revert result, not the full raw typing.
//!
//! Example: "tesst" = t-e-s-s-t
//! - First 's' applies sắc → "tét", raw=[t,e,s]
//! - Second 's' reverts mark → "tes", raw=[t,e,s] (first 's' popped from raw)
//! - 't' added → "test", raw=[t,e,s,t]
//! - Auto-restore produces "test" from raw_input (not "tesst")

mod common;
use common::telex_auto_restore;

// =============================================================================
// DOUBLE MODIFIER (REVERT) + AUTO-RESTORE
// =============================================================================

#[test]
fn revert_then_more_chars_keeps_post_revert_result() {
    // When user types double modifier (revert) THEN more characters,
    // the post-revert result is kept because the modifier key was consumed.
    telex_auto_restore(&[
        // Double s followed by more chars → keeps post-revert "test"
        ("tesst ", "test "),
    ]);
}

// =============================================================================
// EDGE CASES: REVERT BUT VALID VIETNAMESE
// =============================================================================

#[test]
fn revert_at_end_keeps_result() {
    // When user types double modifier at END of word (no more chars after),
    // the reverted result should be KEPT (user intentionally typed double to revert).
    // Examples: "ass" = a-s-s, the 'ss' at end means user wanted "as"
    telex_auto_restore(&[
        // Double s at end → keep reverted result
        ("ass ", "as "),
        // Double x at end → keep reverted result
        ("maxx ", "max "),
        // Double f at end → keep reverted result
        ("off ", "of "),
        ("eff ", "ef "),
        // Double r at end → keep reverted result
        ("err ", "er "),
        // Double j at end → keep reverted result
        ("ajj ", "aj "),
    ]);
}

#[test]
fn double_vowel_with_mark() {
    telex_auto_restore(&[
        // "maas" → "ma" + 'a' (circumflex) + 's' (sắc) = "mấ"
        // In Telex, double 'a' = circumflex, then 's' = sắc mark on top
        ("maas ", "mấ "),
    ]);
}
