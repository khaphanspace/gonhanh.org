//! Static data of the engine.
//!
//! - `keys`: virtual keycode definitions (platform-specific)
//! - `lexicon`: the word lists (English, English reference, Vietnamese syllables, keep words,
//!   Telex doubles), packed at build time (`core/build.rs`) and searched in place
//! - `english_dict`, `telex_doubles`: thin lookups on those lists

pub mod english_dict;
pub mod keys;
pub mod lexicon;
pub mod telex_doubles;

pub use keys::{is_break, is_letter, is_vowel};
