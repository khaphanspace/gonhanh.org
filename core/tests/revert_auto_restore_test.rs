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
fn revert_at_end_short_words() {
    // Short words (3 chars raw) with double modifiers
    // EXCEPTIONS: "off", "iff", "ass" keep reverted form
    // Other -ss/-ff words restore to English
    telex_auto_restore(&[
        // Double ss: "ass" is exception, keeps reverted "as"
        ("ass ", "as "), // EXCEPTION: a-s-s → as
        // Double ff: exceptions keep reverted, others restore
        ("off ", "of "),  // EXCEPTION: o-f-f → of (common English word)
        ("iff ", "if "),  // EXCEPTION: i-f-f → if (common English word)
        ("eff ", "eff "), // e-f-f → eff (restore to English)
        ("aff ", "aff "), // a-f-f → aff (restore to English)
        // Other modifiers (rr, xx, jj) keep reverted form
        ("err ", "er "), // e-r-r → er
        ("ajj ", "aj "), // a-j-j → aj
        ("axx ", "ax "), // a-x-x → ax
    ]);
}

#[test]
fn revert_at_end_restores_or_keeps_4char() {
    // 4-char raw with double modifiers:
    // - 'ss' and 'ff' → restore to English (s/f not valid VN finals)
    // - 'rr', 'xx', 'jj' → keep reverted form
    telex_auto_restore(&[
        // Double ss: restore to English (s not valid VN final)
        ("SOSS ", "SOSS "), // S-O-S-S → SOSS
        ("BOSS ", "BOSS "), // B-O-S-S → BOSS
        ("LOSS ", "LOSS "), // L-O-S-S → LOSS
        ("MOSS ", "MOSS "), // M-O-S-S → MOSS
        ("boss ", "boss "), // lowercase also works
        // Double ff: restore to English (f not valid VN final)
        ("buff ", "buff "), // b-u-f-f → buff
        ("cuff ", "cuff "), // c-u-f-f → cuff
        ("puff ", "puff "), // p-u-f-f → puff
        // Double r: keep reverted (programming keywords)
        ("varr ", "var "), // v-a-r-r → var (JS keyword)
        ("VARR ", "VAR "), // V-A-R-R → VAR
        ("norr ", "nor "), // n-o-r-r → nor
        // Double x: keep reverted
        ("boxx ", "box "), // b-o-x-x → box
        // Double j: keep reverted
        ("hajj ", "haj "), // h-a-j-j → haj
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
        // carre: double r in middle followed by vowel → restore to "care"
        ("carre ", "care "),
    ]);
}

// =============================================================================
// DOUBLE D (Đ) + AUTO-RESTORE
// Tests for dd → đ conversion and validation of resulting syllables
// =============================================================================

/// Test basic mark apply and revert (without auto-restore)
#[test]
fn basic_mark_apply_revert() {
    telex(&[
        // 'r' adds hỏi to preceding vowel
        ("car", "cả"),     // c-a-r → cả (r adds hỏi to a)
        ("carr", "car"),   // c-a-r-r → car (second r reverts, output 'r')
        ("carre", "care"), // c-a-r-r-e → car + e = care (buffer after revert)
    ]);
}

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
