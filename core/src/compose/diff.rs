//! Display change → what the platform must do: delete `backspace` chars, type `chars`.
//! One function replaces every hand-counted "+1 backspace" of engine v1.

use super::render::Display;
use super::MAXK;

pub struct Edit {
    pub backspace: u8,
    pub chars: [char; MAXK],
    pub count: u8,
}

/// Minimal edit that turns what is on screen (`prev`) into `next` (longest common prefix kept).
pub fn diff(prev: &Display, next: &Display) -> Edit {
    let (a, b) = (prev.as_slice(), next.as_slice());
    let common = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let mut e = Edit {
        backspace: (a.len() - common) as u8,
        chars: ['\0'; MAXK],
        count: (b.len() - common) as u8,
    };
    e.chars[..b.len() - common].copy_from_slice(&b[common..]);
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Display {
        let mut x = Display::empty();
        s.chars().for_each(|c| x.push(c));
        x
    }

    #[test]
    fn minimal_edits() {
        let e = diff(&d("tie"), &d("tiê"));
        assert_eq!((e.backspace, &e.chars[..e.count as usize]), (1, &['ê'][..]));
        let e = diff(&d("duơ"), &d("dươc"));
        assert_eq!(
            (e.backspace, &e.chars[..e.count as usize]),
            (2, &['ư', 'ơ', 'c'][..])
        );
        let e = diff(&d("ab"), &d("abc"));
        assert_eq!((e.backspace, e.count), (0, 1));
    }
}
