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
    // Auto-restore on space: if no mark was typed, restore to raw input
    // This handles English words like "data" that look like Vietnamese patterns
    use gonhanh_core::utils::type_word;

    let cases = [
        ("toto ", "toto "),  // No mark → restore
        ("data ", "data "),  // No mark → restore
        ("dataa ", "data "), // Revert: dataa → data (no restore needed, already plain)
        ("noto ", "noto "),  // No mark → restore
        ("hete ", "hete "),  // No mark → restore
        ("tetee ", "tete "), // Revert: tetee → tete (no restore needed)
        ("cocoo ", "coco "), // Revert: cocoo → coco (no restore needed)
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
    // Punctuation marks (comma, dot, semicolon, colon, @) also trigger auto-restore
    use gonhanh_core::utils::type_word;

    let cases = [
        ("toto,", "toto,"), // Comma triggers restore
        ("data.", "data."), // Dot triggers restore
        ("data;", "data;"), // Semicolon triggers restore
        ("dausa,", "dấu,"), // Valid Vietnamese stays (with punctuation)
        ("user.", "user."), // English word + dot
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
    // - Words where foreign word detection triggers
    // - Words with invalid diphthong patterns
    use gonhanh_core::utils::type_word;

    // "expect" = e-x-p-e-c-t: 'x' applies ngã to 'e', but 'p' (consonant) follows
    // Foreign word detection triggers on "x + consonant" pattern → word becomes plain "expect"
    let mut e1 = Engine::new();
    let result1 = type_word(&mut e1, "expect");
    assert_eq!(
        result1, "expect",
        "expect should be plain (foreign word detection), got: '{}'",
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

#[test]
fn debug_user_at() {
    use gonhanh_core::data::keys;
    use gonhanh_core::utils::{char_to_key, type_word};

    let mut e = Engine::new();
    e.set_english_auto_restore(true);

    // Type "user" first
    for c in "user".chars() {
        e.on_key(char_to_key(c), false, false);
    }

    println!("After 'user':");
    println!("  Buffer: '{}'", e.get_buffer_string());

    // Now type '@' (Shift+2)
    let result = e.on_key_ext(keys::N2, false, false, true);
    println!("\nAfter '@':");
    println!("  Action: {}", result.action);
    println!("  Backspace: {}", result.backspace);
    println!("  Count: {}", result.count);
    if result.count > 0 {
        let chars: String = result.chars[..result.count as usize]
            .iter()
            .filter_map(|&c| char::from_u32(c))
            .collect();
        println!("  Chars: '{}'", chars);
    }

    // Compare with type_word
    let mut e2 = Engine::new();
    e2.set_english_auto_restore(true);
    let result = type_word(&mut e2, "user@");
    println!("\ntype_word result: '{}'", result);
}

#[test]
fn debug_ddepj() {
    use gonhanh_core::data::keys;
    use gonhanh_core::utils::{char_to_key, type_word};

    // Test 1: Simple ddepj without auto_restore
    let mut e1 = Engine::new();
    let r1 = type_word(&mut e1, "ddepj ");
    println!("Simple (no auto_restore): '{}' (expected: 'đẹp ')", r1);

    // Test 2: Step by step with auto_restore
    let mut e2 = Engine::new();
    e2.set_english_auto_restore(true);
    for c in "ddepj".chars() {
        let key = char_to_key(c);
        let r = e2.on_key(key, false, false);
        println!(
            "After '{}': buffer='{}', action={}",
            c,
            e2.get_buffer_string(),
            r.action
        );
    }
    // Now type space
    let r = e2.on_key(keys::SPACE, false, false);
    println!(
        "After SPACE: action={}, backspace={}, count={}",
        r.action, r.backspace, r.count
    );

    // Test 3: Full word with auto_restore via type_word
    let mut e3 = Engine::new();
    e3.set_english_auto_restore(true);
    let r3 = type_word(&mut e3, "ddepj ");
    println!(
        "With auto_restore via type_word: '{}' (expected: 'đẹp ')",
        r3
    );
}

#[test]
fn debug_raast() {
    use gonhanh_core::data::keys;
    use gonhanh_core::utils::{char_to_key, type_word};

    // Test 1: Fresh engine, type raast
    println!("Test 1: Fresh engine, type raast ");
    let mut e1 = Engine::new();
    e1.set_method(0);
    e1.set_english_auto_restore(true);
    let r1 = type_word(&mut e1, "raast ");
    println!("Result: '{}' (expected: 'rất ')", r1);

    // Test 2: Step by step raast
    println!("\nTest 2: Step by step raast");
    let mut e2 = Engine::new();
    e2.set_method(0);
    e2.set_english_auto_restore(true);
    for c in "raast".chars() {
        let key = char_to_key(c);
        let r = e2.on_key(key, false, false);
        println!(
            "After '{}': buffer='{}', action={}",
            c,
            e2.get_buffer_string(),
            r.action
        );
    }
    let r = e2.on_key(keys::SPACE, false, false);
    println!(
        "After SPACE: action={}, backspace={}, count={}",
        r.action, r.backspace, r.count
    );
    if r.action == 2 {
        let chars: String = r.chars[..r.count as usize]
            .iter()
            .filter_map(|&c| char::from_u32(c))
            .collect();
        println!("Restore chars: '{}'", chars);
    }

    // Test 3: After Tab, type raast
    println!("\nTest 3: Engine after lawsm, Tab, then raast");
    let mut e3 = Engine::new();
    e3.set_method(0);
    e3.set_english_auto_restore(true);
    type_word(&mut e3, "lawsm");
    println!("After lawsm: buffer = '{}'", e3.get_buffer_string());
    e3.on_key(keys::TAB, false, false);
    println!("After Tab: buffer = '{}'", e3.get_buffer_string());
    let r3 = type_word(&mut e3, "raast ");
    println!("Result: '{}' (expected: 'rất ')", r3);

    assert_eq!(r1, "rất ");
}
