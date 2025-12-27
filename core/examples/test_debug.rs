use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

fn main() {
    let mut e = Engine::new();
    e.set_english_auto_restore(true);

    // Test tiengs → should be tiếng
    let result = type_word(&mut e, "tiengs ");
    println!("'tiengs ' → '{}'", result);

    e.clear();

    // Test bieng (no tone) → should be biêng
    let result2 = type_word(&mut e, "bieeng ");
    println!("'bieeng ' → '{}'", result2);

    e.clear();

    // Test biengs (with tone) → should be biếng
    let result3 = type_word(&mut e, "biengs ");
    println!("'biengs ' → '{}'", result3);
}
