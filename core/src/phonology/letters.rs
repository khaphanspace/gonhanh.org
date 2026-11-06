//! Letters of a Vietnamese syllable: base letter + modifier (+ stroke on d) + tone.
//!
//! Pure data, no allocation. `Unit` is the atom every phonology rule works on; the engine
//! converts its own representation (keycodes, buffer chars) into `Unit`s at the boundary.

/// Diacritic on a vowel. `Breve` is the ă of `a`; horn is the ơ/ư of `o`/`u`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Mod {
    None,
    Circ,
    Horn,
    Breve,
}

/// The six tones, in the order of the tables below.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tone {
    Ngang,
    Sac,
    Huyen,
    Hoi,
    Nga,
    Nang,
}

/// One letter: lowercase ASCII base letter, vowel modifier, and `stroke` (d → đ).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unit {
    pub ch: u8,
    pub md: Mod,
    pub stroke: bool,
}

/// (composed lowercase vowel without tone, base ASCII letter, modifier, the five toned forms).
/// Toned forms are ordered sắc, huyền, hỏi, ngã, nặng.
const VOWELS: [(char, u8, Mod, [char; 5]); 12] = [
    ('a', b'a', Mod::None, ['á', 'à', 'ả', 'ã', 'ạ']),
    ('ă', b'a', Mod::Breve, ['ắ', 'ằ', 'ẳ', 'ẵ', 'ặ']),
    ('â', b'a', Mod::Circ, ['ấ', 'ầ', 'ẩ', 'ẫ', 'ậ']),
    ('e', b'e', Mod::None, ['é', 'è', 'ẻ', 'ẽ', 'ẹ']),
    ('ê', b'e', Mod::Circ, ['ế', 'ề', 'ể', 'ễ', 'ệ']),
    ('i', b'i', Mod::None, ['í', 'ì', 'ỉ', 'ĩ', 'ị']),
    ('o', b'o', Mod::None, ['ó', 'ò', 'ỏ', 'õ', 'ọ']),
    ('ô', b'o', Mod::Circ, ['ố', 'ồ', 'ổ', 'ỗ', 'ộ']),
    ('ơ', b'o', Mod::Horn, ['ớ', 'ờ', 'ở', 'ỡ', 'ợ']),
    ('u', b'u', Mod::None, ['ú', 'ù', 'ủ', 'ũ', 'ụ']),
    ('ư', b'u', Mod::Horn, ['ứ', 'ừ', 'ử', 'ữ', 'ự']),
    ('y', b'y', Mod::None, ['ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ']),
];

const TONES: [Tone; 5] = [Tone::Sac, Tone::Huyen, Tone::Hoi, Tone::Nga, Tone::Nang];

impl Unit {
    pub const fn new(ch: u8, md: Mod) -> Self {
        Unit {
            ch,
            md,
            stroke: false,
        }
    }

    /// Unit from a table code: modifier 0 none, 1 circ, 2 horn, 3 breve, 4 stroke (d → đ).
    pub const fn from_code(ch: u8, code: u8) -> Self {
        let md = match code {
            1 => Mod::Circ,
            2 => Mod::Horn,
            3 => Mod::Breve,
            _ => Mod::None,
        };
        Unit {
            ch,
            md,
            stroke: code == 4,
        }
    }

    #[inline]
    pub fn is_vowel(self) -> bool {
        matches!(self.ch, b'a' | b'e' | b'i' | b'o' | b'u' | b'y')
    }

    /// Lowercase composed letter without tone (`ă`, `đ`, `k`, ...).
    pub fn to_char(self) -> char {
        match (self.ch, self.md) {
            (b'd', _) if self.stroke => 'đ',
            (b'a', Mod::Breve) => 'ă',
            (b'a', Mod::Circ) => 'â',
            (b'e', Mod::Circ) => 'ê',
            (b'o', Mod::Circ) => 'ô',
            (b'o', Mod::Horn) => 'ơ',
            (b'u', Mod::Horn) => 'ư',
            _ => self.ch as char,
        }
    }

    /// Lowercase composed letter carrying `tone` (only vowels can carry a tone).
    pub fn compose(self, tone: Tone) -> char {
        let plain = self.to_char();
        if tone == Tone::Ngang {
            return plain;
        }
        for (c, _, _, toned) in VOWELS {
            if c == plain {
                return toned[TONES.iter().position(|&t| t == tone).unwrap_or(0)];
            }
        }
        plain
    }

    /// Decompose one lowercase Vietnamese letter into (unit, tone). `None` for non-letters.
    pub fn from_char(c: char) -> Option<(Unit, Tone)> {
        if c == 'đ' {
            return Some((
                Unit {
                    ch: b'd',
                    md: Mod::None,
                    stroke: true,
                },
                Tone::Ngang,
            ));
        }
        for (plain, b, m, toned) in VOWELS {
            if c == plain {
                return Some((Unit::new(b, m), Tone::Ngang));
            }
            if let Some(i) = toned.iter().position(|&t| t == c) {
                return Some((Unit::new(b, m), TONES[i]));
            }
        }
        if c.is_ascii_lowercase() {
            return Some((Unit::new(c as u8, Mod::None), Tone::Ngang));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_vietnamese_letter_roundtrips() {
        for (plain, _, _, toned) in VOWELS {
            let (u, t) = Unit::from_char(plain).unwrap();
            assert_eq!((u.to_char(), t), (plain, Tone::Ngang));
            for (i, ch) in toned.iter().enumerate() {
                let (u, t) = Unit::from_char(*ch).unwrap();
                assert_eq!(t, TONES[i]);
                assert_eq!(u.compose(t), *ch);
            }
        }
        let (d, _) = Unit::from_char('đ').unwrap();
        assert_eq!(d.to_char(), 'đ');
    }
}
