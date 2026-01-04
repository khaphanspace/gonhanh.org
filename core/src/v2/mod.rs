//! Engine V2 - Vietnamese IME (Spec v3.7)
//!
//! Implements the 7-step keystroke pipeline:
//! 0. Key Classification
//! 1. Pre-check (foreign mode)
//! 2. Dispatch & Execute
//! 3. Tone/Mark Placement
//! 4. Buffer Update
//! 5. 9-Layer Validation
//! 6. Restore Decision
//! 7. Output Generation

pub mod bitmask;
pub mod buffer;
pub mod classify;
pub mod dict;
pub mod dispatch;
pub mod engine;
pub mod output;
pub mod placement;
pub mod precheck;
pub mod restore;
pub mod state;
pub mod types;
pub mod validate;

#[cfg(test)]
mod tests;

// Re-exports
pub use bitmask::{char_idx, get_base_vowel, get_tone, is_vn_vowel};
pub use buffer::Buffer;
pub use classify::{classify_key, Method};
pub use dict::Dict;
pub use dispatch::dispatch;
pub use engine::{Action, Engine, Result, FLAG_KEY_CONSUMED, MAX};
pub use output::generate_output;
pub use placement::{
    apply_breve, apply_circumflex, apply_horn, apply_stroke, apply_tone_to_vowel, detect_context,
    extract_vowels, find_tone_position, find_tone_position_with_context, has_final_consonant,
    replace_char_at, telex_key_to_tone, PlacementContext, VowelInfo,
};
pub use precheck::{pre_check, Mode};
pub use restore::{is_english, should_restore, Decision};
pub use state::{BufferState, VnState};
pub use types::*;
pub use validate::{parse_syllable, validate_vn, Syllable};
