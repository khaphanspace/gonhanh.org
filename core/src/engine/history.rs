//! Committed words, so Backspace right after a space can bring the word back for editing.

use crate::compose::{Display, RawKey, MAXK};

pub const CAPACITY: usize = 10;

/// A word as it was committed: the keys (to resume editing) and the text left on screen.
#[derive(Clone, Copy)]
pub struct Entry {
    pub keys: [RawKey; MAXK],
    pub len: u8,
    pub shown: Display,
    /// the word was restored to its raw spelling: replay keys as plain letters
    pub literal: bool,
}

pub struct History {
    data: [Option<Entry>; CAPACITY],
    head: usize,
    len: usize,
}

impl History {
    pub const fn new() -> Self {
        History {
            data: [None; CAPACITY],
            head: 0,
            len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, e: Entry) {
        self.data[self.head] = Some(e);
        self.head = (self.head + 1) % CAPACITY;
        self.len = (self.len + 1).min(CAPACITY);
    }

    pub fn pop(&mut self) -> Option<Entry> {
        if self.len == 0 {
            return None;
        }
        self.head = (self.head + CAPACITY - 1) % CAPACITY;
        self.len -= 1;
        self.data[self.head].take()
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.head = 0;
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
