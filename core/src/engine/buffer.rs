//! Typing buffer

pub const MAX: usize = 64;

use crate::utils;

/// Single character in buffer
///
/// Modifiers:
/// - `tone`: vowel diacritics (^, horn, breve)
/// - `mark`: tone marks (sắc, huyền, hỏi, ngã, nặng)
/// - `stroke`: consonant stroke (d → đ)
#[derive(Clone, Copy, Default)]
pub struct Char {
    pub key: u16,
    pub caps: bool,
    pub tone: u8,     // 0=none, 1=circumflex(^), 2=horn/breve
    pub mark: u8,     // 0=none, 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
    pub stroke: bool, // true if 'd' → 'đ' (stroke through)
}

impl Char {
    pub fn new(key: u16, caps: bool) -> Self {
        Self {
            key,
            caps,
            tone: 0,
            mark: 0,
            stroke: false,
        }
    }

    pub fn has_tone(&self) -> bool {
        self.tone > 0
    }

    pub fn has_mark(&self) -> bool {
        self.mark > 0
    }
}

/// Typing buffer
#[derive(Clone)]
pub struct Buffer {
    data: [Char; MAX],
    len: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            data: [Char::default(); MAX],
            len: 0,
        }
    }

    pub fn push(&mut self, c: Char) {
        if self.len < MAX {
            self.data[self.len] = c;
            self.len += 1;
        }
    }

    pub fn pop(&mut self) -> Option<Char> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.data[self.len])
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, i: usize) -> Option<&Char> {
        if i < self.len {
            Some(&self.data[i])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, i: usize) -> Option<&mut Char> {
        if i < self.len {
            Some(&mut self.data[i])
        } else {
            None
        }
    }

    pub fn last(&self) -> Option<&Char> {
        if self.len > 0 {
            Some(&self.data[self.len - 1])
        } else {
            None
        }
    }

    /// Remove element at index, shifting subsequent elements left
    pub fn remove(&mut self, index: usize) {
        if index < self.len {
            // Shift elements left to fill the gap
            for i in index..self.len - 1 {
                self.data[i] = self.data[i + 1];
            }
            self.len -= 1;
        }
    }

    /// Find indices of vowels in buffer
    pub fn find_vowels(&self) -> Vec<usize> {
        use crate::data::keys;
        (0..self.len)
            .filter(|&i| keys::is_vowel(self.data[i].key))
            .collect()
    }

    /// Find vowel position by key (from end)
    pub fn find_vowel_by_key(&self, key: u16) -> Option<usize> {
        use crate::data::keys;
        (0..self.len)
            .rev()
            .find(|&i| self.data[i].key == key && keys::is_vowel(key))
    }

    /// Iterate over chars
    pub fn iter(&self) -> std::slice::Iter<'_, Char> {
        self.data[..self.len].iter()
    }

    /// Convert buffer to lowercase string (for shortcut matching)
    pub fn to_lowercase_string(&self) -> String {
        self.data[..self.len]
            .iter()
            .filter_map(|c| utils::key_to_char(c.key, false))
            .collect()
    }

    /// Convert buffer to string preserving case (for shortcut case matching)
    pub fn to_string_preserve_case(&self) -> String {
        self.data[..self.len]
            .iter()
            .filter_map(|c| utils::key_to_char(c.key, c.caps))
            .collect()
    }

    /// Convert buffer to full Vietnamese string with diacritics (for shortcut matching)
    ///
    /// This includes tone marks (sắc/huyền/hỏi/ngã/nặng), vowel marks (circumflex/horn/breve),
    /// and stroked consonants (đ). Use this for shortcut matching to ensure exact comparison.
    pub fn to_full_string(&self) -> String {
        use crate::data::{chars, keys};
        self.data[..self.len]
            .iter()
            .filter_map(|c| {
                // Handle đ/Đ (stroked D)
                if c.key == keys::D && c.stroke {
                    return Some(chars::get_d(c.caps));
                }
                // Try to get full Vietnamese character with diacritics
                if let Some(ch) = chars::to_char(c.key, c.caps, c.tone, c.mark) {
                    return Some(ch);
                }
                // Fallback to basic character
                utils::key_to_char(c.key, c.caps)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer() {
        let mut buf = Buffer::new();
        assert!(buf.is_empty());

        buf.push(Char::new(0, false));
        buf.push(Char::new(1, true));
        assert_eq!(buf.len(), 2);

        let c = buf.pop().unwrap();
        assert_eq!(c.key, 1);
        assert!(c.caps);

        buf.clear();
        assert!(buf.is_empty());
    }
}
