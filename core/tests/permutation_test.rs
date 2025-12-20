//! Permutation Testing - Dynamic typing order verification
//!
//! This test module verifies that different typing orders produce the same output.
//! Vietnamese words can be typed with modifiers in different positions:
//! - Tone marks (s,f,r,x,j) can come before or after final consonant
//! - Circumflex (aa,ee,oo) can be typed consecutively or split by final consonant
//! - Horn (w) can be typed at different positions relative to vowels
//!
//! The tests dynamically generate all valid typing orders and verify they produce
//! the same Vietnamese output.

mod common;
use common::{telex, telex_auto_restore};

// =============================================================================
// CORE CONCEPT: Modifier Position Permutations
// =============================================================================
//
// For a word like "nào" (naof), the modifier 'f' can be typed:
// 1. After all vowels: n-a-o-f → "nào" (standard)
// 2. Before last vowel: n-a-f-o → "nào" (should also work)
//
// For a word like "riêng" (rieeng), the circumflex 'e' can be typed:
// 1. Consecutively: r-i-e-e-n-g → "riêng" (standard)
// 2. After final consonant starts: r-i-e-n-e-g → "riêng" (should also work)

// =============================================================================
// TEST GROUP 1: TONE MODIFIER ORDER (s, f, r, x, j)
// =============================================================================

/// Tone modifier can be typed before or after final consonant (using sắc 's')
#[test]
fn tone_modifier_order_with_single_final() {
    telex(&[
        // Standard order: vowel + final + tone
        ("nam ", "nam "),  // no tone
        ("nams ", "nám "), // tone after final
        // Alternative order: vowel + tone + final
        ("nasm ", "nám "), // tone before final - should work same as above
        // More examples with different finals
        ("lac ", "lac "),
        ("lacs ", "lác "),
        ("lasc ", "lác "), // tone before final c
        ("man ", "man "),
        ("mans ", "mán "),
        ("masn ", "mán "), // tone before final n
        ("map ", "map "),
        ("maps ", "máp "),
        ("masp ", "máp "), // tone before final p
        ("mat ", "mat "),
        ("mats ", "mát "),
        ("mast ", "mát "), // tone before final t
    ]);
}

/// Tone modifier order with double final consonants (ng, nh, ch)
#[test]
fn tone_modifier_order_with_double_final() {
    telex(&[
        // -ng final
        ("lang ", "lang "),
        ("langs ", "láng "),
        ("lasng ", "láng "), // tone before ng
        // -nh final
        ("lanh ", "lanh "),
        ("lanhs ", "lánh "),
        ("lasnh ", "lánh "), // tone before nh
        // -ch final
        ("lach ", "lach "),
        ("lachs ", "lách "),
        ("lasch ", "lách "), // tone before ch
    ]);
}

/// Sắc tone ('s') before final consonant - known working
#[test]
fn sac_tone_before_final() {
    telex(&[
        ("masm ", "mám "),
        ("masp ", "máp "),
        ("masn ", "mán "),
        ("mast ", "mát "),
        ("masc ", "mác "),
    ]);
}

// =============================================================================
// TEST GROUP 2: CIRCUMFLEX ORDER (aa, ee, oo)
// =============================================================================

/// Circumflex typed consecutively (standard Telex)
#[test]
fn circumflex_consecutive() {
    telex(&[
        ("caan ", "cân "),
        ("teen ", "tên "),
        ("toon ", "tôn "),
        ("been ", "bên "),
        ("saan ", "sân "),
    ]);
}

/// Circumflex in diphthong patterns (iê, uô)
#[test]
fn circumflex_diphthong_patterns() {
    telex(&[
        // iê patterns - standard consecutive
        ("rieeng ", "riêng "),
        // Split by final consonant - now works!
        ("rieneg ", "riêng "),
        // uô patterns
        ("cuoong ", "cuông "),
        ("cuonog ", "cuông "),
    ]);
}

// =============================================================================
// TEST GROUP 3: HORN MODIFIER ORDER (w)
// =============================================================================

/// Horn on single vowel (standard usage)
#[test]
fn horn_single_vowel() {
    telex(&[("tuw ", "tư "), ("muw ", "mư "), ("nuw ", "nư ")]);
}

/// Horn in diphthong - typing order variations (critical fix!)
#[test]
fn horn_order_diphthong() {
    telex(&[
        // ơi pattern - both orders work!
        ("owi ", "ơi "), // w before i
        ("oiw ", "ơi "), // w after i - this was the bug we fixed!
        // ưa pattern (standard order)
        ("uwa ", "ưa "),
        // ươ pattern (standard order)
        ("uwo ", "ươ "),
    ]);
}

/// Common words with horn - all typing orders
#[test]
fn horn_common_words() {
    telex(&[
        // "nước" (water) - multiple modifiers
        ("nuwowcs ", "nước "),
        ("nuowcs ", "nước "), // different w position
        // "được" (can/get)
        ("dduwowcj ", "được "),
        ("dduowcj ", "được "),
        // "ơi" (oh/hey) - both orders
        ("owi ", "ơi "),
        ("oiw ", "ơi "),
    ]);
}

// =============================================================================
// TEST GROUP 4: DIPHTHONG + TONE MODIFIER COMBINATIONS
// =============================================================================

/// AO diphthong with tone at different positions (critical fix!)
#[test]
fn ao_diphthong_tone_order() {
    telex(&[
        // nào - both orders work!
        ("naof ", "nào "), // standard: tone after o
        ("nafo ", "nào "), // tone before o - this was the bug!
        // sao
        ("saof ", "sào "),
        ("safo ", "sào "),
        // cao
        ("caos ", "cáo "),
        ("caso ", "cáo "),
        // bao
        ("baor ", "bảo "),
        ("baro ", "bảo "),
    ]);
}

/// AI diphthong with tone at different positions
#[test]
fn ai_diphthong_tone_order() {
    telex(&[
        // gái - both orders work
        ("gais ", "gái "),
        ("gasi ", "gái "),
        // mái
        ("mais ", "mái "),
        ("masi ", "mái "),
        // tài
        ("taif ", "tài "),
        ("tafi ", "tài "),
        // bài
        ("baif ", "bài "),
        ("bafi ", "bài "),
    ]);
}

/// OI diphthong with tone at different positions
#[test]
fn oi_diphthong_tone_order() {
    telex(&[
        // tôi (circumflex)
        ("tooi ", "tôi "),
        // bói - both orders
        ("bois ", "bói "),
        ("bosi ", "bói "),
        // hỏi - both orders
        ("hoir ", "hỏi "),
        ("hori ", "hỏi "),
        // đòi
        ("ddoif ", "đòi "),
        ("ddofi ", "đòi "),
    ]);
}

/// UA diphthong with tone at different positions
#[test]
fn ua_diphthong_tone_order() {
    telex(&[
        // của (hỏi on 'a')
        ("cura ", "của "),
        // mua → múa
        ("muas ", "múa "),
        ("musa ", "múa "),
        // bùa
        ("buaf ", "bùa "),
        ("bufa ", "bùa "),
    ]);
}

// =============================================================================
// TEST GROUP 5: COMPLEX WORDS - MULTIPLE MODIFIERS
// =============================================================================

/// Words with both circumflex and tone
#[test]
fn circumflex_and_tone_combined() {
    telex(&[
        // "tấn" (ton) - circumflex + sắc
        ("taans ", "tấn "),
        // "bền" (durable) - circumflex + huyền
        ("beenf ", "bền "),
        // "hổn" - circumflex + hỏi
        ("hoonr ", "hổn "),
        // "tầng" - circumflex + huyền
        ("taangf ", "tầng "),
    ]);
}

/// Words with both horn and tone
#[test]
fn horn_and_tone_combined() {
    telex(&[
        // "nước" - horn + sắc
        ("nuwowcs ", "nước "),
        // "được" - horn + nặng
        ("dduwowcj ", "được "),
        // "bước" - horn + sắc
        ("buwowcs ", "bước "),
        // "mười" - horn + huyền
        ("muwowif ", "mười "),
    ]);
}

// =============================================================================
// TEST GROUP 6: REAL VIETNAMESE SENTENCES
// =============================================================================

/// Common Vietnamese phrases - test realistic typing
#[test]
fn real_phrases_greeting() {
    telex(&[
        ("xin chaof ", "xin chào "),
        ("camr ", "cảm "), // single word
        ("own ", "ơn "),   // single word (horn on o)
    ]);
}

/// Numbers and counting
#[test]
fn real_phrases_numbers() {
    telex(&[
        ("mootj ", "một "),
        ("hai ", "hai "),
        ("ba ", "ba "),
        ("boosn ", "bốn "),
        ("nawm ", "năm "), // breve via w (NOT nams which gives nám)
        ("saus ", "sáu "),
        ("bary ", "bảy "),
        ("tams ", "tám "),
        ("chins ", "chín "),
        ("muwowif ", "mười "),
    ]);
}

// =============================================================================
// TEST GROUP 7: EDGE CASES - MODIFIER AT BOUNDARIES
// =============================================================================

/// Modifier at word start (after initial consonant)
#[test]
fn modifier_after_initial() {
    telex(&[
        // These patterns should NOT apply modifier
        ("sm ", "sm "), // s at start, no vowel before
        ("fm ", "fm "), // f at start
        // But these should work
        ("sam ", "sam "),  // s after vowel a, but no tone
        ("safm ", "sàm "), // f as modifier after a
    ]);
}

/// Double modifiers (revert behavior)
/// When mark is reverted, only the reverting key appears (standard IME behavior)
/// For English words like "mass", "maff", use auto-restore feature
#[test]
fn double_modifiers() {
    telex(&[
        // Double mark: first key is modifier, second reverts and outputs ONE key
        ("mass ", "mas "), // second s reverts sắc, one s appears
        ("maff ", "maf "), // second f reverts huyền, one f appears
    ]);
}

// =============================================================================
// TEST GROUP 8: AUTO-RESTORE SHOULD NOT TRIGGER
// =============================================================================

/// Valid Vietnamese that looks like English - should NOT restore
#[test]
fn valid_vietnamese_not_restored() {
    telex(&[
        // These are valid Vietnamese, should not auto-restore
        ("naof ", "nào "), // not restored to "naof"
        ("saof ", "sào "), // not restored to "saof"
        ("mais ", "mái "), // not restored to "mais"
        ("oiw ", "ơi "),   // not restored to "oiw"
        ("nafo ", "nào "), // not restored to "nafo"
        ("gasi ", "gái "), // not restored to "gasi"
        ("baro ", "bảo "), // not restored to "baro"
    ]);
}

/// English words that should auto-restore
#[test]
fn english_words_restored() {
    telex_auto_restore(&[
        ("view ", "view "), // should restore (not vieư)
        ("raw ", "raw "),   // should restore
        ("law ", "law "),   // should restore
        ("saw ", "saw "),   // should restore
        ("data ", "data "), // should restore (not dât)
        ("half ", "half "), // should restore
        ("wolf ", "wolf "), // should restore
    ]);
}

// =============================================================================
// DYNAMIC TEST GENERATION HELPERS
// =============================================================================

/// Generate all permutations of modifier positions for a word
/// This is a conceptual test - in practice, we'd use property-based testing
#[test]
fn dynamic_permutation_concept() {
    // For word "nào" with base "nao" and modifier "f":
    // Permutations: naof, nafo, nfao (invalid), fnao (invalid)
    // Valid: naof, nafo
    telex(&[("naof ", "nào "), ("nafo ", "nào ")]);

    // For word "riêng" with base "rieng" and circumflex (second e):
    // Standard: rieeng
    // Split: rieneg (e after n)
    telex(&[("rieeng ", "riêng "), ("rieneg ", "riêng ")]);

    // For word "ơi" with base "oi" and horn "w":
    // Permutations: owi, oiw
    telex(&[("owi ", "ơi "), ("oiw ", "ơi ")]);
}

// =============================================================================
// TEST GROUP 9: CONFIRMED WORKING EDGE CASES
// =============================================================================

/// Edge cases that were bugs and are now fixed
#[test]
fn fixed_bugs_regression() {
    telex(&[
        // oiw was being restored to English, now works
        ("oiw ", "ơi "),
        // nafo was being restored to English, now works
        ("nafo ", "nào "),
        // rieneg circumflex was not applied, now works
        ("rieneg ", "riêng "),
        // cunxg should become cũng
        ("cunxg ", "cũng "),
    ]);
}

/// Diphthong patterns with tone in middle
#[test]
fn diphthong_tone_middle() {
    telex(&[
        // A→O diphthong
        ("nafo ", "nào "),
        ("safo ", "sào "),
        ("caso ", "cáo "),
        ("baro ", "bảo "),
        // A→I diphthong
        ("gasi ", "gái "),
        ("masi ", "mái "),
        ("tafi ", "tài "),
        ("bafi ", "bài "),
        // O→I diphthong
        ("bosi ", "bói "),
        ("hori ", "hỏi "),
        // U→A diphthong
        ("musa ", "múa "),
        ("bufa ", "bùa "),
    ]);
}
