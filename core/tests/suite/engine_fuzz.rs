//! Random key storms: the engine must never panic, never ask to delete more than a word,
//! and always produce characters that exist. Deterministic (fixed seed), a couple of seconds.

use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn pick<T: Copy>(&mut self, xs: &[T]) -> T {
        xs[(self.next() % xs.len() as u64) as usize]
    }
}

/// Letters weigh most, then digits and tone keys, then boundaries and Backspace.
fn key_pool() -> Vec<u16> {
    let letters = [
        keys::A,
        keys::S,
        keys::D,
        keys::F,
        keys::H,
        keys::G,
        keys::Z,
        keys::X,
        keys::C,
        keys::V,
        keys::B,
        keys::Q,
        keys::W,
        keys::E,
        keys::R,
        keys::Y,
        keys::T,
        keys::O,
        keys::U,
        keys::I,
        keys::P,
        keys::L,
        keys::J,
        keys::K,
        keys::N,
        keys::M,
    ];
    let digits = [
        keys::N1,
        keys::N2,
        keys::N3,
        keys::N4,
        keys::N5,
        keys::N6,
        keys::N7,
        keys::N8,
        keys::N9,
        keys::N0,
    ];
    let mut pool = Vec::new();
    for _ in 0..6 {
        pool.extend_from_slice(&letters);
    }
    pool.extend_from_slice(&digits);
    pool.extend_from_slice(&[
        keys::SPACE,
        keys::SPACE,
        keys::DELETE,
        keys::DELETE,
        keys::DELETE,
        keys::DOT,
        keys::COMMA,
        keys::ESC,
        keys::RETURN,
    ]);
    pool
}

/// One storm: random settings, then `keys_n` random keys. Nothing may panic, a result never deletes
/// more than a word plus its space, and every character it sends is a real char.
fn storm(seed: u64, method: u8, restore: bool, keys_n: usize) {
    let pool = key_pool();
    let mut rng = Rng(seed);
    let mut e = Engine::new();
    e.set_method(method);
    e.set_english_auto_restore(restore);
    e.set_auto_capitalize(rng.next().is_multiple_of(2));
    e.set_modern_tone(rng.next().is_multiple_of(2));
    e.set_free_tone(rng.next().is_multiple_of(3));
    e.set_allow_foreign_consonants(rng.next().is_multiple_of(3));
    e.set_skip_w_shortcut(rng.next().is_multiple_of(4));
    for step in 0..keys_n {
        let key = rng.pick(&pool);
        let caps = rng.next().is_multiple_of(9);
        let r = e.on_key_ext(key, caps, false, rng.next().is_multiple_of(13));
        if r.action == Action::Send as u8 {
            // a word is at most MAXK letters (plus the space that follows it)
            assert!(
                r.backspace <= 26,
                "step {step}: key {key}: backspace {}",
                r.backspace
            );
            for &c in &r.chars[..r.count as usize] {
                assert!(char::from_u32(c).is_some(), "step {step}: invalid char {c}");
            }
        }
        if step % 50_000 == 49_999 {
            e.clear_all(); // a fresh sentence now and then
        }
    }
}

/// `GN_FUZZ_KEYS=2000000 cargo test --test suite engine_fuzz` for a longer run.
fn keys_per_storm() -> usize {
    std::env::var("GN_FUZZ_KEYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(150_000)
}

#[test]
fn random_key_storms_do_not_panic() {
    for (seed, method, restore) in [
        (1, 0, false),
        (2, 0, true),
        (3, 1, false),
        (4, 1, true),
        (5, 0, true),
        (6, 1, true),
    ] {
        storm(
            0x9E37_79B9_7F4A_7C15 ^ seed,
            method,
            restore,
            keys_per_storm(),
        );
    }
}
