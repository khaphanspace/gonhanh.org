//! Engine benchmark: latency, allocations per key, cold start, typing-UX KPI.
//!
//! std-only (core stays zero-dependency). Run from `core/`:
//!   cargo bench --bench engine_bench                  # compare with bench/baseline.json
//!   cargo bench --bench engine_bench -- --save        # overwrite bench/baseline.json
//!   cargo bench --bench engine_bench -- --smoke       # fewer words (gate)
//!
//! Noise policy: the dev machine is often heavily loaded, so only medians (min over
//! passes) and deterministic counters (allocations, flicker) are gated; p99.9/max are not.

use gonhanh_core::compose::{diff::diff, Compose, Options, RawKey};
use gonhanh_core::engine::{Action, Engine};
use gonhanh_core::utils::char_to_key;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

struct Counting;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        BYTES.fetch_add(l.size(), Relaxed);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        BYTES.fetch_add(n, Relaxed);
        System.realloc(p, l, n)
    }
}

#[global_allocator]
static A: Counting = Counting;

const BASELINE: &str = "bench/baseline.json";

fn engine(auto_restore: bool) -> Engine {
    let mut e = Engine::new();
    e.set_english_auto_restore(auto_restore);
    e
}

/// First typing variant of each corpus line ("word<TAB>v1,v2,...").
fn load_variants(path: &str, take: usize) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{path}: {e} (run from core/)"))
        .lines()
        .filter(|l| !l.starts_with('#') && l.contains('\t'))
        .filter_map(|l| l.split('\t').nth(1)?.split(',').next().map(String::from))
        .take(take)
        .collect()
}

struct Latency {
    median_ns: u64,
    p90_ns: u64,
    allocs_per_key: f64,
    bytes_per_key: f64,
}

/// One pass over the corpus; returns per-key latencies (ns) and allocation counters.
fn pass(words: &[String], auto_restore: bool) -> (Vec<u64>, usize, usize) {
    let mut e = engine(auto_restore);
    let mut lat = Vec::with_capacity(words.len() * 8);
    let (a0, b0) = (ALLOCS.load(Relaxed), BYTES.load(Relaxed));
    for w in words {
        for c in w.chars().chain(std::iter::once(' ')) {
            let k = char_to_key(c);
            if k == 255 {
                continue;
            }
            let t = Instant::now();
            let r = e.on_key_ext(k, c.is_uppercase(), false, false);
            lat.push(t.elapsed().as_nanos() as u64);
            std::hint::black_box(&r);
        }
    }
    (lat, ALLOCS.load(Relaxed) - a0, BYTES.load(Relaxed) - b0)
}

/// Whole-corpus time per key, best of `passes` runs (noise on a busy machine only adds time).
fn per_key_ns(words: &[String], auto_restore: bool, passes: usize) -> f64 {
    let keys: usize = words.iter().map(|w| w.chars().count() + 1).sum();
    let mut best = f64::MAX;
    for _ in 0..passes.max(5) {
        let mut e = engine(auto_restore);
        let t = Instant::now();
        for w in words {
            for c in w.chars().chain(std::iter::once(' ')) {
                let k = char_to_key(c);
                if k != 255 {
                    std::hint::black_box(e.on_key_ext(k, c.is_uppercase(), false, false));
                }
            }
        }
        best = best.min(t.elapsed().as_nanos() as f64 / keys as f64);
    }
    best
}

fn measure(words: &[String], auto_restore: bool, passes: usize) -> Latency {
    let _ = pass(&words[..words.len().min(200)], auto_restore); // warm code + dictionaries
    let mut best: Option<Latency> = None;
    for _ in 0..passes {
        let (mut lat, allocs, bytes) = pass(words, auto_restore);
        let n = lat.len();
        lat.sort_unstable();
        let cur = Latency {
            median_ns: lat[n / 2],
            p90_ns: lat[n * 9 / 10],
            allocs_per_key: allocs as f64 / n as f64,
            bytes_per_key: bytes as f64 / n as f64,
        };
        if best.as_ref().is_none_or(|b| cur.median_ns < b.median_ns) {
            best = Some(cur);
        }
    }
    best.unwrap()
}

/// Typing core: per key push + render + diff, one word at a time (word boundary = clear).
fn measure_compose(words: &[String], passes: usize) -> Latency {
    let run = |words: &[String]| -> (Vec<u64>, usize, usize) {
        let mut c = Compose::new(Options::default());
        let mut lat = Vec::with_capacity(words.len() * 8);
        let (a0, b0) = (ALLOCS.load(Relaxed), BYTES.load(Relaxed));
        for w in words {
            c.clear();
            let mut screen = gonhanh_core::compose::Display::empty();
            for ch in w.chars() {
                if !ch.is_ascii_alphanumeric() {
                    continue;
                }
                let t = Instant::now();
                c.push(RawKey {
                    ch: ch.to_ascii_lowercase() as u8,
                    caps: ch.is_ascii_uppercase(),
                });
                let next = c.display();
                let e = diff(&screen, &next);
                screen = next;
                lat.push(t.elapsed().as_nanos() as u64);
                std::hint::black_box(&e);
            }
        }
        (lat, ALLOCS.load(Relaxed) - a0, BYTES.load(Relaxed) - b0)
    };
    let _ = run(&words[..words.len().min(200)]);
    let mut best: Option<Latency> = None;
    for _ in 0..passes {
        let (mut lat, allocs, bytes) = run(words);
        let n = lat.len();
        lat.sort_unstable();
        let cur = Latency {
            median_ns: lat[n / 2],
            p90_ns: lat[n * 9 / 10],
            allocs_per_key: allocs as f64 / n as f64,
            bytes_per_key: bytes as f64 / n as f64,
        };
        if best.as_ref().is_none_or(|b| cur.median_ns < b.median_ns) {
            best = Some(cur);
        }
    }
    best.unwrap()
}

/// UX KPI: share of words where a Vietnamese diacritic is shown and later removed,
/// and screen rewrites (backspaces) per word. Deterministic.
fn flicker(words: &[String]) -> (f64, f64) {
    let (mut flick, mut bs) = (0usize, 0usize);
    for w in words {
        let mut e = engine(true);
        let mut screen = String::new();
        let mut shown = false;
        for c in w.chars().chain(std::iter::once(' ')) {
            let k = char_to_key(c);
            if k == 255 {
                screen.push(c);
                continue;
            }
            let r = e.on_key_ext(k, c.is_uppercase(), false, false);
            if r.action == Action::Send as u8 {
                bs += r.backspace as usize;
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        screen.push(ch);
                    }
                }
                if c == ' ' && r.flags & 1 == 0 && !screen.ends_with(' ') {
                    screen.push(' ');
                }
            } else {
                screen.push(c);
            }
            shown |= !screen.is_ascii();
        }
        if shown && screen.trim_end().is_ascii() {
            flick += 1;
        }
    }
    (
        100.0 * flick as f64 / words.len() as f64,
        bs as f64 / words.len() as f64,
    )
}

/// Child mode: first keystroke of a fresh process (dictionary init included), in ns.
fn cold_probe() {
    let mut e = engine(false);
    let t = Instant::now();
    let r = e.on_key_ext(char_to_key('v'), false, false, false);
    std::hint::black_box(&r);
    println!("{}", t.elapsed().as_nanos());
}

fn cold_first_key_us(runs: usize) -> f64 {
    let exe = std::env::current_exe().unwrap();
    let mut v: Vec<u64> = (0..runs)
        .filter_map(|_| {
            let o = std::process::Command::new(&exe)
                .arg("--cold-probe")
                .output()
                .ok()?;
            String::from_utf8_lossy(&o.stdout).trim().parse().ok()
        })
        .collect();
    v.sort_unstable();
    v.get(v.len() / 2).copied().unwrap_or(0) as f64 / 1000.0
}

fn json(rows: &[(&str, f64)]) -> String {
    let body: Vec<String> = rows
        .iter()
        .map(|(k, v)| format!("  \"{k}\": {v:.3}"))
        .collect();
    format!("{{\n{}\n}}\n", body.join(",\n"))
}

/// Minimal reader for the flat JSON this file writes.
fn read_baseline() -> Vec<(String, f64)> {
    std::fs::read_to_string(BASELINE)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (k, v) = l.trim().trim_end_matches(',').split_once(':')?;
            Some((
                k.trim().trim_matches('"').to_string(),
                v.trim().parse().ok()?,
            ))
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--cold-probe") {
        return cold_probe();
    }
    let smoke = args.iter().any(|a| a == "--smoke");
    let (n_vi, n_en, passes) = if smoke {
        (1500, 800, 2)
    } else {
        (6000, 3000, 3)
    };

    let cold = cold_first_key_us(if smoke { 5 } else { 15 });
    let vi = load_variants("tests/data/vietnamese_22k_typing_variants.txt", n_vi);
    let en = load_variants("tests/data/english_100k_typing_variants.txt", n_en);

    let mut rows: Vec<(&str, f64)> = vec![("cold_first_key_us", cold)];
    for (name, words, auto) in [
        ("vi_auto_off", &vi, false),
        ("vi_auto_on", &vi, true),
        ("en_auto_off", &en, false),
        ("en_auto_on", &en, true),
    ] {
        let l = measure(words, auto, passes);
        let leak = |suffix: &str| -> &'static str {
            Box::leak(format!("{name}_{suffix}").into_boxed_str())
        };
        rows.push((leak("median_ns"), l.median_ns as f64));
        rows.push((leak("p90_ns"), l.p90_ns as f64));
        rows.push((leak("allocs_per_key"), l.allocs_per_key));
        rows.push((leak("bytes_per_key"), l.bytes_per_key));
        rows.push((leak("best_ns_per_key"), per_key_ns(words, auto, passes)));
    }
    for (name, words) in [("compose_vi", &vi), ("compose_en", &en)] {
        let l = measure_compose(words, passes);
        let leak = |suffix: &str| -> &'static str {
            Box::leak(format!("{name}_{suffix}").into_boxed_str())
        };
        rows.push((leak("median_ns"), l.median_ns as f64));
        rows.push((leak("p90_ns"), l.p90_ns as f64));
        rows.push((leak("allocs_per_key"), l.allocs_per_key));
        rows.push((leak("bytes_per_key"), l.bytes_per_key));
    }
    let (flick_pct, bs_per_word) = flicker(&en[..en.len().min(5000)]);
    rows.push(("en_flicker_pct", flick_pct));
    rows.push(("en_backspaces_per_word", bs_per_word));

    let out = json(&rows);
    if args.iter().any(|a| a == "--save") {
        std::fs::create_dir_all("bench").unwrap();
        std::fs::write(BASELINE, &out).unwrap();
        println!("saved {BASELINE}\n{out}");
        return;
    }
    let base = read_baseline();
    println!(
        "{:<28}{:>14}{:>14}{:>9}",
        "metric", "now", "baseline", "delta"
    );
    for (k, v) in &rows {
        match base.iter().find(|(bk, _)| bk == k) {
            Some((_, b)) if *b != 0.0 => {
                println!("{k:<28}{v:>14.2}{b:>14.2}{:>8.0}%", 100.0 * (v - b) / b)
            }
            _ => println!("{k:<28}{v:>14.2}{:>14}{:>9}", "-", "-"),
        }
    }
}
