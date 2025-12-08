# GoNhanh Core Typing Engine V2 - Proposed Algorithm

> Tài liệu thuật toán đề xuất cho engine gõ tiếng Việt thế hệ mới.

**Tài liệu liên quan**:
- [core-engine-algorithm.md](./core-engine-algorithm.md) - Thuật toán hiện tại (v1)
- [vietnamese-language-system.md](./vietnamese-language-system.md) - Hệ thống chữ viết tiếng Việt & Quy tắc âm vị học

---

## 1. VẤN ĐỀ VỚI THUẬT TOÁN HIỆN TẠI (V1)

### 1.1 Hạn chế của Case-by-Case Processing

```
VẤN ĐỀ:
│
├── Xử lý theo từng case riêng lẻ, kiểm tra prev + current
│   ├── Stage 1: is_d(key, prev) → dd → đ
│   ├── Stage 2: is_tone_for(key, vowels) → aa → â
│   ├── Stage 3: is_mark(key) → s → sắc
│   └── Stage 4: is_remove(key) → z → xóa
│
├── BUG: Không xử lý được các pattern phức tạp
│   │
│   ├── "Dod" → Kỳ vọng: "Đo"
│   │   └── Thực tế: "Dod" (không nhận vì expect "Ddo")
│   │
│   ├── "truongw" → Kỳ vọng: "trương"
│   │   └── Cần xử lý uo → ươ đồng thời
│   │
│   └── Thứ tự gõ ảnh hưởng kết quả
│       ├── "as" → "á" ✓
│       └── "sa" → "sa" (không thành "á")
│
└── NGUYÊN NHÂN:
    ├── Chỉ kiểm tra immediate context (prev + current)
    ├── Không đọc lại toàn bộ buffer
    └── Không có pattern matching đa ký tự
```

### 1.2 Thiếu Validation

```
VẤN ĐỀ:
│
├── Không kiểm tra buffer có phải tiếng Việt hợp lệ
│   ├── "Claus" + s → áp dụng dấu sắc (sai!)
│   ├── "John" + s → áp dụng dấu sắc (sai!)
│   └── "HTTP" + s → áp dụng dấu sắc (sai!)
│
└── HẬU QUẢ:
    ├── Gõ code bị ảnh hưởng
    ├── Gõ tiếng Anh bị biến đổi
    └── UX kém
```

---

## 2. KIẾN TRÚC ĐỀ XUẤT (V2)

### 2.1 Nguyên tắc thiết kế

```
NGUYÊN TẮC V2:
│
├── 1. VALIDATION FIRST (★ QUAN TRỌNG NHẤT)
│   └── Khi detect modifier → VALIDATE buffer có phải tiếng Việt không?
│       ├── Không care buffer là gì, chỉ care có hợp lệ không
│       ├── "nghieng" hợp lệ? → YES → cho phép transform
│       ├── "claus" hợp lệ? → NO → không transform
│       └── Nếu INVALID → không làm gì, thêm key vào buffer bình thường
│
├── 2. PATTERN-BASED REPLACEMENT
│   └── Nếu VALID → đọc lại TOÀN BỘ buffer → replace theo pattern
│
├── 3. LONGEST-MATCH-FIRST (cho vị trí đặt dấu)
│   └── Tìm pattern nguyên âm dài nhất để xác định VỊ TRÍ đặt dấu
│       ├── "nghieng" + 'e' → tìm "ieng" → "iêng"
│       ├── "nguoi" + 'w' → tìm "uoi" → "ươi"
│       └── Không phải để filter, mà để biết đặt dấu ở đâu
│
└── 4. FLEXIBLE ORDER
    └── Thứ tự gõ không quan trọng
```

### 2.2 Pipeline mới

```
V2 PIPELINE
│
on_key(key, caps)
│
├─► [is_break(key)?] ──► clear buffer ──► return NONE
│
├─► [key == DELETE?] ──► pop buffer ──► return NONE
│
├─► [is_modifier(key)?] ..................... ★ ĐIỂM KHÁC BIỆT
│   │
│   │   ╔══════════════════════════════════════════════════════════╗
│   │   ║  MODIFIER DETECTED → TRIGGER PATTERN REPLACEMENT        ║
│   │   ╚══════════════════════════════════════════════════════════╝
│   │
│   ├── STEP 1: Validate buffer
│   │   ├── is_valid_vietnamese_syllable(buffer)?
│   │   │   ├── YES → tiếp tục
│   │   │   └── NO → return NONE (giữ nguyên, thêm key vào buffer)
│   │
│   ├── STEP 2: Read entire buffer
│   │   └── raw_string = buffer_to_string()
│   │
│   ├── STEP 3: Apply pattern replacement (longest-first)
│   │   └── transformed = apply_patterns(raw_string, modifier_key)
│   │
│   ├── STEP 4: Validate result
│   │   └── is_valid_vietnamese_syllable(transformed)?
│   │
│   └── STEP 5: Output
│       └── return Result::send(backspace_count, transformed)
│
└─► [is_letter(key)?] ──► push to buffer ──► return NONE
```

---

## 3. MODIFIER DETECTION

### 3.1 Bảng Modifier Keys

```
MODIFIERS = TONE_MODIFIERS ∪ MARK_MODIFIERS

TELEX:
├── TONE_MODIFIERS (dấu phụ):
│   ├── 'a' → có thể là aa (mũ) hoặc aw (trăng)
│   ├── 'e' → có thể là ee (mũ)
│   ├── 'o' → có thể là oo (mũ) hoặc ow (móc)
│   ├── 'w' → móc/trăng
│   └── 'd' → có thể là dd (đ)
│
├── MARK_MODIFIERS (dấu thanh):
│   ├── 's' → sắc
│   ├── 'f' → huyền
│   ├── 'r' → hỏi
│   ├── 'x' → ngã
│   └── 'j' → nặng
│
└── REMOVE_MODIFIER:
    └── 'z' → xóa dấu

VNI:
├── TONE_MODIFIERS:
│   ├── '6' → mũ (â, ê, ô)
│   ├── '7' → móc (ơ, ư)
│   ├── '8' → trăng (ă)
│   └── '9' → đ
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
```

### 3.2 Decision: Is Modifier?

```
is_modifier(key, buffer)
│
├─► [buffer.is_empty()?]
│   └── return false (không có gì để transform)
│
├─► [key ∈ MARK_MODIFIERS?]
│   └── return true
│
├─► [key ∈ REMOVE_MODIFIER?]
│   └── return true
│
├─► [key ∈ TONE_MODIFIERS?]
│   │
│   ├── Telex special cases:
│   │   ├── 'a' → check if buffer has 'a' (aa pattern)
│   │   ├── 'e' → check if buffer has 'e' (ee pattern)
│   │   ├── 'o' → check if buffer has 'o' (oo pattern)
│   │   ├── 'w' → check if buffer has a, o, u
│   │   └── 'd' → check if buffer has 'd' (dd pattern)
│   │
│   └── return true if pattern possible
│
└── return false
```

---

## 4. PATTERN REPLACEMENT ENGINE

### 4.1 Luồng xử lý chính

```
LUỒNG XỬ LÝ KHI DETECT MODIFIER:
│
├── STEP 1: VALIDATE (★ LUÔN ĐẦU TIÊN)
│   │
│   │   is_valid_vietnamese_syllable(buffer)?
│   │
│   ├── NO → return (thêm key vào buffer, không transform)
│   │   └── VD: "claus" + 's' → "clauss"
│   │
│   └── YES → tiếp tục STEP 2
│       └── VD: "nghieng" + 'e' → tiếp tục
│
├── STEP 2: TÌM PATTERN NGUYÊN ÂM (longest-first cho VỊ TRÍ)
│   │
│   │   Mục đích: Xác định đặt dấu/biến đổi ở ĐÂU trong buffer
│   │
│   ├── Tìm nguyên âm ba trước: iêng, ươi, oai, uôi, ...
│   ├── Rồi nguyên âm đôi: iê, ươ, uô, oa, ai, ao, ...
│   └── Cuối cùng nguyên âm đơn: a, e, i, o, u, ...
│
├── STEP 3: ÁP DỤNG TRANSFORMATION
│   │
│   ├── Tone modifier (aa, aw, ow, dd, ...):
│   │   └── Biến đổi nguyên âm/phụ âm tương ứng
│   │
│   └── Mark modifier (s, f, r, x, j, ...):
│       └── Đặt dấu thanh lên vị trí đã xác định
│
└── STEP 4: OUTPUT
    └── Rebuild và trả về kết quả

────────────────────────────────────────────────────────────

VÍ DỤ THỰC TẾ:

"nghieng" + 'e' (Telex ee → mũ)
├── Validate: ngh + ieng → VALID ✓
├── Tìm pattern: "ieng" là nguyên âm (i + e + ng? hoặc ie + ng)
├── 'e' trigger "ee" → mũ trên 'e' trong "ie"
├── Transform: ie → iê
└── Output: "nghiêng" ✓

"duoc" + 'w' (Telex w → móc)
├── Validate: d + uoc → VALID ✓
├── Tìm pattern: "uo" là nguyên âm đôi
├── 'w' trigger móc trên "uo"
├── Transform: uo → ươ (cả u và o)
└── Output: "dược" ✓

"nguoi" + 'w' + 'f' (Telex)
├── "nguoi" + 'w':
│   ├── Validate: ng + uoi → VALID ✓
│   ├── Tìm pattern: "uoi" là nguyên âm (uo + i)
│   ├── Transform: uo → ươ
│   └── Kết quả: "ngươi"
│
└── "ngươi" + 'f':
    ├── Validate: ng + ươi → VALID ✓
    ├── Tìm pattern: "ươi" là nguyên âm ba
    ├── Vị trí dấu: giữa (ơ)
    └── Output: "người" ✓
```

### 4.2 Pattern cho Tone Modifiers

```
TONE PATTERNS (biến đổi nguyên âm/phụ âm):
│
├── MŨ (^): Telex aa/ee/oo hoặc VNI 6
│   ├── a → â
│   ├── e → ê
│   └── o → ô
│
├── MÓC/TRĂNG: Telex w hoặc VNI 7/8
│   ├── a + 'w' hoặc '8' → ă (trăng)
│   ├── o + 'w' hoặc '7' → ơ (móc)
│   ├── u + 'w' hoặc '7' → ư (móc)
│   │
│   └── ĐẶC BIỆT: uo compound
│       ├── Nếu buffer có "uo" liền kề
│       └── Transform CẢ HAI: uo → ươ
│
└── GẠCH NGANG: Telex dd hoặc VNI 9
    └── d → đ (bất kỳ vị trí trong buffer)
```

### 4.3 Algorithm: Apply Patterns

```
apply_patterns(buffer_string, modifier_key)
│
├── Xác định loại modifier
│   ├── TONE_MODIFIER → apply_tone_patterns()
│   ├── MARK_MODIFIER → apply_mark_patterns()
│   └── REMOVE_MODIFIER → apply_remove_patterns()
│
└── Trả về transformed string

────────────────────────────────────────────────────────────

apply_tone_patterns(buffer, tone_key)
│
├── CASE: Telex 'w' hoặc VNI '7'/'8'
│   │
│   ├── TÌM PATTERN DÀI TRƯỚC:
│   │   │
│   │   ├── "uo" trong buffer?
│   │   │   └── YES → replace "uo" với "ươ" (cả u và o)
│   │   │
│   │   ├── "ua" trong buffer? (không có 'q' trước)
│   │   │   └── YES → replace "ua" với "ưa"
│   │   │
│   │   └── Fallback to single char:
│   │       ├── 'a' → 'ă' (với 'w' hoặc '8')
│   │       ├── 'o' → 'ơ' (với 'w' hoặc '7')
│   │       └── 'u' → 'ư' (với 'w' hoặc '7')
│   │
│   └── Reposition mark nếu cần
│
├── CASE: Telex 'aa'/'ee'/'oo' hoặc VNI '6'
│   │
│   ├── Tìm nguyên âm target
│   │   ├── 'a' → 'â'
│   │   ├── 'e' → 'ê'
│   │   └── 'o' → 'ô'
│   │
│   └── Reposition mark nếu cần
│
└── CASE: Telex 'dd' hoặc VNI '9'
    │
    ├── TÌM 'd' TRONG BUFFER (bất kỳ vị trí)
    │   ├── Found at position i
    │   └── buffer[i] = 'đ'
    │
    └── LƯU Ý: "Dod" → tìm 'd' ở vị trí 0 → "Đod" → xóa 'd' cuối → "Đo"

────────────────────────────────────────────────────────────

apply_mark_patterns(buffer, mark_key)
│
├── mark_value = get_mark_value(mark_key)
│   ├── 's'/1 → sắc
│   ├── 'f'/2 → huyền
│   ├── 'r'/3 → hỏi
│   ├── 'x'/4 → ngã
│   └── 'j'/5 → nặng
│
├── VALIDATE: Tone + Final Consonant Rule
│   ├── final = get_final_consonant(buffer)
│   ├── if final ∈ {p, t, c, ch}:
│   │   └── if mark_value ∉ {sắc, nặng}:
│   │       └── return buffer (không apply, không hợp lệ)
│
├── TÌM VỊ TRÍ ĐẶT DẤU (Phonology Algorithm)
│   │
│   ├── Thu thập nguyên âm
│   │   └── vowels = collect_vowels(buffer)
│   │
│   ├── LONGEST PATTERN FIRST:
│   │   │
│   │   ├── 3 nguyên âm? (iêu, ươi, oai, ...)
│   │   │   └── Đặt dấu theo quy tắc nguyên âm ba
│   │   │
│   │   ├── 2 nguyên âm?
│   │   │   ├── Có phụ âm cuối? → dấu trên nguyên âm thứ 2
│   │   │   ├── oa, oe, uy? → dấu trên nguyên âm thứ 2
│   │   │   ├── ai, ao, au? → dấu trên nguyên âm thứ 1
│   │   │   └── ươ, uô, iê? → dấu trên nguyên âm thứ 2
│   │   │
│   │   └── 1 nguyên âm?
│   │       └── Đặt dấu trực tiếp
│   │
│   └── target_pos = find_tone_position(vowels, ...)
│
└── Apply mark to buffer[target_pos]
```

### 4.4 Ví dụ: Pattern Matching cho "Dod"

```
CASE: "Dod" + enter (trong Telex, 'd' cuối là modifier nếu trước đó có 'd')

LUỒNG XỬ LÝ MỚI:
│
├── User gõ: D → o → d
│
├── Khi gõ 'd':
│   ├── buffer = ['D', 'o', 'd']
│   ├── is_modifier('d', buffer)?
│   │   └── Check: buffer có 'd' hoặc 'D'? → YES (vị trí 0)
│   │   └── return true
│   │
│   ├── STEP 1: Validate "Dod"
│   │   ├── C₁ = "d" ∈ VALID_INITIALS ✓
│   │   ├── V = "o" ∈ VALID_VOWELS ✓
│   │   └── is_valid = true
│   │
│   ├── STEP 2: Read buffer → "Dod"
│   │
│   ├── STEP 3: Apply patterns
│   │   ├── Modifier = 'd' (Telex dd → đ)
│   │   ├── Tìm 'd' hoặc 'D' trong buffer
│   │   ├── Found 'D' at position 0
│   │   ├── Transform: 'D' → 'Đ'
│   │   ├── Remove trigger 'd' at position 2
│   │   └── Result: "Đo"
│   │
│   └── STEP 4: Output
│       └── Result::send(3, "Đo")
│
└── OUTPUT: "Đo" ✓

SO SÁNH VỚI V1:
│
├── V1: "Dod" → Không match vì check prev=='d' && key=='d'
│   └── prev='o', key='d' → không match → output "Dod"
│
└── V2: "Dod" → Scan buffer, tìm 'd' bất kỳ → match → "Đo"
```

---

## 5. VALIDATION PIPELINE

### 5.1 Khi nào Validate?

```
VALIDATION TIMING:
│
├── TRƯỚC khi apply transformation
│   └── is_valid_vietnamese_syllable(buffer)?
│       ├── YES → tiếp tục transform
│       └── NO → không transform, thêm key vào buffer như bình thường
│
└── SAU khi transform (optional)
    └── Đảm bảo kết quả vẫn hợp lệ
```

### 5.2 Validation Algorithm

```
is_valid_vietnamese_syllable(buffer)
│
├─► STEP 1: Normalize buffer
│   └── input = buffer.to_lowercase().remove_marks()
│
├─► STEP 2: Check vowel exists
│   ├── has_vowel(input)?
│   │   ├── NO → return false ("HTTP", "CTRL")
│   │   └── YES → continue
│
├─► STEP 3: Parse syllable structure
│   │
│   │   parse_syllable(input) → {
│   │       initial: Option<String>,  // C₁
│   │       vowel: String,            // V (required)
│   │       final: Option<String>     // C₂
│   │   }
│   │
│   ├── Identify initial consonant (longest match first)
│   │   ├── "ngh" match? → initial = "ngh"
│   │   ├── "ng", "nh", "ch", "gh", "gi", "kh", "ph", "qu", "th", "tr" match?
│   │   ├── Single consonant match?
│   │   └── No match → initial = None (vowel-initial syllable)
│   │
│   ├── Identify vowel (longest match first)
│   │   ├── Triple vowels: iêu, yêu, ươi, ươu, uôi, oai, oay, oeo, uây, uyê
│   │   ├── Double vowels: ai, ao, au, âu, ây, eo, êu, ia, iê, ...
│   │   └── Single vowels: a, ă, â, e, ê, i, o, ô, ơ, u, ư, y
│   │
│   └── Remainder = final consonant
│
├─► STEP 4: Validate initial consonant
│   │
│   ├── initial ∈ VALID_INITIALS?
│   │   └── NO → return false ("Clau", "John", "Black")
│   │
│   └── Check spelling rules:
│       ├── "c" before e,ê,i,y? → return false
│       ├── "k" before a,ă,â,o,ô,ơ,u,ư? → return false
│       ├── "g" before e,ê,i? → return false
│       ├── "gh" before a,ă,â,o,ô,ơ,u,ư? → return false
│       ├── "ng" before e,ê,i? → return false
│       └── "ngh" before a,ă,â,o,ô,ơ,u,ư? → return false
│
├─► STEP 5: Validate vowel
│   └── vowel ∈ VALID_VOWELS? (should always be true if parsed)
│
├─► STEP 6: Validate final consonant
│   │
│   ├── final ∈ VALID_FINALS?
│   │   └── c, ch, m, n, ng, nh, p, t, i, y, o, u
│   │
│   └── Check vowel+final combination:
│       ├── -ch only after a, ă, ê, i
│       ├── -nh only after a, ă, ê, i, y
│       └── -ng not after e, ê
│
└─► return true
```

### 5.3 Validation Examples

```
VALIDATION EXAMPLES:
│
├── "duoc" → VALID
│   ├── initial = "d" ✓
│   ├── vowel = "uo" ✓
│   ├── final = "c" ✓
│   └── Can apply 'j' → "được" ✓
│
├── "clau" → INVALID
│   ├── initial = "cl" ✗ (not in VALID_INITIALS)
│   └── 's' pressed → ignore, output "claus"
│
├── "john" → INVALID
│   ├── initial = "j" ✗ (not in Vietnamese)
│   └── 's' pressed → ignore, output "johns"
│
├── "http" → INVALID
│   ├── No vowel found ✗
│   └── Any modifier → ignore
│
├── "nguoi" → VALID
│   ├── initial = "ng" ✓
│   ├── vowel = "uoi" (→ "ươi") ✓
│   ├── final = none ✓
│   └── Can apply 'w' → "người" ✓
│
└── "cap" + 'r' (hỏi) → INVALID TONE
    ├── Syllable valid: c + a + p ✓
    ├── But: p is stop consonant
    ├── hỏi (3) not allowed with -p
    └── Reject → output "capr" or ignore 'r'
```

---

## 6. UO COMPOUND HANDLING

### 6.1 Nguyên tắc

```
UO COMPOUND:
│
├── Khi gặp 'w' (Telex) hoặc '7' (VNI)
│
├── TÌM PATTERN "uo" hoặc "ou" trong buffer
│   ├── Found → Apply móc cho CẢ HAI
│   │   ├── u → ư
│   │   └── o → ơ
│   │
│   └── Not found → Apply cho single vowel
│
└── VÍ DỤ:
    ├── "truong" + 'w' → "trương"
    │   ├── Tìm "uo" tại vị trí 2-3
    │   ├── u → ư
    │   ├── o → ơ
    │   └── Result: "trương"
    │
    └── "mua" + 'w' → "mưa"
        ├── Tìm "ua" (không phải "uo")
        ├── Chỉ u → ư
        └── Result: "mưa"
```

---

## 7. DOUBLE-KEY REVERT (V2)

### 7.1 Cơ chế

```
DOUBLE-KEY REVERT (V2):
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
└── VÍ DỤ:
    │
    ├── "a" + 'a' → "â" (save: {key:'a', result:'â'})
    │   └── 'a' again → revert to "a" + add 'a' → "aa"
    │
    ├── "a" + 's' → "á" (save: {key:'s', result:'á'})
    │   └── 's' again → revert to "a" + add 's' → "as"
    │
    └── "truong" + 'w' → "trương"
        └── 'w' again → "truongw" (revert compound)
```

---

## 8. SO SÁNH V1 vs V2

```
┌─────────────────────┬─────────────────────────┬─────────────────────────┐
│       Tính năng     │          V1             │          V2             │
├─────────────────────┼─────────────────────────┼─────────────────────────┤
│ Processing          │ Case-by-case            │ Pattern-based           │
│                     │ (prev + current)        │ (full buffer scan)      │
├─────────────────────┼─────────────────────────┼─────────────────────────┤
│ Pattern matching    │ Immediate context only  │ Longest-match-first     │
├─────────────────────┼─────────────────────────┼─────────────────────────┤
│ Validation          │ Không có                │ Trước khi transform     │
├─────────────────────┼─────────────────────────┼─────────────────────────┤
│ "Dod" → ?           │ "Dod" (bug)             │ "Đo" ✓                  │
├─────────────────────┼─────────────────────────┼─────────────────────────┤
│ "Claus" + s → ?     │ "Cláus" (sai)           │ "Clauss" (giữ nguyên)   │
├─────────────────────┼─────────────────────────┼─────────────────────────┤
│ "HTTP" + s → ?      │ Có thể lỗi              │ "HTTPs" (giữ nguyên)    │
├─────────────────────┼─────────────────────────┼─────────────────────────┤
│ Gõ linh hoạt        │ Thứ tự quan trọng       │ Thứ tự linh hoạt        │
├─────────────────────┼─────────────────────────┼─────────────────────────┤
│ Code/Email/URL      │ Bị ảnh hưởng            │ Không ảnh hưởng         │
├─────────────────────┼─────────────────────────┼─────────────────────────┤
│ Tone+Stop rule      │ Không enforce           │ Enforce (cấp ✓, cảp ✗) │
└─────────────────────┴─────────────────────────┴─────────────────────────┘
```

---

## 9. IMPLEMENTATION ROADMAP

### 9.1 Các bước triển khai

```
IMPLEMENTATION STEPS:
│
├── PHASE 1: Validation Module
│   ├── Implement is_valid_vietnamese_syllable()
│   ├── Implement parse_syllable()
│   ├── Add VALID_INITIALS, VALID_VOWELS, VALID_FINALS constants
│   └── Add spelling rule checks (c/k, g/gh, ng/ngh)
│
├── PHASE 2: Pattern Matching Engine
│   ├── Define PATTERN_PRIORITY list
│   ├── Implement longest_match_first() algorithm
│   ├── Implement apply_tone_patterns()
│   └── Implement apply_mark_patterns()
│
├── PHASE 3: Modifier Detection
│   ├── Refactor is_modifier() to scan buffer
│   ├── Handle Telex special cases (aa, dd, etc.)
│   └── Handle VNI modifiers (1-9, 0)
│
├── PHASE 4: Main Pipeline
│   ├── Integrate validation into on_key()
│   ├── Replace case-by-case handlers with pattern engine
│   └── Maintain double-key revert mechanism
│
└── PHASE 5: Testing
    ├── Test "Dod" → "Đo"
    ├── Test validation (Clau, John, HTTP)
    ├── Test tone+stop rule (cấp ✓, cảp ✗)
    ├── Test UO compound (trương, người)
    └── Regression tests for all existing features
```

### 9.2 Data Structures

```rust
// Proposed data structures for V2

/// Modifier type
enum ModifierType {
    Tone(ToneModifier),   // aa, aw, ow, dd, 6, 7, 8, 9
    Mark(MarkModifier),   // s, f, r, x, j, 1-5
    Remove,               // z, 0
}

/// Pattern for replacement
struct Pattern {
    input: &'static str,   // "uo", "aa", "dd"
    output: &'static str,  // "ươ", "â", "đ"
    priority: u8,          // Higher = try first
}

/// Syllable structure
struct Syllable {
    initial: Option<String>,  // C₁
    vowel: String,            // V (required)
    final_c: Option<String>,  // C₂
}

/// Validation result
enum ValidationResult {
    Valid,
    InvalidInitial(String),
    InvalidVowel,
    InvalidFinal(String),
    InvalidToneFinal { tone: u8, final_c: String },
    NoVowel,
}

/// Main engine entry point (V2)
fn on_key_v2(key: Key, caps: bool) -> Result {
    // ... implementation following the V2 pipeline
}
```

---

## 10. TÓM TẮT

```
GONHANH ENGINE V2 SUMMARY
│
├── NGUYÊN TẮC CHÍNH
│   ├── 1. VALIDATION FIRST - Luôn validate buffer trước
│   ├── 2. Pattern-based replacement (không case-by-case)
│   ├── 3. Longest-match-first cho vị trí đặt dấu
│   └── 4. Flexible input order
│
├── VALIDATION
│   ├── Kiểm tra syllable structure
│   ├── Áp dụng quy tắc chính tả (c/k, g/gh, ng/ngh)
│   ├── Áp dụng quy tắc tone+stop consonant
│   └── Bảo vệ từ tiếng Anh/code/URL
│
├── PATTERN ENGINE
│   ├── Scan toàn bộ buffer
│   ├── Match patterns dài trước
│   ├── UO compound handling
│   └── Flexible 'd' position for đ
│
├── SỬA BUG
│   ├── "Dod" → "Đo" ✓
│   ├── "Claus" không bị transform ✓
│   └── Thứ tự gõ linh hoạt ✓
│
└── BACKWARD COMPATIBLE
    ├── Giữ double-key revert
    ├── Giữ Telex/VNI rules
    └── Giữ Unicode output format
```

---

## Changelog

- **2025-12-08**: Tạo tài liệu V2
  - Phân tích vấn đề với V1 (case-by-case processing)
  - Thiết kế kiến trúc mới (pattern-based, validation-first)
  - Chi tiết validation pipeline
  - Chi tiết pattern replacement engine
  - So sánh V1 vs V2
  - Implementation roadmap

---

*Tài liệu thiết kế cho GoNhanh Core Engine V2*
