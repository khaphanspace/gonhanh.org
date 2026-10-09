//! Typing-experience and speed audits, one tool: `cargo run --release --features engine_v2 --example audit -- <name> [flags]`.
//!
//!   jumps        tone marks that move to another vowel while typing English words (`ar`, `free` flags, words on stdin)
//!   stretch      stretch the last vowel of every toned Vietnamese syllable (mùa → mùaaaa) and check every key (`ar`; pairs on stdin)
//!   runon        two Vietnamese words typed with no space between them (`ar`, `free`; pairs on stdin)
//!   en_compound  two English words typed with no space (helloworld): how often free typing changes them beyond normal typing
//!   free_cost    CPU per key and rewrites on English words, normal vs free typing
//!   spin         a 12 s typing loop to attach a profiler to (`free`, `vn` flags)

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("jumps") => jumps::run(),
        Some("stretch") => stretch::run(),
        Some("runon") => runon::run(),
        Some("en_compound") => en_compound::run(),
        Some("free_cost") => free_cost::run(),
        Some("spin") => spin::run(),
        _ => eprintln!("usage: audit <jumps|stretch|runon|en_compound|free_cost|spin> [flags]"),
    }
}

mod jumps {
    use gonhanh_core::engine::Engine;
    use gonhanh_core::utils::type_word;
    use std::io::BufRead;

    pub fn run() {
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
}

mod stretch {
    use gonhanh_core::engine::Engine;
    use gonhanh_core::utils::type_word;
    use std::io::BufRead;

    const TONED: &str = "áàảãạắằẳẵặấầẩẫậéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ";
    const MARKED: &str = "ăắằẳẵặâấầẩẫậêếềểễệôốồổỗộơớờởỡợưứừửữựđ";

    pub fn run() {
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
            let base_marks = base.chars().filter(|c| MARKED.contains(*c)).count();
            let mut why = None;
            for k in 1..=5 {
                let s = screen(&format!("{typed}{}", v.to_string().repeat(k)));
                let marks = s.chars().filter(|c| MARKED.contains(*c)).count();
                if s.chars().position(|c| TONED.contains(c)) != Some(at) {
                    why = Some(format!("tone moved at +{k}: {s}"));
                } else if marks > base_marks {
                    why = Some(format!("new mark at +{k}: {s}"));
                } else if s.chars().count() != base.chars().count() + k {
                    why = Some(format!("length at +{k}: {s}"));
                }
                if why.is_some() {
                    break;
                }
            }
            if let Some(w) = why {
                bad += 1;
                shown.push(format!("{typed}+{v}: {base} → {w}"));
            }
        }
        println!("{n} toned syllables, {bad} with a jump while the last vowel is stretched");
        println!("{}", shown.join("\n"));
    }
}

mod runon {
    use gonhanh_core::engine::Engine;
    use gonhanh_core::utils::type_word;
    use std::io::BufRead;

    pub fn run() {
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
}

mod en_compound {
    use gonhanh_core::engine::Engine;
    use gonhanh_core::utils::type_word;
    use std::collections::HashSet;

    pub fn run() {
        let words: Vec<String> = std::fs::read_to_string("src/data/english_dict_merged.txt")
            .unwrap()
            .lines()
            .filter(|w| w.len() >= 3 && w.chars().all(|c| c.is_ascii_lowercase()))
            .map(String::from)
            .collect();
        let mut seed = 99u64;
        let mut next = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as usize
        };
        let pairs: Vec<String> = (0..4000)
            .map(|_| {
                format!(
                    "{}{}",
                    words[next() % words.len()],
                    words[next() % words.len()]
                )
            })
            .collect();
        let run = |free: bool, p: &str| {
            let mut e = Engine::new();
            e.set_english_auto_restore(true);
            if free {
                e.set_free_tone(true);
                e.set_allow_foreign_consonants(true);
            }
            type_word(&mut e, &format!("{p} ")).trim_end().to_string()
        };
        let normal: HashSet<&String> = pairs.iter().filter(|p| run(false, p) != **p).collect();
        let extra: Vec<String> = pairs
            .iter()
            .filter(|p| !normal.contains(p) && run(true, p) != **p)
            .map(|p| format!("{p}→{}", run(true, p)))
            .collect();
        println!(
            "changed by normal typing: {} of {}",
            normal.len(),
            pairs.len()
        );
        println!("changed only by free typing: {}", extra.len());
        for e in extra.iter().take(70) {
            println!("{e}");
        }
    }
}

mod free_cost {
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
    pub fn run() {
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
}

mod spin {
    use gonhanh_core::engine::Engine;
    use gonhanh_core::utils::type_word;
    pub fn run() {
        let free = std::env::args().any(|a| a == "free");
        let vn = std::env::args().any(|a| a == "vn");
        let path = if vn {
            "tests/data/vietnamese_telex_pairs.txt"
        } else {
            "src/data/english_dict_merged.txt"
        };
        let words: Vec<String> = std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|l| l.split('\t').next().unwrap().to_string())
            .filter(|w| w.is_ascii() && w.len() > 1)
            .map(|w| format!("{w} "))
            .collect();
        let mut e = Engine::new();
        e.set_english_auto_restore(true);
        if free {
            e.set_free_tone(true);
            e.set_allow_foreign_consonants(true);
        }
        let t = std::time::Instant::now();
        while t.elapsed().as_secs() < 12 {
            for w in &words {
                std::hint::black_box(type_word(&mut e, w));
            }
        }
    }
}
