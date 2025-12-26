use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

fn main() {
    let mut e = Engine::new();

    for (key, name) in [
        (keys::H, "H"),
        (keys::O, "o"),
        (keys::O, "o"),
        (keys::M, "m"),
    ] {
        let _ = e.on_key(key, false, false);
        println!("After {}: '{}'", name, e.get_buffer_string());
    }

    println!("Final: '{}'", e.get_buffer_string());
    println!("Expected: 'hôm'");
}
