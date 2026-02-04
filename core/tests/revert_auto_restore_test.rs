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
fn revert_then_more_chars_keeps_buffer() {
    // When user types double modifier (revert) THEN more characters,
    // if buffer is in whitelist, keep buffer instead of raw.
    // Example: "tesst" → buffer "test" is in whitelist → keep "test"
    telex_auto_restore(&[
        // Double s followed by more chars → buffer "test" in whitelist → keep "test"
        ("tesst ", "test "),
    ]);
}

// =============================================================================
// EDGE CASES: REVERT BUT VALID VIETNAMESE
// =============================================================================

#[test]
fn revert_at_end_short_words() {
    // Short words (3 chars raw) with double modifiers
    // Only restore if raw is in dict, otherwise keep buffer
    telex_auto_restore(&[
        // Double ss/ff: keep buffer (not in dict)
        ("ass ", "as "), // a-s-s → as (not in dict)
        ("off ", "of "), // o-f-f → of (not in dict)
        ("iff ", "if "), // i-f-f → if (not in dict)
        ("eff ", "ef "), // e-f-f → ef (not in dict)
        ("aff ", "af "), // a-f-f → af (not in dict)
        // Other modifiers (rr, xx, jj) keep reverted form
        ("err ", "er "), // e-r-r → er
        ("ajj ", "aj "), // a-j-j → aj
        ("axx ", "ax "), // a-x-x → ax
    ]);
}

#[test]
fn revert_at_end_keeps_buffer_4char() {
    // 4-char raw with double modifiers:
    // - Words IN whitelist → restore to raw (boss, buff, cuff, loss, moss, puff)
    // - Words NOT in whitelist → keep buffer (soss → sos, varr → var, etc.)
    telex_auto_restore(&[
        // IN whitelist → restore to raw
        ("BOSS ", "BOSS "),
        ("LOSS ", "LOSS "),
        ("MOSS ", "MOSS "),
        ("boss ", "boss "),
        ("buff ", "buff "),
        ("cuff ", "cuff "),
        ("puff ", "puff "),
        // NOT in whitelist → keep buffer (clean, no marks)
        ("SOSS ", "SOS "), // soss not in whitelist → keep buffer SOS
        ("varr ", "var "),
        ("VARR ", "VAR "),
        ("norr ", "nor "),
        ("boxx ", "box "),
        ("hajj ", "haj "),
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

#[test]
fn debug_deeper_issue() {
    // This test checks the "deeper" → "ddeeper" bug
    // Words with double 'ee' pattern:
    // - Invalid Vietnamese buffer → restore to English (deeper, keeper, between)
    // - Valid Vietnamese buffer → keep Vietnamese (teen → tên)
    telex_auto_restore(&[
        ("deeper ", "deeper "),   // Invalid VN → restore
        ("keeper ", "keeper "),   // Invalid VN → restore
        ("teen ", "tên "),        // Valid VN "tên" → keep Vietnamese
        ("between ", "between "), // Invalid VN → restore
    ]);
}

// =============================================================================
// STROKE (đ) + DICTIONARY CHECK
// =============================================================================
// When buffer has stroke (đ from dd), use English dictionary to decide:
// - đ + in English dict → restore to English (daddy, add, odd)
// - đ + NOT in English dict → keep Vietnamese (đc, đt, đi)

#[test]
fn stroke_vietnamese_abbreviations() {
    // Vietnamese abbreviations with đ should stay Vietnamese
    // These are NOT in English dictionary
    telex_auto_restore(&[
        ("ddc ", "đc "),   // đc - được (common abbreviation)
        ("ddcs ", "đcs "), // đcs - đảng cộng sản
        ("ddt ", "đt "),   // đt - điện thoại (phone)
    ]);
}

#[test]
fn stroke_valid_vietnamese_words() {
    // Valid Vietnamese words with đ should stay Vietnamese
    telex_auto_restore(&[
        ("dde ", "đe "), // đe - to threaten
        ("ddi ", "đi "), // đi - to go
        ("ddo ", "đo "), // đo - to measure
        ("ddu ", "đu "), // đu - to swing
        ("dda ", "đa "), // đa - many/much
    ]);
}

#[test]
fn stroke_english_words_restore() {
    // English words with dd should restore to English
    // These ARE in English dictionary
    telex_auto_restore(&[
        ("daddy ", "daddy "),   // daddy - father
        ("add ", "add "),       // add - addition
        ("odd ", "odd "),       // odd - strange
        ("ladder ", "ladder "), // ladder - stairs
    ]);
}

#[test]
fn common_vietnamese_words_with_tone() {
    // Common Vietnamese words with tone marks should stay Vietnamese
    // These should NOT be in telex_doubles whitelist
    telex_auto_restore(&[
        ("chir ", "chỉ "), // chỉ - only/just (hỏi tone)
        ("chis ", "chí "), // chí - will/spirit (sắc tone)
        ("chij ", "chị "), // chị - older sister (nặng tone)
        ("thir ", "thỉ "), // thỉ - rare (hỏi tone)
        ("nhir ", "nhỉ "), // nhỉ - right? (hỏi tone)
    ]);
}
