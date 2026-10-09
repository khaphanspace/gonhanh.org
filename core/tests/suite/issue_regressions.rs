//! One case per typing problem reported in the project's issues (github.com/khaphanspace/gonhanh.org).
//! `#N` is the issue number; `<` is Backspace. Every case runs on the active engine and all
//! failures are listed together, so one run shows the whole picture.

use crate::common::type_word;
use gonhanh_core::engine::Engine;

#[derive(Clone, Copy, Debug)]
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
    /// Telex as the app sets it with free typing on: free typing, foreign initials, auto-restore
    TelexFreeAr,
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
        TelexFreeAr => {
            e.set_free_tone(true);
            e.set_allow_foreign_consonants(true);
            e.set_english_auto_restore(true);
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
    // the report: with w as a consonant allowed (free typing / foreign initials) ww must still give w
    // --- free typing relaxes the initial only: foreign codas and stray vowels are left as typed --
    (0,   Telex,       "muafaaaa ",  "mùaaaaa "),
    (0,   Telex,       "mufaaaaa ",  "mùaaaaa "),
    (0,   Telex,       "buoifooo ",  "buòioo "),
    (0,   Telex,       "chuyenfe ",  "chuyền "),
    (211, Telex,       "hara ",      "hẩ "),
    (211, Telex,       "quasa ",     "quấ "),
    (0,   Telex,       "bafan ",     "bần "),
    (0,   Telex,       "mufaaa ",    "mùaaa "),
    (0,   Telex,       "hoafaa ",    "hoàaa "),
    (0,   TelexFree,   "dddd ",      "ddd "),
    (0,   TelexFreeAr, "wwww ",      "www "),
    (0,   Telex,       "aaaa ",      "aaa "),
    (0,   TelexAr,     "wws ",       "ws "),
    (0,   TelexFreeAr, "wws ",       "ws "),
    (0,   TelexFreeAr, "wwes ",      "wé "),
    (0,   TelexFreeAr, "xinchaof ",  "xinchào "),
    (0,   TelexFreeAr, "thuwrgoxTieengsVieetj ", "thửgõTiếngViệt "),
    (0,   TelexFreeAr, "anhyeeu ",   "anhyêu "),
    (0,   TelexFree,   "xinchaof ",  "xinchào "),
    (0,   TelexFreeAr, "xinchaoftooithuwr ", "xinchàotôithử "),
    (0,   TelexFreeAr, "helloworld ", "helloworld "),
    (0,   TelexFreeAr, "grassesrailway ", "grassesrailway "),
    (0,   TelexFreeAr, "dennisdungarees ", "dennisdungarees "),
    (0,   TelexFreeAr, "popover ",   "popover "),
    (0,   TelexFreeAr, "nginx ",     "nginx "),
    (0,   TelexFreeAr, "kubernetes ", "kubernetes "),
    (0,   TelexFreeAr, "mongodb ",   "mongodb "),
    (0,   TelexFree,   "nginx ",     "nginx "),
    (0,   TelexFreeAr, "ww ",        "w "),
    (0,   TelexFree,   "ww ",        "w "),
    (0,   TelexFree,   "revert ",    "revert "),
    (0,   TelexFreeAr, "revert ",    "revert "),
    (0,   TelexFree,   "reverted ",  "reverted "),
    (0,   TelexFreeAr, "reverts ",   "reverts "),
    (0,   TelexFree,   "tioo ",      "tioo "),
    (0,   TelexFreeAr, "tioo ",      "tioo "),
    (0,   Telex,       "perr ",      "per "),
    (0,   TelexAr,     "perr ",      "per "),
    (0,   TelexFree,   "perr ",      "per "),
    (0,   TelexFreeAr, "perr ",      "per "),
    (0,   TelexFree,   "perry ",     "perry "),
    (0,   TelexFree,   "class ",     "class "),
    (0,   TelexFreeAr, "terr ",      "ter "),
    (0,   TelexFreeAr, "thanks ",    "thanks "),
    (0,   TelexFreeAr, "hello ",     "hello "),
    (0,   TelexFree,   "ads ",       "ads "),
    (0,   TelexFreeAr, "ads ",       "ads "),
    (0,   TelexFreeAr, "expect ",    "expect "),
    (0,   TelexFreeAr, "express ",   "express "),
    (0,   TelexFree,   "haaaas ",    "haaas "), // the revert eats one a
    (0,   TelexFree,   "hasaaaaaa ", "háaaaaa "),
    (0,   TelexFreeAr, "hasaaaaaa ", "háaaaaa "),
    (0,   TelexFreeAr, "foresee ",   "foresee "),
    (0,   TelexFreeAr, "west ",      "west "),
    (0,   TelexFreeAr, "were ",      "were "),
    (0,   TelexFreeAr, "warm ",      "warm "),
    (0,   TelexFreeAr, "khphas ",    "khphá "),
    (0,   TelexFreeAr, "qcaos ",     "qcáo "),
    (0,   TelexFreeAr, "wes ",       "wé "),
    (0,   TelexFreeAr, "zij ",       "zị "),
    (252, TelexFree, "ww ",         "w "),
    (252, TelexFree, "Ww ",         "W "),
    (252, TelexFree, "WW ",         "W "),
    (252, TelexForeign, "ww ",      "w "),
    (252, TelexFreeForeign, "ww ",  "w "),
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
    // --- #26 (auto-restore feedback thread): what users typed and what they meant ---------------
    (26,  TelexAr, "tieeps ",       "tiếp "),
    (26,  TelexAr, "xeeps ",        "xếp "),
    (26,  TelexAr, "ddaay ",        "đây "),
    (26,  TelexAr, "ddaya ",        "đây "),
    (26,  TelexAr, "ura ",          "ủa "),
    (26,  TelexAr, "chiuj ",        "chịu "),
    (26,  TelexAr, "chiju ",        "chịu "),
    (26,  TelexAr, "thuyr ",        "thuỷ "),
    (26,  TelexOld, "thury ",       "thủy "),
    (26,  TelexAr, "aro ",          "ảo "),
    (26,  TelexAr, "arro ",         "aro "),
    (26,  TelexAr, "users ",        "users "),
    (26,  TelexAr, "things ",       "things "),
    // "see" is both English and a (rare) Vietnamese syllable: Vietnamese wins, ESC restores
    (26,  TelexAr, "see ",          "sê "),
    (26,  TelexAr, "offline ",      "offline "),
    (26,  TelexAr, "offensive ",    "offensive "),
    // "off" vs "of": the cancelled form is a word too, and a double key is how Telex cancels
    (26,  TelexAr, "off ",          "of "),
    (26,  TelexAr, "param ",        "param "),
    (26,  TelexAr, "goes ",         "goes "),
    (26,  TelexAr, "guess ",        "guess "),
    (26,  TelexAr, "mason ",        "mason "),
    (26,  TelexAr, "massive ",      "massive "),
    (26,  TelexAr, "reff ",         "ref "),
    (26,  TelexAr, "taxxi ",        "taxi "),
    (26,  TelexAr, "too ",          "tô "),
    (26,  TelexAr, "moef ",         "moè "),
    (26,  TelexAr, "tuji ",         "tụi "),
    (26,  TelexAr, "tuij ",         "tụi "),
    (26,  TelexAr, "vajan ",        "vận "),
    (26,  TelexAr, "thajat ",       "thật "),
    (26,  TelexAr, "cowork ",       "cowork "),
    (146, TelexAr, "toms ",         "tóm "),
    (319, Telex,   "dataad ",       "datad "),
    (337, TelexAr, "buss ",         "bus "),
    // "moss" is a word: the typed spelling wins over the cancelled "mos"
    (356, TelexAr, "moss ",         "moss "),
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
    (0,   TelexFree, "wes ",        "wé "),
    (232, TelexFree, "zij ",        "zị "),
    (0,   TelexFree, "wes ",        "wé "),
    (0,   TelexFree, "zij ",        "zị "),
    (0,   TelexFree, "wa ",         "ưa "),
    (0,   TelexFree, "wl ",         "wl "),
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

// --- shortcuts (gõ tắt): what users expect after typing, correcting and punctuation -----------------

use gonhanh_core::engine::shortcut::Shortcut;

fn with_shortcuts(mode: Mode, list: &[(&str, &str)]) -> Engine {
    let mut e = engine(mode);
    for (trigger, text) in list {
        e.shortcuts_mut().add(Shortcut::new(trigger, text));
    }
    e
}

// (issue, mode, shortcuts, typed, expected)
#[rustfmt::skip]
type Shortcuts = &'static [(&'static str, &'static str)];

const SHORTCUT_CASES: &[(u32, Mode, Shortcuts, &str, &str)] = &[
    (23,  Telex,   &[("zz", "hello")],            "zz ",        "hello "),
    (25,  Telex,   &[("qc", "quảng cáo")],        "qc ",        "quảng cáo "),
    (212, Telex,   &[("qc", "quảng cáo")],        "qx<c ",      "quảng cáo "),
    (382, Telex,   &[("qc", "quảng cáo")],        "qa<c ",      "quảng cáo "),
    (383, TelexAr, &[("qc", "quảng cáo")],        "qc. ",       "quảng cáo. "),
    (383, TelexAr, &[("qc", "quảng cáo")],        "qc, ",       "quảng cáo, "),
    (275, Telex,   &[("dc", "được")],             "dc! ",       "được! "),
    (129, Telex,   &[("hn", "Hà Nội")],           "hn ",        "Hà Nội "),
    (130, Telex,   &[("hn", "Hà Nội")],           "hn hn hn ",  "Hà Nội Hà Nội Hà Nội "),
    (86,  Telex,   &[("vn", "Việt Nam")],         "vn ",        "Việt Nam "),
    // an expansion longer than 63 characters must arrive whole (#178)
    (178, Telex,   &[("ml", "một hai ba bốn năm sáu bảy tám chín mười một hai ba bốn năm sáu bảy tám chín mười mười một")], "ml ", "một hai ba bốn năm sáu bảy tám chín mười một hai ba bốn năm sáu bảy tám chín mười mười một "),
];

#[test]
fn every_shortcut_issue_case_expands_as_expected() {
    let mut failures = Vec::new();
    for &(issue, mode, list, typed, want) in SHORTCUT_CASES {
        let got = type_word(&mut with_shortcuts(mode, list), typed);
        if got != want {
            failures.push(format!("#{issue}: {typed:?} → {got:?}, expected {want:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} shortcut cases fail:\n{}",
        failures.len(),
        SHORTCUT_CASES.len(),
        failures.join("\n")
    );
}

// --- breaking the rhythm with Control (#150, #359, #360): the app calls `clear()` on a Control tap,
// the next letters start a fresh word and no shortcut sees the letters before the break --------------

/// Type `parts` with a Control tap (`clear`) between them.
fn typed_with_breaks(mode: Mode, shortcuts: &[(&str, &str)], parts: &[&str]) -> String {
    let mut e = with_shortcuts(mode, shortcuts);
    let mut screen = String::new();
    for part in parts {
        let mut shown = type_word(&mut e, part);
        // type_word restarts the screen each call: only the new part is returned
        screen.push_str(&shown);
        shown.clear();
        e.clear();
    }
    screen
}

#[test]
fn control_tap_breaks_the_word_and_the_shortcut() {
    // #359 / #360: kh ⌃ phas → khphá, without any free-typing option
    assert_eq!(typed_with_breaks(Telex, &[], &["kh", "phas "]), "khphá ");
    // #360: with a shortcut qc → quảng cáo, q ⌃ c keeps the letters
    assert_eq!(
        typed_with_breaks(Telex, &[("qc", "quảng cáo")], &["q", "c "]),
        "qc "
    );
    // and without the break the shortcut still fires
    assert_eq!(
        typed_with_breaks(Telex, &[("qc", "quảng cáo")], &["qc "]),
        "quảng cáo "
    );
}

/// Once a word is not Vietnamese the tone stays on the vowel it was shown on: letters typed after
/// it never make the mark jump (háaa + e must not become haáae).
#[test]
fn tone_stays_put_while_typing_a_non_vietnamese_word() {
    let tone_at = |s: &str| s.chars().position(|c| !c.is_ascii());
    for mode in [Telex, TelexAr, TelexFree, TelexFreeAr] {
        for word in [
            "hasaaaaaae",
            "hasaaaee",
            "hoasaaae",
            "hasaaaaan",
            "tasoooox",
        ] {
            let mut prev: Option<usize> = None;
            for k in 5..=word.len() {
                let shown = type_word(&mut engine(mode), &word[..k]);
                let at = tone_at(&shown);
                if let (Some(a), Some(b)) = (prev, at) {
                    assert_eq!(
                        a, b,
                        "{mode:?} {word:?}: {shown:?} moved the tone at key {k}"
                    );
                }
                prev = at.or(prev);
            }
        }
    }
}

/// Stretching the last vowel of a toned syllable (mùa + aaaa) leaves the tone where it was: the
/// cancelled circumflex puts the word back to how it was before the modifier.
#[test]
fn tone_stays_when_the_last_vowel_is_stretched() {
    let tone_at = |s: &str| s.chars().position(|c| !c.is_ascii());
    for mode in [Telex, TelexAr, TelexFree, TelexFreeAr] {
        for (typed, v) in [
            ("muaf", 'a'),
            ("mufa", 'a'),
            ("chuas", 'a'),
            ("cuar", 'a'),
            ("buaj", 'a'),
            ("buoif", 'o'),
            ("chuoiso", 'o'),
            ("hoaf", 'a'),
            ("laf", 'a'),
            ("hoef", 'e'),
            ("khuyaf", 'a'),
        ] {
            let base = type_word(&mut engine(mode), typed);
            let want = tone_at(&base);
            assert!(want.is_some(), "{typed:?} shows no tone: {base:?}");
            let long = type_word(&mut engine(mode), &format!("{typed}{v}{v}{v}{v}"));
            // with auto-restore a word that is no longer Vietnamese may show its raw letters
            if long.is_ascii() {
                continue;
            }
            assert_eq!(
                tone_at(&long),
                want,
                "{mode:?} {typed:?}+{v}x4: {base:?} → {long:?}"
            );
        }
    }
}

/// Free typing must not make English words flicker more than normal typing: a reading that only
/// free typing allows (west → wét) is not shown while the letters still begin an English word.
#[test]
fn free_typing_does_not_add_rewrites_to_english_words() {
    let rewrites = |mode: Mode, word: &str| -> usize {
        let mut e = engine(mode);
        word.chars()
            .map(|c| {
                e.on_key_ext(gonhanh_core::utils::char_to_key(c), false, false, false)
                    .backspace as usize
            })
            .sum()
    };
    for word in [
        "west", "were", "warm", "wasp", "foresee", "thanks", "hello", "expect", "zoo", "fix",
        "file", "jazz", "brass", "first", "world", "fresh", "wrote", "flow", "jump", "zero",
    ] {
        let (free, normal) = (rewrites(TelexFreeAr, word), rewrites(TelexAr, word));
        assert!(
            free <= normal + 1,
            "{word:?}: {free} rewrites in free mode, {normal} normally"
        );
    }
}

/// Typing the tone key twice cancels the tone at once, in every mode and without a space (a word
/// of the English dictionary, like "tess", is shown as typed while auto-restore is on):
/// the word on screen is already the cancelled one (tesst → test), not the raw letters.
#[test]
fn a_doubled_key_cancels_without_waiting_for_a_space() {
    for mode in [Telex, TelexAr, TelexFree, TelexFreeAr] {
        for (typed, want) in [
            ("tesst", "test"),
            ("wws", "ws"),
            ("eee", "ee"),
            ("dddd", "ddd"),
        ] {
            let got = type_word(&mut engine(mode), typed);
            assert_eq!(got, want, "{mode:?} {typed:?}");
        }
    }
}

#[test]
fn a_doubled_key_cancels_in_modes_without_english_guard() {
    for mode in [Telex, TelexFree] {
        assert_eq!(type_word(&mut engine(mode), "xuss"), "xus", "{mode:?}");
    }
}

/// Cancelling by typing a key twice goes back to the plain letters, in every mode, at once and
/// again at the space, for any syllable: tone keys (perr → per), circumflex (baaa → baa), stroke
/// (ddd → dd) and w (ww → w). Stems that are, or start, a word of the English dictionaries are
/// left out: there the dictionary may rightly keep the letters as typed.
#[test]
fn a_doubled_key_goes_back_to_the_plain_letters_for_any_syllable() {
    use gonhanh_core::data::lexicon::{EN, REF};
    let initials = [
        "b", "c", "d", "g", "h", "k", "l", "m", "n", "p", "r", "s", "t", "v", "x", "kh", "ng",
        "th", "tr",
    ];
    let vowels = ["a", "e", "i", "o", "u", "ai", "ao", "ua", "ie"];
    let finals = ["", "n", "m", "t", "c", "p", "ng"];
    // a word, the start of one, or two words stuck together (die + css)
    let is_english = |w: &str| {
        EN.begins_inflected(w)
            || REF.begins_inflected(w)
            || (3..w.len().saturating_sub(2)).any(|i| {
                EN.contains_lower(&w[..i])
                    && (EN.contains_lower(&w[i..]) || EN.has_prefix_lower(&w[i..]))
            })
    };
    let mut checked = 0;
    let mut failures = Vec::new();
    for ini in initials {
        for v in vowels {
            for fin in finals {
                let stem = format!("{ini}{v}{fin}");
                // (typed keys, plain letters expected)
                let mut cases = Vec::new();
                for k in ["s", "f", "r", "x", "j"] {
                    cases.push((format!("{stem}{k}{k}"), format!("{stem}{k}")));
                }
                // a doubled vowel key: the third press cancels the circumflex
                for c in ["a", "e", "o"] {
                    if v == c && fin.is_empty() {
                        cases.push((format!("{ini}{c}{c}{c}"), format!("{ini}{c}{c}")));
                    }
                }
                cases.push((format!("{stem}ww"), format!("{stem}w")));
                if ini == "d" {
                    cases.push((format!("ddd{v}{fin}"), format!("dd{v}{fin}")));
                }
                for (typed, plain) in cases {
                    if is_english(&typed) || is_english(&plain) {
                        continue;
                    }
                    for mode in [Telex, TelexAr, TelexFree, TelexFreeAr] {
                        // nothing to cancel when the key typed once changes nothing
                        if type_word(&mut engine(mode), &plain) == plain {
                            continue;
                        }
                        checked += 1;
                        let now = type_word(&mut engine(mode), &typed);
                        let at_space = type_word(&mut engine(mode), &format!("{typed} "));
                        if now != plain || at_space != format!("{plain} ") {
                            failures.push(format!(
                                "{mode:?} {typed:?}: {now:?} / {at_space:?}, plain is {plain:?}"
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {checked} cancels did not go back:\n{}",
        failures.len(),
        failures
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The same for VNI: a digit typed twice cancels what the first one did.
#[test]
fn a_doubled_vni_digit_goes_back_to_the_plain_letters() {
    let initials = [
        "b", "c", "d", "g", "h", "k", "l", "m", "n", "t", "v", "x", "th", "tr",
    ];
    let vowels = ["a", "e", "i", "o", "u", "ai", "ao", "ua"];
    let finals = ["", "n", "m", "t", "c", "ng"];
    let mut checked = 0;
    let mut failures = Vec::new();
    for ini in initials {
        for v in vowels {
            for fin in finals {
                let stem = format!("{ini}{v}{fin}");
                for k in ["1", "2", "3", "4", "5", "6", "7", "8", "9"] {
                    let (typed, plain) = (format!("{stem}{k}{k}"), format!("{stem}{k}"));
                    // nothing to cancel when the digit typed once changes nothing
                    if type_word(&mut engine(Vni), &plain) == plain {
                        continue;
                    }
                    checked += 1;
                    let now = type_word(&mut engine(Vni), &typed);
                    let at_space = type_word(&mut engine(Vni), &format!("{typed} "));
                    if now != plain || at_space != format!("{plain} ") {
                        failures.push(format!(
                            "{typed:?}: {now:?} / {at_space:?}, plain is {plain:?}"
                        ));
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {checked} VNI cancels did not go back:\n{}",
        failures.len(),
        failures
            .iter()
            .take(30)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
