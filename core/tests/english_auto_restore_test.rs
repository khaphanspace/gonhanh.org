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
use common::telex_auto_restore;

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
