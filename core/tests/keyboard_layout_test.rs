//! Keyboard Layout Independence Tests
//!
//! Verify engine processes characters (not keycodes) for DVORAK/Colemak support.
//! Uses `on_key_with_char()` to map OS-reported characters to QWERTY keycodes.

use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine};
use gonhanh_core::utils::char_to_key;

// ============================================================
// HELPER FUNCTIONS
// ============================================================

/// Type a character using on_key_with_char API (layout-independent)
/// This simulates how the OS sends us keystrokes with character codes
fn type_char(engine: &mut Engine, ch: char) {
    // The keycode doesn't matter - we use char_code for layout independence
    // Using 0 as keycode to emphasize that the keycode is ignored
    engine.on_key_with_char(0, ch.is_uppercase(), false, false, Some(ch));
}

/// Type a string using on_key_with_char API
fn type_string(engine: &mut Engine, s: &str) {
    for ch in s.chars() {
        type_char(engine, ch);
    }
}

/// Simulate QWERTY typing (keycode-based, for comparison)
fn type_qwerty(engine: &mut Engine, s: &str) -> String {
    let mut screen = String::new();
    for ch in s.chars() {
        let key = char_to_key(ch);
        if key == 255 {
            screen.push(ch);
            continue;
        }
        let result = engine.on_key(key, ch.is_uppercase(), false);
        if result.action == Action::Send as u8 {
            for _ in 0..result.backspace {
                screen.pop();
            }
            for i in 0..result.count as usize {
                if let Some(c) = char::from_u32(result.chars[i]) {
                    screen.push(c);
                }
            }
        } else {
            screen.push(ch);
        }
    }
    screen
}

// ============================================================
// QWERTY BASELINE TESTS
// ============================================================

#[test]
fn test_qwerty_baseline_simple() {
    let mut engine = Engine::default();

    // Simple 'a' + 's' should produce 'á' (a with sắc tone)
    type_string(&mut engine, "as");

    assert_eq!(
        engine.get_buffer_string(),
        "á",
        "QWERTY baseline: 'as' should produce 'á'"
    );
}

#[test]
fn test_qwerty_baseline_viet() {
    let mut engine = Engine::default();

    // "viets" should produce "viét" (with sắc tone on 'e')
    type_string(&mut engine, "viets");

    assert_eq!(
        engine.get_buffer_string(),
        "viét",
        "QWERTY baseline: 'viets' should produce 'viét'"
    );
}

#[test]
fn test_qwerty_baseline_circumflex() {
    let mut engine = Engine::default();

    // "aa" should produce "â" (a with circumflex)
    type_string(&mut engine, "aa");

    assert_eq!(
        engine.get_buffer_string(),
        "â",
        "QWERTY baseline: 'aa' should produce 'â'"
    );
}

#[test]
fn test_qwerty_baseline_all_tones() {
    // Test all tone markers
    let test_cases = [
        ("as", "á"), // sắc tone
        ("af", "à"), // huyền tone
        ("ar", "ả"), // hỏi tone
        ("ax", "ã"), // ngã tone
        ("aj", "ạ"), // nặng tone
    ];

    for (input, expected) in test_cases {
        let mut engine = Engine::default();
        type_string(&mut engine, input);
        assert_eq!(
            engine.get_buffer_string(),
            expected,
            "Input '{}' should produce '{}'",
            input,
            expected
        );
    }
}

// ============================================================
// LAYOUT INDEPENDENCE: CHARACTER-BASED API
// ============================================================

#[test]
fn test_character_api_same_as_qwerty() {
    // The character-based API should produce the same result as QWERTY keycodes
    let test_inputs = ["as", "viets", "aa", "aw", "ow"];

    for input in test_inputs {
        let mut engine_char = Engine::default();
        let mut engine_qwerty = Engine::default();

        type_string(&mut engine_char, input);
        let _qwerty_result = type_qwerty(&mut engine_qwerty, input);

        assert_eq!(
            engine_char.get_buffer_string(),
            engine_qwerty.get_buffer_string(),
            "Character API and QWERTY should match for input '{}'",
            input
        );
    }
}

#[test]
fn test_character_s_applies_sac_tone() {
    let mut engine = Engine::default();

    // Regardless of which physical key produces 's', it should apply sắc tone
    // On QWERTY: 's' is keycode 1
    // On DVORAK: 's' is keycode 41 (';' key on QWERTY)
    // Both should produce the same result when using on_key_with_char

    // Using keycode 0 (which is 'a' on QWERTY) but char 's'
    // This simulates DVORAK where physical 'a' key produces 's'
    engine.on_key_with_char(0, false, false, false, Some('a')); // 'a'
    engine.on_key_with_char(keys::A, false, false, false, Some('s')); // 's' (sắc tone)

    assert_eq!(
        engine.get_buffer_string(),
        "á",
        "Character 's' should apply sắc tone regardless of keycode"
    );
}

// ============================================================
// DVORAK SIMULATION TESTS
// ============================================================

/// DVORAK keyboard layout simulation
/// In DVORAK, keys are remapped but the OS sends us the correct characters
#[test]
fn test_dvorak_simulation_viet() {
    let mut engine = Engine::default();

    // DVORAK user types physical keys that produce characters 'v', 'i', 'e', 't', 's'
    // The physical key positions are different, but the characters are the same
    // The engine should process these as characters, not keycodes

    // Simulate DVORAK by passing wrong keycodes but correct char codes
    // DVORAK 'v' is on QWERTY '.' position (keycode 47)
    // DVORAK 'i' is on QWERTY 'g' position (keycode 5)
    // etc.

    for ch in ['v', 'i', 'e', 't', 's'] {
        engine.on_key_with_char(0, false, false, false, Some(ch));
    }

    assert_eq!(
        engine.get_buffer_string(),
        "viét",
        "DVORAK user typing 'viets' should produce 'viét'"
    );
}

#[test]
fn test_dvorak_tone_markers() {
    // In DVORAK, the tone marker keys are at different physical positions
    // 's' (sắc) is at QWERTY ';' position
    // 'f' (huyền) is at QWERTY 'y' position
    // 'r' (hỏi) is at QWERTY 'p' position
    // 'x' (ngã) is at QWERTY 'b' position
    // 'j' (nặng) is at QWERTY 'c' position

    let test_cases = [
        ("as", "á"), // sắc
        ("af", "à"), // huyền
        ("ar", "ả"), // hỏi
        ("ax", "ã"), // ngã
        ("aj", "ạ"), // nặng
    ];

    for (input, expected) in test_cases {
        let mut engine = Engine::default();

        // Simulate DVORAK by using keycode 0 but correct char codes
        for ch in input.chars() {
            engine.on_key_with_char(0, false, false, false, Some(ch));
        }

        assert_eq!(
            engine.get_buffer_string(),
            expected,
            "DVORAK tone marker test: '{}' should produce '{}'",
            input,
            expected
        );
    }
}

#[test]
fn test_dvorak_circumflex_modifiers() {
    // Test vowel modifiers in DVORAK context
    let test_cases = [
        ("aa", "â"), // a + a = â
        ("ee", "ê"), // e + e = ê
        ("oo", "ô"), // o + o = ô
    ];

    for (input, expected) in test_cases {
        let mut engine = Engine::default();

        for ch in input.chars() {
            engine.on_key_with_char(0, false, false, false, Some(ch));
        }

        assert_eq!(
            engine.get_buffer_string(),
            expected,
            "DVORAK circumflex test: '{}' should produce '{}'",
            input,
            expected
        );
    }
}

#[test]
fn test_dvorak_horn_and_breve() {
    // Test horn (ư, ơ) and breve (ă) modifiers
    let test_cases = [
        ("aw", "ă"), // a + w = ă (breve)
        ("ow", "ơ"), // o + w = ơ (horn)
        ("uw", "ư"), // u + w = ư (horn)
    ];

    for (input, expected) in test_cases {
        let mut engine = Engine::default();

        for ch in input.chars() {
            engine.on_key_with_char(0, false, false, false, Some(ch));
        }

        assert_eq!(
            engine.get_buffer_string(),
            expected,
            "DVORAK horn/breve test: '{}' should produce '{}'",
            input,
            expected
        );
    }
}

// ============================================================
// COLEMAK SIMULATION TESTS
// ============================================================

/// Colemak keyboard layout simulation
/// Similar to DVORAK, but with different key positions
#[test]
fn test_colemak_simulation_viet() {
    let mut engine = Engine::default();

    // Colemak user types 'viets'
    // Key positions are different from QWERTY but OS sends correct characters

    for ch in ['v', 'i', 'e', 't', 's'] {
        engine.on_key_with_char(0, false, false, false, Some(ch));
    }

    assert_eq!(
        engine.get_buffer_string(),
        "viét",
        "Colemak user typing 'viets' should produce 'viét'"
    );
}

#[test]
fn test_colemak_complex_word() {
    let mut engine = Engine::default();

    // Type "nghieeng" -> "nghiêng"
    for ch in "nghieeng".chars() {
        engine.on_key_with_char(0, false, false, false, Some(ch));
    }

    assert_eq!(
        engine.get_buffer_string(),
        "nghiêng",
        "Colemak: 'nghieeng' should produce 'nghiêng'"
    );
}

// ============================================================
// char_to_key() UTILITY FUNCTION TESTS
// ============================================================

#[test]
fn test_char_to_key_lowercase_letters() {
    // Test that char_to_key maps all lowercase letters correctly
    let expected_mappings = [
        ('a', keys::A),
        ('b', keys::B),
        ('c', keys::C),
        ('d', keys::D),
        ('e', keys::E),
        ('f', keys::F),
        ('g', keys::G),
        ('h', keys::H),
        ('i', keys::I),
        ('j', keys::J),
        ('k', keys::K),
        ('l', keys::L),
        ('m', keys::M),
        ('n', keys::N),
        ('o', keys::O),
        ('p', keys::P),
        ('q', keys::Q),
        ('r', keys::R),
        ('s', keys::S),
        ('t', keys::T),
        ('u', keys::U),
        ('v', keys::V),
        ('w', keys::W),
        ('x', keys::X),
        ('y', keys::Y),
        ('z', keys::Z),
    ];

    for (ch, expected_key) in expected_mappings {
        assert_eq!(
            char_to_key(ch),
            expected_key,
            "char_to_key('{}') should return keys::{}",
            ch,
            ch.to_uppercase()
        );
    }
}

#[test]
fn test_char_to_key_digits() {
    let expected_mappings = [
        ('0', keys::N0),
        ('1', keys::N1),
        ('2', keys::N2),
        ('3', keys::N3),
        ('4', keys::N4),
        ('5', keys::N5),
        ('6', keys::N6),
        ('7', keys::N7),
        ('8', keys::N8),
        ('9', keys::N9),
    ];

    for (ch, expected_key) in expected_mappings {
        assert_eq!(
            char_to_key(ch),
            expected_key,
            "char_to_key('{}') should return keys::N{}",
            ch,
            ch
        );
    }
}

#[test]
fn test_char_to_key_unknown_returns_255() {
    // Unknown characters should return 255
    assert_eq!(char_to_key('€'), 255, "Unknown char should return 255");
    assert_eq!(char_to_key('中'), 255, "Non-ASCII char should return 255");
    assert_eq!(char_to_key('á'), 255, "Accented char should return 255");
}

// ============================================================
// EDGE CASES AND SPECIAL CHARACTERS
// ============================================================

#[test]
fn test_uppercase_letters() {
    let mut engine = Engine::default();

    // Uppercase 'A' followed by lowercase 's' should produce 'Á'
    engine.on_key_with_char(0, true, false, false, Some('A'));
    engine.on_key_with_char(0, false, false, false, Some('s'));

    assert_eq!(
        engine.get_buffer_string(),
        "Á",
        "Uppercase 'A' + 's' should produce 'Á'"
    );
}

#[test]
fn test_mixed_case() {
    let mut engine = Engine::default();

    // "Viets" should produce "Viét"
    type_char(&mut engine, 'V');
    for ch in "iets".chars() {
        type_char(&mut engine, ch);
    }

    assert_eq!(
        engine.get_buffer_string(),
        "Viét",
        "Mixed case 'Viets' should produce 'Viét'"
    );
}

#[test]
fn test_numbers_passthrough() {
    let mut engine = Engine::default();

    // Numbers should work normally
    for ch in "123".chars() {
        engine.on_key_with_char(0, false, false, false, Some(ch));
    }

    // Numbers don't modify the buffer in the same way, check they don't break anything
    // The buffer might be empty or contain the numbers depending on engine behavior
    // This test ensures no panics or errors
}

#[test]
fn test_special_characters_dont_break_engine() {
    let mut engine = Engine::default();

    // Special characters should not cause panics
    let special_chars = ['@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '+', '='];

    for ch in special_chars {
        engine.on_key_with_char(0, false, false, false, Some(ch));
    }

    // Just ensure we didn't panic
}

// ============================================================
// CONSISTENCY TESTS
// ============================================================

#[test]
fn test_layout_independence_same_output() {
    // The core principle: same characters should produce same output
    // regardless of the keycode passed to on_key_with_char

    let test_cases = [
        ("as", "á"),
        ("viets", "viét"),
        ("aa", "â"),
        ("ddaay", "đây"),
        ("nghieeng", "nghiêng"),
    ];

    for (input, expected) in test_cases {
        // Test with keycode 0
        let mut engine1 = Engine::default();
        for ch in input.chars() {
            engine1.on_key_with_char(0, false, false, false, Some(ch));
        }

        // Test with different keycodes (simulating different layouts)
        let mut engine2 = Engine::default();
        for (i, ch) in input.chars().enumerate() {
            // Use different keycodes to simulate different layouts
            engine2.on_key_with_char((i as u16) * 7 % 50, false, false, false, Some(ch));
        }

        // Test with QWERTY keycodes
        let mut engine3 = Engine::default();
        for ch in input.chars() {
            let key = char_to_key(ch);
            engine3.on_key_with_char(key, false, false, false, Some(ch));
        }

        assert_eq!(
            engine1.get_buffer_string(),
            expected,
            "Layout test 1 (keycode 0): '{}' should produce '{}'",
            input,
            expected
        );
        assert_eq!(
            engine2.get_buffer_string(),
            expected,
            "Layout test 2 (varied keycodes): '{}' should produce '{}'",
            input,
            expected
        );
        assert_eq!(
            engine3.get_buffer_string(),
            expected,
            "Layout test 3 (QWERTY keycodes): '{}' should produce '{}'",
            input,
            expected
        );
    }
}

// ============================================================
// REAL-WORLD VIETNAMESE WORDS
// ============================================================

#[test]
fn test_common_vietnamese_words() {
    let test_cases = [
        ("xin", "xin"),
        ("chafo", "chào"),      // chào (hello)
        ("cams", "cám"),        // cám (thank - as in "cám ơn")
        ("own", "ơn"),          // ơn (favor)
        ("ddaays", "đấy"),      // đấy (there)
        ("dduwowfng", "đường"), // đường (road/sugar)
    ];

    for (input, expected) in test_cases {
        let mut engine = Engine::default();

        for ch in input.chars() {
            engine.on_key_with_char(0, false, false, false, Some(ch));
        }

        assert_eq!(
            engine.get_buffer_string(),
            expected,
            "Vietnamese word: '{}' should produce '{}'",
            input,
            expected
        );
    }
}

#[test]
fn test_vietnamese_sentence_fragments() {
    // Test that the engine handles word-by-word typing correctly
    // Note: This tests the buffer state, not the full sentence

    let mut engine = Engine::default();

    // Type "xinf chafof"
    for ch in "xinf".chars() {
        engine.on_key_with_char(0, false, false, false, Some(ch));
    }

    assert_eq!(
        engine.get_buffer_string(),
        "xìn",
        "First word 'xinf' should produce 'xìn'"
    );

    // Clear for next word
    engine.clear();

    for ch in "chafo".chars() {
        engine.on_key_with_char(0, false, false, false, Some(ch));
    }

    assert_eq!(
        engine.get_buffer_string(),
        "chào",
        "Second word 'chafo' should produce 'chào'"
    );
}
