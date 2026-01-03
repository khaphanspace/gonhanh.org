# V3 Auto-Restore Pipeline

## Overview

Pipeline xử lý Vietnamese IME V3 engine với 2 mechanisms:

1. **FOREIGN_MODE**: Real-time detection khi đang gõ. Detect không phải VN → skip transforms, keep-as-is.
2. **AUTO_RESTORE**: Chỉ trigger on terminator (space/enter/punctuation). Buffer invalid VN + raw/buffer ∈ Dict → restore.

| Mechanism | Timing | Trigger | Action |
|-----------|--------|---------|--------|
| FOREIGN_MODE | Real-time (every keystroke) | English pattern detected | Skip transforms, keep raw |
| AUTO_RESTORE | On terminator only | Invalid VN + dictionary match | Restore to raw or keep buffer |

---

## Pattern Reference Table (Master)

Bảng tổng hợp patterns + functions cho cả English detection và Vietnamese validation.

### English Detection Functions (check on RAW)

| Position | English only patterns | Function | Step |
|----------|----------------------|----------|------|
| **ONSET** | `f,j,w,z` | `has_english_onset()` | 0 |
| **ONSET CLUSTER** | `bl,br,cl,cr,dr,dw,fl,fr,gl,gr,pl,pr,sc,sk,sl,sm,sn,sp,st,sw,tw,wr` | `has_english_onset()` | 0 |
| **CODA (single)** | `b,d,g,h,k,l,q,r,s,v,x` | `has_english_coda()` | 2A |
| **CODA (cluster)** | `ct,ft,ld,lf,lk,lm,lp,lt,lv,mb,mp,nd,nk,nt,pt,rb,rd,rk,rl,rm,rn,rp,rt,sk,sp,st,xt` | `has_english_coda()` | 2A |
| **DIPHTHONG** | `ea,ee,ou,ei,eu,yo,ae,yi` | `has_invalid_vowel_pattern()` | 2A |
| **TRIPHTHONG** | `eau,iou,you` | `has_invalid_vowel_pattern()` | 2A |
| **SUFFIX** | `tion,sion,ness,ment,able,ible,ful,less,ing,ous,ive,ize,ise,ity,ly,ed` | `has_english_suffix()` | 2A |
| **PREFIX** | `un,re,pre,dis,mis,over,out,sub` | `has_english_prefix()` | 2A |
| **BIGRAM** | `bk,cb,dk,gk,hb,jb,kb,kx,kz,pb,qb,tb,vb,wb,xb,zb,...` | `has_impossible_bigram()` | 2A |
| **DOUBLE CONS.** | `ll,ss,ff,tt,pp,mm,nn,rr,dd,gg,bb,zz,cc` | ⚠️ SKIP (Telex revert) | 2B |

### Vietnamese Validation Functions (check on BUFFER)

| Position | Vietnamese patterns | Function | Layer |
|----------|---------------------|----------|-------|
| **ONSET (single)** | `b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x` (17) | `is_valid_onset()` | L2 |
| **ONSET (cluster)** | `ch,gh,gi,kh,kr¹,ng,ngh,nh,ph,qu,th,tr` (12) | `is_valid_onset_cluster()` | L3 |
| **VOWEL (base)** | `a,e,i,o,u,y` (6) | `is_valid_vowel()` | L1 |
| **VOWEL (modified)** | `ă,â,ê,ô,ơ,ư` (6) | `is_valid_vowel()` | L1 |
| **DIPHTHONG** | `ai,ao,au,ay,âu,ây,eo,êu,ia,iê,iu,oa,oă,oe,oi,ôi,ơi,oo²,ua,uâ,uê,ui,uô,uy,ưa,ưi,ươ,ưu,yê` (29) | `is_valid_diphthong()` | L4 |
| **TRIPHTHONG** | `iêu,yêu,oai,oay,oao,oeo,uây,uôi,uya,uyê,uyu,uêu,ươi,ươu` (14) | `is_valid_triphthong()` | L4 |
| **CODA (single)** | `c,m,n,p,t` (5) | `is_valid_coda()` | L5 |
| **CODA (cluster)** | `ch,ng,nh` (3) | `is_valid_coda_cluster()` | L6 |
| **CODA (semi-vowel)** | `i,o,u,y` (4) | `is_valid_coda()` | L5 |
| **TONE-STOP** | sắc/nặng only with `-c,-ch,-p,-t` | `is_valid_tone_coda()` | L7 |
| **SPELLING** | c→k, g→gh, ng→ngh before e,i,y | `is_valid_spelling()` | L8 |

### Full Pattern Matrix

| Position | Vietnamese only | Shared (VN & EN) | English only |
|----------|-----------------|------------------|--------------|
| **ONSET** | `đ` | `b,c,d,g,h,k,l,m,n,p,q,r,s,t,v,x` | `f,j,w,z` |
| **ONSET CLUSTER** | `gh,gi,kh,ngh,nh,ph,qu,kr¹` | `ch,ng,th,tr` | `bl,br,cl,cr,dr,dw,fl,fr,gl,gr,pl,pr,sc,sk,sl,sm,sn,sp,st,sw,tw,wr` |
| **CODA (single)** | - | `c,m,n,p,t` | `b,d,g,h,k,l,q,r,s,v,x` |
| **CODA (cluster)** | `ch,nh` | `ng` | `ct,ft,ld,lf,lk,lm,lp,lt,lv,mb,mp,nd,nk,nt,pt,rb,rd,rk,rl,rm,rn,rp,rt,sk,sp,st,xt` |
| **CODA (semi-vowel)** | - | `i,o,u,y` | - |
| **VOWEL (base)** | - | `a,e,i,o,u,y` | - |
| **VOWEL (modified)** | `ă,â,ê,ô,ơ,ư` | - | - |
| **DIPHTHONG** | `âu,ây,êu,iê,oă,ôi,ơi,uâ,uê,uô,ươ,ưa,ưi,ưu,yê,oo²` | `ai,ao,au,ay,eo,ia,iu,oa,oe,oi,ua,ui,uy` | `ea,ee,ou,ei,eu,yo,ae,yi` |
| **TRIPHTHONG** | `iêu,yêu,ươu,uôi,ươi,oai,oay,uây,uya,uyê,uyu,uêu,oao,oeo` | - | `eau,iou,you` |
| **SUFFIX** | - | - | `tion,sion,ness,ment,able,ible,ful,less,ing,ous,ive,ize,ise,ity,ly,ed` |
| **PREFIX** | - | - | `un,re,pre,dis,mis,over,out,sub` |
| **BIGRAM** | - | (most combinations) | `bk,cb,dk,gk,hb,jb,kb,kx,kz,pb,qb,tb,vb,wb,xb,zb,...` |
| **DOUBLE CONS.** | - | - | `ll,ss,ff,tt,pp,mm,nn,rr,dd,gg,bb,zz,cc` |

**Notes:**
- ¹ `kr` - ethnic minority place names (Krông Búk, Đắk Lắk)
- ² `oo` - thoòng, thoông (valid VN)

### Source Comparison

| Source | Onset (single) | Onset (cluster) | Coda (single) | Coda (cluster) | Diphthongs | Triphthongs |
|--------|----------------|-----------------|---------------|----------------|------------|-------------|
| **V1 Engine** | 16 | 11+ngh+kr | 6+semi-vowels | 3 | 13 base | 13 |
| **V3 Engine** | 17 (đ) | 11+ngh | 6 | 3 | 18 modified | 13 |
| **OpenKey** | 17+4 foreign | 10 | 5 | 3 | ~20 | ~15 |

### V1 Foreign Detection (validation.rs)

```rust
// V1 detects foreign via is_foreign_word_pattern():
// 1. Vowel patterns: ou, yo (never valid VN)
// 2. Consonant clusters: T+R, P+R, C+R after finals
// 3. Invalid finals: b,d,g,h,k,l,q,r,s,v,x (single)
```

### OpenKey Spell Checking (Engine.cpp)

```cpp
// OpenKey validates via checkSpelling():
// 1. _consonantTable - valid initials
// 2. _endConsonantTable - valid finals
// 3. _vowelCombine - valid vowel combos
// 4. Tone-stop restriction: ch/t finals can't have hỏi/ngã/huyền
// 5. vRestoreIfWrongSpelling flag - restore raw if invalid
```

**Detection logic:**
- **English only** → trigger FOREIGN_MODE (invalid VN)
- **Shared** → ambiguous, cần thêm context hoặc dictionary
- **Vietnamese only** → valid VN, không trigger

**Notes:**
- **ONSET/CODA**: Kiểm tra đầu/cuối raw
- **DIPHTHONG/TRIPHTHONG**: Tìm vowel pattern trong raw
- **DOUBLE CONSONANT**: KHÔNG trigger FOREIGN_MODE (có thể từ Telex revert `ss` → revert sắc). Chỉ dùng ở Step 2B dictionary lookup.

---

## Function Naming Convention

```rust
// Pattern detection (check on RAW) - maps to Pattern Reference Table
fn has_english_onset(raw: &str) -> bool;           // ONSET, ONSET CLUSTER (EN-only)
fn has_english_coda(raw: &str) -> bool;            // CODA single + cluster (EN-only)
fn has_invalid_vowel_pattern(raw: &str) -> bool;   // DIPHTHONG, TRIPHTHONG (EN-only)
fn has_english_suffix(raw: &str) -> bool;          // SUFFIX (EN-only)
fn has_english_prefix(raw: &str) -> bool;          // PREFIX (EN-only)
fn has_impossible_bigram(raw: &str) -> bool;       // BIGRAM (EN-only)

// Combined check (ALL patterns from table except DOUBLE CONS.)
fn has_english_pattern(raw: &str) -> bool {
    has_english_onset(raw)           // Step 0 only
    || has_english_coda(raw)         // Step 2A
    || has_invalid_vowel_pattern(raw) // Step 2A
    || has_english_suffix(raw)       // Step 2A ← NEW
    || has_english_prefix(raw)       // Step 2A ← NEW
    || has_impossible_bigram(raw)    // Step 2A
    // NOTE: DOUBLE CONS. skipped (Telex revert) → Step 2B dict only
}

// Vietnamese validation (check on BUFFER)
fn is_valid_vietnamese(buffer: &str) -> ValidationResult;
fn is_impossible_vietnamese(buffer: &str) -> bool;

// Dictionary lookup
fn is_in_english_dictionary(word: &str) -> bool;

// Mode decisions
fn should_enter_foreign_mode(raw: &str, buffer: &str) -> bool;
fn should_auto_restore(raw: &str, buffer: &str) -> Option<String>;
```

### Pattern → Function Mapping

| Pattern (from Table) | Column Used | Function | Step |
|---------------------|-------------|----------|------|
| ONSET | English only: `f,j,w,z` | `has_english_onset()` | 0 |
| ONSET CLUSTER | English only: `bl,br,cl,cr,...` | `has_english_onset()` | 0 |
| CODA (single) | English only: `b,d,g,h,k,l,...` | `has_english_coda()` | 2A |
| CODA (cluster) | English only: `ct,ft,ld,nd,...` | `has_english_coda()` | 2A |
| DIPHTHONG | English only: `ea,ee,ou,ei,...` | `has_invalid_vowel_pattern()` | 2A |
| TRIPHTHONG | English only: `eau,iou,you` | `has_invalid_vowel_pattern()` | 2A |
| SUFFIX | English only: `tion,ing,ed,...` | `has_english_suffix()` | 2A |
| PREFIX | English only: `un,re,pre,dis,...` | `has_english_prefix()` | 2A |
| BIGRAM | English only: `bk,cb,dk,...` | `has_impossible_bigram()` | 2A |
| DOUBLE CONS. | ⚠️ SKIP | - | 2B (dict) |

---

## Full Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                         INPUT                                   │
│  • raw: keystrokes đang gõ (không dấu)                         │
│  • key: phím vừa nhấn                                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              STEP 0: PRE-CHECK → FOREIGN_MODE                   │
│─────────────────────────────────────────────────────────────────│
│  Check: has_english_onset(raw)?                                 │
│  Patterns: See Pattern Reference Table → ONSET, ONSET CLUSTER  │
│                                                                 │
│  If pattern found → ENTER FOREIGN_MODE (skip all transforms)   │
│  If no pattern → Vietnamese mode (allow transforms)             │
└─────────────────────────────────────────────────────────────────┘
                              │
                     ┌────────┴────────┐
                     ▼                 ▼
              [EN Pattern]       [No EN Pattern]
                     │                 │
                     ▼                 ▼
            ┌─────────────────┐  ALLOW TRANSFORM
            │  FOREIGN_MODE   │  (apply VN transforms)
            │  Skip transforms│  → buffer = transform(raw)
            │  Keep raw as-is │        │
            └─────────────────┘        │
                     │                 ▼
                     ▼           Continue to STEP 1...
                  [DONE]
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                 STEP 1: VIETNAMESE VALIDATION                   │
│─────────────────────────────────────────────────────────────────│
│  Check: is_valid_vietnamese(buffer)?                            │
│  Patterns: See Pattern Reference Table → Vietnamese (valid)    │
│                                                                 │
│  Validate buffer against phonotactic rules:                     │
│  1. Valid onset (+ clusters)                                    │
│  2. Valid vowel nucleus                                         │
│  3. Valid diphthong/triphthong                                  │
│  4. Valid coda                                                  │
│  5. Tone-stop restriction: sắc/nặng only with -c,-ch,-p,-t     │
│                                                                 │
│  Result: Complete | Incomplete | Impossible                     │
└─────────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
     [Complete]          [Incomplete]       [Impossible]
          │                   │                   │
          ▼                   ▼                   │
      KEEP VN             KEEP VN                │
     (valid word)        (keep typing)           │
          │                   │                   │
          ▼                   ▼                   ▼
       [DONE]              [DONE]          Continue to STEP 2A...
                                                 │
                                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│            STEP 2A: FOREIGN_MODE DETECTION                      │
│                    (runs EVERY keystroke)                       │
│─────────────────────────────────────────────────────────────────│
│  CHECK 1: is_impossible_vietnamese(buffer)?  ← check trên BUFFER│
│  CHECK 2: has_english_pattern(raw)?          ← check trên RAW  │
│                                                                 │
│  Condition: CHECK 1 && CHECK 2 → FOREIGN_MODE                   │
│                                                                 │
│  Patterns: See Pattern Reference Table (English Detection)      │
│  ✓ CODA         → has_english_coda(raw)                        │
│  ✓ DIPHTHONG    → has_invalid_vowel_pattern(raw)               │
│  ✓ SUFFIX       → has_english_suffix(raw)      ← NEW           │
│  ✓ PREFIX       → has_english_prefix(raw)      ← NEW           │
│  ✓ BIGRAM       → has_impossible_bigram(raw)                   │
│  ✗ ONSET        → already checked at Step 0                    │
│  ✗ DOUBLE CONS. → skip (Telex revert), wait Step 2B            │
│                                                                 │
│  If both checks pass → ENTER FOREIGN_MODE (keep raw)            │
│  If not → Continue to terminator check                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                     ┌────────┴────────┐
                     ▼                 ▼
          [Pattern Match]        [No Pattern]
                  │                    │
                  ▼                    ▼
        ┌─────────────────┐    ┌─────────────────┐
        │  FOREIGN_MODE   │    │ Is terminator?  │
        │  Keep raw as-is │    │ key ∈ {space,   │
        │                 │    │ enter,,.;:!?'"} │
        └─────────────────┘    └─────────────────┘
                  │                    │
                  ▼           ┌────────┴────────┐
               [DONE]         ▼                 ▼
                            [Yes]              [No]
                              │                 │
                              ▼                 ▼
                              │           KEEP AS-IS
                              │          (wait for more
                              │            keystrokes)
                              │                 │
                              ▼                 ▼
                              │              [DONE]
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│            STEP 2B: AUTO_RESTORE                                │
│                 (only on terminator key)                        │
│─────────────────────────────────────────────────────────────────│
│  PRE-CONDITION: is_terminator(key)                              │
│  CHECK: is_impossible_vietnamese(buffer)?  ← check trên BUFFER │
│                                                                 │
│  If CHECK = false → KEEP VN (valid Vietnamese)                  │
│  If CHECK = true  → Continue dictionary lookup                  │
│                                                                 │
│  Dictionary: HashSet<10,000 common English words>               │
│  Size: ~100KB, Lookup: O(1)                                     │
│                                                                 │
│  Check order:                                                   │
│  1. is_in_english_dictionary(raw)?    → AUTO_RESTORE(raw)      │
│  2. is_in_english_dictionary(buffer)? → KEEP(buffer)           │
│  3. Neither match                     → KEEP AS-IS (typo)      │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
        [raw match]     [buffer match]   [No match]
              │               │               │
              ▼               ▼               ▼
      AUTO_RESTORE(raw) KEEP(buffer)     KEEP AS-IS
              │               │            (typo)
              ▼               ▼               │
           [DONE]          [DONE]             ▼
                                           [DONE]
```

---

## Decision Summary

| Step | Timing | Check ON | Function | Pattern (from Table) | Action | Output |
|------|--------|----------|----------|---------------------|--------|--------|
| 0 | Before transform | **raw** | `has_english_onset()` | ONSET, ONSET CLUSTER | FOREIGN_MODE | raw |
| 1 | Every key | **buffer** | `is_valid_vietnamese()` | VN Validation (L1-L8) | KEEP VN | buffer |
| 2A | Every key | **buffer** + **raw** | `is_impossible_vietnamese()` + `has_english_coda()` | CODA (single+cluster) | FOREIGN_MODE | raw |
| 2A | Every key | **buffer** + **raw** | `is_impossible_vietnamese()` + `has_invalid_vowel_pattern()` | DIPHTHONG, TRIPHTHONG | FOREIGN_MODE | raw |
| 2A | Every key | **buffer** + **raw** | `is_impossible_vietnamese()` + `has_english_suffix()` | SUFFIX | FOREIGN_MODE | raw |
| 2A | Every key | **buffer** + **raw** | `is_impossible_vietnamese()` + `has_english_prefix()` | PREFIX | FOREIGN_MODE | raw |
| 2A | Every key | **buffer** + **raw** | `is_impossible_vietnamese()` + `has_impossible_bigram()` | BIGRAM | FOREIGN_MODE | raw |
| 2B | Terminator | **raw** | `is_in_english_dictionary()` | DOUBLE CONS. (via dict) | AUTO_RESTORE | raw |
| 2B | Terminator | **buffer** | `is_in_english_dictionary()` | - | KEEP | buffer |
| 2B | Terminator | - | - | No match | KEEP | buffer (typo) |

**Terminology:**
- **FOREIGN_MODE**: Real-time, skip transforms, keep raw as-is
- **AUTO_RESTORE**: On terminator only, restore raw from dictionary

**Check targets:**
- **buffer**: Text sau khi transform (có thể có dấu VN)
- **raw**: Original keystrokes (không dấu)

**Trigger conditions:**
```
Step 0:  has_english_onset(raw)                                      → FOREIGN_MODE
Step 2A: is_impossible_vietnamese(buffer) && has_english_pattern(raw) → FOREIGN_MODE
Step 2B: is_impossible_vietnamese(buffer) && is_in_english_dictionary(word) → AUTO_RESTORE/KEEP
```

**NOTE:** Double consonant (ss, ff, tt...) từ Telex revert → KHÔNG trigger FOREIGN_MODE.
Chờ đến Step 2B để AUTO_RESTORE check dictionary.

---

## Terminator Keys

```
space, enter, tab
, . ; : ! ? ' " ( ) [ ] { } / \ - + = @ # $ % ^ & * < >
```

---

## Examples

### Example 1: "class" (FOREIGN_MODE at start)

```
Keystroke sequence: c → l → a → s → s → [space]

Step 0: PRE-CHECK on RAW
├── raw = "cl"
├── Check: "cl" is EN-only onset cluster? ✓
└── Result: ENTER FOREIGN_MODE

Subsequent keystrokes (FOREIGN_MODE):
├── l → "cl"
├── a → "cla"
├── s → "clas" (no tone transform)
├── s → "class" (literal, no revert logic)
└── [space] → output "class"

Final: "class" (no Vietnamese transform ever applied)
```

| Key | raw | buffer | Mode | Action |
|-----|-----|--------|------|--------|
| c | c | c | VN | - |
| l | cl | cl | **FOREIGN_MODE** | Step 0: "cl" pattern on raw |
| a | cla | cla | FOREIGN_MODE | literal |
| s | clas | clas | FOREIGN_MODE | literal |
| s | class | class | FOREIGN_MODE | literal |
| space | class | class | - | Output: "class" |

---

### Example 2: "file" (FOREIGN_MODE - invalid VN initial)

```
Keystroke sequence: f → i → l → e → [space]

Step 0: PRE-CHECK on RAW
├── raw = "f"
├── Check: "f" is invalid VN initial? ✓
└── Result: ENTER FOREIGN_MODE

Subsequent keystrokes (FOREIGN_MODE):
├── i → "fi"
├── l → "fil"
├── e → "file"
└── [space] → output "file"

Final: "file" (no Vietnamese transform ever applied)
```

| Key | raw | buffer | Mode | Action |
|-----|-----|--------|------|--------|
| f | f | f | **FOREIGN_MODE** | Step 0: "f" invalid VN initial |
| i | fi | fi | FOREIGN_MODE | literal |
| l | fil | fil | FOREIGN_MODE | literal |
| e | file | file | FOREIGN_MODE | literal |
| space | file | file | - | Output: "file" |

---

### Example 3: "case" (AUTO_RESTORE via dictionary)

```
Keystroke sequence: c → a → s → e → [space]

Step 0: PRE-CHECK
├── raw = "c"
├── Check: No EN pattern at start
└── Result: ALLOW TRANSFORM → Vietnamese mode

Keystroke processing:
├── c → buffer = "c"
├── a → buffer = "ca"
├── s → 's' is Telex sắc modifier → buffer = "cá"
├── e → buffer = "cáe"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "cáe"
├── Check: "áe" valid VN vowel combination? ✗
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="cáe")? ✓ (áe not valid)
├── CHECK 2: EN_Pattern(raw="case")?
│   ├── L5 Coda cluster? ✗ (se not coda cluster)
│   ├── L7 Vowel pattern? ✗ (no ea/ee/oo/ou/ei/eu)
│   └── L8 Impossible bigram? ✗
└── Result: No pattern match → continue to Step 2B

Step 2B: AUTO_RESTORE (terminator = space)
├── CHECK: Invalid_VN(buffer="cáe")? ✓
├── CHECK: raw="case" ∈ Dictionary? ✓
└── Result: AUTO_RESTORE → "case"

Final: "case"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| c | c | c | - | No pattern, VN mode |
| a | ca | ca | - | - |
| s | cas | cá | - | 's' = sắc tone |
| e | case | cáe | - | - |
| space | case | cáe | Impossible | AUTO_RESTORE: Dict(raw) ✓ |

---

### Example 4: "casse" (Telex revert → AUTO_RESTORE keeps buffer)

```
Keystroke sequence: c → a → s → s → e → [space]

Step 0: PRE-CHECK
├── raw = "c"
├── Check: No EN pattern at start
└── Result: ALLOW TRANSFORM → Vietnamese mode

Keystroke processing (normal Telex behavior):
├── c → buffer = "c"
├── a → buffer = "ca"
├── s → 's' is Telex sắc modifier → buffer = "cá"
├── s → second 's' triggers REVERT (normal Telex) → buffer = "cas"
├── e → buffer = "case"
└── [space] → trigger validation

NOTE: Double 's' for revert is NORMAL Telex behavior, NOT English signal.
      Do NOT trigger immediate restore on revert.

Step 1: VN VALIDATION
├── buffer = "case"
├── Check: "se" valid VN final? ✗ ('s' not valid final consonant)
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="case")? ✓ (se not valid final)
├── CHECK 2: EN_Pattern(raw="casse")?
│   ├── L3 Double consonant "ss"? - SKIP (could be from revert)
│   ├── L5 Coda cluster? ✗
│   └── L7 Vowel pattern? ✗
└── Result: No FOREIGN_MODE trigger → continue to Step 2B

Step 2B: AUTO_RESTORE (terminator = space)
├── CHECK: Invalid_VN(buffer="case")? ✓
├── CHECK 1: raw="casse" ∈ Dictionary? ✗ (not a word)
├── CHECK 2: buffer="case" ∈ Dictionary? ✓
└── Result: KEEP buffer → "case"

Final: "case"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| c | c | c | - | No pattern, VN mode |
| a | ca | ca | - | - |
| s | cas | cá | - | 's' = sắc tone |
| s | cass | cas | - | REVERT (normal Telex) |
| e | casse | case | - | - |
| space | casse | case | Impossible | KEEP: Dict(buffer) ✓ |

**Key insight:** User typed "ss" to revert tone and get literal 's', then added 'e'.
User wanted "case", not "casse". Output = buffer = "case".

---

### Example 5: "coffee" (double consonant - actual English word)

```
Keystroke sequence: c → o → f → f → e → e → [space]

Step 0: PRE-CHECK
├── raw = "c"
├── Check: No EN pattern at start
└── Result: ALLOW TRANSFORM → Vietnamese mode

Keystroke processing:
├── c → buffer = "c"
├── o → buffer = "co"
├── f → 'f' is Telex huyền modifier → buffer = "cò"
├── f → second 'f' triggers REVERT → buffer = "cof"
├── e → buffer = "cofe"
├── e → second 'e' = circumflex → buffer = "cofê"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "cofê"
├── Check: 'f' valid in VN word? ✗ (f only valid as initial, not medial)
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="cofê")? ✓
├── CHECK 2: EN_Pattern(raw="coffee")?
│   └── L7: Vowel pattern "ee"? ✓
└── Result: ENTER FOREIGN_MODE → keep raw

Final: "coffee"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| c | c | c | - | No pattern, VN mode |
| o | co | co | - | - |
| f | cof | cò | - | 'f' = huyền tone |
| f | coff | cof | - | REVERT (normal Telex) |
| e | coffe | cofe | - | - |
| e | coffee | cofê | Impossible | FOREIGN_MODE: L7 "ee" ✓ |
| space | coffee | coffee | - | Output: "coffee" |

**Note:** "ee" vowel pattern triggers FOREIGN_MODE immediately, not the "ff" double consonant.

---

### Example 6: "bass" (AUTO_RESTORE to raw)

```
Keystroke sequence: b → a → s → s → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── b → buffer = "b"
├── a → buffer = "ba"
├── s → 's' sắc → buffer = "bá"
├── s → REVERT → buffer = "bas"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "bas"
├── Check: 's' valid VN final? ✗
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="bas")? ✓ (s not valid final)
├── CHECK 2: EN_Pattern(raw="bass")?
│   ├── L3 Double consonant "ss"? - SKIP (could be from revert)
│   └── No other patterns
└── Result: No FOREIGN_MODE trigger → continue to Step 2B

Step 2B: AUTO_RESTORE (terminator = space)
├── CHECK: Invalid_VN(buffer="bas")? ✓
├── CHECK 1: raw="bass" ∈ Dictionary? ✓
└── Result: AUTO_RESTORE → "bass"

Final: "bass"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| b | b | b | - | - |
| a | ba | ba | - | - |
| s | bas | bá | - | 's' = sắc tone |
| s | bass | bas | - | REVERT |
| space | bass | bas | Impossible | AUTO_RESTORE: Dict(raw) ✓ |

**Contrast với "casse":**
- "bass" → raw ∈ Dict → AUTO_RESTORE(raw) → "bass"
- "casse" → raw ∉ Dict, buffer ∈ Dict → KEEP(buffer) → "case"

---

### Example 7: "their" (FOREIGN_MODE via vowel pattern)

```
Keystroke sequence: t → h → e → i → r → [space]

Step 0: PRE-CHECK
├── raw = "th"
├── Check: "th" is EN onset cluster? ✗ (th valid in VN: thành, thì)
└── Result: ALLOW TRANSFORM → Vietnamese mode

Keystroke processing:
├── t → buffer = "t"
├── h → buffer = "th"
├── e → buffer = "the"
├── i → buffer = "thei"
├── r → 'r' is Telex hỏi modifier → buffer = "thẻi"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "thẻi"
├── Check: "ei" valid VN diphthong? ✗
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="thẻi")? ✓ (ei not valid diphthong)
├── CHECK 2: EN_Pattern(raw="their")?
│   └── L7: Vowel pattern "ei"? ✓
└── Result: ENTER FOREIGN_MODE → keep raw

Final: "their"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| t | t | t | - | - |
| h | th | th | - | th valid VN, continue |
| e | the | the | - | - |
| i | thei | thei | - | - |
| r | their | thẻi | Impossible | FOREIGN_MODE: L7 "ei" ✓ |
| space | their | their | - | Output: "their" |

---

### Example 8: "user" (dictionary only)

```
Keystroke sequence: u → s → e → r → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── u → buffer = "u"
├── s → 's' sắc on 'u' → buffer = "ú"
├── e → buffer = "úe"
├── r → 'r' hỏi, but 'e' already has no tone → buffer = "úe" (r ignored or "ủe"?)
└── [space] → trigger validation

Assuming buffer = "úer" or "ủe":

Step 1: VN VALIDATION
├── buffer = "ủe" or similar
├── Check: Invalid structure
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer)? ✓
├── CHECK 2: EN_Pattern(raw="user")?
│   └── No coda, vowel pattern, or bigram match
└── Result: No FOREIGN_MODE trigger → continue to Step 2B

Step 2B: AUTO_RESTORE (terminator = space)
├── CHECK: Invalid_VN(buffer)? ✓
├── CHECK: raw="user" ∈ Dictionary? ✓
└── Result: AUTO_RESTORE → "user"

Final: "user"
```

---

### Example 9: "việt" (valid Vietnamese)

```
Keystroke sequence: v → i → e → e → j → t → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── v → buffer = "v"
├── i → buffer = "vi"
├── e → buffer = "vie"
├── e → second 'e' = circumflex on 'e' → buffer = "viê"
├── j → 'j' = nặng tone → buffer = "việ"
├── t → buffer = "việt"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "việt"
├── Check all rules: ✓
└── Result: Complete (valid VN syllable)

Final: "việt" (KEEP VN)
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| v | v | v | - | - |
| i | vi | vi | - | - |
| e | vie | vie | - | - |
| e | viee | viê | - | Circumflex |
| j | vieej | việ | - | Nặng tone |
| t | vieejt | việt | Complete | KEEP VN |
| space | vieejt | việt | - | Output: "việt" |

---

### Example 10: "xyz" (typo - no match)

```
Keystroke sequence: x → y → z → [space]

Step 0: PRE-CHECK → No pattern (x valid VN initial)

Keystroke processing:
├── x → buffer = "x"
├── y → buffer = "xy"
├── z → buffer = "xyz" (z causes issue?)
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "xyz" or transformed
├── Result: Impossible

Step 2A: EN PATTERN DETECTION
├── raw = "xyz"
├── Check all layers: ✗ (x valid VN, no clusters, no double consonant)
└── No match

Step 2B: DICTIONARY LOOKUP
├── raw = "xyz" ∈ Dictionary? ✗
├── buffer ∈ Dictionary? ✗
└── Result: No match → KEEP AS-IS (typo)

Final: buffer (kept as typed, possibly with transforms)
```

---

### Example 11: "text" (coda cluster xt)

```
Keystroke sequence: t → e → x → t → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── t → buffer = "t"
├── e → buffer = "te"
├── x → 'x' is Telex ngã modifier → buffer = "tẽ"
├── t → buffer = "tẽt"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "tẽt"
├── Check: Structure valid? (single vowel + tone + final t)
└── Result: Possibly valid or Impossible (depends on rules)

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="tẽt")? ✓ (ngã tone invalid with -t final)
├── CHECK 2: EN_Pattern(raw="text")?
│   └── L5: Coda cluster "xt"? ✓
└── Result: ENTER FOREIGN_MODE → keep raw

Final: "text"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| t | t | t | - | - |
| e | te | te | - | - |
| x | tex | tẽ | - | 'x' = ngã tone |
| t | text | tẽt | Impossible | FOREIGN_MODE: L5 "xt" ✓ |
| space | text | text | - | Output: "text" |

---

### Example 12: "expect" (coda cluster ct)

```
Keystroke sequence: e → x → p → e → c → t → [space]

Step 0: PRE-CHECK → No pattern (e valid VN initial)

Keystroke processing:
├── e → buffer = "e"
├── x → 'x' ngã → buffer = "ẽ"
├── p → buffer = "ẽp"
├── e → buffer = "ẽpe"
├── c → buffer = "ẽpec"
├── t → buffer = "ẽpect"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "ẽpect"
├── Check: Multi-syllable structure invalid
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="ẽpect")? ✓
├── CHECK 2: EN_Pattern(raw="expect")?
│   └── L5: Coda cluster "ct"? ✓
└── Result: ENTER FOREIGN_MODE → keep raw

Final: "expect"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| e | e | e | - | - |
| x | ex | ẽ | - | 'x' = ngã tone |
| p | exp | ẽp | - | - |
| e | expe | ẽpe | - | - |
| c | expec | ẽpec | - | - |
| t | expect | ẽpect | Impossible | FOREIGN_MODE: L5 "ct" ✓ |
| space | expect | expect | - | Output: "expect" |

---

### Example 13: "perfect" (coda cluster ct)

```
Keystroke sequence: p → e → r → f → e → c → t → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── p → buffer = "p"
├── e → buffer = "pe"
├── r → 'r' hỏi → buffer = "pẻ"
├── f → 'f' huyền, but 'e' already has tone, might be ignored or conflict
│       → buffer = "pẻf" (f as literal since tone conflict)
├── e → buffer = "pẻfe"
├── c → buffer = "pẻfec"
├── t → buffer = "pẻfect"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "pẻfect"
├── Check: 'f' not valid VN consonant in medial position
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="pẻfect")? ✓
├── CHECK 2: EN_Pattern(raw="perfect")?
│   └── L5: Coda cluster "ct"? ✓
└── Result: ENTER FOREIGN_MODE → keep raw

Final: "perfect"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| p | p | p | - | - |
| e | pe | pe | - | - |
| r | per | pẻ | - | 'r' = hỏi tone |
| f | perf | pẻf | - | 'f' literal (tone conflict) |
| e | perfe | pẻfe | - | - |
| c | perfec | pẻfec | - | - |
| t | perfect | pẻfect | Impossible | FOREIGN_MODE: L5 "ct" ✓ |
| space | perfect | perfect | - | Output: "perfect" |

---

### Example 14: "sarah" (dictionary lookup - proper name)

```
Keystroke sequence: s → a → r → a → h → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── s → buffer = "s"
├── a → buffer = "sa"
├── r → 'r' hỏi → buffer = "sả"
├── a → buffer = "sảa" (a after toned vowel)
├── h → buffer = "sảah"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "sảah"
├── Check: "ảa" not valid VN vowel combination, 'h' not valid final
└── Result: Impossible

Step 2A: EN PATTERN DETECTION
├── raw = "sarah"
├── Check all layers: No coda cluster, no vowel pattern (ea,ee,oo,ou,ei,eu)
└── Result: No immediate pattern

Step 2B: AUTO_RESTORE (terminator = space)
├── CHECK: Invalid_VN(buffer="sảah")? ✓
├── CHECK: raw="sarah" ∈ Dictionary? ✓ (common name)
└── Result: AUTO_RESTORE → "sarah"

Final: "sarah"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| s | s | s | - | - |
| a | sa | sa | - | - |
| r | sar | sả | - | 'r' = hỏi tone |
| a | sara | sảa | - | - |
| h | sarah | sảah | - | - |
| space | sarah | sảah | Impossible | AUTO_RESTORE: Dict(raw) ✓ |

**Note:** "sarah" has no immediate EN pattern, relies on dictionary lookup at terminator.

---

## Intentional VN Detection & Restore Logic

### VN Intent Signals

```
SIGNAL STRENGTH (từ mạnh → yếu):
├── STROKE (đ)       = 100% intentional VN → NEVER restore
├── TONE KEY applied = Strong intentional  → Don't restore
├── MARK KEY only    = Moderate signal     → Check validity
├── REVERT (ss,ff)   = User cancel tone    → Check dictionary
└── No special key   = Ambiguous           → May restore
```

### Special Keys in Telex

| Type | Keys | Function | Example |
|------|------|----------|---------|
| **STROKE** | `dd` | Đ/đ character | `ddang` → `đang` |
| **TONE** | `s,f,r,x,j` | Sắc/Huyền/Hỏi/Ngã/Nặng | `bas` → `bá` |
| **MARK** | `w,aa,oo,ee,aw,ow,uw` | Breve/Circumflex/Horn | `law` → `lă` |
| **REVERT** | Double tone/mark | Undo transform | `bass` → `bas` |
| **OVERWRITE** | Tone after tone | Replace tone | `banjs` → `bán` |

### Tone Overwrite Case

```
banjs:
b → "b"
a → "ba"
n → "ban"
j → "bạn"  (j = nặng tone applied)
s → "bán"  (s = sắc tone OVERWRITES nặng)

Result: "bán" (last tone wins)
Signal: TONE applied → intentional VN → don't restore
```

### Continuous Typing Case

```
chaofooo:
c  → "c"
h  → "ch"
a  → "cha"
o  → "chao"
f  → "chào"   (f = huyền tone applied to 'a')
o  → "chàoo"  (o appended)
o  → "chàooo"
o  → "chàoooo"

Result: "chàoooo" (invalid VN but tone was used)
Signal: TONE applied → intentional VN → DON'T restore
Reason: User gõ tone key 'f' = họ muốn tiếng Việt
```

### Comprehensive Case Table

| Input | Buffer | Transform | Signal | Buffer State | Raw EN? | Decision | Reason |
|-------|--------|-----------|--------|--------------|---------|----------|--------|
| `bans␣` | bán | tone(s) | TONE | Complete ✓ | no | **KEEP** | Valid VN |
| `banjs␣` | bán | tone(j→s) | TONE | Complete ✓ | no | **KEEP** | Tone overwrite |
| `chaofooo␣` | chàoooo | tone(f) | TONE | Invalid | no | **KEEP** | Tone = intentional |
| `đang␣` | đang | stroke | STROKE | Complete ✓ | no | **KEEP** | Stroke = 100% VN |
| `dder␣` | đẻ | stroke+tone | STROKE | Complete ✓ | no | **KEEP** | Stroke present |
| `lawm␣` | lăm | mark(w) | MARK | Complete ✓ | no | **KEEP** | Valid VN result |
| `law␣` | lă | mark(w) | MARK | Incomplete | yes | **RESTORE** | Incomplete + EN |
| `aw␣` | ă | mark(w) | MARK | Incomplete | no | **KEEP** | No EN match |
| `texs␣` | téx | tone(s) | TONE | Invalid | yes | **KEEP** | Tone = intentional |
| `texts␣` | texts | PRE-CHECK | - | - | yes | **KEEP** | Step 0 skip |
| `hello␣` | hello | none | - | Invalid | no | **KEEP** | No transform |
| `file␣` | file | PRE-CHECK | - | - | yes | **KEEP** | Step 0 skip |
| `bass␣` | bass | revert(ss) | REVERT | Valid | yes | **KEEP** | Revert + Dict |
| `issue␣` | isue | revert(ss) | REVERT | Invalid | yes | **RESTORE** | Revert + EN Dict |
| `coffee␣` | cofee | revert(ff) | REVERT | Invalid | yes | **RESTORE** | Revert + EN Dict |
| `caffe␣` | cafê | revert+mark | REVERT | Invalid | no | **KEEP** | No EN match |

### Restore Logic (Rust)

```rust
struct RestoreContext {
    had_transform: bool,      // buffer != raw
    has_stroke: bool,         // đ present in buffer
    has_tone_applied: bool,   // tone key was used (s,f,r,x,j)
    has_mark_only: bool,      // only mark key used (w,aa,oo...)
    is_revert_case: bool,     // double key caused revert
    buffer_state: BufferState,// Complete/Incomplete/Impossible
    raw_is_valid_en: bool,    // raw in EN dictionary
}

fn should_restore(ctx: &RestoreContext) -> bool {
    // Rule 0: No transform = no restore
    if !ctx.had_transform {
        return false;
    }

    // Rule 1: STROKE (đ) = 100% intentional VN, NEVER restore
    if ctx.has_stroke {
        return false;
    }

    // Rule 2: TONE KEY applied = intentional VN, don't restore
    // (includes tone overwrite case like banjs → bán)
    if ctx.has_tone_applied {
        return false;
    }

    // Rule 3: MARK only + incomplete VN + valid EN = restore
    // (case: "law" → "lă" incomplete, "law" is EN word)
    if ctx.has_mark_only
       && ctx.buffer_state == BufferState::Incomplete
       && ctx.raw_is_valid_en {
        return true;
    }

    // Rule 4: REVERT case + invalid VN + valid EN = restore
    // (case: "issue" → "isue" invalid, "issue" is EN word)
    if ctx.is_revert_case
       && ctx.buffer_state != BufferState::Complete
       && ctx.raw_is_valid_en {
        return true;
    }

    // Default: keep buffer
    false
}
```

### Signal Detection

```rust
fn detect_vn_signals(keystrokes: &[Keystroke]) -> VnSignals {
    let mut signals = VnSignals::default();

    for ks in keystrokes {
        match ks.key {
            // Stroke detection
            'd' if ks.prev == 'd' => signals.has_stroke = true,

            // Tone detection (even if overwritten)
            's' | 'f' | 'r' | 'x' | 'j'
                if ks.is_modifier_context => signals.has_tone_applied = true,

            // Mark detection
            'w' if ks.is_modifier_context => signals.has_mark = true,
            'a' | 'o' | 'e' | 'u'
                if ks.prev == ks.key => signals.has_mark = true,

            _ => {}
        }
    }

    signals.has_mark_only = signals.has_mark && !signals.has_tone_applied;
    signals
}
```

### Decision Flow

```
INPUT: buffer, raw, signals
│
├── had_transform? ─NO──→ KEEP (no change)
│   │
│   YES
│   │
├── has_stroke? ─YES──→ KEEP (đ = 100% VN)
│   │
│   NO
│   │
├── has_tone_applied? ─YES──→ KEEP (intentional VN)
│   │
│   NO
│   │
├── has_mark_only?
│   │
│   YES───→ buffer_incomplete && raw_EN? ─YES──→ RESTORE
│   │                                     │
│   │                                     NO───→ KEEP
│   NO
│   │
├── is_revert_case?
│   │
│   YES───→ buffer_invalid && raw_EN? ─YES──→ RESTORE
│   │                                  │
│   │                                  NO───→ KEEP
│   NO
│   │
└── KEEP (default)
```

---

## Implementation Checklist

**Pattern Detection (on RAW):**
- [ ] `has_english_onset(raw)` - ONSET patterns (đầu từ)
- [ ] `has_english_coda(raw)` - CODA patterns (cuối từ)
- [ ] `has_invalid_vowel_pattern(raw)` - VOWEL patterns (giữa từ)
- [ ] `has_impossible_bigram(raw)` - BIGRAM patterns (bất kỳ)
- [ ] `has_english_pattern(raw)` - Combined check (coda + vowel + bigram)

**Vietnamese Validation (on BUFFER):**
- [ ] `is_valid_vietnamese(buffer)` - Full VN validation
- [ ] `is_impossible_vietnamese(buffer)` - Quick invalid check

**Dictionary:**
- [ ] `is_in_english_dictionary(word)` - HashSet lookup
- [ ] 10K English word dictionary (~100KB)

**Mode Management:**
- [ ] `should_enter_foreign_mode(raw, buffer)` - Step 0 + 2A logic
- [ ] `should_auto_restore(raw, buffer)` - Step 2B logic
- [ ] `is_terminator(key)` - Terminator detection
- [ ] Foreign mode flag in engine state

---

## Memory Budget

| Component | Size |
|-----------|------|
| EN Pattern matrices | ~500 bytes |
| EN Dictionary (10K words) | ~100 KB |
| VN Validation matrices | ~2 KB |
| **Total** | **~103 KB** |

---

## Validation Approach: Layered Bitmask Matrix

### Overview

O(1) lookup validation system using bitmask matrices. Total ~520 bytes memory, 8 validation layers.

```
┌─────────────────────────────────────────────────────────────────┐
│                    VALIDATION PIPELINE                          │
├─────────────────────────────────────────────────────────────────┤
│  Layer 1: CHAR_TYPE[32] → Classify char (onset/vowel/coda/inv) │
│  Layer 2: M_ONSET[32] → Valid single onset bitmask             │
│  Layer 3: M_ONSET_PAIR[32][32] → Valid onset clusters          │
│  Layer 4: M_VOWEL_PAIR[32][32] → Valid diphthongs              │
│  Layer 5: M_CODA[32] → Valid single coda bitmask               │
│  Layer 6: M_CODA_PAIR[32][32] → Valid coda clusters            │
│  Layer 7: M_TONE_CODA[6][8] → Tone-stop restriction            │
│  Layer 8: M_SPELL[32][32] → Spelling rules (c/k, g/gh, ng/ngh) │
└─────────────────────────────────────────────────────────────────┘
```

### Char Type Encoding

Map a-z (26 chars) to 5-bit index (0-25). Special chars: đ=26, ă=27, â=28, ê=29, ô=30, ơ=31, ư=special.

```rust
/// Character type classification
#[repr(u8)]
enum CharType {
    Onset     = 0b0001,  // Valid as onset (b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x)
    Vowel     = 0b0010,  // Valid as vowel (a,ă,â,e,ê,i,o,ô,ơ,u,ư,y)
    Coda      = 0b0100,  // Valid as coda (c,m,n,p,t,i,o,u,y + ch,ng,nh)
    Invalid   = 0b1000,  // Invalid in VN (f,j,w,z)
}

/// 32-byte lookup table
const CHAR_TYPE: [u8; 32] = [
    // a    b    c    d    e    f    g    h    i    j    k    l    m    n    o    p
    0b0010, 0b0001, 0b0101, 0b0001, 0b0010, 0b1000, 0b0001, 0b0001, 0b0110, 0b1000, 0b0001, 0b0001, 0b0101, 0b0101, 0b0110, 0b0101,
    // q    r    s    t    u    v    w    x    y    z    đ    ă    â    ê    ô    ơ
    0b0001, 0b0001, 0b0001, 0b0101, 0b0110, 0b0001, 0b1000, 0b0001, 0b0110, 0b1000, 0b0001, 0b0010, 0b0010, 0b0010, 0b0010, 0b0010,
];
```

### Bitmask Matrices

#### Layer 2: Single Onset (4 bytes)

```rust
/// Valid single onsets: b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x (17 chars)
const M_ONSET: u32 = 0b_0010_0101_1111_1110_1111_1110_0110;
//                      ơôêâăđzyxwvutsrqponmlkjihgfedcba

fn is_valid_onset(c: u8) -> bool {
    (M_ONSET >> c) & 1 == 1
}
```

#### Layer 3: Onset Clusters (128 bytes)

```rust
/// Valid onset pairs: ch,gh,gi,kh,kr,ng,nh,ph,qu,th,tr + ngh
/// 32x32 bit matrix compressed to 32x4 bytes
const M_ONSET_PAIR: [[u8; 4]; 32] = [
    // Each row = first char, bits = second chars that form valid cluster
    // Row 'c': bit 'h' set (ch)
    // Row 'g': bit 'h','i' set (gh, gi)
    // Row 'k': bit 'h','r' set (kh, kr)
    // Row 'n': bit 'g','h' set (ng, nh)
    // Row 'p': bit 'h' set (ph)
    // Row 'q': bit 'u' set (qu)
    // Row 't': bit 'h','r' set (th, tr)
    // ...
];

fn is_valid_onset_cluster(c1: u8, c2: u8) -> bool {
    (M_ONSET_PAIR[c1 as usize][c2 as usize / 8] >> (c2 % 8)) & 1 == 1
}
```

#### Layer 4: Valid Diphthongs (128 bytes)

```rust
/// Valid diphthongs from Pattern Reference Table
/// VN only + Shared = valid, EN only = invalid
const M_VOWEL_PAIR: [[u8; 4]; 32] = [
    // Row 'a': i,o,u,y valid (ai,ao,au,ay)
    // Row 'e': o valid (eo), u requires circumflex (êu)
    // Row 'i': a,ê,u valid (ia,iê,iu)
    // Row 'o': a,ă,e,i valid (oa,oă,oe,oi)
    // Row 'u': a,â,ê,i,ô,ơ,y valid
    // Row 'ư': a,i,ơ,u valid (ưa,ưi,ươ,ưu)
    // Row 'y': ê valid (yê)
    // ...
];

fn is_valid_diphthong(v1: u8, v2: u8) -> bool {
    (M_VOWEL_PAIR[v1 as usize][v2 as usize / 8] >> (v2 % 8)) & 1 == 1
}
```

#### Layer 5: Single Coda (4 bytes)

```rust
/// Valid single codas: c,m,n,p,t + semi-vowels i,o,u,y
const M_CODA: u32 = 0b_0000_0011_0100_1001_0011_0100_0000;
//                     ơôêâăđzyxwvutsrqponmlkjihgfedcba

fn is_valid_coda(c: u8) -> bool {
    (M_CODA >> c) & 1 == 1
}
```

#### Layer 6: Coda Clusters (128 bytes)

```rust
/// Valid coda clusters: ch, ng, nh
const M_CODA_PAIR: [[u8; 4]; 32] = [
    // Row 'c': bit 'h' set (ch)
    // Row 'n': bit 'g','h' set (ng, nh)
    // ...
];

fn is_valid_coda_cluster(c1: u8, c2: u8) -> bool {
    (M_CODA_PAIR[c1 as usize][c2 as usize / 8] >> (c2 % 8)) & 1 == 1
}
```

#### Layer 7: Tone-Stop Restriction (48 bytes)

```rust
/// Tone marks: 0=none, 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
/// Stop codas: 0=c, 1=ch, 2=p, 3=t (encoded as 3 bits)
///
/// Rule: Stop codas (-c,-ch,-p,-t) only allow sắc(1) or nặng(5)
const M_TONE_CODA: [[u8; 8]; 6] = [
    // tone=0 (none): all codas valid
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    // tone=1 (sắc): all codas valid
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    // tone=2 (huyền): stops invalid (c,ch,p,t = 0)
    [0xF0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    // tone=3 (hỏi): stops invalid
    [0xF0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    // tone=4 (ngã): stops invalid
    [0xF0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    // tone=5 (nặng): all codas valid
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
];

fn is_valid_tone_coda(tone: u8, coda: u8) -> bool {
    (M_TONE_CODA[tone as usize][coda as usize / 8] >> (coda % 8)) & 1 == 1
}
```

#### Layer 8: Spelling Rules (128 bytes)

```rust
/// Spelling rules:
/// - c before e,i,y → invalid (use k)
/// - k before a,o,u → invalid (use c)
/// - g before e → invalid (use gh)
/// - ng before e,i → invalid (use ngh)
/// - gh before a,o,u → invalid (use g)
/// - ngh before a,o,u → invalid (use ng)
const M_SPELL: [[u8; 4]; 32] = [
    // Row 'c': bits e,i,y = 0 (invalid combos)
    // Row 'k': bits a,o,u = 0
    // Row 'g': bit e = 0
    // ...
];

fn is_valid_spelling(onset: u8, vowel: u8) -> bool {
    (M_SPELL[onset as usize][vowel as usize / 8] >> (vowel % 8)) & 1 == 1
}
```

### Full Validation Function

```rust
/// O(1) Vietnamese syllable validation
fn validate_syllable(onset: Option<(u8, u8)>, vowels: &[u8], coda: Option<(u8, u8)>, tone: u8) -> bool {
    // Layer 1: Check char types
    for &c in vowels {
        if CHAR_TYPE[c as usize] & CharType::Vowel == 0 {
            return false; // Not a vowel
        }
    }

    // Layer 2-3: Validate onset
    if let Some((c1, c2)) = onset {
        if c2 == 0 {
            if !is_valid_onset(c1) { return false; }
        } else {
            if !is_valid_onset_cluster(c1, c2) { return false; }
        }
    }

    // Layer 4: Validate vowel pattern (diphthong)
    if vowels.len() == 2 {
        if !is_valid_diphthong(vowels[0], vowels[1]) { return false; }
    }

    // Layer 5-6: Validate coda
    if let Some((c1, c2)) = coda {
        if c2 == 0 {
            if !is_valid_coda(c1) { return false; }
        } else {
            if !is_valid_coda_cluster(c1, c2) { return false; }
        }
    }

    // Layer 7: Tone-stop restriction
    if let Some((c1, _)) = coda {
        if !is_valid_tone_coda(tone, c1) { return false; }
    }

    // Layer 8: Spelling rules
    if let Some((o1, o2)) = onset {
        if vowels.len() > 0 {
            let onset_key = if o2 != 0 { o2 } else { o1 };
            if !is_valid_spelling(onset_key, vowels[0]) { return false; }
        }
    }

    true
}
```

### Memory Summary

| Matrix | Size | Purpose |
|--------|------|---------|
| CHAR_TYPE[32] | 32 bytes | Char classification |
| M_ONSET | 4 bytes | Single onset validation |
| M_ONSET_PAIR[32][4] | 128 bytes | Onset cluster validation |
| M_VOWEL_PAIR[32][4] | 128 bytes | Diphthong validation |
| M_CODA | 4 bytes | Single coda validation |
| M_CODA_PAIR[32][4] | 128 bytes | Coda cluster validation |
| M_TONE_CODA[6][8] | 48 bytes | Tone-stop restriction |
| M_SPELL[32][4] | 128 bytes | Spelling rules |
| **Total** | **~600 bytes** | All validation |

### Comparison with OpenKey Approach

| Aspect | OpenKey (Whitelist) | V3 (Bitmask Matrix) |
|--------|---------------------|---------------------|
| **Memory** | ~2KB (vectors) | ~600 bytes |
| **Lookup** | O(n) linear search | O(1) bit lookup |
| **Extensibility** | Add to list | Set bit in matrix |
| **Tone-stop** | Hardcoded if-else | Matrix lookup |
| **Spelling** | Implicit in vowel list | Explicit matrix |
| **Maintenance** | Modify vector data | Modify bit patterns |
