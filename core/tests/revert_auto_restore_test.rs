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
use common::{telex, telex_auto_restore};

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
fn revert_at_end_restores_invalid_finals() {
    // Words ending with invalid Vietnamese finals ('s', 'f', 'r') → restore to English
    // Only 'x' and 'j' keep reverted (uncommon doubles in English)
    telex_auto_restore(&[
        // 3-char raw with double s/f/r at end → restore (invalid VN final)
        ("ass ", "ass "), // a-s-s → restore (s is invalid final)
        ("off ", "off "), // o-f-f → restore (f is invalid final)
        ("eff ", "eff "), // e-f-f → restore (f is invalid final)
        ("err ", "err "), // e-r-r → restore (r is invalid final)
        // 3-char raw with double x/j at end → keep reverted (uncommon in English)
        ("ajj ", "aj "), // a-j-j → keep (j is uncommon double)
        ("axx ", "ax "), // a-x-x → keep (x is uncommon double)
    ]);
}

#[test]
fn revert_at_end_4char_restores_or_keeps() {
    // 4-char raw with double modifier at end:
    // - s/f/r doubles → restore to English (common English doubles, invalid VN final)
    // - x/j doubles → keep reverted (uncommon English doubles)
    telex_auto_restore(&[
        // Double s: restore (s is invalid VN final, common in English)
        ("SOSS ", "SOSS "), // S-O-S-S → restore
        ("BOSS ", "BOSS "), // B-O-S-S → restore
        ("LOSS ", "LOSS "), // L-O-S-S → restore
        ("MOSS ", "MOSS "), // M-O-S-S → restore
        ("boss ", "boss "), // lowercase also restores
        // Double r: restore (r is invalid VN final)
        ("varr ", "varr "), // v-a-r-r → restore
        ("VARR ", "VARR "), // V-A-R-R → restore
        ("norr ", "norr "), // n-o-r-r → restore
        // Double f: restore (f is invalid VN final)
        ("buff ", "buff "), // b-u-f-f → restore
        ("cuff ", "cuff "), // c-u-f-f → restore
        ("puff ", "puff "), // p-u-f-f → restore
        // Double x: keep reverted (uncommon in English)
        ("boxx ", "box "), // b-o-x-x → keep
        // Double j: keep reverted (uncommon in English)
        ("hajj ", "haj "), // h-a-j-j → keep
    ]);
}

#[test]
fn invalid_initial_no_transform() {
    // Words starting with invalid Vietnamese initials (f, j, w, z) don't get marks applied
    // So typing double modifier just adds the character, no revert happens
    telex_auto_restore(&[
        // f is not a valid Vietnamese initial, so 'r' mark is not applied
        ("for ", "for "),   // No transform, stays as-is
        ("forr ", "forr "), // No transform, second 'r' just added
        ("foxx ", "foxx "), // No transform, second 'x' just added
    ]);
}

#[test]
fn revert_at_end_restores_long_english_words() {
    // 5+ char raw words with common double letters → restore to English
    // These are real English words that should be preserved
    telex_auto_restore(&[
        // Double s: common English words (5+ chars)
        ("class ", "class "),
        ("grass ", "grass "),
        ("glass ", "glass "),
        ("press ", "press "),
        ("dress ", "dress "),
        ("cross ", "cross "),
        ("gross ", "gross "),
        ("stress ", "stress "),
        // Double f: common English words (5+ chars)
        ("staff ", "staff "),
        ("stuff ", "stuff "),
        ("cliff ", "cliff "),
        ("stiff ", "stiff "),
        // Double r: common English words (5+ chars)
        ("error ", "error "),
        ("mirror ", "mirror "),
        ("horror ", "horror "),
        ("terror ", "terror "),
        // Double w: programming keywords
        ("await ", "await "),  // normal typing, no double w
        ("awwait ", "await "), // double w reverts horn, restore to English
        // Double s in middle: usser → user (ss reverts sắc, buffer has "user")
        ("usser ", "user "), // u-s-s-e-r → buffer "user", restore to buffer
                             // Note: "user" without double s also works (tested in english_auto_restore_test.rs)
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

// =============================================================================
// DOUBLE D (Đ) + AUTO-RESTORE
// Tests for dd → đ conversion and validation of resulting syllables
// =============================================================================

#[test]
fn double_s_middle_pattern() {
    // Pattern: V-ss-V-C → buffer uses reverted result
    // "usser" typed as u-s-s-e-r:
    // - 's' applies sắc → "ú"
    // - second 's' reverts → "us"
    // - 'e' + 'r' → "user"
    // Buffer is "user", raw_input is [u,s,s,e,r] (5 chars)
    // Since double 's' in middle + consonant end → use buffer
    telex_auto_restore(&[
        ("usser ", "user "),
        // Note: "issue" has different pattern (i-ss-u-e ends with vowel)
        // so it uses raw_input → "issue"
        ("issue ", "issue "),
    ]);
}

#[test]
fn consecutive_modifiers_followed_by_vowel() {
    // Pattern: consecutive tone modifiers (r+s, s+r, etc.) followed by vowel → English
    // Vietnamese doesn't have this pattern; it's characteristic of English words
    telex_auto_restore(&[
        // cursor: c-u-r-s-o-r → "rs" + vowel 'o' → English
        ("cursor ", "cursor "),
        // version: v-e-r-s-i-o-n → "rs" + vowel 'i' → English
        ("version ", "version "),
        // person: p-e-r-s-o-n → "rs" + vowel 'o' → English
        ("person ", "person "),
        // jersey: j-e-r-s-e-y → "rs" + vowel 'e' → English
        ("jersey ", "jersey "),
        // versus: v-e-r-s-u-s → "rs" + vowel 'u' → English
        ("versus ", "versus "),
        // parser: p-a-r-s-e-r → "rs" + vowel 'e' → English
        ("parser ", "parser "),
        // nursery: n-u-r-s-e-r-y → "rs" + vowel 'e' → English
        ("nursery ", "nursery "),
        // cusor (typo): no consecutive modifiers + vowel pattern → stays Vietnamese
        ("cusor ", "cuỏ "),
    ]);
}

// =============================================================================
// DOUBLE D (Đ) + AUTO-RESTORE
// Tests for dd → đ conversion and validation of resulting syllables
// =============================================================================

/// Test delayed stroke without auto-restore
#[test]
fn delayed_stroke_basic() {
    // Without auto-restore, delayed stroke should work
    telex(&[
        // Adjacent dd at start
        ("ddau ", "đau "),
        // ddinrh → đỉnh - adjacent dd
        ("ddinrh ", "đỉnh "),
    ]);
}

#[test]
fn double_d_valid_vietnamese() {
    // In Telex, second 'd' triggers stroke on first 'd' (delayed stroke)
    // This creates đ which combines with the vowels to form valid Vietnamese
    telex_auto_restore(&[
        // ddau → đau (pain) - adjacent dd produces đ
        ("ddau ", "đau "),
        // ddinrh → đỉnh (peak) - adjacent dd→đ, i vowel, nh final, r=hỏi mark
        ("ddinrh ", "đỉnh "),
    ]);
}

#[test]
fn delayed_stroke_with_vowel_between() {
    // Delayed stroke pattern: d + vowel + d → đ + vowel
    // The second 'd' triggers stroke on first 'd' even with vowel in between
    telex_auto_restore(&[
        // dadu → đau (pain) - delayed stroke with vowel between
        ("dadu ", "đau "),
        // didnrh → đỉnh (peak) - delayed stroke with vowel between
        ("didnrh ", "đỉnh "),
    ]);
}
