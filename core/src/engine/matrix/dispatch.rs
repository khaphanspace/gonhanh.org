//! U3: Dispatch Matrix
//!
//! State × Category → (Action, NextState)
//!
//! This is the core of the matrix-based engine. Every keystroke
//! results in a single lookup to determine:
//! 1. What action to take (PASS, TONE, MARK, STROKE, etc.)
//! 2. What state to transition to
//!
//! Memory: 5 states × 8 categories = 40 bytes

use super::{act, pack, st, unpack};

/// Dispatch table: DISPATCH[state][category] → packed(action, next_state)
///
/// Table layout:
/// ```text
/// State\Cat  | VOWEL | INIT_ONLY | INIT_FINAL | FINAL_PART | SPECIAL_W | TONE_KEY | D_KEY | OTHER
/// -----------|-------|-----------|------------|------------|-----------|----------|-------|-------
/// EMPTY      | VOW   | INIT      | INIT       | INIT       | VOW*      | INIT     | INIT  | -
/// INIT       | VOW   | INIT      | INIT       | INIT       | VOW*      | INIT     | INIT  | -
/// VOW        | VOW   | FIN       | FIN        | FIN        | MARK      | TONE     | FIN   | -
/// DIA        | VOW   | FIN       | FIN        | FIN        | MARK      | TONE*    | FIN   | -
/// FIN        | VOW   | INIT      | FIN        | FIN        | -         | FIN      | INIT  | -
/// ```
///
/// Notes:
/// - VOW*: W in EMPTY/INIT creates vowel ư (special handling)
/// - TONE*: In DIA state, tone key may replace existing tone
/// - Actual behavior depends on additional context (e.g., what vowels exist)
pub static DISPATCH: [[u8; 8]; 5] = [
    // State: EMPTY
    [
        pack(act::PASS, st::VOW),     // VOWEL → append, go to VOW
        pack(act::PASS, st::INIT),    // INIT_ONLY → append, go to INIT
        pack(act::PASS, st::INIT),    // INIT_FINAL → append, go to INIT
        pack(act::PASS, st::INIT),    // FINAL_PART → append, go to INIT
        pack(act::PASS, st::VOW),     // SPECIAL_W → becomes ư, go to VOW
        pack(act::PASS, st::INIT),    // TONE_KEY → treat as consonant
        pack(act::PASS, st::INIT),    // D_KEY → append d, go to INIT
        pack(act::REJECT, st::EMPTY), // OTHER → reject
    ],
    // State: INIT (has initial consonant)
    [
        pack(act::PASS, st::VOW),    // VOWEL → append, go to VOW
        pack(act::PASS, st::INIT),   // INIT_ONLY → append to cluster
        pack(act::PASS, st::INIT),   // INIT_FINAL → append to cluster
        pack(act::PASS, st::INIT),   // FINAL_PART → append (gh, ng, etc.)
        pack(act::PASS, st::VOW),    // SPECIAL_W → becomes ư or part of qu
        pack(act::PASS, st::INIT),   // TONE_KEY → treat as consonant (tr, etc.)
        pack(act::STROKE, st::INIT), // D_KEY → try stroke (dd → đ)
        pack(act::REJECT, st::INIT), // OTHER → reject
    ],
    // State: VOW (has vowel, ready for tone/mark/final)
    [
        pack(act::PASS, st::VOW),    // VOWEL → append (compound vowel)
        pack(act::PASS, st::FIN),    // INIT_ONLY → becomes final (rare)
        pack(act::PASS, st::FIN),    // INIT_FINAL → becomes final
        pack(act::PASS, st::FIN),    // FINAL_PART → becomes final (ng, ch)
        pack(act::MARK, st::DIA),    // SPECIAL_W → apply mark (horn/breve)
        pack(act::TONE, st::DIA),    // TONE_KEY → apply tone
        pack(act::STROKE, st::FIN),  // D_KEY → try stroke, else becomes final
        pack(act::REJECT, st::VOW),  // OTHER → reject
    ],
    // State: DIA (has diacritic)
    [
        pack(act::PASS, st::VOW),    // VOWEL → append (may shift diacritic)
        pack(act::PASS, st::FIN),    // INIT_ONLY → becomes final
        pack(act::PASS, st::FIN),    // INIT_FINAL → becomes final
        pack(act::PASS, st::FIN),    // FINAL_PART → becomes final
        pack(act::MARK, st::DIA),    // SPECIAL_W → apply/replace mark
        pack(act::TONE, st::DIA),    // TONE_KEY → replace tone
        pack(act::STROKE, st::FIN),  // D_KEY → try stroke, else becomes final
        pack(act::REJECT, st::DIA),  // OTHER → reject
    ],
    // State: FIN (has final consonant)
    [
        pack(act::PASS, st::VOW),    // VOWEL → start new syllable (rare in VN)
        pack(act::PASS, st::INIT),   // INIT_ONLY → start new syllable
        pack(act::PASS, st::FIN),    // INIT_FINAL → extend final (ng, ch)
        pack(act::PASS, st::FIN),    // FINAL_PART → extend final (ng, nh)
        pack(act::MARK, st::FIN),    // SPECIAL_W → apply horn/breve (for ung→ưng)
        pack(act::TONE, st::FIN),    // TONE_KEY → apply tone (toans → toán)
        pack(act::STROKE, st::FIN),  // D_KEY → try stroke (dojcd → đọc)
        pack(act::REJECT, st::FIN),  // OTHER → reject
    ],
];

/// Perform dispatch lookup
///
/// Returns (action, next_state) for the given state and key category.
/// This is the hot path - must be O(1) with no branches.
#[inline]
pub fn dispatch(state: u8, category: u8) -> (u8, u8) {
    let packed = DISPATCH[state as usize][category as usize];
    unpack(packed)
}

/// Dispatch with key lookup
///
/// Convenience function that combines category lookup and dispatch.
#[inline]
pub fn dispatch_key(state: u8, key: u16) -> (u8, u8) {
    let category = super::get_key_category(key);
    dispatch(state, category)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::keys;
    use crate::engine::matrix::cat;

    #[test]
    fn test_dispatch_table_size() {
        assert_eq!(DISPATCH.len(), 5); // 5 states
        for row in &DISPATCH {
            assert_eq!(row.len(), 8); // 8 categories
        }
    }

    #[test]
    fn test_empty_state_vowel() {
        // EMPTY + VOWEL → PASS to VOW
        let (action, next_state) = dispatch(st::EMPTY, cat::VOWEL);
        assert_eq!(action, act::PASS);
        assert_eq!(next_state, st::VOW);
    }

    #[test]
    fn test_empty_state_consonant() {
        // EMPTY + INIT_ONLY → PASS to INIT
        let (action, next_state) = dispatch(st::EMPTY, cat::INIT_ONLY);
        assert_eq!(action, act::PASS);
        assert_eq!(next_state, st::INIT);
    }

    #[test]
    fn test_vow_state_tone() {
        // VOW + TONE_KEY → TONE to DIA
        let (action, next_state) = dispatch(st::VOW, cat::TONE_KEY);
        assert_eq!(action, act::TONE);
        assert_eq!(next_state, st::DIA);
    }

    #[test]
    fn test_vow_state_mark() {
        // VOW + SPECIAL_W → MARK to DIA
        let (action, next_state) = dispatch(st::VOW, cat::SPECIAL_W);
        assert_eq!(action, act::MARK);
        assert_eq!(next_state, st::DIA);
    }

    #[test]
    fn test_init_state_stroke() {
        // INIT + D_KEY → STROKE (dd → đ)
        let (action, next_state) = dispatch(st::INIT, cat::D_KEY);
        assert_eq!(action, act::STROKE);
        assert_eq!(next_state, st::INIT);
    }

    #[test]
    fn test_vow_state_final() {
        // VOW + INIT_FINAL → PASS to FIN
        let (action, next_state) = dispatch(st::VOW, cat::INIT_FINAL);
        assert_eq!(action, act::PASS);
        assert_eq!(next_state, st::FIN);
    }

    #[test]
    fn test_fin_state_extend() {
        // FIN + FINAL_PART → PASS to FIN (extend: n + g = ng)
        let (action, next_state) = dispatch(st::FIN, cat::FINAL_PART);
        assert_eq!(action, act::PASS);
        assert_eq!(next_state, st::FIN);
    }

    #[test]
    fn test_typing_viet() {
        // Simulate typing "viets" (việt with tone s)
        let mut state = st::EMPTY;

        // v (consonant)
        let (action, next) = dispatch_key(state, keys::V);
        assert_eq!(action, act::PASS);
        state = next;
        assert_eq!(state, st::INIT);

        // i (vowel)
        let (action, next) = dispatch_key(state, keys::I);
        assert_eq!(action, act::PASS);
        state = next;
        assert_eq!(state, st::VOW);

        // e (vowel)
        let (action, next) = dispatch_key(state, keys::E);
        assert_eq!(action, act::PASS);
        state = next;
        assert_eq!(state, st::VOW);

        // t (final)
        let (action, next) = dispatch_key(state, keys::T);
        assert_eq!(action, act::PASS);
        state = next;
        assert_eq!(state, st::FIN);

        // s (tone) - in FIN state, applies tone (viets → việt)
        let (action, next) = dispatch_key(state, keys::S);
        assert_eq!(action, act::TONE);
        assert_eq!(next, st::FIN);
    }

    #[test]
    fn test_typing_vietnamese_with_tone() {
        // Simulate typing "vie" then 's' for tone (before final)
        let mut state = st::EMPTY;

        // v (consonant)
        let (_, next) = dispatch_key(state, keys::V);
        state = next;

        // i (vowel)
        let (_, next) = dispatch_key(state, keys::I);
        state = next;

        // e (vowel)
        let (_, next) = dispatch_key(state, keys::E);
        state = next;
        assert_eq!(state, st::VOW);

        // s (tone) - in VOW state, applies tone
        let (action, next) = dispatch_key(state, keys::S);
        assert_eq!(action, act::TONE);
        assert_eq!(next, st::DIA);
    }
}
