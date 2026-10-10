//! Show the readings of a word: `cargo run -q --example dbg -- repea`
use gonhanh_core::compose::{Compose, Options, RawKey};
fn main() {
    for w in std::env::args().skip(1) {
        let mut c = Compose::new(Options::default());
        for ch in w.chars() {
            c.push(RawKey {
                ch: ch as u8,
                caps: false,
            });
            let b = c.best();
            let s: String = c.display().as_slice().iter().collect();
            println!(
                "{ch}: shown {s:?} roles {:?} tone {:?} reverted {}",
                &b.roles[..c.len()],
                b.tone,
                b.reverted
            );
        }
    }
}
