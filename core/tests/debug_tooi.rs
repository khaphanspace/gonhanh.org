use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::{english_likelihood_keys, is_foreign_pattern_keys};
use gonhanh_core::engine::Engine;

#[test]
fn debug_tooi() {
    // Test what happens when typing "tooi "
    let key_seq = [keys::T, keys::O, keys::O, keys::I, keys::SPACE];

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

    // Also check the foreign pattern and EN likelihood
    let all_keys = vec![keys::T, keys::O, keys::O, keys::I];
    let is_foreign = is_foreign_pattern_keys(&all_keys);
    let en = english_likelihood_keys(&all_keys);
    eprintln!("'tooi': foreign={}, EN(R)={:?}", is_foreign, en);
}
