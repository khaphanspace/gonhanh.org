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
        // Issue #183: Flexible order - mark AFTER the second vowel trigger
        // Pattern: C + V₁ + V₂ + V₁ + mark → circumflex on V₁ + mark
        // --- âu diphthong (A-U-A pattern) ---
        ("dauas", "dấu"), // dấu - sắc
        ("dauaf", "dầu"), // dầu - huyền
        ("dauar", "dẩu"), // dẩu - hỏi
        ("dauax", "dẫu"), // dẫu - ngã
        ("dauaj", "dậu"), // dậu - nặng
        ("xauas", "xấu"), // xấu
        ("xauaf", "xầu"), // xầu
        ("cauas", "cấu"), // cấu
        ("lauas", "lấu"), // lấu
        ("mauas", "mấu"), // mấu
        ("tauas", "tấu"), // tấu
        // --- êu diphthong (E-U-E pattern) ---
        ("neues", "nếu"), // nếu - sắc
        ("neuef", "nều"), // nều - huyền
        ("neuer", "nểu"), // nểu - hỏi
        ("neuex", "nễu"), // nễu - ngã
        ("neuej", "nệu"), // nệu - nặng
        ("keues", "kếu"), // kếu
        ("leues", "lếu"), // lếu
        ("meues", "mếu"), // mếu (crying sound)
        ("teues", "tếu"), // tếu
        // --- ôi diphthong (O-I-O pattern) ---
        ("toios", "tối"), // tối - sắc
        ("toiof", "tồi"), // tồi - huyền
        ("toior", "tổi"), // tổi - hỏi
        ("toiox", "tỗi"), // tỗi - ngã
        ("toioj", "tội"), // tội - nặng
        ("coios", "cối"), // cối
        ("doios", "dối"), // dối
        ("loios", "lối"), // lối
        ("moios", "mối"), // mối
        ("noios", "nối"), // nối
        // --- ây diphthong (A-Y-A pattern) ---
        ("dayas", "dấy"), // dấy - sắc
        ("dayaf", "dầy"), // dầy - huyền
        ("dayar", "dẩy"), // dẩy - hỏi
        ("dayax", "dẫy"), // dẫy - ngã
        ("dayaj", "dậy"), // dậy - nặng
        ("cayas", "cấy"), // cấy
        ("layas", "lấy"), // lấy
        ("mayas", "mấy"), // mấy
        ("tayas", "tấy"), // tấy
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

// ============================================================
// SINGLE VOWEL + TONE MARK + SPACE (AUTO-RESTORE)
// ============================================================

#[test]
fn single_vowel_sac_space() {
    // Single vowel + sắc (s) + space
    // ALL are valid Vietnamese - should NOT restore
    // Logic: valid_VN → keep VN (don't check English first)
    common::telex_auto_restore(&[
        ("as ", "á "), // Valid VN: á (exclamation)
        ("es ", "é "), // Valid VN: é (exclamation)
        ("is ", "í "), // Valid VN: í (exclamation)
        ("os ", "ó "), // Valid VN: ó (agreement, like ừ)
        ("us ", "ú "), // Valid VN: ú (exclamation)
        ("ys ", "ý "), // Valid VN: ý (meaning/idea)
    ]);
}

#[test]
fn single_vowel_huyen_space() {
    // Single vowel + huyền (f) + space
    // ALL are valid Vietnamese - should NOT restore
    common::telex_auto_restore(&[
        ("af ", "à "), // Valid VN: à (realization)
        ("ef ", "è "), // Valid VN: è (exclamation)
        ("if ", "ì "), // Valid VN: ì (as in "ì ạch")
        ("of ", "ò "), // Valid VN: ò (exclamation)
        ("uf ", "ù "), // Valid VN: ù (as in "ù ù")
        ("yf ", "ỳ "), // Valid VN: ỳ
    ]);
}

#[test]
fn single_vowel_hoi_space() {
    // Single vowel + hỏi (r) + space
    // ALL are valid Vietnamese - should NOT restore
    common::telex_auto_restore(&[
        ("ar ", "ả "), // Valid VN: ả
        ("er ", "ẻ "), // Valid VN: ẻ
        ("ir ", "ỉ "), // Valid VN: ỉ
        ("or ", "ỏ "), // Valid VN: ỏ
        ("ur ", "ủ "), // Valid VN: ủ (as in "ủ rũ")
        ("yr ", "ỷ "), // Valid VN: ỷ
    ]);
}

#[test]
fn single_vowel_nga_space() {
    // Single vowel + ngã (x) + space
    // ALL are valid Vietnamese - should NOT restore
    common::telex_auto_restore(&[
        ("ax ", "ã "), // Valid VN: ã
        ("ex ", "ẽ "), // Valid VN: ẽ
        ("ix ", "ĩ "), // Valid VN: ĩ
        ("ox ", "õ "), // Valid VN: õ
        ("ux ", "ũ "), // Valid VN: ũ
        ("yx ", "ỹ "), // Valid VN: ỹ
    ]);
}

#[test]
fn single_vowel_nang_space() {
    // Single vowel + nặng (j) + space
    // ALL are valid Vietnamese - should NOT restore
    // Especially "ạ" which is very common ("vâng ạ", "dạ ạ")
    common::telex_auto_restore(&[
        ("aj ", "ạ "), // Valid VN: ạ (respectful particle)
        ("ej ", "ẹ "), // Valid VN: ẹ
        ("ij ", "ị "), // Valid VN: ị
        ("oj ", "ọ "), // Valid VN: ọ
        ("uj ", "ụ "), // Valid VN: ụ
        ("yj ", "ỵ "), // Valid VN: ỵ
    ]);
}

#[test]
fn single_horn_ow_tone_space() {
    // ơ (ow) + tone marks + space
    // All are valid Vietnamese interjections: ớ, ờ, ở, ỡ, ợ
    common::telex_auto_restore(&[
        ("ows ", "ớ "), // Valid VN: ớ (exclamation)
        ("owf ", "ờ "), // Valid VN: ờ (exclamation)
        ("owr ", "ở "), // Valid VN: ở (to live/stay)
        ("owx ", "ỡ "), // Valid VN: ỡ
        ("owj ", "ợ "), // Valid VN: ợ
    ]);
}

#[test]
fn single_horn_uw_tone_space() {
    // ư (uw) + tone marks + space
    // All are valid Vietnamese interjections: ứ, ừ, ử, ữ, ự
    common::telex_auto_restore(&[
        ("uws ", "ứ "), // Valid VN: ứ (exclamation)
        ("uwf ", "ừ "), // Valid VN: ừ (informal yes/agreement)
        ("uwr ", "ử "), // Valid VN: ử
        ("uwx ", "ữ "), // Valid VN: ữ
        ("uwj ", "ự "), // Valid VN: ự
    ]);
}

// ============================================================
// DIPHTHONG (DOUBLE VOWEL) + TONE MARK + SPACE (AUTO-RESTORE)
// ============================================================

#[test]
fn diphthong_ai_tone_space() {
    // ai + tone marks + space
    common::telex_auto_restore(&[
        ("ais ", "ái "),
        ("aif ", "ài "),
        ("air ", "ải "),
        ("aix ", "ãi "),
        ("aij ", "ại "),
    ]);
}

#[test]
fn diphthong_ao_tone_space() {
    // ao + tone marks + space
    common::telex_auto_restore(&[
        ("aos ", "áo "),
        ("aof ", "ào "),
        ("aor ", "ảo "),
        ("aox ", "ão "),
        ("aoj ", "ạo "),
    ]);
}

#[test]
fn diphthong_au_tone_space() {
    // au + tone marks + space
    common::telex_auto_restore(&[
        ("aus ", "áu "),
        ("auf ", "àu "),
        ("aur ", "ảu "),
        ("aux ", "ãu "),
        ("auj ", "ạu "),
    ]);
}

#[test]
fn diphthong_ay_tone_space() {
    // ay + tone marks + space
    common::telex_auto_restore(&[
        ("ays ", "áy "),
        ("ayf ", "ày "),
        ("ayr ", "ảy "),
        ("ayx ", "ãy "),
        ("ayj ", "ạy "),
    ]);
}

#[test]
fn diphthong_aau_tone_space() {
    // âu (aa + u) + tone marks + space
    common::telex_auto_restore(&[
        ("aaus ", "ấu "),
        ("aauf ", "ầu "),
        ("aaur ", "ẩu "),
        ("aaux ", "ẫu "),
        ("aauj ", "ậu "),
    ]);
}

#[test]
fn diphthong_aay_tone_space() {
    // ây (aa + y) + tone marks + space
    common::telex_auto_restore(&[
        ("aays ", "ấy "),
        ("aayf ", "ầy "),
        ("aayr ", "ẩy "),
        ("aayx ", "ẫy "),
        ("aayj ", "ậy "),
    ]);
}

#[test]
fn diphthong_eo_tone_space() {
    // eo + tone marks + space
    common::telex_auto_restore(&[
        ("eos ", "éo "),
        ("eof ", "èo "),
        ("eor ", "ẻo "),
        ("eox ", "ẽo "),
        ("eoj ", "ẹo "),
    ]);
}

#[test]
fn diphthong_eeu_tone_space() {
    // êu (ee + u) + tone marks + space
    common::telex_auto_restore(&[
        ("eeus ", "ếu "),
        ("eeuf ", "ều "),
        ("eeur ", "ểu "),
        ("eeux ", "ễu "),
        ("eeuj ", "ệu "),
    ]);
}

#[test]
fn diphthong_ia_tone_space() {
    // ia + tone marks + space (tone on i - main vowel)
    common::telex_auto_restore(&[
        ("ias ", "ía "),
        ("iaf ", "ìa "),
        ("iar ", "ỉa "),
        ("iax ", "ĩa "),
        ("iaj ", "ịa "),
    ]);
}

#[test]
fn diphthong_iu_tone_space() {
    // iu + tone marks + space
    common::telex_auto_restore(&[
        ("ius ", "íu "),
        ("iuf ", "ìu "),
        ("iur ", "ỉu "),
        ("iux ", "ĩu "),
        ("iuj ", "ịu "),
    ]);
}

#[test]
fn diphthong_oa_tone_space() {
    // oa + tone marks + space
    // Note: in "oa" glide pattern, tone goes on 'a' (main vowel), not 'o'
    common::telex_auto_restore(&[
        ("oas ", "oá "),
        ("oaf ", "oà "),
        ("oar ", "oả "),
        ("oax ", "oã "),
        ("oaj ", "oạ "),
    ]);
}

#[test]
fn diphthong_oe_tone_space() {
    // oe + tone marks + space
    // Note: in "oe" glide pattern, tone goes on 'e' (main vowel), not 'o'
    common::telex_auto_restore(&[
        ("oes ", "oé "),
        ("oef ", "oè "),
        ("oer ", "oẻ "),
        ("oex ", "oẽ "),
        ("oej ", "oẹ "),
    ]);
}

#[test]
fn diphthong_oi_tone_space() {
    // oi + tone marks + space
    common::telex_auto_restore(&[
        ("ois ", "ói "),
        ("oif ", "òi "),
        ("oir ", "ỏi "),
        ("oix ", "õi "),
        ("oij ", "ọi "),
    ]);
}

#[test]
fn diphthong_ooi_tone_space() {
    // ôi (oo + i) + tone marks + space
    common::telex_auto_restore(&[
        ("oois ", "ối "),
        ("ooif ", "ồi "),
        ("ooir ", "ổi "),
        ("ooix ", "ỗi "),
        ("ooij ", "ội "),
    ]);
}

#[test]
fn diphthong_owi_tone_space() {
    // ơi (ow + i) + tone marks + space
    common::telex_auto_restore(&[
        ("owis ", "ới "),
        ("owif ", "ời "),
        ("owir ", "ởi "),
        ("owix ", "ỡi "),
        ("owij ", "ợi "),
    ]);
}

#[test]
fn diphthong_ua_tone_space() {
    // ua + tone marks + space
    common::telex_auto_restore(&[
        ("uas ", "úa "),
        ("uaf ", "ùa "),
        ("uar ", "ủa "),
        ("uax ", "ũa "),
        ("uaj ", "ụa "),
    ]);
}

#[test]
fn diphthong_uee_tone_space() {
    // uê (u + ee) + tone marks + space
    common::telex_auto_restore(&[
        ("uees ", "uế "),
        ("ueef ", "uề "),
        ("ueer ", "uể "),
        ("ueex ", "uễ "),
        ("ueej ", "uệ "),
    ]);
}

#[test]
fn diphthong_ui_tone_space() {
    // ui + tone marks + space
    common::telex_auto_restore(&[
        ("uis ", "úi "),
        ("uif ", "ùi "),
        ("uir ", "ủi "),
        ("uix ", "ũi "),
        ("uij ", "ụi "),
    ]);
}

#[test]
fn diphthong_uow_tone_space() {
    // uơ (u + ow) + tone marks + space
    // Issue #133: only 'ơ' gets horn when no final consonant ("uơ" not "ươ")
    common::telex_auto_restore(&[
        ("uows ", "uớ "),
        ("uowf ", "uờ "),
        ("uowr ", "uở "),
        ("uowx ", "uỡ "),
        ("uowj ", "uợ "),
    ]);
}

#[test]
fn diphthong_uwi_tone_space() {
    // ưi (uw + i) + tone marks + space
    common::telex_auto_restore(&[
        ("uwis ", "ứi "),
        ("uwif ", "ừi "),
        ("uwir ", "ửi "),
        ("uwix ", "ữi "),
        ("uwij ", "ựi "),
    ]);
}

// ============================================================
// TRIPHTHONG (TRIPLE VOWEL) + TONE MARK + SPACE (AUTO-RESTORE)
// ============================================================

#[test]
fn triphthong_ieu_tone_space() {
    // iêu (i + ee + u) + tone marks + space
    common::telex_auto_restore(&[
        ("ieeus ", "iếu "),
        ("ieeuf ", "iều "),
        ("ieeur ", "iểu "),
        ("ieeux ", "iễu "),
        ("ieeuj ", "iệu "),
    ]);
}

#[test]
fn triphthong_oai_tone_space() {
    // oai + tone marks + space
    common::telex_auto_restore(&[
        ("oais ", "oái "),
        ("oaif ", "oài "),
        ("oair ", "oải "),
        ("oaix ", "oãi "),
        ("oaij ", "oại "),
    ]);
}

#[test]
fn triphthong_oay_tone_space() {
    // oay + tone marks + space
    common::telex_auto_restore(&[
        ("oays ", "oáy "),
        ("oayf ", "oày "),
        ("oayr ", "oảy "),
        ("oayx ", "oãy "),
        ("oayj ", "oạy "),
    ]);
}

#[test]
fn triphthong_uoi_tone_space() {
    // uôi (u + oo + i) + tone marks + space
    common::telex_auto_restore(&[
        ("uoois ", "uối "),
        ("uooif ", "uồi "),
        ("uooir ", "uổi "),
        ("uooix ", "uỗi "),
        ("uooij ", "uội "),
    ]);
}

#[test]
fn triphthong_uowi_tone_space() {
    // ươi (u + ow + i) + tone marks + space
    common::telex_auto_restore(&[
        ("uowis ", "ưới "),
        ("uowif ", "ười "),
        ("uowir ", "ưởi "),
        ("uowix ", "ưỡi "),
        ("uowij ", "ượi "),
    ]);
}

#[test]
fn triphthong_uya_tone_space() {
    // uya + tone marks + space
    // Note: "uya" patterns often get restored as they look like English
    common::telex_auto_restore(&[
        ("uyas ", "uyas "), // Restored: looks like English
        ("uyaf ", "uyaf "), // Restored: looks like English
        ("uyar ", "uyar "), // Restored: looks like English
        ("uyax ", "uyax "), // Restored: looks like English
        ("uyaj ", "uyaj "), // Restored: looks like English
    ]);
}

#[test]
fn triphthong_yeu_tone_space() {
    // yêu (y + ee + u) + tone marks + space
    common::telex_auto_restore(&[
        ("yeeus ", "yếu "),
        ("yeeuf ", "yều "),
        ("yeeur ", "yểu "),
        ("yeeux ", "yễu "),
        ("yeeuj ", "yệu "),
    ]);
}
