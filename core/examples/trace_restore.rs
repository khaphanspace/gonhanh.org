use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine};

fn char_to_key(c: char) -> u16 {
    match c.to_ascii_lowercase() {
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
        _ => 255,
    }
}

fn main() {
    let mut e = Engine::new();

    // Simulate restore_word("việt")
    e.restore_word("việt");
    let mut screen = String::from("việt");

    println!("After restore_word('việt'):");
    println!("  screen = '{}'", screen);

    // Type "nam"
    for c in "nam".chars() {
        let key = char_to_key(c);
        let is_caps = c.is_uppercase();
        let r = e.on_key(key, is_caps, false);

        print!(
            "'{}'(key={}) → action={}, bs={}, count={}, chars=[",
            c, key, r.action, r.backspace, r.count
        );
        for i in 0..r.count as usize {
            if let Some(ch) = char::from_u32(r.chars[i]) {
                print!("{}", ch);
            }
        }
        print!("]");

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
            screen.push(c);
        }
        println!(" → screen: '{}'", screen);
    }

    println!("\nFinal result: '{}'", screen);
}
