//! Data module - chars and keycodes

pub mod chars;
pub mod keys;

pub use chars::{get_d, to_char};
pub use keys::{is_break, is_letter, is_vowel};
