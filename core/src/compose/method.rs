//! Input methods as a table: key → intent. The engine never asks "is this Telex?".

use crate::phonology::Tone;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Method {
    Telex,
    Vni,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Intent {
    /// ordinary letter or digit
    Plain,
    Tone(Tone),
    /// remove the last tone/modifier
    Remove,
    /// circumflex; `Some(letter)` = Telex doubling (the letter must repeat), `None` = VNI digit
    Circ(Option<u8>),
    /// Telex `w`: horn on o/u, breve on a, or ư by itself
    W,
    /// VNI 7 (horn) and 8 (breve)
    Horn,
    Breve,
    /// d → đ
    Stroke,
}

pub fn intent(m: Method, ch: u8) -> Intent {
    match m {
        Method::Telex => match ch {
            b's' => Intent::Tone(Tone::Sac),
            b'f' => Intent::Tone(Tone::Huyen),
            b'r' => Intent::Tone(Tone::Hoi),
            b'x' => Intent::Tone(Tone::Nga),
            b'j' => Intent::Tone(Tone::Nang),
            b'z' => Intent::Remove,
            b'a' | b'e' | b'o' => Intent::Circ(Some(ch)),
            b'w' => Intent::W,
            b'd' => Intent::Stroke,
            _ => Intent::Plain,
        },
        Method::Vni => match ch {
            b'1' => Intent::Tone(Tone::Sac),
            b'2' => Intent::Tone(Tone::Huyen),
            b'3' => Intent::Tone(Tone::Hoi),
            b'4' => Intent::Tone(Tone::Nga),
            b'5' => Intent::Tone(Tone::Nang),
            b'6' => Intent::Circ(None),
            b'7' => Intent::Horn,
            b'8' => Intent::Breve,
            b'9' => Intent::Stroke,
            b'0' => Intent::Remove,
            _ => Intent::Plain,
        },
    }
}
