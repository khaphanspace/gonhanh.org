//! V2 Buffer - tracks raw keystrokes and transformed output
//!
//! Maintains two parallel strings:
//! - `raw`: Original ASCII keystrokes (e.g., "vieejt")
//! - `transformed`: Vietnamese output (e.g., "viet")
//!
//! This separation enables:
//! - Accurate auto-restore (restore to raw when English detected)
//! - Character consumption tracking (raw.len - transformed.len)
//! - Clean diff calculation for output

/// V2 Buffer - dual-string buffer for raw/transformed tracking
#[derive(Clone, Default, Debug)]
pub struct Buffer {
    /// Raw ASCII keystrokes as typed
    raw: String,
    /// Transformed Vietnamese output
    transformed: String,
}

impl Buffer {
    /// Create new empty buffer
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create buffer with pre-allocated capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            raw: String::with_capacity(capacity),
            transformed: String::with_capacity(capacity),
        }
    }

    // ===== Accessors =====

    /// Get raw keystrokes
    #[inline]
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Get transformed output
    #[inline]
    pub fn transformed(&self) -> &str {
        &self.transformed
    }

    /// Check if buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Get raw character count (not bytes)
    #[inline]
    pub fn raw_len(&self) -> usize {
        self.raw.chars().count()
    }

    /// Get transformed character count (not bytes)
    #[inline]
    pub fn transformed_len(&self) -> usize {
        self.transformed.chars().count()
    }

    // ===== Mutators =====

    /// Push character to raw buffer
    #[inline]
    pub fn push_raw(&mut self, c: char) {
        self.raw.push(c);
    }

    /// Push character to transformed buffer
    #[inline]
    pub fn push_transformed(&mut self, c: char) {
        self.transformed.push(c);
    }

    /// Set entire transformed string
    #[inline]
    pub fn set_transformed(&mut self, s: String) {
        self.transformed = s;
    }

    /// Replace transformed buffer content
    #[inline]
    pub fn replace_transformed(&mut self, s: &str) {
        self.transformed.clear();
        self.transformed.push_str(s);
    }

    /// Pop last character from raw buffer
    #[inline]
    pub fn pop_raw(&mut self) -> Option<char> {
        self.raw.pop()
    }

    /// Pop last character from transformed buffer
    #[inline]
    pub fn pop_transformed(&mut self) -> Option<char> {
        self.transformed.pop()
    }

    /// Clear both buffers
    #[inline]
    pub fn clear(&mut self) {
        self.raw.clear();
        self.transformed.clear();
    }

    // ===== Utility =====

    /// Get character consumption count (raw - transformed)
    /// Used for restore decision: if >= 2, likely English being typed as VN
    #[inline]
    pub fn char_consumption(&self) -> i32 {
        self.raw_len() as i32 - self.transformed_len() as i32
    }

    /// Check if raw and transformed are identical
    #[inline]
    pub fn is_unchanged(&self) -> bool {
        self.raw == self.transformed
    }

    /// Get last character of raw buffer
    #[inline]
    pub fn last_raw(&self) -> Option<char> {
        self.raw.chars().last()
    }

    /// Get last character of transformed buffer
    #[inline]
    pub fn last_transformed(&self) -> Option<char> {
        self.transformed.chars().last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_is_empty() {
        let buf = Buffer::new();
        assert!(buf.is_empty());
        assert_eq!(buf.raw(), "");
        assert_eq!(buf.transformed(), "");
        assert_eq!(buf.raw_len(), 0);
        assert_eq!(buf.transformed_len(), 0);
    }

    #[test]
    fn test_push_raw() {
        let mut buf = Buffer::new();
        buf.push_raw('a');
        buf.push_raw('b');
        assert_eq!(buf.raw(), "ab");
        assert_eq!(buf.raw_len(), 2);
    }

    #[test]
    fn test_push_transformed() {
        let mut buf = Buffer::new();
        buf.push_transformed('a');
        buf.push_transformed('\u{00e1}'); // á
        assert_eq!(buf.transformed(), "a\u{00e1}");
        assert_eq!(buf.transformed_len(), 2);
    }

    #[test]
    fn test_set_transformed() {
        let mut buf = Buffer::new();
        buf.push_raw('v');
        buf.push_raw('i');
        buf.set_transformed("vi".to_string());
        assert_eq!(buf.transformed(), "vi");
    }

    #[test]
    fn test_replace_transformed() {
        let mut buf = Buffer::new();
        buf.push_transformed('a');
        buf.replace_transformed("bc");
        assert_eq!(buf.transformed(), "bc");
    }

    #[test]
    fn test_pop_raw() {
        let mut buf = Buffer::new();
        buf.push_raw('a');
        buf.push_raw('b');
        assert_eq!(buf.pop_raw(), Some('b'));
        assert_eq!(buf.raw(), "a");
    }

    #[test]
    fn test_pop_transformed() {
        let mut buf = Buffer::new();
        buf.push_transformed('a');
        buf.push_transformed('\u{00e1}'); // á
        assert_eq!(buf.pop_transformed(), Some('\u{00e1}'));
        assert_eq!(buf.transformed(), "a");
    }

    #[test]
    fn test_clear() {
        let mut buf = Buffer::new();
        buf.push_raw('a');
        buf.push_transformed('a');
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.raw(), "");
        assert_eq!(buf.transformed(), "");
    }

    #[test]
    fn test_char_consumption() {
        let mut buf = Buffer::new();

        // Same length: 0 consumption
        buf.push_raw('v');
        buf.push_raw('i');
        buf.set_transformed("vi".to_string());
        assert_eq!(buf.char_consumption(), 0);

        // Raw longer: positive consumption
        buf.clear();
        buf.push_raw('v');
        buf.push_raw('i');
        buf.push_raw('e');
        buf.push_raw('e');
        buf.push_raw('j');
        buf.push_raw('t');
        buf.set_transformed("viet".to_string()); // "vieejt" -> "viet"
        assert_eq!(buf.char_consumption(), 2); // 6 - 4 = 2
    }

    #[test]
    fn test_is_unchanged() {
        let mut buf = Buffer::new();

        buf.push_raw('a');
        buf.push_raw('b');
        buf.set_transformed("ab".to_string());
        assert!(buf.is_unchanged());

        buf.set_transformed("a".to_string());
        assert!(!buf.is_unchanged());
    }

    #[test]
    fn test_last_char() {
        let mut buf = Buffer::new();

        assert_eq!(buf.last_raw(), None);
        assert_eq!(buf.last_transformed(), None);

        buf.push_raw('a');
        buf.push_raw('b');
        buf.push_transformed('x');
        buf.push_transformed('\u{1ec7}'); // ệ

        assert_eq!(buf.last_raw(), Some('b'));
        assert_eq!(buf.last_transformed(), Some('\u{1ec7}'));
    }

    #[test]
    fn test_unicode_len() {
        let mut buf = Buffer::new();

        // Vietnamese chars are multi-byte but count as 1 char
        // vi\u{1ec7}t = việt (4 chars)
        // Byte breakdown: 'v'(1) + 'i'(1) + 'ệ'(3) + 't'(1) = 6 bytes
        buf.set_transformed("vi\u{1ec7}t".to_string());
        assert_eq!(buf.transformed_len(), 4); // Not byte length!
        assert_eq!(buf.transformed().len(), 6); // Byte length (ệ is 3 bytes)
    }

    #[test]
    fn test_with_capacity() {
        let buf = Buffer::with_capacity(64);
        assert!(buf.is_empty());
        // Capacity is set but buffer is empty
    }
}
