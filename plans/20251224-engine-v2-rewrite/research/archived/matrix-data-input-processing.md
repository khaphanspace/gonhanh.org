# Input Processing Matrices - Complete Rust Data

**Date**: 2025-12-24
**Purpose**: Complete matrix data for random order typing support
**Total Memory**: ~1.8KB

---

## Key Type Classification

```rust
/// Key type indices for matrix lookups
pub mod key_type {
    // Letters a-z (indices 0-25)
    pub const A: u8 = 0;
    pub const B: u8 = 1;
    pub const C: u8 = 2;
    pub const D: u8 = 3;
    pub const E: u8 = 4;
    pub const F: u8 = 5;
    pub const G: u8 = 6;
    pub const H: u8 = 7;
    pub const I: u8 = 8;
    pub const J: u8 = 9;
    pub const K: u8 = 10;
    pub const L: u8 = 11;
    pub const M: u8 = 12;
    pub const N: u8 = 13;
    pub const O: u8 = 14;
    pub const P: u8 = 15;
    pub const Q: u8 = 16;
    pub const R: u8 = 17;
    pub const S: u8 = 18;
    pub const T: u8 = 19;
    pub const U: u8 = 20;
    pub const V: u8 = 21;
    pub const W: u8 = 22;
    pub const X: u8 = 23;
    pub const Y: u8 = 24;
    pub const Z: u8 = 25;

    // Tone keys (Telex mode)
    pub const TONE_SAC: u8 = 26;      // s
    pub const TONE_HUYEN: u8 = 27;    // f
    pub const TONE_HOI: u8 = 28;      // r
    pub const TONE_NGA: u8 = 29;      // x
    pub const TONE_NANG: u8 = 30;     // j
    pub const TONE_NGANG: u8 = 31;    // z (remove tone)

    // Modifier keys (Telex mode)
    pub const MOD_CIRCUM_A: u8 = 32;  // a (for â)
    pub const MOD_CIRCUM_E: u8 = 33;  // e (for ê)
    pub const MOD_CIRCUM_O: u8 = 34;  // o (for ô)
    pub const MOD_HORN: u8 = 35;      // w (for ơ, ư, ă)

    // Special keys
    pub const BACKSPACE: u8 = 36;
    pub const SPACE: u8 = 37;

    pub const MAX: u8 = 38;
}

/// Classify raw key to key type based on input method
pub fn classify_key(key: u16, method: u8, context: KeyContext) -> u8 {
    match method {
        0 => classify_telex(key, context),
        1 => classify_vni(key, context),
        _ => key_type::MAX, // Unknown
    }
}
```

---

## I1: ACTION_DISPATCH Matrix

```rust
/// Actions returned by dispatch matrix
pub mod action {
    pub const PASS_THROUGH: u8 = 0;
    pub const ADD_LETTER: u8 = 1;
    pub const APPLY_TONE: u8 = 2;
    pub const DEFER_TONE: u8 = 3;
    pub const APPLY_MODIFIER: u8 = 4;
    pub const DEFER_MODIFIER: u8 = 5;
    pub const CHECK_REVERT: u8 = 6;
    pub const COMPLETE_WORD: u8 = 7;
}

/// States for input processing
pub mod state {
    pub const EMPTY: u8 = 0;
    pub const HAS_INITIAL: u8 = 1;
    pub const HAS_VOWEL: u8 = 2;
    pub const HAS_MODIFIER: u8 = 3;
    pub const HAS_TONE: u8 = 4;
    pub const HAS_FINAL: u8 = 5;
}

/// Action dispatch matrix: state × key_type → action
/// Size: 6 states × 38 key_types = 228 bytes
pub static I1_ACTION: [[u8; 38]; 6] = [
    // EMPTY state: Only letters pass, tones/mods ignored
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, 0,0,0,0,0,0, 0,0,0,0, 0,7],
    //a b c d e f g h i j k l m n o p q r s t u v w x y z  s f r x j z  A E O W  BS SP

    // HAS_INITIAL state: Letters OK, need vowel before tones
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, 0,0,0,0,0,0, 0,0,0,0, 0,7],

    // HAS_VOWEL state: Tones and modifiers can apply
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, 2,2,2,2,2,2, 4,4,4,4, 0,7],
    //                                                     ^tones       ^mods

    // HAS_MODIFIER state: Tone OK, mod needs revert check
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, 2,2,2,2,2,2, 6,6,6,6, 0,7],
    //                                                                  ^revert check

    // HAS_TONE state: Mod OK, tone needs revert check
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, 6,6,6,6,6,6, 4,4,4,4, 0,7],
    //                                                     ^revert      ^mod OK

    // HAS_FINAL state: Limited letters, revert checks for tones/mods
    [1,0,0,0,1,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,1,0, 6,6,6,6,6,6, 6,6,6,6, 0,7],
    //a     e       i             o         u       y      ^revert       ^revert
];

/// Check if letter is vowel (for state transitions)
pub static IS_VOWEL: [bool; 26] = [
    true,  // a
    false, // b
    false, // c
    false, // d
    true,  // e
    false, // f
    false, // g
    false, // h
    true,  // i
    false, // j
    false, // k
    false, // l
    false, // m
    false, // n
    true,  // o
    false, // p
    false, // q
    false, // r
    false, // s
    false, // t
    true,  // u
    false, // v
    false, // w
    false, // x
    true,  // y
    false, // z
];

/// Check if consonant can be final
pub static IS_FINAL_CONSONANT: [bool; 26] = [
    false, // a
    false, // b
    true,  // c (stop final)
    false, // d
    false, // e
    false, // f
    true,  // g (only in ng)
    true,  // h (only in ch, nh)
    false, // i
    false, // j
    true,  // k (ethnic words)
    false, // l
    true,  // m (nasal final)
    true,  // n (nasal final)
    false, // o
    true,  // p (stop final)
    false, // q
    false, // r
    false, // s
    true,  // t (stop final)
    false, // u
    false, // v
    false, // w
    false, // x
    false, // y
    false, // z
];
```

---

## I2: DEFERRED_RESOLUTION Matrix

```rust
/// Pending transformation types
/// UPDATED: Added CAPITALIZE (Review fix 11.3)
pub mod pending {
    pub const BREVE: u8 = 0;        // aw pattern waiting for final
    pub const U_HORN: u8 = 1;       // uơ pattern waiting for context
    pub const MARK_POP: u8 = 2;     // mark revert waiting for letter type
    pub const CAPITALIZE: u8 = 3;   // NEW: auto-capitalize waiting for letter
}

/// Resolution results
pub mod resolution {
    pub const KEEP_PENDING: u8 = 0;
    pub const APPLY_NOW: u8 = 1;
    pub const CANCEL: u8 = 2;
    pub const POP_RAW: u8 = 3;
}

/// Deferred resolution matrix: pending_type × key_type → resolution
/// Size: 4 pending_types × 38 key_types = 152 bytes (was 114)
pub static I2_DEFERRED: [[u8; 38]; 4] = [
    // PENDING_BREVE: Apply on valid final, cancel on vowel, keep on other consonant
    // Valid finals for breve: c, k, m, n, p, t
    [2,0,1,0,2,0,0,0,2,0,1,0,1,1,2,1,0,0,0,1,2,0,0,0,2,0, 0,0,0,0,0,0, 0,0,0,0, 0,0],
    //a   c     e         i   k   m n o p     t u       y
    //^cancel ^apply    ^cancel ^apply       ^apply ^cancel

    // PENDING_U_HORN: Apply on any final consonant or vowel
    [1,0,1,0,1,0,0,0,1,0,1,0,1,1,1,1,0,0,0,1,1,0,0,0,1,0, 0,0,0,0,0,0, 0,0,0,0, 0,0],
    //a   c     e       i   k   m n o p     t u       y
    //^apply (vowel)        ^apply (final)

    // PENDING_MARK_POP: Pop on consonant, keep on vowel
    [0,3,3,3,0,3,3,3,0,3,3,3,3,3,0,3,3,3,3,3,0,3,3,3,0,3, 0,0,0,0,0,0, 0,0,0,0, 0,0],
    //a b c d e f g h i j k l m n o p q r s t u v w x y z
    //^keep   ^pop      ^keep         ^keep     ^keep

    // PENDING_CAPITALIZE: Apply to any letter, cancel on number/space
    // NEW: Handles auto-capitalize after sentence-ending punctuation
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, 0,0,0,0,0,0, 0,0,0,0, 2,2],
    //a b c d e f g h i j k l m n o p q r s t u v w x y z  tones       mods     BS SP
    //^apply (letter)                                                            ^cancel
];
```

---

## I3: REVERT_LOOKUP Matrix

```rust
/// Transform types for revert tracking
/// UPDATED: Added W_AS_VOWEL, W_SHORTCUT_SKIPPED, SHORT_PATTERN_STROKE (Review fix 11.4)
pub mod transform {
    pub const STROKE_D: u8 = 0;
    pub const TONE_SAC: u8 = 1;
    pub const TONE_HUYEN: u8 = 2;
    pub const TONE_HOI: u8 = 3;
    pub const TONE_NGA: u8 = 4;
    pub const TONE_NANG: u8 = 5;
    pub const MOD_CIRCUM_A: u8 = 6;
    pub const MOD_CIRCUM_E: u8 = 7;
    pub const MOD_CIRCUM_O: u8 = 8;
    pub const MOD_BREVE: u8 = 9;
    pub const MOD_HORN: u8 = 10;
    pub const W_AS_VOWEL: u8 = 11;          // NEW: w→ư conversion
    pub const W_SHORTCUT_SKIPPED: u8 = 12;  // NEW: ww→w revert, prevent re-transform
    pub const SHORT_PATTERN_STROKE: u8 = 13; // NEW: delayed stroke (dadu→đau)
}

/// Revert actions
pub mod revert_action {
    pub const NO_REVERT: u8 = 0;
    pub const REVERT: u8 = 1;
    pub const LOCKED: u8 = 2;  // Already reverted, prevent oscillation
}

/// Revert lookup matrix: transform_type × key_type → revert_action
/// Size: 14 transform_types × 38 key_types = 532 bytes (was 418)
/// Only non-zero entries shown (sparse matrix)
pub static I3_REVERT: [[u8; 38]; 14] = [
    // STROKE_D: d+d reverts (index 3 = D)
    [0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0, 0,0,0,0, 0,0],
    //      d

    // TONE_SAC: s+s reverts (index 18 = S, also check tone key index 26)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0, 1,0,0,0,0,0, 0,0,0,0, 0,0],
    //                                  s                  ^tone_sac

    // TONE_HUYEN: f+f reverts (index 5 = F, also check tone key index 27)
    [0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,1,0,0,0,0, 0,0,0,0, 0,0],
    //          f                                            ^tone_huyen

    // TONE_HOI: r+r reverts (index 17 = R, also check tone key index 28)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0, 0,0,1,0,0,0, 0,0,0,0, 0,0],
    //                                r                        ^tone_hoi

    // TONE_NGA: x+x reverts (index 23 = X, also check tone key index 29)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0, 0,0,0,1,0,0, 0,0,0,0, 0,0],
    //                                            x              ^tone_nga

    // TONE_NANG: j+j reverts (index 9 = J, also check tone key index 30)
    [0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,0,1,0, 0,0,0,0, 0,0],
    //                j                                            ^tone_nang

    // MOD_CIRCUM_A: a+a reverts (index 0 = A, also check mod key index 32)
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0, 1,0,0,0, 0,0],
    //a                                                               ^mod_circum_a

    // MOD_CIRCUM_E: e+e reverts (index 4 = E, also check mod key index 33)
    [0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0, 0,1,0,0, 0,0],
    //        e                                                          ^mod_circum_e

    // MOD_CIRCUM_O: o+o reverts (index 14 = O, also check mod key index 34)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0, 0,0,1,0, 0,0],
    //                          o                                           ^mod_circum_o

    // MOD_BREVE: w+w reverts (index 22 = W, also check mod key index 35)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0, 0,0,0,0,0,0, 0,0,0,1, 0,0],
    //                                          w                              ^mod_horn

    // MOD_HORN: w+w reverts (same as breve, both use W)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0, 0,0,0,0,0,0, 0,0,0,1, 0,0],

    // W_AS_VOWEL: w+w reverts to plain w (ư→w)
    // After "w"→"ư", pressing 'w' again reverts to "w"
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0, 0,0,0,0,0,0, 0,0,0,0, 0,0],
    //                                          w

    // W_SHORTCUT_SKIPPED: All keys pass, no revert (lock state)
    // After ww→w, subsequent w should NOT re-trigger transformation
    [2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, 2,2,2,2,2,2, 2,2,2,2, 2,2],
    // All keys locked - no revert possible, prevents oscillation

    // SHORT_PATTERN_STROKE: d+d reverts, vowel after triggers validity check
    // For delayed stroke (dadu→đau), 'd' reverts stroke, vowel triggers I7 check
    [0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0, 0,0,0,0, 0,0],
    //      d (same as STROKE_D for d+d revert)
];
```

---

## I4: STATE_TRANSITION Matrix

```rust
/// State transition matrix: current_state × (action << 1 | success) → new_state
/// action_result = action * 2 + (1 if success else 0)
/// Size: 6 states × 16 action_results = 96 bytes
pub static I4_TRANSITION: [[u8; 16]; 6] = [
    // EMPTY (0)
    // action:    PASS  ADD   TONE  DEF_T MOD   DEF_M REV   WORD
    // success:   0  1  0  1  0  1  0  1  0  1  0  1  0  1  0  1
    [             0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],
    //               ^ADD_SUCCESS: check vowel → HAS_INIT or HAS_VOWEL (dynamic)

    // HAS_INITIAL (1)
    [             1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0 ],
    //               ^vowel letter → HAS_VOWEL (2)

    // HAS_VOWEL (2)
    [             2, 2, 2, 2, 2, 4, 2, 2, 2, 3, 2, 2, 2, 2, 0, 0 ],
    //                        ^TONE_SUCCESS → HAS_TONE (4)
    //                                 ^MOD_SUCCESS → HAS_MODIFIER (3)

    // HAS_MODIFIER (3)
    [             3, 3, 3, 5, 3, 4, 3, 3, 3, 3, 2, 3, 3, 3, 0, 0 ],
    //               ^final letter → HAS_FINAL (5)
    //                        ^TONE_SUCCESS → HAS_TONE (4)
    //                                       ^REVERT_SUCCESS → HAS_VOWEL (2)

    // HAS_TONE (4)
    [             4, 4, 4, 5, 4, 4, 4, 4, 4, 3, 4, 4, 2, 4, 0, 0 ],
    //               ^final letter → HAS_FINAL (5)
    //                                 ^MOD_SUCCESS → HAS_MODIFIER (3)
    //                                       ^REVERT_SUCCESS → HAS_VOWEL (2)

    // HAS_FINAL (5)
    [             5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0 ],
    // Mostly stays in HAS_FINAL, only WORD resets to EMPTY
];

/// Dynamic state transition for ADD_LETTER based on letter type
#[inline(always)]
pub fn transition_add_letter(current: u8, key_idx: u8) -> u8 {
    if IS_VOWEL[key_idx as usize] {
        state::HAS_VOWEL
    } else if IS_FINAL_CONSONANT[key_idx as usize] && current >= state::HAS_VOWEL {
        state::HAS_FINAL
    } else if current == state::EMPTY {
        state::HAS_INITIAL
    } else {
        current
    }
}
```

---

## I5: MODIFIER_COMPAT Matrix

```rust
/// Modifier compatibility: mod_type × vowel_idx → valid
/// Size: 4 mod_types × 12 vowels = 48 bytes
pub static I5_MODIFIER_COMPAT: [[u8; 12]; 4] = [
    // CIRCUMFLEX: a→â, e→ê, o→ô
    //      a  ă  â  e  ê  i  o  ô  ơ  u  ư  y
    [       1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0 ],

    // BREVE: a→ă only
    [       1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ],

    // HORN: o→ơ, u→ư
    [       0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0 ],

    // NONE: always valid (no-op)
    [       1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 ],
];

/// Vowel index mapping
pub mod vowel_idx {
    pub const A: u8 = 0;
    pub const A_BREVE: u8 = 1;   // ă
    pub const A_CIRCUM: u8 = 2;  // â
    pub const E: u8 = 3;
    pub const E_CIRCUM: u8 = 4;  // ê
    pub const I: u8 = 5;
    pub const O: u8 = 6;
    pub const O_CIRCUM: u8 = 7;  // ô
    pub const O_HORN: u8 = 8;    // ơ
    pub const U: u8 = 9;
    pub const U_HORN: u8 = 10;   // ư
    pub const Y: u8 = 11;
}
```

---

## I6: TONE_COMPAT Matrix

```rust
/// Tone compatibility: tone_idx × pattern_has_stop_final → valid
/// Simplified: only need to check Rule 7 (stop finals limit tones)
/// Size: 6 tones × 2 = 12 bytes
pub static I6_TONE_COMPAT_STOP: [u8; 6] = [
    // Tones:  ngang sắc  huyền hỏi  ngã  nặng
    //         0     1    2     3    4    5
    [          0,    1,   0,    0,   0,   1    ][..],
    // Only sắc(1) and nặng(5) allowed with stop finals
];

/// Full compatibility check
#[inline(always)]
pub fn is_tone_compat(tone_idx: u8, has_stop_final: bool) -> bool {
    if !has_stop_final {
        true // All tones valid for open syllables and non-stop finals
    } else {
        I6_TONE_COMPAT_STOP[tone_idx as usize] == 1
    }
}

/// Check if final is stop (p, t, c, ch)
pub static IS_STOP_FINAL: [bool; 26] = [
    false, // a
    false, // b
    true,  // c (stop)
    false, // d
    false, // e
    false, // f
    false, // g
    false, // h (but ch is stop - handled separately)
    false, // i
    false, // j
    true,  // k (ethnic words, treated as stop)
    false, // l
    false, // m (nasal, not stop)
    false, // n (nasal, not stop)
    false, // o
    true,  // p (stop)
    false, // q
    false, // r
    false, // s
    true,  // t (stop)
    false, // u
    false, // v
    false, // w
    false, // x
    false, // y
    false, // z
];
```

---

## Integrated Processing Example

```rust
use super::*;

pub struct InputProcessor {
    state: u8,
    pending: Option<PendingTransform>,
    last_transform: Option<u8>,
    reverted: bool,
}

#[derive(Clone, Copy)]
pub struct PendingTransform {
    kind: u8,
    position: usize,
}

impl InputProcessor {
    pub fn process_key(&mut self, key: u16, method: u8) -> ProcessResult {
        let key_type = classify_key(key, method, self.get_context());

        // Step 1: Check deferred resolutions
        if let Some(pending) = self.pending {
            let resolution = I2_DEFERRED[pending.kind as usize][key_type as usize];
            match resolution {
                resolution::APPLY_NOW => {
                    self.apply_pending(pending);
                    self.pending = None;
                }
                resolution::CANCEL => {
                    self.pending = None;
                }
                resolution::POP_RAW => {
                    self.pop_raw_input();
                    self.pending = None;
                }
                resolution::KEEP_PENDING => {}
            }
        }

        // Step 2: Dispatch action
        let action = I1_ACTION[self.state as usize][key_type as usize];

        // Step 3: Check revert if applicable
        if action == action::CHECK_REVERT || action == action::APPLY_TONE || action == action::APPLY_MODIFIER {
            if let Some(last) = self.last_transform {
                let revert_action = I3_REVERT[last as usize][key_type as usize];
                if revert_action == revert_action::REVERT && !self.reverted {
                    self.do_revert(last);
                    self.reverted = true;
                    return ProcessResult::Reverted;
                } else if revert_action == revert_action::REVERT && self.reverted {
                    // Already reverted, pass through (oscillation prevention)
                    return ProcessResult::PassThrough;
                }
            }
        }

        // Step 4: Execute action
        let success = match action {
            action::PASS_THROUGH => return ProcessResult::PassThrough,
            action::ADD_LETTER => {
                self.add_letter(key);
                true
            }
            action::APPLY_TONE => self.try_apply_tone(key_type),
            action::DEFER_TONE => {
                self.pending = Some(PendingTransform { kind: pending::BREVE, position: 0 });
                true
            }
            action::APPLY_MODIFIER => self.try_apply_modifier(key_type),
            action::DEFER_MODIFIER => {
                self.pending = Some(PendingTransform { kind: pending::BREVE, position: 0 });
                true
            }
            action::COMPLETE_WORD => {
                self.reset();
                return ProcessResult::WordComplete;
            }
            _ => false,
        };

        // Step 5: State transition
        if action == action::ADD_LETTER && success {
            self.state = transition_add_letter(self.state, key_type);
        } else {
            let action_result = (action << 1) | (success as u8);
            self.state = I4_TRANSITION[self.state as usize][action_result as usize];
        }

        ProcessResult::Processed
    }

    fn try_apply_modifier(&mut self, key_type: u8) -> bool {
        let mod_idx = key_type - key_type::MOD_CIRCUM_A;
        let vowel_idx = self.get_target_vowel_idx();

        if I5_MODIFIER_COMPAT[mod_idx as usize][vowel_idx as usize] == 1 {
            // Check if open syllable for breve deferral
            if mod_idx == 1 && !self.has_final() {
                self.pending = Some(PendingTransform { kind: pending::BREVE, position: self.vowel_pos() });
                return true;
            }
            self.apply_modifier(mod_idx);
            self.last_transform = Some(transform::MOD_BREVE + mod_idx);
            self.reverted = false;
            true
        } else {
            false
        }
    }
}
```

---

## Memory Summary

**UPDATED** after Review fixes (2025-12-24):

| Matrix | Size | Bytes | Notes |
|--------|------|-------|-------|
| I1_ACTION | 6×38 | 228 | Unchanged |
| I2_DEFERRED | **4×38** | **152** | +CAPITALIZE |
| I3_REVERT | **14×38** | **532** | +W_AS_VOWEL, W_SKIP, SHORT_STROKE |
| I4_TRANSITION | 6×16 | 96 | Unchanged |
| I5_MODIFIER_COMPAT | 4×12 | 48 | Unchanged |
| I6_TONE_COMPAT | 6×2 | 12 | Unchanged |
| IS_VOWEL | 26 | 26 | Unchanged |
| IS_FINAL_CONSONANT | 26 | 26 | Unchanged |
| IS_STOP_FINAL | 26 | 26 | Unchanged |
| **Total** | | **1,146 bytes** | +152 bytes from original |

Changes from original (994 bytes):
- I2: 114 → 152 (+38 bytes) - Added PENDING_CAPITALIZE
- I3: 418 → 532 (+114 bytes) - Added 3 transform types

---

## Test Cases for Random Order Typing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_after_word() {
        // "toan" + 's' → "toán"
        let mut p = InputProcessor::new();
        p.process_key(T, TELEX); // t
        p.process_key(O, TELEX); // to
        p.process_key(A, TELEX); // toa
        p.process_key(N, TELEX); // toan
        p.process_key(S, TELEX); // toán
        assert_eq!(p.output(), "toán");
    }

    #[test]
    fn test_deferred_breve() {
        // "trawm" → "trắm"
        let mut p = InputProcessor::new();
        p.process_key(T, TELEX);
        p.process_key(R, TELEX);
        p.process_key(A, TELEX);
        p.process_key(W, TELEX); // Breve deferred
        assert!(p.pending.is_some());
        p.process_key(M, TELEX); // Final → apply breve
        assert!(p.pending.is_none());
        assert_eq!(p.output(), "trắm");
    }

    #[test]
    fn test_cancel_breve() {
        // "awwo" → "awo" (not Vietnamese)
        let mut p = InputProcessor::new();
        p.process_key(A, TELEX);
        p.process_key(W, TELEX); // Breve pending
        p.process_key(W, TELEX); // Revert → cancel
        assert!(p.pending.is_none());
        p.process_key(O, TELEX);
        assert_eq!(p.output(), "awo");
    }

    #[test]
    fn test_mark_revert_consonant() {
        // "tesst" → "test"
        let mut p = InputProcessor::new();
        p.process_key(T, TELEX);
        p.process_key(E, TELEX);
        p.process_key(S, TELEX); // Sắc applied
        p.process_key(S, TELEX); // Revert → pending pop
        p.process_key(T, TELEX); // Consonant → pop
        assert_eq!(p.output(), "test");
    }

    #[test]
    fn test_mark_revert_vowel() {
        // "issue" → "issue" (English)
        let mut p = InputProcessor::new();
        p.process_key(I, TELEX);
        p.process_key(S, TELEX); // Sắc applied
        p.process_key(S, TELEX); // Revert → pending pop
        p.process_key(U, TELEX); // Vowel → don't pop
        p.process_key(E, TELEX);
        assert_eq!(p.output(), "issue");
    }

    #[test]
    fn test_oscillation_prevention() {
        // "ddd" → "dd" (not ddd oscillating)
        let mut p = InputProcessor::new();
        p.process_key(D, TELEX); // d
        p.process_key(D, TELEX); // đ
        p.process_key(D, TELEX); // dd (reverted)
        p.process_key(D, TELEX); // ddd (locked, just adds d)
        assert_eq!(p.output(), "ddd");
    }
}
```
