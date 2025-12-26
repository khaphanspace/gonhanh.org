// Run: cd /Users/khaphan/Documents/Work/gonhanh_2/core && cargo test --test debug_hoom -- --nocapture
use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine};

#[test]
fn debug_hoom() {
    let mut e = Engine::new();

    let test_keys = [
        (keys::H, "H"),
        (keys::O, "o"),
        (keys::O, "o"),
        (keys::M, "m"),
    ];

    for (i, &(key, name)) in test_keys.iter().enumerate() {
        let r = e.on_key_ext(key, false, false, false);
        let buffer = e.get_buffer_string();

        println!("Step {}: key={}", i, name);
        println!("  action={} (None=0, Send=1, Restore=2)", r.action);
        println!("  backspace={}", r.backspace);
        println!("  count={}", r.count);
        if r.count > 0 {
            let chars: String = (0..r.count as usize)
                .filter_map(|i| char::from_u32(r.chars[i]))
                .collect();
            println!("  chars='{}'", chars);
        }
        println!("  buffer='{}'", buffer);
        println!();
    }

    // Now simulate what type_word does
    let mut e2 = Engine::new();
    let mut screen = String::new();

    for &(key, name) in &test_keys {
        let r = e2.on_key_ext(key, false, false, false);

        if r.action == Action::Send as u8 {
            for _ in 0..r.backspace {
                screen.pop();
            }
            for i in 0..r.count as usize {
                if let Some(ch) = char::from_u32(r.chars[i]) {
                    screen.push(ch);
                }
            }
        } else {
            // Pass through
            screen.push(match key {
                keys::H => 'h',
                keys::O => 'o',
                keys::M => 'm',
                _ => '?',
            });
        }
        println!(
            "After {}: screen='{}', buffer='{}'",
            name,
            screen,
            e2.get_buffer_string()
        );
    }

    println!("\nFinal screen: '{}'", screen);
    println!("Expected: 'hôm'");
    assert_eq!(screen, "hôm");
}
