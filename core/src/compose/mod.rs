//! Typing core v2: raw keys in, display text out. Pure and allocation-free.
//!
//! One word = a fixed array of raw keys. Every ambiguous key (a e o w d s f r x j z, VNI digits)
//! is either a letter or a modifier; instead of deciding greedily and undoing later, `Compose`
//! keeps a small beam of live interpretations and lets `phonology::validate` prune the ones that
//! can never become Vietnamese. The display is the best surviving interpretation, so typing
//! order never matters and nothing has to be reverted.
//!
//! Layers: `method` (key → intent) → `parse` (interpretations) → `lattice` (beam, select)
//! → `render` (letters → text) → `diff` (text → backspaces + new chars).

pub mod diff;
pub mod lattice;
pub mod method;
pub mod parse;
pub mod render;

pub use lattice::Compose;
pub use method::Method;
pub use render::Display;

/// Longest word the lattice tracks; longer input is shown raw.
pub const MAXK: usize = 24;

/// One physical key of the current word: lowercase ASCII letter, digit or bracket, plus case.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RawKey {
    pub ch: u8,
    pub caps: bool,
}

/// Behaviour switches of the typing core (all become parameters of rules, never branches).
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub method: Method,
    /// oà/uý (true) or òa/úy (false)
    pub modern_tone: bool,
    /// skip validation (free tone placement)
    pub free: bool,
    /// accept f j w z as initials
    pub foreign_initials: bool,
    /// `w` alone becomes ư (Telex)
    pub w_as_vowel: bool,
    /// Telex `]` → ư and `[` → ơ
    pub bracket: bool,
    /// English auto-restore is on: a word with no Vietnamese reading is shown as typed unless a
    /// stroke or horn proves Vietnamese intent (the restore at the boundary decides the rest)
    pub english_guard: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            method: Method::Telex,
            modern_tone: true,
            free: false,
            foreign_initials: false,
            w_as_vowel: true,
            bracket: false,
            english_guard: false,
        }
    }
}
