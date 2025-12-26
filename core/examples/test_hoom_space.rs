use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

fn main() {
    // Test with type_word helper
    let mut e = Engine::new();
    let result = type_word(&mut e, "hoom nay");
    println!("type_word result: '{}'", result);
    println!("Expected: 'hôm nay'");

    // Test manually
    let mut e2 = Engine::new();
    for (key, name) in [
        (keys::H, "H"),
        (keys::O, "o"),
        (keys::O, "o"),
        (keys::M, "m"),
    ] {
        let _ = e2.on_key(key, false, false);
        println!("After {}: '{}'", name, e2.get_buffer_string());
    }
    println!("Final before space: '{}'", e2.get_buffer_string());
}
