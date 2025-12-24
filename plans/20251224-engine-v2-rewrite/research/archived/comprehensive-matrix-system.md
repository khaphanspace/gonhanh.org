# Comprehensive Matrix-Based Validation System

**Date**: 2025-12-24
**Status**: Research Complete
**Scope**: ALL operations → Matrix rules (Vietnamese + English)

---

## Design Philosophy

**Core Principle**: Every validation, placement, and transformation decision = matrix lookup. Zero case-by-case logic.

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
│  YES: M_TONE_PLACEMENT[vowel_pattern][tone_idx] → position     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Part 1: Vietnamese Matrix System (12 Matrices)

### M1: INITIAL_VALID (28×1) - Valid Initial Consonants

Position constraints for syllable-initial consonants.

```
Index: b=0, c=1, d=2, đ=3, g=4, h=5, k=6, l=7, m=8, n=9,
       p=10, q=11, r=12, s=13, t=14, v=15, x=16,
       ch=17, gh=18, gi=19, kh=20, kr=21, ng=22, nh=23,
       ph=24, qu=25, th=26, tr=27, ngh=28

M1_INITIAL[i] = {
    CAN_START: bool,      // Can start a syllable
    NEEDS_VOWEL: VowelSet // Which vowels can follow
}

Example:
M1_INITIAL[c=1] = { CAN_START: true, NEEDS_VOWEL: {a,ă,â,o,ô,ơ,u,ư} }
M1_INITIAL[k=6] = { CAN_START: true, NEEDS_VOWEL: {e,ê,i,y} }  // k before e,ê,i,y only
M1_INITIAL[gh=18] = { CAN_START: true, NEEDS_VOWEL: {e,ê,i} }  // gh before e,ê,i only
```

### M2: INITIAL_VOWEL (29×12) - Initial + Vowel Spelling Rules

Validates initial consonant + first vowel combinations.

```
Rows: 29 initial consonants (including digraphs/trigraph)
Cols: 12 base vowels (a,ă,â,e,ê,i,o,ô,ơ,u,ư,y)

Values:
  0 = INVALID (spelling error)
  1 = VALID
  2 = DEPRECATED (warn but allow)

       a  ă  â  e  ê  i  o  ô  ơ  u  ư  y
c      1  1  1  0  0  0  1  1  1  1  1  0   // c before e/ê/i/y → 0 (use k)
k      0  0  0  1  1  1  0  0  0  0  0  1   // k before a/o/u → 0 (use c)
g      1  1  1  0  0  1  1  1  1  1  1  0   // g before e → 0 (use gh)
gh     0  0  0  1  1  1  0  0  0  0  0  0   // gh before a/o/u → 0 (use g)
ng     1  1  1  0  0  0  1  1  1  1  1  0   // ng before e/i → 0 (use ngh)
ngh    0  0  0  1  1  1  0  0  0  0  0  0   // ngh before a/o/u → 0 (use ng)
...
```

### M3: VOWEL_PAIR (12×12) - Valid Diphthong Base Keys

Which V1+V2 combinations are valid diphthongs.

```
Rows: V1 (first vowel base key)
Cols: V2 (second vowel base key)

Values:
  0 = INVALID
  1 = VALID (no modifier required)
  2 = REQUIRES_CIRCUMFLEX_V1
  3 = REQUIRES_CIRCUMFLEX_V2
  4 = REQUIRES_HORN_V1
  5 = REQUIRES_HORN_BOTH
  6 = CONTEXT_DEPENDENT (ua pattern)

       a  ă  â  e  ê  i  o  ô  ơ  u  ư  y
a      0  0  0  0  0  1  1  0  0  1  0  1   // ai,ao,au,ay
ă      0  0  0  0  0  0  0  0  0  0  0  0   // ă never V1
â      0  0  0  0  0  0  0  0  0  1  0  1   // âu,ây (base a+u/y with ^)
e      0  0  0  0  0  0  1  0  0  2  0  0   // eo, e+u=êu (V1 needs ^)
ê      0  0  0  0  0  0  0  0  0  1  0  0   // êu
i      1  0  0  3  0  0  0  0  0  1  0  0   // ia, i+e=iê (V2 needs ^), iu
o      1  1  0  1  0  1  0  0  0  0  0  0   // oa,oă,oe,oi
ô      0  0  0  0  0  1  0  0  0  0  0  0   // ôi
ơ      0  0  0  0  0  1  0  0  0  0  0  0   // ơi
u      6  0  1  3  0  1  1  0  0  0  0  1   // ua(context),uâ,u+e=uê,ui,uô,uy
ư      4  0  0  0  0  4  0  0  5  4  0  0   // ưa,ưi,ươ(both horn),ưu
y      0  0  0  3  0  0  0  0  0  0  0  0   // y+e=yê (V2 needs ^)
```

### M4: VOWEL_TRIPLE (6×12) - Valid Triphthong Extensions

Which V1V2 + V3 combinations are valid triphthongs.

```
Rows: V1V2 patterns (iê,yê,oa,oe,uâ,uô,ươ,uy)
Cols: V3 options

       a  ă  â  e  ê  i  o  ô  ơ  u  ư  y
iê     0  0  0  0  0  0  0  0  0  1  0  0   // iêu
yê     0  0  0  0  0  0  0  0  0  1  0  0   // yêu
oa     0  0  0  0  0  1  1  0  0  0  0  1   // oai,oao,oay
oe     0  0  0  0  0  0  1  0  0  0  0  0   // oeo
uâ     0  0  0  0  0  0  0  0  0  0  0  1   // uây
uê     0  0  0  0  0  0  0  0  0  1  0  0   // uêu
uô     0  0  0  0  0  1  0  0  0  0  0  0   // uôi
ươ     0  0  0  0  0  1  0  0  0  1  0  0   // ươi,ươu
uy     1  0  0  3  0  0  0  0  0  1  0  0   // uya,uy+e=uyê,uyu
```

### M5: VOWEL_FINAL (12×9) - Vowel + Final Consonant Compatibility

```
Rows: Vowel nuclei (last vowel or modified vowel)
Cols: Final consonants (c,ch,m,n,ng,nh,p,t,semivowel)

       c  ch m  n  ng nh p  t  sv
a      1  1  1  1  1  1  1  1  1
ă      1  1  1  1  1  1  1  1  0   // ă has no semivowel ending
â      1  0  1  1  1  0  1  1  0   // âch,ânh invalid
e      0  0  1  1  0  0  0  0  1   // e+ng→invalid (use nh)
ê      0  1  1  1  0  1  1  1  1   // ê+c,ê+ng invalid
i      0  1  1  1  0  1  1  1  1   // i+c,i+ng invalid (use ch/nh)
o      1  0  1  1  1  0  1  1  1
ô      1  0  1  1  1  0  1  1  1
ơ      1  0  1  1  0  0  1  1  1   // ơ+ng,ơ+nh invalid
u      0  0  1  1  1  0  1  1  1   // u+c,u+ch invalid
ư      1  0  1  1  1  0  1  1  1   // ư+ch invalid
y      0  0  0  0  0  0  0  0  1   // y only as semivowel or special
```

### M6: TONE_FINAL (6×4) - Tone + Stop Final Compatibility

Rule 7: Stop finals only allow sắc/nặng.

```
Rows: 6 tones (ngang,sắc,huyền,hỏi,ngã,nặng)
Cols: 4 stop finals (p,t,c,ch)

           p  t  c  ch
ngang      0  0  0  0    // INVALID with stop
sắc        1  1  1  1    // VALID
huyền      0  0  0  0    // INVALID with stop
hỏi        0  0  0  0    // INVALID with stop
ngã        0  0  0  0    // INVALID with stop
nặng       1  1  1  1    // VALID
```

### M7: TONE_PLACEMENT (43×3) - Which Vowel Gets Tone

Based on 7.6.1 matrix - exact vowel position for each pattern.

```
43 vowel patterns × 3 context flags (has_final, after_q, modern_mode)

Pattern | Default | After_Q | With_Final | Modern
--------|---------|---------|------------|--------
ai      | 1       | 1       | 1          | 1      // always V1
ao      | 1       | 1       | 1          | 1      // always V1
ua      | 1       | 2       | 2          | 2      // V1 open, V2 after q/with final
oa      | 2       | 2       | 2          | 2      // always V2 (modern: still V2)
oe      | 2       | 2       | 2          | 2      // always V2
iê      | 2       | 2       | 2          | 2      // always V2 (ê)
ươ      | 2       | 2       | 2          | 2      // always V2 (ơ)
oai     | 2       | 2       | 2          | 2      // middle (a)
iêu     | 2       | 2       | 2          | 2      // middle (ê)
uyê     | 3       | 3       | 3          | 3      // last (ê)
...
```

### M8: MODIFIER_PLACEMENT (43×4) - Which Vowel(s) Get Modifier

Horn/circumflex/breve placement for each pattern.

```
Values: bitmask indicating which vowels get modified
  0x01 = V1 gets modifier
  0x02 = V2 gets modifier
  0x04 = V3 gets modifier
  0x03 = V1+V2 get modifier (ươ pattern)

Pattern | Modifier | Target | Notes
--------|----------|--------|-------
ai      | -        | 0x00   | no modifier
âu      | ^        | 0x01   | V1 (a→â)
êu      | ^        | 0x01   | V1 (e→ê)
iê      | ^        | 0x02   | V2 (e→ê)
oă      | ˘        | 0x02   | V2 (a→ă)
ưa      | ʼ        | 0x01   | V1 (u→ư)
ươ      | ʼ        | 0x03   | V1+V2 (u→ư, o→ơ)
ưu      | ʼ        | 0x01   | V1 only (u₁→ư)
ươi     | ʼ        | 0x03   | V1+V2 (u→ư, o→ơ)
ươu     | ʼ        | 0x03   | V1+V2 only (u₃ stays)
uyê     | ^        | 0x04   | V3 (e→ê)
```

### M9: SPECIAL_UO (4×2) - Special ươ/uơ Pattern Resolution

Handles Issue #133 (huơ vs ươ).

```
Context      | Both_Horn | Single_Horn
-------------|-----------|------------
standard     | 1         | 0          // ươ: both get horn
huơ_pattern  | 0         | 1          // uơ: only o gets horn
khuơ_pattern | 0         | 1          // uơ: only o gets horn
default      | 1         | 0          // default: both
```

### M10: POSITION_START (26×1) - What Can Start a Word

```
Letter | Can_Start | Notes
-------|-----------|-------
a      | 1         | Vowel start OK
b      | 1         | Standard initial
c      | 1         | Standard initial
d      | 1         | Standard initial
đ      | 1         | Standard initial
f      | 0         | Not Vietnamese (loan only)
g      | 1         | Standard initial
h      | 1         | Standard initial
j      | 0         | Not Vietnamese (modifier key)
k      | 1         | Standard initial
...
p      | 2         | Rare (loan words only)
q      | 1         | Must be followed by u
w      | 0         | Not Vietnamese (modifier key)
x      | 1         | Standard initial
z      | 0         | Not Vietnamese (modifier key)
```

### M11: POSITION_END (26×1) - What Can End a Word

```
Letter | Can_End | Notes
-------|---------|-------
a      | 1       | Semivowel ending
c      | 1       | Stop final
h      | 1       | Only after c/n (ch/nh)
i      | 1       | Semivowel ending
m      | 1       | Nasal final
n      | 1       | Nasal final
o      | 1       | Semivowel ending
p      | 1       | Stop final
t      | 1       | Stop final
u      | 1       | Semivowel ending
y      | 1       | Semivowel ending
g      | 1       | Only after n (ng)
...others | 0    | Cannot end syllable
```

### M12: FINAL_DIGRAPH (26×26) - Valid Final Consonant Pairs

```
Only valid: ch, ng, nh

Rows: First letter
Cols: Second letter

     a  b  c  ...  g  h  ...  n  ...
c    0  0  0  ...  0  1  ...  0  ...  // ch valid
n    0  0  0  ...  1  1  ...  0  ...  // ng, nh valid
...rest: all 0
```

---

## Part 2: English Matrix System (8 Matrices)

### E1: ONSET_SINGLE (26×1) - Single Letter Onsets

```
All letters except: h(sometimes), q(needs u), x(rarely)
```

### E2: ONSET_DOUBLE (26×26) - Valid Onset Clusters

```
Based on sonority sequencing principle:
- Obstruent + Liquid: bl, br, cl, cr, dr, fl, fr, gl, gr, pl, pr, tr
- Obstruent + Glide: dw, gw, kw, sw, tw
- s + Consonant: sc, sk, sl, sm, sn, sp, st, sw

       a  b  c  ...  l  ...  r  ...  w
b      0  0  0  ...  1  ...  1  ...  0   // bl, br
c      0  0  0  ...  1  ...  1  ...  0   // cl, cr
d      0  0  0  ...  0  ...  1  ...  1   // dr, dw
f      0  0  0  ...  1  ...  1  ...  0   // fl, fr
g      0  0  0  ...  1  ...  1  ...  1   // gl, gr, (gw rare)
k      0  0  0  ...  0  ...  0  ...  1   // kw (qu)
p      0  0  0  ...  1  ...  1  ...  0   // pl, pr
s      0  0  1  ...  1  ...  0  ...  1   // sc,sk,sl,sm,sn,sp,st,sw
t      0  0  0  ...  0  ...  1  ...  1   // tr, tw
...
```

### E3: ONSET_TRIPLE (26×26×26) - Valid Triple Onsets

Only s-clusters: scr, spl, spr, str, squ

Sparse matrix - only ~5 valid entries.

### E4: CODA_SINGLE (26×1) - Single Letter Codas

```
Valid: b,c,d,f,g,k,l,m,n,p,r,s,t,v,x,z
Invalid: a,e,h(mostly),i,j,o,q,u,w,y
```

### E5: CODA_DOUBLE (26×26) - Valid Coda Clusters

```
Voicing agreement + sonority:
- ct, ft, ld, lf, lk, lm, ln, lp, lt, lth
- mp, nd, ng, nk, nt, nch, nge
- pt, rb, rc, rd, rf, rg, rk, rl, rm, rn, rp, rs, rt, rv
- sk, sp, st, sk

       b  c  d  ...  k  ...  s  t  ...
c      0  0  0  ...  0  ...  0  1  ...  // ct
f      0  0  0  ...  0  ...  0  1  ...  // ft
l      0  0  1  ...  1  ...  0  1  ...  // ld,lk,lt
m      0  0  0  ...  0  ...  0  0  ...  // mp(p row)
n      0  0  1  ...  1  ...  0  1  ...  // nd,nk,nt
...
```

### E6: CODA_TRIPLE (26×26×26) - Valid Triple Codas

Examples: nds, ngs, nks, nts, rks, sts

Sparse matrix.

### E7: IMPOSSIBLE_BIGRAM (26×26) - Never-Occurring Letter Pairs

```
Position-independent impossible combinations:
- bx, cj, dx, fq, gx, hx, jq, kx, qb, qc, qd...
- Doubled consonants (mostly invalid): aa, bb, etc.

       a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z
a      0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  0  0  0  0  0  0  0  0  0
b      0  1  0  0  0  0  0  0  0  1  0  0  0  0  0  0  1  0  0  0  0  0  0  1  0  1
c      0  0  0  0  0  0  0  0  0  1  0  0  0  0  0  0  1  0  0  0  0  0  0  0  0  0
...
q      1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  1  0  1  1  1  1  1  // q only before u
...
```

### E8: VOWEL_DIGRAPH (5×5) - Valid Vowel Pairs

```
       a  e  i  o  u
a      0  1  1  0  1   // ae,ai,au (no aa except loan)
e      1  1  1  0  1   // ea,ee,ei,eu
i      0  1  0  0  0   // ie
o      1  1  1  1  1   // oa,oe,oi,oo,ou
u      0  1  1  0  0   // ue,ui
```

---

## Part 3: Position-Aware Constraints

### P1: WORD_START (Lang×26) - What Can Start Words

```
Language | Letter | Valid | Notes
---------|--------|-------|-------
VN       | đ      | 1     | Vietnamese specific
VN       | p      | 2     | Rare (loan words)
VN       | f,j,w,z| 0     | Not Vietnamese
EN       | x      | 2     | Rare (xylophone)
EN       | all    | 1     | Most letters OK
```

### P2: WORD_END (Lang×26) - What Can End Words

```
Language | Letter | Valid | Notes
---------|--------|-------|-------
VN       | c,ch   | 1     | Stop final
VN       | m,n,ng,nh | 1  | Nasal final
VN       | p,t    | 1     | Stop final
VN       | i,y,o,u| 1     | Semivowel
VN       | others | 0     | Cannot end
EN       | most   | 1     | Flexible
EN       | q      | 0     | Never ends
```

### P3: AFTER_Q (Lang×26) - What Can Follow Q

```
VN: ONLY 'u' → qu + vowel (mandatory)
EN: ONLY 'u' → qu + vowel (nearly always)
```

---

## Part 4: Transformation Matrices

### T1: TONE_TRANSFORM (12×6) - Apply Tone to Vowel

```
Rows: Base vowels (a,ă,â,e,ê,i,o,ô,ơ,u,ư,y)
Cols: Tones (ngang,sắc,huyền,hỏi,ngã,nặng)
Values: Unicode codepoint of result

         ngang  sắc   huyền  hỏi   ngã   nặng
a        0x61   0xe1  0xe0   0x1ea3 0xe3  0x1ea1
ă        0x103  0x1eaf 0x1eb1 0x1eb3 0x1eb5 0x1eb7
â        0xe2   0x1ea5 0x1ea7 0x1ea9 0x1eab 0x1ead
e        0x65   0xe9  0xe8   0x1ebb 0x1ebd 0x1eb9
...
```

### T2: MODIFIER_TRANSFORM (12×4) - Apply Modifier to Vowel

```
Rows: Base vowels
Cols: Modifiers (none, circumflex, breve, horn)
Values: Unicode codepoint of result

       none  ^      ˘      ʼ
a      0x61  0xe2   0x103  -
e      0x65  0xea   -      -
o      0x6f  0xf4   -      0x1a1
u      0x75  -      -      0x1b0
```

---

## Part 5: Edge Case Matrices

### X1: DOUBLE_KEY_REVERT - Revert Patterns

```
Input      | Revert_To | Preserve
-----------|-----------|----------
ss (sắc)   | s + key   | remove tone, keep s
ff (huyền) | f + key   | remove tone, keep f
dd (đ)     | d + key   | remove stroke, keep d
aa (â)     | a + key   | remove ^, keep a
ww (horn)  | w + key   | remove horn, keep w
```

### X2: DEFERRED_MODIFIER - Pending Transformation

```
Pattern     | Defer_Until    | Action
------------|----------------|--------
"uoc" + 'w' | consonant seen | Apply horn to uo→ươ
"ao" + '^'  | next input     | Resolve: âo or aô based on pattern
```

### X3: CONTEXT_UA - ua Pattern Resolution

```
Context       | Tone_Position | Example
--------------|---------------|--------
Open syllable | V1 (u)        | mùa, múa
After q       | V2 (a)        | quà, quá
With final    | V2 (a)        | thuận, chuẩn
```

---

## Implementation Strategy

### Data Structures

```rust
/// Compact matrix storage - bitpacked for cache efficiency
pub struct PhonMatrix<const R: usize, const C: usize> {
    data: [[u8; C]; R],  // Or u16/u32 for larger values
}

impl<const R: usize, const C: usize> PhonMatrix<R, C> {
    #[inline(always)]
    pub const fn get(&self, row: usize, col: usize) -> u8 {
        self.data[row][col]
    }

    #[inline(always)]
    pub const fn is_valid(&self, row: usize, col: usize) -> bool {
        self.data[row][col] != 0
    }
}

/// Pre-computed static matrices
pub static M_INITIAL_VOWEL: PhonMatrix<29, 12> = PhonMatrix { data: [...] };
pub static M_VOWEL_PAIR: PhonMatrix<12, 12> = PhonMatrix { data: [...] };
pub static M_VOWEL_FINAL: PhonMatrix<12, 9> = PhonMatrix { data: [...] };
pub static M_TONE_FINAL: PhonMatrix<6, 4> = PhonMatrix { data: [...] };
pub static M_TONE_PLACEMENT: PhonMatrix<43, 4> = PhonMatrix { data: [...] };
pub static M_MODIFIER_PLACEMENT: PhonMatrix<43, 1> = PhonMatrix { data: [...] };
```

### Validation Flow (All Matrix)

```rust
pub fn validate_vietnamese(syllable: &Syllable) -> ValidationResult {
    // Step 1: Initial + Vowel spelling check
    if let Some(initial) = syllable.initial {
        let init_idx = initial_to_idx(initial);
        let vowel_idx = vowel_to_idx(syllable.first_vowel());
        if M_INITIAL_VOWEL.get(init_idx, vowel_idx) == 0 {
            return ValidationResult::InvalidSpelling;
        }
    }

    // Step 2: Vowel pattern check
    let pattern_idx = match syllable.vowel_count() {
        1 => 0, // Single vowel always valid
        2 => {
            let v1 = vowel_to_idx(syllable.vowels[0]);
            let v2 = vowel_to_idx(syllable.vowels[1]);
            let result = M_VOWEL_PAIR.get(v1, v2);
            if result == 0 { return ValidationResult::InvalidVowelPattern; }
            diphthong_to_pattern_idx(v1, v2)
        }
        3 => {
            // Check triphthong validity via M4
            let v12_idx = diphthong_to_idx(syllable.vowels[0], syllable.vowels[1]);
            let v3_idx = vowel_to_idx(syllable.vowels[2]);
            if M_VOWEL_TRIPLE.get(v12_idx, v3_idx) == 0 {
                return ValidationResult::InvalidTriphthong;
            }
            triphthong_to_pattern_idx(syllable.vowels)
        }
        _ => return ValidationResult::TooManyVowels,
    };

    // Step 3: Vowel + Final compatibility
    if let Some(final_c) = syllable.final_consonant {
        let vowel_idx = vowel_to_idx(syllable.main_vowel());
        let final_idx = final_to_idx(final_c);
        if M_VOWEL_FINAL.get(vowel_idx, final_idx) == 0 {
            return ValidationResult::InvalidVowelFinal;
        }
    }

    // Step 4: Tone + Stop Final (Rule 7)
    if is_stop_final(syllable.final_consonant) {
        let tone_idx = tone_to_idx(syllable.tone);
        let final_idx = stop_final_to_idx(syllable.final_consonant);
        if M_TONE_FINAL.get(tone_idx, final_idx) == 0 {
            return ValidationResult::InvalidToneStopFinal;
        }
    }

    // Step 5: Modifier requirements
    let required_mod = M_MODIFIER_REQUIRED.get(pattern_idx, 0);
    if required_mod != 0 && !syllable.has_modifier(required_mod) {
        return ValidationResult::MissingRequiredModifier;
    }

    ValidationResult::Valid(pattern_idx)
}
```

### Tone Placement (Matrix Lookup)

```rust
pub fn get_tone_position(pattern_idx: usize, context: &ToneContext) -> usize {
    let col = match context {
        ToneContext::Default => 0,
        ToneContext::AfterQ => 1,
        ToneContext::WithFinal => 2,
        ToneContext::Modern => 3,
    };
    M_TONE_PLACEMENT.get(pattern_idx, col) as usize
}
```

### Modifier Placement (Matrix Lookup)

```rust
pub fn get_modifier_targets(pattern_idx: usize) -> ModifierTargets {
    let mask = M_MODIFIER_PLACEMENT.get(pattern_idx, 0);
    ModifierTargets {
        v1: (mask & 0x01) != 0,
        v2: (mask & 0x02) != 0,
        v3: (mask & 0x04) != 0,
    }
}
```

---

## Performance Analysis

| Matrix | Size | Memory | Lookup |
|--------|------|--------|--------|
| M_INITIAL_VOWEL | 29×12 | 348 bytes | O(1) |
| M_VOWEL_PAIR | 12×12 | 144 bytes | O(1) |
| M_VOWEL_TRIPLE | 8×12 | 96 bytes | O(1) |
| M_VOWEL_FINAL | 12×9 | 108 bytes | O(1) |
| M_TONE_FINAL | 6×4 | 24 bytes | O(1) |
| M_TONE_PLACEMENT | 43×4 | 172 bytes | O(1) |
| M_MODIFIER_PLACEMENT | 43×1 | 43 bytes | O(1) |
| M_TONE_TRANSFORM | 12×6 | 288 bytes (u32) | O(1) |
| **Total Vietnamese** | - | **~1.2KB** | O(1) |
| E_ONSET_CC | 26×26 | 676 bytes | O(1) |
| E_CODA_CC | 26×26 | 676 bytes | O(1) |
| E_IMPOSSIBLE | 26×26 | 676 bytes | O(1) |
| **Total English** | - | **~2KB** | O(1) |
| **Grand Total** | - | **~3.5KB** | O(1) |

All lookups are O(1) constant time with cache-friendly access patterns.

---

## Unresolved Questions

1. **M_SPECIAL_UO Matrix**: Need to identify ALL words that use uơ (single horn) vs ươ (both horn). Currently: huơ, khuơ. Are there more?

2. **Loan Word Handling**: Should loan words (pizza, video) validate against Vietnamese matrices or bypass? Current assumption: bypass.

3. **Old vs New Tone Style**: M_TONE_PLACEMENT has modern column. Need to verify all 43 patterns have correct old/new positions.

4. **Triple Consonant Onsets in English**: scr, spl, spr, str, squ - are there others? Need linguistic verification.

---

---

## Part 6: Input Processing Matrices (Giải quyết Random Order Typing)

**Core Problem**: User có thể gõ dấu/modifier theo bất kỳ thứ tự nào:
- "toan" + 's' → "toán" (tone after word)
- "ts" + "oan" → "toán" (tone first, vowels after)
- "oans" + 't' → "toán" (initial consonant last)
- "aows" → "áo" (mark before tone)

**Solution**: Matrix-based state machine thay vì if-else logic.

### I1: ACTION_DISPATCH (5 States × 64 KeyTypes) → 8 Actions

Dispatch table cho mọi input scenario:

```
States (5):
  0 = EMPTY        (buffer empty)
  1 = HAS_INITIAL  (has initial consonant only)
  2 = HAS_VOWEL    (has at least one vowel)
  3 = HAS_MODIFIER (has modifier applied)
  4 = HAS_TONE     (has tone applied)
  5 = HAS_FINAL    (has final consonant)

KeyTypes (64):
  0-25  = Letters a-z
  26-31 = Tone keys (Telex: s,f,r,x,j,z / VNI: 1-5,0)
  32-35 = Modifier keys (Telex: w,a,e,o / VNI: 6,7,8,9)
  36    = Backspace
  37    = Space
  38-63 = Reserved

Actions (8):
  0 = PASS_THROUGH    → Key không xử lý, gửi thẳng
  1 = ADD_LETTER      → Thêm letter vào buffer
  2 = APPLY_TONE      → Áp dụng tone ngay
  3 = DEFER_TONE      → Lưu tone, chờ context
  4 = APPLY_MODIFIER  → Áp dụng modifier ngay
  5 = DEFER_MODIFIER  → Lưu modifier, chờ validation
  6 = REVERT          → Check revert pattern (double-key)
  7 = COMPLETE_WORD   → Word boundary (space/punctuation)
```

**Matrix Data**:
```
I1_ACTION[state][key_type] = action

         a  b  c  d  e ... w  s  f  r  x  j ...
EMPTY    1  1  1  1  1 ... 1  0  0  0  0  0 ...  // Letters pass, tones ignored
HAS_INIT 1  1  1  1  1 ... 1  0  0  0  0  0 ...  // Need vowel first
HAS_VOW  1  1  1  1  1 ... 4  2  2  2  2  2 ...  // Tones/mods apply
HAS_MOD  1  1  1  1  1 ... 6  2  2  2  2  2 ...  // Tone still OK, mod revert check
HAS_TONE 1  1  1  1  1 ... 4  6  6  6  6  6 ...  // Mod OK, tone revert check
HAS_FIN  1  0  0  0  1 ... 0  6  6  6  6  6 ...  // Final limits letters
```

### I2: DEFERRED_RESOLUTION (3 DeferTypes × 64 TriggerKeys) → 4 Results

Resolve pending transformations khi context đủ:

```
DeferTypes (3):
  0 = PENDING_BREVE      (aw pattern, chờ final)
  1 = PENDING_U_HORN     (uơ pattern, chờ final/vowel)
  2 = PENDING_MARK_POP   (mark revert, chờ letter type)

TriggerKeys (64):
  Same as I1

Results (4):
  0 = KEEP_PENDING   → Chưa đủ context, giữ pending
  1 = APPLY_NOW      → Context đủ, apply transformation
  2 = CANCEL         → Context invalid, cancel pending
  3 = POP_RAW        → Remove from raw_input (mark revert case)
```

**Matrix Data**:
```
I2_DEFERRED[defer_type][trigger_key] = result

               a  b  c  d  e ... m  n  p  t ... vowels
PENDING_BREVE  2  0  1  0  2 ... 1  1  1  1 ... 2
               ^cancel       ... ^apply        ... ^cancel (invalid)

PENDING_U_HRN  0  0  1  0  0 ... 1  1  1  1 ... 1
               ^keep        ... ^apply        ... ^apply

PENDING_POP    0  3  3  3  0 ... 3  3  3  3 ... 0
               ^don't ... ^pop (consonant)   ... ^don't (vowel)
```

**Logic giải thích**:
- PENDING_BREVE: "aw" pattern
  - Final consonant (c,k,m,n,p,t) → APPLY (breve valid: trắm, tắc)
  - Vowel → CANCEL (breve invalid in open syllable: *awo)
  - Other consonant → KEEP_PENDING (might be initial of next syllable)

- PENDING_U_HORN: "uơ" pattern
  - Final consonant/vowel → APPLY (horn on 'u' valid: dược, thuở)
  - Keep waiting otherwise

- PENDING_MARK_POP: Mark revert (tesst → test vs issue)
  - Consonant → POP (remove consumed modifier from raw)
  - Vowel → KEEP (don't pop, user typing English)

### I3: REVERT_LOOKUP (6 TransformTypes × 64 Keys) → 3 RevertActions

Double-key revert patterns:

```
TransformTypes (6):
  0 = STROKE_D      (đ from d+d)
  1 = TONE_SAC      (sắc from s/1)
  2 = TONE_HUYEN    (huyền from f/2)
  3 = TONE_HOI      (hỏi from r/3)
  4 = TONE_NGA      (ngã from x/4)
  5 = TONE_NANG     (nặng from j/5)
  6 = MOD_CIRCUM    (circumflex from a/e/o)
  7 = MOD_BREVE     (breve from w)
  8 = MOD_HORN      (horn from w)

RevertActions (3):
  0 = NO_REVERT     → Different key, no revert
  1 = REVERT_TRANSFORM → Same key, revert transformation
  2 = REVERT_LOCK   → Already reverted, lock (prevent oscillation)
```

**Matrix Data**:
```
I3_REVERT[transform_type][key] = action

            a  b  c  d  e  f ... s  w  x ...
STROKE_D    0  0  0  1  0  0 ... 0  0  0 ...  // d+d = revert
TONE_SAC    0  0  0  0  0  0 ... 1  0  0 ...  // s+s = revert (Telex)
MOD_CIRCUM  1  0  0  0  1  0 ... 0  0  0 ...  // a+a or e+e = revert
MOD_HORN    0  0  0  0  0  0 ... 0  1  0 ...  // w+w = revert
```

**Oscillation Prevention**:
```rust
// State tracks if transform was reverted
enum TransformState {
    None,
    Applied(TransformType),   // Transform active
    Reverted(TransformType),  // Was reverted, locked
}

// After revert: I3_REVERT returns REVERT_LOCK for subsequent same keys
```

### I4: STATE_TRANSITION (5 States × 8 Actions × 4 Results) → New State

State machine transition table:

```
I4_TRANSITION[current_state][action][result] = new_state

Example entries:
[EMPTY, ADD_LETTER, SUCCESS]      → HAS_INITIAL or HAS_VOWEL (depends on letter type)
[HAS_VOWEL, APPLY_TONE, SUCCESS]  → HAS_TONE
[HAS_TONE, APPLY_MODIFIER, FAIL]  → HAS_TONE (modifier failed, state unchanged)
[HAS_MOD, REVERT, SUCCESS]        → HAS_VOWEL (modifier removed)
```

**Optimized Form** (eliminate 3D matrix):
```rust
fn next_state(state: State, action: Action, result: bool) -> State {
    // Encode as 2D matrix with result as bit
    let action_result = (action as u8) << 1 | (result as u8);
    I4_TRANSITION_FLAT[state as usize][action_result as usize]
}
```

### I5: MODIFIER_COMPAT (4 ModTypes × 12 Vowels × 2 HasTone) → Validity

Check if modifier can apply given current vowel state:

```
ModTypes (4):
  0 = CIRCUMFLEX
  1 = BREVE
  2 = HORN
  3 = NONE

         a  ă  â  e  ê  i  o  ô  ơ  u  ư  y
CIRCUM   1  0  0  1  0  0  1  0  0  0  0  0  // a→â, e→ê, o→ô
BREVE    1  0  0  0  0  0  0  0  0  0  0  0  // a→ă only
HORN     0  0  0  0  0  0  1  0  0  1  0  0  // o→ơ, u→ư

HasTone=1: Same pattern (modifier doesn't affect tone compatibility)
```

### I6: TONE_COMPAT (6 Tones × 43 Patterns × 2 HasMod) → Validity

Check if tone can apply given current pattern:

```
Rule 7: Stop finals (p,t,c,ch) only allow sắc/nặng

Patterns 0-42 × Tones 0-5:
- For open patterns (no stop final): All tones valid (1)
- For stop-final patterns: Only sắc(1) and nặng(5) valid

Pattern indices for stop-finals:
  Pattern 35-42: Patterns ending with p/t/c/ch

         ngang sắc  huyền hỏi  ngã  nặng
PAT_35   0     1    0     0    0    1     // Stop final - only sắc/nặng
PAT_36   0     1    0     0    0    1
...
PAT_0    1     1    1     1    1    1     // Open - all valid
```

---

## Part 7: Complete Input Processing Flow (All Matrix)

```rust
pub fn process_key(&mut self, key: u16) -> Result {
    // Step 1: Classify input key
    let key_type = classify_key(key, self.method);

    // Step 2: Check deferred resolutions first
    if let Some(pending) = self.pending_state {
        let resolution = I2_DEFERRED.get(pending.type_idx(), key_type);
        match resolution {
            APPLY_NOW => self.apply_pending(pending),
            CANCEL => self.cancel_pending(),
            POP_RAW => self.pop_raw_input(),
            KEEP_PENDING => {} // Continue with normal processing
        }
    }

    // Step 3: Dispatch action based on state
    let action = I1_ACTION.get(self.state as usize, key_type);

    // Step 4: Check revert if applicable
    if action == REVERT || matches!(action, APPLY_TONE | APPLY_MODIFIER) {
        if let Some(last) = self.last_transform {
            let revert = I3_REVERT.get(last.type_idx(), key_type);
            if revert == REVERT_TRANSFORM {
                return self.do_revert(last);
            } else if revert == REVERT_LOCK {
                return self.pass_through(key);
            }
        }
    }

    // Step 5: Execute action
    let result = match action {
        PASS_THROUGH => return self.pass_through(key),
        ADD_LETTER => self.add_letter(key),
        APPLY_TONE => {
            // Check compatibility first
            let tone_idx = key_to_tone(key_type);
            let pattern_idx = self.current_pattern_idx();
            if I6_TONE_COMPAT.get(tone_idx, pattern_idx, self.has_modifier()) {
                self.apply_tone(tone_idx)
            } else {
                self.pass_through(key)
            }
        }
        DEFER_TONE => self.defer_tone(key_type),
        APPLY_MODIFIER => {
            let mod_idx = key_to_modifier(key_type);
            let vowel_idx = self.target_vowel_idx();
            if I5_MODIFIER_COMPAT.get(mod_idx, vowel_idx, self.has_tone()) {
                self.apply_modifier(mod_idx)
            } else {
                self.defer_modifier(mod_idx)
            }
        }
        DEFER_MODIFIER => self.defer_modifier(key_type),
        COMPLETE_WORD => self.complete_word(),
        _ => unreachable!(),
    };

    // Step 6: State transition
    self.state = I4_TRANSITION.get(self.state, action, result.success);

    result
}
```

---

## Part 8: Random Order Typing Examples (Matrix Walkthrough)

### Example 1: "toán" typed as "oans" + "t"

```
Step 1: 'o' → I1[EMPTY, o] = ADD_LETTER → state = HAS_VOWEL
Step 2: 'a' → I1[HAS_VOWEL, a] = ADD_LETTER → state = HAS_VOWEL (pattern: oa)
Step 3: 'n' → I1[HAS_VOWEL, n] = ADD_LETTER → state = HAS_FINAL
Step 4: 's' → I1[HAS_FINAL, s] = APPLY_TONE
        → I6[SAC, oa_pattern, no_mod] = VALID
        → Tone position: M7[oa_pattern, WITH_FINAL] = V2 (á)
        → state = HAS_TONE, result = "oán"
Step 5: 't' → I1[HAS_TONE, t] = ADD_LETTER (initial consonant at start)
        → Rebuild buffer: t + oán = "toán"
```

### Example 2: "trắm" typed as "trawm" (deferred breve)

```
Step 1: 't' → ADD_LETTER → state = HAS_INITIAL
Step 2: 'r' → ADD_LETTER → state = HAS_INITIAL
Step 3: 'a' → ADD_LETTER → state = HAS_VOWEL
Step 4: 'w' → I1[HAS_VOWEL, w] = APPLY_MODIFIER
        → Check: pattern = single 'a', no final yet
        → I5[BREVE, a, no_tone] = VALID
        → BUT: open syllable → DEFER_MODIFIER
        → pending = PENDING_BREVE(pos=2), state = HAS_VOWEL
Step 5: 'm' → I2[PENDING_BREVE, m] = APPLY_NOW (valid final)
        → Apply breve at pos 2: 'a' → 'ă'
        → Add 'm' as final
        → state = HAS_FINAL, result = "trắm"
```

### Example 3: "awwo" - Cancel pending breve

```
Step 1: 'a' → ADD_LETTER → state = HAS_VOWEL
Step 2: 'w' → DEFER_MODIFIER → pending = PENDING_BREVE(pos=0)
Step 3: 'w' → I3[MOD_BREVE, w] = REVERT_TRANSFORM
        → Cancel pending, consume key
        → pending = None, state = HAS_VOWEL
Step 4: 'o' → I2[no_pending, o] = N/A
        → I1[HAS_VOWEL, o] = ADD_LETTER
        → result = "awo" (user doesn't want Vietnamese)
```

### Example 4: "tesst" → "test" (mark revert with consonant)

```
Step 1-4: 't','e','s','s'
          ...
Step 4b: Second 's' → I3[TONE_SAC, s] = REVERT_TRANSFORM
         → Remove tone from 'e', add 's'
         → pending = PENDING_MARK_POP, result = "tess"
Step 5: 't' → I2[PENDING_MARK_POP, t] = POP_RAW (t is consonant)
        → Pop consumed 's' from raw_input
        → result = "test"
```

### Example 5: "issue" - No pop (vowel after)

```
Step 1-4: Same as above, pending = PENDING_MARK_POP, buffer = "issu"
Step 5: 'e' → I2[PENDING_MARK_POP, e] = KEEP (e is vowel)
        → Don't pop, keep raw_input intact
        → result = "issue"
```

---

## Part 9: Complete Word Example - "đau" (5 Typing Variations)

Minh họa cách matrix system xử lý CÙNG MỘT TỪ với các thứ tự gõ khác nhau.

### Case 1: "ddau" → "đau" (Standard double-d stroke)

```
Input sequence: d, d, a, u

Step 1: 'd'
        → I1[EMPTY, d] = ADD_LETTER
        → state = HAS_INITIAL, buf = "d"

Step 2: 'd' (second d)
        → I1[HAS_INITIAL, d] = ADD_LETTER
        → BUT: Check stroke pattern first
        → I3[NONE, d] = NO_REVERT (no previous transform)
        → Special: dd pattern detected → APPLY_STROKE
        → last_transform = STROKE_D
        → state = HAS_INITIAL, buf = "đ"

Step 3: 'a'
        → I1[HAS_INITIAL, a] = ADD_LETTER
        → IS_VOWEL[a] = true → state = HAS_VOWEL
        → buf = "đa"

Step 4: 'u'
        → I1[HAS_VOWEL, u] = ADD_LETTER
        → IS_VOWEL[u] = true, buf = "đau"
        → Pattern detected: "au" (index 2)

Result: "đau" ✓
```

### Case 2: "dadu" → "đau" (Delayed stroke - Telex pattern)

```
Input sequence: d, a, d, u

Step 1: 'd'
        → I1[EMPTY, d] = ADD_LETTER
        → state = HAS_INITIAL, buf = "d"

Step 2: 'a'
        → I1[HAS_INITIAL, a] = ADD_LETTER
        → IS_VOWEL[a] = true → state = HAS_VOWEL
        → buf = "da"

Step 3: 'd' (after vowel)
        → I1[HAS_VOWEL, d] = ADD_LETTER (consonant)
        → Check: Delayed stroke pattern? buf[0]='d', has_vowel, new_key='d'
        → Pattern "d + vowels + d" detected!
        → Apply stroke to buf[0]: 'd' → 'đ'
        → Second 'd' consumed (not added to buffer)
        → last_transform = STROKE_D
        → state = HAS_VOWEL, buf = "đa"

Step 4: 'u'
        → I1[HAS_VOWEL, u] = ADD_LETTER
        → buf = "đau"

Result: "đau" ✓
```

### Case 3: "daud" → "đau" (Stroke at end)

```
Input sequence: d, a, u, d

Step 1: 'd'
        → state = HAS_INITIAL, buf = "d"

Step 2: 'a'
        → state = HAS_VOWEL, buf = "da"

Step 3: 'u'
        → state = HAS_VOWEL, buf = "dau"
        → Pattern "au" detected

Step 4: 'd' (at end)
        → I1[HAS_VOWEL, d] = ADD_LETTER
        → Check delayed stroke: buf[0]='d', has_vowels, key='d'
        → Pattern matched! Apply stroke to initial 'd'
        → buf = "đau", second 'd' consumed

Result: "đau" ✓
```

### Case 4: "dadud" → "đaud" (Stroke + extra d = just letter)

```
Input sequence: d, a, d, u, d

Step 1-3: Same as Case 2
        → buf = "đa", last_transform = STROKE_D

Step 4: 'u'
        → buf = "đau"

Step 5: 'd' (third d)
        → I1[HAS_VOWEL, d] = ADD_LETTER
        → Check I3[STROKE_D, d] = REVERT?
        → NO! Revert only works when same key pressed IMMEDIATELY after transform
        → Here: transform was at step 3, now step 5
        → Delayed stroke pattern? buf[0]='đ' (already stroked) → NO
        → Just add 'd' as letter (next syllable or typo)
        → buf = "đaud"

Result: "đaud" ✓
```

### Case 5: "daudu" → "dauu" (Stroke revert via double-d at end)

```
Input sequence: d, a, u, d, u

Step 1: 'd' → buf = "d", state = HAS_INITIAL

Step 2: 'a' → buf = "da", state = HAS_VOWEL

Step 3: 'u' → buf = "dau"

Step 4: 'd' (after vowels)
        → Delayed stroke pattern detected: d + au + d
        → Apply stroke: buf = "đau"
        → last_transform = STROKE_D

Step 5: 'u' (immediately after stroke)
        → I1[HAS_VOWEL, u] = ADD_LETTER
        → Add 'u': buf = "đauu"

Wait - user expects "dauu". Let me re-analyze...

Alternative interpretation: User wants stroke REVERTED when 'u' follows 'd'?

Re-analysis with REVERT rule for 'd' + vowel:
Step 4: 'd' → stroke applied, buf = "đau", last_transform = STROKE_D
Step 5: 'u'
        → Check: should 'u' after stroke 'd' trigger revert?
        → Current logic: NO (revert only on same key)
        → User expectation: YES (vowel after stroke-d reverts)

This needs NEW rule in I2_DEFERRED:
        PENDING_STROKE_CHECK: After delayed stroke, if next key is vowel → REVERT
```

### Case 5 REVISED: "daudu" → "dauu" (với rule mới)

```
New matrix rule needed: I7_POST_STROKE_RESOLUTION

Step 4: 'd' after "dau"
        → Delayed stroke applied: buf = "đau"
        → Set pending = PENDING_STROKE_VERIFY
        → Wait for next key to confirm/revert

Step 5: 'u' (vowel after pending stroke)
        → I7[PENDING_STROKE_VERIFY, u] = REVERT_STROKE
        → Revert: 'đ' → 'd', buf = "dau"
        → Add 'u': buf = "dauu"

Result: "dauu" ✓
```

---

## Part 10: New Matrix I7 - Post-Stroke Resolution

Để handle Cases 4 & 5 theo user expectation:

```
I7_POST_STROKE[trigger_key] = action

Trigger after delayed stroke (d+vowels+d pattern):
- Next key is CONSONANT → KEEP stroke (user wants đ + consonant)
- Next key is VOWEL → REVERT stroke (user doesn't want đ, was typing pattern like "daud+u")
- Next key is same 'd' → REVERT + add 'd' (standard double-revert)

         a  b  c  d  e ... u  v  w ...
POST_STK 2  1  1  2  2 ... 2  1  1 ...
         ^revert (vowel)   ^revert
            ^keep (consonant)
```

**Updated processing flow**:

```rust
// After delayed stroke is applied:
self.pending = Some(PendingTransform {
    kind: pending::STROKE_VERIFY,
    position: 0, // initial position
});

// On next key:
if let Some(p) = self.pending {
    if p.kind == pending::STROKE_VERIFY {
        let resolution = I7_POST_STROKE[key_type];
        match resolution {
            KEEP_STROKE => {
                self.pending = None;
                // Continue normal processing
            }
            REVERT_STROKE => {
                self.revert_stroke(p.position);
                self.pending = None;
                // Then process current key normally
            }
        }
    }
}
```

---

## Summary: All 5 Cases for "đau"

| Input | Steps | Final | Matrix Path |
|-------|-------|-------|-------------|
| ddau | d→d, d→đ, a→đa, u→đau | đau ✓ | I3[STROKE_D] |
| dadu | d→d, a→da, d→đa(delayed), u→đau | đau ✓ | Delayed stroke |
| daud | d→d, a→da, u→dau, d→đau(delayed) | đau ✓ | Delayed stroke |
| dadud | d→đa(delayed), u→đau, d→đaud | đaud ✓ | Stroke locked |
| daudu | d→đau(delayed), u→revert→dauu | dauu ✓ | I7[POST_STROKE] |

---

## Updated Matrix Summary

| Matrix | Size | Purpose | Memory |
|--------|------|---------|--------|
| **Validation (Part 1)** | | | |
| M1-M6 | Various | Phonotactic rules | ~800B |
| **Placement (Part 1)** | | | |
| M7-M8 | 43×4, 43×1 | Tone/modifier position | ~220B |
| **Transform (Part 4)** | | | |
| T1-T2 | 12×6, 12×4 | Unicode mapping | ~400B |
| **English (Part 2)** | | | |
| E1-E8 | Various | English phonotactics | ~2KB |
| **Input Processing (Part 6)** | | | |
| I1: ACTION_DISPATCH | 5×64 | State→Action dispatch | 320B |
| I2: DEFERRED_RESOLUTION | 3×64 | Pending resolution | 192B |
| I3: REVERT_LOOKUP | 9×64 | Double-key revert | 576B |
| I4: STATE_TRANSITION | 5×16 | State machine | 80B |
| I5: MODIFIER_COMPAT | 4×12×2 | Modifier validity | 96B |
| I6: TONE_COMPAT | 6×43×2 | Tone validity | 516B |
| **Total** | | | **~5KB** |

---

## Part 11: Review Findings & Required Fixes

**Review Date**: 2025-12-24
**Full Report**: `MATRIX_REVIEW_REPORT.md`

### 11.1 Missing Cases Found (8 gaps)

| # | Issue | Status | Fix |
|---|-------|--------|-----|
| 1 | `pending_capitalize` not in I2 | CRITICAL | Add as 4th defer type |
| 2 | `W_SHORTCUT_SKIPPED` not in I3 | CRITICAL | Add as 12th transform type |
| 3 | Post-tone circumflex (xepse→xếp) | CRITICAL | Add to I2 or I1 |
| 4 | Delayed circumflex revert (dataa→data) | MEDIUM | Add to I3 |
| 5 | Auto-restore on space | MEDIUM | Post-COMPLETE_WORD check |
| 6 | ESC/Break handling | OUT OF SCOPE | Gate-level, not matrix |
| 7 | Word shortcuts | OUT OF SCOPE | Separate system |
| 8 | try_remove() (z key) | LOW | Add REMOVE action |

### 11.2 State Model Issue

**Problem**: HAS_MODIFIER and HAS_TONE are NOT mutually exclusive.

A vowel can have BOTH:
- Modifier: circumflex (â, ê, ô) / horn (ơ, ư) / breve (ă)
- Mark: sắc, huyền, hỏi, ngã, nặng

Example: "ấ" = 'a' + circumflex + sắc

**Solution Options**:

Option A: Combined State
```
States (revised):
  0 = EMPTY
  1 = HAS_INITIAL
  2 = HAS_VOWEL
  3 = HAS_DIACRITIC (modifier OR mark OR both)
  4 = HAS_FINAL
```

Option B: Flags (chosen)
```rust
struct InputState {
    has_initial: bool,
    has_vowel: bool,
    has_modifier: bool,
    has_mark: bool,
    has_final: bool,
}
// 5 bits = 32 states max, but most combinations invalid
// Use 6 relevant states:
enum State {
    EMPTY,              // 00000
    HAS_INITIAL,        // 10000
    HAS_VOWEL,          // x1000
    HAS_VOWEL_MOD,      // x1100 (vowel + modifier)
    HAS_VOWEL_MARK,     // x1010 (vowel + mark)
    HAS_VOWEL_BOTH,     // x1110 (vowel + modifier + mark)
    HAS_FINAL,          // x1xx1 (has final)
}
```

### 11.3 Updated I2: DEFERRED_RESOLUTION

```
DeferTypes (4) ← was 3:
  0 = PENDING_BREVE
  1 = PENDING_U_HORN
  2 = PENDING_MARK_POP
  3 = PENDING_CAPITALIZE    ← NEW

Size: 4×38 = 152 bytes (was 114)
```

### 11.4 Updated I3: REVERT_LOOKUP

```
TransformTypes (12) ← was 11:
  0 = STROKE_D
  1 = TONE_SAC
  2 = TONE_HUYEN
  3 = TONE_HOI
  4 = TONE_NGA
  5 = TONE_NANG
  6 = MOD_CIRCUMFLEX
  7 = MOD_BREVE
  8 = MOD_HORN
  9 = W_AS_VOWEL
  10 = W_SHORTCUT_SKIPPED   ← NEW
  11 = SHORT_PATTERN_STROKE ← was missing

Size: 12×38 = 456 bytes (was 418)
```

### 11.5 Post-Tone Circumflex Pattern

New rule for I1 or I2:

```
Pattern: "xepse" → "xếp"
Condition:
  - Has initial consonant
  - Vowel has mark (sắc) but no modifier
  - Non-extending final (t, m, p)
  - Same vowel typed again

Action: Add circumflex to vowel (keeping mark)

Matrix location: I1 APPLY_MODIFIER with extra check
OR: I2 PENDING_POST_TONE_CIRCUMFLEX
```

### 11.6 Updated Memory Summary

| Matrix | Original | After Fix | Change |
|--------|----------|-----------|--------|
| I2 | 114B | 152B | +38B |
| I3 | 418B | 456B | +38B |
| **Total** | ~5KB | ~5.1KB | +76B |

---

## Next Steps

1. Populate all matrix data from docs/vietnamese-language-system.md
2. Implement PhonMatrix struct with const initialization
3. Create comprehensive test suite validating all 43 patterns
4. Profile memory and cache performance
5. Integrate with engine validation flow
6. Create input-processing-matrices.md with full Rust data
7. Test random order typing scenarios against matrices
8. **NEW: Apply Part 11 fixes before implementation**
9. **NEW: Verify I7 POST_STROKE behavior with actual tests**
