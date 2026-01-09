# Validation Algorithm

> Thuật toán xác định chuỗi ký tự có phải âm tiết tiếng Việt hợp lệ hay không.

**Liên quan**: [vietnamese-language-system.md](./vietnamese-language-system.md) | [core-engine-algorithm.md](./core-engine-algorithm.md)

---

## 1. Mục đích

```
Validation xảy ra TRƯỚC khi transform:

"duoc" + j → VALID   → transform → "được" ✓
"claus" + s → INVALID → giữ nguyên → "clauss" ✓
"http" + s → INVALID → giữ nguyên → "https" ✓
```

Bảo vệ: code (`function`, `const`), tên riêng (`John`, `Claude`), từ mượn (`pizza`), URL/email.

---

## 2. Cấu trúc âm tiết

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

## 3. Data Constants

### 3.1 Phụ âm đầu (C₁)

```rust
// 16 phụ âm đơn
VALID_INITIALS_1: [b, c, d, g, h, k, l, m, n, p, q, r, s, t, v, x]

// 11 phụ âm đôi (kr cho tên dân tộc: Krông)
VALID_INITIALS_2: [ch, gh, gi, kh, kr, ng, nh, ph, qu, th, tr]

// 1 phụ âm ba: ngh
```

### 3.2 Âm cuối (C₂)

```rust
// 10 âm cuối đơn (gồm bán nguyên âm + k cho tên dân tộc)
VALID_FINALS_1: [c, k, m, n, p, t, i, y, o, u]

// 3 âm cuối đôi
VALID_FINALS_2: [ch, ng, nh]
```

> **Lưu ý**: `k` được hỗ trợ cho tên riêng từ ngôn ngữ dân tộc thiểu số (Đắk Lắk, Đắk Nông).

### 3.3 Quy tắc chính tả

| Consonant | Invalid trước | Nên dùng |
|-----------|---------------|----------|
| `c` | e, i, y | → `k` |
| `k` | a, o, u | → `c` |
| `g` | e | → `gh` |
| `ng` | e, i | → `ngh` |
| `gh` | a, o, u | → `g` |
| `ngh` | a, o, u | → `ng` |

### 3.4 Valid Vowel Pairs (Inclusion Approach)

```rust
// Valid vowel combinations in Vietnamese
VALID_VOWEL_PAIRS: [
    // Standard Vietnamese diphthongs (direction matters!)
    [A, I], [A, O], [A, U], [A, Y],  // ai, ao, au, ay
    [E, I], [E, O], [E, U],          // ei (Telex), eo, êu
    [I, A], [I, E], [I, U],          // ia, iê, iu
    [O, A], [O, E], [O, I],          // oa, oe, oi/ôi/ơi
    [U, A], [U, E], [U, I], [U, O], [U, Y], [U, U],  // ua/ưa, uê, ui/ưi, uo/uô/ươ, uy, ưu
    [Y, E],                          // yê
    // Telex intermediate states (for delayed transformations)
    [A, A], [E, E], [O, O],          // aa→â, ee→ê, oo→ô toggle
]
// Note: [O, U] (như "ou" trong "you") KHÔNG có trong list → Invalid
```

**Tại sao dùng Inclusion thay vì Exclusion?**

| Aspect | Inclusion (valid patterns) | Exclusion (invalid patterns) |
|--------|---------------------------|------------------------------|
| Coverage | Toàn diện - bắt tất cả invalid | Chỉ bắt patterns được list |
| Maintenance | Cần thêm Telex intermediate | Dễ miss edge cases |
| Risk | False negative (cần thêm Telex states) | False positive (miss invalid) |

**Lưu ý:** Danh sách bao gồm cả Telex intermediate states như `[E, I]`, `[A, A]`, `[E, E]`, `[O, O]` để hỗ trợ các pattern như "eie" → "êi" và "aaaa" → "aâ".

**Invalid patterns (for reference):**
- `ea` → sea, beach, teacher, search
- `ou` → you, our, house, about, would
- `yo` → yoke, York, your, beyond

---

## 4. Thuật toán Parse

### 4.1 Syllable Structure

```rust
pub struct Syllable {
    pub initial: Vec<usize>,    // indices trong buffer
    pub glide: Option<usize>,   // index của âm đệm
    pub vowel: Vec<usize>,      // indices nguyên âm
    pub final_c: Vec<usize>,    // indices âm cuối
}
```

### 4.2 Parse Algorithm

```
parse(buffer_keys: &[u16]) -> Syllable

1. Tìm vị trí nguyên âm đầu tiên
   - Special case: gi + vowel → gi là initial
   - Special case: qu + vowel → qu là initial

2. Xác định glide (âm đệm)
   - o + (a, e) → o là glide
   - u + (y, e) khi không phải qu → u là glide

3. Thu thập nguyên âm liên tiếp

4. Phần còn lại là final consonant
   - Try 2-char: ch, ng, nh
   - Try 1-char: c, k, m, n, p, t, i, y, o, u
```

---

## 5. Validation Rules

Engine chạy 6 rules tuần tự. Rule đầu tiên fail → trả về lỗi ngay.

```rust
const RULES: &[Rule] = &[
    rule_has_vowel,           // Rule 1
    rule_valid_initial,       // Rule 2
    rule_all_chars_parsed,    // Rule 3
    rule_spelling,            // Rule 4
    rule_valid_final,         // Rule 5
    rule_valid_vowel_pattern, // Rule 6 (NEW)
];
```

### Rule 1: Has Vowel

```rust
// Phải có ít nhất 1 nguyên âm
if syllable.vowel.is_empty() → NoVowel
```

### Rule 2: Valid Initial

```rust
// Phụ âm đầu phải thuộc danh sách hợp lệ
match initial.len() {
    1 → check VALID_INITIALS_1
    2 → check VALID_INITIALS_2
    3 → chỉ cho phép "ngh"
    _ → InvalidInitial
}
```

### Rule 3: All Chars Parsed

```rust
// Mọi ký tự phải được parse vào cấu trúc
parsed_count = initial.len + glide(0|1) + vowel.len + final.len
if parsed_count != buffer.len → InvalidFinal
```

### Rule 4: Spelling

```rust
// Kiểm tra quy tắc chính tả c/k, g/gh, ng/ngh
for (consonant, invalid_vowels) in SPELLING_RULES {
    if initial == consonant && first_vowel in invalid_vowels {
        → InvalidSpelling
    }
}
```

### Rule 5: Valid Final

```rust
// Âm cuối phải thuộc danh sách hợp lệ
match final.len() {
    1 → check VALID_FINALS_1
    2 → check VALID_FINALS_2
    _ → InvalidFinal
}
```

### Rule 6: Valid Vowel Pattern

```rust
// INCLUSION approach: Check vowel pairs phải thuộc VALID_VOWEL_PAIRS
for pair in consecutive_vowel_pairs {
    if pair NOT in VALID_VOWEL_PAIRS → InvalidVowelPattern
}
// Ví dụ: "you" có pair [O,U] → Invalid (không trong list)
// Ví dụ: "sea" có pair [E,A] → Invalid (không trong list)
// Ví dụ: "yeu" có pair [E,U] → Valid (trong list - êu)
// Ví dụ: "eie" có pair [E,I] → Valid (trong list - Telex intermediate)
```

---

## 6. Foreign Word Detection

Ngoài validation, engine còn có hàm `is_foreign_word_pattern()` để detect foreign words:

```rust
pub fn is_foreign_word_pattern(buffer_keys: &[u16], modifier_key: u16) -> bool
```

**Patterns detected:**
1. Invalid vowel patterns (ou, yo) trong buffer
2. Consonant clusters sau finals: T+R, P+R, C+R (metric, spectrum)
3. English prefix: "de" + 's' (describe, design)

**Đặc biệt:** Skip check khi đã có horn transforms (ư, ơ, ươ) → user đang gõ tiếng Việt có chủ đích (vd: "rượu").

---

## 7. API

```rust
/// Validate và trả về kết quả chi tiết
pub fn validate(buffer_keys: &[u16]) -> ValidationResult

/// Quick check
pub fn is_valid(buffer_keys: &[u16]) -> bool

/// Check foreign word pattern (for modifier skipping)
pub fn is_foreign_word_pattern(buffer_keys: &[u16], modifier_key: u16) -> bool

pub enum ValidationResult {
    Valid,
    InvalidInitial,
    InvalidFinal,
    InvalidSpelling,
    InvalidVowelPattern,  // NEW
    NoVowel,
}
```

---

## 8. Test Cases

### Valid

```
ba, ca, an, em, gi, gia, giau, ke, ki, ky
nghe, nghi, nghieng, truong, nguoi, duoc
```

### Invalid - No Vowel

```
bcd, bcdfgh
```

### Invalid - Bad Initial

```
clau, john, bla, string, chrome
```

### Invalid - Spelling

```
ci, ce, cy     → nên dùng ki, ke, ky
ka, ko, ku     → nên dùng ca, co, cu
ngi, nge       → nên dùng nghi, nghe
ge             → nên dùng ghe
```

### Invalid - Foreign Words

```
exp, expect, test, claudeco, claus
```

### Invalid - Vowel Patterns (Inclusion Check)

```
# Pairs NOT in VALID_VOWEL_PAIRS:
search, teacher, beach, real           → "ea" pattern [E,A]
you, your, house, about, would, south  → "ou" pattern [O,U]
yoke, York, beyond                     → "yo" pattern [Y,O]

# Other foreign word patterns (detected by is_foreign_word_pattern):
metric, spectrum, matrix               → T+R, C+R clusters
describe, design                       → "de" + 's' prefix
```

---

## 9. Integration với Engine

```
on_key(key)
│
├─ [is_modifier(key)?]
│  │
│  ├─ ★ VALIDATION: Trước khi transform
│  │   └─ is_valid(buffer)?
│  │       ├─ NO  → return NONE (không transform)
│  │       └─ YES → tiếp tục transform
│  │
│  └─ Apply transformation
│
└─ [is_letter(key)?] → push to buffer
```

---

## 10. Auto-Restore Rules

Ngoài validation (chặn transform), engine còn có auto-restore (khôi phục English khi space):

### 10.1 Invalid Rhyme Patterns

| Rhyme | Valid? | Reason |
|-------|--------|--------|
| `-inh` + tone | ✅ | tính, kính, lính |
| `-ing` + tone | ❌ | thíng, kíng không tồn tại |
| `-ưng` + tone | ✅ | hứng, dựng, bừng |
| `-ung` + tone | ✅ | húng, bùng, cùng |

**Rule:** `-ing` + tone mark → invalid Vietnamese → auto-restore

```
things → thíng → restore "things"
kings  → kíng  → restore "kings"
tính   → tính  → keep Vietnamese ✓
```

### 10.2 Uncommon Single-Vowel Words

| Buffer | Common VN? | Action |
|--------|------------|--------|
| `ò` | ❌ | restore → "of" |
| `ì` | ❌ | restore → "if" |
| `à` | ✅ | keep Vietnamese |
| `ồ` | ✅ | keep Vietnamese |

**Rule:** Single vowel + tone (no final) → check if common Vietnamese interjection

### 10.3 Circumflex Without Final

| Buffer | Real VN word? | Action |
|--------|---------------|--------|
| `sê` | ❌ | restore → "see" |
| `tê` | ⚠️ (rare) | restore → "tee" |
| `bê` | ✅ (calf) | keep Vietnamese |
| `mê` | ✅ (obsessed) | keep Vietnamese |
| `lê` | ✅ (pear) | keep Vietnamese |

**Rule:** C + circumflex (from double vowel) + no final → restore unless common VN word

### 10.4 Double-F Preservation

Khi user gõ double 'f', giữ nguyên cả 2 'f' trong output:

```
off     → of  (bug)  → nên là "off"
offline → ofline     → nên là "offline"
```

**Rule:** Raw input có `ff` → output phải có `ff` (không collapse)

---

## Changelog

- **2025-12-31**: Thêm Auto-Restore Rules section
  - Rule 10.1: `-ing` + tone = invalid Vietnamese
  - Rule 10.2: Uncommon single-vowel words (ò, ì) restore
  - Rule 10.3: Circumflex without final (sê, tê) restore
  - Rule 10.4: Double-f preservation (off, offline)

- **2025-12-17**: Chuyển sang Inclusion approach với VALID_VOWEL_PAIRS
  - Thay đổi từ Exclusion (INVALID_VOWEL_PATTERNS) sang Inclusion (VALID_VOWEL_PAIRS)
  - Thêm ~30 valid vowel pairs dựa trên Vietnamese phonology matrix
  - Thêm Telex intermediate states: [E,I], [A,A], [E,E], [O,O]
  - Fix "search" → "search" (không transform vì "ea" không valid)
  - Fix "teacher", "beach", "real" - tất cả foreign words với "ea"
  - Cập nhật `rule_valid_vowel_pattern` và `is_foreign_word_pattern()`

- **2025-12-16**: Thêm Rule 6 (Vowel Pattern Validation)
  - Thêm `INVALID_VOWEL_PATTERNS` (ou, yo) vào constants
  - Thêm `rule_valid_vowel_pattern` - Rule 6
  - Thêm `is_foreign_word_pattern()` cho foreign word detection
  - Thêm `InvalidVowelPattern` vào ValidationResult
  - Fix issue #15: "metric" không còn bị transform thành "mẻtic"

- **2025-12-11**: Viết lại document theo code thực tế
  - Cập nhật Syllable struct với `Vec<usize>` và `glide` field
  - Chỉnh lại 5 validation rules theo code
  - Loại bỏ pseudo-code sai, thay bằng code snippets chính xác
  - Rút gọn từ ~800 dòng xuống ~200 dòng
