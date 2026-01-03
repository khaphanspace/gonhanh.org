# V3 Engine Specification

> **Version**: 3.7 | **Status**: Ready for Implementation

## Table of Contents

**Core Reference**
1. [Full Keystroke Pipeline](#1-full-keystroke-pipeline) ⭐⭐⭐
2. [Pattern Reference Table](#2-pattern-reference-table) ⭐

**Processing Steps**
3. [Step 0: Key Classification](#3-step-0-key-classification)
4. [Step 1: PRE-CHECK (Foreign Mode)](#4-step-1-pre-check-foreign-mode)
5. [Step 2: Dispatch & Execute](#5-step-2-dispatch--execute)
6. [Step 3: Tone/Mark Placement](#6-step-3-tonemark-placement)
7. [Step 4: Buffer & State Update](#7-step-4-buffer--state-update)
8. [Step 5: Validation](#8-step-5-validation)
9. [Step 6: Restore Decision](#9-step-6-restore-decision)
10. [Step 7: Output Generation](#10-step-7-output-generation)

**Implementation**
11. [Data Structures](#11-data-structures)
12. [Bitmask Constants](#12-bitmask-constants)
13. [Examples](#13-examples)
14. [V1 vs V3 Comparison](#14-v1-vs-v3-comparison) ⭐

**Appendix**
- [A: English Detection (7 Tiers)](#appendix-a-english-detection-7-tiers)
- [B: Vietnamese Validation (9 Layers)](#appendix-b-vietnamese-validation-9-layers)
- [C: Implementation Checklist](#appendix-c-implementation-checklist)

---

## 1. Full Keystroke Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              KEYSTROKE INPUT                                 │
│  key: u8 (ASCII) | caps: bool | ctrl: bool                                  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 0: KEY CLASSIFICATION                                                  │
│─────────────────────────────────────────────────────────────────────────────│
│  classify(key) → KeyType                                                     │
│                                                                              │
│  KeyType:                                                                    │
│  • LETTER      → a-z (normal character)                                      │
│  • TONE        → s,f,r,x,j (Telex) or 1-5 (VNI)                             │
│  • MARK        → w,aa,oo,ee,dd (Telex) or 6,7,8,9,0 (VNI)                   │
│  • TERMINATOR  → space, enter, tab, punctuation                              │
│  • SPECIAL     → backspace, delete, arrow keys                               │
│  • PASSTHROUGH → ctrl+key, function keys                                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┬─────────────┐
                    ▼               ▼               ▼             ▼
              [TERMINATOR]     [LETTER/TONE/     [SPECIAL]   [PASSTHROUGH]
                    │          MARK]                │             │
                    │               │               │             │
                    ▼               ▼               ▼             ▼
             Go to STEP 6      Continue        Handle it      Pass through
             (Restore Check)       │           (backspace)    to system
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 1: PRE-CHECK (FOREIGN MODE)                                            │
│─────────────────────────────────────────────────────────────────────────────│
│  1A. SHORTCUT PREFIX CHECK (first char):                                     │
│      if first_char in [@, #, :, /] → FOREIGN_MODE                            │
│      Examples: @user, #tag, :smile:, /cmd                                    │
│                                                                              │
│  1B. NON-LETTER PREFIX CHECK:                                                │
│      if has_digit_prefix → FOREIGN_MODE                                      │
│      Examples: 149k, 2024, 3rd                                               │
│                                                                              │
│  1C. ENGLISH PATTERN CHECK (first 2-3 chars):                                │
│      if !foreign_mode && is_letter(key):                                     │
│        raw.push(key)                                                         │
│        check_foreign_pattern(raw):                                           │
│          • tier1_invalid_initial() → f,j,w,z                                 │
│          • tier2_onset_cluster()   → bl,br,cl,cr,dr,fl,fr...                │
│                                                                              │
│  Result: FOREIGN_MODE | VIETNAMESE_MODE                                      │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
             [FOREIGN_MODE]                  [VIETNAMESE_MODE]
                    │                               │
                    ▼                               ▼
             buffer.push(key)              Continue to STEP 2
             Skip all transforms
             Go to STEP 7
                                                    │
                                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 2: DISPATCH & EXECUTE                                                  │
│─────────────────────────────────────────────────────────────────────────────│
│  Based on KeyType, execute action:                                           │
│                                                                              │
│  [LETTER]:                                                                   │
│    raw.push(key)                                                             │
│    buffer.push(key)                                                          │
│    → continue                                                                │
│                                                                              │
│  [TONE] (s,f,r,x,j):                                                         │
│    if double_key(key, prev_key):     // ss, ff, rr, xx, jj                   │
│      → REVERT_TONE                                                           │
│    else:                                                                     │
│      → ADD_TONE (go to STEP 3)                                               │
│                                                                              │
│  [MARK] (w,aa,oo,ee,dd):                                                     │
│    if key == 'd' && prev == 'd':     // dd → đ                               │
│      → ADD_STROKE                                                            │
│    elif key == 'w':                                                          │
│      → ADD_HORN_OR_BREVE                                                     │
│    elif key == prev_key:              // aa, oo, ee                          │
│      → ADD_CIRCUMFLEX                                                        │
│    else:                                                                     │
│      → LETTER (fallback)                                                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 3: TONE/MARK PLACEMENT                                                 │
│─────────────────────────────────────────────────────────────────────────────│
│                                                                              │
│  3A. TONE PLACEMENT (find_tone_position):                                    │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │ Rule 1: If single vowel → place on that vowel                           ││
│  │ Rule 2: If diphthong + final → place on FIRST vowel (ái, áo, áu)        ││
│  │ Rule 3: If diphthong + no final → place on SECOND vowel (ia, oà, uê)    ││
│  │ Rule 4: If triphthong → place on MIDDLE vowel (oái, uối, ươi)           ││
│  │ Rule 5: If oa, oe, uy → place on SECOND vowel (always)                  ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│  3B. MARK PLACEMENT:                                                         │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │ CIRCUMFLEX (â,ê,ô): place on matching base vowel                        ││
│  │   aa→â, ee→ê, oo→ô                                                      ││
│  │                                                                         ││
│  │ HORN (ơ,ư): place on o→ơ or u→ư                                         ││
│  │   ow→ơ, uw→ư                                                            ││
│  │   uow→ươ (horn on both: u→ư, o→ơ)                                       ││
│  │   C+W+A pattern (3 chars exactly): mwa→mưa, cwa→cưa                     ││
│  │   But 4+ chars: swan, swap → FOREIGN (don't apply horn)                 ││
│  │                                                                         ││
│  │ BREVE (ă): DEFERRED - set pending_breve, apply later                    ││
│  │   aw → keep "aw", set pending_breve = true                              ││
│  │   aw + consonant → "ăm", "ăn", "ăp", "ăt", "ăc"                         ││
│  │   aw + tone → "ă" + tone                                                ││
│  │   aw + terminator → restore to "aw" (likely EN: law, saw)               ││
│  │                                                                         ││
│  │ STROKE (đ): replace d→đ                                                 ││
│  │   dd→đ                                                                  ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│  3C. REVERT HANDLING:                                                        │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │ Double tone key → remove tone: bás + s → bas                            ││
│  │ Double mark key → remove mark: bâ + â → ba (via baa + a → baaa)        ││
│  │ Set had_revert = true                                                   ││
│  └─────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 4: BUFFER & STATE UPDATE                                               │
│─────────────────────────────────────────────────────────────────────────────│
│                                                                              │
│  4A. UPDATE BUFFERS:                                                         │
│    raw.push(key)           // Always append raw keystroke                    │
│    buffer = transform(raw) // Apply all transforms to get buffer             │
│                                                                              │
│  4B. UPDATE STATE (BufferState):                                             │
│    if transform_applied:                                                     │
│      state.had_transform = true                                              │
│    if stroke_added:                                                          │
│      state.has_stroke = true                                                 │
│    if tone_added:                                                            │
│      state.has_tone = true                                                   │
│      state.tone_type = tone                                                  │
│    if mark_added:                                                            │
│      state.has_mark = true                                                   │
│      state.mark_type = mark                                                  │
│    if reverted:                                                              │
│      state.had_revert = true                                                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 5: VALIDATION                                                          │
│─────────────────────────────────────────────────────────────────────────────│
│  validate_vn(buffer) → VnState { Complete | Incomplete | Impossible }        │
│                                                                              │
│  9-Layer validation pipeline:                                                │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │ L1: CHAR_TYPE     - reject f,j,w,z in invalid positions                 ││
│  │ L2: ONSET         - valid single onset (b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x)│
│  │ L3: ONSET_CLUSTER - valid cluster (ch,gh,gi,kh,ng,ngh,nh,ph,qu,th,tr)   ││
│  │ L4: VOWEL_PATTERN - valid diphthong/triphthong                          ││
│  │ L5: CODA          - valid single coda (c,m,n,p,t + semi-vowels)         ││
│  │ L6: CODA_CLUSTER  - valid cluster (ch,ng,nh)                            ││
│  │ L7: TONE_STOP     - sắc/nặng only with stop codas (-c,-ch,-p,-t)        ││
│  │ L8: SPELLING      - c→k, g→gh, ng→ngh before e/i/y                      ││
│  │ L9: MODIFIER_REQ  - circumflex required for certain diphthongs          ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│  Cache result: state.vn_state = result                                       │
│                                                                              │
│  IMMEDIATE RESTORE CHECK:                                                    │
│    if vn_state == Impossible && is_english(raw):                             │
│      → RESTORE immediately (don't wait for terminator)                       │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 6: RESTORE DECISION (on TERMINATOR only)                               │
│─────────────────────────────────────────────────────────────────────────────│
│  Trigger: space, enter, tab, punctuation (.,;:!?'"()[]{}/)                   │
│                                                                              │
│  should_restore(state, raw, buffer) → Keep | Restore                         │
│                                                                              │
│  DICTIONARY-BASED RESTORE (Primary Method):                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │ 1. dict_match(buffer) → KEEP buffer                                     ││
│  │ 2. dict_match(raw)    → RESTORE to raw                                  ││
│  │ 3. Neither matches    → SKIP (keep buffer as-is, user handles)          ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│  Examples:                                                                   │
│  • "case" typed → buffer="case", raw="case" → dict(buffer)=EN → KEEP        │
│  • "casse" typed → buffer="case", raw="casse" → dict(buffer)=EN → KEEP      │
│  • "off" typed → buffer="of", raw="off" → dict(raw)=EN → RESTORE "off"      │
│  • "tets" typed → buffer="tét", raw="tets" → neither match → SKIP           │
│                                                                              │
│  FALLBACK (when no dictionary available):                                    │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │ P1: !had_transform           → KEEP (no transform = nothing to restore) ││
│  │ P2: has_stroke (đ)           → KEEP (100% intentional VN)               ││
│  │ P3: pending_breve            → RESTORE (aw+term = law, saw)             ││
│  │ P4: vn_state == Impossible   → RESTORE (broken VN = restore to raw)     ││
│  │ P5: char_consumed >= 2       → RESTORE (await→âit = too much consumed)  ││
│  │ P6: has_tone && Complete     → KEEP (intentional VN)                    ││
│  │ P7: Complete                 → KEEP (valid VN word)                     ││
│  │ P8: Otherwise                → SKIP (keep as-is)                        ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│  char_consumed = len(raw) - len(buffer)  // "await"(5) - "âit"(3) = 2       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
               [RESTORE]                         [KEEP]
                    │                               │
                    ▼                               ▼
             output = raw                    output = buffer
                    │                               │
                    └───────────────┬───────────────┘
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 7: OUTPUT GENERATION                                                   │
│─────────────────────────────────────────────────────────────────────────────│
│                                                                              │
│  Calculate diff between prev_output and new_output:                          │
│    backspaces = len(prev_output) - common_prefix_len                         │
│    commit = new_output[common_prefix_len..]                                  │
│                                                                              │
│  Generate output:                                                            │
│    ImeResult {                                                               │
│      backspaces: u8,      // Number of backspaces to send                    │
│      commit: String,      // New characters to commit                        │
│      display: String,     // Current buffer for display                      │
│    }                                                                         │
│                                                                              │
│  Example:                                                                    │
│    "ba" → "bá" (add tone)                                                    │
│    backspaces = 1 (delete 'a')                                               │
│    commit = "á"                                                              │
│                                                                              │
│    "bás" → "bas" (revert)                                                    │
│    backspaces = 2 (delete 'ás')                                              │
│    commit = "as"                                                             │
│                                                                              │
│  RESET on terminator:                                                        │
│    Clear raw, buffer, state                                                  │
│    Ready for next word                                                       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.1 Pipeline Summary Table

| Step | Name | Input | Output | When |
|------|------|-------|--------|------|
| 0 | Key Classification | key | KeyType | Every key |
| 1 | PRE-CHECK | raw | FOREIGN/VN mode | First 2 chars |
| 2 | Dispatch | KeyType | Action | Every key |
| 3 | Placement | Action + buffer | Modified buffer | Tone/Mark keys |
| 4 | Update | buffer | State flags | Every key |
| 5 | Validation | buffer | VnState | Every key |
| 6 | Restore | state + raw | Keep/Restore | Terminator only |
| 7 | Output | output | ImeResult | Every key |

### 1.2 Key Types

| Type | Keys (Telex) | Keys (VNI) | Action |
|------|--------------|------------|--------|
| LETTER | a-z (except tone/mark) | a-z (except 0-9) | Append |
| TONE | s,f,r,x,j | 1,2,3,4,5 | Add/Revert tone |
| MARK | w,aa,oo,ee,dd | 6,7,8,9,0 | Add mark/stroke |
| TERMINATOR | space,enter,tab,punct | same | Check restore |
| SPECIAL | backspace,delete,arrows | same | Handle special |
| PASSTHROUGH | ctrl+key, function | same | Pass to system |

### 1.3 Tone Mapping

| Telex | VNI | Tone | Example |
|-------|-----|------|---------|
| s | 1 | sắc (acute) | á |
| f | 2 | huyền (grave) | à |
| r | 3 | hỏi (hook) | ả |
| x | 4 | ngã (tilde) | ã |
| j | 5 | nặng (dot) | ạ |

### 1.4 Mark Mapping

| Telex | VNI | Mark | Example |
|-------|-----|------|---------|
| aa | 6 | circumflex | â |
| oo | 6 | circumflex | ô |
| ee | 6 | circumflex | ê |
| aw | 8 | breve | ă |
| ow | 7 | horn | ơ |
| uw | 7 | horn | ư |
| dd | 9 | stroke | đ |

---

## 2. Pattern Reference Table

### 2.1 English Detection Patterns (check on RAW)

| Tier | Patterns | Confidence | Step |
|------|----------|------------|------|
| 1 | `f, j, w, z` (invalid initial) | 100% | 1 |
| 2 | `bl,br,cl,cr,dr,fl,fr,gl,gr,pl,pr,sc,sk,sl,sm,sn,sp,st,sw,tw,wr` (onset cluster) | 98% | 1 |
| 3 | `ct,ft,ld,lf,lk,lm,lp,lt,xt,nd,nk,nt,pt,rb,rd,rk,rm,rn,rp,rt,sk,sp,st,sh,ry,se,ks,fe,re` (coda cluster) | 90% | 6 |
| 4 | `ea,ee,ou,ei,eu,yo,ae,yi,oo,io` (vowel pattern) | 85% | 6 |
| 5 | `tion,sion,ness,ment,able,ible,ing,ful,ous,ive` (suffix) | 90% | 6 |
| 6 | V₁+(r\|l\|t\|s\|n\|m)+V₂ pattern (e.g., ore,are,ase,ile,ure) | 75% | 6 |
| 7 | `-ew,-ow,-aw,-iew` (W-as-vowel) | 70% | 6 |

### 2.2 Vietnamese Patterns (check on BUFFER)

| Position | Valid Patterns |
|----------|---------------|
| **Onset (single)** | `b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x` (17) |
| **Onset (cluster)** | `ch,gh,gi,kh,ng,ngh,nh,ph,qu,th,tr` (11) |
| **Vowel (base)** | `a,e,i,o,u,y` (6) |
| **Vowel (modified)** | `ă,â,ê,ô,ơ,ư` (6) |
| **Diphthong** | `ai,ao,au,ay,âu,ây,eo,êu,ia,iê,iu,oa,oă,oe,oi,ôi,ơi,ua,uâ,uê,ui,uô,uy,ưa,ưi,ươ,ưu,yê` (29) |
| **Triphthong** | `iêu,yêu,oai,oay,oao,oeo,uây,uôi,uya,uyê,uyu,uêu,ươi,ươu` (14) |
| **Coda (single)** | `c,m,n,p,t` (5) |
| **Coda (cluster)** | `ch,ng,nh` (3) |
| **Coda (semi-vowel)** | `i,o,u,y` (4) |

### 2.3 Spelling Rules

| Rule | Invalid | Valid | Condition |
|------|---------|-------|-----------|
| C/K | ce,ci,cy | ke,ki,ky | c → k before e/i/y |
| G/GH | ge,gi | ghe,ghi | g → gh before e/i |
| NG/NGH | nge,ngi | nghe,nghi | ng → ngh before e/i |

### 2.4 Tone-Stop Restriction

| Coda | sắc(1) | huyền(2) | hỏi(3) | ngã(4) | nặng(5) |
|------|--------|----------|--------|--------|---------|
| -c | ✓ | ✗ | ✗ | ✗ | ✓ |
| -ch | ✓ | ✗ | ✗ | ✗ | ✓ |
| -p | ✓ | ✗ | ✗ | ✗ | ✓ |
| -t | ✓ | ✗ | ✗ | ✗ | ✓ |

---

## 3. Step 0: Key Classification

```rust
fn classify_key(key: u8, prev_key: Option<u8>, method: Method) -> KeyType {
    match method {
        Method::Telex => classify_telex(key, prev_key),
        Method::Vni => classify_vni(key, prev_key),
    }
}

fn classify_telex(key: u8, prev: Option<u8>) -> KeyType {
    match key {
        // Tone keys
        b's' | b'f' | b'r' | b'x' | b'j' => KeyType::Tone(key),

        // Mark keys (context-dependent)
        b'd' if prev == Some(b'd') => KeyType::Mark(Mark::Stroke),
        b'w' => KeyType::Mark(Mark::HornOrBreve),
        b'a' if prev == Some(b'a') => KeyType::Mark(Mark::Circumflex),
        b'o' if prev == Some(b'o') => KeyType::Mark(Mark::Circumflex),
        b'e' if prev == Some(b'e') => KeyType::Mark(Mark::Circumflex),

        // Terminators
        b' ' | b'\n' | b'\t' => KeyType::Terminator,
        b'.' | b',' | b';' | b':' | b'!' | b'?' => KeyType::Terminator,
        b'"' | b'\'' | b'(' | b')' | b'[' | b']' | b'{' | b'}' => KeyType::Terminator,

        // Letters
        b'a'..=b'z' | b'A'..=b'Z' => KeyType::Letter(key),

        // Passthrough
        _ => KeyType::Passthrough,
    }
}
```

---

## 4. Step 1: PRE-CHECK (Foreign Mode)

```rust
fn pre_check(raw: &str) -> Mode {
    // Only check on first 2-3 characters
    if raw.len() > 3 { return Mode::Vietnamese; }

    // Tier 1: Invalid VN initials
    if let Some(first) = raw.chars().next() {
        if matches!(first.to_ascii_lowercase(), 'f' | 'j' | 'w' | 'z') {
            return Mode::Foreign;
        }
    }

    // Tier 2: English-only onset clusters
    if raw.len() >= 2 {
        let bytes = raw.as_bytes();
        if is_en_onset_pair(bytes[0], bytes[1]) {
            return Mode::Foreign;
        }
    }

    Mode::Vietnamese
}
```

---

## 5. Step 2: Dispatch & Execute

```rust
fn dispatch(key_type: KeyType, buffer: &mut Buffer, state: &mut State) -> Action {
    match key_type {
        KeyType::Letter(c) => {
            buffer.raw.push(c);
            buffer.transformed.push(c);
            Action::Continue
        }

        KeyType::Tone(tone) => {
            if is_double_key(tone, state.prev_key) {
                Action::RevertTone
            } else {
                Action::AddTone(tone)
            }
        }

        KeyType::Mark(mark) => {
            Action::AddMark(mark)
        }

        KeyType::Terminator => {
            Action::CheckRestore
        }

        KeyType::Passthrough => {
            Action::Passthrough
        }
    }
}
```

---

## 6. Step 3: Tone/Mark Placement

### 6.1 Tone Position Rules

```rust
fn find_tone_position(vowels: &[VowelInfo], has_final: bool) -> usize {
    match vowels.len() {
        // Single vowel: place on it
        1 => vowels[0].position,

        // Diphthong with final: place on FIRST vowel
        // bán, báo, bấu
        2 if has_final => vowels[0].position,

        // Diphthong without final: place on SECOND vowel
        // tìa, hoà, khuê (all diphthongs follow this rule)
        2 => vowels[1].position,

        // Triphthong: place on MIDDLE vowel
        // oái, uối, ươi
        3 => vowels[1].position,

        _ => 0,
    }
}
```

### 6.2 Mark Application

```rust
fn apply_mark(buffer: &mut Buffer, mark: Mark, state: &mut State) {
    match mark {
        Mark::Circumflex => {
            // Find last a/o/e and add circumflex
            if let Some(pos) = find_vowel_for_circumflex(buffer) {
                buffer.transformed[pos] = add_circumflex(buffer.transformed[pos]);
                state.has_mark = true;
            }
        }

        Mark::HornOrBreve => {
            // Find o/u → add horn, or a → add breve
            if let Some(pos) = find_vowel_for_horn(buffer) {
                buffer.transformed[pos] = add_horn(buffer.transformed[pos]);
                state.has_mark = true;
            } else if let Some(pos) = find_vowel_for_breve(buffer) {
                buffer.transformed[pos] = add_breve(buffer.transformed[pos]);
                state.has_mark = true;
            }
        }

        Mark::Stroke => {
            // Find d → replace with đ
            if let Some(pos) = find_d_for_stroke(buffer) {
                buffer.transformed[pos] = 'đ';
                state.has_stroke = true;
            }
        }
    }
}
```

---

## 7. Step 4: Buffer & State Update

```rust
struct Buffer {
    raw: String,          // Original keystrokes
    transformed: String,  // After VN transforms
}

struct State {
    had_transform: bool,   // Any VN transform applied
    has_stroke: bool,      // đ present
    has_tone: bool,        // Tone mark present
    has_mark: bool,        // Circumflex/horn/breve present
    had_revert: bool,      // Double-key revert occurred
    pending_breve: bool,   // "aw" typed, waiting for next char
    tone_type: Option<Tone>,
    mark_type: Option<Mark>,
    vn_state: VnState,
    prev_key: Option<u8>,
}

fn update_state(action: Action, state: &mut State) {
    match action {
        Action::AddTone(t) => {
            state.had_transform = true;
            state.has_tone = true;
            state.tone_type = Some(t);
        }
        Action::AddMark(m) => {
            state.had_transform = true;
            state.has_mark = true;
            state.mark_type = Some(m);
        }
        Action::AddStroke => {
            state.had_transform = true;
            state.has_stroke = true;
        }
        Action::RevertTone | Action::RevertMark => {
            state.had_revert = true;
        }
        _ => {}
    }
}
```

---

## 8. Step 5: Validation

```rust
fn validate_vn(buffer: &str) -> VnState {
    // L1: Character type check
    for c in buffer.chars() {
        if is_invalid_char(c) { return VnState::Impossible; }
    }

    // L2-L3: Onset validation
    let onset = extract_onset(buffer);
    if !is_valid_onset(&onset) { return VnState::Impossible; }

    // L4: Vowel pattern validation
    let vowels = extract_vowels(buffer);
    if vowels.is_empty() { return VnState::Incomplete; }
    if !is_valid_vowel_pattern(&vowels) { return VnState::Impossible; }

    // L5-L6: Coda validation
    let coda = extract_coda(buffer);
    if !is_valid_coda(&coda) { return VnState::Impossible; }

    // L7: Tone-stop restriction
    if is_stop_coda(&coda) && has_invalid_tone_for_stop(buffer) {
        return VnState::Impossible;
    }

    // L8: Spelling rules
    if !check_spelling(&onset, &vowels) { return VnState::Impossible; }

    // L9: Modifier requirements
    if !check_modifier_req(buffer) { return VnState::Impossible; }

    VnState::Complete
}
```

---

## 9. Step 6: Restore Decision

```rust
fn should_restore(state: &State, raw: &str, buffer: &str, dict: &Dict) -> Decision {
    // P1: No transform = nothing to restore
    if !state.had_transform { return Decision::Keep; }

    // P2: Stroke (đ) = 100% intentional VN
    if state.has_stroke { return Decision::Keep; }

    // P3: Pending breve = restore (aw + terminator = law, saw)
    if state.pending_breve { return Decision::Restore; }

    // ═══════════════════════════════════════════════════════════════════════
    // DICTIONARY-BASED RESTORE (Primary Method)
    // ═══════════════════════════════════════════════════════════════════════
    // Priority: buffer match > raw match > skip
    // Examples:
    //   "case"  → buffer="case", raw="case"  → dict(buffer)=EN → KEEP
    //   "casse" → buffer="case", raw="casse" → dict(buffer)=EN → KEEP
    //   "off"   → buffer="of",   raw="off"   → dict(raw)=EN    → RESTORE
    //   "tets"  → buffer="tét",  raw="tets"  → neither         → SKIP

    if dict.contains(buffer) {
        return Decision::Keep;  // buffer is valid word
    }

    if dict.contains(raw) {
        return Decision::Restore;  // raw is valid word
    }

    // Neither buffer nor raw is valid word → SKIP (keep buffer as-is)
    // User will manually fix if needed
    Decision::Skip
}

// Fallback when no dictionary available
fn should_restore_fallback(state: &State, raw: &str, buffer: &str) -> Decision {
    if !state.had_transform { return Decision::Keep; }
    if state.has_stroke { return Decision::Keep; }
    if state.pending_breve { return Decision::Restore; }

    // P4: Impossible VN = restore
    if state.vn_state == VnState::Impossible {
        return Decision::Restore;
    }

    // P5: Significant char consumption = restore
    let consumed = raw.chars().count() as i32 - buffer.chars().count() as i32;
    if consumed >= 2 { return Decision::Restore; }

    // P6: Complete + tone = intentional VN
    if state.has_tone && state.vn_state == VnState::Complete {
        return Decision::Keep;
    }

    // P7: Complete = valid VN
    if state.vn_state == VnState::Complete {
        return Decision::Keep;
    }

    // P8: Otherwise = skip (don't guess)
    Decision::Skip
}
```

---

## 10. Step 7: Output Generation

```rust
fn generate_output(prev: &str, current: &str) -> ImeResult {
    let common_len = common_prefix_length(prev, current);
    let backspaces = prev.chars().count() - common_len;
    let commit: String = current.chars().skip(common_len).collect();

    ImeResult {
        backspaces,
        commit,
        display: current.to_string(),
    }
}

// Example outputs:
// "ba" → "bá": backspaces=1, commit="á"
// "bá" → "bás" (revert): backspaces=2, commit="as"
// "vie" → "viê": backspaces=1, commit="ê"
// "viêt" → "việt": backspaces=2, commit="ệt"
```

---

## 11. Data Structures

### 11.1 BufferState (u16)

```rust
struct BufferState(u16);

impl BufferState {
    // Bit layout:
    // bit 0:     had_transform
    // bit 1:     has_stroke
    // bit 2:     has_tone
    // bit 3:     has_mark
    // bit 4:     had_revert
    // bits 5-6:  revert_type (0=none, 1=tone, 2=mark)
    // bit 7:     pending_breve
    // bit 8:     pending_horn
    // bits 9-11: vn_state (0=unknown, 1=complete, 2=incomplete, 3=impossible)

    const HAD_TRANSFORM: u16 = 1 << 0;
    const HAS_STROKE: u16 = 1 << 1;
    const HAS_TONE: u16 = 1 << 2;
    const HAS_MARK: u16 = 1 << 3;
    const HAD_REVERT: u16 = 1 << 4;
}
```

### 11.2 VnState

```rust
enum VnState {
    Unknown = 0,    // Not yet validated
    Complete = 1,   // Valid complete syllable
    Incomplete = 2, // Could become valid
    Impossible = 3, // Cannot be valid VN
}
```

### 11.3 ImeResult

```rust
struct ImeResult {
    backspaces: u8,   // Number of backspaces
    commit: String,   // Characters to commit
    display: String,  // Current buffer display
}
```

### 11.4 Dictionary (for auto-restore)

```rust
/// Unified dictionary for both EN and VN words
/// Used by should_restore() to decide: buffer vs raw
struct Dict {
    /// Bloom filter for fast negative lookup (~10KB for 100K words)
    bloom: BloomFilter,
    /// Optional: full word set for exact match
    words: Option<HashSet<String>>,
}

impl Dict {
    /// Check if word exists in dictionary
    /// Returns: true if word is valid EN or VN
    fn contains(&self, word: &str) -> bool {
        // Fast path: bloom filter says NO → definitely not in dict
        if !self.bloom.may_contain(word) {
            return false;
        }
        // Slow path: check exact match (if available)
        self.words.as_ref().map(|w| w.contains(word)).unwrap_or(true)
    }
}

// Dictionary sources:
// - EN: Top 10K English words (~50KB compressed)
// - VN: Top 10K Vietnamese words (~60KB compressed)
// - Combined Bloom filter: ~10KB for 20K words (0.1% false positive)
```

**Dictionary Priority:**
```
1. dict.contains(buffer) → KEEP buffer (e.g., "case" is valid EN)
2. dict.contains(raw)    → RESTORE raw (e.g., "off" is valid EN)
3. Neither               → SKIP (keep buffer, let user fix)
```

**Example Cases:**
| Input | Buffer | Raw | dict(buffer) | dict(raw) | Decision |
|-------|--------|-----|--------------|-----------|----------|
| case | case | case | ✓ EN | ✓ EN | KEEP |
| casse | case | casse | ✓ EN | ✗ | KEEP |
| off | of | off | ✓ EN | ✓ EN | KEEP (buffer priority) |
| offf | of | offf | ✓ EN | ✗ | KEEP |
| tets | tét | tets | ✗ | ✗ | SKIP |
| bán | bán | bans | ✓ VN | ✗ | KEEP |

---

## 12. Bitmask Constants

### 12.1 Character Index

```rust
// a=0, b=1, ..., z=25, đ=26, ă=27, â=28, ê=29, ô=30, ơ=31, ư=32
fn char_idx(c: char) -> usize {
    match c.to_ascii_lowercase() {
        'a'..='z' => (c as usize) - ('a' as usize),
        'đ' => 26, 'ă' => 27, 'â' => 28, 'ê' => 29,
        'ô' => 30, 'ơ' => 31, 'ư' => 32,
        _ => 33,
    }
}
```

### 12.2 Character Type

```rust
const ONSET: u8 = 0b0001;
const VOWEL: u8 = 0b0010;
const CODA: u8 = 0b0100;
const INVALID: u8 = 0b1000;

/// Char type classification (32 bytes)
const CHAR_TYPE: [u8; 32] = [
    // a      b      c      d      e      f      g      h
    0b0010, 0b0001, 0b0101, 0b0001, 0b0010, 0b1000, 0b0001, 0b0001,
    // i      j      k      l      m      n      o      p
    0b0110, 0b1000, 0b0001, 0b0001, 0b0101, 0b0101, 0b0110, 0b0101,
    // q      r      s      t      u      v      w      x
    0b0001, 0b0001, 0b0001, 0b0101, 0b0110, 0b0001, 0b1000, 0b0001,
    // y      z      đ      ă      â      ê      ô      ơ
    0b0110, 0b1000, 0b0001, 0b0010, 0b0010, 0b0010, 0b0010, 0b0010,
];

/// Valid VN single onsets: b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x (4 bytes)
const M_ONSET: u32 = 0b_0010_0101_1111_1110_1111_1110_0110;

/// Valid VN single codas: c,m,n,p,t + semi-vowels i,o,u,y (4 bytes)
const M_CODA: u32 = 0b_0000_0011_0100_1001_0011_0100_0100;
```

### 12.3 English-only Coda Clusters (M_EN_CODA)

```rust
/// EN-only coda clusters
/// ORIGINAL: ct, ft, ld, lf, lk, lm, lp, lt, lv, xt, nd, nk, nt, pt, rb, rd, rk, rl, rm, rn, rp, rt, sk, sp, st
/// ADDED: sh (push), ry (story), se (case), ks (books), fe (safe), re (core, care)
/// Index: a=0, b=1, c=2, d=3, e=4, f=5, g=6, h=7, i=8, j=9, k=10, l=11, m=12,
///        n=13, o=14, p=15, q=16, r=17, s=18, t=19, u=20, v=21, w=22, x=23, y=24, z=25
const M_EN_CODA: [u32; 32] = [
    0x00000000, // a
    0x00000000, // b
    0x00080000, // c: +t (ct) → bit 19
    0x00000000, // d
    0x00000000, // e
    0x00080010, // f: +t (ft) → bit 19, +e (fe) → bit 4
    0x00000000, // g
    0x00000000, // h
    0x00000000, // i
    0x00000000, // j
    0x00080000, // k: +s (ks) → bit 18
    0x000AC930, // l: +d,f,k,m,p,t,v → bits 3,5,10,12,15,19,21
    0x00000000, // m
    0x000A0400, // n: +d,k,t → bits 3,10,19
    0x00000000, // o
    0x00080000, // p: +t (pt) → bit 19
    0x00000000, // q
    0x0108FC06, // r: +b,d,e,k,l,m,n,p,t,y → bits 1,3,4,10,11,12,13,15,19,24
    0x001E0090, // s: +e,h,k,p,t → bits 4,7,10,15,19
    0x00000000, // t
    0x00000000, // u
    0x00000000, // v
    0x00000000, // w
    0x00080000, // x: +t (xt) → bit 19
    0x00000000, // y
    0x00000000, // z
    0x00000000, // đ
    0x00000000, // ă
    0x00000000, // â
    0x00000000, // ê
    0x00000000, // ô
    0x00000000, // ơ
];
```

### 12.4 English-only Vowel Patterns (M_EN_VOWEL)

```rust
/// EN-only vowel pairs
/// ORIGINAL: ea, ee, ou, ei, eu, yo, ae, yi
/// ADDED: oo (book, too), io (action, ratio)
/// NOTE: oa removed - valid VN diphthong (hoà, toà, loà)
/// Index: a=0, e=4, i=8, o=14, u=20, y=24
const M_EN_VOWEL: [u32; 32] = [
    0x00000010, // a: +e (ae) → bit 4
    0x00000000, // b
    0x00000000, // c
    0x00000000, // d
    0x00100111, // e: +a,e,i,u (ea,ee,ei,eu) → bits 0,4,8,20
    0x00000000, // f
    0x00000000, // g
    0x00000000, // h
    0x00004000, // i: +o (io) → bit 14
    0x00000000, // j
    0x00000000, // k
    0x00000000, // l
    0x00000000, // m
    0x00000000, // n
    0x00104000, // o: +o,u (oo,ou) → bits 14,20 (oa removed)
    0x00000000, // p
    0x00000000, // q
    0x00000000, // r
    0x00000000, // s
    0x00000000, // t
    0x00000000, // u
    0x00000000, // v
    0x00000000, // w
    0x00000000, // x
    0x00004100, // y: +i,o (yi,yo) → bits 8,14
    0x00000000, // z
    0x00000000, // đ
    0x00000000, // ă
    0x00000000, // â
    0x00000000, // ê
    0x00000000, // ô
    0x00000000, // ơ
];
```

### 12.5 Tone-Stop Restriction

```rust
/// Tone-stop restriction: stops (c,ch,p,t) only allow sắc(1) or nặng(5)
const M_TONE_CODA: [[bool; 8]; 6] = [
    // tone 0 (none): all codas valid
    [true, true, true, true, true, true, true, true],
    // tone 1 (sắc): all codas valid
    [true, true, true, true, true, true, true, true],
    // tone 2 (huyền): stops invalid
    [false, false, true, true, true, true, false, true],
    // tone 3 (hỏi): stops invalid
    [false, false, true, true, true, true, false, true],
    // tone 4 (ngã): stops invalid
    [false, false, true, true, true, true, false, true],
    // tone 5 (nặng): all codas valid
    [true, true, true, true, true, true, true, true],
];
```

---

## 13. Examples

### 13.1 Normal Vietnamese: "chào"

```
Input: c h a o f [space]

Step 0: c → LETTER
Step 1: PRE-CHECK("c") → VN mode
Step 2: buffer = "c"
Step 4: state = {}

Step 0: h → LETTER
Step 2: buffer = "ch"

Step 0: a → LETTER
Step 2: buffer = "cha"

Step 0: o → LETTER
Step 2: buffer = "chao"

Step 0: f → TONE(huyền)
Step 3: find_tone_pos → position 3 (second vowel of "ao")
Step 2: buffer = "chào"
Step 4: state.has_tone = true

Step 0: [space] → TERMINATOR
Step 6: should_restore() → KEEP (Complete + tone)
Step 7: output = "chào "
```

### 13.2 Auto-Restore: "tesla"

```
Input: t e s l a [space]

Step 0: t → LETTER, buffer = "t"
Step 0: e → LETTER, buffer = "te"
Step 0: s → TONE(sắc)
Step 3: place tone on 'e' → buffer = "té"
Step 4: state.has_tone = true

Step 0: l → LETTER, buffer = "tél"
Step 0: a → LETTER, buffer = "téla"

Step 5: validate_vn("téla") → Impossible
        (invalid structure: consonant 'l' between toned vowel and 'a')

Step 0: [space] → TERMINATOR
Step 6: should_restore():
        - P1: had_transform = YES → continue
        - P2: has_stroke = NO → continue
        - P3: pending_breve = NO → continue
        - dict("téla") = NO → continue
        - dict("tesla") = YES (EN word) → RESTORE

Step 7: output = "tesla "
```

### 13.3 Foreign Mode: "class"

```
Input: c l a s s [space]

Step 0: c → LETTER
Step 1: PRE-CHECK("c") → VN mode
Step 2: buffer = "c"

Step 0: l → LETTER
Step 1: PRE-CHECK("cl") → FOREIGN mode (EN onset cluster)
        All subsequent keys: passthrough

Step 2: buffer = "cl"
Step 2: buffer = "cla"
Step 2: buffer = "clas"
Step 2: buffer = "class"

Step 0: [space] → TERMINATOR
Step 7: output = "class " (no transform applied)
```

### 13.4 Breve Deferral: "law"

```
Input: l a w [space]

Step 0: l → LETTER, buffer = "l"
Step 0: a → LETTER, buffer = "la"
Step 0: w → MARK(HornOrBreve)
Step 3: Breve deferral → keep buffer = "law", set pending_breve = true
        (waiting for next char to decide: consonant → VN, terminator → EN)

Step 0: [space] → TERMINATOR
Step 6: should_restore():
        - P3: pending_breve = true → RESTORE
        (aw + terminator = English word like law, saw, raw)

Step 7: output = "law "
```

### 13.4b Breve Applied: "lăm"

```
Input: l a w m [space]

Step 0: l → LETTER, buffer = "l"
Step 0: a → LETTER, buffer = "la"
Step 0: w → MARK(HornOrBreve)
Step 3: Breve deferral → buffer = "law", pending_breve = true

Step 0: m → LETTER
Step 3: pending_breve + consonant → apply breve → buffer = "lăm"
Step 4: pending_breve = false, has_mark = true

Step 0: [space] → TERMINATOR
Step 5: validate_vn("lăm") → Complete
Step 6: should_restore():
        - P7: Complete → KEEP

Step 7: output = "lăm "
```

### 13.5 VCV Pattern: "core"

```
Input: c o r e [space]

Step 0: c → LETTER, buffer = "c"
Step 0: o → LETTER, buffer = "co"
Step 0: r → TONE(hỏi)
Step 3: place tone on 'o' → buffer = "cỏ"
Step 4: state.has_tone = true

Step 0: e → LETTER, buffer = "cỏe"

Step 5: validate_vn("cỏe") → Impossible (invalid vowel pattern ỏe)

Step 0: [space] → TERMINATOR
Step 6: should_restore():
        - P3: vn_state = Impossible → RESTORE

Step 7: output = "core "
```

### 13.6 Coda Cluster: "push"

```
Input: p u s h [space]

Step 0: p → LETTER, buffer = "p"
Step 0: u → LETTER, buffer = "pu"
Step 0: s → TONE(sắc)
Step 3: place tone on 'u' → buffer = "pú"
Step 4: state.has_tone = true

Step 0: h → LETTER, buffer = "púh"

Step 5: validate_vn("púh") → Impossible (invalid coda 'h')

Step 0: [space] → TERMINATOR
Step 6: should_restore():
        - P3: vn_state = Impossible → RESTORE

Step 7: output = "push "

Note: "sh" is also tier3_coda_cluster, but P3 already triggers.
```

### 13.7 Impossible Restore: "user"

```
Input: u s e r [space]

Step 0: u → LETTER, buffer = "u"
Step 0: s → TONE(sắc)
Step 3: place tone on 'u' → buffer = "ú"
Step 4: state.has_tone = true

Step 0: e → LETTER, buffer = "úe"
Step 0: r → TONE(hỏi)
Step 3: already has tone, treated as LETTER → buffer = "úer"

Step 5: validate_vn("úer") → Impossible (invalid structure)

Step 0: [space] → TERMINATOR
Step 6: should_restore():
        - P3: vn_state = Impossible → RESTORE
        (no EN tier matches "user", but Impossible always restores)

Step 7: output = "user "
```

### 13.8 Valid Vietnamese Kept: "bán"

```
Input: b a n s [space]

Step 0: b → LETTER, buffer = "b"
Step 0: a → LETTER, buffer = "ba"
Step 0: n → LETTER, buffer = "ban"
Step 0: s → TONE(sắc)
Step 3: place tone on 'a' → buffer = "bán"
Step 4: state.has_tone = true

Step 5: validate_vn("bán") → Complete

Step 0: [space] → TERMINATOR
Step 6: should_restore():
        - P4: has_tone && Complete → KEEP

Step 7: output = "bán "
```

---

## 14. V1 vs V3 Comparison

### 14.1 Memory Comparison

| Component | V1 Size | V3 Size | Reduction |
|-----------|---------|---------|-----------|
| Validation tables | ~2KB | ~600B | 70% |
| Whitelist arrays | ~1KB | 0B (bitmask) | 100% |
| State flags | 7 bools (7B) | 1 u16 (2B) | 70% |
| Pattern checks | runtime | compile-time | N/A |
| **Total core** | **~3KB** | **~600B** | **80%** |
| EN Dictionary | ~100KB | ~100KB | Same |

### 14.2 Performance Comparison

| Operation | V1 | V3 | Speedup |
|-----------|----|----|---------|
| Single char validation | O(n) search | O(1) bit | 10x |
| Onset cluster check | strcmp | bit lookup | 5x |
| Diphthong validation | 29-item scan | bit lookup | 20x |
| Full syllable validation | ~50 ops | ~15 ops | 3x |
| Restore decision | ~20 conditions | ~8 conditions | 2.5x |
| **Total per keystroke** | **~100 ops** | **~30 ops** | **3x** |

### 14.3 Architecture Comparison

| Aspect | V1 (Production) | V3 (Smart) | Improvement |
|--------|-----------------|------------|-------------|
| **State tracking** | 7 separate bool flags | 1 BufferState u16 | Unified |
| **Validation** | 6 rules, O(n) each | 9 layers, O(1) each | 3-20x faster |
| **Validation timing** | Called twice | Called once, cached | 2x less work |
| **Restore logic** | 15+ scattered conditions | 8 structured phases | Cleaner |
| **EN detection** | 200+ lines if-else | 50 lines tiered | Maintainable |
| **Whitelist search** | O(n) array scan | O(1) bitmask | 10-20x faster |
| **Memory** | ~3KB tables | ~600B bitmasks | 80% smaller |
| **Ops per keystroke** | ~100 | ~30 | 3x faster |

### 14.4 V1 Case Coverage Checklist

```
V1 VALIDATION RULES:
☑ Rule 1: HAS_VOWEL        → L1 CHAR_TYPE vowel check
☑ Rule 2: VALID_INITIAL    → L2 M_ONSET bitmask
☑ Rule 3: ALL_CHARS_PARSED → L1-L6 syllable structure
☑ Rule 4: SPELLING_RULES   → L8 M_SPELL matrix
☑ Rule 5: VALID_FINAL      → L5-L6 M_CODA bitmask
☑ Rule 6: VALID_VOWEL      → L4 M_VOWEL_PAIR/TRIPLE

V1 MODIFIER REQUIREMENTS:
☑ Circumflex required (êu,iê,uê,yê)  → L9 M_CIRCUMFLEX_REQ
☑ Breve restrictions (ă+vowel)       → L9 M_BREVE_INVALID

V1 FOREIGN DETECTION:
☑ Invalid initials (f,j,w,z)         → CHAR_TYPE & INVALID
☑ EN onset clusters                  → M_EN_ONSET
☑ EN coda clusters                   → M_EN_CODA
☑ EN vowel patterns (ou,yo,ea)       → M_EN_VOWEL

V1 RESTORE SIGNALS:
☑ Stroke (đ)                         → state.has_stroke
☑ Tone (s,f,r,x,j)                   → state.has_tone
☑ Mark (w,aa,oo,ee)                  → state.has_mark
☑ Revert (ss,ff,rr)                  → state.had_revert

V1 RESTORE TRIGGERS:
☑ Two-check (buffer_invalid && raw_EN) → should_restore() phases
☑ Char consumption                     → raw.len() - buffer.len()
☑ V+C+V circumflex pattern             → matches_vcv_stop_pattern()
☑ Double modifier collapse             → revert_type check

V1 PREVENT RESTORE:
☑ Has stroke                           → Phase 1 quick exit
☑ Has tone + Complete VN               → Phase 6 quick exit
☑ Double modifier at end               → revert logic
☑ Non-letter prefix                    → pre-check
☑ No transform                         → Phase 1 quick exit
☑ Never collapse "ff"                  → special case in revert

V1 SPECIAL CASES:
☑ Breve deferral (aw→ăn/aw)           → pending_breve state
☑ Horn deferral (uo→ươ)               → pending_horn state
☑ Tone overwrite (banjs→bán)          → has_tone stays true
☑ Continuous typing (chaofooo)        → has_tone prevents restore
```

### 14.5 Implicit Edge Case Handling

These edge cases are caught by existing validation, no special code needed:

| Edge Case | Example | How Handled |
|-----------|---------|-------------|
| "uo" horn pattern | `uow→ươ` | Step 3B: horn applied to both u→ư, o→ơ |
| rs modifier pattern | `first→fírst` | Impossible VN (f invalid initial), auto-restore |
| V1-V2-V1 vowel collapse | `queue→quêu` | L4 vowel pattern check rejects "uêu" as invalid VN triphthong |
| Double modifier in EN | `coffee→côffêe` | Impossible VN + tier3_coda_cluster (ff) → restore |
| Consecutive modifiers | `stress→strếss` | Impossible VN (str onset) → FOREIGN_MODE at Step 1 |

### 14.6 Full Pattern Matrix

| Position | Vietnamese only | Shared (VN & EN) | English only |
|----------|-----------------|------------------|--------------|
| **ONSET** | `đ` | `b,c,d,g,h,k,l,m,n,p,q,r,s,t,v,x` | `f,j,w,z` |
| **ONSET CLUSTER** | `gh,gi,kh,ngh,nh,ph,qu` | `ch,ng,th,tr` | `bl,br,cl,cr,dr,fl,fr,gl,gr,pl,pr,sc,sk,sl,sm,sn,sp,st,sw,tw,wr` |
| **CODA (single)** | - | `c,m,n,p,t` | `b,d,g,h,k,l,q,r,s,v,x` |
| **CODA (cluster)** | `ch,nh` | `ng` | `ct,ft,ld,lf,lk,lm,lp,lt,mb,mp,nd,nk,nt,pt,rb,rd,rk,rm,rn,rp,rt,sh,sk,sp,st,xt` |
| **CODA (semi-vowel)** | - | `i,o,u,y` | - |
| **VOWEL (base)** | - | `a,e,i,o,u,y` | - |
| **VOWEL (modified)** | `ă,â,ê,ô,ơ,ư` | - | - |
| **DIPHTHONG** | `âu,ây,êu,iê,oă,ôi,ơi,uâ,uê,uô,ươ,ưa,ưi,ưu,yê` | `ai,ao,au,ay,eo,ia,iu,oa,oe,oi,ua,ui,uy` | `ea,ee,ou,ei,eu,yo,ae,yi,oo,io` |
| **TRIPHTHONG** | `iêu,yêu,ươu,uôi,ươi,oai,oay,uây,uya,uyê,uyu,uêu,oao,oeo` | - | `eau,iou,you` |
| **SUFFIX** | - | - | `tion,sion,ness,ment,able,ible,ful,less,ing,ous,ive,ize,ise,ity,ly,ed` |
| **PREFIX** | - | - | `un,re,pre,dis,mis,over,out,sub` |

---

## Appendix A: English Detection (7 Tiers)

```rust
fn is_english(raw: &str) -> bool {
    tier1_invalid_initial(raw) ||
    tier2_onset_cluster(raw) ||
    tier3_coda_cluster(raw) ||
    tier4_vowel_pattern(raw) ||
    tier5_suffix(raw) ||
    tier6_vcv_pattern(raw) ||
    tier7_w_as_vowel(raw)
}
```

[See full implementation in Section 2.1]

---

## Appendix B: Vietnamese Validation (9 Layers)

```rust
fn validate_vn(buffer: &str) -> VnState {
    // L1: CHAR_TYPE
    // L2: ONSET
    // L3: ONSET_CLUSTER
    // L4: VOWEL_PATTERN
    // L5: CODA
    // L6: CODA_CLUSTER
    // L7: TONE_STOP
    // L8: SPELLING
    // L9: MODIFIER_REQ
}
```

[See full implementation in Section 8]

---

## Appendix C: Implementation Checklist

### C.1 Core Pipeline
- [ ] Key classification (classify_key)
- [ ] Pre-check foreign mode (pre_check)
- [ ] Dispatch & execute (dispatch)
- [ ] Tone placement (find_tone_position)
- [ ] Mark placement (apply_mark)
- [ ] Buffer update
- [ ] State update
- [ ] Validation (validate_vn)
- [ ] Restore decision (should_restore)
- [ ] Output generation (generate_output)

### C.2 Data Structures
- [ ] BufferState (u16)
- [ ] VnState enum
- [ ] ImeResult struct
- [ ] Buffer struct

### C.3 Bitmask Constants
- [ ] CHAR_TYPE[33]
- [ ] M_ONSET, M_CODA
- [ ] M_EN_ONSET, M_EN_CODA, M_EN_VOWEL
- [ ] M_VOWEL_PAIR, M_VOWEL_TRIPLE
- [ ] M_TONE_CODA

### C.4 Testing
- [ ] All examples pass
- [ ] V1 regression tests pass
- [ ] Performance: <1ms per keystroke

---

**Document Version**: 3.6
**Last Updated**: 2026-01-03

---

## Changelog

### v3.7 (2026-01-03)
- **Removed redundant tone Rule 5**: Diphthong without final always places tone on 2nd vowel (no special case for oa/oe/uy needed)
- **Fixed M_EN_VOWEL matrix**: Removed `oa` (valid VN diphthong: hoà, toà, loà)
- **Generalized VCV pattern**: Now V₁+(r|l|t|s|n|m)+V₂ formula instead of case-by-case listing
- **Fixed tesla example**: Shows dictionary-based restore flow (primary method)

### v3.6 (2026-01-03)
- **Merged valuable content from old docs**:
  - Complete bitmask implementations (M_EN_CODA, M_EN_VOWEL with hex values)
  - M_ONSET, M_CODA with bitmask values
  - M_TONE_CODA matrix for tone-stop restriction
- **Added V1 vs V3 Comparison section** (Section 14):
  - Memory comparison table (80% reduction)
  - Performance comparison table (3x faster)
  - Architecture comparison table
  - V1 Case Coverage Checklist (all 30+ V1 features mapped)
  - Implicit Edge Case Handling table (uo horn, rs modifier, V1-V2-V1 collapse)
  - Full Pattern Matrix (VN only | Shared | EN only)
- **Updated Step 3B**: Added `uow→ươ` horn pattern explicitly

### v3.5 (2026-01-03)
- **Dictionary-based restore**: Primary method for restore decision
- **Priority**: `dict(buffer)` > `dict(raw)` > `SKIP`
- **Bloom filter**: Fast negative lookup (~10KB for 20K words)
- **Example cases**: case/casse, off/of, tets/tét
- **Fallback logic**: Pattern-based when no dictionary available

### v3.4 (2026-01-03)
- **Breve Deferral**: "aw" now deferred until consonant/tone/terminator
- **PRE-CHECK expanded**: Added shortcut prefix (@#:/) and digit prefix checks
- **C+W+A pattern**: mwa→mưa (3 chars), but swan→foreign (4+ chars)
- **Char consumption check**: Restore if raw is 2+ chars longer than buffer
- **10-Phase Decision**: Restructured from 7 phases to 10 phases
- **State.pending_breve**: New field for breve deferral tracking

### v3.3 (2026-01-03)
- **Tier 6 VCV expanded**: Added `ife,ose,use,ory,ary,ery` patterns
- **Phase 2 simplified**: Impossible VN now always restores (no EN check needed)
- **Examples added**: law, core, push, user, bán
- **Tesla example fixed**: Clarified P3 decision flow

### v3.2 (2026-01-03)
- Initial 7-step pipeline specification
- 7-tier English detection
- 9-layer Vietnamese validation
