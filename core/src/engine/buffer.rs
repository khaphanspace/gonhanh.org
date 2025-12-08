//! Typing buffer

pub const MAX: usize = 32;

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
