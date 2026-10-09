//! Where does the time go? best-of-N ns per key for each layer. `cargo run -q --release --example prof`
use gonhanh_core::compose::{diff::diff, Compose, Display, Options, RawKey};
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::char_to_key;

extern "C" {
    fn getrusage(who: i32, usage: *mut i64) -> i32;
}

/// User CPU time of this process in ns: unlike wall time it ignores other busy processes.
fn cpu_ns() -> f64 {
    let mut buf = [0i64; 36];
    unsafe { getrusage(0, buf.as_mut_ptr()) };
    buf[0] as f64 * 1e9 + (buf[1] & 0xFFFF_FFFF) as f64 * 1e3
}

fn words(path: &str, n: usize) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#') && l.contains('\t'))
        .filter_map(|l| l.split('\t').nth(1)?.split(',').next().map(String::from))
        .take(n)
        .collect()
}

fn best<F: FnMut()>(mut f: F, keys: usize) -> f64 {
    (0..9)
        .map(|_| {
            let t = cpu_ns();
            f();
            (cpu_ns() - t) / keys as f64
        })
        .fold(f64::MAX, f64::min)
}

fn main() {
    for (name, path) in [
        ("vi", "tests/data/vietnamese_22k_typing_variants.txt"),
        ("en", "tests/data/english_100k_typing_variants.txt"),
    ] {
        let w = words(path, 5000);
        let keys: usize = w.iter().map(|x| x.len() + 1).sum();
        let ks: Vec<Vec<RawKey>> = w
            .iter()
            .map(|x| {
                x.chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .map(|c| RawKey {
                        ch: c.to_ascii_lowercase() as u8,
                        caps: false,
                    })
                    .collect()
            })
            .collect();
        let push = best(
            || {
                let mut c = Compose::new(Options::default());
                for k in &ks {
                    c.clear();
                    for key in k {
                        c.push(*key);
                    }
                }
            },
            keys,
        );
        let disp = best(
            || {
                let mut c = Compose::new(Options::default());
                for k in &ks {
                    c.clear();
                    for key in k {
                        c.push(*key);
                        std::hint::black_box(c.display());
                    }
                }
            },
            keys,
        );
        let full = best(
            || {
                let mut c = Compose::new(Options::default());
                for k in &ks {
                    c.clear();
                    let mut s = Display::empty();
                    for key in k {
                        c.push(*key);
                        let n = c.display();
                        std::hint::black_box(diff(&s, &n));
                        s = n;
                    }
                }
            },
            keys,
        );
        let eng = best(
            || {
                let mut e = Engine::new();
                for x in &w {
                    for ch in x.chars().chain(std::iter::once(' ')) {
                        let k = char_to_key(ch);
                        if k != 255 {
                            std::hint::black_box(e.on_key_ext(k, false, false, false));
                        }
                    }
                }
            },
            keys,
        );
        let eng_ar = best(
            || {
                let mut e = Engine::new();
                e.set_english_auto_restore(true);
                for x in &w {
                    for ch in x.chars().chain(std::iter::once(' ')) {
                        let k = char_to_key(ch);
                        if k != 255 {
                            std::hint::black_box(e.on_key_ext(k, false, false, false));
                        }
                    }
                }
            },
            keys,
        );
        let new = best(
            || {
                for _ in 0..2000 {
                    std::hint::black_box(Compose::new(Options::default()));
                }
            },
            2000,
        );
        println!("{name}: push {push:.0}  +display {disp:.0}  +diff {full:.0}  engine/key {eng:.0}  engine+AR/key {eng_ar:.0}   Compose::new {new:.0} ns");
    }
}
