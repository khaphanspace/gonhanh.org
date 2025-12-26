use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine};

#[test]
fn debug_dede() {
    println!("=== Testing 'dede' ===");
    let mut e = Engine::new();
    let chars = [
        (keys::D, 'd'),
        (keys::E, 'e'),
        (keys::D, 'd'),
        (keys::E, 'e'),
    ];

    let mut screen = String::new();
    for (key, ch) in chars {
        let r = e.on_key_ext(key, false, false, false);
        let buffer = e.get_buffer_string();
        println!(
            "Key '{}': action={}, bs={}, cnt={}, buffer='{}'",
            ch, r.action, r.backspace, r.count, buffer
        );

        if r.action == Action::Send as u8 || r.action == Action::Restore as u8 {
            for _ in 0..r.backspace {
                screen.pop();
            }
            for i in 0..r.count as usize {
                if let Some(c) = char::from_u32(r.chars[i]) {
                    screen.push(c);
                }
            }
        } else {
            screen.push(ch);
        }
        println!("  screen: '{}'", screen);
    }
    println!("\n=== Testing 'dedicated' ===");
    let mut e2 = Engine::new();
    let chars2 = [
        (keys::D, 'd'),
        (keys::E, 'e'),
        (keys::D, 'd'),
        (keys::I, 'i'),
        (keys::C, 'c'),
        (keys::A, 'a'),
        (keys::T, 't'),
        (keys::E, 'e'),
        (keys::D, 'd'),
    ];

    let mut screen2 = String::new();
    for (key, ch) in chars2 {
        let r = e2.on_key_ext(key, false, false, false);
        let buffer = e2.get_buffer_string();
        println!(
            "Key '{}': action={}, bs={}, cnt={}, buffer='{}'",
            ch, r.action, r.backspace, r.count, buffer
        );

        if r.action == Action::Send as u8 || r.action == Action::Restore as u8 {
            for _ in 0..r.backspace {
                screen2.pop();
            }
            for i in 0..r.count as usize {
                if let Some(c) = char::from_u32(r.chars[i]) {
                    screen2.push(c);
                }
            }
        } else {
            screen2.push(ch);
        }
        println!("  screen: '{}'", screen2);
    }
    println!("\nFinal 'dede': '{}'", screen);
    println!("Final 'dedicated': '{}'", screen2);
}
