//! Dual buffer system for V3 engine
//!
//! Two buffers are maintained:
//! 1. Raw buffer: Original keystrokes (for English restore)
//! 2. Transformed buffer: Vietnamese output
//!
//! ## Critical: Dual Restore System
//!
//! - `restore()`: Synced version (consumed keys EXCLUDED) - for VN sync
//! - `restore_all()`: Full raw (consumed keys INCLUDED) - for EN detection

/// Maximum buffer size
pub const MAX_BUFFER_SIZE: usize = 32;

/// Raw keystroke with metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawKeystroke {
    /// Original key
    key: char,
    /// Whether key was consumed by transformation
    consumed: bool,
    /// Whether key is uppercase
    uppercase: bool,
}

impl RawKeystroke {
    /// Create new keystroke
    pub fn new(key: char) -> Self {
        Self {
            key: key.to_ascii_lowercase(),
            consumed: false,
            uppercase: key.is_ascii_uppercase(),
        }
    }

    /// Get the key
    pub fn key(&self) -> char {
        self.key
    }

    /// Check if consumed
    pub fn consumed(&self) -> bool {
        self.consumed
    }

    /// Check if uppercase
    pub fn uppercase(&self) -> bool {
        self.uppercase
    }

    /// Mark as consumed
    pub fn consume(&mut self) {
        self.consumed = true;
    }

    /// Mark as not consumed (for revert)
    pub fn unconsume(&mut self) {
        self.consumed = false;
    }
}

/// Pending pop state for mark revert handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingPop {
    /// Whether pending pop is active
    pub active: bool,
    /// The consumed modifier key (e.g., 's' for tone)
    pub consumed_key: char,
    /// The key that triggered revert
    pub revert_key: char,
}

impl PendingPop {
    /// Create new inactive pending pop
    pub fn new() -> Self {
        Self {
            active: false,
            consumed_key: '\0',
            revert_key: '\0',
        }
    }

    /// Set pending pop active
    pub fn set(&mut self, consumed_key: char, revert_key: char) {
        self.active = true;
        self.consumed_key = consumed_key;
        self.revert_key = revert_key;
    }

    /// Clear pending pop
    pub fn clear(&mut self) {
        self.active = false;
    }
}

impl Default for PendingPop {
    fn default() -> Self {
        Self::new()
    }
}

/// Raw buffer for keystroke tracking
#[derive(Debug, Clone)]
pub struct RawBuffer {
    data: [RawKeystroke; MAX_BUFFER_SIZE],
    len: usize,
    /// Pending pop state
    pub pending_pop: PendingPop,
}

impl RawBuffer {
    /// Create new raw buffer
    pub fn new() -> Self {
        Self {
            data: [RawKeystroke::new('\0'); MAX_BUFFER_SIZE],
            len: 0,
            pending_pop: PendingPop::new(),
        }
    }

    /// Push a keystroke
    pub fn push(&mut self, key: char) {
        if self.len < MAX_BUFFER_SIZE {
            // Clear pending pop when new key is pushed (user continued typing after revert)
            self.pending_pop.active = false;
            self.data[self.len] = RawKeystroke::new(key);
            self.len += 1;
        }
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.len = 0;
        self.pending_pop.clear();
    }

    /// Pop last keystroke
    pub fn pop(&mut self) -> Option<RawKeystroke> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.data[self.len])
        } else {
            None
        }
    }

    /// Check if key is consonant (for pending pop decision)
    fn is_consonant(key: char) -> bool {
        matches!(
            key,
            'b' | 'c' | 'd' | 'g' | 'h' | 'k' | 'l' | 'm' | 'n' | 'p' | 'q' | 'r' | 't' | 'v' | 'x'
        )
    }

    /// Handle pending pop on next keystroke
    /// Called BEFORE processing the current keystroke
    ///
    /// Logic:
    /// - If pending_pop is active and current key is CONSONANT:
    ///   Pop: consumed, revert, current → push: revert, current
    /// - Exception: Double 'f' never pops
    pub fn handle_pending_pop(&mut self, current_key: char) {
        if !self.pending_pop.active {
            return;
        }
        self.pending_pop.active = false;

        // Only pop if current key is CONSONANT
        if !Self::is_consonant(current_key) {
            return;
        }

        // Exception: Double 'f' - never pop (off, offer, office, coffee)
        if self.pending_pop.consumed_key == 'f' && self.pending_pop.revert_key == 'f' {
            return;
        }

        // Pop consumed modifier from raw
        // raw: [..., consumed, revert, current]
        // want: [..., revert, current]
        if self.len >= 2 {
            let revert = self.pop().unwrap();
            self.pop(); // consumed, discard
            self.push(revert.key);
        }
    }

    /// Handle pending pop on SPACE (word boundary)
    /// Called when space/boundary is detected
    pub fn handle_pending_pop_space(&mut self) {
        if !self.pending_pop.active {
            return;
        }
        self.pending_pop.active = false;

        // Exception: Double 'f' - never pop
        if self.pending_pop.consumed_key == 'f' && self.pending_pop.revert_key == 'f' {
            return;
        }

        // Pop consumed modifier from raw
        // raw: [..., consumed, revert]
        // want: [..., revert]
        if self.len >= 2 {
            let revert = self.pop().unwrap();
            self.pop(); // consumed, discard
            self.push(revert.key);
        }
    }

    /// Set pending pop after mark revert
    pub fn set_pending_pop(&mut self, consumed_key: char, revert_key: char) {
        self.pending_pop.set(consumed_key, revert_key);
    }

    /// Get last keystroke (mutable)
    pub fn last_mut(&mut self) -> Option<&mut RawKeystroke> {
        if self.len > 0 {
            Some(&mut self.data[self.len - 1])
        } else {
            None
        }
    }

    /// Get keystroke at index (mutable)
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut RawKeystroke> {
        if idx < self.len {
            Some(&mut self.data[idx])
        } else {
            None
        }
    }

    /// restore() - Synced version (consumed keys EXCLUDED)
    /// Used for Vietnamese raw sync after revert
    pub fn restore(&self) -> String {
        self.data[..self.len]
            .iter()
            .filter(|k| !k.consumed())
            .map(|k| {
                if k.uppercase() {
                    k.key().to_ascii_uppercase()
                } else {
                    k.key()
                }
            })
            .collect()
    }

    /// restore_all() - FULL raw (consumed keys INCLUDED)
    /// Used for English word detection
    pub fn restore_all(&self) -> String {
        self.data[..self.len]
            .iter()
            .map(|k| {
                if k.uppercase() {
                    k.key().to_ascii_uppercase()
                } else {
                    k.key()
                }
            })
            .collect()
    }
}

impl Default for RawBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Transform type for tracking last transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum XformType {
    /// No transformation
    #[default]
    None,
    /// Tone applied (sắc, huyền, hỏi, ngã, nặng)
    Tone,
    /// Vowel mark applied (circumflex, horn, breve)
    Mark,
    /// Stroke applied (đ)
    Stroke,
}

/// Transform tracking for revert detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformTrack {
    /// Type of last transformation
    pub xform_type: XformType,
    /// Key that triggered the transform (for double-key revert)
    pub trigger_key: char,
    /// Consumed key (for pending pop)
    pub consumed_key: char,
    /// Position of transformed character in buffer
    pub position: usize,
    /// Tone value (1-5) if tone was applied
    pub tone_value: u8,
    /// Mark value (1-3) if mark was applied
    pub mark_value: u8,
}

impl TransformTrack {
    /// Create new empty transform track
    pub fn new() -> Self {
        Self {
            xform_type: XformType::None,
            trigger_key: '\0',
            consumed_key: '\0',
            position: 0,
            tone_value: 0,
            mark_value: 0,
        }
    }

    /// Record tone transformation
    pub fn record_tone(&mut self, key: char, position: usize, tone: u8) {
        self.xform_type = XformType::Tone;
        self.trigger_key = key;
        self.consumed_key = key;
        self.position = position;
        self.tone_value = tone;
    }

    /// Record mark transformation
    pub fn record_mark(&mut self, key: char, position: usize, mark: u8) {
        self.xform_type = XformType::Mark;
        self.trigger_key = key;
        self.consumed_key = key;
        self.position = position;
        self.mark_value = mark;
    }

    /// Record stroke transformation
    pub fn record_stroke(&mut self, key: char, position: usize) {
        self.xform_type = XformType::Stroke;
        self.trigger_key = key;
        self.consumed_key = key;
        self.position = position;
    }

    /// Clear tracking
    pub fn clear(&mut self) {
        self.xform_type = XformType::None;
        self.trigger_key = '\0';
        self.consumed_key = '\0';
        self.position = 0;
        self.tone_value = 0;
        self.mark_value = 0;
    }

    /// Check if should revert (same key pressed twice)
    pub fn should_revert(&self, key: char) -> bool {
        self.xform_type != XformType::None && self.trigger_key == key
    }
}

impl Default for TransformTrack {
    fn default() -> Self {
        Self::new()
    }
}

/// Dual buffer: raw + transformed
#[derive(Debug, Clone)]
pub struct DualBuffer {
    /// Raw keystrokes
    raw: RawBuffer,
    /// Transformed output
    transformed: String,
    /// Transform tracking for revert
    pub track: TransformTrack,
    /// Has any transformation occurred
    pub has_transform: bool,
    /// Has stroke (đ) - indicates intentional Vietnamese
    pub has_stroke: bool,
}

impl DualBuffer {
    /// Create new dual buffer
    pub fn new() -> Self {
        Self {
            raw: RawBuffer::new(),
            transformed: String::with_capacity(MAX_BUFFER_SIZE),
            track: TransformTrack::new(),
            has_transform: false,
            has_stroke: false,
        }
    }

    /// Clear both buffers
    pub fn clear(&mut self) {
        self.raw.clear();
        self.transformed.clear();
        self.track.clear();
        self.has_transform = false;
        self.has_stroke = false;
    }

    /// Push key to raw buffer
    pub fn push_raw(&mut self, key: char) {
        self.raw.push(key);
    }

    /// Push char to transformed buffer
    pub fn push_transformed(&mut self, c: char) {
        self.transformed.push(c);
    }

    /// Set transformed buffer
    pub fn set_transformed(&mut self, s: &str) {
        self.transformed.clear();
        self.transformed.push_str(s);
    }

    /// Get transformed content
    pub fn transformed(&self) -> String {
        self.transformed.clone()
    }

    /// Get raw content (excluding consumed)
    pub fn raw(&self) -> String {
        self.raw.restore()
    }

    /// Get raw content (including consumed)
    pub fn raw_all(&self) -> String {
        self.raw.restore_all()
    }

    /// Get raw buffer reference
    pub fn raw_buffer(&self) -> &RawBuffer {
        &self.raw
    }

    /// Get raw buffer mutable reference
    pub fn raw_buffer_mut(&mut self) -> &mut RawBuffer {
        &mut self.raw
    }

    /// Check if buffers differ (transformation occurred)
    pub fn has_difference(&self) -> bool {
        self.raw.restore_all() != self.transformed
    }
}

impl Default for DualBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_keystroke() {
        let mut k = RawKeystroke::new('A');
        assert_eq!(k.key(), 'a'); // lowercase
        assert!(k.uppercase());
        assert!(!k.consumed());

        k.consume();
        assert!(k.consumed());

        k.unconsume();
        assert!(!k.consumed());
    }

    #[test]
    fn test_raw_buffer_restore() {
        let mut buf = RawBuffer::new();
        buf.push('i');
        buf.push('s');
        buf.push('s');

        // Simulate revert: first 's' unconsume, second 's' consume
        buf.get_mut(1).unwrap().unconsume(); // already not consumed
        buf.get_mut(2).unwrap().consume();

        assert_eq!(buf.restore(), "is"); // skip consumed
        assert_eq!(buf.restore_all(), "iss"); // include all
    }

    #[test]
    fn test_dual_buffer() {
        let mut buf = DualBuffer::new();
        buf.push_raw('t');
        buf.push_raw('e');
        buf.push_raw('s');
        buf.push_raw('t');
        buf.set_transformed("tét");

        assert_eq!(buf.raw_all(), "test");
        assert_eq!(buf.transformed(), "tét");
        assert!(buf.has_difference());
    }

    #[test]
    fn test_issue_pattern() {
        // Simulate "issue" typing with revert
        let mut buf = RawBuffer::new();
        buf.push('i');
        buf.push('s'); // tone applied (consumed=true later via unconsume/consume dance)
        buf.push('s'); // revert trigger
        buf.push('u');
        buf.push('e');

        // After revert: first 's' unconsume (literal), second 's' consume (trigger)
        // buf.data[1] stays not consumed (literal 's')
        buf.get_mut(2).unwrap().consume(); // trigger 's'

        assert_eq!(buf.restore(), "isue"); // skip consumed
        assert_eq!(buf.restore_all(), "issue"); // include all
    }

    // ============================================
    // Phase 05: Raw Buffer Management Tests
    // ============================================

    #[test]
    fn test_pending_pop_struct() {
        let mut pp = PendingPop::new();
        assert!(!pp.active);

        pp.set('s', 's');
        assert!(pp.active);
        assert_eq!(pp.consumed_key, 's');
        assert_eq!(pp.revert_key, 's');

        pp.clear();
        assert!(!pp.active);
    }

    #[test]
    fn test_raw_pop_consonant_after_revert() {
        // "tesst" → raw should be [t,e,s,t] after pending pop
        // Step 1-3: type "tes" → raw=[t,e,s]
        // Step 4: type 's' → mark revert, raw=[t,e,s,s], pending_pop active
        // Step 5: type 't' → consonant triggers pop → raw=[t,e,s,t]

        let mut buf = RawBuffer::new();
        buf.push('t');
        buf.push('e');
        buf.push('s'); // first s - was consumed for tone
        buf.push('s'); // second s - revert trigger

        // Simulate: set pending pop (consumed='s', revert='s')
        buf.set_pending_pop('s', 's');

        // Now type 't' - consonant should trigger pop
        buf.handle_pending_pop('t');
        buf.push('t');

        assert_eq!(buf.restore_all(), "test");
    }

    #[test]
    fn test_raw_no_pop_vowel_after_revert() {
        // "issue" → raw should stay [i,s,s,u,e]
        // 'u' is vowel, not consonant → NO pop

        let mut buf = RawBuffer::new();
        buf.push('i');
        buf.push('s'); // consumed for tone
        buf.push('s'); // revert trigger

        // Set pending pop
        buf.set_pending_pop('s', 's');

        // Type 'u' - vowel should NOT trigger pop
        buf.handle_pending_pop('u');
        buf.push('u');
        buf.push('e');

        assert_eq!(buf.restore_all(), "issue");
    }

    #[test]
    fn test_raw_double_f_exception() {
        // "off " → raw should stay [o,f,f]
        // Double 'f' is common in English, NEVER pop

        let mut buf = RawBuffer::new();
        buf.push('o');
        buf.push('f'); // consumed for tone
        buf.push('f'); // revert trigger

        // Set pending pop for double f
        buf.set_pending_pop('f', 'f');

        // Space triggers pending_pop_space
        buf.handle_pending_pop_space();

        // Should NOT pop due to double-f exception
        assert_eq!(buf.restore_all(), "off");
    }

    #[test]
    fn test_raw_pop_on_space() {
        // "simss " → after space, raw should be [s,i,m,s]
        // (not double-f, so should pop)

        let mut buf = RawBuffer::new();
        buf.push('s');
        buf.push('i');
        buf.push('m');
        buf.push('s'); // consumed for tone
        buf.push('s'); // revert trigger

        // Set pending pop
        buf.set_pending_pop('s', 's');

        // Space should trigger pop
        buf.handle_pending_pop_space();

        assert_eq!(buf.restore_all(), "sims");
    }

    #[test]
    fn test_pending_pop_cleared_on_buffer_clear() {
        let mut buf = RawBuffer::new();
        buf.push('t');
        buf.push('e');
        buf.push('s');
        buf.set_pending_pop('s', 's');

        assert!(buf.pending_pop.active);

        buf.clear();

        assert!(!buf.pending_pop.active);
        assert_eq!(buf.len(), 0);
    }
}
