# Validation Algorithm

> Thuật toán xác định chuỗi ký tự có phải âm tiết tiếng Việt hợp lệ hay không.

**Liên quan**: [vietnamese-language-system.md](./vietnamese-language-system.md) | [core-engine-algorithm.md](./core-engine-algorithm.md) | [core-architecture.md](./core-architecture.md)

---

## 1. Mục đích

```
Mỗi cách đọc của chuỗi phím được validate; cách đọc không thể thành tiếng Việt bị loại:

"duocwj" → "dươc" VALID   → "dược" ✓
"claus"  → "clau" + s INVALID → giữ nguyên "claus" ✓
"https"  → INVALID → giữ nguyên "https" ✓
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

```
16 phụ âm đơn:  b c d đ g h k l m n p q r s t v x
11 phụ âm đôi:  ch gh gi kh ng nh ph qu th tr  (+ kr cho tên dân tộc: Krông)
 1 phụ âm ba:   ngh
Chữ vay mượn f j w z chỉ được nhận khi bật tuỳ chọn phụ âm ngoại (foreign_initials).
```

### 3.2 Âm cuối (C₂)

```
Âm cuối phụ âm:   c ch m n ng nh p t  (+ k cho tên dân tộc: Đắk Lắk)
Bán nguyên âm:    i y o u  (thuộc nucleus, ví dụ "ai", "ao", "uy")
```

> **Lưu ý**: `k` được hỗ trợ cho tên riêng từ ngôn ngữ dân tộc thiểu số (Đắk Lắk, Đắk Nông); tên riêng nằm trong `names.dic`.

### 3.3 Quy tắc chính tả

| Consonant | Invalid trước | Nên dùng |
|-----------|---------------|----------|
| `c` | e, i, y | → `k` |
| `k` | a, o, u | → `c` |
| `g` | e | → `gh` |
| `ng` | e, i | → `ngh` |
| `gh` | a, o, u | → `g` |
| `ngh` | a, o, u | → `ng` |

### 3.4 Nucleus × coda (sinh từ từ điển)

Nguyên âm hợp lệ và coda nào đi được với nó **không gõ tay**: `scripts/gen/phonology_tables.py` đọc `vi.dic` (và `names.dic` cho tên riêng) rồi sinh `core/src/phonology/tables.rs` (`NUCLEUS_CODAS`, `NUCLEUS_INDEX`, `NAMES`). Mỗi dòng cho một nucleus: tập coda nó nhận (`OPEN`, `C`, `CH`, `M`, `N`, `NG`, `NH`, `P`, `T`, `K`).

```
Nucleus hợp lệ gồm nguyên âm đơn, đôi (ai, ao, au, iê, oa, uô, ươ, ...) và ba (oai, uyê, ươi, ...);
danh sách đầy đủ nằm trong tables.rs.

Không có trong ma trận → Invalid:
  "ea"  → search, teacher, beach, real
  "ou"  → you, our, house, about, would
  "yo"  → yoke, York, beyond
```

Sửa từ điển rồi chạy lại script để cập nhật bảng; `make gate` kiểm bảng còn mới (`--check`).

---

## 4. Tách âm tiết

`split(units)` trong `core/src/phonology/validity.rs` trả về (độ dài initial, đầu nucleus, cuối nucleus); phần còn lại là coda.

```
1. Bỏ qua phụ âm đến nguyên âm đầu tiên
   - Special case: gi + nguyên âm khác → gi là initial
   - Special case: qu + nguyên âm → qu là initial

2. Nucleus = dãy nguyên âm liên tiếp (âm đệm o/u thuộc nucleus)

3. Phần còn lại là coda
   - Try 2-char: ch, ng, nh
   - Try 1-char: c, k, m, n, p, t
```

---

## 5. Validation Rules

`phonology::validate(units, tone, opts)` trả về `Invalid | NamePrefix | Loose | Prefix | Complete`. Các bất biến (kiểm lần lượt):

```
I1  Cấu trúc (C₁)(G)V(C₂) + thanh; qu/gi nuốt u/i; ≥ 1 nguyên âm; mọi chữ được dùng hết
I2  Chính tả phụ âm đầu: c/k, g/gh, ng/ngh (bảng 3.3); initial phải nằm trong danh sách 3.1
I3  Nucleus thuộc danh sách nguyên âm hợp lệ
I4  Coda tắc (c, ch, p, t) chỉ nhận thanh sắc hoặc nặng
I5  Ma trận nucleus × coda (3.4)
```

Kết quả:

| Kết quả | Ý nghĩa |
|---------|---------|
| `Invalid` | không cách nào gõ tiếp thành tiếng Việt |
| `Prefix` | chưa xong nhưng còn đường hợp lệ (thiếu dấu, thiếu coda) |
| `Complete` | âm tiết hoàn chỉnh |
| `Loose` | chỉ khi bật Gõ tự do: vần đúng, phụ âm đầu ngoài tiếng Việt (`khphá`, `zị`) |
| `NamePrefix` | đầu của tên riêng (`kô` của `kông`), giữ sống nhưng không tự hiện |

Tuỳ chọn (`Opts`): `foreign_initials`, `free`, `names`, `lenient`, `at_end`.

---

## 6. Nhận biết từ nước ngoài

Không có hàm dò riêng cho từ nước ngoài: một từ nước ngoài thường không có cách đọc tiếng Việt hợp lệ, nên `validate` loại hết và chữ hiện như đã gõ. Từ vẫn còn một cách đọc hợp lệ được bảng restore ở dấu cách quyết định, dựa vào từ điển tiếng Anh (`EN`, `REF`) và `vi.dic` (xem [core-architecture.md](./core-architecture.md)).

**Đặc biệt:** có bằng chứng ý định tiếng Việt (đ, ư, ơ, ă) thì không trả về chữ thô (vd: "rượu", "đc").

---

## 7. API

```rust
// core/src/phonology
pub fn validate(units: &[Unit], tone: Tone, o: &Opts) -> Validity
pub fn tone_index(units: &[Unit], modern: bool) -> Option<usize>
pub fn nucleus_len(units: &[Unit]) -> usize

pub enum Validity { Invalid, NamePrefix, Loose, Prefix, Complete }
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
# Nucleus không có trong ma trận:
search, teacher, beach, real           → "ea"
you, your, house, about, would, south  → "ou"
yoke, York, beyond                     → "yo"

# Coda hoặc cụm phụ âm ngoài tiếng Việt:
metric, spectrum, matrix               → T+R, C+R clusters
describe, design                       → không có cách đọc hợp lệ
```

---

## 9. Integration với Engine

```
Compose::push(phím)
│
├─ parse.extend: mỗi phím là chữ hoặc dấu → các cách đọc con
│
├─ ★ VALIDATION: phonology::validate trên từng cách đọc
│   ├─ Invalid  → loại khỏi beam
│   └─ còn lại → xếp hạng Complete > Prefix > Loose > NamePrefix
│
└─ Hiển thị cách đọc tốt nhất; không còn cách đọc nào → chữ như đã gõ
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

- **2026-10**: Cập nhật theo `phonology::validate`
  - Bảng nucleus × coda sinh từ từ điển thay cho danh sách cặp nguyên âm viết tay
  - Bỏ các hàm kiểm tra cũ; nhận biết từ nước ngoài dựa vào beam + bảng restore

- **2025-12-31**: Thêm Auto-Restore Rules section
  - Rule 10.1: `-ing` + tone = invalid Vietnamese
  - Rule 10.2: Uncommon single-vowel words (ò, ì) restore
  - Rule 10.3: Circumflex without final (sê, tê) restore
  - Rule 10.4: Double-f preservation (off, offline)

- **2025-12-11**: Viết lại document theo code thực tế
  - Cập nhật Syllable struct với `Vec<usize>` và `glide` field
  - Chỉnh lại 5 validation rules theo code
  - Loại bỏ pseudo-code sai, thay bằng code snippets chính xác
  - Rút gọn từ ~800 dòng xuống ~200 dòng
