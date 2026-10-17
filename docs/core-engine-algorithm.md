# GoNhanh Core Typing Engine

> Tài liệu thuật toán cho engine gõ tiếng Việt.

**Tài liệu liên quan**:
- [core-architecture.md](./core-architecture.md) - Kiến trúc ba tầng, bất biến, dữ liệu, cách kiểm thử
- [vietnamese-language-system.md](./vietnamese-language-system.md) - Hệ thống chữ viết tiếng Việt & Quy tắc âm vị học
- [validation-algorithm.md](./validation-algorithm.md) - Chi tiết quy tắc validation
- [system-architecture.md](./system-architecture.md) - Kiến trúc tổng thể

---

## 1. NGUYÊN TẮC THIẾT KẾ

### 1.1 Core Principles

```
NGUYÊN TẮC:
│
├── 1. VALIDATION FIRST (★ QUAN TRỌNG NHẤT)
│   └── Mỗi cách đọc của chuỗi phím được kiểm bằng phonology::validate
│       ├── "nghieng" hợp lệ? → YES → cách đọc còn sống
│       ├── "claus" hợp lệ? → NO → cách đọc bị loại, hiện chữ như đã gõ
│       └── Không cách đọc nào hợp lệ → không biến đổi gì
│
├── 2. DẤU LÀ HÀM CỦA CHỮ CUỐI CÙNG
│   └── Dấu thanh đặt trên âm tiết hoàn chỉnh (tone_index), không phụ thuộc thứ tự gõ
│       ├── "hoaf" và "hofa" → cùng "hoà"
│       └── Không case-by-case (prev + current)
│
├── 3. LONGEST-MATCH-FIRST
│   └── Cho phụ âm đầu/cuối và gõ tắt
│       ├── "ngh" match trước "ng", "ch"/"ng"/"nh" match trước "c"/"n"
│       └── Gõ tắt: trigger dài nhất thắng
│
└── 4. GÕ ĐÔI = QUAY VỀ CHỮ THƯỜNG
    └── Nhấn cùng phím 2 lần → hoàn tác biến đổi, luôn về các chữ thường
        ├── "aa" → "â", "aaa" → "aa"
        └── "ss" → "as" (không phải "á")
```

---

## 2. KIẾN TRÚC XỬ LÝ

### 2.1 Ba tầng

```
core/src/
├─ phonology/   "Đây có phải tiếng Việt không? Dấu đặt ở đâu?"   validate, tone_index
├─ compose/     "Chuỗi phím này đang gõ ra chữ gì?"               beam các cách đọc → Display → diff
└─ engine/      "Từ này kết thúc thì làm gì?"                      dấu cách, ESC, gõ tắt, restore, lịch sử
```

Phụ thuộc chỉ đi xuống: `engine` → `compose` → `phonology`. Chi tiết, bất biến và dữ liệu: [core-architecture.md](./core-architecture.md).

### 2.2 Luồng một phím

```
Engine::on_key_ext(key, caps, ctrl, shift) → Result
│
├─► [ctrl?] ──► xoá từ + lịch sử ──► return NONE
│
├─► [!enabled?] ──► chỉ gõ tắt (disabled.rs)
│
├─► [Space / dấu ngắt / ESC?] ──► boundary.rs
│   └── gõ tắt → bảng restore tiếng Anh → lịch sử → xoá từ
│
├─► [DELETE?] ──► quay về trạng thái phím trước (word.rs)
│
└─► [chữ / số / ngoặc] ──► word.rs
    │
    ├── Compose::push(phím)
    │   ├── parse.extend   mỗi phím là chữ hoặc dấu → các cách đọc con
    │   ├── validate       phonology loại cách đọc không thể thành tiếng Việt
    │   └── beam ≤ 8       xếp hạng Complete > Prefix > Loose > NamePrefix
    │
    ├── render             cách đọc tốt nhất → Display (dấu đặt bởi tone_index)
    └── diff               Display cũ → mới = (backspace, chữ mới) → Result

Ref: core/src/engine/mod.rs (key_out), core/src/engine/word.rs, core/src/engine/boundary.rs
     core/src/compose/lattice.rs
```

### 2.3 Result Structure

```rust
/// FFI Result - 1028 bytes
#[repr(C)]
pub struct Result {
    pub chars: [u32; 256], // UTF-32 codepoints
    pub action: u8,        // 0=None, 1=Send, 2=Restore
    pub backspace: u8,     // Characters to delete
    pub count: u8,         // Valid chars count
    pub flags: u8,         // bit 0: key consumed (shortcut)
}

Ref: core/src/engine/mod.rs
```

---

## 3. MODIFIER DETECTION

Bàn phím được mô tả bằng một bảng `phím → ý định` (`Intent`): chữ thường, dấu thanh, dấu mũ, móc/breve, gạch đ, xoá dấu. Engine không bao giờ hỏi "đây là Telex?".

### 3.1 Telex Modifiers

```
TELEX:
├── Dấu phụ:
│   ├── 'a','e','o' → lặp lại → â, ê, ô (circumflex)
│   ├── 'w' → horn (ơ, ư) hoặc breve (ă); đứng một mình → ư
│   └── 'd' → dd (đ - stroke)
│
├── Dấu thanh:
│   ├── 's' → sắc
│   ├── 'f' → huyền
│   ├── 'r' → hỏi
│   ├── 'x' → ngã
│   └── 'j' → nặng
│
└── Xoá dấu:
    └── 'z' → xóa dấu

Ref: core/src/compose/method.rs
```

### 3.2 VNI Modifiers

```
VNI:
├── Dấu phụ:
│   ├── '6' → circumflex (â, ê, ô)
│   ├── '7' → horn (ơ, ư)
│   ├── '8' → breve (ă)
│   └── '9' → stroke (đ)
│
├── Dấu thanh:
│   ├── '1' → sắc
│   ├── '2' → huyền
│   ├── '3' → hỏi
│   ├── '4' → ngã
│   └── '5' → nặng
│
└── Xoá dấu:
    └── '0' → xóa dấu

Ref: core/src/compose/method.rs
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
│   ├── Đôi: ch, gh, gi, kh, kr, ng, nh, ph, qu, th, tr (11) - kr cho tên dân tộc
│   └── Ba: ngh (1)
│
├── G = Âm đệm (Glide) - TÙY CHỌN
│   └── o (oa, oe), u (uy, ue)
│
├── V = Nguyên âm chính (Vowel) - BẮT BUỘC
│   └── a, ă, â, e, ê, i, o, ô, ơ, u, ư, y (12)
│
└── C₂ = Âm cuối (Final) - TÙY CHỌN
    ├── Phụ âm: c, k, m, n, p, t (6) - k cho tên dân tộc
    ├── Đôi: ch, ng, nh (3)
    └── Bán nguyên âm: i, y, o, u (4)

Ref: core/src/phonology/validity.rs, core/src/phonology/letters.rs
```

### 4.2 Tách âm tiết

```
split(units) → (độ dài initial, nucleus start, nucleus end)
│
├── STEP 1: Bỏ qua phụ âm đến nguyên âm đầu tiên
│   ├── Special: "gi" + nguyên âm khác → gi là initial (gia, giu, gieo; "gii" không)
│   └── Special: "qu" + nguyên âm → qu là initial
│
├── STEP 2: Nucleus = dãy nguyên âm liên tiếp (âm đệm o/u thuộc nucleus)
│
└── STEP 3: Phần còn lại là coda (khớp dài nhất trước)
    ├── 2 chữ: ch, ng, nh
    └── 1 chữ: c, k, m, n, p, t, i, y, o, u

Ref: core/src/phonology/validity.rs (split)
```

### 4.3 Split Examples

```
VÍ DỤ:

"nghieng":
├── initial = "ngh" (3 chữ)
├── nucleus = "ie"
├── coda = "ng"
└── Result: valid ✓

"hoa":
├── initial = "h"
├── nucleus = "oa" (o là âm đệm)
├── coda = ""
└── Result: valid ✓

"qua":
├── initial = "qu" (u thuộc initial)
├── nucleus = "a"
├── coda = ""
└── Result: valid ✓

"giau":
├── initial = "gi" (i thuộc initial vì sau có nguyên âm)
├── nucleus = "au"
├── coda = ""
└── Result: valid ✓
```

---

## 5. VALIDATION RULES

### 5.1 Các bất biến của `validate`

```
phonology::validate(units, tone, opts) → Invalid | NamePrefix | Loose | Prefix | Complete
│
├── I1 Cấu trúc: (C₁)(G)V(C₂) + thanh; qu/gi nuốt u/i
│   └── Phải có nguyên âm; mọi chữ phải thuộc initial / nucleus / coda
│
├── I2 Chính tả phụ âm đầu
│   ├── c + (e,i,y) → INVALID (dùng k)
│   ├── k + (a,o,u) → INVALID (dùng c)
│   ├── g + (e) → INVALID (dùng gh)
│   ├── ng + (e,i) → INVALID (dùng ngh)
│   ├── gh + (a,o,u) → INVALID (dùng g)
│   └── ngh + (a,o,u) → INVALID (dùng ng)
│
├── I3 Nucleus nằm trong danh sách nguyên âm hợp lệ
│
├── I4 Coda tắc (c, ch, p, t) chỉ nhận thanh sắc / nặng
│
└── I5 Ma trận nucleus × coda (sinh từ vi.dic vào phonology/tables.rs)

Trạng thái:
├── Invalid    không cách nào gõ tiếp thành tiếng Việt
├── Prefix     chưa xong nhưng còn đường hợp lệ (thiếu dấu, thiếu coda)
├── Complete   âm tiết hoàn chỉnh
├── Loose      chỉ khi bật Gõ tự do: vần đúng, phụ âm đầu ngoài tiếng Việt (khphá, zị)
└── NamePrefix đầu của tên riêng (kô của kông), giữ sống nhưng không tự hiện

Ref: core/src/phonology/validity.rs, core/src/phonology/tables.rs (sinh tự động)
```

### 5.2 Validation Examples

```
VALIDATION EXAMPLES:

"duoc" → VALID ✓
├── initial = "d" ✓
├── nucleus = "uo" ✓
└── coda = "c" ✓

"clau" → INVALID ✗
└── initial = "cl" không phải phụ âm đầu tiếng Việt

"john" → INVALID ✗
└── initial = "j" không phải phụ âm đầu tiếng Việt

"http" → INVALID ✗
└── No vowel found

"ci" → INVALID ✗
└── Chính tả: c + i → phải dùng k
```

---

## 6. BIẾN ĐỔI VÀ ĐẶT DẤU

Không có bước biến đổi riêng: `parse` sinh các cách đọc (mỗi phím là chữ hoặc dấu), `render` dựng chữ hiển thị từ cách đọc thắng, `diff` tính số backspace.

### 6.1 Stroke (d → đ)

```
'd' có hai cách đọc: chữ d, hoặc gạch ngang chữ d đứng trước.
│
├── "dd"   → "đ" (gạch chữ d trước đó)
├── "Dod"  → "Đo" (d cuối gạch d đầu; Telex cho phép d cách xa)
└── "ddd"  → "dd" (gõ đôi lần nữa hoàn tác)

Ref: core/src/compose/parse.rs, core/src/compose/method.rs
```

### 6.2 Dấu mũ / móc / breve

```
Telex:
├── aa, ee, oo → circumflex (â, ê, ô)
└── w → horn trên o/u (ơ, ư) hoặc breve trên a (ă); "w" đứng một mình → ư

VNI:
├── 6 → circumflex (a, e, o)
├── 7 → horn (o, u)
└── 8 → breve (a)

Khi có nhiều nguyên âm đích (uo, ươ) cả hai cách đọc cùng sống trong beam;
validate loại cách đọc không hợp lệ, xếp hạng chọn cách đọc còn lại tốt nhất.

Ref: core/src/compose/parse.rs
```

### 6.3 Dấu thanh (sắc/huyền/hỏi/ngã/nặng)

```
Dấu thanh là một thuộc tính của cả âm tiết (parse.tone), không gắn vào phím.
│
├── render gọi tone_index(units, modern) trên âm tiết hoàn chỉnh
├── Đổi dấu = thay giá trị tone, dấu cũ biến mất
└── Dấu về lại đúng nguyên âm khi âm tiết thay đổi ("hoa" + f + i → "hoài")

Ref: core/src/compose/render.rs, core/src/phonology/tone_place.rs
```

### 6.4 Tone Placement Rules

```
tone_index(units, modern) → vị trí nguyên âm mang dấu
│
├── qu / gi: u / i thuộc phụ âm đầu, không mang dấu
├── Nguyên âm có dấu phụ (ư, ơ, ô, ê, â, ă) → mang dấu (ươ → ơ, uyê → ê)
├── Nguyên âm đơn → mang dấu
│
├── Nguyên âm đôi:
│   ├── Có coda → vowel[1]
│   ├── oa, oe, uy: kiểu mới (oà, uý) → vowel[1]; kiểu cũ (òa, úy) → vowel[0]
│   ├── uo, ue, ie, ye, ea (sắp có dấu mũ) → vowel[1]
│   └── Còn lại (ai, ao, au, ua, ia...) → vowel[0]
│
└── Nguyên âm ba: vowel[1] (giữa), ví dụ oai, uyê

Ref: core/src/phonology/tone_place.rs
```

---

## 7. UO COMPOUND HANDLING

```
UO COMPOUND:
│
├── Khi gặp 'w' (Telex) hoặc '7' (VNI)
│
├── "uo" liền kề → cách đọc thắng áp HORN cho CẢ HAI
│   ├── u → ư
│   └── o → ơ
│
└── VÍ DỤ:
    ├── "truongw" → "trương"
    ├── "nguoiw" → "ngươi"
    └── "mwa" → "mưa" ("ua" không phải "uo": chỉ u → ư)
```

---

## 8. GÕ ĐÔI HOÀN TÁC

### 8.1 Cơ chế

```
GÕ ĐÔI:
│
├── Mỗi phím dấu là một cách đọc "dấu" song song với cách đọc "chữ"
│
├── Gõ lại đúng phím vừa tạo dấu:
│   ├── cách đọc dấu bị loại
│   └── hiện các chữ thường, kể cả khi chuỗi phím trông như từ tiếng Anh
│
├── Phím vừa hoàn tác, gõ tiếp thì chỉ là chữ ("aaa" → "aa", "aaaa" → "aaa")
│
└── Từ điển chỉ đổi điều này ở dấu cách khi chữ gõ chính là một từ có chữ đôi
    (perry, class); danh sách telex_doubles chỉ giữ từ có trong từ điển tiếng Anh

Ref: core/src/compose/lattice.rs, core/src/engine/restore.rs, core/src/data/telex_doubles.txt
```

### 8.2 Revert Examples

```
VÍ DỤ:

"aa" → "â"           "aaa" → "aa"
"as" → "á"           "ass" → "as"
"dd" → "đ"           "ddd" → "dd"
"w"  → "ư"           "ww"  → "w"
VNI: "ba1" → "bá"    "ba11" → "ba1"
```

---

## 9. W-AS-VOWEL (TELEX)

```
'w' trong Telex có ba cách đọc, beam giữ tất cả:
│
├── horn trên o/u hoặc breve trên a đứng trước
├── "ư" (w đứng một mình như một nguyên âm)
└── chữ "w" thường
│
├── validate loại cách đọc không hợp lệ
│   ├── "w"   → "ư" (nucleus hợp lệ)
│   ├── "nhw" → "như" (nh + ư)
│   └── "kw"  → "kw" (k không đứng trước ư)
│
└── "ww" → "w" (gõ đôi hoàn tác)

Ref: core/src/compose/parse.rs
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

Ref: core/src/engine/shortcut.rs
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

Ref: core/src/engine/shortcut.rs (ShortcutTable::try_match)
```

---

## 11. DATA STRUCTURES

### 11.1 Từ đang gõ

```rust
/// Mỗi phím vật lý của từ hiện tại: chữ thường / số / ngoặc, kèm hoa-thường
pub struct RawKey { pub ch: u8, pub caps: bool }

/// Một Compose = mảng cố định MAXK = 24 phím + beam các cách đọc (không cấp phát khi gõ)
pub struct Compose { /* keys, beam ≤ 8 Parse */ }

/// Chữ hiển thị (tối đa một ký tự mỗi phím)
pub struct Display { pub chars: [char; MAXK], pub len: u8 }

Ref: core/src/compose/mod.rs, core/src/compose/lattice.rs, core/src/compose/render.rs
```

### 11.2 Danh sách từ

```
data/lexicon.rs: danh sách tĩnh, đã sắp xếp, tìm kiếm nhị phân ngay trên blob (không heap)
├── VI       âm tiết tiếng Việt thật (vi.dic)
├── KEEP     từ luôn giữ nguyên (keep.dic)
├── EN       từ tiếng Anh tuyển chọn (dictionaries/en/*.txt): bằng chứng mạnh
├── REF      danh sách tham khảo lớn (dictionaries/en-ref/*.txt): bằng chứng yếu
└── DOUBLES  từ Anh chứa chữ đôi kiểu Telex

Sinh bởi core/build.rs từ core/src/data/dictionaries/

Ref: core/src/data/lexicon.rs
```

---

## 12. FFI INTERFACE

Engine là một instance toàn cục sau Mutex; các hàm không nhận con trỏ engine.

```rust
/// Initialize engine (once)
#[no_mangle]
pub extern "C" fn ime_init()

/// Process keystroke (Result phải được giải phóng bằng ime_free)
#[no_mangle]
pub extern "C" fn ime_key(key: u16, caps: bool, ctrl: bool) -> *mut Result

/// Process keystroke with Shift
#[no_mangle]
pub extern "C" fn ime_key_ext(key: u16, caps: bool, ctrl: bool, shift: bool) -> *mut Result

/// Set input method (0=Telex, 1=VNI)
#[no_mangle]
pub extern "C" fn ime_method(method: u8)

/// Enable/disable engine
#[no_mangle]
pub extern "C" fn ime_enabled(enabled: bool)

/// Clear current word
#[no_mangle]
pub extern "C" fn ime_clear()

/// Free a Result
#[no_mangle]
pub unsafe extern "C" fn ime_free(r: *mut Result)

Ref: core/src/lib.rs (22 hàm, gồm cả các cài đặt và gõ tắt)
```

---

## 13. EXAMPLES

### 13.1 Complete Flow: "dược"

```
User types: d → u → o → c → w → j

Màn hình sau từng phím (cargo run --release --example trace -- telex duocwj):
  d | du | duo | duoc | dươc | dược

5. 'w':
   ├── parse.extend: "w" là horn trên "uo", hoặc "ư", hoặc chữ w
   ├── validate: "dươc" Prefix (coda tắc c còn chờ thanh sắc/nặng) ✓
   └── diff: backspace=3, "ươc" → "dươc"

6. 'j' (nặng):
   ├── tone = nặng cho cả âm tiết
   ├── tone_index("dươc") → ơ (ươ → ơ)
   └── diff: backspace=2, "ợc" → "dược"

Final: "dược" ✓
```

### 13.2 Validation Rejection: "Claus"

```
User types: C → l → a → u → s

   C | Cl | Cla | Clau | Claus

5. 's' (sắc):
   ├── Cách đọc "dấu sắc": validate("Clau", sắc) → Invalid ✗
   │   └── initial = "cl" không phải phụ âm đầu tiếng Việt
   └── Cách đọc "chữ s" còn lại → hiện "Claus"

Final: "Claus" (không bị biến đổi) ✓
```

---

## Changelog

- **2026-10**: Viết lại theo kiến trúc phonology → compose → engine
  - Bỏ mô tả buffer + 7 stage cũ, thay bằng beam các cách đọc
  - Đối chiếu với code thực tế trong core/src/
  - Thêm liên kết tới core-architecture.md

---

*Tài liệu thuật toán GoNhanh Core Engine*
