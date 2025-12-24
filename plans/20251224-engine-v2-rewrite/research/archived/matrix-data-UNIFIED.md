# Unified Matrix System - Pure Dynamic Design

**Date**: 2025-12-24
**Status**: REDESIGN - Eliminates all case-by-case logic
**Goal**: 100% matrix lookup, zero if-else in hot path

---

## Problems with Previous Design

| Issue | Location | Type |
|-------|----------|------|
| `transition_add_letter()` | I4 | IF-ELSE logic |
| `is_tone_compat()` | I6 | IF-ELSE logic |
| `classify_key()` | Input | MATCH logic |
| Breve defer check | try_apply_modifier | IF-ELSE logic |
| IS_VOWEL + IS_FINAL + IS_STOP | 3 tables | Redundant |
| HAS_MODIFIER vs HAS_TONE | State model | Incorrect |
| MOD_BREVE = MOD_HORN in I3 | Revert | Duplicate |

---

## Part 1: Unified Letter Classification

**Replace 3 tables with 1 bitfield table:**

```rust
/// Letter classification - single lookup, all info
/// Bits: 0=vowel, 1=initial_consonant, 2=final_consonant, 3=stop_final
pub mod letter_class {
    pub const VOWEL: u8 = 0b0001;
    pub const INITIAL: u8 = 0b0010;
    pub const FINAL: u8 = 0b0100;
    pub const STOP: u8 = 0b1000;

    pub const VOWEL_ONLY: u8 = VOWEL;                    // a,e,i,o,u,y
    pub const CONSONANT: u8 = INITIAL;                   // b,d,g,h,l,r,v,x
    pub const FINAL_NASAL: u8 = INITIAL | FINAL;         // m,n
    pub const FINAL_STOP: u8 = INITIAL | FINAL | STOP;   // c,p,t
    pub const SPECIAL_G: u8 = INITIAL | FINAL;           // g (in ng)
}

pub static LETTER_CLASS: [u8; 26] = [
    0b0001, // a - vowel
    0b0010, // b - initial only
    0b1110, // c - initial + final + stop
    0b0010, // d - initial only
    0b0001, // e - vowel
    0b0000, // f - none (not Vietnamese)
    0b0110, // g - initial + final (ng)
    0b0010, // h - initial (or after c/n)
    0b0001, // i - vowel
    0b0000, // j - none (not Vietnamese)
    0b1110, // k - initial + final + stop (ethnic)
    0b0010, // l - initial only
    0b0110, // m - initial + final (nasal)
    0b0110, // n - initial + final (nasal)
    0b0001, // o - vowel
    0b1110, // p - initial + final + stop
    0b0010, // q - initial only (always qu)
    0b0010, // r - initial only
    0b0010, // s - initial only
    0b1110, // t - initial + final + stop
    0b0001, // u - vowel
    0b0010, // v - initial only
    0b0000, // w - none (modifier key)
    0b0010, // x - initial only
    0b0001, // y - vowel
    0b0000, // z - none (modifier key)
];

/// Single inline check
#[inline(always)]
pub fn is_vowel(c: u8) -> bool { LETTER_CLASS[c as usize] & letter_class::VOWEL != 0 }
#[inline(always)]
pub fn is_final(c: u8) -> bool { LETTER_CLASS[c as usize] & letter_class::FINAL != 0 }
#[inline(always)]
pub fn is_stop(c: u8) -> bool { LETTER_CLASS[c as usize] & letter_class::STOP != 0 }
```

**Memory: 26 bytes (was 78 bytes for 3 tables)**

---

## Part 2: Unified State Model

**Problem**: HAS_MODIFIER (3) and HAS_TONE (4) are mutually exclusive in old design.
**Reality**: A vowel can have BOTH modifier AND tone (e.g., "ấ").

**Solution**: Use single HAS_DIACRITIC state + flags

```rust
/// States - 5 total (reduced from 6)
pub mod state {
    pub const EMPTY: u8 = 0;
    pub const HAS_INITIAL: u8 = 1;
    pub const HAS_VOWEL: u8 = 2;
    pub const HAS_DIACRITIC: u8 = 3;  // has modifier OR mark OR both
    pub const HAS_FINAL: u8 = 4;
}

/// Diacritic flags tracked separately (not in state machine)
pub struct DiacriticFlags {
    has_modifier: bool,  // circumflex/horn/breve
    has_mark: bool,      // sắc/huyền/hỏi/ngã/nặng
}

// State machine only cares: "does buffer have any diacritic?"
// The specific type is tracked in flags, not state
```

**Why**:
- State machine simpler (5 states, not 6)
- No impossible state combinations
- Flags handle the "which diacritic" detail

---

## Part 3: Unified Action Dispatch (I1)

**Replace**: I1_ACTION + transition_add_letter() + IS_VOWEL checks

**With**: Single matrix that encodes EVERYTHING

```rust
/// Key categories (8 total)
pub mod key_cat {
    pub const VOWEL: u8 = 0;        // a,e,i,o,u,y
    pub const CONSONANT: u8 = 1;    // b,d,g,h,l,r,v,x
    pub const FINAL_NASAL: u8 = 2;  // m,n
    pub const FINAL_STOP: u8 = 3;   // c,p,t (k rare)
    pub const TONE: u8 = 4;         // s,f,r,x,j,z (Telex)
    pub const MODIFIER: u8 = 5;     // w,a,e,o (Telex for ^)
    pub const STROKE: u8 = 6;       // d (stroke key)
    pub const SPECIAL: u8 = 7;      // space, backspace, etc
}

/// Classify key to category - ONE lookup
pub static KEY_TO_CAT: [u8; 38] = [
    0,1,3,6,0,4,1,1,0,4,3,1,2,2,0,3,1,4,4,3,0,1,5,4,0,4, // a-z mapped
    4,4,4,4,4,4,  // tone keys 26-31
    5,5,5,5,      // modifier keys 32-35
    7,7,          // BS, space 36-37
];

/// Action + Next State combined (8 bits)
/// High 4 bits = action, Low 4 bits = next state
pub mod action_state {
    pub const PASS: u8 = 0x00;
    pub const ADD_INITIAL: u8 = 0x11;    // action=1, state=HAS_INITIAL
    pub const ADD_VOWEL: u8 = 0x12;      // action=1, state=HAS_VOWEL
    pub const ADD_FINAL: u8 = 0x14;      // action=1, state=HAS_FINAL
    pub const APPLY_TONE: u8 = 0x23;     // action=2, state=HAS_DIACRITIC
    pub const APPLY_MOD: u8 = 0x33;      // action=3, state=HAS_DIACRITIC
    pub const CHECK_REVERT: u8 = 0x40;   // action=4, state unchanged
    pub const COMPLETE: u8 = 0x50;       // action=5, state=EMPTY
    pub const DEFER_MOD: u8 = 0x62;      // action=6, state=HAS_VOWEL (pending)
    pub const APPLY_STROKE: u8 = 0x71;   // action=7, state unchanged
}

/// Unified dispatch: state × key_cat → action_state (8 bits)
/// Size: 5 states × 8 categories = 40 bytes
pub static DISPATCH: [[u8; 8]; 5] = [
    //              VOWEL  CONS  F_NAS F_STP TONE  MOD   STRK  SPEC
    /* EMPTY */    [0x12, 0x11, 0x11, 0x11, 0x00, 0x00, 0x11, 0x50],
    /* HAS_INIT */ [0x12, 0x11, 0x11, 0x11, 0x00, 0x00, 0x40, 0x50],
    /* HAS_VOW */  [0x12, 0x14, 0x14, 0x14, 0x23, 0x33, 0x40, 0x50],
    /* HAS_DIA */  [0x12, 0x14, 0x14, 0x14, 0x40, 0x40, 0x40, 0x50],
    /* HAS_FIN */  [0x12, 0x00, 0x00, 0x00, 0x40, 0x40, 0x00, 0x50],
];

/// Process key - PURE MATRIX, zero if-else
#[inline(always)]
pub fn dispatch(state: u8, key_idx: u8) -> (u8, u8) {
    let cat = KEY_TO_CAT[key_idx as usize];
    let action_state = DISPATCH[state as usize][cat as usize];
    let action = action_state >> 4;
    let new_state = action_state & 0x0F;
    (action, new_state)
}
```

**Eliminated**:
- `transition_add_letter()` function (was if-else)
- IS_VOWEL check in process loop
- IS_FINAL_CONSONANT check in process loop

**Memory**: 40 bytes (was 228 bytes for I1 + 26 for IS_VOWEL)

---

## Part 4: Unified Defer Resolution (I2)

**Problem**: I2 has 4 pending types × 38 keys = 152 bytes, but logic still needs special cases.

**Solution**: Encode defer behavior in same dispatch

```rust
/// Defer actions encoded in dispatch response
/// When action = DEFER_MOD, also set pending type
pub mod defer {
    pub const NONE: u8 = 0;
    pub const BREVE: u8 = 1;
    pub const U_HORN: u8 = 2;
    pub const MARK_POP: u8 = 3;
}

/// Pending resolution - simplified
/// Only need: pending_type × is_final_consonant → apply/cancel/keep
/// Size: 4 × 2 = 8 bytes
pub static DEFER_RESOLVE: [[u8; 2]; 4] = [
    //              not_final  is_final
    /* NONE */     [0,         0],       // no pending, ignore
    /* BREVE */    [0,         1],       // keep if not final, apply if final
    /* U_HORN */   [1,         1],       // apply on any following letter
    /* MARK_POP */ [0,         2],       // keep on vowel, pop on consonant
];
// 0=keep, 1=apply, 2=pop

/// Check: letter_class already tells us is_final
#[inline(always)]
pub fn resolve_defer(pending: u8, next_key: u8) -> u8 {
    let is_final = (LETTER_CLASS[next_key as usize] & letter_class::FINAL) != 0;
    DEFER_RESOLVE[pending as usize][is_final as usize]
}
```

**Memory**: 8 bytes (was 152 bytes)

---

## Part 5: Unified Revert Lookup (I3)

**Problem**:
- 14 transform types × 38 keys = 532 bytes
- MOD_BREVE and MOD_HORN rows are IDENTICAL (both use 'w')

**Solution**: Group by revert key, not transform type

```rust
/// Revert is simple: same key pressed twice → revert
/// Only need: which key triggers revert for which transform
/// Group transforms by their trigger key

pub mod transform {
    pub const NONE: u8 = 0;
    pub const STROKE: u8 = 1;         // d→đ, revert key: d
    pub const TONE_SAC: u8 = 2;       // revert key: s
    pub const TONE_HUYEN: u8 = 3;     // revert key: f
    pub const TONE_HOI: u8 = 4;       // revert key: r
    pub const TONE_NGA: u8 = 5;       // revert key: x
    pub const TONE_NANG: u8 = 6;      // revert key: j
    pub const MOD_CIRCUM_A: u8 = 7;   // revert key: a
    pub const MOD_CIRCUM_E: u8 = 8;   // revert key: e
    pub const MOD_CIRCUM_O: u8 = 9;   // revert key: o
    pub const MOD_HORN: u8 = 10;      // revert key: w (breve also!)
}

/// Transform → Revert key mapping (11 entries)
pub static REVERT_KEY: [u8; 11] = [
    0xFF, // NONE - no revert
    3,    // STROKE → d
    18,   // TONE_SAC → s
    5,    // TONE_HUYEN → f
    17,   // TONE_HOI → r
    23,   // TONE_NGA → x
    9,    // TONE_NANG → j
    0,    // MOD_CIRCUM_A → a
    4,    // MOD_CIRCUM_E → e
    14,   // MOD_CIRCUM_O → o
    22,   // MOD_HORN → w (covers both horn and breve!)
];

/// Check revert - O(1)
#[inline(always)]
pub fn should_revert(last_transform: u8, current_key: u8) -> bool {
    REVERT_KEY[last_transform as usize] == current_key
}
```

**Memory**: 11 bytes (was 532 bytes!)

---

## Part 6: Unified Tone/Modifier Compatibility

**Problem**:
- I5: 4×12 = 48 bytes for modifier compat
- I6: 6×2 = 12 bytes for tone compat (simplified)
- `is_tone_compat()` still uses if-else

**Solution**: Single validity matrix

```rust
/// Tone validity: only stop finals restrict tones
/// All patterns with non-stop finals → all 6 tones valid
/// Patterns with stop finals → only sắc(1), nặng(5) valid
pub static TONE_VALID_STOP: [bool; 6] = [
    false, // ngang
    true,  // sắc
    false, // huyền
    false, // hỏi
    false, // ngã
    true,  // nặng
];

/// Modifier validity: which vowels accept which modifiers
/// Encoded as bitmask per vowel
pub mod mod_mask {
    pub const CIRCUM: u8 = 0b001;  // â, ê, ô
    pub const BREVE: u8 = 0b010;   // ă
    pub const HORN: u8 = 0b100;    // ơ, ư
}

/// Vowel → valid modifiers (12 entries)
pub static VOWEL_MOD_MASK: [u8; 12] = [
    0b001, // a → circumflex only
    0b000, // ă → none (already has breve)
    0b000, // â → none (already has circumflex)
    0b001, // e → circumflex only
    0b000, // ê → none
    0b000, // i → none
    0b101, // o → circumflex OR horn
    0b000, // ô → none
    0b000, // ơ → none
    0b100, // u → horn only
    0b000, // ư → none
    0b000, // y → none
];

/// Check modifier validity - O(1)
#[inline(always)]
pub fn can_apply_mod(vowel_idx: u8, mod_type: u8) -> bool {
    let mod_bit = match mod_type {
        0 => mod_mask::CIRCUM,
        1 => mod_mask::BREVE,
        2 => mod_mask::HORN,
        _ => return false,
    };
    VOWEL_MOD_MASK[vowel_idx as usize] & mod_bit != 0
}
```

**Memory**: 6 + 12 = 18 bytes (was 60 bytes)

---

## Part 7: Complete Unified Processing

```rust
pub struct UnifiedProcessor {
    state: u8,              // 5 possible states
    pending: u8,            // 4 possible defer types
    last_transform: u8,     // 11 possible transforms
    reverted: bool,         // oscillation prevention
}

impl UnifiedProcessor {
    /// Main entry - PURE MATRIX LOOKUPS
    pub fn process(&mut self, key_idx: u8) -> ProcessResult {
        // Step 1: Check pending resolution (matrix)
        if self.pending != defer::NONE {
            let resolution = resolve_defer(self.pending, key_idx);
            match resolution {
                1 => self.apply_pending(),   // APPLY
                2 => self.pop_raw(),         // POP
                _ => {}                       // KEEP
            }
            if resolution != 0 {
                self.pending = defer::NONE;
            }
        }

        // Step 2: Dispatch action (matrix)
        let (action, new_state) = dispatch(self.state, key_idx);

        // Step 3: Check revert (matrix)
        if action >= 4 && action <= 7 {  // CHECK_REVERT range
            if should_revert(self.last_transform, key_idx) && !self.reverted {
                self.reverted = true;
                return self.do_revert();
            }
        }

        // Step 4: Execute action
        let result = self.execute(action, key_idx);

        // Step 5: Update state
        self.state = new_state;

        result
    }
}
```

---

## Memory Comparison

| Component | Old Design | New Design | Savings |
|-----------|------------|------------|---------|
| Letter tables | 78 bytes | 26 bytes | **67%** |
| I1 Dispatch | 228 bytes | 40 bytes | **82%** |
| I2 Defer | 152 bytes | 8 bytes | **95%** |
| I3 Revert | 532 bytes | 11 bytes | **98%** |
| I5+I6 Compat | 60 bytes | 18 bytes | **70%** |
| **Total** | **1,050 bytes** | **103 bytes** | **90%** |

---

## Eliminated Case-by-Case Logic

| Function | Old | New |
|----------|-----|-----|
| `transition_add_letter()` | if-else | DISPATCH matrix |
| `is_tone_compat()` | if-else | TONE_VALID_STOP array |
| `classify_key()` | match | KEY_TO_CAT array |
| Breve defer check | if-else | DEFER_RESOLVE matrix |
| IS_VOWEL check | separate call | Encoded in KEY_TO_CAT |

---

## Trade-offs

**Pros**:
- 90% memory reduction
- Zero if-else in hot path
- Single matrix lookup per step
- Cache-friendly sequential access

**Cons**:
- Less readable (packed data)
- Harder to debug individual rules
- KEY_TO_CAT must be input-method specific (Telex vs VNI)

---

## Input Method Handling

For Telex vs VNI, create separate KEY_TO_CAT tables:

```rust
pub static KEY_TO_CAT_TELEX: [u8; 38] = [...]; // s=tone, w=mod
pub static KEY_TO_CAT_VNI: [u8; 38] = [...];   // 1-9=tone/mod

/// Select at init, not per-key
pub fn init_processor(method: u8) -> &'static [u8; 38] {
    match method {
        0 => &KEY_TO_CAT_TELEX,
        1 => &KEY_TO_CAT_VNI,
        _ => &KEY_TO_CAT_TELEX,
    }
}
```

The `match` is at INIT time, not per-keystroke. Hot path is pure matrix.

---

## Remaining Questions

1. **Post-tone circumflex (xepse→xếp)**: How to model without case logic?
   - Solution: Extend DISPATCH with "has_mark" dimension? Or post-process?

2. **Delayed circumflex revert**: Same issue
   - Solution: Track in pending, resolve via matrix

3. **Auto-capitalize**: Where does it fit?
   - Solution: Separate flags, not in state machine

---

## Next Steps

1. Implement UNIFIED tables in Rust
2. Benchmark against old design
3. Test all 43 vowel patterns
4. Test random order typing scenarios
5. Profile cache behavior

---

**This design achieves**: True matrix-first architecture with zero case-by-case logic in hot path.
