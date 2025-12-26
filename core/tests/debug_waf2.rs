use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine};

#[test]
fn debug_waf_auto_restore() {
    println!("\n=== Testing 'waf ' with english_auto_restore ===");
    let mut e = Engine::new();
    e.set_english_auto_restore(true);
    let chars = [
        (keys::W, 'w'),
        (keys::A, 'a'),
        (keys::F, 'f'),
        (keys::SPACE, ' '),
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
    println!("\nExpected: 'ừa '");
    println!("Actual:   '{}'", screen);
}
