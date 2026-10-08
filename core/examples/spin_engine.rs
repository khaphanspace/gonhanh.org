//! Busy loop of the whole engine (feed the sampling profiler): `cargo run --release --features engine_v2 --example spin_engine`
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::char_to_key;
fn main() {
    let text = std::fs::read_to_string("tests/data/vietnamese_22k_typing_variants.txt").unwrap();
    let ws: Vec<String> = text
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split('\t').nth(1)?.split(',').next().map(String::from))
        .collect();
    let mut e = Engine::new();
    let t = std::time::Instant::now();
    while t.elapsed().as_secs() < 14 {
        for w in &ws {
            for c in w.chars().chain(std::iter::once(' ')) {
                let k = char_to_key(c);
                if k != 255 {
                    std::hint::black_box(e.on_key_ext(k, false, false, false));
                }
            }
        }
    }
}
