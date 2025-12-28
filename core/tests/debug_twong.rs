use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::{english_likelihood_keys, is_foreign_pattern_keys};

#[test]
fn debug_twong() {
    let test_cases = [
        ("twong", vec![keys::T, keys::W, keys::O, keys::N, keys::G]),
        ("maxnh", vec![keys::M, keys::A, keys::X, keys::N, keys::H]),
    ];

    for (name, keys) in test_cases {
        let is_foreign = is_foreign_pattern_keys(&keys);
        let en = english_likelihood_keys(&keys);
        eprintln!("'{}': foreign={}, EN(R)={:?}", name, is_foreign, en);
    }
}
