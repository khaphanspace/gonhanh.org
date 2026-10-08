//! Busy loop over the Vietnamese corpus for a sampling profiler: `cargo run --release --example spin`
use gonhanh_core::compose::{Compose, Options, RawKey};
fn main() {
    let text = std::fs::read_to_string("tests/data/vietnamese_22k_typing_variants.txt").unwrap();
    let ks: Vec<Vec<RawKey>> = text
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            l.split('\t').nth(1)?.split(',').next().map(|w| {
                w.chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .map(|c| RawKey {
                        ch: c.to_ascii_lowercase() as u8,
                        caps: false,
                    })
                    .collect()
            })
        })
        .collect();
    let mut c = Compose::new(Options::default());
    let t = std::time::Instant::now();
    while t.elapsed().as_secs() < 12 {
        for k in &ks {
            c.clear();
            for key in k {
                c.push(*key);
            }
        }
    }
}
