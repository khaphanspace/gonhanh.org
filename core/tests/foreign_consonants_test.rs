//! Foreign Consonants Tests - Testing allow_foreign_consonants option
//!
//! Tests that z, w, j, f can be used as valid initial consonants when
//! the allow_foreign_consonants option is enabled.

mod common;
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

/// Helper to run telex tests with foreign consonants enabled
fn telex_foreign(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_allow_foreign_consonants(true);
        let result = type_word(&mut e, input);
        assert_eq!(
            result, *expected,
            "[Telex ForeignConsonants] '{}' → '{}'",
            input, result
        );
    }
}

/// Helper to run telex tests with foreign consonants DISABLED (default)
fn telex_no_foreign(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_allow_foreign_consonants(false);
        let result = type_word(&mut e, input);
        assert_eq!(
            result, *expected,
            "[Telex NoForeignConsonants] '{}' → '{}'",
            input, result
        );
    }
}

// ============================================================
// FOREIGN CONSONANTS ENABLED TESTS
// ============================================================

#[test]
fn foreign_z_with_diacritics() {
    // Z as initial consonant with Vietnamese diacritics
    telex_foreign(&[
        ("zas", "zá"),  // z + a + sắc
        ("zaf", "zà"),  // z + a + huyền
        ("zar", "zả"),  // z + a + hỏi
        ("zax", "zã"),  // z + a + ngã
        ("zaj", "zạ"),  // z + a + nặng
        ("zoos", "zố"), // z + ô + sắc (oo = ô in Telex)
    ]);
}

#[test]
fn foreign_j_with_diacritics() {
    // J as initial consonant with Vietnamese diacritics
    telex_foreign(&[
        ("jas", "já"),  // j + a + sắc
        ("jaf", "jà"),  // j + a + huyền
        ("jar", "jả"),  // j + a + hỏi
        ("jax", "jã"),  // j + a + ngã
        ("jaj", "jạ"),  // j + a + nặng
        ("joos", "jố"), // j + ô + sắc
    ]);
}

#[test]
fn foreign_f_with_diacritics() {
    // F as initial consonant with Vietnamese diacritics
    telex_foreign(&[
        ("fas", "fá"),  // f + a + sắc
        ("faf", "fà"),  // f + a + huyền
        ("far", "fả"),  // f + a + hỏi
        ("fax", "fã"),  // f + a + ngã
        ("faj", "fạ"),  // f + a + nặng
        ("foos", "fố"), // f + ô + sắc
    ]);
}

#[test]
fn foreign_with_marks() {
    // Foreign consonants with vowel marks (horn, breve, circumflex)
    telex_foreign(&[
        ("zaw", "ză"), // z + ă (breve via w)
        ("zaa", "zâ"), // z + â (circumflex via aa)
        ("zow", "zơ"), // z + ơ (horn)
        ("zoo", "zô"), // z + ô (circumflex via oo)
        ("fuw", "fư"), // f + ư (horn)
    ]);
}

#[test]
fn foreign_with_full_syllables() {
    // Foreign consonants with complete syllables (initial + vowel + final)
    telex_foreign(&[
        ("zans", "zán"), // z + án
        ("fams", "fám"), // f + ám
        ("jacs", "jác"), // j + ác
    ]);
}

// ============================================================
// FOREIGN CONSONANTS DISABLED TESTS (default behavior)
// ============================================================

#[test]
fn no_foreign_z_passthrough() {
    // When disabled, z should not get diacritics (invalid initial)
    telex_no_foreign(&[
        ("zas", "zas"), // No transformation - invalid initial
        ("zaf", "zaf"), // No transformation
    ]);
}

#[test]
fn no_foreign_f_passthrough() {
    // When disabled, f should not get diacritics (invalid initial)
    telex_no_foreign(&[
        ("fas", "fas"), // No transformation - invalid initial
        ("faf", "faf"), // No transformation
    ]);
}

#[test]
fn no_foreign_j_passthrough() {
    // When disabled, j should not get diacritics (invalid initial)
    telex_no_foreign(&[
        ("jas", "jas"), // No transformation - invalid initial
        ("jaf", "jaf"), // No transformation
    ]);
}

// ============================================================
// EDGE CASES
// ============================================================

#[test]
fn foreign_toggle_works() {
    // Test that setting works correctly
    let mut e = Engine::new();

    // Start with disabled (default)
    assert!(!e.allow_foreign_consonants());

    // Enable
    e.set_allow_foreign_consonants(true);
    assert!(e.allow_foreign_consonants());

    // Disable again
    e.set_allow_foreign_consonants(false);
    assert!(!e.allow_foreign_consonants());
}

#[test]
fn foreign_valid_vietnamese_unchanged() {
    // Valid Vietnamese initials should work the same with or without foreign option
    telex_foreign(&[
        ("bas", "bá"),
        ("cas", "cá"),
        ("das", "dá"),
        ("gas", "gá"),
        ("has", "há"),
        ("las", "lá"),
        ("mas", "má"),
        ("nas", "ná"),
    ]);
}

// ============================================================
// W CONSONANT TESTS
// Note: W in Telex has special behavior (vowel modifier: aw→ă, ow→ơ, uw→ư)
// When skip_w_shortcut is enabled, W at word start stays as 'w'
// ============================================================

/// Helper to run telex tests with foreign consonants AND skip_w_shortcut enabled
fn telex_foreign_with_w(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_allow_foreign_consonants(true);
        e.set_skip_w_shortcut(true); // Keep W as 'w' at word start
        let result = type_word(&mut e, input);
        assert_eq!(
            result, *expected,
            "[Telex ForeignConsonants+SkipW] '{}' → '{}'",
            input, result
        );
    }
}

#[test]
fn foreign_w_with_diacritics() {
    // W as initial consonant with Vietnamese diacritics
    // Requires both allow_foreign_consonants AND skip_w_shortcut to work
    telex_foreign_with_w(&[
        ("was", "wá"), // w + a + sắc
        ("waf", "wà"), // w + a + huyền
        ("war", "wả"), // w + a + hỏi
        ("wax", "wã"), // w + a + ngã
        ("waj", "wạ"), // w + a + nặng
    ]);
}

#[test]
fn foreign_w_with_full_syllables() {
    // W as initial consonant with complete syllables
    telex_foreign_with_w(&[
        ("wans", "wán"), // w + án
        ("wams", "wám"), // w + ám
        ("wacs", "wác"), // w + ác
        ("wats", "wát"), // w + át
    ]);
}

#[test]
fn no_foreign_w_becomes_u_horn() {
    // When foreign consonants is disabled, W at start becomes ư (default Telex behavior)
    telex_no_foreign(&[
        ("was", "ứa"), // w→ư, a, s→sắc on ư
        ("waf", "ừa"), // w→ư, a, f→huyền on ư
    ]);
}

// ============================================================
// FOREIGN CONSONANTS + ENGLISH AUTO-RESTORE COMPATIBILITY
// When both options are enabled, foreign consonants should NOT trigger auto-restore
// ============================================================

/// Helper to run telex tests with BOTH foreign consonants AND english auto-restore enabled
fn telex_foreign_with_auto_restore(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_allow_foreign_consonants(true);
        e.set_english_auto_restore(true);
        let result = type_word(&mut e, input);
        assert_eq!(
            result, *expected,
            "[Telex ForeignConsonants+AutoRestore] '{}' → '{}'",
            input, result
        );
    }
}

#[test]
fn foreign_with_auto_restore_no_conflict() {
    // When both options are enabled, words starting with foreign consonants
    // should get diacritics and NOT be auto-restored to English
    telex_foreign_with_auto_restore(&[
        ("zas", "zá"),  // z + á, should NOT restore to "zas"
        ("zaf", "zà"),  // z + à
        ("fas", "fá"),  // f + á
        ("jas", "já"),  // j + á
        ("zoos", "zố"), // z + ố
        ("foos", "fố"), // f + ố
    ]);
}

#[test]
fn foreign_full_syllable_with_auto_restore() {
    // Full syllables with foreign consonants should work with auto-restore enabled
    telex_foreign_with_auto_restore(&[
        ("zans", "zán"), // z + án
        ("fams", "fám"), // f + ám
        ("jacs", "jác"), // j + ác
    ]);
}
