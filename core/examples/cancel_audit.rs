//! English words typed the way a user cancels a tone by hand: the tone key doubled at its place
//! (dis + s + connect). Prints words that do not come back as spelled.
//! `cargo run --release --example cancel_audit -- [free] < words.txt`
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
use std::io::Read;

fn main() {
    let free = std::env::args().any(|a| a == "free");
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text).unwrap();
    let mut e = Engine::new();
    e.set_english_auto_restore(true);
    e.set_free_tone(free);
    let (mut total, mut bad) = (0, 0);
    for w in text
        .lines()
        .map(str::trim)
        .filter(|w| w.len() >= 4 && w.chars().all(|c| c.is_ascii_lowercase()))
    {
        let b = w.as_bytes();
        for i in 1..b.len() {
            if !matches!(b[i], b's' | b'f' | b'r' | b'x' | b'j') || b[i] == b[i - 1] {
                continue;
            }
            // only where the first press really toned something; otherwise the pair is literal
            let before = type_word(&mut e, &w[..i]);
            e.clear();
            let after = type_word(&mut e, &w[..=i]);
            e.clear();
            // the press must tone the vowel just before it, on a word still plain until then
            if !before.is_ascii()
                || after.is_ascii()
                || !matches!(b[i - 1], b'a' | b'e' | b'i' | b'o' | b'u' | b'y')
            {
                continue;
            }
            let typed = format!("{}{}{} ", &w[..=i], b[i] as char, &w[i + 1..]);
            total += 1;
            let got = type_word(&mut e, &typed);
            if got != format!("{w} ") {
                bad += 1;
                println!("{typed:?}\t{}\twant {w}", got.trim_end());
            }
        }
    }
    eprintln!("{bad} of {total} differ");
}
