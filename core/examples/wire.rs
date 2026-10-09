//! What the platform layer must inject: for the same typing, how many replacements, backspaces
//! and characters does the engine ask for? `cargo run --release --example wire [ar] < pairs`
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::char_to_key;
use std::io::BufRead;

fn main() {
    let ar = std::env::args().any(|a| a == "ar");
    let free = std::env::args().any(|a| a == "free");
    let (mut words, mut keys, mut sends, mut bs, mut chars, mut max_bs) =
        (0usize, 0usize, 0usize, 0usize, 0usize, 0u8);
    let (mut mid, mut at_space) = (0usize, 0usize);
    let mut gaps = [0usize; 6]; // replacements per key position tail: how many keys have bs 0,1,2,3,4,5+
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let typed = line.split('\t').next().unwrap_or("");
        if typed.is_empty() || !typed.chars().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        words += 1;
        let mut e = Engine::new();
        e.set_english_auto_restore(ar);
        if free {
            e.set_free_tone(true);
            e.set_allow_foreign_consonants(true);
        }
        for c in typed.chars().chain(std::iter::once(' ')) {
            let r = e.on_key_ext(char_to_key(c), false, false, false);
            keys += 1;
            if r.action != 0 {
                if c == ' ' {
                    at_space += 1;
                } else {
                    mid += 1;
                }
                sends += 1;
                bs += r.backspace as usize;
                chars += r.count as usize;
                max_bs = max_bs.max(r.backspace);
            }
            gaps[(r.backspace as usize).min(5)] += 1;
        }
    }
    println!(
        "{words} words, {keys} keys: {sends} replacements ({:.1}% of keys), {:.2} backspaces and {:.2} chars per replacement, longest {max_bs} backspaces",
        100.0 * sends as f64 / keys as f64,
        bs as f64 / sends.max(1) as f64,
        chars as f64 / sends.max(1) as f64
    );
    println!("replacements inside a word: {mid}, at the space: {at_space}");
    println!(
        "keys by backspaces asked: 0:{} 1:{} 2:{} 3:{} 4:{} 5+:{}",
        gaps[0], gaps[1], gaps[2], gaps[3], gaps[4], gaps[5]
    );
}
