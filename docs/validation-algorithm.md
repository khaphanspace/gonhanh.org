# Validation Algorithm

> Thuật toán validation chuỗi ký tự: Vietnamese syllable validation + English phonotactic detection.

**Liên quan**: [vietnamese-language-system.md](./vietnamese-language-system.md) | [core-engine-algorithm.md](./core-engine-algorithm.md) | [auto-restore-behavior.md](./auto-restore-behavior.md)

---

## 1. Mục đích

```
Validation xảy ra TRƯỚC khi transform:

"duoc" + j → VN VALID   → transform → "được" ✓
"claus" + s → VN INVALID → giữ nguyên → "clauss" ✓
"http" + s → VN INVALID → giữ nguyên → "https" ✓

Auto-restore xảy ra SAU khi gõ space:

"thíng " → VN INVALID + EN VALID → restore "things "
"lă " → VN INVALID + EN VALID → restore "law "
```

Bảo vệ: code (`function`, `const`), tên riêng (`John`, `Claude`), từ mượn (`pizza`), URL/email.

---

## 2. Validation Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      VALIDATION SYSTEM                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────┐    ┌─────────────────────┐            │
│  │   VN VALIDATION     │    │   EN DETECTION      │            │
│  │   (6 rules)         │    │   (8 layers)        │            │
│  │                     │    │                     │            │
│  │  1. has_vowel       │    │  1. invalid_vn_init │            │
│  │  2. valid_initial   │    │  2. onset_clusters  │            │
│  │  3. all_parsed      │    │  3. double_consonant│            │
│  │  4. spelling        │    │  4. suffix          │            │
│  │  5. valid_final     │    │  5. coda_clusters   │            │
│  │  6. vowel_pattern   │    │  6. prefix          │            │
│  │                     │    │  7. invalid_vowels  │            │
│  │                     │    │  8. impossible_bigram│            │
│  └─────────────────────┘    └─────────────────────┘            │
│                                                                 │
│  Decision: VN_VALID && !EN_PATTERN → Keep Vietnamese           │
│            VN_INVALID && EN_VALID → Auto-restore English       │
│            VN_VALID && EN_PATTERN → Context-dependent          │
└─────────────────────────────────────────────────────────────────┘
```

---

# PART A: VIETNAMESE VALIDATION

## 3. Cấu trúc âm tiết

```
Syllable = (C₁)(G)V(C₂)

C₁ = Initial consonant (phụ âm đầu)  - optional
G  = Glide (âm đệm: o, u)            - optional
V  = Vowel nucleus (nguyên âm)       - REQUIRED
C₂ = Final consonant (âm cuối)       - optional
```

**Ví dụ:**
| Input | C₁ | G | V | C₂ |
|-------|------|-----|------|------|
| `a` | - | - | a | - |
| `ban` | b | - | a | n |
| `hoa` | h | o | a | - |
| `qua` | qu | - | a | - |
| `giau` | gi | - | au | - |
| `nghieng` | ngh | - | ie | ng |
| `duoc` | d | - | uo | c |

---

## 4. Vietnamese Data Constants

### 4.1 Phụ âm đầu (C₁) - 28 patterns

```rust
// 16 phụ âm đơn
VALID_INITIALS_1: [b, c, d, g, h, k, l, m, n, p, q, r, s, t, v, x]

// 11 phụ âm đôi (kr cho tên dân tộc: Krông)
VALID_INITIALS_2: [ch, gh, gi, kh, kr, ng, nh, ph, qu, th, tr]

// 1 phụ âm ba
VALID_INITIALS_3: [ngh]
```

**Total: 16 + 11 + 1 = 28 valid initials**

### 4.2 Âm cuối (C₂) - 13 patterns

```rust
// 10 âm cuối đơn (gồm bán nguyên âm + k cho tên dân tộc)
VALID_FINALS_1: [c, k, m, n, p, t, i, y, o, u]

// 3 âm cuối đôi
VALID_FINALS_2: [ch, ng, nh]
```

**Total: 10 + 3 = 13 valid finals**

> **Lưu ý**: `k` được hỗ trợ cho tên riêng từ ngôn ngữ dân tộc thiểu số (Đắk Lắk, Đắk Nông).

### 4.3 Quy tắc chính tả

| Consonant | Invalid trước | Nên dùng |
|-----------|---------------|----------|
| `c` | e, i, y | → `k` |
| `k` | a, o, u | → `c` |
| `g` | e | → `gh` |
| `ng` | e, i | → `ngh` |
| `gh` | a, o, u | → `g` |
| `ngh` | a, o, u | → `ng` |

### 4.4 Valid Vowel Pairs (Whitelist Approach)

```rust
// 29 diphthongs (nguyên âm đôi)
VALID_DIPHTHONGS: [
    [A, I], [A, O], [A, U], [A, Y],  // ai, ao, au, ay
    [E, O], [E, U],                   // eo, êu
    [I, A], [I, E], [I, U],          // ia, iê, iu
    [O, A], [O, E], [O, I],          // oa, oe, oi/ôi/ơi
    [U, A], [U, E], [U, I], [U, O], [U, Y], [U, U],  // ua/ưa, uê, ui/ưi, uô/ươ, uy, ưu
    [Y, E],                          // yê
]

// 13 triphthongs (nguyên âm ba)
VALID_TRIPHTHONGS: [
    [I, E, U], [Y, E, U],            // iêu, yêu
    [O, A, I], [O, A, Y], [O, E, O], [O, A, O],  // oai, oay, oeo, oao
    [U, A, Y], [U, O, I], [U, Y, A], [U, O, U],  // uây, uôi/ươi, uya, ươu
    [U, Y, E], [U, Y, U], [U, E, U], // uyê, uyu, uêu
]
```

**Tại sao dùng Whitelist thay vì Blacklist?**

| Aspect | Whitelist (valid patterns) | Blacklist (invalid patterns) |
|--------|---------------------------|------------------------------|
| Coverage | Toàn diện - bắt TẤT CẢ invalid | Chỉ bắt patterns được list |
| Maintenance | Thêm pattern mới = thêm vào list | Dễ miss edge cases |
| False positive | Thấp | Cao |

**Invalid patterns (auto-detected bởi whitelist):**
- `ea` → search, beach, teacher (không trong list)
- `ou` → you, our, house, about (không trong list)
- `yo` → yoke, York, your (không trong list)

---

## 5. Vietnamese Validation Rules

Engine chạy 6 rules tuần tự. Rule đầu tiên fail → trả về lỗi ngay.

```rust
const VN_RULES: &[Rule] = &[
    vn::rule_has_vowel,           // Rule 1
    vn::rule_valid_initial,       // Rule 2
    vn::rule_all_chars_parsed,    // Rule 3
    vn::rule_spelling,            // Rule 4
    vn::rule_valid_final,         // Rule 5
    vn::rule_valid_vowel_pattern, // Rule 6
];
```

### Rule 1: vn::has_vowel

```rust
// Phải có ít nhất 1 nguyên âm
if syllable.vowel.is_empty() → NoVowel
```

### Rule 2: vn::valid_initial

```rust
// Phụ âm đầu phải thuộc danh sách hợp lệ
match initial.len() {
    0 → VALID (no initial is OK)
    1 → check VALID_INITIALS_1
    2 → check VALID_INITIALS_2
    3 → chỉ cho phép "ngh"
    _ → InvalidInitial
}
```

### Rule 3: vn::all_chars_parsed

```rust
// Mọi ký tự phải được parse vào cấu trúc
parsed_count = initial.len + glide(0|1) + vowel.len + final.len
if parsed_count != buffer.len → InvalidFinal
```

### Rule 4: vn::spelling

```rust
// Kiểm tra quy tắc chính tả c/k, g/gh, ng/ngh
for (consonant, invalid_vowels) in SPELLING_RULES {
    if initial == consonant && first_vowel in invalid_vowels {
        → InvalidSpelling
    }
}
```

### Rule 5: vn::valid_final

```rust
// Âm cuối phải thuộc danh sách hợp lệ
match final.len() {
    0 → VALID (no final is OK)
    1 → check VALID_FINALS_1
    2 → check VALID_FINALS_2
    _ → InvalidFinal
}
```

### Rule 6: vn::valid_vowel_pattern

```rust
// WHITELIST approach: Check vowel patterns
match vowel.len() {
    1 → VALID (single vowel always OK)
    2 → check VALID_DIPHTHONGS + modifier requirements
    3 → check VALID_TRIPHTHONGS + modifier requirements
    _ → InvalidVowelPattern (>3 vowels invalid)
}
```

---

# PART B: ENGLISH PHONOTACTIC DETECTION

## 6. English Detection Overview

```
PURPOSE: Detect English words for auto-restore
TARGET: 98% accuracy on top 20,000 English words
SPEED: <0.1ms per check
MEMORY: ~1KB lookup tables
```

## 7. English Detection Layers (8 layers)

### Layer 1: Invalid Vietnamese Initials (100% confidence)

**Purpose**: Detect consonant clusters impossible in Vietnamese at word start.

```rust
// These patterns NEVER appear at start of Vietnamese words
EN_INVALID_VN_INITIALS_2: [
    // pr, cl, gr, cr, fl, sh, kn, dr, etc.
    [P, R], [C, L], [G, R], [C, R], [F, L], [F, R],
    [S, H], [K, N], [D, R], [B, L], [B, R], [G, L],
    [P, L], [S, C], [S, K], [S, L], [S, M], [S, N],
    [S, P], [S, T], [S, W], [T, W], [W, H], [W, R],
]

EN_INVALID_VN_INITIALS_3: [
    // str, spl, spr, scr, shr, thr, squ, etc.
    [S, T, R], [S, P, L], [S, P, R], [S, C, R],
    [S, H, R], [T, H, R], [S, Q, U], [S, C, H],
]
```

**Coverage**: ~15-20% of English words
**Examples**: press, class, stress, string, flow, show

### Layer 2: Onset Clusters (98% confidence)

**Purpose**: Detect valid English onset clusters that Vietnamese lacks.

| Pattern | Examples | Coverage |
|---------|----------|----------|
| `str-` | strong, street, string | ~0.8% |
| `spl-` | split, splash | ~0.3% |
| `scr-` | script, screen | ~0.2% |
| `spr-` | spray, spring | ~0.2% |
| `shr-` | shrink, shrug | ~0.1% |
| `fl-` | flow, flag, flower | ~0.5% |
| `fr-` | from, free, friend | ~0.8% |
| `br-` | break, bring, bright | ~0.5% |
| `tr-` | tree, try, true | ~0.8% |
| `cl-` | close, class, clear | ~0.6% |
| `cr-` | create, cream, cross | ~0.3% |
| `dr-` | draw, dream, drink | ~0.5% |
| `pl-` | place, plan, play | ~0.6% |
| `pr-` | present, press | ~0.8% |
| `gr-` | great, grow, green | ~0.7% |
| `bl-` | blood, blue, black | ~0.4% |
| `gl-` | glass, glad, global | ~0.4% |
| `sm-` | small, smart, smile | ~0.3% |
| `sn-` | snow, snare | ~0.1% |
| `sp-` | space, speak, speed | ~0.6% |
| `st-` | stop, story, start | ~1.2% |

**Total coverage**: ~11-14% of English words

### Layer 3: Double Consonants (95% confidence)

**Purpose**: Detect doubled consonants impossible in Vietnamese.

```rust
// Vietnamese NEVER uses doubled consonants (uses digraphs instead)
EN_DOUBLE_CONSONANTS: [
    LL, SS, FF, TT, PP, MM, NN, RR, DD, GG, BB, ZZ, CC
]
```

| Double | Coverage | Examples |
|--------|----------|----------|
| `-ll-` | 0.8% | tell, all, will, small |
| `-ss-` | 0.6% | class, glass, pass, miss |
| `-tt-` | 0.5% | letter, better, sitting |
| `-ff-` | 0.4% | off, staff, coffee, office |
| `-pp-` | 0.3% | happy, apple, supper |
| `-mm-` | 0.3% | summer, hammer, swimming |
| `-nn-` | 0.3% | dinner, funny, manner |
| `-rr-` | 0.1% | sorry, worry, mirror |
| `-dd-` | 0.1% | add, ladder, middle |
| `-gg-` | 0.1% | egg, bigger, suggest |
| `-bb-` | 0.1% | rabbit, hobby, ribbon |
| `-zz-` | 0.05% | buzz, pizza, puzzle |
| `-cc-` | 0.3% | access, success, account |

**Total coverage**: ~4-5% of words

### Layer 4: English Suffixes (90% confidence)

**Purpose**: Detect morphological patterns impossible in Vietnamese (isolating language).

```rust
EN_SUFFIXES: [
    "-ing",  // 12% - gerunds, present participle
    "-ed",   // 8% - past tense
    "-er",   // 6% - comparatives, agent nouns
    "-ly",   // 4% - adverbs
    "-tion", // 3% - nominalization
    "-ness", // 2.5% - abstract nouns
    "-ment", // 2% - nominalization
    "-able", // 2% - adjectives
    "-ous",  // 1.5% - adjectives
]
```

**Special case**: `-ing` + tone mark = 96% confidence (gerund + diacritic impossible in Vietnamese)

**Total coverage**: ~40-45% of words

### Layer 5: Complex Codas (90% confidence)

**Purpose**: Detect consonant clusters at word end impossible in Vietnamese.

```rust
// Vietnamese allows max 2 consonants at end; English has up to 5
EN_COMPLEX_CODAS: [
    "-st", "-nd", "-nt", "-ld", "-nk", "-ng",
    "-mp", "-lt", "-ft", "-pt", "-ct", "-xt",
]
```

| Coda | Coverage | Examples |
|------|----------|----------|
| `-st` | 3-4% | first, last, best, most |
| `-nd` | 2% | land, kind, find, hand |
| `-nt` | 1.5% | want, plant, point |
| `-ng` | 1.2% | sing, thing, bring |
| `-ld` | 0.5% | hold, world, cold |
| `-nk` | 0.3% | thank, think, bank |
| `-mp` | 0.3% | jump, camp, stamp |
| `-lt` | 0.2% | salt, belt, result |
| `-ft` | 0.2% | left, soft, gift |
| `-pt` | 0.2% | kept, concept |
| `-ct` | 0.2% | fact, act, effect |
| `-xt` | 0.2% | next, text, context |

**Total coverage**: ~15-20% of words

### Layer 6: English Prefixes (75% confidence)

**Purpose**: Detect morphological prefixes impossible in Vietnamese.

```rust
EN_PREFIXES: [
    "un-",  // 3.5% - unhappy, unclear
    "re-",  // 2.5% - repeat, rebuild
    "in-/im-/il-/ir-", // 2% - impossible, illegal
    "dis-", // 1.5% - disagree, discover
    "pre-", // 1.2% - prepare, prevent
]
```

**Total coverage**: ~12-14% of words

### Layer 7: Invalid Vowel Patterns (85% confidence)

**Purpose**: Detect vowel combinations not in Vietnamese phonotactics.

```rust
EN_INVALID_VOWELS: [
    "ea",  // search, beach, teacher
    "ou",  // you, our, house
    "yo",  // yoke, York, your
    "oo",  // book, look, food (without circumflex)
]
```

**Note**: These overlap with VN validation Rule 6 but provide English-specific confidence.

### Layer 8: Impossible Bigrams (80% confidence)

**Purpose**: Detect letter pairs that NEVER appear in valid English words.

```rust
// 80+ impossible two-letter combinations in English
EN_IMPOSSIBLE_BIGRAMS: [
    "bq", "bx", "bz", "cf", "cg", "cj", "cp", "cv", "cw", "cx",
    "dx", "fq", "fv", "fx", "fz", "gv", "gx", "hx", "hz",
    "jb", "jc", "jd", "jf", "jg", "jh", "jj", "jl", "jm", "jn",
    "jp", "jq", "jt", "jv", "jw", "jx", "jy", "jz",
    "kq", "kv", "kx", "kz", "lx",
    "mg", "mj", "mx", "mz",
    "pq", "pv", "px",
    "qc", "qd", "qe", "qf", "qg", "qh", "qj", "qk", "ql", "qm",
    "qn", "qo", "qp", "qq", "qr", "qs", "qt", "qv", "qw", "qx", "qy", "qz",
    "sx", "tq", "tx",
    "vb", "vc", "vf", "vj", "vk", "vm", "vp", "vq", "vw", "vx", "vz",
    "wj", "wq", "wv", "wx",
    "xd", "xj", "xk", "xm", "xr",
    "yq",
    "zf", "zj", "zx",
]
```

**If ANY of these appear → NOT valid English word**

---

## 8. Confidence Scoring Algorithm

```rust
fn calculate_english_confidence(buffer: &str, raw: &str) -> u32 {
    let mut score = 0;

    // Layer 1: Invalid VN initial (highest priority)
    if has_invalid_vn_initial(buffer) { score += 50; }

    // Layer 2: Onset clusters
    if has_onset_cluster(buffer) { score += 40; }

    // Layer 3: Double consonants
    if has_double_consonant(buffer) { score += 20; }

    // Layer 4: Suffixes
    if has_english_suffix(buffer) { score += 15; }
    if has_suffix_ing_with_tone(buffer) { score += 20; } // bonus

    // Layer 5: Complex codas
    if has_complex_coda(buffer) { score += 15; }

    // Layer 6: Prefixes
    if has_english_prefix(buffer) { score += 10; }

    // Layer 7: Invalid vowel patterns
    if has_invalid_vowel_pattern(buffer) { score += 10; }

    // Layer 8: Impossible bigrams (negative = not English)
    if has_impossible_bigram(raw) { score = 0; }

    score
}

// Decision thresholds
THRESHOLD_CERTAIN: 50      // Auto-restore immediately
THRESHOLD_VERY_HIGH: 35    // Restore if no stroke (đ)
THRESHOLD_HIGH: 25         // Restore if buffer invalid VN
THRESHOLD_MEDIUM: 15       // Check context + validation
```

---

## 9. Auto-Restore Decision Flow

```
on_space():
    if !had_transform:
        return PASSTHROUGH  // No transform = no restore needed

    // Step 1: Check if buffer is valid Vietnamese
    vn_valid = vn::is_valid(buffer)

    // Step 2: Check English confidence
    en_score = calculate_english_confidence(buffer, raw)

    // Step 3: Decision
    if vn_valid && !has_stroke:
        if en_score >= THRESHOLD_VERY_HIGH:
            return RESTORE  // High English confidence overrides
        else:
            return KEEP_VN  // Valid VN, keep it

    if !vn_valid:
        if is_valid_english(raw):
            return RESTORE  // Invalid VN + valid EN = restore
        else:
            return KEEP_AS_IS  // Invalid both = keep buffer

    return KEEP_VN  // Default: keep Vietnamese
```

---

## 10. API

```rust
// Vietnamese validation
pub fn vn::validate(buffer_keys: &[u16]) -> ValidationResult
pub fn vn::is_valid(buffer_keys: &[u16]) -> bool
pub fn vn::is_valid_with_tones(keys: &[u16], tones: &[u8]) -> bool
pub fn vn::is_valid_for_transform(buffer_keys: &[u16]) -> bool

// English detection
pub fn en::calculate_confidence(buffer: &str, raw: &str) -> u32
pub fn en::has_english_pattern(buffer: &str) -> bool
pub fn en::is_valid_english_word(raw: &str) -> bool

// Combined
pub fn is_foreign_word_pattern(buffer_keys: &[u16], modifier_key: u16) -> bool
pub fn should_auto_restore(buffer: &Buffer, raw: &str) -> bool

pub enum ValidationResult {
    Valid,
    InvalidInitial,
    InvalidFinal,
    InvalidSpelling,
    InvalidVowelPattern,
    NoVowel,
}
```

---

## 11. Test Cases

### 11.1 Vietnamese Valid

```
ba, ca, an, em, gi, gia, giau, ke, ki, ky
nghe, nghi, nghieng, truong, nguoi, duoc
tính, được, việt, hoàng, phương
```

### 11.2 Vietnamese Invalid - By Rule

| Input | Rule Failed | Reason |
|-------|-------------|--------|
| `bcd` | No Vowel | consonants only |
| `clau` | Invalid Initial | cl not VN |
| `john` | Invalid Initial | j not VN |
| `ci` | Spelling | c before i |
| `nge` | Spelling | ng before e |
| `search` | Vowel Pattern | ea not in list |
| `you` | Vowel Pattern | ou not in list |

### 11.3 English Detection - By Layer

| Input | Layer | Confidence | Action |
|-------|-------|------------|--------|
| `press` | Invalid VN Init (pr) | 50 | RESTORE |
| `stress` | Onset Cluster (str) | 40+50=90 | RESTORE |
| `class` | Invalid Init (cl) + Double (ss) | 50+20=70 | RESTORE |
| `coffee` | Double (ff) | 20 | RESTORE if invalid VN |
| `running` | Suffix (-ing) | 15 | RESTORE if invalid VN |
| `understand` | Coda (-nd) | 15 | RESTORE if invalid VN |
| `unhappy` | Prefix (un-) | 10 | RESTORE if invalid VN |

### 11.4 Ambiguous Cases

| Buffer | VN Valid? | EN Score | Action |
|--------|-----------|----------|--------|
| `con` | YES | 0 | KEEP_VN |
| `an` | YES | 0 | KEEP_VN |
| `để` | YES (stroke) | N/A | KEEP_VN |
| `dất` | YES | 0 | KEEP_VN (datas needs context) |

---

## 12. Integration với Engine

```
on_key(key)
│
├─ [is_modifier(key)?]
│  │
│  ├─ ★ PRE-VALIDATION: Trước khi transform
│  │   └─ vn::is_valid_for_transform(buffer)?
│  │       ├─ NO  → return NONE (không transform)
│  │       └─ YES → tiếp tục transform
│  │
│  └─ Apply transformation
│
├─ [is_letter(key)?] → push to buffer
│
└─ [is_space(key)?]
    └─ ★ AUTO-RESTORE:
        └─ should_auto_restore(buffer, raw)?
            ├─ YES → output raw (English)
            └─ NO  → output buffer (Vietnamese)
```

---

## 13. Performance Budget

| Operation | Target | Notes |
|-----------|--------|-------|
| VN Validation | <0.05ms | 6 rules, no allocation |
| EN Detection | <0.1ms | 8 layers, lookup tables |
| Auto-restore decision | <0.1ms | Combined VN+EN check |
| Memory (constants) | <2KB | All lookup tables |

---

## Changelog

- **2026-01-03**: Major rewrite - comprehensive VI + EN rules
  - Added Part B: English Phonotactic Detection (8 layers)
  - Added confidence scoring algorithm
  - Added auto-restore decision flow
  - Added test cases by layer
  - Restructured document for clarity

- **2025-12-31**: Thêm Auto-Restore Rules section
  - Rule 10.1: `-ing` + tone = invalid Vietnamese
  - Rule 10.2: Uncommon single-vowel words restore
  - Rule 10.3: Circumflex without final restore
  - Rule 10.4: Double-f preservation

- **2025-12-17**: Chuyển sang Whitelist approach với VALID_VOWEL_PAIRS

- **2025-12-16**: Thêm Rule 6 (Vowel Pattern Validation)

- **2025-12-11**: Viết lại document theo code thực tế
