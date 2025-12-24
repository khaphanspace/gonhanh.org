# Vietnamese Matrix Data - Complete Implementation Reference

**Date**: 2025-12-24
**Source**: docs/vietnamese-language-system.md Section 7.6.1

---

## M7: TONE_PLACEMENT Matrix - Complete 43 Patterns

Pattern index → which vowel position receives tone mark.

| Idx | Pattern | Keys | Default | After_Q | With_Final | Notes |
|-----|---------|------|---------|---------|------------|-------|
| 0 | (single) | V | 1 | 1 | 1 | Single vowel always V1 |
| 1 | ai | A+I | 1 | 1 | 1 | **á**i |
| 2 | ao | A+O | 1 | 1 | 1 | **á**o |
| 3 | au | A+U | 1 | 1 | 1 | **á**u |
| 4 | ay | A+Y | 1 | 1 | 1 | **á**y |
| 5 | âu | A+U+^ | 1 | 1 | 1 | **ấ**u |
| 6 | ây | A+Y+^ | 1 | 1 | 1 | **ấ**y |
| 7 | eo | E+O | 1 | 1 | 1 | **é**o |
| 8 | êu | E+U+^ | 1 | 1 | 1 | **ế**u |
| 9 | ia | I+A | 1 | 1 | 1 | **í**a |
| 10 | iê | I+E+^ | 2 | 2 | 2 | i**ế** |
| 11 | iu | I+U | 1 | 1 | 1 | **í**u |
| 12 | oa | O+A | 2 | 2 | 2 | o**á** (modern) |
| 13 | oă | O+A+˘ | 2 | 2 | 2 | o**ắ** |
| 14 | oe | O+E | 2 | 2 | 2 | o**é** (modern) |
| 15 | oi | O+I | 1 | 1 | 1 | **ó**i |
| 16 | ôi | O+I+^ | 1 | 1 | 1 | **ố**i |
| 17 | ơi | O+I+ʼ | 1 | 1 | 1 | **ớ**i |
| 18 | ua | U+A | 1 | 2 | 2 | **ú**a open, qu**á**/mu**á**n |
| 19 | uâ | U+A+^ | 2 | 2 | 2 | u**ấ** |
| 20 | uê | U+E+^ | 2 | 2 | 2 | u**ế** |
| 21 | ui | U+I | 1 | 1 | 1 | **ú**i |
| 22 | uô | U+O+^ | 2 | 2 | 2 | u**ố** |
| 23 | uy | U+Y | 2 | 2 | 2 | u**ý** |
| 24 | ưa | U+A+ʼ | 1 | 1 | 1 | **ứ**a |
| 25 | ưi | U+I+ʼ | 1 | 1 | 1 | **ứ**i |
| 26 | ươ | U+O+ʼʼ | 2 | 2 | 2 | ư**ớ** |
| 27 | ưu | U+U+ʼ | 1 | 1 | 1 | **ứ**u |
| 28 | yê | Y+E+^ | 2 | 2 | 2 | y**ế** |
| 29 | iêu | I+E+U+^ | 2 | 2 | 2 | i**ế**u |
| 30 | yêu | Y+E+U+^ | 2 | 2 | 2 | y**ế**u |
| 31 | oai | O+A+I | 2 | 2 | 2 | o**á**i |
| 32 | oay | O+A+Y | 2 | 2 | 2 | o**á**y |
| 33 | oeo | O+E+O | 2 | 2 | 2 | o**é**o |
| 34 | uây | U+A+Y+^ | 2 | 2 | 2 | u**ấ**y |
| 35 | uôi | U+O+I+^ | 2 | 2 | 2 | u**ố**i |
| 36 | uya | U+Y+A | 2 | 2 | 2 | u**ý**a |
| 37 | ươi | U+O+I+ʼʼ | 2 | 2 | 2 | ư**ớ**i |
| 38 | ươu | U+O+U+ʼʼ | 2 | 2 | 2 | ư**ớ**u |
| 39 | uyê | U+Y+E+^ | 3 | 3 | 3 | uy**ế** |
| 40 | uyu | U+Y+U | 2 | 2 | 2 | u**ý**u |
| 41 | uêu | U+E+U+^ | 2 | 2 | 2 | u**ế**u |
| 42 | oao | O+A+O | 2 | 2 | 2 | o**á**o |

### Rust Constant

```rust
/// Tone placement position for each vowel pattern
/// Values: 1=V1, 2=V2, 3=V3
/// Columns: [Default, After_Q, With_Final, Modern_Style]
pub static M_TONE_PLACEMENT: [[u8; 4]; 43] = [
    [1, 1, 1, 1], // 0: single vowel
    [1, 1, 1, 1], // 1: ai
    [1, 1, 1, 1], // 2: ao
    [1, 1, 1, 1], // 3: au
    [1, 1, 1, 1], // 4: ay
    [1, 1, 1, 1], // 5: âu
    [1, 1, 1, 1], // 6: ây
    [1, 1, 1, 1], // 7: eo
    [1, 1, 1, 1], // 8: êu
    [1, 1, 1, 1], // 9: ia
    [2, 2, 2, 2], // 10: iê
    [1, 1, 1, 1], // 11: iu
    [2, 2, 2, 2], // 12: oa
    [2, 2, 2, 2], // 13: oă
    [2, 2, 2, 2], // 14: oe
    [1, 1, 1, 1], // 15: oi
    [1, 1, 1, 1], // 16: ôi
    [1, 1, 1, 1], // 17: ơi
    [1, 2, 2, 2], // 18: ua (context-dependent!)
    [2, 2, 2, 2], // 19: uâ
    [2, 2, 2, 2], // 20: uê
    [1, 1, 1, 1], // 21: ui
    [2, 2, 2, 2], // 22: uô
    [2, 2, 2, 2], // 23: uy
    [1, 1, 1, 1], // 24: ưa
    [1, 1, 1, 1], // 25: ưi
    [2, 2, 2, 2], // 26: ươ
    [1, 1, 1, 1], // 27: ưu
    [2, 2, 2, 2], // 28: yê
    [2, 2, 2, 2], // 29: iêu
    [2, 2, 2, 2], // 30: yêu
    [2, 2, 2, 2], // 31: oai
    [2, 2, 2, 2], // 32: oay
    [2, 2, 2, 2], // 33: oeo
    [2, 2, 2, 2], // 34: uây
    [2, 2, 2, 2], // 35: uôi
    [2, 2, 2, 2], // 36: uya
    [2, 2, 2, 2], // 37: ươi
    [2, 2, 2, 2], // 38: ươu
    [3, 3, 3, 3], // 39: uyê (V3!)
    [2, 2, 2, 2], // 40: uyu
    [2, 2, 2, 2], // 41: uêu
    [2, 2, 2, 2], // 42: oao
];
```

---

## M8: MODIFIER_PLACEMENT Matrix - Complete 43 Patterns

Pattern index → bitmask of which vowels receive modifiers.

| Idx | Pattern | Modifier | Targets | Bitmask | Notes |
|-----|---------|----------|---------|---------|-------|
| 0 | (single) | varies | V1 | 0x01 | Depends on input |
| 1 | ai | - | none | 0x00 | No modifier |
| 2 | ao | - | none | 0x00 | No modifier |
| 3 | au | - | none | 0x00 | No modifier |
| 4 | ay | - | none | 0x00 | No modifier |
| 5 | âu | ^ | V1 | 0x01 | a→â |
| 6 | ây | ^ | V1 | 0x01 | a→â |
| 7 | eo | - | none | 0x00 | No modifier |
| 8 | êu | ^ | V1 | 0x01 | e→ê |
| 9 | ia | - | none | 0x00 | No modifier |
| 10 | iê | ^ | V2 | 0x02 | e→ê |
| 11 | iu | - | none | 0x00 | No modifier |
| 12 | oa | - | none | 0x00 | No modifier |
| 13 | oă | ˘ | V2 | 0x02 | a→ă |
| 14 | oe | - | none | 0x00 | No modifier |
| 15 | oi | - | none | 0x00 | No modifier |
| 16 | ôi | ^ | V1 | 0x01 | o→ô |
| 17 | ơi | ʼ | V1 | 0x01 | o→ơ |
| 18 | ua | - | none | 0x00 | No modifier |
| 19 | uâ | ^ | V2 | 0x02 | a→â |
| 20 | uê | ^ | V2 | 0x02 | e→ê |
| 21 | ui | - | none | 0x00 | No modifier |
| 22 | uô | ^ | V2 | 0x02 | o→ô |
| 23 | uy | - | none | 0x00 | No modifier |
| 24 | ưa | ʼ | V1 | 0x01 | u→ư |
| 25 | ưi | ʼ | V1 | 0x01 | u→ư |
| 26 | ươ | ʼ | V1+V2 | 0x03 | u→ư, o→ơ |
| 27 | ưu | ʼ | V1 | 0x01 | u₁→ư (u₂ unchanged) |
| 28 | yê | ^ | V2 | 0x02 | e→ê |
| 29 | iêu | ^ | V2 | 0x02 | e→ê |
| 30 | yêu | ^ | V2 | 0x02 | e→ê |
| 31 | oai | - | none | 0x00 | No modifier |
| 32 | oay | - | none | 0x00 | No modifier |
| 33 | oeo | - | none | 0x00 | No modifier |
| 34 | uây | ^ | V2 | 0x02 | a→â |
| 35 | uôi | ^ | V2 | 0x02 | o→ô |
| 36 | uya | - | none | 0x00 | No modifier |
| 37 | ươi | ʼ | V1+V2 | 0x03 | u→ư, o→ơ |
| 38 | ươu | ʼ | V1+V2 | 0x03 | u→ư, o→ơ (u₃ unchanged) |
| 39 | uyê | ^ | V3 | 0x04 | e→ê |
| 40 | uyu | - | none | 0x00 | No modifier |
| 41 | uêu | ^ | V2 | 0x02 | e→ê |
| 42 | oao | - | none | 0x00 | No modifier |

### Rust Constant

```rust
/// Modifier placement bitmask for each vowel pattern
/// Bits: 0x01=V1, 0x02=V2, 0x04=V3
pub static M_MODIFIER_PLACEMENT: [u8; 43] = [
    0x01, // 0: single (placeholder)
    0x00, // 1: ai
    0x00, // 2: ao
    0x00, // 3: au
    0x00, // 4: ay
    0x01, // 5: âu (V1)
    0x01, // 6: ây (V1)
    0x00, // 7: eo
    0x01, // 8: êu (V1)
    0x00, // 9: ia
    0x02, // 10: iê (V2)
    0x00, // 11: iu
    0x00, // 12: oa
    0x02, // 13: oă (V2)
    0x00, // 14: oe
    0x00, // 15: oi
    0x01, // 16: ôi (V1)
    0x01, // 17: ơi (V1)
    0x00, // 18: ua
    0x02, // 19: uâ (V2)
    0x02, // 20: uê (V2)
    0x00, // 21: ui
    0x02, // 22: uô (V2)
    0x00, // 23: uy
    0x01, // 24: ưa (V1)
    0x01, // 25: ưi (V1)
    0x03, // 26: ươ (V1+V2)
    0x01, // 27: ưu (V1 only)
    0x02, // 28: yê (V2)
    0x02, // 29: iêu (V2)
    0x02, // 30: yêu (V2)
    0x00, // 31: oai
    0x00, // 32: oay
    0x00, // 33: oeo
    0x02, // 34: uây (V2)
    0x02, // 35: uôi (V2)
    0x00, // 36: uya
    0x03, // 37: ươi (V1+V2)
    0x03, // 38: ươu (V1+V2)
    0x04, // 39: uyê (V3)
    0x00, // 40: uyu
    0x02, // 41: uêu (V2)
    0x00, // 42: oao
];

/// Modifier type for each pattern
/// 0=none, 1=circumflex, 2=breve, 3=horn
pub static M_MODIFIER_TYPE: [u8; 43] = [
    0, // 0: single
    0, // 1: ai
    0, // 2: ao
    0, // 3: au
    0, // 4: ay
    1, // 5: âu (^)
    1, // 6: ây (^)
    0, // 7: eo
    1, // 8: êu (^)
    0, // 9: ia
    1, // 10: iê (^)
    0, // 11: iu
    0, // 12: oa
    2, // 13: oă (˘)
    0, // 14: oe
    0, // 15: oi
    1, // 16: ôi (^)
    3, // 17: ơi (ʼ)
    0, // 18: ua
    1, // 19: uâ (^)
    1, // 20: uê (^)
    0, // 21: ui
    1, // 22: uô (^)
    0, // 23: uy
    3, // 24: ưa (ʼ)
    3, // 25: ưi (ʼ)
    3, // 26: ươ (ʼ)
    3, // 27: ưu (ʼ)
    1, // 28: yê (^)
    1, // 29: iêu (^)
    1, // 30: yêu (^)
    0, // 31: oai
    0, // 32: oay
    0, // 33: oeo
    1, // 34: uây (^)
    1, // 35: uôi (^)
    0, // 36: uya
    3, // 37: ươi (ʼ)
    3, // 38: ươu (ʼ)
    1, // 39: uyê (^)
    0, // 40: uyu
    1, // 41: uêu (^)
    0, // 42: oao
];
```

---

## M2: INITIAL_VOWEL Matrix (29×12)

Validates initial consonant + first vowel combinations.

```rust
/// Initial consonant index mapping
pub const INIT_B: usize = 0;
pub const INIT_C: usize = 1;
pub const INIT_D: usize = 2;
pub const INIT_DD: usize = 3;  // đ
pub const INIT_G: usize = 4;
pub const INIT_H: usize = 5;
pub const INIT_K: usize = 6;
pub const INIT_L: usize = 7;
pub const INIT_M: usize = 8;
pub const INIT_N: usize = 9;
pub const INIT_P: usize = 10;
pub const INIT_Q: usize = 11;
pub const INIT_R: usize = 12;
pub const INIT_S: usize = 13;
pub const INIT_T: usize = 14;
pub const INIT_V: usize = 15;
pub const INIT_X: usize = 16;
pub const INIT_CH: usize = 17;
pub const INIT_GH: usize = 18;
pub const INIT_GI: usize = 19;
pub const INIT_KH: usize = 20;
pub const INIT_KR: usize = 21;
pub const INIT_NG: usize = 22;
pub const INIT_NH: usize = 23;
pub const INIT_PH: usize = 24;
pub const INIT_QU: usize = 25;
pub const INIT_TH: usize = 26;
pub const INIT_TR: usize = 27;
pub const INIT_NGH: usize = 28;

/// Vowel index mapping
pub const VOW_A: usize = 0;
pub const VOW_AW: usize = 1;  // ă
pub const VOW_AA: usize = 2;  // â
pub const VOW_E: usize = 3;
pub const VOW_EE: usize = 4;  // ê
pub const VOW_I: usize = 5;
pub const VOW_O: usize = 6;
pub const VOW_OO: usize = 7;  // ô
pub const VOW_OW: usize = 8;  // ơ
pub const VOW_U: usize = 9;
pub const VOW_UW: usize = 10; // ư
pub const VOW_Y: usize = 11;

/// Initial + Vowel validity matrix
/// 0=INVALID, 1=VALID, 2=RARE/LOAN
///          a  ă  â  e  ê  i  o  ô  ơ  u  ư  y
pub static M_INITIAL_VOWEL: [[u8; 12]; 29] = [
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // b
    [1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0], // c (not before e,ê,i,y)
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // d
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // đ
    [1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0], // g (not before e,ê; gi handles i)
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // h
    [0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1], // k (only before e,ê,i,y)
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // l
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // m
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // n
    [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0], // p (rare, loan words)
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0], // q (only qu- pattern)
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // r
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // s
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // t
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // v
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // x
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // ch
    [0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0], // gh (only before e,ê,i)
    [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0], // gi (gi+a, gi+o patterns)
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // kh
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // kr (ethnic minorities)
    [1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0], // ng (not before e,ê,i)
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // nh
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // ph
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1], // qu (always qu+vowel, not qu+u/ư)
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // th
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // tr
    [0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0], // ngh (only before e,ê,i)
];
```

---

## M5: VOWEL_FINAL Matrix (12×9)

Validates vowel nucleus + final consonant combinations.

```rust
/// Final consonant index mapping
pub const FIN_C: usize = 0;
pub const FIN_CH: usize = 1;
pub const FIN_M: usize = 2;
pub const FIN_N: usize = 3;
pub const FIN_NG: usize = 4;
pub const FIN_NH: usize = 5;
pub const FIN_P: usize = 6;
pub const FIN_T: usize = 7;
pub const FIN_SEMI: usize = 8;  // semivowel (i/y/o/u)

/// Vowel + Final consonant validity matrix
/// 0=INVALID, 1=VALID
///          c  ch m  n  ng nh p  t  sv
pub static M_VOWEL_FINAL: [[u8; 9]; 12] = [
    [1, 1, 1, 1, 1, 1, 1, 1, 1], // a
    [1, 1, 1, 1, 1, 1, 1, 1, 0], // ă (no semivowel ending)
    [1, 0, 1, 1, 1, 0, 1, 1, 0], // â (no -âch, -ânh)
    [0, 0, 1, 1, 0, 0, 0, 0, 1], // e (no consonant finals except m,n)
    [0, 1, 1, 1, 0, 1, 1, 1, 1], // ê (no -êc, -êng)
    [0, 1, 1, 1, 0, 1, 1, 1, 1], // i (no -ic, -ing)
    [1, 0, 1, 1, 1, 0, 1, 1, 1], // o
    [1, 0, 1, 1, 1, 0, 1, 1, 1], // ô
    [1, 0, 1, 1, 0, 0, 1, 1, 1], // ơ (no -ơng, -ơnh)
    [0, 0, 1, 1, 1, 0, 1, 1, 1], // u (no -uc, -uch)
    [1, 0, 1, 1, 1, 0, 1, 1, 1], // ư (no -ưch)
    [0, 0, 0, 0, 0, 0, 0, 0, 1], // y (only semivowel)
];
```

---

## M6: TONE_FINAL Matrix (6×4)

Validates tone + stop final combinations (Rule 7).

```rust
/// Tone index mapping
pub const TONE_NGANG: usize = 0;
pub const TONE_SAC: usize = 1;
pub const TONE_HUYEN: usize = 2;
pub const TONE_HOI: usize = 3;
pub const TONE_NGA: usize = 4;
pub const TONE_NANG: usize = 5;

/// Stop final index mapping
pub const STOP_P: usize = 0;
pub const STOP_T: usize = 1;
pub const STOP_C: usize = 2;
pub const STOP_CH: usize = 3;

/// Tone + Stop Final validity matrix
/// 0=INVALID, 1=VALID
///          p  t  c  ch
pub static M_TONE_FINAL: [[u8; 4]; 6] = [
    [0, 0, 0, 0], // ngang - INVALID with stop
    [1, 1, 1, 1], // sắc - VALID
    [0, 0, 0, 0], // huyền - INVALID with stop
    [0, 0, 0, 0], // hỏi - INVALID with stop
    [0, 0, 0, 0], // ngã - INVALID with stop
    [1, 1, 1, 1], // nặng - VALID
];
```

---

## M3: VOWEL_PAIR Matrix (12×12)

Validates diphthong base key combinations.

```rust
/// Vowel pair validity matrix
/// Values: 0=INVALID, 1=VALID, 2=REQ_CIRCUMFLEX_V1, 3=REQ_CIRCUMFLEX_V2,
///         4=REQ_HORN_V1, 5=REQ_HORN_BOTH, 6=CONTEXT_DEPENDENT
///          a  ă  â  e  ê  i  o  ô  ơ  u  ư  y
pub static M_VOWEL_PAIR: [[u8; 12]; 12] = [
    [0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1], // a: ai,ao,au,ay
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // ă: never V1
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1], // â: âu,ây
    [0, 0, 0, 0, 0, 0, 1, 0, 0, 2, 0, 0], // e: eo, e+u=êu (V1 needs ^)
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0], // ê: êu
    [1, 0, 0, 3, 0, 0, 0, 0, 0, 1, 0, 0], // i: ia, i+e=iê (V2 needs ^), iu
    [1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0], // o: oa,oă,oe,oi
    [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0], // ô: ôi
    [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0], // ơ: ơi
    [6, 0, 1, 3, 0, 1, 1, 0, 0, 0, 0, 1], // u: ua(ctx),uâ,uê,ui,uô,uy
    [4, 0, 0, 0, 0, 4, 0, 0, 5, 4, 0, 0], // ư: ưa,ưi,ươ(both),ưu
    [0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0], // y: y+e=yê (V2 needs ^)
];
```

---

## Pattern Index Lookup

```rust
/// Convert vowel pattern to index for matrix lookup
pub fn pattern_to_idx(v1: char, v2: Option<char>, v3: Option<char>) -> usize {
    match (v1, v2, v3) {
        (_, None, None) => 0, // Single vowel
        ('a', Some('i'), None) => 1,
        ('a', Some('o'), None) => 2,
        ('a', Some('u'), None) => 3,
        ('a', Some('y'), None) => 4,
        ('â', Some('u'), None) => 5,
        ('â', Some('y'), None) => 6,
        ('e', Some('o'), None) => 7,
        ('ê', Some('u'), None) => 8,
        ('i', Some('a'), None) => 9,
        ('i', Some('ê'), None) => 10,
        ('i', Some('u'), None) => 11,
        ('o', Some('a'), None) => 12,
        ('o', Some('ă'), None) => 13,
        ('o', Some('e'), None) => 14,
        ('o', Some('i'), None) => 15,
        ('ô', Some('i'), None) => 16,
        ('ơ', Some('i'), None) => 17,
        ('u', Some('a'), None) => 18,
        ('u', Some('â'), None) => 19,
        ('u', Some('ê'), None) => 20,
        ('u', Some('i'), None) => 21,
        ('u', Some('ô'), None) => 22,
        ('u', Some('y'), None) => 23,
        ('ư', Some('a'), None) => 24,
        ('ư', Some('i'), None) => 25,
        ('ư', Some('ơ'), None) => 26,
        ('ư', Some('u'), None) => 27,
        ('y', Some('ê'), None) => 28,
        // Triphthongs
        ('i', Some('ê'), Some('u')) => 29,
        ('y', Some('ê'), Some('u')) => 30,
        ('o', Some('a'), Some('i')) => 31,
        ('o', Some('a'), Some('y')) => 32,
        ('o', Some('e'), Some('o')) => 33,
        ('u', Some('â'), Some('y')) => 34,
        ('u', Some('ô'), Some('i')) => 35,
        ('u', Some('y'), Some('a')) => 36,
        ('ư', Some('ơ'), Some('i')) => 37,
        ('ư', Some('ơ'), Some('u')) => 38,
        ('u', Some('y'), Some('ê')) => 39,
        ('u', Some('y'), Some('u')) => 40,
        ('u', Some('ê'), Some('u')) => 41,
        ('o', Some('a'), Some('o')) => 42,
        _ => 0, // Unknown pattern
    }
}
```

---

## Usage Example

```rust
fn validate_and_place_tone(syllable: &ParsedSyllable) -> Result<TonePlacement, ValidationError> {
    // Step 1: Get pattern index
    let pattern_idx = pattern_to_idx(
        syllable.v1,
        syllable.v2,
        syllable.v3
    );

    // Step 2: Determine context
    let context_col = if syllable.has_final() { 2 }
                      else if syllable.after_q() { 1 }
                      else { 0 };

    // Step 3: Matrix lookup for tone position
    let tone_pos = M_TONE_PLACEMENT[pattern_idx][context_col] as usize;

    // Step 4: Matrix lookup for modifier placement
    let mod_mask = M_MODIFIER_PLACEMENT[pattern_idx];
    let mod_type = M_MODIFIER_TYPE[pattern_idx];

    Ok(TonePlacement {
        position: tone_pos,
        modifier_targets: mod_mask,
        modifier_type: mod_type,
    })
}
```
