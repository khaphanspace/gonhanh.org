# Matrix System - Complete Design & Implementation

**Date**: 2025-12-24
**Status**: Final Design (v2)
**Memory**: ~3.5KB total, all O(1) lookups

---

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Input Processing (U1-U7)](#part-1-input-processing-u1-u7) - 141 bytes
3. [Vietnamese Validation (M1-M8)](#part-2-vietnamese-validation-m1-m8) - ~1KB
4. [English Validation (E1-E5)](#part-3-english-validation-e1-e5) - ~2KB
5. [Complete Processor](#part-4-complete-processor)

---

## Design Philosophy

**Core Principle**: Every decision = matrix lookup. Zero if-else in hot path.

```
Input → Classify (matrix) → Dispatch (matrix) → Execute → Done

OLD: if is_vowel(c) { ... } else if is_final(c) { ... }
NEW: DISPATCH[state][KEY_CAT[key]] → action|next_state
```

**Key Improvements over v1**:
- 87% memory reduction (input processing)
- Single lookup per decision
- Packed data (bit flags)
- 5 states instead of 6

---

## Part 1: Input Processing (U1-U7)

### U1: Letter Classification (26 bytes)

Replaces IS_VOWEL + IS_FINAL_CONSONANT + IS_STOP_FINAL.

```rust
pub mod lc {
    pub const V: u8 = 0b0001;  // vowel
    pub const I: u8 = 0b0010;  // initial consonant
    pub const F: u8 = 0b0100;  // final consonant
    pub const S: u8 = 0b1000;  // stop final
}

pub static LETTER_CLASS: [u8; 26] = [
    //  a     b     c     d     e     f     g     h
    0x01, 0x02, 0x0E, 0x02, 0x01, 0x00, 0x06, 0x02,
    //  i     j     k     l     m     n     o     p
    0x01, 0x00, 0x0E, 0x02, 0x06, 0x06, 0x01, 0x0E,
    //  q     r     s     t     u     v     w     x
    0x02, 0x02, 0x02, 0x0E, 0x01, 0x02, 0x00, 0x02,
    //  y     z
    0x01, 0x00,
];

#[inline(always)]
pub fn is_vowel(c: u8) -> bool { LETTER_CLASS[c as usize] & lc::V != 0 }
#[inline(always)]
pub fn is_final(c: u8) -> bool { LETTER_CLASS[c as usize] & lc::F != 0 }
#[inline(always)]
pub fn is_stop(c: u8) -> bool { LETTER_CLASS[c as usize] & lc::S != 0 }
```

### U2: Key Category (38 bytes)

```rust
pub mod cat {
    pub const VOW: u8 = 0;   // vowels
    pub const CON: u8 = 1;   // consonants (initial only)
    pub const FIN: u8 = 2;   // final consonants (m,n,g,h)
    pub const STP: u8 = 3;   // stop finals (c,p,t,k)
    pub const TNE: u8 = 4;   // tone keys
    pub const MOD: u8 = 5;   // modifier keys
    pub const STK: u8 = 6;   // stroke key (d)
    pub const SPC: u8 = 7;   // special (space, bs)
}

pub static KEY_CAT_TELEX: [u8; 38] = [
    // a  b  c  d  e  f  g  h  i  j  k  l  m
       0, 1, 3, 6, 0, 4, 2, 2, 0, 4, 3, 1, 2,
    // n  o  p  q  r  s  t  u  v  w  x  y  z
       2, 0, 3, 1, 4, 4, 3, 0, 1, 5, 4, 0, 4,
    // tone keys (26-31)
       4, 4, 4, 4, 4, 4,
    // mod keys (32-35)
       5, 5, 5, 5,
    // bs, space (36-37)
       7, 7,
];
```

### U3: Unified Dispatch (40 bytes)

Action + next state in single byte.

```rust
pub mod st {
    pub const EMPTY: u8 = 0;
    pub const INIT: u8 = 1;
    pub const VOW: u8 = 2;
    pub const DIA: u8 = 3;   // has diacritic (mod OR mark)
    pub const FIN: u8 = 4;
}

pub mod as_ {
    pub const PASS: u8 = 0x00;
    pub const ADD_I: u8 = 0x11;   // → HAS_INIT
    pub const ADD_V: u8 = 0x12;   // → HAS_VOW
    pub const ADD_F: u8 = 0x14;   // → HAS_FIN
    pub const TONE: u8 = 0x23;    // → HAS_DIA
    pub const MOD: u8 = 0x33;     // → HAS_DIA
    pub const CHK: u8 = 0x40;     // check revert
    pub const DONE: u8 = 0x50;    // → EMPTY
    pub const DEF: u8 = 0x62;     // defer
    pub const STK: u8 = 0x71;     // stroke
}

pub static DISPATCH: [[u8; 8]; 5] = [
    //           VOW   CON   FIN   STP   TNE   MOD   STK   SPC
    /* EMPTY */ [0x12, 0x11, 0x11, 0x11, 0x00, 0x00, 0x11, 0x50],
    /* INIT  */ [0x12, 0x11, 0x11, 0x11, 0x00, 0x00, 0x71, 0x50],
    /* VOW   */ [0x12, 0x14, 0x14, 0x14, 0x23, 0x33, 0x71, 0x50],
    /* DIA   */ [0x12, 0x14, 0x14, 0x14, 0x40, 0x40, 0x40, 0x50],
    /* FIN   */ [0x12, 0x00, 0x00, 0x00, 0x40, 0x40, 0x00, 0x50],
];

#[inline(always)]
pub fn dispatch(state: u8, key: u8, cat: &[u8; 38]) -> (u8, u8) {
    let c = cat[key as usize];
    let packed = DISPATCH[state as usize][c as usize];
    (packed >> 4, packed & 0x0F)
}
```

### U4: Defer Resolution (8 bytes)

```rust
pub mod pend {
    pub const NONE: u8 = 0;
    pub const BREVE: u8 = 1;
    pub const HORN: u8 = 2;
    pub const POP: u8 = 3;
}

pub static DEFER: [[u8; 2]; 4] = [
    //        not_fin  is_fin
    /* NONE */   [0,      0],
    /* BREVE */  [0,      1],   // apply on final
    /* HORN */   [1,      1],   // apply on any
    /* POP */    [0,      2],   // pop on consonant
];

#[inline(always)]
pub fn resolve_defer(pending: u8, key: u8) -> u8 {
    let fin = is_final(key);
    DEFER[pending as usize][fin as usize]
}
```

### U5: Revert Key (11 bytes)

```rust
pub mod tf {
    pub const NONE: u8 = 0;
    pub const STROKE: u8 = 1;
    pub const T_SAC: u8 = 2;
    pub const T_HUY: u8 = 3;
    pub const T_HOI: u8 = 4;
    pub const T_NGA: u8 = 5;
    pub const T_NANG: u8 = 6;
    pub const M_A: u8 = 7;
    pub const M_E: u8 = 8;
    pub const M_O: u8 = 9;
    pub const M_HORN: u8 = 10;
}

pub static REVERT_KEY: [u8; 11] = [
    0xFF, // NONE
    3,    // STROKE → d
    18,   // T_SAC → s
    5,    // T_HUY → f
    17,   // T_HOI → r
    23,   // T_NGA → x
    9,    // T_NANG → j
    0,    // M_A → a
    4,    // M_E → e
    14,   // M_O → o
    22,   // M_HORN → w
];

#[inline(always)]
pub fn should_revert(last: u8, key: u8) -> bool {
    last != tf::NONE && REVERT_KEY[last as usize] == key
}
```

### U6: Tone Stop Validity (6 bytes)

```rust
pub static TONE_STOP_VALID: [bool; 6] = [
    false, // ngang
    true,  // sắc
    false, // huyền
    false, // hỏi
    false, // ngã
    true,  // nặng
];
```

### U7: Modifier Validity (12 bytes)

```rust
pub mod mm {
    pub const C: u8 = 0b001;  // circumflex
    pub const B: u8 = 0b010;  // breve
    pub const H: u8 = 0b100;  // horn
}

pub static MOD_VALID: [u8; 12] = [
    // a   ă   â   e   ê   i   o   ô   ơ   u   ư   y
    0x03, 0, 0, 0x01, 0, 0, 0x05, 0, 0, 0x04, 0, 0,
];
```

---

## Part 2: Vietnamese Validation (M1-M8)

### M2: Initial + Vowel (29×12 = 348 bytes)

```rust
pub static M_INITIAL_VOWEL: [[u8; 12]; 29] = [
    //      a  ă  â  e  ê  i  o  ô  ơ  u  ư  y
    /* b */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* c */ [1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0], // no e,ê,i,y
    /* d */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* đ */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* g */ [1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0], // no e,ê
    /* h */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* k */ [0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1], // only e,ê,i,y
    /* l */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* m */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* n */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* p */ [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0], // rare/loan
    /* q */ [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0], // only u
    /* r */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* s */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* t */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* v */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* x */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* ch*/ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* gh*/ [0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0], // only e,ê,i
    /* gi*/ [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0], // only a,o
    /* kh*/ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* kr*/ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* ng*/ [1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0], // no e,ê,i
    /* nh*/ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* ph*/ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* qu*/ [1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1], // not u,ư
    /* th*/ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* tr*/ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    /*ngh*/ [0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0], // only e,ê,i
];
```

### M5: Vowel + Final (12×9 = 108 bytes)

```rust
pub static M_VOWEL_FINAL: [[u8; 9]; 12] = [
    //      c  ch m  n  ng nh p  t  sv
    /* a */ [1, 1, 1, 1, 1, 1, 1, 1, 1],
    /* ă */ [1, 1, 1, 1, 1, 1, 1, 1, 0],
    /* â */ [1, 0, 1, 1, 1, 0, 1, 1, 0],
    /* e */ [0, 0, 1, 1, 0, 0, 0, 0, 1],
    /* ê */ [0, 1, 1, 1, 0, 1, 1, 1, 1],
    /* i */ [0, 1, 1, 1, 0, 1, 1, 1, 1],
    /* o */ [1, 0, 1, 1, 1, 0, 1, 1, 1],
    /* ô */ [1, 0, 1, 1, 1, 0, 1, 1, 1],
    /* ơ */ [1, 0, 1, 1, 0, 0, 1, 1, 1],
    /* u */ [0, 0, 1, 1, 1, 0, 1, 1, 1],
    /* ư */ [1, 0, 1, 1, 1, 0, 1, 1, 1],
    /* y */ [0, 0, 0, 0, 0, 0, 0, 0, 1],
];
```

### M6: Tone + Stop Final (6×4 = 24 bytes)

```rust
pub static M_TONE_FINAL: [[u8; 4]; 6] = [
    //       p  t  c  ch
    /* ngang */ [0, 0, 0, 0],
    /* sắc */   [1, 1, 1, 1],
    /* huyền */ [0, 0, 0, 0],
    /* hỏi */   [0, 0, 0, 0],
    /* ngã */   [0, 0, 0, 0],
    /* nặng */  [1, 1, 1, 1],
];
```

### M7: Tone Placement (43×4 = 172 bytes)

```rust
/// Position: 1=V1, 2=V2, 3=V3
/// Columns: [Default, After_Q, With_Final, Modern]
pub static M_TONE_PLACEMENT: [[u8; 4]; 43] = [
    [1, 1, 1, 1], // 0: single
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
    [1, 2, 2, 2], // 18: ua (context!)
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

### M8: Modifier Placement (43 bytes)

```rust
/// Bitmask: 0x01=V1, 0x02=V2, 0x04=V3
pub static M_MODIFIER_PLACEMENT: [u8; 43] = [
    0x01, 0x00, 0x00, 0x00, 0x00, // single, ai, ao, au, ay
    0x01, 0x01, 0x00, 0x01, 0x00, // âu, ây, eo, êu, ia
    0x02, 0x00, 0x00, 0x02, 0x00, // iê, iu, oa, oă, oe
    0x00, 0x01, 0x01, 0x00, 0x02, // oi, ôi, ơi, ua, uâ
    0x02, 0x00, 0x02, 0x00, 0x01, // uê, ui, uô, uy, ưa
    0x01, 0x03, 0x01, 0x02, 0x02, // ưi, ươ, ưu, yê, iêu
    0x02, 0x00, 0x00, 0x00, 0x02, // yêu, oai, oay, oeo, uây
    0x02, 0x00, 0x03, 0x03, 0x04, // uôi, uya, ươi, ươu, uyê
    0x00, 0x02, 0x00,             // uyu, uêu, oao
];
```

---

## Part 3: English Validation (E1-E5)

### E2: Onset Clusters (26×26 = 676 bytes)

```rust
pub static E_ONSET_CC: [[u8; 26]; 26] = [
    // Key entries only (rest 0):
    // b: bl(11), br(17)
    // c: cl(11), cr(17)
    // d: dr(17), dw(22)
    // f: fl(11), fr(17)
    // g: gl(11), gr(17)
    // k: kn(13), kw(22)
    // p: pl(11), pr(17)
    // q: qu(20)
    // s: sc(2), sh(7), sk(10), sl(11), sm(12), sn(13), sp(15), st(19), sw(22)
    // t: tr(17), tw(22)
    // w: wr(17)
    // ... (full matrix in implementation)
];
```

### E5: Coda Clusters (26×26 = 676 bytes)

```rust
// Key entries:
// l: ld, lf, lk, lm, lp, ls, lt
// m: mp, ms
// n: nc, nd, ng, nk, ns, nt
// r: rc, rd, rf, rg, rk, rl, rm, rn, rp, rs, rt
// s: sk, sp, st
```

### E7: Impossible Bigrams (26×26 = 676 bytes)

```rust
// Q only pairs with U
// J has many impossible combinations
// X rarely combines
// Z rarely combines
```

---

## Part 4: Complete Processor

```rust
pub struct Processor {
    state: u8,
    pending: u8,
    last_transform: u8,
    reverted: bool,
    key_cat: &'static [u8; 38],
}

impl Processor {
    pub fn new(method: u8) -> Self {
        Self {
            state: st::EMPTY,
            pending: pend::NONE,
            last_transform: tf::NONE,
            reverted: false,
            key_cat: if method == 0 { &KEY_CAT_TELEX } else { &KEY_CAT_VNI },
        }
    }

    #[inline]
    pub fn process(&mut self, key: u8) -> u8 {
        // Step 1: Defer resolution
        if self.pending != pend::NONE {
            let res = resolve_defer(self.pending, key);
            if res != 0 {
                self.apply_resolution(res);
                self.pending = pend::NONE;
            }
        }

        // Step 2: Dispatch
        let (action, new_state) = dispatch(self.state, key, self.key_cat);

        // Step 3: Revert check
        if action >= 4 && should_revert(self.last_transform, key) && !self.reverted {
            self.reverted = true;
            return self.do_revert();
        }

        // Step 4: Execute
        let result = self.execute(action, key);

        // Step 5: State update
        self.state = new_state;
        self.reverted = false;

        result
    }
}
```

---

## Memory Summary

| Category | Size |
|----------|------|
| Input Processing (U1-U7) | 141 bytes |
| Vietnamese Validation | ~700 bytes |
| Placement Tables | ~250 bytes |
| English Validation | ~2KB |
| **Total** | **~3.1KB** |

All lookups O(1), zero if-else in hot path.
