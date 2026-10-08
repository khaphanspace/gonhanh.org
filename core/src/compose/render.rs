//! Letters → text. The tone goes where `phonology::tone_index` says, on the *finished*
//! syllable, so the order in which tone and modifier keys arrived is irrelevant.

use super::parse::Parse;
use super::{Options, RawKey, MAXK};
use crate::phonology::{tone_index, Tone};

/// Text shown for the current word (at most one char per key).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Display {
    pub chars: [char; MAXK],
    pub len: u8,
}

impl Display {
    pub const fn empty() -> Self {
        Display {
            chars: ['\0'; MAXK],
            len: 0,
        }
    }

    pub fn as_slice(&self) -> &[char] {
        &self.chars[..self.len as usize]
    }

    pub fn push(&mut self, c: char) {
        if (self.len as usize) < MAXK {
            self.chars[self.len as usize] = c;
            self.len += 1;
        }
    }
}

pub fn render(p: &Parse, raw: &[RawKey], o: &Options) -> Display {
    let mut out = Display::empty();
    let units = p.units();
    let at = if p.tone != Tone::Ngang {
        tone_index(units, o.modern_tone)
    } else {
        None
    };
    for (k, u) in units.iter().enumerate() {
        let c = if Some(k) == at {
            u.compose(p.tone)
        } else {
            u.to_char()
        };
        let upper = raw.get(p.unit_key[k] as usize).is_some_and(|r| r.caps);
        out.push(if upper {
            c.to_uppercase().next().unwrap_or(c)
        } else {
            c
        });
    }
    out
}

/// The word exactly as typed.
pub fn render_raw(raw: &[RawKey]) -> Display {
    let mut out = Display::empty();
    for r in raw {
        let c = r.ch as char;
        out.push(match (r.caps, c) {
            (true, '[') => '{',
            (true, ']') => '}',
            (true, _) => c.to_ascii_uppercase(),
            _ => c,
        });
    }
    out
}
