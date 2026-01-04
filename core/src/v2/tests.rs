//! V2 Engine Tests

#[cfg(test)]
mod buffer_state_tests {
    use super::super::state::*;

    #[test]
    fn test_initial_state() {
        let state = BufferState::new();
        assert!(!state.had_transform());
        assert!(!state.has_tone());
        assert!(!state.has_mark());
        assert!(!state.has_stroke());
        assert!(!state.pending_breve());
        assert_eq!(state.vn_state(), VnState::Unknown);
    }

    #[test]
    fn test_bitmask_operations() {
        let mut state = BufferState::new();

        state.set_had_transform(true);
        assert!(state.had_transform());

        state.set_has_tone(true);
        assert!(state.has_tone());

        state.set_vn_state(VnState::Complete);
        assert_eq!(state.vn_state(), VnState::Complete);
    }

    #[test]
    fn test_clear() {
        let mut state = BufferState::new();
        state.set_had_transform(true);
        state.set_has_tone(true);
        state.clear();
        assert!(!state.had_transform());
        assert!(!state.has_tone());
    }
}

#[cfg(test)]
mod classify_tests {
    use super::super::classify::*;
    use super::super::types::*;

    #[test]
    fn test_telex_tones() {
        let method = Method::Telex;
        assert!(matches!(classify_key(b's', None, method), KeyType::Tone(_)));
        assert!(matches!(classify_key(b'f', None, method), KeyType::Tone(_)));
        assert!(matches!(classify_key(b'r', None, method), KeyType::Tone(_)));
        assert!(matches!(classify_key(b'x', None, method), KeyType::Tone(_)));
        assert!(matches!(classify_key(b'j', None, method), KeyType::Tone(_)));
    }

    #[test]
    fn test_telex_marks() {
        let method = Method::Telex;
        // dd → stroke
        assert!(matches!(
            classify_key(b'd', Some(b'd'), method),
            KeyType::Mark(MarkType::Stroke)
        ));
        // w → horn/breve
        assert!(matches!(
            classify_key(b'w', None, method),
            KeyType::Mark(MarkType::HornOrBreve)
        ));
    }

    #[test]
    fn test_vni_tones() {
        let method = Method::Vni;
        assert!(matches!(classify_key(b'1', None, method), KeyType::Tone(1)));
        assert!(matches!(classify_key(b'2', None, method), KeyType::Tone(2)));
        assert!(matches!(classify_key(b'5', None, method), KeyType::Tone(5)));
    }
}

#[cfg(test)]
mod precheck_tests {
    use super::super::precheck::*;

    #[test]
    fn test_tier1_foreign_initials() {
        assert_eq!(pre_check("f"), Mode::Foreign);
        assert_eq!(pre_check("j"), Mode::Foreign);
        assert_eq!(pre_check("w"), Mode::Foreign);
        assert_eq!(pre_check("z"), Mode::Foreign);
    }

    #[test]
    fn test_tier2_onset_clusters() {
        assert_eq!(pre_check("bl"), Mode::Foreign);
        assert_eq!(pre_check("cl"), Mode::Foreign);
        assert_eq!(pre_check("st"), Mode::Foreign);
        assert_eq!(pre_check("pr"), Mode::Foreign);
    }

    #[test]
    fn test_valid_vietnamese_onsets() {
        assert_eq!(pre_check("ch"), Mode::Vietnamese);
        assert_eq!(pre_check("tr"), Mode::Vietnamese);
        assert_eq!(pre_check("ng"), Mode::Vietnamese);
        assert_eq!(pre_check("ph"), Mode::Vietnamese);
    }
}

#[cfg(test)]
mod validation_tests {
    use super::super::state::VnState;
    use super::super::validate::*;

    #[test]
    fn test_empty_buffer() {
        assert_eq!(validate_vn(""), VnState::Incomplete);
    }

    #[test]
    fn test_valid_syllables() {
        assert_eq!(validate_vn("ba"), VnState::Complete);
        assert_eq!(validate_vn("an"), VnState::Complete);
        assert_eq!(validate_vn("cha"), VnState::Complete);
        assert_eq!(validate_vn("nghia"), VnState::Complete);
    }

    #[test]
    fn test_invalid_onset_clusters() {
        assert_eq!(validate_vn("cla"), VnState::Impossible);
        assert_eq!(validate_vn("bla"), VnState::Impossible);
    }

    #[test]
    fn test_spelling_rules() {
        // c → k before e/i
        assert_eq!(validate_vn("ce"), VnState::Impossible);
        assert_eq!(validate_vn("ci"), VnState::Impossible);
        assert_eq!(validate_vn("ke"), VnState::Complete);
        assert_eq!(validate_vn("ki"), VnState::Complete);

        // g → gh before e/i
        assert_eq!(validate_vn("ge"), VnState::Impossible);
        assert_eq!(validate_vn("ghe"), VnState::Complete);
    }
}

#[cfg(test)]
mod restore_tests {
    use super::super::restore::*;
    use super::super::state::*;

    #[test]
    fn test_no_transform_keeps() {
        let state = BufferState::new();
        assert_eq!(should_restore(&state, "test", "test", None), Decision::Keep);
    }

    #[test]
    fn test_stroke_keeps() {
        let mut state = BufferState::new();
        state.set_had_transform(true);
        state.set_has_stroke(true);
        assert_eq!(should_restore(&state, "dde", "đe", None), Decision::Keep);
    }

    #[test]
    fn test_english_patterns() {
        assert!(is_english("text")); // tier3 coda
        assert!(is_english("search")); // tier4 vowel
        assert!(is_english("action")); // tier5 suffix
        assert!(!is_english("ban"));
    }
}

#[cfg(test)]
mod output_tests {
    use super::super::output::*;

    #[test]
    fn test_add_tone() {
        let (bs, commit) = generate_output("ba", "bá");
        assert_eq!(bs, 1);
        assert_eq!(commit, "á");
    }

    #[test]
    fn test_add_letter() {
        let (bs, commit) = generate_output("ba", "ban");
        assert_eq!(bs, 0);
        assert_eq!(commit, "n");
    }

    #[test]
    fn test_revert() {
        let (bs, commit) = generate_output("bá", "bas");
        assert_eq!(bs, 1);
        assert_eq!(commit, "as");
    }

    #[test]
    fn test_complete_replace() {
        let (bs, commit) = generate_output("abc", "xyz");
        assert_eq!(bs, 3);
        assert_eq!(commit, "xyz");
    }
}

#[cfg(test)]
mod dict_tests {
    use super::super::dict::*;

    #[test]
    fn test_bloom_filter_basic() {
        let dict = Dict::from_words(&["hello", "world", "test"]);
        assert!(dict.contains("hello"));
        assert!(dict.contains("world"));
        assert!(dict.contains("test"));
    }

    #[test]
    fn test_bloom_filter_negative() {
        let dict = Dict::from_words(&["hello", "world"]);
        // Random string should likely not match
        assert!(!dict.contains("xyzabcdef123"));
    }

    #[test]
    fn test_case_insensitive() {
        let dict = Dict::from_words(&["hello"]);
        assert!(dict.contains("HELLO"));
        assert!(dict.contains("Hello"));
    }
}

#[cfg(test)]
mod engine_tests {
    use super::super::engine::*;

    #[test]
    fn test_engine_new() {
        let engine = Engine::new();
        // Engine starts enabled by default
        assert!(engine.is_enabled());
    }

    #[test]
    fn test_engine_enable_disable() {
        let mut engine = Engine::new();
        engine.set_enabled(false);
        assert!(!engine.is_enabled());
        engine.set_enabled(true);
        assert!(engine.is_enabled());
    }

    #[test]
    fn test_engine_clear() {
        let mut engine = Engine::new();
        // Process some keys
        engine.on_key(0x00, false, false); // 'a'
        engine.clear();
        // Buffer should be empty after clear
    }

    #[test]
    fn test_engine_method_switch() {
        let mut engine = Engine::new();
        engine.set_method(1); // VNI
        engine.set_method(0); // Telex
    }

    use crate::data::keys;

    /// Convert ASCII char to macOS virtual keycode
    fn char_to_keycode(ch: char) -> u16 {
        match ch.to_ascii_lowercase() {
            'a' => keys::A,
            'b' => keys::B,
            'c' => keys::C,
            'd' => keys::D,
            'e' => keys::E,
            'f' => keys::F,
            'g' => keys::G,
            'h' => keys::H,
            'i' => keys::I,
            'j' => keys::J,
            'k' => keys::K,
            'l' => keys::L,
            'm' => keys::M,
            'n' => keys::N,
            'o' => keys::O,
            'p' => keys::P,
            'q' => keys::Q,
            'r' => keys::R,
            's' => keys::S,
            't' => keys::T,
            'u' => keys::U,
            'v' => keys::V,
            'w' => keys::W,
            'x' => keys::X,
            'y' => keys::Y,
            'z' => keys::Z,
            ' ' => keys::SPACE,
            _ => 0xFFFF, // Invalid keycode
        }
    }

    /// Helper to simulate typing a string
    fn type_str(engine: &mut Engine, s: &str) -> String {
        let mut result = String::new();
        for ch in s.chars() {
            let key = char_to_keycode(ch);
            if key == 0xFFFF {
                continue; // Skip unknown chars
            }
            let r = engine.on_key(key, false, false);
            // Apply backspaces
            for _ in 0..r.backspace {
                result.pop();
            }
            // Add new chars
            for i in 0..r.count as usize {
                if let Some(c) = char::from_u32(r.chars[i]) {
                    result.push(c);
                }
            }
        }
        result
    }

    #[test]
    fn test_duoc_tone_placement() {
        // Test "được" - tone should be on ơ, not ư (ươ pattern → second vowel)
        let mut engine = Engine::new();

        // With đ: dduwowcj → được
        let result = type_str(&mut engine, "dduwowcj");
        assert_eq!(result, "được", "dduwowcj should produce được");

        // Without đ (different order): duwowjc → dược (no stroke, tone still on ợ)
        engine.clear();
        let result2 = type_str(&mut engine, "duwowjc");
        assert_eq!(result2, "dược", "duwowjc should place tone on ợ not ụ");

        // Verify tone is NOT on first vowel (which would be dụơc)
        assert_ne!(result2, "dụơc", "tone should NOT be on first vowel");
    }

    #[test]
    fn test_modern_tone_placement_oa() {
        // Modern style (default): hoàf -> hoà (tone on second vowel)
        let mut engine = Engine::new();
        engine.set_modern_tone(true);
        let result = type_str(&mut engine, "hoaf");
        assert_eq!(result, "hoà", "modern style: oa -> oà");

        // Traditional style: hoaf -> hòa (tone on first vowel)
        engine.clear();
        engine.set_modern_tone(false);
        let result = type_str(&mut engine, "hoaf");
        assert_eq!(result, "hòa", "traditional style: oa -> òa");
    }

    #[test]
    fn test_modern_tone_placement_oe() {
        // Modern style: khoẻ (tone on second vowel)
        let mut engine = Engine::new();
        engine.set_modern_tone(true);
        let result = type_str(&mut engine, "khoer");
        assert_eq!(result, "khoẻ", "modern style: oe -> oẻ");

        // Traditional style: khỏe (tone on first vowel)
        engine.clear();
        engine.set_modern_tone(false);
        let result = type_str(&mut engine, "khoer");
        assert_eq!(result, "khỏe", "traditional style: oe -> ỏe");
    }

    #[test]
    fn test_modern_tone_placement_uy() {
        // Modern style: thuỳ (tone on second vowel)
        let mut engine = Engine::new();
        engine.set_modern_tone(true);
        let result = type_str(&mut engine, "thuyf");
        assert_eq!(result, "thuỳ", "modern style: uy -> uỳ");

        // Traditional style: thùy (tone on first vowel)
        engine.clear();
        engine.set_modern_tone(false);
        let result = type_str(&mut engine, "thuyf");
        assert_eq!(result, "thùy", "traditional style: uy -> ùy");
    }
}

#[cfg(test)]
mod buffer_tests {
    use super::super::buffer::*;

    #[test]
    fn test_buffer_new() {
        let buffer = Buffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.raw_len(), 0);
    }

    #[test]
    fn test_buffer_push() {
        let mut buffer = Buffer::new();
        buffer.push_raw('a');
        assert!(!buffer.is_empty());
        assert_eq!(buffer.raw_len(), 1);
    }

    #[test]
    fn test_buffer_clear() {
        let mut buffer = Buffer::new();
        buffer.push_raw('a');
        buffer.push_raw('b');
        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_buffer_raw_str() {
        let mut buffer = Buffer::new();
        buffer.push_raw('a');
        buffer.push_raw('b');
        buffer.push_raw('c');
        assert_eq!(buffer.raw(), "abc");
    }

    #[test]
    fn test_buffer_pop() {
        let mut buffer = Buffer::new();
        buffer.push_raw('a');
        buffer.push_raw('b');
        assert_eq!(buffer.pop_raw(), Some('b'));
        assert_eq!(buffer.pop_raw(), Some('a'));
        assert_eq!(buffer.pop_raw(), None);
    }
}

#[cfg(test)]
mod placement_tests {
    use super::super::placement::*;

    #[test]
    fn test_find_tone_position_single_vowel() {
        // Create vowel info for "ba"
        let vowels = vec![VowelInfo {
            position: 1,
            vowel: 'a',
            has_modifier: false,
        }];
        let pos = find_tone_position(&vowels, false);
        assert_eq!(pos, Some(1)); // Position of vowel in string
    }

    #[test]
    fn test_find_tone_position_diphthong() {
        // Create vowel info for "bai"
        let vowels = vec![
            VowelInfo {
                position: 1,
                vowel: 'a',
                has_modifier: false,
            },
            VowelInfo {
                position: 2,
                vowel: 'i',
                has_modifier: false,
            },
        ];
        let pos = find_tone_position(&vowels, false);
        assert!(pos.is_some());
    }

    #[test]
    fn test_find_tone_position_no_vowel() {
        let vowels: Vec<VowelInfo> = vec![];
        let pos = find_tone_position(&vowels, false);
        assert!(pos.is_none());
    }
}

#[cfg(test)]
mod bitmask_tests {
    use super::super::bitmask::*;

    #[test]
    fn test_is_vn_vowel() {
        assert!(is_vn_vowel('a'));
        assert!(is_vn_vowel('e'));
        assert!(is_vn_vowel('i'));
        assert!(is_vn_vowel('o'));
        assert!(is_vn_vowel('u'));
        assert!(is_vn_vowel('y'));
        assert!(is_vn_vowel('ă'));
        assert!(is_vn_vowel('â'));
        assert!(is_vn_vowel('ê'));
        assert!(is_vn_vowel('ô'));
        assert!(is_vn_vowel('ơ'));
        assert!(is_vn_vowel('ư'));
        assert!(!is_vn_vowel('b'));
        assert!(!is_vn_vowel('c'));
    }

    #[test]
    fn test_get_base_vowel() {
        // get_base_vowel returns char, not Option<char>
        assert_eq!(get_base_vowel('á'), 'a');
        assert_eq!(get_base_vowel('à'), 'a');
        assert_eq!(get_base_vowel('ả'), 'a');
        assert_eq!(get_base_vowel('ã'), 'a');
        assert_eq!(get_base_vowel('ạ'), 'a');
        assert_eq!(get_base_vowel('a'), 'a');
        assert_eq!(get_base_vowel('b'), 'b'); // Non-vowel returns itself
    }

    #[test]
    fn test_get_tone() {
        assert_eq!(get_tone('a'), 0); // no tone
        assert_eq!(get_tone('á'), 1); // sac
        assert_eq!(get_tone('à'), 2); // huyen
        assert_eq!(get_tone('ả'), 3); // hoi
        assert_eq!(get_tone('ã'), 4); // nga
        assert_eq!(get_tone('ạ'), 5); // nang
    }

    #[test]
    fn test_char_idx() {
        // char_idx returns usize, not Option
        let idx_a = char_idx('a');
        let idx_e = char_idx('e');
        assert_eq!(idx_a, 0);
        assert_eq!(idx_e, 4);
        assert_ne!(idx_a, idx_e);
    }
}
