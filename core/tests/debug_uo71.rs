use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

#[test]
fn debug_uo71() {
    println!("\n=== Testing VNI 'uo71' - detailed ===");
    let mut e = Engine::new();
    e.set_method(1); // VNI

    // Type u-o
    e.on_key(keys::U, false, false);
    e.on_key(keys::O, false, false);
    println!("After 'uo': '{}'", e.get_buffer_string());

    // Apply horn (7)
    e.on_key(keys::N7, false, false);
    let after_horn = e.get_buffer_string();
    println!("After '7' (horn): '{}'", after_horn);

    // Check each char's representation
    // ư is U+01B0 (449), ơ is U+01A1 (417)
    for (i, c) in after_horn.chars().enumerate() {
        println!("  char[{}] = '{}' (U+{:04X})", i, c, c as u32);
    }

    // Apply tone (1)
    e.on_key(keys::N1, false, false);
    let after_tone = e.get_buffer_string();
    println!("After '1' (sắc): '{}'", after_tone);

    // Check each char's representation
    // Expected: u (no tone) + ớ (o with horn + sắc)
    // ớ is U+1EDB (7899)
    // Actual: ứ (u with horn + sắc) + ơ (o with horn only)
    // ứ is U+1EE9 (7913)
    for (i, c) in after_tone.chars().enumerate() {
        println!("  char[{}] = '{}' (U+{:04X})", i, c, c as u32);
    }

    println!("\nExpected: 'uớ' = 'u' (U+0075) + 'ớ' (U+1EDB)");
    println!("Actual:   '{}' = ?", after_tone);
}
