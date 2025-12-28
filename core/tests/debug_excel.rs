use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::{
    english_likelihood_keys, is_buffer_invalid_vietnamese, is_foreign_pattern_keys,
};
use gonhanh_core::engine::Engine;

#[test]
fn debug_excel() {
    // Test what happens when typing "excel "
    let key_seq = [keys::E, keys::X, keys::C, keys::E, keys::L, keys::SPACE];

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

    // Check validations
    let all_keys = vec![keys::E, keys::X, keys::C, keys::E, keys::L];
    let is_foreign = is_foreign_pattern_keys(&all_keys);
    let en = english_likelihood_keys(&all_keys);
    eprintln!("\n'excel': VN(R)foreign={}, EN(R)={:?}", is_foreign, en);

    // Check buffer validation
    eprintln!(
        "is_buffer_invalid_vietnamese('ễcl')={}",
        is_buffer_invalid_vietnamese("ễcl")
    );
}
