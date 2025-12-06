//! Vietnamese IME Engine

pub mod buffer;

use buffer::{Buffer, Char, MAX};
use crate::data::{chars, keys};
use crate::input;

/// Engine action result
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    None,    // Pass through
    Send,    // Delete + send new chars
    Restore, // Invalid, restore original
}

/// Result for FFI
#[repr(C)]
pub struct Result {
    pub action: Action,
    pub backspace: u8,
    pub chars: [u32; MAX],
    pub count: u8,
}

impl Result {
    pub fn none() -> Self {
        Self {
            action: Action::None,
            backspace: 0,
            chars: [0; MAX],
            count: 0,
        }
    }

    pub fn send(backspace: u8, chars: &[char]) -> Self {
        let mut result = Self {
            action: Action::Send,
            backspace,
            chars: [0; MAX],
            count: chars.len() as u8,
        };
        for (i, &c) in chars.iter().enumerate() {
            if i < MAX {
                result.chars[i] = c as u32;
            }
        }
        result
    }
}

/// Main engine
pub struct Engine {
    buf: Buffer,
    method: u8,   // 0=Telex, 1=VNI
    enabled: bool,
    modern: bool, // oà vs òa
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            buf: Buffer::new(),
            method: 0,
            enabled: true,
            modern: true,
        }
    }

    pub fn set_method(&mut self, method: u8) {
        self.method = method;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.buf.clear();
        }
    }

    pub fn set_modern(&mut self, modern: bool) {
        self.modern = modern;
    }

    /// Handle key event
    pub fn on_key(&mut self, key: u16, caps: bool, ctrl: bool) -> Result {
        // Disabled or ctrl held
        if !self.enabled || ctrl {
            self.buf.clear();
            return Result::none();
        }

        // Break keys (space, punctuation, etc.)
        if keys::is_break(key) {
            self.buf.clear();
            return Result::none();
        }

        // Backspace
        if key == keys::DELETE {
            self.buf.pop();
            return Result::none();
        }

        // Process Vietnamese
        self.process(key, caps)
    }

    fn process(&mut self, key: u16, caps: bool) -> Result {
        let m = input::get(self.method);
        let prev_key = self.buf.last().map(|c| c.key);

        // Check đ (dd or d9)
        if m.is_d(key, prev_key) {
            return self.handle_d(caps);
        }

        // Check tone (aa, aw, a6, a7, etc.)
        if let Some(tone) = m.is_tone(key, prev_key) {
            return self.handle_tone(tone, caps);
        }

        // Check mark (s/f/r/x/j or 1-5)
        if let Some(mark) = m.is_mark(key) {
            return self.handle_mark(mark);
        }

        // Check remove mark (z or 0)
        if m.is_remove(key) {
            return self.handle_remove();
        }

        // Normal key - add to buffer
        if keys::is_letter(key) {
            self.buf.push(Char::new(key, caps));
        } else {
            // Non-letter breaks word
            self.buf.clear();
        }

        Result::none()
    }

    /// Handle đ
    fn handle_d(&mut self, caps: bool) -> Result {
        // Pop previous 'd'
        self.buf.pop();
        let ch = chars::get_d(caps);
        Result::send(1, &[ch])
    }

    /// Handle tone (^, ˘)
    fn handle_tone(&mut self, tone: u8, caps: bool) -> Result {
        if let Some(last) = self.buf.last_mut() {
            // Can only add tone to vowel
            if keys::is_vowel(last.key) {
                last.tone = tone;
                // Generate new char
                if let Some(ch) = chars::to_char(last.key, last.caps || caps, last.tone, last.mark) {
                    return Result::send(1, &[ch]);
                }
            }
        }
        Result::none()
    }

    /// Handle mark (sắc, huyền, hỏi, ngã, nặng)
    fn handle_mark(&mut self, mark: u8) -> Result {
        let vowels = self.buf.find_vowels();
        if vowels.is_empty() {
            return Result::none();
        }

        // Find position to place mark
        let pos = self.find_mark_pos(&vowels);

        if let Some(c) = self.buf.get_mut(pos) {
            c.mark = mark;
            // Rebuild word from mark position
            return self.rebuild_from(pos);
        }

        Result::none()
    }

    /// Handle remove mark
    fn handle_remove(&mut self) -> Result {
        let vowels = self.buf.find_vowels();
        if vowels.is_empty() {
            return Result::none();
        }

        // Find vowel with mark and remove it
        for &i in vowels.iter().rev() {
            if let Some(c) = self.buf.get_mut(i) {
                if c.mark > 0 {
                    c.mark = 0;
                    return self.rebuild_from(i);
                }
                if c.tone > 0 {
                    c.tone = 0;
                    return self.rebuild_from(i);
                }
            }
        }

        Result::none()
    }

    /// Find position to place mark
    fn find_mark_pos(&self, vowels: &[usize]) -> usize {
        match vowels.len() {
            0 => 0,
            1 => vowels[0],
            2 => {
                // oa, oe, uy -> modern: last, old: first
                if self.modern {
                    vowels[1]
                } else {
                    vowels[0]
                }
            }
            _ => vowels[1], // 3+ vowels: middle
        }
    }

    /// Rebuild output from position
    fn rebuild_from(&self, from: usize) -> Result {
        let mut output = Vec::new();
        let mut backspace = 0u8;

        for i in from..self.buf.len() {
            if let Some(c) = self.buf.get(i) {
                backspace += 1;
                if let Some(ch) = chars::to_char(c.key, c.caps, c.tone, c.mark) {
                    output.push(ch);
                }
            }
        }

        if output.is_empty() {
            Result::none()
        } else {
            Result::send(backspace, &output)
        }
    }

    /// Clear buffer (new session)
    pub fn clear(&mut self) {
        self.buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telex_basic() {
        let mut e = Engine::new();
        e.set_method(0); // Telex

        // Type 'a'
        let r = e.on_key(keys::A, false, false);
        assert_eq!(r.action, Action::None);

        // Type 's' -> á
        let r = e.on_key(keys::S, false, false);
        assert_eq!(r.action, Action::Send);
        assert_eq!(r.chars[0], 'á' as u32);
    }

    #[test]
    fn test_vni_basic() {
        let mut e = Engine::new();
        e.set_method(1); // VNI

        // Type 'a'
        let r = e.on_key(keys::A, false, false);
        assert_eq!(r.action, Action::None);

        // Type '1' -> á
        let r = e.on_key(keys::N1, false, false);
        assert_eq!(r.action, Action::Send);
        assert_eq!(r.chars[0], 'á' as u32);
    }
}
