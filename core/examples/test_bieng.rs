use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

fn main() {
    let mut e = Engine::new();
    e.set_english_auto_restore(true);

    // Test bieeng → biêng
    let result = type_word(&mut e, "bieeng ");
    println!("'bieeng ' → '{}'", result);

    e.clear();

    // Test tieng with tone
    let result2 = type_word(&mut e, "tiengs ");
    println!("'tiengs ' → '{}'", result2);

    e.clear();

    // Check if 'tiêng' (without tone) is valid
    let is_invalid = gonhanh_core::engine::matrix::is_buffer_invalid_vietnamese("tiêng");
    println!("is_buffer_invalid_vietnamese('tiêng') = {}", is_invalid);

    let is_invalid2 = gonhanh_core::engine::matrix::is_buffer_invalid_vietnamese("tiếng");
    println!("is_buffer_invalid_vietnamese('tiếng') = {}", is_invalid2);
}
