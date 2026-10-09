//! What the engine tells the platform to do after a key, before it is packed into the FFI
//! `Result` (1 KB). Most edits are a few characters, so the engine passes this small value
//! around and builds the big struct once, at the public entry point.

use super::{Result, FLAG_KEY_CONSUMED};

const SHORT: usize = 32;

pub(super) enum Out {
    /// leave the key to the platform
    None,
    /// delete `backspace` characters, type `chars[..n]`
    Short {
        backspace: u8,
        n: u8,
        consumed: bool,
        chars: [char; SHORT],
    },
    /// a shortcut expansion longer than a word
    Long {
        backspace: u8,
        consumed: bool,
        chars: Vec<char>,
    },
}

impl Out {
    pub fn none() -> Out {
        Out::None
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Out::None)
    }

    pub fn send(backspace: u8, chars: &[char]) -> Out {
        Self::pack(backspace, chars, false)
    }

    /// The shortcut consumed the trigger key: the platform must not type it.
    pub fn send_consumed(backspace: u8, chars: &[char]) -> Out {
        Self::pack(backspace, chars, true)
    }

    fn pack(backspace: u8, chars: &[char], consumed: bool) -> Out {
        if chars.len() <= SHORT {
            let mut buf = ['\0'; SHORT];
            buf[..chars.len()].copy_from_slice(chars);
            Out::Short {
                backspace,
                n: chars.len() as u8,
                consumed,
                chars: buf,
            }
        } else {
            Out::Long {
                backspace,
                consumed,
                chars: chars.to_vec(),
            }
        }
    }
}

impl From<Out> for Result {
    fn from(o: Out) -> Result {
        match o {
            Out::None => Result::none(),
            Out::Short {
                backspace,
                n,
                consumed,
                chars,
            } => {
                let mut r = Result::send(backspace, &chars[..n as usize]);
                if consumed {
                    r.flags = FLAG_KEY_CONSUMED;
                }
                r
            }
            Out::Long {
                backspace,
                consumed,
                chars,
            } => {
                let mut r = Result::send(backspace, &chars);
                if consumed {
                    r.flags = FLAG_KEY_CONSUMED;
                }
                r
            }
        }
    }
}
