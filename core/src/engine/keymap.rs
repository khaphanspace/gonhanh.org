//! Platform keycodes ↔ the ASCII keys of the typing core, and word → typing keys (to resume
//! editing a word that is already on screen).

use crate::compose::{Compose, Method, RawKey};
use crate::data::keys;
use crate::phonology::{Mod, Tone, Unit};
use crate::utils;

/// The core key for a platform key: lowercase letter or digit, case separate.
pub fn raw_key(key: u16, caps: bool) -> Option<RawKey> {
    let c = utils::key_to_char(key, false)?;
    (c.is_ascii_alphanumeric() || c == '[' || c == ']').then_some(RawKey {
        ch: c.to_ascii_lowercase() as u8,
        caps,
    })
}

pub fn is_word_key(key: u16) -> bool {
    keys::is_letter(key) || keys::is_number(key)
}

/// A key sequence that makes the typing core show `word` (lowercase + capitals), or `None`.
/// Letters come first with their own modifier key, the tone key last; the result is verified by
/// replaying it, so a wrong guess can never leave the engine out of sync with the screen.
pub fn keys_for_word(
    word: &str,
    method: Method,
    opts: crate::compose::Options,
) -> Option<([RawKey; crate::compose::MAXK], usize)> {
    let (circ, horn, breve, stroke): (&[u8], &[u8], &[u8], &[u8]) = match method {
        Method::Telex => (b"", b"w", b"w", b"d"),
        Method::Vni => (b"6", b"7", b"8", b"9"),
    };
    let tone_key = |t: Tone| -> Option<u8> {
        Some(match (method, t) {
            (Method::Telex, Tone::Sac) => b's',
            (Method::Telex, Tone::Huyen) => b'f',
            (Method::Telex, Tone::Hoi) => b'r',
            (Method::Telex, Tone::Nga) => b'x',
            (Method::Telex, Tone::Nang) => b'j',
            (Method::Vni, Tone::Sac) => b'1',
            (Method::Vni, Tone::Huyen) => b'2',
            (Method::Vni, Tone::Hoi) => b'3',
            (Method::Vni, Tone::Nga) => b'4',
            (Method::Vni, Tone::Nang) => b'5',
            _ => return None,
        })
    };
    let mut out = [RawKey { ch: 0, caps: false }; crate::compose::MAXK];
    let mut n = 0;
    let mut push = |ch: u8, caps: bool, n: &mut usize| -> bool {
        if *n == out.len() {
            return false;
        }
        out[*n] = RawKey { ch, caps };
        *n += 1;
        true
    };
    let mut tone = Tone::Ngang;
    for c in word.chars() {
        let caps = c.is_uppercase();
        let lower = c.to_lowercase().next()?;
        let (u, t) = Unit::from_char(lower)?;
        if t != Tone::Ngang {
            tone = t;
        }
        let base = if u.stroke { b'd' } else { u.ch };
        if !push(base, caps, &mut n) {
            return None;
        }
        let extra: Option<u8> = match (u.md, u.stroke) {
            (_, true) => stroke.first().copied().or(Some(b'd')),
            (Mod::Circ, _) => circ.first().copied().or(Some(u.ch)),
            (Mod::Horn, _) => horn.first().copied(),
            (Mod::Breve, _) => breve.first().copied(),
            _ => None,
        };
        if let Some(k) = extra {
            if !push(k, false, &mut n) {
                return None;
            }
        }
    }
    if let Some(k) = tone_key(tone) {
        if !push(k, false, &mut n) {
            return None;
        }
    }
    // verify
    let mut c = Compose::new(opts);
    for k in &out[..n] {
        c.push(*k);
    }
    let shown: String = c.display().as_slice().iter().collect();
    (shown == word).then_some((out, n))
}
