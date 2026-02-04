//! Vietnamese Language Data Modules
//!
//! This module contains all linguistic data for Vietnamese input:
//! - `keys`: Virtual keycode definitions (platform-specific)
//! - `chars`: Unicode character conversion (includes tone/mark constants)
//! - `vowel`: Vietnamese vowel phonology system
//! - `telex_doubles`: English words with Telex double patterns for auto-restore

pub mod chars;
pub mod constants;
pub mod english_dict;
pub mod keys;
pub mod telex_doubles;
pub mod vietnamese_spellcheck;
pub mod vowel;

pub use chars::{get_d, mark, to_char, tone};
pub use constants::*;
pub use keys::{is_break, is_letter, is_vowel};
pub use vowel::{Modifier, Phonology, Role, Vowel};
