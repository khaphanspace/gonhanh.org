//! Debug test for "osa" → "oá" modern tone issue

use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_osa_modern_tone() {
    let mut engine = Engine::new();
    // Engine should have modern_tone = true by default

    // Type "osa"
    let keys_sequence = [keys::O, keys::S, keys::A];

    for (i, &key) in keys_sequence.iter().enumerate() {
        let r = engine.on_key_ext(key, false, false, false);
        let buf = engine.get_buffer_string();
        eprintln!(
            "[Step {}] Key: {:?}, action={}, buffer='{}'",
            i + 1,
            match key {
                keys::O => "O",
                keys::S => "S",
                keys::A => "A",
                _ => "?",
            },
            r.action,
            buf
        );
    }

    let final_buf = engine.get_buffer_string();
    eprintln!("Final buffer: '{}'", final_buf);

    // Expected: "oá" (modern tone - tone on 'a')
    // If we get "óa" that means modern tone is not working
    assert_eq!(
        final_buf, "oá",
        "Modern tone: 'osa' should produce 'oá' (tone on 'a'), got '{}'",
        final_buf
    );
}
