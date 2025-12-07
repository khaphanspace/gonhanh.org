//! Common Issues Tests
//!
//! Test cases for common issues documented in docs/common-issues.md
//! These tests verify the engine handles edge cases correctly.

mod common;
use common::{run_telex, run_vni};

// ============================================================
// ISSUE 2.1: Dính chữ (aa -> aâ instead of â)
// Engine should return correct backspace count
// ============================================================

#[test]
fn telex_circumflex_not_sticky() {
    // These should produce single character output, not doubled
    run_telex(&[
        ("aa", "â"),   // NOT "aâ"
        ("ee", "ê"),   // NOT "eê"
        ("oo", "ô"),   // NOT "oô"
        ("dd", "đ"),   // NOT "dđ"
        ("DD", "Đ"),   // NOT "DĐ"
    ]);
}

#[test]
fn vni_modifier_not_sticky() {
    run_vni(&[
        ("a6", "â"),   // NOT "a6" or "aâ"
        ("e6", "ê"),
        ("o6", "ô"),
        ("d9", "đ"),
        ("D9", "Đ"),
    ]);
}

// ============================================================
// ISSUE 2.4: Lặp chữ (được -> đđược)
// Engine buffer should handle 'd' correctly
// ============================================================

#[test]
fn telex_no_double_d() {
    // đ should appear once, not twice
    run_telex(&[
        ("dduwowcj", "được"),   // NOT "đđược"
        ("ddif", "đì"),         // NOT "đđì"
        ("ddi", "đi"),          // NOT "đđi"
        ("ddang", "đang"),      // NOT "đđang"
        ("ddaauf", "đầu"),      // NOT "đđầu"
    ]);
}

#[test]
fn vni_no_double_d() {
    run_vni(&[
        ("d9u7o7c5", "được"),   // NOT "đđược"
        ("d9i", "đi"),
        ("d9ang", "đang"),
    ]);
}

// ============================================================
// ISSUE 2.4: Mất dấu (trường -> trương)
// Tone mark should be preserved on correct vowel
// ============================================================

#[test]
fn telex_preserve_tone_mark() {
    // Mark should appear on correct vowel
    run_telex(&[
        ("truwowngf", "trường"),   // NOT "trương"
        ("dduwowngf", "đường"),    // NOT "đương"
        ("nguwowif", "người"),     // NOT "ngươi"
        ("muwowif", "mười"),       // NOT "mươi"
    ]);
}

#[test]
fn vni_preserve_tone_mark() {
    run_vni(&[
        ("tru7o7ng2", "trường"),
        ("d9u7o7ng2", "đường"),
        ("ngu7o7i2", "người"),
    ]);
}

// ============================================================
// Edge case: Rapid typing patterns
// User types faster than normal, keys arrive in quick succession
// ============================================================

#[test]
fn telex_rapid_compound_vowels() {
    // Common words typed rapidly
    run_telex(&[
        // Full ươ compound with various marks
        ("truwowngf", "trường"),
        ("dduwowcj", "được"),
        ("suwowngs", "sướng"),
        ("buwowms", "bướm"),
        // iê compound
        ("vieetj", "việt"),
        ("tieengs", "tiếng"),
        ("bieenr", "biển"),
        // uô compound
        ("muoons", "muốn"),
        ("cuoocj", "cuộc"),
        ("thuoocj", "thuộc"),
    ]);
}

// ============================================================
// Edge case: Mixed order typing
// User types marks/tones at different positions
// ============================================================

#[test]
fn telex_delayed_all_patterns() {
    // Delayed mode: tone key after consonants
    run_telex(&[
        // w after whole syllable
        ("tungw", "tưng"),
        ("tongw", "tơng"),
        ("tangw", "tăng"),
        // Multiple w for ươ
        ("tuoww", "tươ"),
        ("nguoiw", "ngưoi"),  // first w on u
        ("nguoiww", "ngươi"), // second w on o
    ]);
}

#[test]
fn vni_delayed_all_patterns() {
    run_vni(&[
        // Delayed modifier
        ("tung7", "tưng"),
        ("tong7", "tơng"),
        ("tang8", "tăng"),
        // Delayed đ
        ("dung9", "đung"),
        ("Dung9", "Đung"),
    ]);
}

// ============================================================
// Edge case: Backspace and retype
// User corrects mistakes mid-word
// ============================================================

#[test]
fn telex_correction_patterns() {
    // Common correction scenarios
    run_telex(&[
        // Type wrong mark, then correct (mark replacement)
        ("asf", "à"),  // á then f replaces sắc with huyền → à
        ("afs", "á"),  // à then s replaces huyền with sắc → á
        // Backspace and retype (backspace removes last char from screen AND buffer)
        ("aas<s", "ấ"),   // ấ + backspace (removes ấ) + ... wait, this is complex
        // Simple case: type wrong, backspace, type correct
        ("ab<c", "ac"),  // a + b + backspace + c = ac
    ]);
}

// ============================================================
// Edge case: All caps typing
// User types in ALL CAPS mode
// ============================================================

#[test]
fn telex_all_caps_words() {
    run_telex(&[
        ("VIEETJ", "VIỆT"),
        ("DDUWOWCJ", "ĐƯỢC"),
        ("TRUWOWNGF", "TRƯỜNG"),
        ("NGUWOWIF", "NGƯỜI"),
        ("DDUWOWNGF", "ĐƯỜNG"),
    ]);
}

#[test]
fn vni_all_caps_words() {
    run_vni(&[
        ("VIE65T", "VIỆT"),
        ("D9U7O7C5", "ĐƯỢC"),
        ("TRU7O7NG2", "TRƯỜNG"),
    ]);
}

// ============================================================
// Edge case: Words ending with mark/tone keys
// Keys that are both letters and modifiers
// ============================================================

#[test]
fn telex_letter_vs_modifier() {
    // 's' as letter vs 's' as sắc mark
    run_telex(&[
        ("sa", "sa"),    // s is consonant
        ("as", "á"),     // s is sắc mark
        ("sas", "sá"),   // first s consonant, second s mark
        ("sass", "sas"), // revert: sá + s = sas
    ]);

    // 'f' as letter vs 'f' as huyền mark
    run_telex(&[
        ("fa", "fa"),    // f is consonant (borrowed words)
        ("af", "à"),     // f is huyền mark
    ]);
}

// ============================================================
// Edge case: Buffer boundary
// Long words that might overflow buffer
// ============================================================

#[test]
fn telex_long_words() {
    run_telex(&[
        // Long compound words
        ("nghieeng", "nghiêng"),
        ("khuyeens", "khuyến"),
        ("truwowngf", "trường"),
        ("nguoongf", "nguồng"),  // unusual but valid
    ]);
}
