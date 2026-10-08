//! One case per typing problem reported in the project's issues (github.com/khaphanspace/gonhanh.org).
//! `#N` is the issue number; `<` is Backspace. Every case runs on the active engine and all
//! failures are listed together, so one run shows the whole picture.

use crate::common::type_word;
use gonhanh_core::engine::Engine;

#[derive(Clone, Copy)]
enum Mode {
    /// Telex, English auto-restore off
    Telex,
    /// Telex, English auto-restore on
    TelexAr,
    /// Telex, old tone placement (òa úy), auto-restore on
    TelexOld,
    /// Telex with `[` `]` as ơ / ư
    TelexBracket,
    /// Telex, f z j w allowed as initials, auto-restore on
    TelexForeign,
    /// Telex, free typing (tone marks on any syllable)
    TelexFree,
    /// Telex, free typing and f z j w allowed as initials
    TelexFreeForeign,
    Vni,
}
use Mode::*;

fn engine(mode: Mode) -> Engine {
    let mut e = Engine::new();
    match mode {
        Telex => {}
        TelexAr => e.set_english_auto_restore(true),
        TelexOld => {
            e.set_english_auto_restore(true);
            e.set_modern_tone(false);
        }
        TelexBracket => e.set_bracket_shortcut(true),
        TelexForeign => {
            e.set_english_auto_restore(true);
            e.set_allow_foreign_consonants(true);
        }
        TelexFree => e.set_free_tone(true),
        TelexFreeForeign => {
            e.set_free_tone(true);
            e.set_allow_foreign_consonants(true);
        }
        Vni => e.set_method(1),
    }
    e
}

// (issue, mode, typed, expected on screen)
#[rustfmt::skip]
const CASES: &[(u32, Mode, &str, &str)] = &[
    // --- Telex word order: the same word typed in any order gives the same result -------------
    (14,  Telex,   "ddwocj ",       "được "),
    (24,  Telex,   "ddojc ",        "đọc "),
    (29,  Telex,   "dduwowcj ",     "được "),
    (124, Telex,   "duod ",         "đuo "),
    (136, Telex,   "duodwjc ",      "được "),
    (32,  Telex,   "ddojc ",        "đọc "),
    (48,  Telex,   "ddeso ",        "đéo "),
    (48,  Telex,   "ddeos ",        "đéo "),
    (74,  Telex,   "giuawx ",       "giữa "),
    (88,  Telex,   "mafu ",         "màu "),
    (99,  TelexAr, "twong ",        "tương "),
    (105, Telex,   "ddoongf nghieepj ", "đồng nghiệp "),
    (106, Telex,   "giow ",         "giơ "),
    (110, Telex,   "ddeens ",       "đến "),
    (111, Telex,   "sow ",          "sơ "),
    (111, Telex,   "mow ",          "mơ "),
    (122, Telex,   "wfm ",          "ừm "),
    (122, Telex,   "Wfm ",          "Ừm "),
    (122, Telex,   "Wf ",           "Ừ "),
    (125, Telex,   "ddoong ",       "đông "),
    (125, Telex,   "doodng ",       "đông "),
    (127, Telex,   "ddaau ",        "đâu "),
    (151, Telex,   "muwa ",         "mưa "),
    (154, TelexAr, "seeps ",        "sếp "),
    (154, TelexAr, "sesep ",        "sếp "),
    (154, TelexAr, "sepes ",        "sếp "),
    (154, TelexAr, "sepse ",        "sếp "),
    (163, TelexAr, "beeps ",        "bếp "),
    (172, Telex,   "vietes ",       "viết "),
    (172, Telex,   "quyetes ",      "quyết "),
    (172, Telex,   "tuyetej ",      "tuyệt "),
    (182, Telex,   "hieuer ",       "hiểu "),
    (183, Telex,   "neues ",        "nếu "),
    (183, Telex,   "xauas ",        "xấu "),
    (196, Telex,   "thoio ",        "thôi "),
    (200, Telex,   "khoangr ",      "khoảng "),
    (243, Telex,   "quoiws ",       "quới "),
    (247, Telex,   "Dd ",           "Đ "),
    (252, Telex,   "ww ",           "w "),
    (253, Telex,   "hangf nganf ",  "hàng ngàn "),
    (259, Telex,   "xuatas ",       "xuất "),
    (262, Telex,   "banwfg ",       "bằng "),
    (262, Telex,   "thanwfg ",      "thằng "),
    (272, Telex,   "tieenf ",       "tiền "),
    (333, Telex,   "Dda ",          "Đa "),
    (333, Telex,   "ddos ",         "đó "),
    (340, Telex,   "toorng ",       "tổng "),
    (356, Telex,   "mos ",          "mó "),
    (371, Telex,   "vieejc ",       "việc "),
    (400, Telex,   "quets ",        "quét "),
    (303, Telex,   "truy caapj ",   "truy cập "),
    (236, Telex,   "uar ",          "ủa "),
    (236, Telex,   "ofo ",          "ồ "),
    // --- capitals and Caps Lock (typed with Shift / Caps held) ---------------------------------
    (166, Telex,   "DDaau ",        "Đâu "),
    (166, Telex,   "DD ",           "Đ "),
    (166, Telex,   "DDAAU ",        "ĐÂU "),
    (384, Telex,   "NHUWNG ",       "NHƯNG "),
    (384, Telex,   "Nhuw ",         "Như "),
    (199, Telex,   "nhuw ",         "như "),
    // --- tone placement option (òa úy vs oà uý) ------------------------------------------------
    (64,  TelexOld, "xosa ",        "xóa "),
    (64,  TelexOld, "tufy ",        "tùy "),
    (54,  TelexOld, "hoaf ",        "hòa "),
    (54,  Telex,    "hoaf ",        "hoà "),
    // --- English words must survive (auto-restore on) -------------------------------------------
    (15,  TelexAr, "metric ",       "metric "),
    (26,  TelexAr, "text ",         "text "),
    (26,  TelexAr, "where ",        "where "),
    (39,  TelexAr, "new ",          "new "),
    (44,  Telex,   "taiw ",         "taiw "),
    (51,  Telex,   "deadline ",     "deadline "),
    (115, TelexAr, "cursor ",       "cursor "),
    (116, TelexAr, "await ",        "await "),
    (118, TelexAr, "IMSS ",         "IMS "),
    (118, TelexAr, "usser ",        "user "),
    (131, TelexAr, "console ",      "console "),
    (142, TelexAr, "sims ",         "sims "),
    (145, TelexAr, "view ",         "view "),
    (147, TelexAr, "respect ",      "respect "),
    (184, TelexAr, "errovn ",       "erovn "),
    (193, TelexAr, "mussic ",       "music "),
    (197, TelexAr, "serv<r ",       "ser "),
    (230, TelexAr, "Therere ",      "There "),
    (246, TelexAr, "datasuite ",    "datasuite "),
    (296, TelexAr, "sess ",         "ses "),
    (337, TelexAr, "bussiness ",    "business "),
    // "momo" is môm typed with a late circumflex: a valid word wins; the report was the extra "o" (momoo)
    (348, TelexAr, "momo ",         "môm "),
    (355, TelexAr, "fomo ",         "fomo "),
    (367, TelexAr, "TOTO ",         "TOTO "),
    (367, TelexAr, "sata ",         "sata "),
    (402, TelexAr, "daydream ",     "daydream "),
    (403, TelexAr, "OTR ",          "OTR "),
    (410, TelexAr, "useEffect ",    "useEffect "),
    (427, TelexAr, "assam ",        "asam "),
    // --- backspace and corrections ---------------------------------------------------------------
    (217, Telex,   "Meeeee<<<<<<Phee ", "Phê "),
    (294, Telex,   "tar<rar ",      "trả "),
    (98,  Telex,   "toanf<<<<toanf ", "toàn "),
    // --- other typing aids ---------------------------------------------------------------------
    (159, TelexBracket, "a] ",     "aư "),
    (159, TelexBracket, "b[ ",     "bơ "),
    (162, Vni,     "o2o ",          "òo "),
    (232, TelexForeign, "zij ",     "zị "),
    (359, Telex,   "khphas ",       "khphas "),
    // --- free typing: marks on any syllable, grammar still wins when it can ----------------------
    (359, TelexFree, "khphas ",     "khphá "),
    (359, TelexFree, "qcaos ",      "qcáo "),
    (56,  TelexFree, "Zias ",       "Zía "),
    (56,  TelexFreeForeign, "Zias ", "Zía "),
    (232, TelexFreeForeign, "zij ", "zị "),
    (0,   TelexFreeForeign, "wes ", "wé "),
    (0,   TelexFreeForeign, "Wes ", "Wé "),
    (0,   TelexFree, "hoas ",       "hoá "),
    (0,   TelexFree, "toanf ",      "toàn "),
    (0,   TelexFree, "tieengs ",    "tiếng "),
];

#[test]
fn every_issue_case_types_as_reported_fixed() {
    let mut failures = Vec::new();
    for &(issue, mode, typed, want) in CASES {
        let got = type_word(&mut engine(mode), typed);
        if got != want {
            failures.push(format!("#{issue}: {typed:?} → {got:?}, expected {want:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} issue cases fail:\n{}",
        failures.len(),
        CASES.len(),
        failures.join("\n")
    );
}
