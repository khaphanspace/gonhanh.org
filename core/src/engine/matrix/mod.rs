//! Matrix-based Vietnamese IME Engine V2
//!
//! Zero if-else in hot path. Every decision = matrix lookup.
//!
//! ## Architecture
//!
//! ```text
//! Input → Classify (matrix) → Dispatch (matrix) → Execute → Done
//! ```
//!
//! ## Tables
//!
//! - **U1: LETTER_CLASS** (26B) - Bit flags: V|I|F|S
//! - **U2: KEY_CAT** - Key → category mapping
//! - **U3: DISPATCH** - State × Category → Action|State
//! - **U4: DEFER** - Pending resolution rules
//! - **U5: REVERT_KEY** - Transform → revert trigger

pub mod defer;
pub mod dispatch;
pub mod english;
pub mod key_category;
pub mod letter_class;
pub mod processor;
pub mod revert;
pub mod validation;

pub use defer::{breve_valid_with_final, defertype, horn_u_valid_with_final, DeferState};
pub use dispatch::{dispatch, dispatch_key, DISPATCH};
pub use english::{
    english_likelihood, english_likelihood_keys, is_impossible_bigram_keys, is_valid_coda_keys,
    is_valid_onset_keys, BloomFilter, EnglishLikelihood,
};
pub use key_category::{
    get_key_category, is_final_key, is_initial_key, is_tone_key, is_vowel_key, tone_key_to_value,
};
pub use letter_class::{
    get_letter_class, is_final_class, is_initial_class, is_special_class, is_vowel_class, lc,
    LETTER_CLASS,
};
pub use processor::{ProcessResult, Processor, RawBuffer, RawKeystroke};
pub use revert::{get_revert_trigger, should_revert, xform, RevertState};
pub use validation::{
    is_valid_final_1, is_valid_final_2, is_valid_initial_1, is_valid_initial_2,
    is_valid_vowel_pattern, validate, MatrixValidation,
};

/// Engine states (5 total)
///
/// State machine flow:
/// ```text
/// EMPTY → INIT → VOW → DIA → FIN
///   ↑       ↓      ↓     ↓     ↓
///   └───────┴──────┴─────┴─────┘ (reset on word boundary)
/// ```
pub mod st {
    /// Buffer empty, awaiting input
    pub const EMPTY: u8 = 0;
    /// Has initial consonant(s), no vowel yet
    pub const INIT: u8 = 1;
    /// Has vowel (can receive tone/mark)
    pub const VOW: u8 = 2;
    /// Has diacritic (tone or mark applied)
    pub const DIA: u8 = 3;
    /// Has final consonant
    pub const FIN: u8 = 4;
}

/// Engine actions
///
/// Packed with next_state in dispatch table:
/// `packed = (action << 4) | next_state`
pub mod act {
    /// Append character to buffer
    pub const PASS: u8 = 0;
    /// Apply tone mark (sắc, huyền, hỏi, ngã, nặng)
    pub const TONE: u8 = 1;
    /// Apply vowel mark (circumflex, horn, breve)
    pub const MARK: u8 = 2;
    /// Apply stroke (d → đ)
    pub const STROKE: u8 = 3;
    /// Reject input (invalid sequence)
    pub const REJECT: u8 = 4;
    /// Double-key revert
    pub const REVERT: u8 = 5;
    /// Defer action (pending resolution)
    pub const DEFER: u8 = 6;
}

/// Key categories for dispatch
///
/// Categories are designed to minimize dispatch table size
/// while preserving all necessary distinctions.
pub mod cat {
    /// Vowel: a, e, i, o, u, y
    pub const VOWEL: u8 = 0;
    /// Initial-only consonant: b, d, g, h, k, l, q, r, v, x, z
    pub const INIT_ONLY: u8 = 1;
    /// Initial+Final consonant: c, m, n, p, t
    pub const INIT_FINAL: u8 = 2;
    /// Final-only: g (ng), h (ch/nh) - context dependent
    pub const FINAL_PART: u8 = 3;
    /// Special: w (vowel ư or modifier horn/breve)
    pub const SPECIAL_W: u8 = 4;
    /// Tone key (telex): s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng
    pub const TONE_KEY: u8 = 5;
    /// D key (dual: consonant or stroke trigger)
    pub const D_KEY: u8 = 6;
    /// Unknown/other
    pub const OTHER: u8 = 7;
}

/// Unpack dispatch result into (action, next_state)
#[inline]
pub const fn unpack(packed: u8) -> (u8, u8) {
    (packed >> 4, packed & 0x0F)
}

/// Pack (action, next_state) into single byte
#[inline]
pub const fn pack(action: u8, state: u8) -> u8 {
    (action << 4) | (state & 0x0F)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack() {
        // Test all action/state combinations
        for action in 0..8 {
            for state in 0..5 {
                let packed = pack(action, state);
                let (a, s) = unpack(packed);
                assert_eq!(a, action);
                assert_eq!(s, state);
            }
        }
    }

    #[test]
    fn test_state_constants() {
        assert_eq!(st::EMPTY, 0);
        assert_eq!(st::INIT, 1);
        assert_eq!(st::VOW, 2);
        assert_eq!(st::DIA, 3);
        assert_eq!(st::FIN, 4);
    }

    #[test]
    fn test_action_constants() {
        // Actions should fit in 4 bits (0-15)
        assert!(act::PASS < 16);
        assert!(act::TONE < 16);
        assert!(act::MARK < 16);
        assert!(act::STROKE < 16);
        assert!(act::REJECT < 16);
        assert!(act::REVERT < 16);
        assert!(act::DEFER < 16);
    }
}
