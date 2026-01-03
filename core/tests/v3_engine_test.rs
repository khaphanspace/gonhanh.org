//! V3 Engine Tests
//!
//! Run v1 test cases against v3 engine adapter.
//! Usage: cargo test --features v3 --test v3_engine_test

#![cfg(feature = "v3")]

use gonhanh_core::data::keys;
use gonhanh_core::v3::Engine;

// Helper to type a word and get buffer result
fn type_word(e: &mut Engine, input: &str) -> String {
    for c in input.chars() {
        let key = char_to_key(c);
        let caps = c.is_uppercase();
        e.on_key(key, caps, false);
    }
    e.get_buffer_string()
}

fn char_to_key(c: char) -> u16 {
    match c.to_ascii_lowercase() {
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
        '0' => keys::N0,
        '1' => keys::N1,
        '2' => keys::N2,
        '3' => keys::N3,
        '4' => keys::N4,
        '5' => keys::N5,
        '6' => keys::N6,
        '7' => keys::N7,
        '8' => keys::N8,
        '9' => keys::N9,
        _ => keys::SPACE,
    }
}

// ============================================================
// BASIC TELEX TESTS
// ============================================================

#[test]
fn v3_telex_tone_sac() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "as");
    assert_eq!(result, "á", "V3: 'as' → 'á'");
}

#[test]
fn v3_telex_tone_huyen() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "af");
    assert_eq!(result, "à", "V3: 'af' → 'à'");
}

#[test]
fn v3_telex_tone_hoi() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "ar");
    assert_eq!(result, "ả", "V3: 'ar' → 'ả'");
}

#[test]
fn v3_telex_tone_nga() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "ax");
    assert_eq!(result, "ã", "V3: 'ax' → 'ã'");
}

#[test]
fn v3_telex_tone_nang() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "aj");
    assert_eq!(result, "ạ", "V3: 'aj' → 'ạ'");
}

// ============================================================
// CIRCUMFLEX TESTS
// ============================================================

#[test]
fn v3_telex_circumflex_a() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "aa");
    assert_eq!(result, "â", "V3: 'aa' → 'â'");
}

#[test]
fn v3_telex_circumflex_e() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "ee");
    assert_eq!(result, "ê", "V3: 'ee' → 'ê'");
}

#[test]
fn v3_telex_circumflex_o() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "oo");
    assert_eq!(result, "ô", "V3: 'oo' → 'ô'");
}

// ============================================================
// HORN TESTS
// ============================================================

#[test]
fn v3_telex_horn_o() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "ow");
    assert_eq!(result, "ơ", "V3: 'ow' → 'ơ'");
}

#[test]
fn v3_telex_horn_u() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "uw");
    assert_eq!(result, "ư", "V3: 'uw' → 'ư'");
}

// ============================================================
// BREVE TESTS
// ============================================================

#[test]
fn v3_telex_breve() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "aw");
    assert_eq!(result, "ă", "V3: 'aw' → 'ă'");
}

// ============================================================
// STROKE TESTS
// ============================================================

#[test]
fn v3_telex_stroke() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "dd");
    assert_eq!(result, "đ", "V3: 'dd' → 'đ'");
}

// ============================================================
// WORD TESTS
// ============================================================

#[test]
fn v3_telex_viet() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "vieejt");
    assert_eq!(result, "việt", "V3: 'vieejt' → 'việt'");
}

#[test]
fn v3_telex_nam() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "nam");
    assert_eq!(result, "nam", "V3: 'nam' → 'nam'");
}

#[test]
fn v3_telex_xin_chao() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "chaof");
    assert_eq!(result, "chào", "V3: 'chaof' → 'chào'");
}

#[test]
fn v3_telex_duong() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "dduowngf");
    assert_eq!(result, "đường", "V3: 'dduowngf' → 'đường'");
}

#[test]
fn v3_telex_nguoi() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "nguoiwf");
    assert_eq!(result, "người", "V3: 'nguoiwf' → 'người'");
}

// ============================================================
// REVERT TESTS
// ============================================================

#[test]
fn v3_telex_revert_tone() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "ass");
    assert_eq!(result, "as", "V3: 'ass' → 'as' (revert)");
}

#[test]
fn v3_telex_revert_circumflex() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "aaa");
    assert_eq!(result, "aa", "V3: 'aaa' → 'aa' (revert)");
}

#[test]
fn v3_telex_revert_stroke() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "ddd");
    assert_eq!(result, "dd", "V3: 'ddd' → 'dd' (revert)");
}

// ============================================================
// ENGINE STATE TESTS
// ============================================================

#[test]
fn v3_clear_resets_buffer() {
    let mut e = Engine::new();
    type_word(&mut e, "vieejt");
    e.clear();
    assert_eq!(e.get_buffer_string(), "", "V3: clear should reset buffer");
}

#[test]
fn v3_disabled_engine() {
    let mut e = Engine::new();
    e.set_enabled(false);
    let result = type_word(&mut e, "as");
    // When disabled, keys pass through without processing
    assert_eq!(result, "", "V3: disabled engine should not process");
}
