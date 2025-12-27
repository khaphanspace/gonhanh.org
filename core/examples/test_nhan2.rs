use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

fn main() {
    let mut e = Engine::new();
    e.set_english_auto_restore(true);

    let result = type_word(&mut e, "nhanaj ");
    println!("'nhanaj ' → '{}'", result);

    e.clear();

    // Without auto_restore
    e.set_english_auto_restore(false);
    let result2 = type_word(&mut e, "nhanaj ");
    println!("'nhanaj ' (no auto restore) → '{}'", result2);
}
