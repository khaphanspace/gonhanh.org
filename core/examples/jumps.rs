//! Typing-experience audit: how often does an existing tone mark move to another vowel while more
//! letters are typed? `cargo run --release --features engine_v2 --example jumps -- [ar] [free] < words`
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
use std::io::BufRead;

fn main() {
    let flags: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| flags.iter().any(|x| x == f);
    let (mut words, mut jumpy, mut shown) = (0, 0, Vec::new());
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let w = line.trim().to_lowercase();
        if w.is_empty() || !w.is_ascii() {
            continue;
        }
        words += 1;
        let mut prev: Option<usize> = None;
        let mut bad = false;
        for k in 1..=w.len() {
            let mut e = Engine::new();
            e.set_english_auto_restore(has("ar"));
            if has("free") {
                e.set_free_tone(true);
                e.set_allow_foreign_consonants(true);
            }
            let s = type_word(&mut e, &w[..k]);
            let at = s.chars().position(|c| !c.is_ascii());
            if let (Some(a), Some(b)) = (prev, at) {
                if a != b {
                    bad = true;
                }
            }
            prev = at.or(prev);
        }
        if bad {
            jumpy += 1;
            if shown.len() < 25 {
                shown.push(w);
            }
        }
    }
    println!(
        "{words} words, {jumpy} with a tone that moved\n{}",
        shown.join(" ")
    );
}
