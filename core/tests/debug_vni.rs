// Run: cd /Users/khaphan/Documents/Work/gonhanh_2/core && cargo test --test debug_vni -- --nocapture
use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine};

#[test]
fn debug_vni_nguoi() {
    let mut e = Engine::new();
    e.set_method(1); // VNI

    // ngu7o72i2 -> người
    let test_keys = [
        (keys::N, "n"),
        (keys::G, "g"),
        (keys::U, "u"),
        (keys::N7, "7"), // horn
        (keys::O, "o"),
        (keys::N7, "7"), // horn
        (keys::N2, "2"), // grave
        (keys::I, "i"),
        (keys::N2, "2"), // grave? or is this i2?
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

    println!("Final buffer: '{}'", e.get_buffer_string());
    println!("Expected: 'người'");
}
