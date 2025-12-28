use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_wwax2() {
    // Test "wwax" with full debug
    let keys_seq = [keys::W, keys::W, keys::A, keys::X];

    let mut engine = Engine::new();
    engine.set_english_auto_restore(true);

    // Type all keys
    for &k in &keys_seq {
        engine.on_key(k, false, false);
    }

    // Space to commit - this should trigger try_english_restore
    eprintln!("About to press SPACE...");
    let r = engine.on_key(keys::SPACE, false, false);

    // Print what we got
    let output: String = (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect();
    eprintln!(
        "Result: backspace={}, count={}, output='{}'",
        r.backspace, r.count, output
    );
    eprintln!("Expected output: 'wax '");
}
