//! Free typing audit: two Vietnamese words typed with no space between them
//! (xin + chào → xinchaof). `... --example runon -- [ar] [free] < pairs` (pairs: typing<TAB>word)
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
use std::io::BufRead;

fn main() {
    let flags: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| flags.iter().any(|x| x == f);
    let words: Vec<(String, String)> = std::io::stdin()
        .lock()
        .lines()
        .map_while(Result::ok)
        .filter_map(|l| {
            let (t, w) = l.split_once('\t')?;
            (t.chars().all(|c| c.is_ascii_alphabetic()) && t.len() >= 2 && !w.is_empty())
                .then(|| (t.to_string(), w.to_string()))
        })
        .collect();
    let (mut n, mut ok, mut shown) = (0, 0, Vec::new());
    let mut seed = 12345u64;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    for _ in 0..4000 {
        let (t1, w1) = &words[next() % words.len()];
        let (t2, w2) = &words[next() % words.len()];
        let mut e = Engine::new();
        e.set_english_auto_restore(has("ar"));
        if has("free") {
            e.set_free_tone(true);
            e.set_allow_foreign_consonants(true);
        }
        let got = type_word(&mut e, &format!("{t1}{t2}"));
        n += 1;
        if got == format!("{w1}{w2}") {
            ok += 1;
        } else if shown.len() < 30 {
            shown.push(format!("{t1}{t2}: {w1}{w2} ← {got}"));
        }
    }
    println!(
        "{n} pairs, {ok} typed correctly ({:.1}%)",
        100.0 * ok as f64 / n as f64
    );
    println!("{}", shown.join("\n"));
}
