//! Comprehensive test suite for English word auto-restore feature.
//!
//! # How Auto-Restore Works
//!
//! When typing English words using Telex input method, certain letters act as
//! Vietnamese modifiers (s, f, r, x, j for tones; w for horn mark). This causes
//! English words to be incorrectly transformed. The auto-restore feature detects
//! invalid Vietnamese patterns and restores the original English text.
//!
//! # Detection Patterns
//!
//! The engine can detect English words using these patterns:
//!
//! 1. **Modifier + Consonant**: "text" (x followed by t), "expect" (x followed by p)
//! 2. **EI vowel pair + modifier**: "their" (ei+r)
//! 3. **AI vowel pair + P initial**: "pair" (P initial + ai + r)
//! 4. **Vowel + modifier + vowel (no initial)**: "use" (u+s+e)
//! 5. **W at start + consonant or later W**: "window", "wow"
//! 6. **Invalid Vietnamese initial (F)**: "fair", "fix"
//!
//! # Limitations
//!
//! Some English words produce structurally valid Vietnamese and CANNOT be
//! auto-detected without a dictionary:
//! - "mix" → "mĩ" (M is valid initial, ĩ is valid)
//! - "box" → "bõ" (B is valid initial, õ is valid)
//!
//! Users should use raw mode (\word) or Esc to restore these manually.

mod common;
use common::{telex, telex_auto_restore};

// =============================================================================
// PATTERN 1: MODIFIER FOLLOWED BY CONSONANT
// Example: "text" has x followed by t → clearly English
// =============================================================================

#[test]
fn pattern1_modifier_then_consonant() {
    telex_auto_restore(&[
        // x + consonant
        ("text ", "text "),
        ("next ", "next "),
        ("context ", "context "),
        ("textbook ", "textbook "),
        ("extend ", "extend "),
        ("extent ", "extent "),
        ("extern ", "extern "),
        ("extra ", "extra "),
        ("extract ", "extract "),
        ("extreme ", "extreme "),
        // exp- pattern (x + p)
        ("expect ", "expect "),
        ("export ", "export "),
        ("express ", "express "),
        ("expand ", "expand "),
        ("expense ", "expense "),
        ("expert ", "expert "),
        ("explore ", "explore "),
        ("exploit ", "exploit "),
        ("explode ", "explode "),
        ("explain ", "explain "),
        ("explicit ", "explicit "),
        ("experiment ", "experiment "),
        ("experience ", "experience "),
        // exc- pattern (x + c)
        ("excel ", "excel "),
        ("except ", "except "),
        ("excess ", "excess "),
        ("exchange ", "exchange "),
        ("excite ", "excite "),
        ("exclude ", "exclude "),
        ("excuse ", "excuse "),
        ("execute ", "execute "),
        // r + consonant (hỏi modifier followed by consonant)
        ("perfect ", "perfect "),
        ("hard ", "hard "),
        ("support ", "support "),
        // s + consonant: only longer words or invalid VN structure get restored
        // Short words like "test", "rest" form valid Vietnamese (tét, rét)
        ("tesla ", "tesla "),
        ("push ", "push "),
        // x + u patterns (ngã before vowel, then consonant)
        ("luxury ", "luxury "),
    ]);
}

// =============================================================================
// PATTERN 2: EI VOWEL PAIR + MODIFIER AT END
// Example: "their" has ei before r → English pattern
// =============================================================================

#[test]
fn pattern2_ei_vowel_pair() {
    telex_auto_restore(&[("their ", "their "), ("weird ", "weird ")]);
}

#[test]
fn pattern2_oo_vowel_pair() {
    telex_auto_restore(&[("looks ", "looks "), ("took ", "took ")]);
}

#[test]
fn pattern2_ee_vowel_pair() {
    telex_auto_restore(&[
        // With space - restore to English (invalid VN ending with -êp)
        ("keep ", "keep "),
        ("teep ", "teep "),
        // Without space - keep Vietnamese transform (word not complete)
        ("keep", "kêp"),  // k + e + e(circumflex) + p → kêp
        ("keeps", "kếp"), // k + e + e + p + s(sắc) → kếp
        ("teep", "têp"),  // t + e + e(circumflex) + p → têp
        ("teepj", "tệp"), // t + e + e + p + j(nặng) → tệp
    ]);
}

#[test]
fn pattern2_aa_vowel_pair() {
    telex_auto_restore(&[
        // Double 'a' creates circumflex â, but result is not valid Vietnamese
        ("saas ", "saas "),  // s+a+a+s → "sâs" invalid → restore "saas"
        ("saaas ", "saas "), // s+a+a+a+s → third 'a' reverts circumflex → "saas"
        ("sax ", "sax "),    // s+a+x → "sã" invalid word → restore "sax"
        ("saax ", "sax "),   // s+a+a+x → "sẫ" invalid → restore to buffer "sax"
        // Triple 'o' with consonant
        ("xooong ", "xoong "), // x+o+o+o+ng → triple 'o' collapses to double
        ("booong ", "boong "), // b+o+o+o+ng → triple 'o' collapses to double
        // Valid Vietnamese triphthongs - should NOT be restored
        ("ngueeuf ", "nguều "), // ng+u+ê+u with huyền → valid Vietnamese (ee for ê)
        ("ngoafo ", "ngoào "),  // ng+o+à+o - ôa is invalid, so 'o' appends raw
        ("ngoejo ", "ngoẹo "),  // ng+o+ẹ+o - oeo triphthong with nặng → valid Vietnamese
        // Triphthong without initial - should preserve, not apply circumflex
        ("oeo ", "oeo "),  // o+e+o → oeo triphthong, NOT ôe
        ("oejo ", "oẹo "), // o+e+j+o → oẹo (oeo with nặng)
    ]);
}

// =============================================================================
// PATTERN 3: AI VOWEL PAIR + RARE INITIAL (P)
// P alone (not PH) is rare in native Vietnamese
// =============================================================================

#[test]
fn pattern3_ai_with_p_initial() {
    telex_auto_restore(&[("pair ", "pair ")]);
}

// =============================================================================
// PATTERN 4: VOWEL + MODIFIER + VOWEL (NO INITIAL CONSONANT)
// Example: "use" starts with vowel, has s between u and e
// =============================================================================

#[test]
fn pattern4_vowel_modifier_vowel() {
    telex_auto_restore(&[
        ("use ", "use "),
        ("user ", "user "),
        ("users ", "users "),
        // "ussers" → "users": "u+ss" at word start is very rare in English
        // (no English words start with "uss"), so collapse double 's' to single
        ("ussers ", "users "),
    ]);
}

// =============================================================================
// PATTERN 5: W AT START + CONSONANT / W + VOWEL + W
// W is not a valid Vietnamese initial consonant
// =============================================================================

#[test]
fn pattern5_w_start_consonant() {
    telex_auto_restore(&[
        // w + consonant
        ("water ", "water "),
        ("winter ", "winter "),
        ("window ", "window "),
        ("with ", "with "),
        ("wonder ", "wonder "),
        ("wonderful ", "wonderful "),
        ("work ", "work "),
        ("worker ", "worker "),
        ("world ", "world "),
        ("worth ", "worth "),
        ("write ", "write "),
        ("wrong ", "wrong "),
        ("wrap ", "wrap "),
        ("wrist ", "wrist "),
        // wh- words
        ("what ", "what "),
        ("when ", "when "),
        ("where ", "where "),
        ("which ", "which "),
        ("while ", "while "),
        ("white ", "white "),
        ("whole ", "whole "),
        ("why ", "why "),
        ("wheat ", "wheat "),
        ("wheel ", "wheel "),
    ]);
}

#[test]
fn pattern5_w_vowel_w() {
    telex_auto_restore(&[("wow ", "wow ")]);
}

#[test]
fn pattern5_double_w_at_start() {
    // Double 'w' at start should collapse to single 'w' when restoring
    telex_auto_restore(&[("wwax ", "wax ")]);
}

#[test]
fn pattern_double_vowel_after_tone() {
    // When a vowel has a mark (huyền/sắc/etc.) and user types double DIFFERENT vowel,
    // circumflex should NOT be applied. This prevents invalid diphthongs like "ồa", "âi", etc.
    // Example: "tafoo" = t + à (huyền on 'a') + oo → skip circumflex → "tàoo"
    telex_auto_restore(&[
        // huyền (f) + different double vowel
        ("tafoo ", "tàoo "), // t + à + oo → 'a' has mark, 'o' different → skip circumflex
        ("tefoo ", "tèoo "), // t + è + oo → 'e' has mark, 'o' different → skip circumflex
        ("tofaa ", "tòaa "), // t + ò + aa → 'o' has mark, 'a' different → skip circumflex
        ("tofee ", "tòee "), // t + ò + ee → 'o' has mark, 'e' different → skip circumflex
        ("tifaa ", "tìaa "), // t + ì + aa → 'i' has mark, 'a' different → skip circumflex
        ("mufaa ", "mùaa "), // m + ù + aa → 'u' has mark, 'a' different → skip circumflex
        // sắc (s) + different double vowel
        ("tasoo ", "táoo "), // t + á + oo → 'a' has mark, 'o' different → skip circumflex
        ("tesaa ", "téaa "), // t + é + aa → 'e' has mark, 'a' different → skip circumflex
    ]);
}

#[test]
fn pattern_risk_words() {
    // Words ending with -isk/-usk pattern - should auto-restore
    telex_auto_restore(&[
        ("risk ", "risk "),
        ("disk ", "disk "),
        ("task ", "task "),
        ("mask ", "mask "),
        ("desk ", "desk "),
        ("dusk ", "dusk "),
        ("tusk ", "tusk "),
        ("husk ", "husk "),
    ]);
}

// =============================================================================
// PATTERN 6: INVALID VIETNAMESE INITIAL (F)
// F is not a Vietnamese initial (Vietnamese uses PH for /f/ sound)
// =============================================================================

#[test]
fn pattern6_invalid_initial_f() {
    telex_auto_restore(&[
        ("fair ", "fair "),
        ("fall ", "fall "),
        ("false ", "false "),
        ("far ", "far "),
        ("farm ", "farm "),
        ("fast ", "fast "),
        ("fat ", "fat "),
        ("fear ", "fear "),
        ("feature ", "feature "),
        ("feed ", "feed "),
        ("feel ", "feel "),
        ("few ", "few "),
        ("file ", "file "),
        ("fill ", "fill "),
        ("film ", "film "),
        ("final ", "final "),
        ("find ", "find "),
        ("fine ", "fine "),
        ("fire ", "fire "),
        ("firm ", "firm "),
        ("first ", "first "),
        ("fish ", "fish "),
        ("fit ", "fit "),
        ("fix ", "fix "),
        ("flag ", "flag "),
        ("flat ", "flat "),
        ("flex ", "flex "),
        ("floor ", "floor "),
        ("flow ", "flow "),
        ("fly ", "fly "),
        ("focus ", "focus "),
        ("fold ", "fold "),
        ("follow ", "follow "),
        ("font ", "font "),
        ("food ", "food "),
        ("foot ", "foot "),
        ("for ", "for "),
        ("force ", "force "),
        ("fork ", "fork "),
        ("form ", "form "),
        ("format ", "format "),
        ("forward ", "forward "),
        ("found ", "found "),
        ("four ", "four "),
        ("frame ", "frame "),
        ("free ", "free "),
        ("fresh ", "fresh "),
        ("from ", "from "),
        ("front ", "front "),
        ("full ", "full "),
        ("fun ", "fun "),
        ("function ", "function "),
        ("future ", "future "),
        // Tech terms with F
        ("facebook ", "facebook "),
        ("firebase ", "firebase "),
        ("firefox ", "firefox "),
        ("flutter ", "flutter "),
        ("framework ", "framework "),
        ("frontend ", "frontend "),
        ("fullstack ", "fullstack "),
    ]);
}

// =============================================================================
// TECH & PROGRAMMING TERMS (WITH DETECTABLE PATTERNS)
// =============================================================================

#[test]
fn tech_terms_restore() {
    telex_auto_restore(&[
        // exp- pattern
        ("Express ", "Express "),
        // ext- pattern
        ("extension ", "extension "),
        // F initial
        ("Firebase ", "Firebase "),
        ("Flutter ", "Flutter "),
        // W initial
        ("webpack ", "webpack "),
        ("WebSocket ", "WebSocket "),
        // Long words with -est/-ost pattern (validation prevents mark application)
        ("localhost ", "localhost "),
        ("request ", "request "),
        // NOTE: Short words like "post", "host" form valid Vietnamese (pót, hót)
        // and are NOT auto-restored. Users should use ESC for these.
    ]);
}

// =============================================================================
// PUNCTUATION TRIGGERS RESTORE
// =============================================================================

#[test]
fn punctuation_triggers_restore() {
    // Only certain punctuation triggers auto-restore (comma, period)
    telex_auto_restore(&[("text, ", "text, "), ("expect. ", "expect. ")]);
}

// =============================================================================
// VIETNAMESE WORDS THAT SHOULD NOT RESTORE
// =============================================================================

#[test]
fn vietnamese_single_syllable_preserved() {
    telex_auto_restore(&[
        // Single syllable with tones
        ("mas ", "má "), // má (mother)
        ("maf ", "mà "), // mà (but)
        ("mar ", "mả "), // mả (grave)
        ("max ", "mã "), // mã (horse - Sino-Viet)
        ("maj ", "mạ "), // mạ (rice seedling)
        ("bas ", "bá "), // bá (aunt)
        ("baf ", "bà "), // bà (grandmother)
        ("cas ", "cá "), // cá (fish)
        ("caf ", "cà "), // cà (eggplant)
        ("las ", "lá "), // lá (leaf)
        ("laf ", "là "), // là (is)
        ("tas ", "tá "), // tá (dozen)
        ("taf ", "tà "), // tà (side)
        // Post-tone delayed circumflex
        ("onro ", "ổn "), // ổn (okay) - o + n + r(hỏi) + o(circumflex)
    ]);
}

#[test]
fn vietnamese_multi_syllable_preserved() {
    telex_auto_restore(&[
        ("gox ", "gõ "),       // gõ (to type/knock)
        ("tooi ", "tôi "),     // tôi (I)
        ("Vieetj ", "Việt "),  // Việt
        ("thoaij ", "thoại "), // thoại (speech)
        ("giuwax ", "giữa "),  // giữa (middle)
        ("dduowcj ", "được "), // được (can/get)
        ("muwowjt ", "mượt "), // mượt (smooth)
        ("ddeso ", "đéo "),    // đéo (slang: no way)
        ("ddense ", "đến "),   // đến (to come/arrive)
    ]);
}

#[test]
fn vietnamese_ai_pattern_preserved() {
    // AI pattern with common Vietnamese initials should NOT restore
    telex_auto_restore(&[
        ("mais ", "mái "),     // mái (roof)
        ("cais ", "cái "),     // cái (classifier)
        ("xaif ", "xài "),     // xài (to use - Southern)
        ("taif ", "tài "),     // tài (talent)
        ("gais ", "gái "),     // gái (girl)
        ("hoaij ", "hoại "),   // hoại (decay)
        ("ngoaij ", "ngoại "), // ngoại (outside)
    ]);
}

#[test]
fn vietnamese_complex_words_preserved() {
    telex_auto_restore(&[
        // Words with horn marks (ư, ơ)
        ("nuwowcs ", "nước "),     // nước (water)
        ("dduowngf ", "đường "),   // đường (road)
        ("truwowcs ", "trước "),   // trước (before)
        ("giuwowngf ", "giường "), // giường (bed)
        ("twong ", "tương "),      // tương (mutual) - shorthand telex
        ("wng ", "ưng "),          // ưng (agree) - w→ư + ng final
        ("wong ", "ương "),        // ương (foster) - w→ư, o→ơ horn compound
        // Words with circumflex (â, ê, ô)
        ("caaps ", "cấp "), // cấp (level)
        ("taanf ", "tần "), // tần (frequency)
        ("laauj ", "lậu "), // lậu (illegal)
        ("leex ", "lễ "),   // lễ (ceremony)
    ]);
}

// =============================================================================
// AIR PATTERN - SPECIAL CASE
// "air" → "ải" is valid Vietnamese (border/pass), should NOT restore
// =============================================================================

#[test]
fn air_stays_vietnamese() {
    // "air" typed becomes "ải" - valid Vietnamese word
    // Should NOT restore because "ải" (border/pass) is a real word
    telex_auto_restore(&[("air ", "ải ")]);
}

// =============================================================================
// WORDS THAT CANNOT BE AUTO-DETECTED (DOCUMENTATION)
// These produce structurally valid Vietnamese
// =============================================================================

/// Documents words that CANNOT be auto-detected without a dictionary.
/// These produce structurally valid Vietnamese and are intentionally NOT restored.
/// Users should use raw mode prefix (\word) or Esc to get English spelling.
#[test]
fn words_that_stay_transformed() {
    // These produce valid Vietnamese structures - NOT auto-restored by design
    telex_auto_restore(&[
        ("mix ", "mĩ "), // m + i + x(ngã) → mĩ (valid Vietnamese: "beautiful" in Sino-Vietnamese)
        ("box ", "bõ "), // b + o + x(ngã) → bõ (valid Vietnamese structure)
        ("six ", "sĩ "), // s + i + x(ngã) → sĩ (valid Vietnamese: "scholar/official")
        ("tax ", "tã "), // t + a + x(ngã) → tã (valid Vietnamese: "diaper")
        ("max ", "mã "), // m + a + x(ngã) → mã (valid Vietnamese: "horse/code")
        ("fox ", "fox "), // F is invalid initial → auto-restores to "fox"
    ]);
}

// =============================================================================
// PATTERN 7: VOWEL + MODIFIER + VOWEL (WITH INITIAL CONSONANT)
// Example: "core" = c + o + r + e → "cỏe" invalid → restore
// =============================================================================

#[test]
fn pattern7_vowel_modifier_vowel_with_initial() {
    telex_auto_restore(&[
        ("core ", "core "),
        ("more ", "more "),
        ("care ", "care "),
        ("rare ", "rare "),
        ("are ", "are "),
        ("ore ", "ore "),
        ("bore ", "bore "),
        ("fore ", "fore "), // F initial also triggers Pattern 6
        ("sore ", "sore "),
        ("wore ", "wore "), // W initial also triggers Pattern 5
        ("store ", "store "),
        ("score ", "score "),
        ("goes ", "goes "),   // g + o + e + s → goé (invalid) → restore
        ("param ", "param "), // p + a + r + a + m → paảm (invalid) → restore
        ("life ", "life "),   // l + i + f + e → lìe (invalid) → restore
        // Short words: consonant + vowel + modifier (no final vowel)
        ("per ", "per "),      // p + e + r → pẻ (invalid) → restore "per"
        ("thiss ", "this "),   // t + h + i + s + s → double s reverts → buffer "this" (4 chars)
        ("mason ", "mason "),  // m + a + s + o + n → máon (invalid VN) → restore "mason"
        ("masson ", "mason "), // m + a + s + s + o + n → double s reverts → "mason"
    ]);
}

#[test]
fn vietnamese_ua_uo_preserved() {
    // Vietnamese ưa/ươ patterns should NOT restore
    // u + modifier + a → ưa family (cửa, mua, bưa)
    // u + modifier + o → ươ family (được, bước)
    telex_auto_restore(&[
        ("cura ", "của "),      // của (of) - common Vietnamese
        ("muar ", "mủa "),      // mủa (not common but valid structure)
        ("dduwowcj ", "được "), // được (can/get)
    ]);
}

// =============================================================================
// PATTERN 8: W AS FINAL (NOT MODIFIER)
// Example: "raw" = r + a + w → W can't modify A, stays as W final
// =============================================================================

#[test]
fn pattern8_w_as_final() {
    telex_auto_restore(&[
        ("raw ", "raw "),
        ("law ", "law "),
        ("saw ", "saw "),
        ("jaw ", "jaw "),
    ]);
}

// =============================================================================
// VIETNAMESE TONE MODIFIERS WITH SONORANT FINALS
// hỏi (r), huyền (f), ngã (x) + sonorant (m, n, nh, ng) should stay Vietnamese
// =============================================================================

#[test]
fn vietnamese_hoi_with_sonorant_final() {
    telex_auto_restore(&[
        // hỏi (r) + sonorant final (nh) - should stay Vietnamese
        ("nhirnh ", "nhỉnh "), // nhỉnh (a bit)
        ("tirnh ", "tỉnh "),   // tỉnh (province/wake)
        ("ddirnh ", "đỉnh "),  // đỉnh (peak)
        ("chirnh ", "chỉnh "), // chỉnh (adjust)
        // Alternative typing order
        ("nhinhr ", "nhỉnh "),
        ("tinhr ", "tỉnh "),
        ("ddinhr ", "đỉnh "),
        ("chinhr ", "chỉnh "),
        // huyền (f) + sonorant final (m, n, ng)
        ("lafm ", "làm "),   // làm (do/make)
        ("hafng ", "hàng "), // hàng (goods/row)
        ("dufng ", "dùng "), // dùng (use)
        // ngã (x) + sonorant final
        ("maxnh ", "mãnh "), // mãnh (fierce)
        ("haxnh ", "hãnh "), // hãnh (proud)
        // nặng (j) + stop final (c) - should stay Vietnamese
        ("trwjc ", "trực "), // trực (direct)
        ("bwjc ", "bực "),   // bực (annoyed)
        // ngã (x) before ng final - should stay Vietnamese
        // Pattern: C + U + N + X + G → cũng (modifier X splits the ng final)
        ("cunxg ", "cũng "), // cũng (also)
        ("cungx ", "cũng "), // cũng (standard typing order)
        ("cuxng ", "cũng "), // cũng (another valid order)
        ("hunxg ", "hũng "), // similar pattern with h initial
        // Tone modifier BEFORE final vowel (alternative typing order)
        ("gasi ", "gái "), // sắc before i
        ("gais ", "gái "), // sắc after i (standard)
        ("gaxy ", "gãy "), // ngã before y
        ("gayx ", "gãy "), // ngã after y (standard)
    ]);
}

// =============================================================================
// PATTERN 9: COMMON ENGLISH PREFIXES WITH MARK REVERT
// When user types double mark key (ss, ff, rr) to revert, and buffer forms
// a word with common English prefix/suffix, use buffer instead of raw_input.
// =============================================================================

#[test]
fn pattern9_dis_prefix() {
    // "dis-" prefix: double 's' reverts mark, buffer has valid prefix pattern
    telex_auto_restore(&[
        ("disable ", "disable "),  // normal typing
        ("dissable ", "disable "), // double 's' reverts → "disable" (dis- prefix)
        ("disscover ", "discover "),
        ("dissconnect ", "disconnect "),
        ("disscuss ", "discuss "),
        ("disspatch ", "dispatch "),
        ("disspute ", "dispute "),
        ("disstance ", "distance "),
        ("disstinct ", "distinct "),
        ("disstribute ", "distribute "),
        ("disstract ", "distract "),
        ("disstress ", "distress "),
        ("disstrust ", "distrust "),
    ]);
}

#[test]
fn pattern9_mis_prefix() {
    // "mis-" prefix: double 's' reverts mark, buffer has valid prefix pattern
    telex_auto_restore(&[
        ("misstake ", "mistake "),
        ("misstrust ", "mistrust "),
        ("misstype ", "mistype "),
        ("missplace ", "misplace "),
        ("misslead ", "mislead "),
        ("missread ", "misread "),
        ("missmatch ", "mismatch "),
    ]);
}

#[test]
fn pattern9_trans_prefix() {
    // "trans-" prefix: double 's' reverts mark
    telex_auto_restore(&[
        ("transsfer ", "transfer "),
        ("transsform ", "transform "),
        ("transsport ", "transport "),
        ("transsaction ", "transaction "),
        ("transsition ", "transition "),
        ("transsparent ", "transparent "),
        ("transsit ", "transit "),
    ]);
}

#[test]
fn pattern9_con_prefix() {
    // "con-" prefix: double 's' reverts mark, buffer has valid prefix pattern
    telex_auto_restore(&[
        ("console ", "console "),  // normal typing
        ("conssole ", "console "), // double 's' reverts → "console" (con- prefix)
        ("consscious ", "conscious "),
        ("conssider ", "consider "),
        ("conssequence ", "consequence "),
        ("consstant ", "constant "),
        ("consstruct ", "construct "),
        ("conssult ", "consult "),
        ("conssume ", "consume "),
        ("context ", "context "), // ext pattern restores (x+t)
    ]);
}

#[test]
fn pattern9_re_prefix() {
    // "re-" prefix: double 's' (sắc) after 'e' triggers revert
    // Pattern: re + ss → "rés" → "res" (revert)
    telex_auto_restore(&[
        ("ressponse ", "response "),
        ("ressource ", "resource "),
        ("ressult ", "result "),
        ("ressolve ", "resolve "),
        ("ressearch ", "research "),
        ("ressume ", "resume "),
        ("ressist ", "resist "),
        ("ressort ", "resort "),
        ("ressign ", "resign "),
    ]);
}

#[test]
fn pattern9_double_mark_no_prefix() {
    // Words with double mark keys but NO matching prefix/suffix pattern
    // 5+ char words: restore to English (preserve double letter)
    // 4-char words: keep reverted result (user explicitly typed double to revert)
    telex_auto_restore(&[
        // 5+ char words: restore to English
        ("issue ", "issue "),
        ("class ", "class "),
        ("cross ", "cross "),
        ("dress ", "dress "),
        ("glass ", "glass "),
        ("grass ", "grass "),
        ("gross ", "gross "),
        ("press ", "press "),
        ("stress ", "stress "),
        ("assess ", "assess "),
        ("possess ", "possess "),
        ("success ", "success "),
        ("express ", "express "),
        ("impress ", "impress "),
        ("process ", "process "),
        ("profess ", "profess "),
        ("progress ", "progress "),
        ("guess ", "guess "),
        ("massive ", "massive "),
        // Double 'r' without matching prefix (need vowel before rr)
        ("error ", "error "),
        ("mirror ", "mirror "),
        ("horror ", "horror "),
        ("terror ", "terror "),
        ("current ", "current "),
        ("correct ", "correct "),
        ("borrow ", "borrow "),
        ("carry ", "carry "),
        ("marry ", "mary "), // buffer "mary" IN dict → keep collapsed
        ("sorry ", "sorry "),
        ("worry ", "worry "),
        // -ified suffix pattern
        ("verified ", "verified "),  // normal typing
        ("verrified ", "verified "), // double 'r' reverts mark
        // -ect suffix pattern (s applies sắc mark to e)
        ("respect ", "respect "),
        ("respect  ", "respect  "),
    ]);
}

#[test]
fn pattern9_double_ss_english_words() {
    // English 4-char words ending with -ss should restore to English
    // 's' is not a valid Vietnamese final consonant
    // Double 's' reverts tone mark, then auto-restore preserves double 's'
    telex_auto_restore(&[
        ("bass ", "bass "),  // bass - fish or music term
        ("basss ", "bass "), // triple s collapses to double
        ("boss ", "boss "),  // boss - employer
        ("fuss ", "fuss "),  // fuss - commotion
        ("joss ", "joss "),  // joss - Chinese idol
        ("kiss ", "kiss "),  // kiss - embrace
        ("less ", "less "),  // less - smaller amount
        ("loss ", "loss "),  // loss - opposite of gain
        ("mass ", "mass "),  // mass - quantity
        ("mess ", "mess "),  // mess - disorder
        ("miss ", "miss "),  // miss - fail to hit
        ("moss ", "moss "),  // moss - plant
        ("pass ", "pass "),  // pass - go by
        ("toss ", "toss "),  // toss - throw
    ]);
}

#[test]
fn pattern9_double_f_words() {
    // Double 'f' (huyền mark) - need vowel before ff for revert to happen
    telex_auto_restore(&[
        // Words where ff reverts and buffer matches suffix pattern
        ("soffa ", "sofa "), // "soa" buffer + "-fa" → use buffer "sofa"
        // Words that preserve double f (no prefix/suffix match)
        ("staff ", "staff "),
        ("stuff ", "stuff "),
        ("cliff ", "cliff "),
        ("stiff ", "stiff "),
        ("effect ", "effect "),
        ("effort ", "effort "),
        ("offer ", "offer "),
        ("suffer ", "suffer "),
        ("differ ", "differ "),
        ("buffer ", "buffer "),
        ("bufffer ", "buffer "), // triple 'f' → collapse to double 'f'
        ("affair ", "affair "),
        ("afffair ", "affair "), // triple 'f' in middle → collapse to double 'f'
        ("afford ", "afford "),
        ("affford ", "afford "), // triple 'f' → collapse to double 'f'
        ("offend ", "offend "),
        ("offfend ", "offend "), // triple 'f' → collapse to double 'f'
        // Triple 'r' cases
        ("error ", "error "),
        ("errror ", "error "), // triple 'r' → collapse to double 'r'
        ("mirror ", "mirror "),
        ("mirrror ", "mirror "), // triple 'r' → collapse to double 'r'
        ("sorry ", "sorry "),
        ("sorrry ", "sorry "), // triple 'r' → collapse to double 'r'
        // Triple 's' cases (middle of word)
        ("issue ", "issue "),
        ("isssue ", "issue "), // triple 's' → collapse to double 's'
        ("assess ", "assess "),
        ("assssess ", "assess "), // triple 's' in middle → collapse
    ]);
}

// =============================================================================
// ETHNIC MINORITY LANGUAGE PLACE NAMES (ISSUE #134)
// Đắk Lắk, Đắk Nông should stay Vietnamese - NOT auto-restored
// =============================================================================

#[test]
fn ethnic_minority_place_names_not_restored() {
    // Vietnamese province names with breve patterns
    // These are valid Vietnamese and should NOT be auto-restored
    telex_auto_restore(&[
        ("ddawks ", "đắk "),            // đắk - lowercase
        ("Ddawks ", "Đắk "),            // Đắk - capitalized
        ("DDawks ", "Đắk "),            // Đắk - DD pattern
        ("lawks ", "lắk "),             // lắk - lowercase
        ("Lawks ", "Lắk "),             // Lắk - capitalized
        ("Ddawks Lawks ", "Đắk Lắk "),  // Đắk Lắk - full province name
        ("Ddawks Noong ", "Đắk Nông "), // Đắk Nông province
        // Kr initial for ethnic minority words (Krông Búk district)
        ("Kroong ", "Krông "),          // Krông - Kr initial + ô
        ("Busk ", "Búk "),              // Búk - B + ú + k
        ("Kroong Busk ", "Krông Búk "), // Krông Búk - full district name
        // Other breve + final consonant patterns
        ("bawts ", "bắt "),   // bắt - catch
        ("mawts ", "mắt "),   // mắt - eye
        ("nawngs ", "nắng "), // nắng - sunny
    ]);
}

// =============================================================================
// ISSUE #26 / #142 - UNFIXED BUGS (TEST CASES FOR PENDING FIXES)
// =============================================================================

/// Issue #26 - @jackblk: "ủa" with pattern "ura" becomes "ura" instead of "ủa"
/// Vietnamese word without initial consonant - valid interjection
#[test]
fn issue26_ua_with_hook_tone_before_vowel() {
    telex_auto_restore(&[
        ("ura ", "ủa "), // u + r(hỏi) + a → ủa (tone before second vowel)
        ("uar ", "ủa "), // u + a + r → ủa (standard order)
        ("uxa ", "ũa "), // u + x(ngã) + a → ũa
        ("uax ", "ũa "), // u + a + x → ũa (standard order)
        // Similar pattern: a + r + o → ảo (valid Vietnamese)
        ("aro ", "ảo "), // a + r(hỏi) + o → ảo (valid VN, NOT restored)
        ("aor ", "ảo "), // a + o + r → ảo (standard order)
        // Double 'r' reverts hỏi, then restore to raw
        ("arro ", "aro "), // a + r + r(revert) + o → aro (restore to raw)
        // Double 's' at start, more letters after - should collapse
        ("ussers ", "users "), // u + s + s(revert) + e + r + s → users
    ]);
}

/// Issue #26 - @npkhang99: "chịu" with pattern "chiuj" becomes "chiju"
/// Vietnamese diphthong I+U with tone modifier
#[test]
fn issue26_chiu_with_tone_before_final_vowel() {
    telex_auto_restore(&[
        ("chiuj ", "chịu "), // standard order: ch + i + u + j(nặng)
        ("chiju ", "chịu "), // alternative: ch + i + j + u (tone before u)
        ("liuj ", "lịu "),   // l + i + u + j → lịu
        ("niuj ", "nịu "),   // n + i + u + j → nịu
    ]);
}

/// Issue #26 - @jackblk: "thuỷ" with pattern "thury" gets restored incorrectly
/// Vietnamese diphthong U+Y with tone modifier
#[test]
fn issue26_thuy_with_hook_before_y() {
    telex_auto_restore(&[
        ("thuyr ", "thuỷ "), // standard: th + u + y + r(hỏi)
        ("thury ", "thuỷ "), // alternative: th + u + r + y (tone before y)
        ("quyr ", "quỷ "),   // qu + y + r → quỷ
        ("qury ", "quỷ "),   // qu + r + y → quỷ (alternative)
    ]);
}

/// Issue #142: "sims" becomes "simss" (extra 's' added)
/// English word should be restored as-is on space
/// Vietnamese-first: "dím" and "sém" are valid Vietnamese words (in 22k dictionary)
#[test]
fn issue142_sims_extra_s() {
    telex_auto_restore(&[
        ("sims ", "sims "), // should stay "sims" (sím not in VN dictionary)
        ("rims ", "rims "), // rims (rím not in VN dictionary)
        ("dims ", "dím "),  // dím IS in VN dictionary (to press down)
        ("gems ", "gems "), // gems (gém not in VN dictionary)
        ("hems ", "hems "), // hems (hém not in VN dictionary)
        ("sems ", "sém "),  // sém IS in VN dictionary (scorched)
    ]);
}

/// Test: Vowel-triggered circumflex stays Vietnamese when buffer is valid
/// Unified logic: only restore when buffer is INVALID Vietnamese
/// Words with circumflex + non-stop consonant (m, n, ng, nh) are real Vietnamese
/// Words with circumflex + stop consonant (t, c, p) WITHOUT mark are not real Vietnamese
#[test]
fn vowel_triggered_circumflex_stays_vietnamese() {
    telex_auto_restore(&[
        // Non-stop consonant finals (m, n, ng, nh) → real Vietnamese words
        ("homo ", "hôm "), // h+o+m+o → hôm (valid VI: yesterday)
        ("mama ", "mâm "), // m+a+m+a → mâm (valid VI: tray)
        // Stop consonant finals (t, c, p) without mark → NOT real Vietnamese
        ("toto ", "toto "), // t+o+t+o → tôt (no mark, restore to English)
        ("papa ", "papa "), // p+a+p+a → pâp (no mark, restore to English)
    ]);
}

// =============================================================================
// ISSUE #151 - "mưa" (rain) should NOT be auto-restored
// Vietnamese word with horn on 'u' pattern
// =============================================================================

#[test]
fn issue151_mua_horn_not_restored() {
    // "mưa" (rain) is a common Vietnamese word - should NOT be auto-restored
    // Pattern: m + u + w(horn on u) + a → mưa
    // Or: m + u + a + w(horn on u) → mưa
    telex_auto_restore(&[
        ("muwa ", "mưa "), // m + u + w + a → mưa (NOT "mwa")
        ("muaw ", "mưa "), // m + u + a + w → mưa (NOT "mwa")
        ("mwa ", "mưa "),  // Issue #151: shorthand m + w + a → mưa (NOT "mwa")
        // Similar patterns with other initials
        ("chuwa ", "chưa "), // chưa (not yet)
        ("chuaw ", "chưa "), // chưa (alternative typing)
        ("cwa ", "cưa "),    // Issue #151: shorthand c + w + a → cưa
        ("thuwa ", "thưa "), // thưa (dear/sparse)
        ("thuaw ", "thưa "), // thưa (alternative)
        ("luwa ", "lưa "),   // lưa (somewhat valid pattern)
        ("luaw ", "lưa "),   // lưa (alternative)
        ("lwa ", "lưa "),    // Issue #151: shorthand l + w + a → lưa
        // With marks (tones)
        ("muwas ", "mứa "), // mứa (sắc)
        ("muwaf ", "mừa "), // mừa (huyền)
        ("muwar ", "mửa "), // mửa (hỏi) - vomit
        ("muwax ", "mữa "), // mữa (ngã)
        ("muwaj ", "mựa "), // mựa (nặng)
    ]);
}

// =============================================================================
// Vietnamese "êu" diphthong patterns - should NOT be auto-restored
// Pattern: E + tone modifier + U + E → delayed circumflex creates êu
// =============================================================================

#[test]
fn vietnamese_eu_diphthong_not_restored() {
    // "nếu" (if), "kêu" (to call), "nêu" (to state) are valid Vietnamese words
    // Pattern: consonant + e + tone + u + e → delayed circumflex on e → êu diphthong
    telex_auto_restore(&[
        // nếu (if) - common Vietnamese word
        ("nesue ", "nếu "), // n + e + s(sắc) + u + e → nếu
        ("neeus ", "nếu "), // n + e + e(circumflex) + u + s(sắc) → nếu (standard)
        // kêu (to call/cry)
        ("kesue ", "kếu "), // k + e + s + u + e → kếu
        ("keeu ", "kêu "),  // k + e + e + u → kêu (no tone)
        // Similar patterns with valid Vietnamese initials
        ("lesue ", "lếu "), // l + e + s + u + e → lếu
        ("tesue ", "tếu "), // t + e + s + u + e → tếu
        ("mesue ", "mếu "), // m + e + s + u + e → mếu
        // Issue #183: Flexible order - mark after second vowel trigger
        // Pattern: C + V1 + V2 + V1 + mark → circumflex on V1 + mark
        ("neues ", "nếu "), // n + e + u + e + s(sắc) → nếu (mark after vowel trigger)
        ("neuef ", "nều "), // n + e + u + e + f(huyền) → nều
        ("keues ", "kếu "), // k + e + u + e + s → kếu
    ]);
}

// =============================================================================
// Vietnamese standalone vowels with circumflex + tone - should NOT be auto-restored
// Pattern: V + tone_modifier + V (same vowel) → circumflex + tone on single vowel
// Example: OFO → ồ (o with circumflex + huyền)
// =============================================================================

#[test]
fn standalone_vowel_circumflex_with_tone() {
    // When typing V + modifier + same_V, the circumflex is applied to create
    // a single vowel with both circumflex (from doubling) and tone (from modifier).
    // These are valid Vietnamese exclamations/interjections and should NOT be restored.
    telex_auto_restore(&[
        // Circumflex vowels with huyền (f)
        ("ofo ", "ồ "), // ồ - exclamation "oh" (surprised)
        ("efe ", "ề "), // ề - (less common standalone)
        ("afa ", "ầ "), // ầ - exclamation sound
        // Circumflex vowels with sắc (s)
        ("oso ", "ố "), // ố - exclamation
        ("ese ", "ế "), // ế - (contextual)
        ("asa ", "ấ "), // ấ - exclamation
        // Circumflex vowels with hỏi (r)
        ("oro ", "ổ "), // ổ - (contextual)
        ("ere ", "ể "), // ể - (contextual)
        ("ara ", "ẩ "), // ẩ - (contextual)
        // Circumflex vowels with ngã (x)
        ("oxo ", "ỗ "), // ỗ - (contextual)
        ("exe ", "ễ "), // ễ - (contextual)
        ("axa ", "ẫ "), // ẫ - (contextual)
        // Circumflex vowels with nặng (j)
        ("ojo ", "ộ "), // ộ - (contextual)
        ("eje ", "ệ "), // ệ - (contextual)
        ("aja ", "ậ "), // ậ - (contextual)
    ]);
}

// =============================================================================
// PATTERN 10: D+E PATTERN (describe, design, desk...)
// English words starting with "de" + 's' modifier are auto-restored.
// Vietnamese word "dép" (slippers) works correctly.
// =============================================================================

#[test]
fn pattern10_de_s_english_words() {
    // English words with D+E pattern are auto-restored when space is typed
    telex_auto_restore(&[
        ("describe ", "describe "),
        ("design ", "design "),
        ("desk ", "desk "),
        ("desktop ", "desktop "),
        ("destroy ", "destroy "),
        ("desperate ", "desperate "),
        ("despite ", "despite "),
        // NOTE: "dessert" becomes "desert" due to double 's' reverting the mark
        ("destination ", "destination "),
        ("detail ", "detail "),
        ("detect ", "detect "),
        ("develop ", "develop "),
    ]);
}

#[test]
fn pattern10_de_s_vietnamese_words() {
    // Vietnamese words with D+E pattern should NOT be auto-restored
    // "dép" (slippers) is valid Vietnamese
    telex_auto_restore(&[
        // Without space - Vietnamese transform stays
        ("desp", "dép"), // dép - slippers (no space)
        ("desm", "dém"), // dém (no space)
        ("desn", "dén"), // dén (no space)
        ("dest", "dét"), // dét (no space)
        ("desc", "déc"), // déc (no space)
        // With space - still Vietnamese (valid structure)
        ("desp ", "dép "), // dép - slippers
        ("desm ", "dém "), // dém - valid Vietnamese structure
        ("desn ", "dén "), // dén - valid Vietnamese structure
        ("dest ", "dét "), // dét - valid Vietnamese structure
        ("desc ", "déc "), // déc - valid Vietnamese structure (though uncommon)
    ]);
}

// =============================================================================
// PATTERN 11: -ING + TONE MARK = INVALID VIETNAMESE
// Vietnamese uses -inh (tính, kính), NOT -ing with tone marks.
// Words like "thíng", "kíng" are invalid → should auto-restore.
// =============================================================================

#[test]
fn pattern11_ing_with_tone_invalid() {
    // -ing + tone mark is NOT valid Vietnamese rhyme
    // Vietnamese uses -inh for this sound
    telex_auto_restore(&[
        // -ings English plural pattern
        ("things ", "things "), // thíng invalid → restore
        ("kings ", "kings "),   // kíng invalid → restore
        ("rings ", "rings "),   // ríng invalid → restore
        ("sings ", "sings "),   // síng invalid → restore
        ("wings ", "wings "),   // wíng invalid (also W invalid initial)
        ("brings ", "brings "), // bríng invalid (also br- cluster)
        // -ing singular (no 's' at end, but 's' was tone modifier)
        ("thing ", "thing "), // th + i + n + g + s(modifier) → thíng → restore
        ("king ", "king "),   // k + i + n + g + s → kíng → restore
        ("ring ", "ring "),   // r + i + n + g + s → ríng → restore
        ("sing ", "sing "),   // s + i + n + g + s → síng → restore
    ]);
}

#[test]
fn pattern11_inh_valid_vietnamese() {
    // -inh WITH tone marks IS valid Vietnamese
    // These should NOT be restored
    telex_auto_restore(&[
        ("tinhs ", "tính "),   // tính (to calculate) - valid
        ("kinhs ", "kính "),   // kính (glass/respect) - valid
        ("minhs ", "mính "),   // mính - valid structure
        ("linhs ", "lính "),   // lính (soldier) - valid
        ("chinhs ", "chính "), // chính (main/correct) - valid
        // Single vowel with tone - valid Vietnamese
        ("ys ", "ý "), // ý (idea/opinion) - valid
    ]);
}

#[test]
fn pattern11_ing_immediate_output() {
    // -ing + tone mark should output correct result IMMEDIATELY (no space needed)
    // Engine should detect invalid VN and NOT apply tone mark
    telex(&[
        ("things", "things"), // th + i + n + g + s → should stay "things", not "thíng"
        ("kings", "kings"),   // k + i + n + g + s → should stay "kings"
        ("rings", "rings"),   // r + i + n + g + s → should stay "rings"
        ("sings", "sings"),   // s + i + n + g + s → should stay "sings"
                              // Note: "wings" and "brings" have other invalid patterns (w initial, br cluster)
                              // so they may be handled by other validation rules
    ]);
}

#[test]
fn pattern11b_v1v2v1_immediate_output() {
    // V1-V2-V1 vowel pattern should NOT trigger circumflex
    // Example: "queue" = e-u-e, third 'e' should NOT circumflex first 'e'
    telex(&[
        ("queue", "queue"), // qu + e + u + e → should stay "queue", not "quêu"
    ]);
}

// =============================================================================
// PATTERN 12: C + CIRCUMFLEX VOWEL (from ee/oo) + NO FINAL = INVALID
// When double vowel creates circumflex but no final consonant,
// and the result is not a common Vietnamese word → restore.
// Examples: "see" → "sê" (invalid), "fee" → "fê" (F invalid anyway)
// Exceptions: "bê" (calf), "mê" (obsessed) - real Vietnamese words
// =============================================================================

#[test]
fn pattern12_circumflex_no_final_invalid() {
    // Only F initial is invalid Vietnamese → restore to English
    // Vietnamese-first: keep all other C+ê patterns as Vietnamese
    telex_auto_restore(&[
        // "fee" → "fê" - F is invalid Vietnamese initial → restore
        ("fee ", "fee "),
    ]);
}

#[test]
fn pattern12_circumflex_no_final_valid_vietnamese() {
    // Vietnamese-first: C + ê/ô are valid Vietnamese → keep Vietnamese
    // Exceptions that restore to English:
    // - F initial is invalid (fee → fee)
    // - G before E is invalid (gee → gee, should use GH)
    telex_auto_restore(&[
        ("bee ", "bê "),   // bê (calf) - valid Vietnamese
        ("mee ", "mê "),   // mê (obsessed) - valid Vietnamese
        ("lee ", "lê "),   // lê (pear) - valid Vietnamese
        ("ddee ", "đê "),  // đê (dike) - valid Vietnamese
        ("khee ", "khê "), // khê (hoarse) - valid Vietnamese
        ("see ", "sê "),   // sê - valid Vietnamese (Vietnamese-first)
        ("tee ", "tê "),   // tê (numb) - valid Vietnamese
        ("pee ", "pê "),   // pê - valid Vietnamese (Vietnamese-first)
        ("ghee ", "ghê "), // ghê (terrifying) - valid Vietnamese (GH before E)
        ("gee ", "gee "),  // gê - invalid (G before E), restore to English
    ]);
}

// =============================================================================
// PATTERN 13: DOUBLE-F PRESERVATION (off, offline, offensive)
// When user types double 'f', preserve both 'f's in output.
// Current bug: "off" → "of", "offline" → "ofline" (loses one 'f')
// =============================================================================

#[test]
fn pattern13_double_f_in_middle_preserve() {
    // Double 'f' in the MIDDLE of words should trigger restore
    // Note: "off" alone is skipped - keeps current behavior (buffer "o")
    telex_auto_restore(&[
        // Words starting with off- (ff in middle followed by more letters)
        ("offline ", "offline "),
        ("offset ", "offset "),
        ("offend ", "offend "),
        ("offer ", "offer "),
        ("office ", "office "),
        ("officer ", "officer "),
        ("official ", "official "),
        ("offshore ", "offshore "),
        // Words with ff in middle
        ("effect ", "effect "),
        ("effort ", "effort "),
        ("afford ", "afford "),
        ("differ ", "differ "),
        ("suffer ", "suffer "),
        ("buffer ", "buffer "),
        ("coffee ", "coffee "),
        ("traffic ", "traffic "),
        ("stuff ", "stuff "),
        ("staff ", "staff "),
    ]);
}

// =============================================================================
// PATTERN 14: SINGLE VOWEL WITH TONES - VALID VIETNAMESE INTERJECTIONS
// Short vowel patterns like "of" → "ò", "if" → "ì" are SKIPPED (keep current behavior)
// Common interjections like "à", "ồ" should NOT restore
// =============================================================================

#[test]
fn pattern14_single_vowel_valid_vietnamese() {
    // Single vowel + tone that ARE valid Vietnamese interjections
    // These should NOT restore
    telex_auto_restore(&[
        // Common Vietnamese interjections - keep as Vietnamese
        ("af ", "à "), // à (ah, I see) - very common
        ("ax ", "ã "), // ã - interjection
        ("ofo ", "ồ "), // ồ (oh!) - common exclamation (o + f + o = circumflex + huyền)
                       // Note: "of" → "ò" and "if" → "ì" are skipped
                       // We keep current behavior for these short patterns
    ]);
}

// =============================================================================
// PATTERN 15: DELAYED CIRCUMFLEX WITH TONE BEFORE VOWEL
// Pattern: C + V1 + E + U + E + tone → C + iêu + tone (valid Vietnamese)
// Tone modifier (r/s/f/x/j) comes BEFORE the second 'e' that triggers circumflex
// Example: "hieuer" = h + i + e + u + e(hỏi) + r → hiểu
// =============================================================================

#[test]
fn pattern15_delayed_circumflex_with_tone() {
    // When typing Vietnamese words with delayed circumflex pattern,
    // the tone modifier comes before the second vowel that triggers circumflex.
    // Pattern: C + ie + u + e + tone → C + iêu + tone (with ê getting the tone)
    telex_auto_restore(&[
        // hiểu (to understand) - very common Vietnamese word
        // h + i + e + u + e(circumflex) + r(hỏi) → hiểu
        ("hieuer ", "hiểu "),
        // viết (to write) - very common Vietnamese word
        // v + i + e + t + e(circumflex) + s(sắc) → viết
        ("vietes ", "viết "),
        // Similar patterns with other tones
        ("hieues ", "hiếu "), // hiếu (filial piety) - sắc
        ("hieuef ", "hiều "), // hiều - huyền
        ("hieuex ", "hiễu "), // hiễu - ngã
        ("hieuej ", "hiệu "), // hiệu (effect/shop) - nặng
    ]);
}

// =============================================================================
// PATTERN 15b: IÊU TRIPHTHONG WITH DIFFERENT INITIALS
// Common Vietnamese words with iêu triphthong pattern
// =============================================================================

#[test]
fn pattern15b_ieu_triphthong_various_initials() {
    telex_auto_restore(&[
        // Different initials with iêu triphthong
        ("lieues ", "liếu "),   // liếu (willow)
        ("dieuer ", "diểu "),   // diểu
        ("kieues ", "kiếu "),   // kiếu
        ("nieues ", "niếu "),   // niếu
        ("tieues ", "tiếu "),   // tiếu (laugh - Sino-Viet)
        ("mieues ", "miếu "),   // miếu (temple)
        ("bieues ", "biếu "),   // biếu (to give as gift)
        ("chieues ", "chiếu "), // chiếu (mat/to shine)
        ("nhieues ", "nhiếu "), // nhiếu
        ("trieues ", "triếu "), // triều (dynasty) - note: triếu variant
    ]);
}

// =============================================================================
// PATTERN 15c: IÊ DIPHTHONG + CONSONANT FINALS
// Vietnamese words: viết, tiết, miếng, điểm, etc.
// =============================================================================

#[test]
fn pattern15c_ie_diphthong_with_finals() {
    telex_auto_restore(&[
        // iê + t final (delayed circumflex: e after t)
        ("vietes ", "viết "),   // viết (to write)
        ("tietes ", "tiết "),   // tiết (section/blood)
        ("kietes ", "kiết "),   // kiết (dysentery)
        ("nietes ", "niết "),   // niết (nirvana)
        ("thietes ", "thiết "), // thiết (iron/essential)
        // iê + c final (double e for circumflex since no delayed trigger after c)
        ("vieecj ", "việc "), // việc (work/job)
        ("tieecj ", "tiệc "), // tiệc (party/feast)
        // iê + m final (double e for circumflex)
        ("ddieemr ", "điểm "), // điểm (point) - standard dd
        ("tieemf ", "tiềm "),  // tiềm (latent)
        ("kieemr ", "kiểm "),  // kiểm (to check)
        // iê + n final (double e for circumflex)
        ("tieens ", "tiến "),  // tiến (to advance)
        ("vieenj ", "viện "),  // viện (institute)
        ("ddieenj ", "điện "), // điện - standard dd
        // iê + p final (double e for circumflex)
        ("tieeps ", "tiếp "),   // tiếp (to continue/receive)
        ("nhieeps ", "nhiếp "), // nhiếp (photography)
        // iê + ng final (double e for circumflex)
        ("tieengs ", "tiếng "), // tiếng (sound/language) - double e
        ("mieengs ", "miếng "), // miếng (piece) - double e
    ]);
}

// =============================================================================
// PATTERN 15d: UÊ DIPHTHONG PATTERNS
// Vietnamese words with uê diphthong
// =============================================================================

#[test]
fn pattern15d_ue_diphthong_patterns() {
    telex_auto_restore(&[
        // uê standalone/with tones (double e for circumflex)
        ("tueef ", "tuề "), // tuề
        ("hueef ", "huề "), // huề (even/draw)
        ("xueef ", "xuề "), // xuề
        // uê + consonant finals (double e for circumflex)
        ("tueechs ", "tuếch "),   // tuếch (wide open)
        ("thueechs ", "thuếch "), // thuếch
        // uê + n final (double e for circumflex)
        ("thueens ", "thuến "), // valid pattern
        ("queens ", "quến "),   // quến (to attract)
        ("quyeens ", "quyến "), // quyến (to attract) - quy pattern
    ]);
}

// =============================================================================
// PATTERN 15e: YÊU TRIPHTHONG (standalone Y initial)
// Vietnamese words starting with Y + êu
// =============================================================================

#[test]
fn pattern15e_yeu_triphthong() {
    telex_auto_restore(&[
        // yêu patterns - both double e and delayed circumflex are valid
        // Double e method (standard)
        ("yeeu ", "yêu "),  // yêu (to love)
        ("yeeus ", "yếu "), // yếu (weak)
        ("yeeuf ", "yều "), // yều
        ("yeeur ", "yểu "), // yểu
        // Delayed circumflex method (e-u-e pattern)
        ("yeue ", "yêu "),  // yêu (to love)
        ("yeues ", "yếu "), // yếu (weak)
        ("yeuef ", "yều "), // yều
        ("yeuer ", "yểu "), // yểu
    ]);
}

// =============================================================================
// PATTERN 15f: UÔ DIPHTHONG PATTERNS (V2_CIRCUMFLEX_REQUIRED)
// Vietnamese words with uô diphthong
// =============================================================================

#[test]
fn pattern15f_uo_diphthong_patterns() {
    telex_auto_restore(&[
        // uô + consonant finals (double o for circumflex)
        ("cuoocs ", "cuốc "),   // cuốc (hoe)
        ("chuoots ", "chuốt "), // chuốt (to sharpen)
        ("muoons ", "muốn "),   // muốn (to want)
        ("tuooir ", "tuổi "),   // tuổi (age)
        ("buooir ", "buổi "),   // buổi (session/half-day)
    ]);
}

// =============================================================================
// PATTERN 15g: COMMON VIETNAMESE WORDS - COMPREHENSIVE TEST
// Real-world Vietnamese words with delayed circumflex
// =============================================================================

#[test]
fn pattern15g_common_vietnamese_words() {
    telex_auto_restore(&[
        // Education/learning - iêu triphthong (delayed circumflex works)
        ("hieuer ", "hiểu "),  // hiểu (understand)
        ("vietes ", "viết "),  // viết (write) - delayed circumflex after t
        ("ddieemr ", "điểm "), // điểm (point/score) - double e
        ("kieemr ", "kiểm "),  // kiểm (to check) - double e
        // Work/business - double e for circumflex
        ("vieecj ", "việc "), // việc (work)
        ("tieecj ", "tiệc "), // tiệc (party)
        ("tieeps ", "tiếp "), // tiếp (continue)
        // Daily life - double e for circumflex
        ("tieengs ", "tiếng "), // tiếng (sound)
        ("mieengs ", "miếng "), // miếng (piece)
        ("chieues ", "chiếu "), // chiếu (mat) - iêu triphthong
        ("bieues ", "biếu "),   // biếu (gift) - iêu triphthong
        // Technology - double e for circumflex
        ("ddieenj ", "điện "), // điện (electricity)
    ]);
}

// =============================================================================
// OE DIPHTHONG TYPING ORDER CONSISTENCY
// Both "xoef" and "xofe" should produce same result (using modern tone placement)
// Modern: tone on second vowel (xoè), Traditional: tone on first vowel (xòe)
// =============================================================================

#[test]
fn oe_diphthong_typing_orders() {
    // Engine uses modern tone placement by default (tone on second vowel for OE)
    telex_auto_restore(&[
        // xòe (modern: xoè) - both typing orders should work
        ("xoef ", "xoè "), // tone after 'e' - standard typing
        ("xofe ", "xoè "), // tone after 'o' - alternative typing order
        // hòe (modern: hoè)
        ("hoef ", "hoè "),
        ("hofe ", "hoè "),
        // loè
        ("loef ", "loè "),
        // tóe (modern: toé)
        ("toes ", "toé "),
        ("tose ", "toé "),
    ]);
}
