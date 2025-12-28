//! Engine Tests - Syllable parsing, validation, and transformation

mod common;
use common::{telex, vni};
use gonhanh_core::engine::Engine;

// ============================================================
// SYLLABLE PARSING TESTS
// ============================================================

/// Test syllable parsing via engine behavior
/// These test Vietnamese syllable structure recognition

#[test]
fn syllable_simple_cv() {
    // Simple consonant + vowel
    telex(&[
        ("ba", "ba"),
        ("ca", "ca"),
        ("da", "da"),
        ("ma", "ma"),
        ("na", "na"),
    ]);
}

#[test]
fn syllable_cvc() {
    // Consonant + vowel + consonant
    telex(&[
        ("ban", "ban"),
        ("cam", "cam"),
        ("dat", "dat"),
        ("mac", "mac"),
        ("nap", "nap"),
    ]);
}

#[test]
fn syllable_double_initial() {
    // Double consonant initials
    telex(&[
        ("cha", "cha"),
        ("ghi", "ghi"),
        ("kha", "kha"),
        ("nga", "nga"),
        ("nha", "nha"),
        ("pha", "pha"),
        ("tha", "tha"),
        ("tra", "tra"),
    ]);
}

#[test]
fn syllable_triple_initial() {
    // Triple consonant initial (ngh)
    telex(&[("nghe", "nghe"), ("nghi", "nghi"), ("nghieng", "nghieng")]);
}

#[test]
fn syllable_gi_initial() {
    // gi + vowel = gi is initial
    telex(&[("gia", "gia"), ("giau", "giau"), ("gieo", "gieo")]);
}

#[test]
fn syllable_qu_initial() {
    // qu + vowel = qu is initial
    telex(&[("qua", "qua"), ("quan", "quan"), ("quoc", "quoc")]);
}

#[test]
fn syllable_vowel_only() {
    // Vowel-only syllables
    telex(&[
        ("a", "a"),
        ("e", "e"),
        ("i", "i"),
        ("o", "o"),
        ("u", "u"),
        ("y", "y"),
    ]);
}

#[test]
fn syllable_glide_oa() {
    // o as glide before a
    telex(&[("hoa", "hoa"), ("khoa", "khoa"), ("toa", "toa")]);
}

// ============================================================
// VALIDATION TESTS
// ============================================================

#[test]
fn validation_valid_simple() {
    // Valid simple words should transform
    telex(&[("bas", "bá"), ("caf", "cà"), ("dar", "dả")]);
}

#[test]
fn validation_valid_complex() {
    // Valid complex words
    telex(&[
        ("nghieengs", "nghiếng"),
        ("truowngf", "trường"),
        ("nguowif", "người"),
    ]);
}

#[test]
fn validation_spelling_k_before_eiy() {
    // k must be used before e, i, y
    telex(&[("kes", "ké"), ("kis", "kí"), ("kys", "ký")]);
}

#[test]
fn validation_spelling_c_before_aou() {
    // c must be used before a, o, u
    telex(&[("cas", "cá"), ("cos", "có"), ("cus", "cú")]);
}

#[test]
fn validation_spelling_gh_before_eiy() {
    // gh must be used before e, i
    telex(&[("ghes", "ghé"), ("ghis", "ghí")]);
}

#[test]
fn validation_spelling_ngh_before_eiy() {
    // ngh must be used before e, i
    telex(&[("nghes", "nghé"), ("nghis", "nghí")]);
}

// ============================================================
// TONE MODIFIER TESTS (V2 Pattern-based)
// ============================================================

#[test]
fn tone_circumflex_aa() {
    telex(&[
        ("aa", "â"),
        ("aas", "ấ"),
        ("aaf", "ầ"),
        ("aar", "ẩ"),
        ("aax", "ẫ"),
        ("aaj", "ậ"),
    ]);
}

#[test]
fn tone_circumflex_ee() {
    telex(&[
        ("ee", "ê"),
        ("ees", "ế"),
        ("eef", "ề"),
        ("eer", "ể"),
        ("eex", "ễ"),
        ("eej", "ệ"),
    ]);
}

#[test]
fn tone_circumflex_oo() {
    telex(&[
        ("oo", "ô"),
        ("oos", "ố"),
        ("oof", "ồ"),
        ("oor", "ổ"),
        ("oox", "ỗ"),
        ("ooj", "ộ"),
    ]);
}

#[test]
fn tone_circumflex_delayed() {
    // Delayed circumflex: vowel + consonant + same_vowel → circumflex + consonant
    telex(&[("oio", "ôi"), ("aia", "âi"), ("aua", "âu"), ("eie", "êi")]);
    // Delayed circumflex with final consonant: initial + vowel + consonant + same_vowel + final
    telex(&[
        ("nanag", "nâng"), // nâng - common Vietnamese word
        ("lanam", "lânm"), // lânm - partial word (tests pattern)
        ("tanat", "tânt"), // tânt - partial word (tests pattern)
    ]);
}

#[test]
fn tone_horn_ow() {
    telex(&[
        ("ow", "ơ"),
        ("ows", "ớ"),
        ("owf", "ờ"),
        ("owr", "ở"),
        ("owx", "ỡ"),
        ("owj", "ợ"),
    ]);
}

#[test]
fn tone_horn_uw() {
    telex(&[
        ("uw", "ư"),
        ("uws", "ứ"),
        ("uwf", "ừ"),
        ("uwr", "ử"),
        ("uwx", "ữ"),
        ("uwj", "ự"),
    ]);
}

#[test]
fn tone_breve_aw() {
    // Issue #44: Breve in open syllable is deferred when consonant before 'a'
    // "aw" alone becomes "ă" because no consonant before 'a' (pure Vietnamese shortcut)
    // But "raw" → "raw" deferred because consonant before 'a' (could be English)
    telex(&[
        ("aw", "ă"),   // Standalone: no consonant before 'a' → apply breve
        ("aws", "ắ"),  // Mark confirms Vietnamese: breve applied + sắc
        ("awf", "ằ"),  // Mark confirms Vietnamese: breve applied + huyền
        ("awr", "ẳ"),  // Mark confirms Vietnamese: breve applied + hỏi
        ("awx", "ẵ"),  // Mark confirms Vietnamese: breve applied + ngã
        ("awj", "ặ"),  // Mark confirms Vietnamese: breve applied + nặng
        ("awm", "ăm"), // Final consonant: breve applied
        ("awn", "ăn"), // Final consonant: breve applied
    ]);
}

#[test]
fn tone_uo_compound() {
    // Issue #133: "uơ" pattern - only 'o' gets horn when no final consonant
    // "ươ" pattern - both get horn when there IS a final consonant
    telex(&[
        ("dduowc", "đươc"), // dd for đ, final 'c' → both get horn
        ("uow", "uơ"),      // No final → only 'o' gets horn (Issue #133)
        ("muown", "mươn"),  // Final 'n' → both get horn
    ]);
}

// ============================================================
// MARK MODIFIER TESTS
// ============================================================

#[test]
fn mark_sac() {
    telex(&[
        ("as", "á"),
        ("es", "é"),
        ("is", "í"),
        ("os", "ó"),
        ("us", "ú"),
        ("ys", "ý"),
    ]);
}

#[test]
fn mark_huyen() {
    telex(&[
        ("af", "à"),
        ("ef", "è"),
        ("if", "ì"),
        ("of", "ò"),
        ("uf", "ù"),
        ("yf", "ỳ"),
    ]);
}

#[test]
fn mark_hoi() {
    telex(&[
        ("ar", "ả"),
        ("er", "ẻ"),
        ("ir", "ỉ"),
        ("or", "ỏ"),
        ("ur", "ủ"),
        ("yr", "ỷ"),
    ]);
}

#[test]
fn mark_nga() {
    telex(&[
        ("ax", "ã"),
        ("ex", "ẽ"),
        ("ix", "ĩ"),
        ("ox", "õ"),
        ("ux", "ũ"),
        ("yx", "ỹ"),
    ]);
}

#[test]
fn mark_nang() {
    telex(&[
        ("aj", "ạ"),
        ("ej", "ẹ"),
        ("ij", "ị"),
        ("oj", "ọ"),
        ("uj", "ụ"),
        ("yj", "ỵ"),
    ]);
}

#[test]
fn mark_with_final_consonant() {
    // Mark placement with final consonant 'ch'
    // Both typing orders should produce the same result
    telex(&[
        ("casch", "cách"), // c-a-s(sắc)-ch → cách
        ("cachs", "cách"), // c-a-ch-s(sắc) → cách
    ]);
}

// ============================================================
// STROKE TRANSFORMATION (d → đ)
// ============================================================

#[test]
fn stroke_dd() {
    telex(&[("dd", "đ"), ("dda", "đa"), ("ddi", "đi"), ("ddo", "đo")]);
}

#[test]
fn stroke_delayed_valid_vietnamese() {
    // When 'd' is typed after "d + vowel", stroke is applied immediately
    // This allows: "did" → "đi", "dod" → "đo", etc.
    // The trailing 'd' triggers stroke and is consumed (not added to buffer)
    telex(&[
        ("dod", "đo"), // d triggers stroke: đo
        ("dad", "đa"), // d triggers stroke: đa
        ("did", "đi"), // d triggers stroke: đi
        ("dud", "đu"), // d triggers stroke: đu
    ]);

    // Delayed stroke WITH mark key applies both stroke and mark
    telex(&[
        ("dods", "đó"), // Delayed stroke + sắc
        ("dads", "đá"), // Delayed stroke + sắc
        ("dids", "đí"), // Delayed stroke + sắc
        ("duds", "đú"), // Delayed stroke + sắc
        ("dodf", "đò"), // Delayed stroke + huyền
        ("dodx", "đõ"), // Delayed stroke + ngã
    ]);

    // For syllables WITH final consonant, delayed stroke applies immediately
    telex(&[
        ("docd", "đoc"), // Has final 'c' - immediate delayed stroke
        ("datd", "đat"), // Has final 't' - immediate delayed stroke
    ]);
}

#[test]
fn stroke_short_pattern_revert() {
    // When short-pattern stroke is applied (dad → đa), another 'd' reverts it (dadd → dad)
    // Similar to ddd → dd behavior for adjacent stroke
    telex(&[
        ("dadd", "dad"), // Short-pattern stroke reverted
        ("didd", "did"), // Short-pattern stroke reverted
        ("dodd", "dod"), // Short-pattern stroke reverted
        ("dudd", "dud"), // Short-pattern stroke reverted
    ]);
}

#[test]
fn stroke_in_word() {
    telex(&[
        ("ddas", "đá"),
        ("ddef", "đè"),
        ("ddif", "đì"),
        ("ddos", "đó"),
    ]);
}

// ============================================================
// REVERT BEHAVIOR TESTS
// ============================================================

#[test]
fn revert_tone_double_key() {
    // aaa → aa (revert â back to aa)
    telex(&[("aaa", "aa"), ("eee", "ee"), ("ooo", "oo")]);
}

#[test]
fn revert_mark_double_key() {
    // When mark is reverted, only the reverting key appears as a letter.
    // Standard behavior: first key was modifier, second key reverts and outputs one letter.
    // This allows typing words like "test" (tesst), "next" (nexxt), etc.
    // ass → as: first 's' was modifier for á, second 's' reverts and outputs one 's'
    telex(&[
        ("ass", "as"),
        ("aff", "af"),
        ("arr", "ar"),
        ("axx", "ax"),
        ("ajj", "aj"),
    ]);
}

#[test]
fn revert_stroke_double_key() {
    // ddd → dd (third d reverts stroke, returning to raw "dd")
    // This matches user expectation: if you typed too many d's, you get raw text
    telex(&[("ddd", "dd")]);
}

#[test]
fn triple_same_key() {
    // aaaa → aâ
    let mut e = Engine::new();
    let result = common::type_word(&mut e, "aaaa");
    assert_eq!(result, "aâ");
}

// ============================================================
// VNI EQUIVALENTS
// ============================================================

#[test]
fn vni_tone_circumflex() {
    vni(&[("a6", "â"), ("e6", "ê"), ("o6", "ô")]);
}

#[test]
fn vni_tone_horn() {
    vni(&[("o7", "ơ"), ("u7", "ư")]);
}

#[test]
fn vni_tone_breve() {
    // Issue #44: Breve in open syllable is deferred when consonant before 'a'
    // "a8" alone becomes "ă" because no consonant before 'a' (pure Vietnamese shortcut)
    // But "ra8" → "ra8" deferred because consonant before 'a' (could be English)
    vni(&[
        ("a8", "ă"),     // Standalone: no consonant before 'a' → apply breve
        ("a8m", "ăm"),   // Final consonant: breve applied
        ("a8n", "ăn"),   // Final consonant: breve applied
        ("a8c", "ăc"),   // Final consonant: breve applied
        ("a8t", "ăt"),   // Final consonant: breve applied
        ("a8p", "ăp"),   // Final consonant: breve applied
        ("ta8m", "tăm"), // tăm - silkworm
        ("la8m", "lăm"), // lăm - five (colloquial)
    ]);
}

#[test]
fn vni_marks() {
    vni(&[
        ("a1", "á"),
        ("a2", "à"),
        ("a3", "ả"),
        ("a4", "ã"),
        ("a5", "ạ"),
    ]);
}

#[test]
fn vni_stroke() {
    vni(&[("d9", "đ"), ("d9a", "đa")]);
}

// ============================================================
// EDGE CASES & REGRESSION TESTS
// ============================================================

#[test]
fn edge_gi_with_mark() {
    // gi + au + mark = giàu
    telex(&[("giauf", "giàu"), ("giaus", "giáu")]);
}

#[test]
fn edge_qu_with_mark() {
    // qu + a + mark
    telex(&[
        ("quas", "quá"),
        ("quaf", "quà"),
        ("quoocs", "quốc"), // Need oo for ô
    ]);
}

#[test]
fn edge_ia_tone_placement() {
    // ia → tone on i (short vowel), not a
    // kìa, mía, lìa - descending diphthong where i is main vowel
    telex(&[
        ("iaf", "ìa"),
        ("ias", "ía"),
        ("iar", "ỉa"),
        ("iax", "ĩa"),
        ("iaj", "ịa"),
        ("kiaf", "kìa"),
        ("mias", "mía"),
        ("liaf", "lìa"),
    ]);
}

#[test]
fn edge_mixed_modifiers() {
    // Tone + mark combinations
    telex(&[
        ("aas", "ấ"), // â + sắc
        ("ees", "ế"), // ê + sắc
        ("oos", "ố"), // ô + sắc
        ("ows", "ớ"), // ơ + sắc
        ("uws", "ứ"), // ư + sắc
        ("aws", "ắ"), // ă + sắc
    ]);
}

#[test]
fn edge_long_words() {
    telex(&[
        ("nghieengs", "nghiếng"),
        ("khuyeenx", "khuyễn"),
        ("nguowif", "người"),
        ("truowngf", "trường"),
    ]);
}

#[test]
fn edge_invalid_not_transformed() {
    // Invalid Vietnamese should not be transformed
    // These words don't follow Vietnamese phonology rules
    // and should be passed through
    let mut e = Engine::new();

    // "http" has no vowel - should pass through
    let result = common::type_word(&mut e, "https");
    // Note: 's' at the end might trigger mark, but 'http' part stays
    assert!(result.contains("http"));
}

// ============================================================
// DELAYED CIRCUMFLEX TESTS
// ============================================================
//
// Pattern: V + C + V (same vowel) triggers circumflex on first vowel
// Examples: "toto" → "tôt", "data" → "dât"
// With auto-restore: "toto " → "toto " (restored if no mark)

#[test]
fn delayed_circumflex_with_mark() {
    // Delayed circumflex triggered by mark key (s/f/r/x/j)
    // Pattern: V + C + V + mark → circumflex on first V + mark
    // Note: This also works for immediate circumflex (V + V) pattern
    telex(&[
        ("totos", "tốt"),  // tốt - circumflex + sắc
        ("notos", "nốt"),  // nốt - circumflex + sắc
        ("hetes", "hết"),  // hết - circumflex + sắc
        ("datdas", "đất"), // đất - delayed stroke + circumflex + sắc
        ("soos", "số"),    // số - immediate circumflex (oo) + sắc
        ("boos", "bố"),    // bố - immediate circumflex (oo) + sắc
        ("mees", "mế"),    // mế - immediate circumflex (ee) + sắc
    ]);
}

#[test]
fn delayed_circumflex_vowel_trigger() {
    // Delayed circumflex triggered by second matching vowel
    // Pattern: V + C + V (same vowel) → circumflex on first V, remove trigger
    telex(&[
        ("toto", "tôt"),   // tôt - second 'o' triggers circumflex
        ("noto", "nôt"),   // nôt - second 'o' triggers circumflex
        ("data", "dât"),   // dât - second 'a' triggers circumflex
        ("dataa", "data"), // data - third 'a' reverts circumflex (â→a)
        ("hete", "hêt"),   // hêt - second 'e' triggers circumflex
        ("tetee", "tete"), // tete - third 'e' reverts circumflex (ê→e)
        ("cocoo", "coco"), // coco - third 'o' reverts circumflex (ô→o)
    ]);
}

#[test]
fn delayed_circumflex_extending_consonant() {
    // Consonants that can extend (n→ng/nh, c→ch) allow immediate circumflex
    telex(&[
        ("nanag", "nâng"), // nâng - n can extend to ng
    ]);
}

#[test]
fn delayed_circumflex_diphthong_pattern() {
    // Diphthong patterns: circumflex on first vowel of diphthong
    // Pattern: C + V₁ + V₂ + mark + V₁ → circumflex on V₁
    // Note: circumflex requires vowel trigger (second V₁) after mark
    telex(&[
        ("dausa", "dấu"),  // dấu - âu diphthong: sắc then vowel trigger
        ("dausfa", "dầu"), // dầu - âu diphthong: sắc → huyền then vowel trigger
        ("daysa", "dấy"),  // dấy - ây diphthong: sắc then vowel trigger
    ]);
}

#[test]
fn delayed_circumflex_auto_restore_space() {
    // V+C+V circumflex patterns with stop consonant finals (t/c/p) WITHOUT mark
    // are almost never real Vietnamese words → restore to English
    // Compare: "tốt" (with sắc) is Vietnamese, but "tôt" (no mark) is not
    use gonhanh_core::utils::type_word;

    let cases = [
        ("toto ", "toto "),  // tôt (no mark) is NOT real VI → restore to English
        ("data ", "data "),  // dât (no mark) is NOT real VI → restore to English
        ("dataa ", "data "), // Revert: dataa → data (circumflex reverted)
        ("noto ", "noto "),  // nôt (no mark) is NOT real VI → restore to English
        ("hete ", "hete "),  // hêt (no mark) is NOT real VI → restore to English
        ("tetee ", "tete "), // Revert: tetee → tete (circumflex reverted)
        ("cocoo ", "coco "), // Revert: cocoo → coco (circumflex reverted)
    ];

    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_english_auto_restore(true);
        let result = type_word(&mut e, input);
        assert_eq!(result, expected, "Auto-restore failed for '{}'", input);
    }
}

#[test]
fn delayed_circumflex_valid_vietnamese_stays() {
    // Valid Vietnamese with marks should NOT be restored
    use gonhanh_core::utils::type_word;

    let cases = [
        ("dausa ", "dấu "), // Valid: dấu (mark typed)
        ("totos ", "tốt "), // Valid: tốt (mark typed)
        ("soos ", "số "),   // Valid: số (mark typed, immediate circumflex)
        ("notos ", "nốt "), // Valid: nốt (mark typed)
    ];

    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_english_auto_restore(true);
        let result = type_word(&mut e, input);
        assert_eq!(
            result, expected,
            "Valid Vietnamese should stay for '{}'",
            input
        );
    }
}

#[test]
fn delayed_circumflex_punctuation_restore() {
    // Punctuation marks trigger auto-restore for INVALID Vietnamese
    // V+C+V circumflex with stop consonant (t/c/p) without mark → restore to English
    use gonhanh_core::utils::type_word;

    let cases = [
        ("toto,", "toto,"), // tôt (no mark) is NOT real VI → restore to English
        ("data.", "data."), // dât (no mark) is NOT real VI → restore to English
        ("data;", "data;"), // dât (no mark) is NOT real VI → restore to English
        ("dausa,", "dấu,"), // Valid Vietnamese stays (with mark)
        ("user.", "user."), // English word + dot (usẻ invalid VI → restore)
        ("user,", "user,"), // English word + comma
        ("user;", "user;"), // English word + semicolon
        ("user:", "user:"), // English word + colon
        ("user@", "user@"), // English word + @ (email pattern)
    ];

    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_english_auto_restore(true);
        let result = type_word(&mut e, input);
        assert_eq!(
            result, expected,
            "Punctuation auto-restore failed for '{}'",
            input
        );
    }
}

#[test]
fn delayed_circumflex_no_false_positives() {
    // Words that should NOT get circumflex
    // - Words where target vowel already has a mark
    // - Words with invalid diphthong patterns
    use gonhanh_core::utils::type_word;

    // "expect" = e-x-p-e-c-t: 'x' applies ngã to 'e', second 'e' should NOT trigger circumflex
    let mut e1 = Engine::new();
    let result1 = type_word(&mut e1, "expect");
    assert!(
        result1.contains('ẽ') && !result1.contains('ễ'),
        "expect should have ẽ not ễ, got: '{}'",
        result1
    );

    // "teacher" = t-e-a-c-h-e-r: "ea" is not valid diphthong, no circumflex
    let mut e2 = Engine::new();
    let result2 = type_word(&mut e2, "teacher");
    assert_eq!(
        result2, "teacher",
        "teacher should stay unchanged, got: '{}'",
        result2
    );
}
