# Phase 1: Validation Foundation (Comprehensive Matrix-Based)

**Branch:** `feature/engine-v2-phase1`
**Risk Level:** LOW
**Key Change:** Replace ALL case-by-case logic with **Comprehensive Matrix System**

**Research Docs:**
- `research/comprehensive-matrix-system.md` - Full architecture
- `research/matrix-data-vietnamese.md` - 43 patterns + implementation
- `research/matrix-data-english.md` - English phonotactics

---

## Design Philosophy

```
┌─────────────────────────────────────────────────────────────────┐
│                    MATRIX-FIRST ARCHITECTURE                    │
├─────────────────────────────────────────────────────────────────┤
│  Input → Parse → Matrix Lookup → Result                        │
│                                                                 │
│  NO: if letter == 'a' && position == 0 { ... }                 │
│  YES: M_POSITION[letter_idx][pos_idx] → valid/invalid          │
│                                                                 │
│  NO: match tone { Sac => {...}, Huyen => {...} }               │
│  YES: M_TONE_PLACEMENT[pattern_idx][context] → position        │
│                                                                 │
│  EVERY validation, placement, and transformation decision      │
│  = matrix lookup. ZERO case-by-case logic.                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Objectives

1. **Implement Vietnamese PhonMatrix system** (12 matrices)
2. **Implement English PhonMatrix system** (8 matrices)
3. Add Rule 7 (Tone-Stop Final) via M6 matrix
4. Add Tone/Modifier placement via M7/M8 matrices
5. Integrate bidirectional validation in auto-restore
6. Define Foreign state transitions explicitly

---

## Task 1.0: PhonMatrix Constraint System (NEW)

### Background

Current `validation.rs` uses case-by-case rule checking:
- 6 rule functions
- Linear scans of arrays (VALID_INITIALS, SPELLING_RULES, etc.)
- Hard to audit against Vietnamese phonology docs

**New Approach:** Matrix-Based Constraint Solver
- 5 compatibility matrices (O(1) lookup each)
- Declarative rule encoding
- Rule 7 naturally integrated as M4

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                Vietnamese PhonMatrix System (12 Matrices)        │
├─────────────────────────────────────────────────────────────────┤
│ VALIDATION MATRICES:                                             │
│   M1: INITIAL_VALID     │  29×1   │ Valid initial consonants    │
│   M2: INITIAL_VOWEL     │  29×12  │ Initial+Vowel spelling      │
│   M3: VOWEL_PAIR        │  12×12  │ Diphthong validity          │
│   M4: VOWEL_TRIPLE      │   8×12  │ Triphthong extensions       │
│   M5: VOWEL_FINAL       │  12×9   │ Vowel+Final compatibility   │
│   M6: TONE_FINAL        │   6×4   │ Tone+Stop Final (Rule 7)    │
├─────────────────────────────────────────────────────────────────┤
│ PLACEMENT MATRICES (43 patterns):                                │
│   M7: TONE_PLACEMENT    │  43×4   │ Which vowel gets tone       │
│   M8: MODIFIER_PLACEMENT│  43×2   │ Which vowel(s) get modifier │
├─────────────────────────────────────────────────────────────────┤
│ POSITION MATRICES:                                               │
│   M10: POSITION_START   │  26×1   │ What can start a word       │
│   M11: POSITION_END     │  26×1   │ What can end a word         │
│   M12: FINAL_DIGRAPH    │  26×26  │ Valid final consonant pairs │
├─────────────────────────────────────────────────────────────────┤
│ TRANSFORMATION MATRICES:                                         │
│   T1: TONE_TRANSFORM    │  12×6   │ Apply tone to vowel→Unicode │
│   T2: MODIFIER_TRANSFORM│  12×4   │ Apply modifier→Unicode      │
├─────────────────────────────────────────────────────────────────┤
│           Total: ~1.2KB static data, ALL O(1) lookups           │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                 English PhonMatrix System (8 Matrices)           │
├─────────────────────────────────────────────────────────────────┤
│   E1: ONSET_SINGLE      │  26×1   │ Single letter onsets        │
│   E2: ONSET_CC          │  26×26  │ Valid onset clusters        │
│   E3: ONSET_CCC         │  sparse │ Triple onsets (s-clusters)  │
│   E4: CODA_SINGLE       │  26×1   │ Single letter codas         │
│   E5: CODA_CC           │  26×26  │ Valid coda clusters         │
│   E6: CODA_CCC          │  sparse │ Triple codas                │
│   E7: IMPOSSIBLE_BIGRAM │  26×26  │ Never-occurring pairs       │
│   E8: VOWEL_DIGRAPH     │   5×5   │ Valid vowel pairs           │
├─────────────────────────────────────────────────────────────────┤
│           Total: ~2.1KB static data, ALL O(1) lookups           │
└─────────────────────────────────────────────────────────────────┘
```

### Implementation

**File:** `core/src/engine/validation/phonotactics.rs` (NEW)

```rust
//! Matrix-Based Vietnamese Phonotactic Constraints
//!
//! 5 compatibility matrices providing O(1) validation for all rules.
//! See: docs/vietnamese-language-system.md Section 7.6

use crate::data::keys;

// =============================================================================
// ENUMS - Type-safe indices for matrix lookup
// =============================================================================

/// Initial consonant indices (28 initials including None)
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Initial {
    None = 0,
    B, C, CH, D, G, GH, GI, H, K, KH, KR,
    L, M, N, NG, NGH, NH, P, PH, Q, QU, R, S, T, TH, TR, V, X
}

/// Vowel indices (12 core vowels)
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vowel {
    A = 0, A_BREVE, A_CIRC, E, E_CIRC, I, O, O_CIRC, O_HORN, U, U_HORN, Y
}

/// Final consonant indices (9 finals including None)
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FinalC {
    None = 0,
    C, CH, M, N, NG, NH, P, T
}

/// Tone indices (6 tones)
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tone {
    None = 0,  // ngang (level)
    Sac,       // sắc (acute)
    Huyen,     // huyền (grave)
    Hoi,       // hỏi (hook)
    Nga,       // ngã (tilde)
    Nang,      // nặng (dot)
}

// =============================================================================
// M1: INITIAL-VOWEL MATRIX (Spelling Rules)
// =============================================================================

/// M1: Initial → Vowel compatibility (28×12 = 336 cells)
/// Encodes c/k/q, g/gh, ng/ngh spelling rules
/// true = allowed, false = spelling violation
const M1_INIT_VOWEL: [[bool; 12]; 28] = {
    const T: bool = true;
    const F: bool = false;
    //     A    Ă    Â    E    Ê    I    O    Ô    Ơ    U    Ư    Y
    [
    /* None */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* B    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* C    */ [T, T, T, F, F, F, T, T, T, T, T, F], // C not before E,I,Y
    /* CH   */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* D    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* G    */ [T, T, T, F, F, T, T, T, T, T, T, T], // G not before E,Ê
    /* GH   */ [F, F, F, T, T, T, F, F, F, F, F, T], // GH only before E,Ê,I,Y
    /* GI   */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* H    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* K    */ [F, F, F, T, T, T, F, F, F, F, F, T], // K only before E,Ê,I,Y
    /* KH   */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* KR   */ [T, T, T, T, T, T, T, T, T, T, T, T], // ethnic minority
    /* L    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* M    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* N    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* NG   */ [T, T, T, F, F, F, T, T, T, T, T, F], // NG not before E,I,Y
    /* NGH  */ [F, F, F, T, T, T, F, F, F, F, F, T], // NGH only before E,Ê,I,Y
    /* NH   */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* P    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* PH   */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* Q    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* QU   */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* R    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* S    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* T    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* TH   */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* TR   */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* V    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    /* X    */ [T, T, T, T, T, T, T, T, T, T, T, T],
    ]
};

/// O(1) spelling check
#[inline]
pub fn check_initial_vowel(initial: Initial, vowel: Vowel) -> bool {
    M1_INIT_VOWEL[initial as usize][vowel as usize]
}

// =============================================================================
// M2: VOWEL-VOWEL MATRIX (Diphthongs)
// =============================================================================

/// M2: V1 → V2 compatibility (12×12 = 144 cells)
/// 0 = invalid, 1 = valid, 2 = valid but needs modifier check
const M2_VOWEL_PAIR: [[u8; 12]; 12] = {
    //     A  Ă  Â  E  Ê  I  O  Ô  Ơ  U  Ư  Y
    [
    /* A   */ [0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1], // ai,ao,au,ay
    /* Ă   */ [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // ă can't lead
    /* Â   */ [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1], // âu,ây
    /* E   */ [0, 0, 0, 0, 0, 0, 1, 0, 0, 2, 0, 0], // eo, êu(needs ê)
    /* Ê   */ [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0], // êu
    /* I   */ [1, 0, 0, 0, 2, 0, 0, 0, 0, 1, 0, 0], // ia, iê, iu
    /* O   */ [1, 2, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0], // oa, oă, oe, oi
    /* Ô   */ [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0], // ôi
    /* Ơ   */ [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0], // ơi
    /* U   */ [1, 0, 2, 0, 2, 1, 2, 0, 0, 0, 0, 1], // ua,uâ,uê,ui,uô,uy
    /* Ư   */ [1, 0, 0, 0, 0, 1, 0, 0, 2, 1, 0, 0], // ưa,ưi,ươ,ưu
    /* Y   */ [0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0], // yê
    ]
};

/// O(1) diphthong check
/// Returns: (is_valid, needs_modifier_check)
#[inline]
pub fn check_vowel_pair(v1: Vowel, v2: Vowel) -> (bool, bool) {
    let val = M2_VOWEL_PAIR[v1 as usize][v2 as usize];
    (val > 0, val == 2)
}

/// Valid triphthongs (13 patterns) - use array for O(1) with perfect hash
const VALID_TRIPHTHONGS: &[[Vowel; 3]] = &[
    [Vowel::I, Vowel::E_CIRC, Vowel::U],     // iêu
    [Vowel::Y, Vowel::E_CIRC, Vowel::U],     // yêu
    [Vowel::O, Vowel::A, Vowel::I],          // oai
    [Vowel::O, Vowel::A, Vowel::Y],          // oay
    [Vowel::O, Vowel::E, Vowel::O],          // oeo
    [Vowel::U, Vowel::A_CIRC, Vowel::Y],     // uây
    [Vowel::U, Vowel::O_CIRC, Vowel::I],     // uôi
    [Vowel::U, Vowel::Y, Vowel::A],          // uya
    [Vowel::U_HORN, Vowel::O_HORN, Vowel::I],// ươi
    [Vowel::U_HORN, Vowel::O_HORN, Vowel::U],// ươu
    [Vowel::U, Vowel::Y, Vowel::E_CIRC],     // uyê
    [Vowel::U, Vowel::Y, Vowel::U],          // uyu
    [Vowel::U, Vowel::E_CIRC, Vowel::U],     // uêu
];

/// Check if triphthong is valid
pub fn check_triphthong(v1: Vowel, v2: Vowel, v3: Vowel) -> bool {
    VALID_TRIPHTHONGS.iter().any(|t| t[0] == v1 && t[1] == v2 && t[2] == v3)
}

// =============================================================================
// M3: VOWEL-FINAL MATRIX
// =============================================================================

/// M3: Vowel → Final compatibility (12×9 = 108 cells)
/// - CH only after: a, ê, i (ach, êch, ich)
/// - NH only after: a, ê, i, y
/// - NG not after: e, ê
const M3_VOWEL_FINAL: [[bool; 9]; 12] = {
    const T: bool = true;
    const F: bool = false;
    //     None C   CH  M   N   NG  NH  P   T
    [
    /* A   */ [T, T, T, T, T, T, T, T, T],
    /* Ă   */ [T, T, F, T, T, T, F, T, T], // ăch,ănh invalid
    /* Â   */ [T, T, F, T, T, T, F, T, T], // âch,ânh invalid
    /* E   */ [T, T, F, T, T, F, F, T, T], // ech,eng,enh invalid
    /* Ê   */ [T, T, T, T, T, F, T, T, T], // êng invalid
    /* I   */ [T, T, T, T, T, F, T, T, T], // ing invalid
    /* O   */ [T, T, F, T, T, T, F, T, T], // och,onh invalid
    /* Ô   */ [T, T, F, T, T, T, F, T, T], // ôch,ônh invalid
    /* Ơ   */ [T, T, F, T, T, F, F, T, T], // ơch,ơng,ơnh invalid
    /* U   */ [T, T, F, T, T, T, F, T, T], // uch,unh invalid
    /* Ư   */ [T, T, F, T, T, F, F, T, T], // ưch,ưng,ưnh invalid
    /* Y   */ [T, T, F, T, T, F, T, T, T], // ych,yng invalid
    ]
};

/// O(1) vowel-final check
#[inline]
pub fn check_vowel_final(vowel: Vowel, final_c: FinalC) -> bool {
    M3_VOWEL_FINAL[vowel as usize][final_c as usize]
}

// =============================================================================
// M4: TONE-FINAL MATRIX (RULE 7!)
// =============================================================================

/// M4: Tone → Final compatibility (6×9 = 54 cells)
/// Stop finals (p, t, c, ch) ONLY allow: none, sắc, nặng
/// This is RULE 7 - the missing phonotactic constraint!
const M4_TONE_FINAL: [[bool; 9]; 6] = {
    const T: bool = true;
    const F: bool = false;
    //       None C   CH  M   N   NG  NH  P   T
    [
    /* None  */ [T, T, T, T, T, T, T, T, T],
    /* Sắc   */ [T, T, T, T, T, T, T, T, T],
    /* Huyền */ [T, F, F, T, T, T, T, F, F], // p,t,c,ch INVALID
    /* Hỏi   */ [T, F, F, T, T, T, T, F, F], // p,t,c,ch INVALID
    /* Ngã   */ [T, F, F, T, T, T, T, F, F], // p,t,c,ch INVALID
    /* Nặng  */ [T, T, T, T, T, T, T, T, T],
    ]
};

/// O(1) tone-final check (Rule 7)
#[inline]
pub fn check_tone_final(tone: Tone, final_c: FinalC) -> bool {
    M4_TONE_FINAL[tone as usize][final_c as usize]
}

// =============================================================================
// UNIFIED CONSTRAINT SOLVER
// =============================================================================

/// Constraint violation types
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintViolation {
    Valid,
    NoVowel,
    InvalidInitial,
    SpellingViolation { initial: Initial, vowel: Vowel },
    InvalidVowelPattern,
    InvalidFinal,
    VowelFinalMismatch { vowel: Vowel, final_c: FinalC },
    ToneFinalMismatch { tone: Tone, final_c: FinalC },  // Rule 7
    MissingModifier,
}

impl ConstraintViolation {
    pub fn is_valid(&self) -> bool {
        matches!(self, ConstraintViolation::Valid)
    }
}

/// Parsed syllable for validation
pub struct ParsedSyllable {
    pub initial: Initial,
    pub glide: Option<Vowel>,
    pub vowels: Vec<Vowel>,
    pub final_c: FinalC,
    pub tone: Tone,
}

/// Main validation - checks all matrices in order
pub fn validate_syllable(syllable: &ParsedSyllable) -> ConstraintViolation {
    // Phase 1: Must have vowel
    if syllable.vowels.is_empty() {
        return ConstraintViolation::NoVowel;
    }

    // Phase 2: M1 - Initial + Vowel spelling check
    if syllable.initial != Initial::None {
        let first_vowel = syllable.glide.unwrap_or(syllable.vowels[0]);
        if !check_initial_vowel(syllable.initial, first_vowel) {
            return ConstraintViolation::SpellingViolation {
                initial: syllable.initial,
                vowel: first_vowel,
            };
        }
    }

    // Phase 3: M2 - Vowel pattern validation
    match syllable.vowels.len() {
        1 => {}, // Single vowel always valid
        2 => {
            let (valid, _needs_check) = check_vowel_pair(
                syllable.vowels[0],
                syllable.vowels[1]
            );
            if !valid {
                return ConstraintViolation::InvalidVowelPattern;
            }
            // TODO: M5 modifier check when needs_check is true
        }
        3 => {
            if !check_triphthong(
                syllable.vowels[0],
                syllable.vowels[1],
                syllable.vowels[2]
            ) {
                return ConstraintViolation::InvalidVowelPattern;
            }
        }
        _ => return ConstraintViolation::InvalidVowelPattern,
    }

    // Phase 4: M3 - Vowel + Final compatibility
    if syllable.final_c != FinalC::None {
        // Use main vowel (last in diphthong/triphthong for tone placement rules)
        let main_vowel = syllable.vowels.last().copied().unwrap_or(Vowel::A);
        if !check_vowel_final(main_vowel, syllable.final_c) {
            return ConstraintViolation::VowelFinalMismatch {
                vowel: main_vowel,
                final_c: syllable.final_c,
            };
        }
    }

    // Phase 5: M4 - Tone + Final compatibility (RULE 7!)
    if syllable.final_c != FinalC::None && syllable.tone != Tone::None {
        if !check_tone_final(syllable.tone, syllable.final_c) {
            return ConstraintViolation::ToneFinalMismatch {
                tone: syllable.tone,
                final_c: syllable.final_c,
            };
        }
    }

    ConstraintViolation::Valid
}

// =============================================================================
// CONVERSION HELPERS
// =============================================================================

impl Initial {
    /// Convert from key sequence to Initial enum
    pub fn from_keys(keys: &[u16]) -> Option<Self> {
        match keys {
            [] => Some(Initial::None),
            [k] => match *k {
                keys::B => Some(Initial::B),
                keys::C => Some(Initial::C),
                keys::D => Some(Initial::D),
                keys::G => Some(Initial::G),
                keys::H => Some(Initial::H),
                keys::K => Some(Initial::K),
                keys::L => Some(Initial::L),
                keys::M => Some(Initial::M),
                keys::N => Some(Initial::N),
                keys::P => Some(Initial::P),
                keys::Q => Some(Initial::Q),
                keys::R => Some(Initial::R),
                keys::S => Some(Initial::S),
                keys::T => Some(Initial::T),
                keys::V => Some(Initial::V),
                keys::X => Some(Initial::X),
                _ => None,
            },
            [keys::C, keys::H] => Some(Initial::CH),
            [keys::G, keys::H] => Some(Initial::GH),
            [keys::G, keys::I] => Some(Initial::GI),
            [keys::K, keys::H] => Some(Initial::KH),
            [keys::K, keys::R] => Some(Initial::KR),
            [keys::N, keys::G] => Some(Initial::NG),
            [keys::N, keys::H] => Some(Initial::NH),
            [keys::P, keys::H] => Some(Initial::PH),
            [keys::Q, keys::U] => Some(Initial::QU),
            [keys::T, keys::H] => Some(Initial::TH),
            [keys::T, keys::R] => Some(Initial::TR),
            [keys::N, keys::G, keys::H] => Some(Initial::NGH),
            _ => None,
        }
    }
}

impl FinalC {
    /// Convert from key sequence to FinalC enum
    pub fn from_keys(keys: &[u16]) -> Option<Self> {
        match keys {
            [] => Some(FinalC::None),
            [k] => match *k {
                keys::C => Some(FinalC::C),
                keys::M => Some(FinalC::M),
                keys::N => Some(FinalC::N),
                keys::P => Some(FinalC::P),
                keys::T => Some(FinalC::T),
                _ => None,
            },
            [keys::C, keys::H] => Some(FinalC::CH),
            [keys::N, keys::G] => Some(FinalC::NG),
            [keys::N, keys::H] => Some(FinalC::NH),
            _ => None,
        }
    }
}
```

### Tests

**File:** `core/src/engine/validation/phonotactics_test.rs`

```rust
use super::phonotactics::*;

#[test]
fn test_m1_spelling_c_before_e_invalid() {
    assert!(!check_initial_vowel(Initial::C, Vowel::E));
    assert!(!check_initial_vowel(Initial::C, Vowel::I));
    assert!(!check_initial_vowel(Initial::C, Vowel::Y));
}

#[test]
fn test_m1_spelling_k_only_before_eiy() {
    assert!(check_initial_vowel(Initial::K, Vowel::E));
    assert!(check_initial_vowel(Initial::K, Vowel::I));
    assert!(!check_initial_vowel(Initial::K, Vowel::A));
    assert!(!check_initial_vowel(Initial::K, Vowel::O));
}

#[test]
fn test_m1_spelling_ng_ngh() {
    assert!(!check_initial_vowel(Initial::NG, Vowel::E));
    assert!(!check_initial_vowel(Initial::NG, Vowel::I));
    assert!(check_initial_vowel(Initial::NGH, Vowel::E));
    assert!(check_initial_vowel(Initial::NGH, Vowel::I));
}

#[test]
fn test_m2_valid_diphthongs() {
    assert!(check_vowel_pair(Vowel::A, Vowel::I).0);  // ai
    assert!(check_vowel_pair(Vowel::A, Vowel::O).0);  // ao
    assert!(check_vowel_pair(Vowel::E_CIRC, Vowel::U).0);  // êu
    assert!(check_vowel_pair(Vowel::O, Vowel::I).0);  // oi
}

#[test]
fn test_m2_invalid_diphthongs() {
    assert!(!check_vowel_pair(Vowel::O, Vowel::U).0);  // ou - English only
    assert!(!check_vowel_pair(Vowel::Y, Vowel::O).0);  // yo - English only
    assert!(!check_vowel_pair(Vowel::A, Vowel::E).0);  // ae - not Vietnamese
}

#[test]
fn test_m3_ch_only_after_a_e_i() {
    assert!(check_vowel_final(Vowel::A, FinalC::CH));     // ach OK
    assert!(check_vowel_final(Vowel::E_CIRC, FinalC::CH)); // êch OK
    assert!(check_vowel_final(Vowel::I, FinalC::CH));     // ich OK
    assert!(!check_vowel_final(Vowel::O, FinalC::CH));    // och INVALID
    assert!(!check_vowel_final(Vowel::U, FinalC::CH));    // uch INVALID
}

#[test]
fn test_m4_rule7_stop_finals() {
    // Valid: sắc, nặng with stop finals
    assert!(check_tone_final(Tone::Sac, FinalC::P));   // tấp OK
    assert!(check_tone_final(Tone::Nang, FinalC::P));  // tập OK
    assert!(check_tone_final(Tone::Sac, FinalC::T));   // mất OK
    assert!(check_tone_final(Tone::Nang, FinalC::C));  // mặc OK

    // Invalid: huyền, hỏi, ngã with stop finals
    assert!(!check_tone_final(Tone::Huyen, FinalC::P)); // tàp INVALID
    assert!(!check_tone_final(Tone::Hoi, FinalC::P));   // tảp INVALID
    assert!(!check_tone_final(Tone::Nga, FinalC::P));   // tãp INVALID
    assert!(!check_tone_final(Tone::Huyen, FinalC::CH)); // tàch INVALID
}

#[test]
fn test_m4_non_stop_finals_all_tones() {
    // Non-stop finals allow all tones
    assert!(check_tone_final(Tone::Huyen, FinalC::M));  // tàm OK
    assert!(check_tone_final(Tone::Hoi, FinalC::N));    // tản OK
    assert!(check_tone_final(Tone::Huyen, FinalC::NG)); // tàng OK
    assert!(check_tone_final(Tone::Nga, FinalC::NH));   // tãnh OK
}

#[test]
fn test_validate_syllable_valid() {
    let syllable = ParsedSyllable {
        initial: Initial::T,
        glide: None,
        vowels: vec![Vowel::A_CIRC],
        final_c: FinalC::P,
        tone: Tone::Sac,
    };
    assert!(validate_syllable(&syllable).is_valid()); // tấp
}

#[test]
fn test_validate_syllable_rule7_violation() {
    let syllable = ParsedSyllable {
        initial: Initial::T,
        glide: None,
        vowels: vec![Vowel::A],
        final_c: FinalC::P,
        tone: Tone::Huyen,
    };
    assert_eq!(
        validate_syllable(&syllable),
        ConstraintViolation::ToneFinalMismatch {
            tone: Tone::Huyen,
            final_c: FinalC::P
        }
    ); // tàp - Rule 7 violation
}
```

### Acceptance Criteria

- [ ] `phonotactics.rs` created with 4 matrices
- [ ] All enum types defined with conversion helpers
- [ ] `validate_syllable()` checks all matrices
- [ ] 20+ unit tests for matrix lookups
- [ ] Rule 7 naturally integrated via M4
- [ ] O(1) performance verified

---

## Task 1.1: Integrate PhonMatrix with Existing Validation

### Background

Replace current case-by-case rule functions with matrix-based solver while maintaining backward compatibility.

### Implementation

**File:** `core/src/engine/validation.rs` (update)

```rust
use super::phonotactics::{
    self, Initial, Vowel, FinalC, Tone as PhonTone,
    ConstraintViolation, ParsedSyllable, validate_syllable
};

/// Convert parsed syllable to PhonMatrix format and validate
fn validate_with_matrices(snap: &BufferSnapshot, syllable: &Syllable) -> ValidationResult {
    // Convert to PhonMatrix types
    let initial_keys: Vec<u16> = syllable.initial.iter().map(|&i| snap.keys[i]).collect();
    let final_keys: Vec<u16> = syllable.final_c.iter().map(|&i| snap.keys[i]).collect();

    let Some(initial) = Initial::from_keys(&initial_keys) else {
        return ValidationResult::InvalidInitial;
    };

    let Some(final_c) = FinalC::from_keys(&final_keys) else {
        return ValidationResult::InvalidFinal;
    };

    // Convert vowels with modifier info
    let vowels: Vec<Vowel> = syllable.vowel.iter().map(|&i| {
        key_and_tone_to_vowel(snap.keys[i], snap.tones[i])
    }).collect();

    // Extract tone from any vowel
    let tone = extract_phon_tone(&snap.tones);

    let parsed = ParsedSyllable {
        initial,
        glide: syllable.glide.map(|i| key_and_tone_to_vowel(snap.keys[i], snap.tones[i])),
        vowels,
        final_c,
        tone,
    };

    // Use matrix-based validation
    match validate_syllable(&parsed) {
        ConstraintViolation::Valid => ValidationResult::Valid,
        ConstraintViolation::NoVowel => ValidationResult::NoVowel,
        ConstraintViolation::InvalidInitial => ValidationResult::InvalidInitial,
        ConstraintViolation::SpellingViolation { .. } => ValidationResult::InvalidSpelling,
        ConstraintViolation::InvalidVowelPattern => ValidationResult::InvalidVowelPattern,
        ConstraintViolation::InvalidFinal => ValidationResult::InvalidFinal,
        ConstraintViolation::VowelFinalMismatch { .. } => ValidationResult::InvalidVowelPattern,
        ConstraintViolation::ToneFinalMismatch { .. } => ValidationResult::InvalidVowelPattern, // Rule 7
        ConstraintViolation::MissingModifier => ValidationResult::InvalidVowelPattern,
    }
}

/// Updated validate() using matrices
pub fn validate(snap: &BufferSnapshot) -> ValidationResult {
    if snap.keys.is_empty() {
        return ValidationResult::NoVowel;
    }

    let syllable = parse(&snap.keys);

    // Use matrix-based validation
    validate_with_matrices(snap, &syllable)
}
```

### Acceptance Criteria

- [ ] Existing `validate()` calls PhonMatrix solver
- [ ] All 561 existing tests pass
- [ ] Rule 7 violations now detected
- [ ] Performance equal or better

---

## Task 1.2: English Matrix-Based Validation Module (UPDATED)

### Background

Replace case-by-case English detection with **matrix-based phonotactic constraints**.

**Research Sources:**
- [Wikipedia: Phonotactics](https://en.wikipedia.org/wiki/Phonotactics)
- [Wikipedia: Consonant Cluster](https://en.wikipedia.org/wiki/Consonant_cluster)
- [Essentials of Linguistics: Syllable Structure](https://ecampusontario.pressbooks.pub/essentialsoflinguistics/chapter/3-4-syllable-structure/)

### English Phonotactic Rules

```
English syllable: (C)(C)(C)V(C)(C)(C)(C)
Maximum: CCCVCCCC (e.g., "strengths" /strɛŋkθs/)

Key constraints:
1. Onset: Max 3 consonants, strict rules
2. Coda: Max 4 consonants, voicing agreement
3. Bigrams: ~60 letter pairs NEVER occur in English
4. Triple letters: NEVER valid (aaa, bbb, etc.)
```

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│              English PhonMatrix Constraint System                │
├─────────────────────────────────────────────────────────────────┤
│   M1: ONSET_CC      │  26×26  │ Valid C+C onset clusters        │
│   M2: CODA_CC       │  26×26  │ Valid C+C coda clusters         │
│   M3: VOWEL_DIGRAPH │   5×5   │ Valid vowel pairs               │
│   M4: IMPOSSIBLE    │  26×26  │ Never-occurring bigrams         │
├─────────────────────────────────────────────────────────────────┤
│   validate_english() → O(n) where n = word length               │
└─────────────────────────────────────────────────────────────────┘
```

### Implementation

**File:** `core/src/engine/validation/english_phonotactics.rs` (NEW)

```rust
//! Matrix-Based English Phonotactic Constraints
//!
//! Provides O(1) validation for English word patterns.
//! See: plans/20251224-engine-v2-rewrite/research/english-matrix-constraint-system.md

// =============================================================================
// M4: IMPOSSIBLE BIGRAM MATRIX (26×26 = 676 cells)
// =============================================================================

/// Bigrams that NEVER appear in any English word
/// Generated from dictionary analysis
const IMPOSSIBLE_BIGRAM_MATRIX: [[bool; 26]; 26] = {
    // Row = first letter (a=0, z=25)
    // Col = second letter
    // true = this bigram NEVER occurs in English

    // Q row: Q only before U
    const Q_ROW: [bool; 26] = [
        true,true,true,true,true,true,true,true,true,true,true,true,true,
        true,true,true,true,true,true,true,false,true,true,true,true,true
    ]; // Only Q+U (index 20) is false

    // J row: J has many restrictions
    const J_ROW: [bool; 26] = [
        false,true,true,true,false,true,true,true,false,true,true,true,true,
        true,false,true,true,true,true,true,false,true,true,true,true,true
    ]; // ja,je,ji,jo,ju allowed

    // X row: X has many restrictions
    const X_ROW: [bool; 26] = [
        false,true,false,true,false,false,true,false,false,true,true,false,true,
        false,false,false,true,true,false,false,false,false,false,true,false,true
    ];

    // ... full 26×26 matrix
    // (abbreviated for readability, full matrix in implementation)
    [[false; 26]; 26] // Placeholder - actual data in implementation
};

/// O(1) impossible bigram check
#[inline]
pub fn is_impossible_bigram(c1: char, c2: char) -> bool {
    let i1 = (c1.to_ascii_lowercase() as usize).wrapping_sub('a' as usize);
    let i2 = (c2.to_ascii_lowercase() as usize).wrapping_sub('a' as usize);

    if i1 >= 26 || i2 >= 26 {
        return false; // Non-letter, don't reject
    }

    IMPOSSIBLE_BIGRAM_MATRIX[i1][i2]
}

// =============================================================================
// M1: ONSET CLUSTER MATRIX
// =============================================================================

/// Valid two-consonant onset clusters
/// Pattern: Stop/Fricative + Liquid/Glide
const VALID_CC_ONSETS: [[bool; 26]; 26] = {
    // Only L(11), R(17), W(22), Y(24) can be C2 (except after S)
    // B: bl, br, by
    // C: cl, cr
    // D: dr, dw
    // F: fl, fr
    // G: gl, gr
    // K: kl, kr, kw (qu)
    // P: pl, pr
    // S: sc, sk, sl, sm, sn, sp, st, sw (special: S can precede many)
    // T: tr, tw
    // W: wr

    // ... 26×26 matrix (abbreviated)
    [[false; 26]; 26]
};

/// Valid three-consonant onsets (s + voiceless stop + liquid/glide)
const VALID_CCC_ONSETS: &[[u8; 3]] = &[
    [b's', b'p', b'l'], // spl-
    [b's', b'p', b'r'], // spr-
    [b's', b't', b'r'], // str-
    [b's', b'c', b'r'], // scr-
    [b's', b'k', b'w'], // squ- (/skw/)
];

// =============================================================================
// M2: CODA CLUSTER MATRIX
// =============================================================================

/// Valid two-consonant coda clusters
/// Pattern: Liquid/Nasal + Stop/Fricative (with voicing agreement)
const VALID_CC_CODAS: [[bool; 26]; 26] = {
    // L + {b,d,f,k,m,n,p,t,v}: lb,ld,lf,lk,lm,ln,lp,lt,lv
    // R + {b,d,f,g,k,l,m,n,p,s,t,v}: rb,rd,rf,rg,rk,rl,rm,rn,rp,rs,rt,rv
    // M + {p}: mp (homorganic)
    // N + {d,t,k}: nd,nt,nk (homorganic)
    // Stop + Stop (same voicing): pt,kt,ks

    // ... 26×26 matrix
    [[false; 26]; 26]
};

// =============================================================================
// M3: VOWEL DIGRAPH MATRIX
// =============================================================================

/// Valid vowel digraph combinations (5×5 for a,e,i,o,u)
const VOWEL_DIGRAPH_MATRIX: [[bool; 5]; 5] = {
    //     a     e     i     o     u
    /* a */ [false, true,  true,  false, true ], // ae,ai,au
    /* e */ [true,  true,  true,  false, true ], // ea,ee,ei,eu
    /* i */ [false, true,  false, false, false], // ie
    /* o */ [true,  true,  true,  true,  true ], // oa,oe,oi,oo,ou
    /* u */ [false, true,  true,  false, false], // ue,ui
};

// =============================================================================
// VALIDATION RESULT
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum EnglishValidation {
    Valid,
    PossiblyValid,
    InvalidOnset(String),
    InvalidCoda(String),
    ImpossibleBigram(char, char),
    TripleLetter(char),
    NoVowel,
}

impl EnglishValidation {
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid | Self::PossiblyValid)
    }

    pub fn is_definitely_invalid(&self) -> bool {
        !self.is_valid()
    }
}

// =============================================================================
// MAIN VALIDATION FUNCTION
// =============================================================================

/// Matrix-based English word validation
pub fn validate_english(word: &str) -> EnglishValidation {
    let chars: Vec<char> = word.to_lowercase().chars().collect();

    if chars.is_empty() {
        return EnglishValidation::NoVowel;
    }

    // Phase 1: M4 - Check impossible bigrams O(n)
    for window in chars.windows(2) {
        if is_impossible_bigram(window[0], window[1]) {
            return EnglishValidation::ImpossibleBigram(window[0], window[1]);
        }
    }

    // Phase 2: Check triple letters (never valid) O(n)
    for window in chars.windows(3) {
        if window[0] == window[1] && window[1] == window[2] {
            return EnglishValidation::TripleLetter(window[0]);
        }
    }

    // Phase 3: M1 - Validate onset cluster
    if let Some(violation) = validate_onset(&chars) {
        return violation;
    }

    // Phase 4: M2 - Validate coda cluster
    if let Some(violation) = validate_coda(&chars) {
        return violation;
    }

    // Phase 5: Must have vowel
    if !chars.iter().any(|c| "aeiouy".contains(*c)) {
        return EnglishValidation::NoVowel;
    }

    // Phase 6: Morphology patterns (bonus validation)
    if has_english_morphology(word) {
        return EnglishValidation::Valid;
    }

    EnglishValidation::PossiblyValid
}

fn validate_onset(chars: &[char]) -> Option<EnglishValidation> {
    let first_vowel = chars.iter().position(|c| "aeiouy".contains(*c))?;

    if first_vowel > 3 {
        return Some(EnglishValidation::InvalidOnset(
            chars[..first_vowel].iter().collect()
        ));
    }

    if first_vowel == 2 {
        let c1 = chars[0];
        let c2 = chars[1];
        if !is_valid_cc_onset(c1, c2) {
            return Some(EnglishValidation::InvalidOnset(format!("{}{}", c1, c2)));
        }
    }

    if first_vowel == 3 {
        let cluster = [chars[0] as u8, chars[1] as u8, chars[2] as u8];
        if !VALID_CCC_ONSETS.contains(&cluster) {
            return Some(EnglishValidation::InvalidOnset(
                chars[..3].iter().collect()
            ));
        }
    }

    None
}

fn validate_coda(chars: &[char]) -> Option<EnglishValidation> {
    let last_vowel = chars.iter().rposition(|c| "aeiouy".contains(*c))?;

    if last_vowel == chars.len() - 1 {
        return None; // No coda
    }

    let coda_len = chars.len() - last_vowel - 1;
    if coda_len > 4 {
        return Some(EnglishValidation::InvalidCoda(
            chars[last_vowel + 1..].iter().collect()
        ));
    }

    // Check two-consonant coda
    if coda_len >= 2 {
        let c1 = chars[last_vowel + 1];
        let c2 = chars[last_vowel + 2];
        if !is_valid_cc_coda(c1, c2) {
            return Some(EnglishValidation::InvalidCoda(format!("{}{}", c1, c2)));
        }
    }

    None
}

#[inline]
fn is_valid_cc_onset(c1: char, c2: char) -> bool {
    let i1 = (c1 as usize).wrapping_sub('a' as usize);
    let i2 = (c2 as usize).wrapping_sub('a' as usize);
    if i1 >= 26 || i2 >= 26 { return false; }
    VALID_CC_ONSETS[i1][i2]
}

#[inline]
fn is_valid_cc_coda(c1: char, c2: char) -> bool {
    let i1 = (c1 as usize).wrapping_sub('a' as usize);
    let i2 = (c2 as usize).wrapping_sub('a' as usize);
    if i1 >= 26 || i2 >= 26 { return false; }
    VALID_CC_CODAS[i1][i2]
}

/// Morphological patterns (suffixes/prefixes)
fn has_english_morphology(word: &str) -> bool {
    const SUFFIXES: &[&str] = &[
        "tion", "sion", "ness", "ment", "able", "ible",
        "ful", "less", "ing", "ed", "ly", "er", "est",
    ];
    const PREFIXES: &[&str] = &[
        "un", "re", "pre", "dis", "mis", "over", "under",
    ];

    let word_lower = word.to_lowercase();

    for suffix in SUFFIXES {
        if word_lower.ends_with(suffix) && word.len() > suffix.len() + 2 {
            return true;
        }
    }

    for prefix in PREFIXES {
        if word_lower.starts_with(prefix) && word.len() > prefix.len() + 2 {
            return true;
        }
    }

    false
}
```

### Tests

**File:** `core/src/engine/validation/english_phonotactics_test.rs`

```rust
use super::english_phonotactics::*;

#[test]
fn test_impossible_bigrams() {
    // Q not followed by U
    assert!(is_impossible_bigram('q', 'a'));
    assert!(is_impossible_bigram('q', 'x'));
    assert!(!is_impossible_bigram('q', 'u')); // QU is valid

    // Common impossible pairs
    assert!(is_impossible_bigram('b', 'x'));
    assert!(is_impossible_bigram('v', 'x'));
    assert!(is_impossible_bigram('j', 'x'));
}

#[test]
fn test_triple_letters_invalid() {
    assert_eq!(
        validate_english("duongfffff"),
        EnglishValidation::TripleLetter('f')
    );
    assert_eq!(
        validate_english("textxxx"),
        EnglishValidation::TripleLetter('x')
    );
}

#[test]
fn test_valid_onsets() {
    assert!(validate_english("string").is_valid());  // str-
    assert!(validate_english("split").is_valid());   // spl-
    assert!(validate_english("blue").is_valid());    // bl-
    assert!(validate_english("train").is_valid());   // tr-
}

#[test]
fn test_invalid_onsets() {
    assert!(matches!(
        validate_english("tling"),
        EnglishValidation::InvalidOnset(_)
    )); // tl- not valid

    assert!(matches!(
        validate_english("dlam"),
        EnglishValidation::InvalidOnset(_)
    )); // dl- not valid
}

#[test]
fn test_valid_english_words() {
    assert!(validate_english("running").is_valid());   // -ing
    assert!(validate_english("beautiful").is_valid()); // -ful
    assert!(validate_english("string").is_valid());    // str-
    assert!(validate_english("school").is_valid());    // sch-
    assert!(validate_english("text").is_valid());
}

#[test]
fn test_invalid_patterns() {
    // duongfffff - triple f
    assert!(!validate_english("duongfffff").is_valid());

    // Words with impossible bigrams
    assert!(!validate_english("qxtest").is_valid());
}

#[test]
fn test_morphology_detection() {
    assert!(has_english_morphology("running"));    // -ing
    assert!(has_english_morphology("happiness"));  // -ness
    assert!(has_english_morphology("unhappy"));    // un-
    assert!(has_english_morphology("rewrite"));    // re-
    assert!(!has_english_morphology("cat"));       // No affix
}
```

### Acceptance Criteria

- [ ] `english_phonotactics.rs` created with 4 matrices
- [ ] M4: Impossible bigram matrix (26×26)
- [ ] M1: Valid onset matrix (26×26)
- [ ] M2: Valid coda matrix (26×26)
- [ ] M3: Vowel digraph matrix (5×5)
- [ ] `validate_english()` uses all matrices
- [ ] 20+ tests covering all constraint types
- [ ] O(n) validation where n = word length
- [ ] Replaces old case-by-case `is_possibly_english()`

---

## Task 1.3: Bidirectional Validation Integration

*(Unchanged from original plan - see original for full implementation)*

### Acceptance Criteria

- [ ] `try_auto_restore_on_space()` updated
- [ ] "đườngfffff" keeps as-is
- [ ] "running" restores correctly
- [ ] No regression

---

## Task 1.4: Foreign State Documentation

*(Unchanged from original plan)*

---

## Phase 1 Completion Checklist

- [ ] **Task 1.0:** PhonMatrix system with 4 matrices
- [ ] **Task 1.1:** Integration with existing validation
- [ ] **Task 1.2:** english.rs module
- [ ] **Task 1.3:** Bidirectional restore
- [ ] **Task 1.4:** Foreign state documented
- [ ] All 561+ tests passing
- [ ] Rule 7 (Tone-Stop Final) working
- [ ] O(1) performance verified

**Risk Level:** LOW (new module, additive changes)

---

## Performance Comparison

| Operation | Before (Case-by-Case) | After (Matrix) |
|-----------|----------------------|----------------|
| Spelling check | O(n) SPELLING_RULES scan | **O(1)** M1 lookup |
| Vowel pattern | O(n) 42-item scan | **O(1)** M2 lookup |
| Final check | O(n) array scan | **O(1)** M3 lookup |
| Tone-Final (new) | Not implemented | **O(1)** M4 lookup |
| **Total** | ~5 linear scans | **4 O(1) lookups** |

---

## Next Phase

After Phase 1 completion, proceed to **Phase 2: DualBuffer Integration**.

See `phase-02-dual-buffer.md` for details.
