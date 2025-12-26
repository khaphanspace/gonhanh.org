// Run: cd /Users/khaphan/Documents/Work/gonhanh_2/core && cargo test --test debug_hoom_space -- --nocapture
use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine};
use gonhanh_core::utils::type_word;

#[test]
fn debug_hoom_space() {
    // Test "hoom nay" → "hôm nay"
    let mut e = Engine::new();
    let result = type_word(&mut e, "hoom nay");
    println!("type_word result: '{}'", result);
    println!("Expected: 'hôm nay'");

    // Debug step by step
    let mut e2 = Engine::new();
    let input = "hoom nay";
    let mut screen = String::new();

    for c in input.chars() {
        let key = match c {
            'h' => keys::H,
            'o' => keys::O,
            'm' => keys::M,
            'n' => keys::N,
            'a' => keys::A,
            'y' => keys::Y,
            ' ' => keys::SPACE,
            _ => continue,
        };

        let r = e2.on_key_ext(key, false, false, false);
        let buffer = e2.get_buffer_string();

        println!(
            "Key '{}': action={}, bs={}, cnt={}, buffer='{}', screen before='{}'",
            c, r.action, r.backspace, r.count, buffer, screen
        );

        if r.action == Action::Send as u8 || r.action == Action::Restore as u8 {
            for _ in 0..r.backspace {
                screen.pop();
            }
            for i in 0..r.count as usize {
                if let Some(ch) = char::from_u32(r.chars[i]) {
                    screen.push(ch);
                }
            }
            if key == keys::SPACE && r.action == Action::Restore as u8 {
                screen.push(' ');
            }
        } else if key == keys::SPACE {
            screen.push(' ');
        } else {
            screen.push(c);
        }

        println!("  screen after='{}'", screen);
    }

    println!("\nFinal screen: '{}'", screen);
    assert_eq!(screen, "hôm nay");
}
