use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_misslead() {
    // Test what happens when typing "misslead "
    let key_seq = [
        keys::M,
        keys::I,
        keys::S,
        keys::S,
        keys::L,
        keys::E,
        keys::A,
        keys::D,
        keys::SPACE,
    ];

    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    for key in key_seq {
        let result = engine.on_key(key, false, false);
        let output: String = result
            .chars
            .iter()
            .take(result.count as usize)
            .filter_map(|&c| char::from_u32(c))
            .collect();
        eprintln!(
            "Key {}: backspace={}, action={}, output='{}'",
            key, result.backspace, result.action, output
        );
    }
}
