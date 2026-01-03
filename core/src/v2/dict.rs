//! Dictionary for auto-restore (Bloom filter)
//!
//! Uses a simple bloom filter for fast negative lookup:
//! - Returns false = definitely not in dictionary
//! - Returns true = probably in dictionary (small false positive rate)

/// Simple bloom filter for fast word lookup
pub struct Dict {
    bits: Vec<u64>,
    hash_count: usize,
}

impl Dict {
    /// Default size: 8192 u64s = ~64KB for good false positive rate
    const DEFAULT_SIZE: usize = 8192;

    /// Create empty dictionary
    pub fn new() -> Self {
        Self {
            bits: vec![0; Self::DEFAULT_SIZE],
            hash_count: 3,
        }
    }

    /// Create from word list
    pub fn from_words(words: &[&str]) -> Self {
        let mut dict = Self::new();
        for word in words {
            dict.insert(word);
        }
        dict
    }

    /// Insert word into bloom filter
    pub fn insert(&mut self, word: &str) {
        let lower = word.to_ascii_lowercase();
        for i in 0..self.hash_count {
            let hash = self.hash(&lower, i);
            let idx = hash / 64;
            let bit = hash % 64;
            if idx < self.bits.len() {
                self.bits[idx] |= 1 << bit;
            }
        }
    }

    /// Check if word may be in dictionary
    /// Returns false = definitely not in dict
    /// Returns true = probably in dict (small false positive rate)
    pub fn contains(&self, word: &str) -> bool {
        let lower = word.to_ascii_lowercase();
        for i in 0..self.hash_count {
            let hash = self.hash(&lower, i);
            let idx = hash / 64;
            let bit = hash % 64;
            if idx >= self.bits.len() || (self.bits[idx] >> bit) & 1 == 0 {
                return false;
            }
        }
        true
    }

    /// FNV-1a-like hash with seed
    fn hash(&self, word: &str, seed: usize) -> usize {
        let mut h: usize = seed.wrapping_mul(0x9e3779b9);
        for b in word.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as usize);
        }
        h % (self.bits.len() * 64)
    }
}

impl Default for Dict {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dict() {
        let dict = Dict::new();
        assert!(!dict.contains("anything"));
    }

    #[test]
    fn test_insert_and_contains() {
        let mut dict = Dict::new();
        dict.insert("hello");
        dict.insert("world");
        dict.insert("test");

        assert!(dict.contains("hello"));
        assert!(dict.contains("world"));
        assert!(dict.contains("test"));
    }

    #[test]
    fn test_case_insensitive() {
        let mut dict = Dict::new();
        dict.insert("Hello");

        assert!(dict.contains("hello"));
        assert!(dict.contains("HELLO"));
        assert!(dict.contains("HeLLo"));
    }

    #[test]
    fn test_from_words() {
        let dict = Dict::from_words(&["apple", "banana", "cherry"]);
        assert!(dict.contains("apple"));
        assert!(dict.contains("banana"));
        assert!(dict.contains("cherry"));
        // Random word should likely not match
        assert!(!dict.contains("xyzabc123"));
    }

    #[test]
    fn test_not_in_dict() {
        let dict = Dict::from_words(&["cat", "dog", "bird"]);
        // These should definitely return false (not in dict)
        assert!(!dict.contains("elephant"));
        assert!(!dict.contains("xyz123"));
        assert!(!dict.contains("randomword"));
    }

    #[test]
    fn test_vietnamese_words() {
        let dict = Dict::from_words(&["việt", "nam", "tiếng"]);
        assert!(dict.contains("việt"));
        assert!(dict.contains("nam"));
        assert!(dict.contains("tiếng"));
        assert!(!dict.contains("english"));
    }
}
