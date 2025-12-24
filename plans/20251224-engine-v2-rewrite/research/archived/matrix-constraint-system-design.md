# Matrix-Based Phonotactic Constraint System Design

**Report Date:** 2025-12-24
**Author:** AI Assistant
**Purpose:** Replace case-by-case validation with matrix-based constraint solver

---

## Executive Summary

Current validation (`validation.rs`) uses **sequential rule checking** with multiple arrays and pattern matching. This proposal introduces a **Matrix-Based Constraint Solver** that:

1. Consolidates 6+ rules into **5 compatibility matrices**
2. Provides **O(1) lookup** for all constraint checks
3. Encodes Vietnamese phonotactic rules **declaratively** (easier to audit/maintain)
4. Adds missing **Rule 7 (Tone-Stop Final)** naturally

---

## Current Problems

### Case-by-Case Approach (validation.rs)

```rust
// Current: Sequential rules, multiple data structures
const RULES: &[Rule] = &[
    rule_has_vowel,          // Function 1
    rule_valid_initial,       // Function 2 + VALID_INITIALS_1/2 arrays
    rule_all_chars_parsed,    // Function 3
    rule_spelling,            // Function 4 + SPELLING_RULES tuples
    rule_valid_final,         // Function 5 + VALID_FINALS_1/2 arrays
    rule_valid_vowel_pattern, // Function 6 + VALID_DIPHTHONGS/TRIPHTHONGS
];
```

**Issues:**
- Each rule is a separate function with its own data structures
- Spelling rules use tuple arrays requiring linear search
- No unified way to check constraint interactions
- Missing Tone-Stop Final rule (Rule 7)
- Hard to reason about edge cases

---

## Proposed: Matrix-Based Constraint Solver

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    PhonMatrix Constraint System                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│   │  M1: INIT_V  │  │  M2: VOWEL   │  │  M3: V_FINAL │          │
│   │   22×12      │  │   12×12      │  │   12×8       │          │
│   │ Initial+Vowel│  │ Vowel combo  │  │ Vowel+Final  │          │
│   └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
│   ┌──────────────┐  ┌──────────────┐                            │
│   │  M4: TONE_F  │  │  M5: MOD_REQ │                            │
│   │    6×8       │  │   43×3       │                            │
│   │ Tone+Final   │  │ Pattern→Mod  │                            │
│   └──────────────┘  └──────────────┘                            │
│                                                                  │
│              ┌─────────────────────────┐                        │
│              │   Constraint Solver     │                        │
│              │   validate(syllable) →  │                        │
│              │   check_all_matrices()  │                        │
│              └─────────────────────────┘                        │
└─────────────────────────────────────────────────────────────────┘
```

---

## Matrix Definitions

### M1: Initial-Vowel Compatibility (Spelling Rules)

Encodes c/k/q, g/gh, ng/ngh rules in a **22×12 matrix**.

```rust
/// Initial consonant indices (22 initials)
#[repr(u8)]
pub enum Initial {
    None = 0,  // No initial (vowel-only syllables)
    B, C, CH, D, G, GH, GI, H, K, KH, KR,
    L, M, N, NG, NGH, NH, P, PH, Q, QU, R, S, T, TH, TR, V, X
}

/// Vowel indices for matrix lookup (12 core vowels)
#[repr(u8)]
pub enum Vowel {
    A = 0, Ă, Â, E, Ê, I, O, Ô, Ơ, U, Ư, Y
}

/// M1: Initial → Vowel compatibility
/// true = allowed, false = spelling violation
pub const INIT_VOWEL_MATRIX: [[bool; 12]; 28] = {
    // Generate at compile time or define statically
    // Rows: Initial (None, B, C, CH, ...)
    // Cols: Vowel (A, Ă, Â, E, Ê, I, O, Ô, Ơ, U, Ư, Y)

    //       A    Ă    Â    E    Ê    I    O    Ô    Ơ    U    Ư    Y
    /* None */ [true,true,true,true,true,true,true,true,true,true,true,true],
    /* B    */ [true,true,true,true,true,true,true,true,true,true,true,true],
    /* C    */ [true,true,true,FALS,FALS,FALS,true,true,true,true,true,FALS], // c before e,i,y = false
    /* CH   */ [true,true,true,true,true,true,true,true,true,true,true,true],
    /* D    */ [true,true,true,true,true,true,true,true,true,true,true,true],
    /* G    */ [true,true,true,FALS,FALS,true,true,true,true,true,true,true], // g before e = false
    /* GH   */ [FALS,FALS,FALS,true,true,true,FALS,FALS,FALS,FALS,FALS,true], // gh NOT before a,o,u
    /* GI   */ [true,true,true,true,true,true,true,true,true,true,true,true],
    // ... continue for all initials
    /* K    */ [FALS,FALS,FALS,true,true,true,FALS,FALS,FALS,FALS,FALS,true], // k NOT before a,o,u
    /* NG   */ [true,true,true,FALS,FALS,FALS,true,true,true,true,true,FALS], // ng NOT before e,i
    /* NGH  */ [FALS,FALS,FALS,true,true,true,FALS,FALS,FALS,FALS,FALS,true], // ngh NOT before a,o,u
    // ...
};

/// O(1) spelling check
#[inline]
pub fn check_initial_vowel(initial: Initial, vowel: Vowel) -> bool {
    INIT_VOWEL_MATRIX[initial as usize][vowel as usize]
}
```

### M2: Vowel-Vowel Compatibility (Diphthongs/Triphthongs)

Encodes valid vowel combinations in a **12×12 matrix** for diphthongs.

```rust
/// M2: V1 → V2 compatibility for diphthongs
/// 0 = invalid, 1 = valid, 2 = valid but needs modifier check
pub const VOWEL_PAIR_MATRIX: [[u8; 12]; 12] = {
    //       A    Ă    Â    E    Ê    I    O    Ô    Ơ    U    Ư    Y
    /* A  */ [0,   0,   0,   0,   0,   1,   1,   0,   0,   1,   0,   1], // ai,ao,au,ay
    /* Ă  */ [0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0], // ă cannot lead diphthong
    /* Â  */ [0,   0,   0,   0,   0,   0,   0,   0,   0,   1,   0,   1], // âu,ây
    /* E  */ [0,   0,   0,   0,   0,   0,   1,   0,   0,   2,   0,   0], // eo, êu(needs check)
    /* Ê  */ [0,   0,   0,   0,   0,   0,   0,   0,   0,   1,   0,   0], // êu
    /* I  */ [1,   0,   0,   0,   2,   0,   0,   0,   0,   1,   0,   0], // ia, iê(needs check), iu
    /* O  */ [1,   2,   0,   1,   0,   1,   0,   0,   0,   0,   0,   0], // oa, oă(needs check), oe, oi
    /* Ô  */ [0,   0,   0,   0,   0,   1,   0,   0,   0,   0,   0,   0], // ôi
    /* Ơ  */ [0,   0,   0,   0,   0,   1,   0,   0,   0,   0,   0,   0], // ơi
    /* U  */ [1,   0,   2,   0,   2,   1,   2,   0,   0,   0,   0,   1], // ua,uâ,uê,ui,uô,uy
    /* Ư  */ [1,   0,   0,   0,   0,   1,   0,   0,   2,   1,   0,   0], // ưa,ưi,ươ,ưu
    /* Y  */ [0,   0,   0,   0,   2,   0,   0,   0,   0,   0,   0,   0], // yê(needs check)
};

/// Extended check for triphthongs (sparse - only ~13 valid patterns)
pub const VALID_TRIPHTHONGS_SET: phf::Set<[u8; 3]> = phf_set! {
    [Vowel::I, Vowel::Ê, Vowel::U],  // iêu
    [Vowel::Y, Vowel::Ê, Vowel::U],  // yêu
    [Vowel::O, Vowel::A, Vowel::I],  // oai
    [Vowel::O, Vowel::A, Vowel::Y],  // oay
    [Vowel::O, Vowel::E, Vowel::O],  // oeo
    [Vowel::U, Vowel::Â, Vowel::Y],  // uây
    [Vowel::U, Vowel::Ô, Vowel::I],  // uôi
    [Vowel::U, Vowel::Y, Vowel::A],  // uya
    [Vowel::Ư, Vowel::Ơ, Vowel::I],  // ươi
    [Vowel::Ư, Vowel::Ơ, Vowel::U],  // ươu
    [Vowel::U, Vowel::Y, Vowel::Ê],  // uyê
    [Vowel::U, Vowel::Y, Vowel::U],  // uyu
    [Vowel::U, Vowel::Ê, Vowel::U],  // uêu
};

/// O(1) diphthong check
#[inline]
pub fn check_vowel_pair(v1: Vowel, v2: Vowel) -> (bool, bool) {
    let val = VOWEL_PAIR_MATRIX[v1 as usize][v2 as usize];
    (val > 0, val == 2) // (is_valid, needs_modifier_check)
}
```

### M3: Vowel-Final Compatibility

Encodes which finals can follow which vowels.

```rust
/// Final consonant indices (8 finals + none)
#[repr(u8)]
pub enum Final {
    None = 0,
    C, CH, M, N, NG, NH, P, T
}

/// M3: Vowel → Final compatibility
/// Based on Vietnamese phonotactic rules:
/// - CH only after: a, ê, i (ach, êch, ich)
/// - NH only after: a, ê, i, y (anh, ênh, inh, ynh→ính)
/// - NG not after: e, ê (but valid after all others)
pub const VOWEL_FINAL_MATRIX: [[bool; 9]; 12] = {
    //       None  C    CH   M    N    NG   NH   P    T
    /* A  */ [true,true,true,true,true,true,true,true,true],
    /* Ă  */ [true,true,FALS,true,true,true,FALS,true,true], // ăch, ănh invalid
    /* Â  */ [true,true,FALS,true,true,true,FALS,true,true], // âch, ânh invalid
    /* E  */ [true,true,FALS,true,true,FALS,FALS,true,true], // ech,eng,enh invalid
    /* Ê  */ [true,true,true,true,true,FALS,true,true,true], // êng invalid
    /* I  */ [true,true,true,true,true,FALS,true,true,true], // ing invalid
    /* O  */ [true,true,FALS,true,true,true,FALS,true,true], // och, onh invalid
    /* Ô  */ [true,true,FALS,true,true,true,FALS,true,true], // ôch, ônh invalid
    /* Ơ  */ [true,true,FALS,true,true,FALS,FALS,true,true], // ơch,ơng,ơnh invalid
    /* U  */ [true,true,FALS,true,true,true,FALS,true,true], // uch, unh invalid
    /* Ư  */ [true,true,FALS,true,true,FALS,FALS,true,true], // ưch,ưng,ưnh invalid
    /* Y  */ [true,true,FALS,true,true,FALS,true,true,true], // ych, yng invalid
};

/// O(1) vowel-final check
#[inline]
pub fn check_vowel_final(vowel: Vowel, final_c: Final) -> bool {
    VOWEL_FINAL_MATRIX[vowel as usize][final_c as usize]
}
```

### M4: Tone-Final Compatibility (Rule 7)

**THE MISSING RULE!** Stop finals (p, t, c, ch) only allow sắc/nặng tones.

```rust
/// Tone indices
#[repr(u8)]
pub enum Tone {
    None = 0,   // ngang (level)
    Sac,        // sắc (acute) ´
    Huyen,      // huyền (grave) `
    Hoi,        // hỏi (hook) ̉
    Nga,        // ngã (tilde) ~
    Nang,       // nặng (dot) .
}

/// M4: Tone → Final compatibility (Rule 7)
/// Stop finals (p, t, c, ch) ONLY allow: none, sắc, nặng
pub const TONE_FINAL_MATRIX: [[bool; 9]; 6] = {
    //        None  C    CH   M    N    NG   NH   P    T
    /* None */ [true,true,true,true,true,true,true,true,true],
    /* Sắc  */ [true,true,true,true,true,true,true,true,true],
    /* Huyền*/ [true,FALS,FALS,true,true,true,true,FALS,FALS], // p,t,c,ch invalid
    /* Hỏi  */ [true,FALS,FALS,true,true,true,true,FALS,FALS], // p,t,c,ch invalid
    /* Ngã  */ [true,FALS,FALS,true,true,true,true,FALS,FALS], // p,t,c,ch invalid
    /* Nặng */ [true,true,true,true,true,true,true,true,true],
};

/// O(1) tone-final check (Rule 7)
#[inline]
pub fn check_tone_final(tone: Tone, final_c: Final) -> bool {
    TONE_FINAL_MATRIX[tone as usize][final_c as usize]
}
```

### M5: Modifier Requirements

Maps vowel patterns to required modifiers.

```rust
/// Modifier requirements for vowel patterns
/// Key: (v1, v2) or (v1, v2, v3) encoded as u32
/// Value: Required modifier position + type
#[derive(Clone, Copy)]
pub struct ModifierReq {
    pub position: u8,      // 0=V1, 1=V2, 2=V3
    pub modifier: u8,      // 1=circumflex, 2=horn, 3=breve
}

/// Compile-time map of modifier requirements
pub const MODIFIER_REQUIREMENTS: phf::Map<u16, ModifierReq> = phf_map! {
    // Diphthongs requiring circumflex on V1
    encode_pair(E, U) => ModifierReq { position: 0, modifier: CIRCUMFLEX }, // êu

    // Diphthongs requiring circumflex on V2
    encode_pair(I, E) => ModifierReq { position: 1, modifier: CIRCUMFLEX }, // iê
    encode_pair(U, E) => ModifierReq { position: 1, modifier: CIRCUMFLEX }, // uê
    encode_pair(Y, E) => ModifierReq { position: 1, modifier: CIRCUMFLEX }, // yê

    // Diphthongs requiring horn on V1
    encode_pair(U, A) => ModifierReq { position: 0, modifier: HORN_OPTIONAL }, // ưa (optional horn)
    encode_pair(U, O) => ModifierReq { position: 0, modifier: HORN_OPTIONAL }, // ươ (optional horn)

    // ... etc
};
```

---

## Unified Constraint Solver

```rust
/// Validation result with constraint violation details
pub enum ConstraintViolation {
    Valid,
    NoVowel,
    InvalidInitial(Initial),
    SpellingViolation { initial: Initial, vowel: Vowel },
    InvalidVowelPattern { vowels: Vec<Vowel> },
    InvalidFinal(Final),
    VowelFinalMismatch { vowel: Vowel, final_c: Final },
    ToneFinalMismatch { tone: Tone, final_c: Final },  // Rule 7
    MissingModifier { position: u8, required: u8 },
}

/// Main validation entry point - checks all matrices
pub fn validate_syllable(syllable: &ParsedSyllable) -> ConstraintViolation {
    // Phase 1: Structural validation
    if syllable.vowels.is_empty() {
        return ConstraintViolation::NoVowel;
    }

    // Phase 2: Initial validation
    if !is_valid_initial(syllable.initial) {
        return ConstraintViolation::InvalidInitial(syllable.initial);
    }

    // Phase 3: M1 - Initial + Vowel spelling check
    if syllable.initial != Initial::None && !syllable.vowels.is_empty() {
        let first_vowel = syllable.vowels[0];
        if !check_initial_vowel(syllable.initial, first_vowel) {
            return ConstraintViolation::SpellingViolation {
                initial: syllable.initial,
                vowel: first_vowel,
            };
        }
    }

    // Phase 4: M2 - Vowel pattern validation
    match syllable.vowels.len() {
        1 => {}, // Single vowel always valid
        2 => {
            let (valid, needs_check) = check_vowel_pair(
                syllable.vowels[0],
                syllable.vowels[1]
            );
            if !valid {
                return ConstraintViolation::InvalidVowelPattern {
                    vowels: syllable.vowels.to_vec(),
                };
            }
            // Phase 4b: M5 - Modifier requirements
            if needs_check {
                if let Some(violation) = check_modifier_requirements(syllable) {
                    return violation;
                }
            }
        }
        3 => {
            if !check_triphthong(syllable.vowels) {
                return ConstraintViolation::InvalidVowelPattern {
                    vowels: syllable.vowels.to_vec(),
                };
            }
        }
        _ => return ConstraintViolation::InvalidVowelPattern {
            vowels: syllable.vowels.to_vec(),
        },
    }

    // Phase 5: Final validation
    if !is_valid_final(syllable.final_c) {
        return ConstraintViolation::InvalidFinal(syllable.final_c);
    }

    // Phase 6: M3 - Vowel + Final compatibility
    if syllable.final_c != Final::None {
        let main_vowel = get_main_vowel(syllable);
        if !check_vowel_final(main_vowel, syllable.final_c) {
            return ConstraintViolation::VowelFinalMismatch {
                vowel: main_vowel,
                final_c: syllable.final_c,
            };
        }
    }

    // Phase 7: M4 - Tone + Final compatibility (RULE 7!)
    if syllable.final_c != Final::None && syllable.tone != Tone::None {
        if !check_tone_final(syllable.tone, syllable.final_c) {
            return ConstraintViolation::ToneFinalMismatch {
                tone: syllable.tone,
                final_c: syllable.final_c,
            };
        }
    }

    ConstraintViolation::Valid
}
```

---

## Performance Comparison

| Operation | Current (Case-by-Case) | Matrix-Based |
|-----------|----------------------|--------------|
| Spelling check | O(n) linear scan of SPELLING_RULES | **O(1)** matrix lookup |
| Vowel pattern | O(n) scan of 29 diphthongs + 13 triphthongs | **O(1)** matrix + O(1) set |
| Final check | O(n) scan of VALID_FINALS | **O(1)** matrix lookup |
| Tone-Final (new) | Not implemented | **O(1)** matrix lookup |
| Modifier check | Scattered if-statements | **O(1)** map lookup |
| **Total** | **6 function calls + ~5 linear scans** | **7 O(1) lookups** |

---

## Memory Footprint

```
Current approach:
- VALID_INITIALS_1: 16 × 2 bytes = 32 bytes
- VALID_INITIALS_2: 11 × 4 bytes = 44 bytes
- VALID_FINALS_1: 10 × 2 bytes = 20 bytes
- VALID_FINALS_2: 3 × 4 bytes = 12 bytes
- VALID_DIPHTHONGS: 29 × 4 bytes = 116 bytes
- VALID_TRIPHTHONGS: 13 × 6 bytes = 78 bytes
- SPELLING_RULES: ~200 bytes (fat pointers + strings)
Total: ~502 bytes

Matrix approach:
- INIT_VOWEL_MATRIX: 28 × 12 = 336 bytes (bits can compress to 42 bytes)
- VOWEL_PAIR_MATRIX: 12 × 12 = 144 bytes (bits: 18 bytes)
- VOWEL_FINAL_MATRIX: 12 × 9 = 108 bytes (bits: 14 bytes)
- TONE_FINAL_MATRIX: 6 × 9 = 54 bytes (bits: 7 bytes)
- TRIPHTHONG_SET: 13 × 3 = 39 bytes + phf overhead (~100 bytes)
- MODIFIER_MAP: ~200 bytes
Total: ~800 bytes (or ~380 bytes with bit-packing)
```

**Conclusion:** Matrix approach uses ~300 more bytes but provides O(1) for ALL operations.

---

## Benefits Summary

### 1. **Declarative Rules**
Rules are encoded in matrices, not scattered across functions. Easy to audit, verify against documentation.

### 2. **Unified API**
Single `validate_syllable()` function replaces 6+ rule functions.

### 3. **Rule 7 Integrated**
Tone-Stop Final rule naturally fits as another matrix.

### 4. **O(1) Everywhere**
All constraint checks become O(1) matrix/map lookups.

### 5. **Better Error Reporting**
`ConstraintViolation` enum provides specific failure reasons.

### 6. **Compile-Time Verification**
Matrix constants verified at compile time. No runtime surprises.

---

## Implementation Plan

### Phase 1A: Define Enums & Matrices (NEW)
1. Create `core/src/engine/validation/phonotactics.rs`
2. Define `Initial`, `Vowel`, `Final`, `Tone` enums
3. Define all 5 matrices as const arrays
4. Add `phf` dependency for perfect hash maps

### Phase 1B: Implement Constraint Solver
1. Create `core/src/engine/validation/solver.rs`
2. Implement `validate_syllable()` using matrices
3. Add comprehensive tests against current test suite

### Phase 1C: Integration
1. Update `validation.rs` to use new solver
2. Deprecate old rule functions (keep for comparison)
3. Benchmark: old vs new

### Phase 1D: Rule 7 Addition
1. Matrix already includes Tone-Final check
2. Add specific tests for Rule 7
3. Document in `engine-architecture-v2.md`

---

## Appendix: Full Matrix Data

### A1: Complete INIT_VOWEL_MATRIX

Based on Vietnamese spelling rules (c/k, g/gh, ng/ngh):

```
Initial  | A  Ă  Â  E  Ê  I  O  Ô  Ơ  U  Ư  Y  | Notes
---------|----------------------------------------|---------------------------
(None)   | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | Vowel-only syllables OK
B        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
C        | ✓  ✓  ✓  ✗  ✗  ✗  ✓  ✓  ✓  ✓  ✓  ✗  | C not before E,I,Y (use K)
CH       | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
D        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
Đ        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
G        | ✓  ✓  ✓  ✗  ✗  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | G not before E (use GH)
GH       | ✗  ✗  ✗  ✓  ✓  ✓  ✗  ✗  ✗  ✗  ✗  ✓  | GH only before E,Ê,I,Y
GI       | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
H        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
K        | ✗  ✗  ✗  ✓  ✓  ✓  ✗  ✗  ✗  ✗  ✗  ✓  | K only before E,Ê,I,Y
KH       | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
L        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
M        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
N        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
NG       | ✓  ✓  ✓  ✗  ✗  ✗  ✓  ✓  ✓  ✓  ✓  ✗  | NG not before E,I,Y (use NGH)
NGH      | ✗  ✗  ✗  ✓  ✓  ✓  ✗  ✗  ✗  ✗  ✗  ✓  | NGH only before E,Ê,I,Y
NH       | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
P        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
PH       | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
Q        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | (Q always with U)
QU       | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
R        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
S        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
T        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
TH       | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
TR       | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
V        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
X        | ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  ✓  | No restrictions
```

### A2: Complete VOWEL_PAIR_MATRIX

```
V1 → V2  | A  Ă  Â  E  Ê  I  O  Ô  Ơ  U  Ư  Y  | Valid diphthongs
---------|----------------------------------------|------------------
A        | -  -  -  -  -  ✓  ✓  -  -  ✓  -  ✓  | ai, ao, au, ay
Ă        | -  -  -  -  -  -  -  -  -  -  -  -  | (none - ă can't lead)
Â        | -  -  -  -  -  -  -  -  -  ✓  -  ✓  | âu, ây
E        | -  -  -  -  -  -  ✓  -  -  *  -  -  | eo, êu*
Ê        | -  -  -  -  -  -  -  -  -  ✓  -  -  | êu
I        | ✓  -  -  -  *  -  -  -  -  ✓  -  -  | ia, iê*, iu
O        | ✓  *  -  ✓  -  ✓  -  -  -  -  -  -  | oa, oă*, oe, oi
Ô        | -  -  -  -  -  ✓  -  -  -  -  -  -  | ôi
Ơ        | -  -  -  -  -  ✓  -  -  -  -  -  -  | ơi
U        | ✓  -  *  -  *  ✓  *  -  -  -  -  ✓  | ua, uâ*, uê*, ui, uô*, uy
Ư        | ✓  -  -  -  -  ✓  -  -  *  ✓  -  -  | ưa, ưi, ươ*, ưu
Y        | -  -  -  -  *  -  -  -  -  -  -  -  | yê*

* = needs modifier check (circumflex or horn required)
```

---

## Conclusion

The Matrix-Based Constraint System provides:
- **O(1) performance** for all constraint checks
- **Declarative rule encoding** - easy to verify against documentation
- **Natural Rule 7 integration** - Tone-Final matrix
- **Better maintainability** - add rules by updating matrices, not code
- **Comprehensive error reporting** - specific violation types

Recommend proceeding with implementation in Phase 1 of the v2 rewrite.
