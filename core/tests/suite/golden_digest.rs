//! Behavior digest: a deterministic fingerprint of what the engine types for large corpora.
//!
//! Catches every output change that no assertion covers (e.g. which English words are
//! restored). Refactor phases that must not change behavior keep this green; phases that
//! intentionally change it regenerate with `UPDATE_GOLDEN=1` and review the diff of
//! `bench/golden.txt` (counts show how many items moved).

use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

const GOLDEN: &str = "bench/golden.txt";

fn fnv1a(seed: u64, bytes: &[u8]) -> u64 {
    let mut h = seed ^ 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn read(path: &str) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

/// All variants of "word<TAB>v1,v2,..." lines.
fn variants(path: &str) -> Vec<String> {
    read(path)
        .iter()
        .filter_map(|l| l.split('\t').nth(1))
        .flat_map(|v| v.split(',').map(String::from).collect::<Vec<_>>())
        .collect()
}

/// (items, order-independent digest). One engine per thread, reset between items.
fn digest(items: &[String], auto_restore: bool) -> (usize, u64) {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = items.len().div_ceil(threads).max(1);
    let total = std::thread::scope(|s| {
        let handles: Vec<_> = items
            .chunks(chunk)
            .map(|part| {
                s.spawn(move || {
                    let mut e = Engine::new();
                    e.set_english_auto_restore(auto_restore);
                    let mut acc = 0u64;
                    let dump = std::env::var_os("DIGEST_DUMP").is_some();
                    let mut lines: Vec<String> = Vec::new();
                    for item in part {
                        e.clear_all();
                        let out = type_word(&mut e, &format!("{item} "));
                        let h = fnv1a(fnv1a(0, item.as_bytes()), out.as_bytes());
                        acc = acc.wrapping_add(h);
                        if dump {
                            lines.push(format!("{auto_restore}\t{item}\t{out}"));
                        }
                    }
                    (acc, lines)
                })
            })
            .collect();
        let mut total = 0u64;
        let mut dumped: Vec<String> = Vec::new();
        for h in handles {
            let (acc, lines) = h.join().unwrap();
            total = total.wrapping_add(acc);
            dumped.extend(lines);
        }
        if let Some(path) = std::env::var_os("DIGEST_DUMP") {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap();
            for l in dumped {
                writeln!(f, "{l}").unwrap();
            }
        }
        total
    });
    (items.len(), total)
}

#[test]
fn engine_behavior_digest_matches_golden() {
    let en = read("tests/data/english_100k.txt");
    let en_var = variants("tests/data/english_100k_typing_variants.txt");
    let vi_var = variants("tests/data/vietnamese_22k_typing_variants.txt");

    let rows = [
        ("en100k_auto_restore", digest(&en, true)),
        ("en_variants_auto_restore", digest(&en_var, true)),
        ("vi_variants_auto_restore", digest(&vi_var, true)),
        ("vi_variants_raw", digest(&vi_var, false)),
    ];
    let now: String = rows
        .iter()
        .map(|(name, (n, d))| format!("{name}\t{n}\t{d:016x}\n"))
        .collect();

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(GOLDEN, &now).unwrap();
        return;
    }
    let golden = std::fs::read_to_string(GOLDEN).unwrap_or_else(|_| {
        panic!("{GOLDEN} missing: run `make update-golden` once on a known-good tree")
    });
    assert_eq!(
        golden, now,
        "engine behavior changed (name, items, digest). If intentional, review and run `make update-golden`."
    );
}
