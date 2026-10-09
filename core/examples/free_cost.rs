//! What free typing costs on English words: CPU per key, and what the user sees (rewrites).
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::char_to_key;
extern "C" {
    fn getrusage(who: i32, usage: *mut i64) -> i32;
}
fn cpu_ns() -> f64 {
    let mut b = [0i64; 36];
    unsafe { getrusage(0, b.as_mut_ptr()) };
    b[0] as f64 * 1e9 + (b[1] & 0xFFFF_FFFF) as f64 * 1e3
}
fn main() {
    let words: Vec<String> = std::fs::read_to_string("src/data/english_dict_merged.txt")
        .unwrap()
        .lines()
        .filter(|w| w.is_ascii() && w.len() > 1)
        .map(|w| format!("{w} "))
        .collect();
    let keys: usize = words.iter().map(|w| w.len()).sum();
    for (name, free, ar) in [
        ("normal AR", false, true),
        ("free AR", true, true),
        ("normal", false, false),
        ("free", true, false),
    ] {
        let (mut rewrites, mut words_with, mut bs, mut worst) = (0usize, 0usize, 0usize, 0f64);
        let mut e = Engine::new();
        e.set_english_auto_restore(ar);
        if free {
            e.set_free_tone(true);
            e.set_allow_foreign_consonants(true);
        }
        let t0 = cpu_ns();
        for w in &words {
            let mut any = false;
            for c in w.chars() {
                let t = cpu_ns();
                let r = e.on_key_ext(char_to_key(c), false, false, false);
                worst = worst.max(cpu_ns() - t);
                if r.backspace > 0 {
                    rewrites += 1;
                    bs += r.backspace as usize;
                    any = true;
                }
            }
            words_with += any as usize;
        }
        let total = (cpu_ns() - t0) / keys as f64;
        println!("{name:10} {total:6.0} ns/key | keys that rewrite text {:5.2}% | backspaces/word {:.2} | words with any rewrite {:5.1}% | slowest key {:.0} us",
            100.0 * rewrites as f64 / keys as f64, bs as f64 / words.len() as f64, 100.0 * words_with as f64 / words.len() as f64, worst / 1e3);
    }
}
