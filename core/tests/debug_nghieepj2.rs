use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

// Test with the test harness function
fn type_word(e: &mut Engine, input: &str) -> String {
    let mut result = String::new();

    for ch in input.chars() {
        let key = match ch {
            'a' => keys::A,
            'b' => keys::B,
            'c' => keys::C,
            'd' => keys::D,
            'e' => keys::E,
            'f' => keys::F,
            'g' => keys::G,
            'h' => keys::H,
            'i' => keys::I,
            'j' => keys::J,
            'k' => keys::K,
            'l' => keys::L,
            'm' => keys::M,
            'n' => keys::N,
            'o' => keys::O,
            'p' => keys::P,
            'q' => keys::Q,
            'r' => keys::R,
            's' => keys::S,
            't' => keys::T,
            'u' => keys::U,
            'v' => keys::V,
            'w' => keys::W,
            'x' => keys::X,
            'y' => keys::Y,
            'z' => keys::Z,
            ' ' => keys::SPACE,
            _ => continue,
        };

        let r = e.on_key_ext(key, false, false, false);
        let action = r.action;

        if action == 1 || action == 2 {
            // Send or Restore
            for _ in 0..r.backspace {
                result.pop();
            }
            for i in 0..r.count as usize {
                if let Some(c) = char::from_u32(r.chars[i]) {
                    result.push(c);
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[test]
fn debug_nghieepj2() {
    let mut e = Engine::new();
    e.set_english_auto_restore(true);

    // Test without space first
    let result1 = type_word(&mut e, "nghieepj");
    println!("'nghieepj' → '{}'", result1);

    // Reset and test with space
    let mut e2 = Engine::new();
    e2.set_english_auto_restore(true);
    let result2 = type_word(&mut e2, "nghieepj ");
    println!("'nghieepj ' → '{}'", result2);

    println!("\nExpected with space: 'nghiệp '");
}
