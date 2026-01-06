//! Vietnamese Language Data Modules
//!
//! This module contains all linguistic data for Vietnamese input:
//! - `keys`: Virtual keycode definitions (platform-specific)
//! - `chars`: Unicode character conversion (includes tone/mark constants)
//! - `vowel`: Vietnamese vowel phonology system
//! - `english_whitelist`: English words that need explicit auto-restore

pub mod chars;
pub mod constants;
pub mod english_whitelist;
pub mod keys;
pub mod vowel;

pub use chars::{get_d, mark, to_char, tone};
pub use constants::*;
pub use keys::{is_break, is_letter, is_vowel};
pub use vowel::{Modifier, Phonology, Role, Vowel};
