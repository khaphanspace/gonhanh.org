# GoNhanh Core Typing Engine

> Tài liệu thuật toán cho engine gõ tiếng Việt.

**Tài liệu liên quan**:
- [vietnamese-language-system.md](./vietnamese-language-system.md) - Hệ thống chữ viết tiếng Việt & Quy tắc âm vị học
- [validation-algorithm.md](./validation-algorithm.md) - Chi tiết 5 quy tắc validation
- [system-architecture.md](./system-architecture.md) - Kiến trúc tổng thể

---

## 1. NGUYÊN TẮC THIẾT KẾ

### 1.1 Core Principles

```
NGUYÊN TẮC:
│
├── 1. VALIDATION FIRST (★ QUAN TRỌNG NHẤT)
│   └── Khi detect modifier → VALIDATE buffer có phải tiếng Việt không?
│       ├── "nghieng" hợp lệ? → YES → cho phép transform
│       ├── "claus" hợp lệ? → NO → không transform
│       └── Nếu INVALID → không làm gì, thêm key vào buffer bình thường
│
├── 2. PATTERN-BASED REPLACEMENT
│   └── Nếu VALID → scan TOÀN BỘ buffer → apply pattern
│       ├── Không case-by-case (prev + current)
│       └── Scan toàn bộ để tìm pattern
│
├── 3. LONGEST-MATCH-FIRST
│   └── Cho vị trí đặt dấu và matching finals
│       ├── "nghieng" → tìm "ieng" → đặt dấu đúng
│       └── "ch", "ng", "nh" match trước "c", "n"
│
└── 4. DOUBLE-KEY REVERT
    └── Nhấn cùng phím 2 lần → revert transformation
        ├── "aa" → "â", "aa" → "aa" (revert)
        └── "ss" → "á", "ss" → "as" (revert)
```

---

## 2. 7-STAGE PIPELINE

### 2.1 Main Processing Flow

```
on_key_ext(key, caps, ctrl, shift) → Result
│
├─► [!enabled || ctrl?] ──► clear buffer ──► return NONE
│
├─► [is_break(key)?] ──► check shortcuts ──► clear buffer ──► return
│
├─► [key == DELETE?] ──► pop buffer ──► return NONE
│
└─► process(key, caps, shift)
    │
    ├── STAGE 1: Stroke (d → đ)
    │   └── try_stroke() - scan buffer for un-stroked 'd'
    │
    ├── STAGE 2: Tone (circumflex/horn/breve)
    │   └── try_tone() - apply aa→â, ow→ơ, aw→ă patterns
    │
    ├── STAGE 3: Mark (sắc/huyền/hỏi/ngã/nặng)
    │   └── try_mark() - find vowel position, apply mark
    │
    ├── STAGE 4: Remove (z/0)
    │   └── handle_remove() - clear mark or tone
    │
    ├── STAGE 5: W-Vowel (Telex only)
    │   └── try_w_as_vowel() - "w" → "ư" với validation
    │
    ├── STAGE 6: Normal Letter
    │   └── handle_normal_letter() - push to buffer
    │
    └── STAGE 7: Word Boundary Shortcut
        └── try_word_boundary_shortcut() - expand abbreviations

Ref: core/src/engine/mod.rs:179-229
```

### 2.2 Result Structure

```rust
/// FFI Result - 36 bytes
#[repr(C)]
pub struct Result {
    pub chars: [u32; 32],  // UTF-32 codepoints
    pub action: u8,        // 0=None, 1=Send, 2=Restore
    pub backspace: u8,     // Characters to delete
    pub count: u8,         // Valid chars count
    pub _pad: u8,          // Alignment padding
}

Ref: core/src/engine/mod.rs:40-47
```

---

## 3. MODIFIER DETECTION

### 3.1 Telex Modifiers

```
TELEX:
├── TONE_MODIFIERS (dấu phụ):
│   ├── 'a' → aa (â - circumflex)
│   ├── 'e' → ee (ê - circumflex)
│   ├── 'o' → oo (ô - circumflex)
│   ├── 'w' → horn (ơ, ư) hoặc breve (ă)
│   └── 'd' → dd (đ - stroke)
│
├── MARK_MODIFIERS (dấu thanh):
│   ├── 's' → sắc (1)
│   ├── 'f' → huyền (2)
│   ├── 'r' → hỏi (3)
│   ├── 'x' → ngã (4)
│   └── 'j' → nặng (5)
│
└── REMOVE_MODIFIER:
    └── 'z' → xóa dấu

Ref: core/src/input/telex.rs
```

### 3.2 VNI Modifiers

```
VNI:
├── TONE_MODIFIERS:
│   ├── '6' → circumflex (â, ê, ô)
│   ├── '7' → horn (ơ, ư)
│   ├── '8' → breve (ă)
│   └── '9' → stroke (đ)
│
├── MARK_MODIFIERS:
│   ├── '1' → sắc
│   ├── '2' → huyền
│   ├── '3' → hỏi
│   ├── '4' → ngã
│   └── '5' → nặng
│
└── REMOVE_MODIFIER:
    └── '0' → xóa dấu

Ref: core/src/input/vni.rs
```

---

## 4. SYLLABLE PARSING

### 4.1 Vietnamese Syllable Structure

```
CẤU TRÚC ÂM TIẾT:
│
│   Syllable = (C₁)(G)V(C₂)
│
├── C₁ = Phụ âm đầu (Initial) - TÙY CHỌN
│   ├── Đơn: b, c, d, g, h, k, l, m, n, p, q, r, s, t, v, x (16)
│   ├── Đôi: ch, gh, gi, kh, ng, nh, ph, qu, th, tr (10)
│   └── Ba: ngh (1)
│
├── G = Âm đệm (Glide) - TÙY CHỌN
│   └── o (oa, oe), u (uy, ue)
│
├── V = Nguyên âm chính (Vowel) - BẮT BUỘC
│   └── a, ă, â, e, ê, i, o, ô, ơ, u, ư, y (12)
│
└── C₂ = Âm cuối (Final) - TÙY CHỌN
    ├── Phụ âm: c, m, n, p, t (5)
    ├── Đôi: ch, ng, nh (3)
    └── Bán nguyên âm: i, y, o, u (4)

Ref: core/src/engine/syllable.rs, core/src/data/constants.rs
```

### 4.2 Parse Algorithm

```
parse(buffer_keys) → Syllable { initial, glide, vowel, final_c }
│
├── STEP 1: Find first vowel position
│   ├── Special: "gi" + vowel → gi is initial
│   └── Special: "qu" + vowel → qu is initial
│
├── STEP 2: Identify glide
│   ├── o + (a, e) → glide
│   └── u + (y, e) when not after "qu" → glide
│
├── STEP 3: Identify vowel nucleus
│   └── Consecutive vowels after glide
│
└── STEP 4: Match final consonant (longest-first)
    ├── 2 chars: ch, ng, nh
    └── 1 char: c, m, n, p, t, i, y, o, u

Ref: core/src/engine/syllable.rs:50-159
```

### 4.3 Parse Examples

```
VÍ DỤ PARSE:

"nghieng":
├── initial = [0,1,2] → "ngh" (3 chars)
├── glide = None
├── vowel = [3,4] → "ie" (2 chars)
├── final_c = [5,6] → "ng" (2 chars)
└── Result: valid ✓

"hoa":
├── initial = [0] → "h"
├── glide = Some(1) → "o"
├── vowel = [2] → "a"
├── final_c = []
└── Result: valid ✓

"qua":
├── initial = [0,1] → "qu" (u thuộc initial)
├── glide = None
├── vowel = [2] → "a"
├── final_c = []
└── Result: valid ✓

"giau":
├── initial = [0,1] → "gi" (i thuộc initial vì sau có vowel)
├── glide = None
├── vowel = [2,3] → "au"
├── final_c = []
└── Result: valid ✓
```

---

## 5. VALIDATION RULES

### 5.1 Five Validation Rules

```
RULES: (chạy theo thứ tự)
│
├── Rule 1: Must have vowel
│   └── syllable.vowel.is_empty()? → InvalidNoVowel
│
├── Rule 2: Valid initial consonant
│   └── initial ∈ VALID_INITIALS? (16 single + 10 double + ngh)
│
├── Rule 3: All chars parsed
│   └── initial.len + glide.len + vowel.len + final.len == buffer.len
│
├── Rule 4: Spelling rules
│   ├── c + (e,i,y) → INVALID (dùng k)
│   ├── k + (a,o,u) → INVALID (dùng c)
│   ├── g + (e) → INVALID (dùng gh)
│   ├── ng + (e,i) → INVALID (dùng ngh)
│   ├── gh + (a,o,u) → INVALID (dùng g)
│   └── ngh + (a,o,u) → INVALID (dùng ng)
│
└── Rule 5: Valid final consonant
    └── final ∈ VALID_FINALS? (c,m,n,p,t + ch,ng,nh + semi-vowels)

Ref: core/src/engine/validation.rs:34-40
```

### 5.2 Validation Examples

```
VALIDATION EXAMPLES:

"duoc" → VALID ✓
├── initial = "d" ✓
├── vowel = "uo" ✓
└── final = "c" ✓

"clau" → INVALID ✗
└── initial = "cl" ∉ VALID_INITIALS

"john" → INVALID ✗
└── initial = "j" ∉ VALID_INITIALS

"http" → INVALID ✗
└── No vowel found

"ci" → INVALID ✗
└── Spelling rule: c + i → phải dùng k
```

---

## 6. TRANSFORMATION

### 6.1 Stroke Transformation (d → đ)

```
apply_stroke(buffer) → TransformResult
│
├── Scan buffer for 'd' with stroke = false
│   └── Found at any position → mark stroke = true
│
├── Example: "Dod"
│   ├── buffer = ['D', 'o', 'd']
│   ├── First 'd' at position 0
│   ├── Mark D.stroke = true
│   └── Result: "Đo" (remove trigger 'd')
│
└── Double-key revert: "Đo" + 'd' → "Dod"

Ref: core/src/engine/transform.rs:224-234
```

### 6.2 Tone Transformation (circumflex/horn/breve)

```
apply_tone(buffer, key, tone_value, method) → TransformResult
│
├── Find target vowels based on key and method
│
├── Telex patterns:
│   ├── aa → find 'a' → tone = CIRCUMFLEX
│   ├── ee → find 'e' → tone = CIRCUMFLEX
│   ├── oo → find 'o' → tone = CIRCUMFLEX
│   └── w → find a/o/u → tone = HORN
│
├── VNI patterns:
│   ├── 6 → find a/e/o → tone = CIRCUMFLEX
│   ├── 7 → find o/u → tone = HORN
│   └── 8 → find a → tone = HORN (breve)
│
└── UO COMPOUND SPECIAL:
    ├── Buffer có "uo" hoặc "ou" adjacent
    ├── Apply HORN to BOTH
    └── Example: "duoc" + 'w' → "dươc"

Ref: core/src/engine/transform.rs:59-87
```

### 6.3 Mark Transformation (sắc/huyền/hỏi/ngã/nặng)

```
apply_mark(buffer, mark_value, modern) → TransformResult
│
├── Collect vowels from buffer
│
├── Find mark position using Phonology rules
│   ├── Single vowel → đặt trên nó
│   ├── Double vowel + có final → đặt trên vowel thứ 2
│   ├── Double vowel + không final → đặt trên vowel thứ 1
│   └── Triple vowel → đặt trên vowel giữa
│
├── Clear existing marks first
│
└── Apply new mark at position

Ref: core/src/engine/transform.rs:192-218
```

### 6.4 Tone Placement Rules

```
find_tone_position(vowels, has_final, modern, has_qu) → position
│
├── Single vowel: return vowel position
│
├── Double vowel:
│   ├── Có final? → return vowel[1]
│   ├── Có dấu phụ (ư,ơ,ô,ê,â,ă)? → return nó
│   ├── Pattern oa, oe, uy? → return vowel[1]
│   ├── Pattern ai, ao, au? → return vowel[0]
│   └── Default → return vowel[0]
│
└── Triple vowel: return vowel[1] (giữa)

Ref: core/src/data/vowel.rs (Phonology::find_tone_position)
```

---

## 7. UO COMPOUND HANDLING

```
UO COMPOUND:
│
├── Khi gặp 'w' (Telex) hoặc '7' (VNI)
│
├── Scan for "uo" hoặc "ou" ADJACENT trong buffer
│   ├── Found → Apply HORN to BOTH
│   │   ├── u → ư
│   │   └── o → ơ
│   │
│   └── Not found → Apply to single vowel
│
└── VÍ DỤ:
    ├── "truong" + 'w' → "trương"
    │   ├── Found "uo" at positions 2-3
    │   ├── u → ư, o → ơ
    │   └── Result: "trương"
    │
    ├── "nguoi" + 'w' → "ngươi"
    │   ├── Found "uo" at positions 2-3
    │   └── Result: "ngươi"
    │
    └── "mua" + 'w' → "mưa"
        ├── "ua" (not "uo")
        ├── Only u → ư
        └── Result: "mưa"

Ref: core/src/engine/transform.rs:154-190
```

---

## 8. DOUBLE-KEY REVERT

### 8.1 Mechanism

```
DOUBLE-KEY REVERT:
│
├── Lưu last_transform = { key, pattern, result }
│
├── Khi modifier key được nhấn:
│   │
│   ├── [last_transform.key == current_key?]
│   │   ├── YES → REVERT
│   │   │   ├── Xóa transformation trước đó
│   │   │   ├── Thêm key vào output
│   │   │   └── Clear last_transform
│   │   │
│   │   └── NO → Apply transformation bình thường
│   │
│   └── Save current transformation
│
└── Transform Types tracked:
    ├── Mark(key, mark_value)
    ├── Tone(key, tone_value)
    ├── Stroke(key)
    ├── WAsVowel
    └── WShortcutSkipped

Ref: core/src/engine/mod.rs:77-85
```

### 8.2 Revert Examples

```
VÍ DỤ REVERT:

"a" + 'a' → "â" (save: Tone(key:'a'))
"â" + 'a' → "aa" (revert: â → a, add 'a')

"a" + 's' → "á" (save: Mark(key:'s'))
"á" + 's' → "as" (revert: á → a, add 's')

"d" + 'd' → "đ" (save: Stroke(key:'d'))
"đ" + 'd' → "dd" (revert: đ → d, add 'd')

"w" → "ư" (save: WAsVowel)
"ư" + 'w' → "w" (revert: ư → w)
```

---

## 9. W-AS-VOWEL (TELEX)

```
try_w_as_vowel(caps) → Option<Result>
│
├── Chỉ trong Telex mode
│
├── Skip nếu last_transform == WShortcutSkipped
│
├── Revert check:
│   └── last_transform == WAsVowel? → revert to "w"
│
├── Try transformation:
│   ├── Push U with HORN to buffer
│   ├── Validate: is_valid(buffer)?
│   │   ├── YES → return "ư"
│   │   └── NO → pop U, return None
│
└── VÍ DỤ:
    ├── "w" alone → "ư" (valid syllable)
    ├── "nhw" → "như" (valid: nh + ư)
    ├── "kw" → "kw" (invalid: k cannot precede ư)
    └── "ww" → "w" (revert)

Ref: core/src/engine/mod.rs:261-300
```

---

## 10. SHORTCUT TABLE

### 10.1 Data Structures

```rust
/// Shortcut entry
pub struct Shortcut {
    pub trigger: String,        // "vn"
    pub replacement: String,    // "Việt Nam"
    pub condition: TriggerCondition,
    pub case_mode: CaseMode,
    pub enabled: bool,
    pub input_method: InputMethod,
}

/// Trigger conditions
pub enum TriggerCondition {
    Immediate,      // Trigger ngay khi match
    OnWordBoundary, // Trigger khi space/enter/punctuation
}

/// Case handling
pub enum CaseMode {
    Exact,     // Giữ nguyên replacement
    MatchCase, // "VN" → "VIỆT NAM", "vn" → "Việt Nam"
}

/// Input method filter
pub enum InputMethod {
    All,    // Apply cho tất cả
    Telex,  // Chỉ Telex
    Vni,    // Chỉ VNI
}

Ref: core/src/engine/shortcut.rs:20-53
```

### 10.2 Matching Algorithm

```
try_match(buffer, key_char, is_word_boundary, method) → Option<ShortcutMatch>
│
├── STEP 1: Lookup (longest-match-first)
│   └── sorted_triggers sorted by length DESC
│
├── STEP 2: Check condition
│   ├── Immediate → match ngay
│   └── OnWordBoundary → key là space/punctuation?
│
├── STEP 3: Apply case transformation
│   ├── Exact → giữ nguyên
│   ├── MatchCase:
│   │   ├── All uppercase → replacement.to_uppercase()
│   │   ├── First uppercase → capitalize
│   │   └── Lowercase → giữ nguyên
│
└── STEP 4: Return result
    └── ShortcutMatch { backspace_count, output, include_trigger_key }

Ref: core/src/engine/shortcut.rs:278-314
```

---

## 11. DATA STRUCTURES

### 11.1 Buffer

```rust
/// Circular buffer - fixed 32 chars
pub struct Buffer {
    data: [Char; MAX],  // MAX = 32
    len: usize,
}

/// Single character with modifiers
pub struct Char {
    pub key: u16,     // Virtual keycode
    pub caps: bool,   // Uppercase?
    pub tone: u8,     // 0=none, 1=circumflex, 2=horn
    pub mark: u8,     // 0=none, 1-5=sắc/huyền/hỏi/ngã/nặng
    pub stroke: bool, // d → đ
}

Ref: core/src/engine/buffer.rs
```

### 11.2 Vowel Table

```rust
/// 72-entry vowel lookup (12 bases × 5 marks + tones)
const VOWELS: [(u32, u32, u32, u32, u32, u32); 72] = [
    // (base, sắc, huyền, hỏi, ngã, nặng)
    ('a', 'á', 'à', 'ả', 'ã', 'ạ'),
    ('ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'),
    ('â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'),
    // ... 12 bases × 6 variants
];

Ref: core/src/data/chars.rs
```

---

## 12. FFI INTERFACE

```rust
/// Initialize engine
#[no_mangle]
pub extern "C" fn ime_init() -> *mut Engine

/// Process keystroke
#[no_mangle]
pub extern "C" fn ime_key(engine: *mut Engine, key: u16, caps: bool, ctrl: bool) -> Result

/// Process keystroke with Shift
#[no_mangle]
pub extern "C" fn ime_key_ext(engine: *mut Engine, key: u16, caps: bool, ctrl: bool, shift: bool) -> Result

/// Set input method (0=Telex, 1=VNI)
#[no_mangle]
pub extern "C" fn ime_method(engine: *mut Engine, method: u8)

/// Enable/disable engine
#[no_mangle]
pub extern "C" fn ime_enabled(engine: *mut Engine, enabled: bool)

/// Clear buffer
#[no_mangle]
pub extern "C" fn ime_clear(engine: *mut Engine)

/// Free engine
#[no_mangle]
pub extern "C" fn ime_free(engine: *mut Engine)

Ref: core/src/lib.rs
```

---

## 13. EXAMPLES

### 13.1 Complete Flow: "được"

```
User types: d → u → o → c → w → j

1. 'd':
   ├── Stage 1-5: not modifier
   ├── Stage 6: push 'd' to buffer
   └── buffer = ['d']

2. 'u':
   ├── Stage 1-5: not modifier
   ├── Stage 6: push 'u' to buffer
   └── buffer = ['d', 'u']

3. 'o':
   ├── Stage 1-5: not modifier
   ├── Stage 6: push 'o' to buffer
   └── buffer = ['d', 'u', 'o']

4. 'c':
   ├── Stage 1-5: not modifier
   ├── Stage 6: push 'c' to buffer
   └── buffer = ['d', 'u', 'o', 'c']

5. 'w' (horn modifier):
   ├── Stage 2: try_tone()
   │   ├── Validate: "duoc" → VALID ✓
   │   ├── Find UO compound at positions 1-2
   │   ├── Apply HORN to both: u→ư, o→ơ
   │   └── Return: backspace=3, "ươc"
   └── Output: delete "uoc", type "ươc" → "dươc"

6. 'j' (nặng modifier):
   ├── Stage 3: try_mark()
   │   ├── Validate: "dươc" → VALID ✓
   │   ├── Collect vowels: [ư, ơ]
   │   ├── Find position: has_final=true → pos=1 (ơ)
   │   ├── Apply mark: ơ + nặng → ợ
   │   └── Return: backspace=2, "ợc"
   └── Output: delete "ơc", type "ợc" → "dượC"

Final: "được" ✓
```

### 13.2 Validation Rejection: "Claus"

```
User types: C → l → a → u → s

1-4. 'C', 'l', 'a', 'u':
   ├── All normal letters
   └── buffer = ['C', 'l', 'a', 'u']

5. 's' (mark modifier):
   ├── Stage 3: try_mark()
   │   ├── Validate: "Clau" → INVALID ✗
   │   │   └── initial = "cl" ∉ VALID_INITIALS
   │   └── Return None (không transform)
   ├── Stage 6: push 's' to buffer
   └── buffer = ['C', 'l', 'a', 'u', 's']

Final: "Claus" (không bị biến đổi) ✓
```

---

## Changelog

- **2025-12-10**: Viết lại hoàn toàn
  - Loại bỏ V1/V2 terminology
  - Đối chiếu với code thực tế trong core/src/
  - Thêm references đến source files
  - Cập nhật cấu trúc theo 7-stage pipeline
  - Thêm ví dụ thực tế với flow chi tiết

---

*Tài liệu thuật toán GoNhanh Core Engine*
