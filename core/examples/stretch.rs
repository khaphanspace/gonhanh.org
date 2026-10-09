//! Typing-experience audit: stretch the last vowel of every toned Vietnamese syllable
//! (mùa → mùaaaa) and count how often the tone moves. `... --example stretch -- [ar] < pairs`
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
use std::io::BufRead;

const TONED: &str = "áàảãạắằẳẵặấầẩẫậéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ";

fn main() {
    let ar = std::env::args().any(|a| a == "ar");
    let (mut n, mut bad, mut shown) = (0, 0, Vec::new());
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((typed, _)) = line.split_once('\t') else {
            continue;
        };
        let Some(v) = typed.chars().rev().find(|c| "aeiouy".contains(*c)) else {
            continue;
        };
        let screen = |t: &str| {
            let mut e = Engine::new();
            e.set_english_auto_restore(ar);
            type_word(&mut e, t)
        };
        let base = screen(typed);
        let Some(at) = base.chars().position(|c| TONED.contains(c)) else {
            continue;
        };
        n += 1;
        let long = screen(&format!("{typed}{v}{v}{v}{v}"));
        if long.chars().position(|c| TONED.contains(c)) != Some(at) {
            bad += 1;
            if shown.len() < 100000 {
                shown.push(format!("{typed}+{v}x4: {base}→{long}"));
            }
        }
    }
    println!("{n} toned syllables, {bad} move the tone when the last vowel is stretched");
    println!("{}", shown.join("\n"));
}
