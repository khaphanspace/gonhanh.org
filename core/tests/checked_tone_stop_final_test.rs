//! Checked-tone rule: syllables ending in a stop consonant (p, t, c, ch, k)
//! may only carry sắc or nặng. Typing huyền/hỏi/ngã on a stop-final syllable
//! produces an impossible Vietnamese syllable ("ỏt", "òc", "ãch", ...), so the
//! modifier key must fall through as a literal instead of being applied.
//!
//! Regression test for issue #403 ("OTR" → "ỏt").

mod common;
use common::telex;

// =============================================================================
// BUG CASES — huyền(f) / hỏi(r) / ngã(x) + stop final MUST NOT apply the tone.
// The modifier stays a literal letter (the resulting text is the raw input).
// =============================================================================

#[test]
fn hoi_on_stop_final_is_rejected() {
    // r = hỏi
    telex(&[
        ("otr", "otr"), // issue #403 exact repro: was "ỏt"
        ("atr", "atr"),
        ("itr", "itr"),
        ("ocr", "ocr"),
        ("opr", "opr"),
        ("achr", "achr"), // ch final
        ("ichr", "ichr"),
        ("hotr", "hotr"), // with initial consonant
        ("matr", "matr"),
    ]);
}

#[test]
fn huyen_on_stop_final_is_rejected() {
    // f = huyền
    telex(&[
        ("otf", "otf"),
        ("atf", "atf"),
        ("ocf", "ocf"),
        ("opf", "opf"),
        ("echf", "echf"),
        ("matf", "matf"),
        ("hotf", "hotf"),
    ]);
}

#[test]
fn nga_on_stop_final_is_rejected() {
    // x = ngã
    telex(&[
        ("otx", "otx"),
        ("itx", "itx"),
        ("ocx", "ocx"),
        ("opx", "opx"),
        ("ichx", "ichx"),
        ("hotx", "hotx"),
        ("ochx", "ochx"),
    ]);
}

// =============================================================================
// VALID CASES — sắc(s) / nặng(j) + stop final ARE allowed and must be kept.
// =============================================================================

#[test]
fn sac_and_nang_on_stop_final_are_kept() {
    telex(&[
        // sắc (s)
        ("ots", "ót"),
        ("ocs", "óc"),
        ("ops", "óp"),
        ("achs", "ách"),
        ("bats", "bát"),
        ("tots", "tót"),
        ("khachs", "khách"),
        ("vieejt", "việt"),
        ("muowjt", "mượt"),
        // nặng (j)
        ("otj", "ọt"),
        ("ocj", "ọc"),
        ("opj", "ọp"),
        ("achj", "ạch"),
        ("hocj", "học"),
        ("sachj", "sạch"),
    ]);
}

// =============================================================================
// VALID CASES — huyền/hỏi/ngã on SONORANT finals (m, n, ng, nh) stay Vietnamese.
// These share the same "final-first then mark" typing path as the bug cases and
// must not be affected by the fix.
// =============================================================================

#[test]
fn tones_on_sonorant_final_stay_vietnamese() {
    telex(&[
        ("lamf", "làm"),      // huyền + m
        ("hangf", "hàng"),    // huyền + ng
        ("tinhr", "tỉnh"),    // hỏi + nh
        ("manhx", "mãnh"),    // ngã + nh
        ("cungx", "cũng"),    // ngã + ng
        ("nhuwxng", "những"), // ngã + ng with horn
    ]);
}
