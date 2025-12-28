use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::{is_valid_final_1, is_valid_final_2};
use gonhanh_core::engine::Engine;

#[test]
fn debug_maxnh() {
    // Test what happens when typing "maxnh "
    let key_seq = [keys::M, keys::A, keys::X, keys::N, keys::H, keys::SPACE];

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

    // Test validity of finals
    eprintln!("\nFinal validity tests:");
    eprintln!("is_valid_final_1(N): {}", is_valid_final_1(keys::N));
    eprintln!(
        "is_valid_final_2(N,H): {}",
        is_valid_final_2(keys::N, keys::H)
    );
}
