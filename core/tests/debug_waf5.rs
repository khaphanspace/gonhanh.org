//! Debug test for "waf" auto-restore detection

use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::validation::is_foreign_pattern_keys;

#[test]
fn debug_waf_pattern() {
    let keys_seq = [keys::W, keys::A, keys::F];
    let is_foreign = is_foreign_pattern_keys(&keys_seq);
    eprintln!("is_foreign_pattern_keys([W,A,F]) = {}", is_foreign);
    assert!(!is_foreign, "waf should NOT be detected as foreign pattern");
}
