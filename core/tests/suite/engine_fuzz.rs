//! Random key storms: the engine must never panic, never ask to delete more than a word,
//! and always produce characters that exist. Deterministic (fixed seed), a couple of seconds.

use gonhanh_core::data::keys;
use gonhanh_core::engine::{Action, Engine};
use gonhanh_core::utils::key_to_char;

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

fn storm(seed: u64, method: u8, restore: bool, keys_n: usize) {
    let pool = key_pool();
    let mut rng = Rng(seed);
    let mut e = Engine::new();
    e.set_method(method);
    e.set_english_auto_restore(restore);
    e.set_auto_capitalize(rng.next() % 2 == 0);
    e.set_modern_tone(rng.next() % 2 == 0);
    let mut screen: Vec<char> = Vec::new();
    for step in 0..keys_n {
        let key = rng.pick(&pool);
        let caps = rng.next() % 9 == 0;
        let r = e.on_key_ext(key, caps, false, rng.next() % 13 == 0);
        if r.action == Action::Send as u8 {
            // a word is at most MAXK letters (plus the space that follows it)
            assert!(
                r.backspace <= 26,
                "step {step}: key {key}: backspace {}",
                r.backspace
            );
            for _ in 0..r.backspace {
                screen.pop();
            }
            for i in 0..r.count as usize {
                let c = char::from_u32(r.chars[i])
                    .unwrap_or_else(|| panic!("step {step}: invalid char {}", r.chars[i]));
                screen.push(c);
            }
        } else if key == keys::DELETE {
            screen.pop();
        } else if let Some(c) = key_to_char(key, caps) {
            screen.push(c);
        }
        // a new line or a cursor move: what was typed before is out of reach
        if key == keys::RETURN || key == keys::ESC {
            screen.clear();
        }
        if screen.len() > 4000 {
            screen.clear();
        }
    }
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
        storm(0x9E37_79B9_7F4A_7C15 ^ seed, method, restore, 150_000);
    }
}
