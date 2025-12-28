use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::{english_likelihood_keys, is_foreign_pattern_keys};

#[test]
fn debug_caaps() {
    // Test patterns that should NOT be flagged as foreign
    let test_cases = [
        ("caaps", vec![keys::C, keys::A, keys::A, keys::P, keys::S]), // cấp
        ("ddeso", vec![keys::D, keys::D, keys::E, keys::S, keys::O]), // đéo
        ("maxnh", vec![keys::M, keys::A, keys::X, keys::N, keys::H]), // mãnh
        ("tafoo", vec![keys::T, keys::A, keys::F, keys::O, keys::O]), // tàoo
        ("bawts", vec![keys::B, keys::A, keys::W, keys::T, keys::S]), // bắt
    ];

    for (name, keys) in test_cases {
        let is_foreign = is_foreign_pattern_keys(&keys);
        let en = english_likelihood_keys(&keys);
        eprintln!("'{}': foreign={}, EN(R)={:?}", name, is_foreign, en);
    }
}
