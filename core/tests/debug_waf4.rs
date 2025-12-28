//! Debug test for "waf" → "ừa" issue

use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

#[test]
fn debug_waf_step_by_step() {
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    let keys_sequence = [(keys::W, "W"), (keys::A, "A"), (keys::F, "F")];

    for (i, &(key, name)) in keys_sequence.iter().enumerate() {
        let r = engine.on_key_ext(key, false, false, false);
        let buf = engine.get_buffer_string();
        eprintln!(
            "[Step {}] Key: {:?}, action={}, buffer='{}'",
            i + 1,
            name,
            r.action,
            buf
        );
    }

    let final_buf = engine.get_buffer_string();
    eprintln!("Final buffer: '{}'", final_buf);

    // Expected: "ừa" (w→ư, a→a, f→huyền tone)
    assert_eq!(
        final_buf, "ừa",
        "Telex 'waf' should produce 'ừa', got '{}'",
        final_buf
    );
}

#[test]
fn debug_waf_with_space() {
    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    let result = type_word(&mut engine, "waf ");
    eprintln!("Result: '{}'", result);

    // Expected: "ừa " (Vietnamese word, not English)
    assert_eq!(
        result, "ừa ",
        "Telex 'waf ' should produce 'ừa ', got '{}'",
        result
    );
}
