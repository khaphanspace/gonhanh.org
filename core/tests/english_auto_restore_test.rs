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
        // NOTE: s + consonant patterns like "test", "rest", "best" form valid Vietnamese words
        // (tét, rét, bét, etc.) so they are NOT auto-restored.
        // Users who want English spelling should use ESC or raw mode prefix.
        // This is a design decision: preserve Vietnamese words over restoring English.
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
    telex_auto_restore(&[("keep ", "keep ")]);
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
    telex_auto_restore(&[("use ", "use "), ("user ", "user ")]);
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
        ("wonder ", "wonder "),
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
        // Short words: consonant + vowel + modifier (no final vowel)
        ("per ", "per "),    // p + e + r → pẻ (invalid) → restore "per"
        ("thiss ", "this "), // t + h + i + s + s → double s reverts → buffer "this" (4 chars)
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
        // Double 'r' without matching prefix (need vowel before rr)
        ("error ", "error "),
        ("mirror ", "mirror "),
        ("horror ", "horror "),
        ("terror ", "terror "),
        ("current ", "current "),
        ("correct ", "correct "),
        ("borrow ", "borrow "),
        ("carry ", "carry "),
        ("marry ", "marry "),
        ("sorry ", "sorry "),
        ("worry ", "worry "),
    ]);
}

#[test]
fn pattern9_double_mark_4char_keeps_reverted() {
    // 4-char words with double mark: keep reverted result
    // User typed double modifier to explicitly revert the mark
    telex_auto_restore(&[
        ("bass ", "bas "),
        ("boss ", "bos "),
        ("less ", "les "),
        ("loss ", "los "),
        ("mass ", "mas "),
        ("mess ", "mes "),
        ("miss ", "mis "),
        ("pass ", "pas "),
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
        ("affair ", "affair "),
        ("affair ", "affair "),
        ("afford ", "afford "),
        ("offend ", "offend "),
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
#[test]
fn issue142_sims_extra_s() {
    telex_auto_restore(&[
        ("sims ", "sims "), // should stay "sims", not "simss" or "sím"
        ("rims ", "rims "), // rims - similar pattern
        ("dims ", "dims "), // dims - similar pattern
        ("gems ", "gems "), // gems - similar pattern
        ("hems ", "hems "), // hems - similar pattern
    ]);
}

/// Test: "homo" becomes "hôm" (second 'o' consumed for circumflex)
/// "hôm" IS valid Vietnamese (yesterday) but user may intend English "homo"
/// This is an edge case - "hôm" is a real word, so debatable whether to restore
#[test]
fn edge_case_homo() {
    // For now, don't restore since "hôm" is valid Vietnamese
    // User can use ESC or \homo to force English
    telex_auto_restore(&[
        ("homo ", "hôm "), // keeps as Vietnamese "hôm" (yesterday)
    ]);
}
