# GoNhanh Core Typing Engine - Decision Tree Documentation

> Tài liệu thuật toán và logic engine gõ tiếng Việt **hiện tại** theo dạng cây quyết định.

**Tài liệu liên quan**:
- [core-engine-algorithm-v2.md](./core-engine-algorithm-v2.md) - **Thuật toán đề xuất V2** (pattern-based, validation-first)
- [vietnamese-language-system.md](./vietnamese-language-system.md) - Hệ thống chữ viết tiếng Việt & Quy tắc âm vị học

> **Lưu ý**: Tài liệu này mô tả thuật toán **hiện tại (V1)** với cách tiếp cận case-by-case.
> Xem [V2](./core-engine-algorithm-v2.md) cho thiết kế mới với pattern-based replacement và validation.

---

## 1. TỔNG QUAN CẤU TRÚC ENGINE

```
GoNhanh Engine
│
├── 📁 engine/
│   ├── mod.rs ............. Engine chính (4-stage pipeline)
│   └── buffer.rs .......... Buffer gõ (max 32 chars)
│
├── 📁 data/
│   ├── vowel.rs ........... ★ Thuật toán đặt dấu (Phonology)
│   ├── chars.rs ........... Bảng Unicode nguyên âm
│   └── keys.rs ............ Mã phím macOS
│
└── 📁 input/
    ├── mod.rs ............. Trait Method
    ├── telex.rs ........... Telex rules
    └── vni.rs ............. VNI rules
```

---

## 2. CẤU TRÚC DỮ LIỆU

### 2.1 Char (Ký tự trong buffer)

```
Char
├── key: u16 ........... Mã phím (A=0, E=14, I=34, O=31, U=32, Y=16)
├── caps: bool ......... Chữ hoa?
├── tone: u8 ........... Dấu phụ
│   ├── 0 = none ....... a, e, i, o, u, y
│   ├── 1 = mũ (^) ..... â, ê, ô
│   └── 2 = móc/trăng .. ơ, ư / ă
├── mark: u8 ........... Dấu thanh
│   ├── 0 = none
│   ├── 1 = sắc ........ á
│   ├── 2 = huyền ...... à
│   ├── 3 = hỏi ........ ả
│   ├── 4 = ngã ........ ã
│   └── 5 = nặng ....... ạ
└── stroke: bool ....... d → đ?
```

### 2.2 Result (Kết quả FFI)

```
Result
├── chars: [u32; 32] ... Unicode output
├── action: u8
│   ├── 0 = NONE ....... Pass through, không làm gì
│   ├── 1 = SEND ....... Xóa + gửi ký tự mới
│   └── 2 = RESTORE .... Khôi phục (hiếm)
├── backspace: u8 ...... Số ký tự cần xóa
└── count: u8 .......... Số ký tự trong chars[]
```

---

## 3. PIPELINE XỬ LÝ PHÍM - DECISION TREE

### 3.1 Entry Point: on_key()

```
on_key(key, caps, ctrl)
│
├─► [ctrl == true?]
│   └── YES ──► clear buffer ──► return NONE
│
├─► [is_break(key)?] ........... (space, enter, dấu câu, arrows)
│   └── YES ──► clear buffer ──► return NONE
│
├─► [key == DELETE?]
│   └── YES ──► pop buffer ──► return NONE
│
└─► process(key, caps)
```

### 3.2 Process: 4-Stage Pipeline

```
process(key, caps)
│
│   ╔═══════════════════════════════════════════════════════╗
│   ║  STAGE 1: Xử lý đ (try_handle_d)                      ║
│   ╚═══════════════════════════════════════════════════════╝
├─► [is_d(key, prev)?] ............... Telex: dd / VNI: d9
│   └── YES ──► handle_d() ──► return Result
│
├─► [is_d_for(key, buffer)?] ......... VNI delayed: dung9
│   └── YES ──► handle_delayed_d() ──► return Result
│
│   ╔═══════════════════════════════════════════════════════╗
│   ║  STAGE 2: Xử lý dấu phụ (try_handle_tone)            ║
│   ╚═══════════════════════════════════════════════════════╝
├─► [is_tone_for(key, vowels)?] ...... aa/aw/a6/a7...
│   └── YES ──► handle_tone() ──► return Result
│
├─► [double-key revert?] ............. aaa → aa
│   └── YES ──► revert_tone() ──► return Result
│
│   ╔═══════════════════════════════════════════════════════╗
│   ║  STAGE 3: Xử lý dấu thanh (try_handle_mark)          ║
│   ╚═══════════════════════════════════════════════════════╝
├─► [is_mark(key)?] .................. s/f/r/x/j hoặc 1-5
│   ├── [double-key revert?]
│   │   └── YES ──► revert_mark() ──► return Result
│   └── handle_mark() ──► return Result
│
│   ╔═══════════════════════════════════════════════════════╗
│   ║  STAGE 4: Xử lý xóa dấu                              ║
│   ╚═══════════════════════════════════════════════════════╝
├─► [is_remove(key)?] ................ z hoặc 0
│   └── YES ──► handle_remove() ──► return Result
│
│   ╔═══════════════════════════════════════════════════════╗
│   ║  DEFAULT: Ký tự thường                               ║
│   ╚═══════════════════════════════════════════════════════╝
└─► handle_normal_letter(key, caps)
    ├── [is_letter(key)?]
    │   └── YES ──► push to buffer ──► return NONE
    └── NO ──► clear buffer ──► return NONE
```

---

## 4. INPUT METHOD RULES - DECISION TREE

### 4.1 Telex

```
TELEX INPUT METHOD
│
├── DẤU THANH (is_mark)
│   ├── S ──► 1 (sắc)   ─► á
│   ├── F ──► 2 (huyền) ─► à
│   ├── R ──► 3 (hỏi)   ─► ả
│   ├── X ──► 4 (ngã)   ─► ã
│   └── J ──► 5 (nặng)  ─► ạ
│
├── DẤU PHỤ (is_tone)
│   ├── [key == prev?]
│   │   ├── A + A ──► tone=1 ─► â
│   │   ├── E + E ──► tone=1 ─► ê
│   │   └── O + O ──► tone=1 ─► ô
│   │
│   └── [key == W?]
│       ├── prev=A ──► tone=2 ─► ă (trăng)
│       ├── prev=O ──► tone=2 ─► ơ (móc)
│       └── prev=U ──► tone=2 ─► ư (móc)
│
├── CHỮ Đ (is_d)
│   └── D + D ──► đ
│
└── XÓA DẤU (is_remove)
    └── Z ──► xóa dấu
```

### 4.2 VNI

```
VNI INPUT METHOD
│
├── DẤU THANH (is_mark)
│   ├── 1 ──► sắc   ─► á
│   ├── 2 ──► huyền ─► à
│   ├── 3 ──► hỏi   ─► ả
│   ├── 4 ──► ngã   ─► ã
│   └── 5 ──► nặng  ─► ạ
│
├── DẤU PHỤ (is_tone)
│   ├── 6 + [A|E|O] ──► tone=1 ─► â/ê/ô (mũ)
│   ├── 7 + [O|U]   ──► tone=2 ─► ơ/ư (móc)
│   └── 8 + A       ──► tone=2 ─► ă (trăng)
│
├── CHỮ Đ
│   ├── is_d: D + 9 ──► đ (tức thời)
│   └── is_d_for: buffer có 'd' + 9 ──► đ (delayed)
│       └── Ví dụ: dung9 ──► đung
│
└── XÓA DẤU (is_remove)
    └── 0 ──► xóa dấu
```

### 4.3 So sánh Telex vs VNI

```
┌────────────┬─────────────────┬─────────────────┐
│  Chức năng │      Telex      │       VNI       │
├────────────┼─────────────────┼─────────────────┤
│  sắc       │   s             │   1             │
│  huyền     │   f             │   2             │
│  hỏi       │   r             │   3             │
│  ngã       │   x             │   4             │
│  nặng      │   j             │   5             │
├────────────┼─────────────────┼─────────────────┤
│  mũ (^)    │   aa, ee, oo    │   a6, e6, o6    │
│  móc       │   ow, uw        │   o7, u7        │
│  trăng     │   aw            │   a8            │
├────────────┼─────────────────┼─────────────────┤
│  đ         │   dd            │   d9, delayed   │
│  xóa dấu   │   z             │   0             │
└────────────┴─────────────────┴─────────────────┘
```

---

## 5. THUẬT TOÁN ĐẶT DẤU THANH (PHONOLOGY)

### 5.1 Quy tắc tổng quát

```
find_tone_position(vowels, has_final, modern, has_qu)
│
├─► [vowels.len == 0?]
│   └── return 0
│
├─► [vowels.len == 1?]
│   └── return vowels[0].pos ......... Một nguyên âm: dấu trên nó
│
├─► [vowels.len == 2?]
│   └── (xem chi tiết 5.2)
│
├─► [vowels.len == 3?]
│   └── (xem chi tiết 5.3)
│
└─► [vowels.len >= 4?]
    ├── Tìm nguyên âm giữa có dấu phụ
    └── Mặc định: nguyên âm giữa
```

### 5.2 Decision Tree: 2 Nguyên âm

```
2 NGUYÊN ÂM (v1, v2)
│
├─► [has_final_consonant?] .............. Có phụ âm cuối?
│   └── YES ──► return v2.pos .......... toán, hoàn, tiến, biển
│
├─► [v1.has_diacritic && !v2.has_diacritic?]
│   └── YES ──► return v1.pos .......... ưa → mưa, sứa (dấu trên ư)
│
├─► [is_compound_vowel(v1, v2)?] ........ ươ, uô, iê
│   └── YES ──► return v2.pos .......... mười, muốn, biển
│
├─► [v2.has_diacritic?]
│   └── YES ──► return v2.pos .......... uê → thuế
│
├─► [is_medial_pair(v1, v2)?] ........... oa, oe, uy, uê, (ua với q)
│   └── YES ──► return modern ? v2 : v1  hoà, loé, qúa
│
├─► [v1=U && v2=A && !has_qu?] .......... ua không có q
│   └── YES ──► return v1.pos .......... mùa (dấu trên u)
│
├─► [is_main_glide_pair(v1, v2)?] ....... ai, ao, au, oi, ui
│   └── YES ──► return v1.pos .......... tài, sáo, bầu
│
└─► DEFAULT ──► return v2.pos
```

#### Chi tiết các hàm phụ:

```
is_compound_vowel(v1, v2)
├── (U, O) ──► true ......... ươ, uô
├── (I, E) ──► true ......... iê
└── else   ──► false

is_medial_pair(v1, v2, has_qu)
├── (U, A) && has_qu ──► true ... qua (u là âm đệm)
├── (O, A) ──► true ............ oa
├── (O, E) ──► true ............ oe
├── (U, E) ──► true ............ uê
├── (U, Y) ──► true ............ uy
└── else   ──► false

is_main_glide_pair(v1, v2)
├── v2 in [I, Y, O, U]? ........ Nguyên âm cuối là bán âm?
│   └── NO ──► false
├── is_medial_pair? ............ Loại trừ cặp âm đệm
│   └── YES ──► false
├── is_compound_vowel? ......... Loại trừ nguyên âm kép
│   └── YES ──► false
└── else ──► true
```

### 5.3 Decision Tree: 3 Nguyên âm

```
3 NGUYÊN ÂM (v0, v1, v2)
│
├─► [v1.has_diacritic?] ................ Nguyên âm giữa có dấu phụ?
│   └── YES ──► return v1.pos .......... ươi → mười, người (dấu trên ơ)
│
├─► [v2.has_diacritic?] ................ Nguyên âm cuối có dấu phụ?
│   └── YES ──► return v2.pos .......... uyê → khuyến (dấu trên ê)
│
├─► [v0=U && v1=O?] .................... Mẫu ươi, uôi
│   └── YES ──► return v1.pos .......... tuổi, chuối
│
├─► [v0=O && v1=A?] .................... Mẫu oai, oay
│   └── YES ──► return v1.pos .......... toại, ngoài
│
├─► [v0=U && v1=Y && v2=E?] ............ Mẫu uyê
│   └── YES ──► return v2.pos .......... khuyên
│
└─► DEFAULT ──► return mid.pos
```

### 5.4 Bảng Tổng hợp Quy tắc

```
┌─────────────────┬────────────────┬────────────────┬─────────────────┐
│      Mẫu        │  Phụ âm cuối   │  Vị trí dấu    │     Ví dụ       │
├─────────────────┼────────────────┼────────────────┼─────────────────┤
│ 1 nguyên âm    │       -        │   nguyên âm    │ á, è, ì, ọ      │
├─────────────────┼────────────────┼────────────────┼─────────────────┤
│ oa, oe, uy     │      Không     │   thứ 2 (a,e,y)│ hoà, loè, thuý  │
│ oa, oe, uy     │       Có       │   thứ 2        │ toán, hoàn      │
├─────────────────┼────────────────┼────────────────┼─────────────────┤
│ qua            │      Không     │   thứ 2 (a)    │ quá, qùa        │
│ ua (ko có q)   │      Không     │   thứ 1 (u)    │ mùa, của        │
├─────────────────┼────────────────┼────────────────┼─────────────────┤
│ ai, ao, au     │      Không     │   thứ 1        │ tài, sáo, bầu   │
│ oi, ui         │      Không     │   thứ 1        │ tôi, túi        │
├─────────────────┼────────────────┼────────────────┼─────────────────┤
│ ươ, uô, iê     │       -        │   thứ 2        │ mười, muốn      │
│ ưa             │      Không     │   thứ 1 (ư)    │ sứa, mưa        │
├─────────────────┼────────────────┼────────────────┼─────────────────┤
│ ươi, uôi       │       -        │   giữa (ơ,ô)   │ mười, tuổi      │
│ oai, oay       │       -        │   giữa (a)     │ toại, ngoài     │
│ uyê            │       -        │   cuối (ê)     │ khuyên, chuyện  │
└─────────────────┴────────────────┴────────────────┴─────────────────┘
```

---

## 6. CƠ CHẾ ĐẶC BIỆT

### 6.1 Double-Key Revert (Hoàn tác nhấn đúp)

```
DOUBLE-KEY REVERT
│
├── Lưu last_transform sau mỗi transformation
│   ├── Transform::Mark(key, mark_value)
│   └── Transform::Tone(key, tone_value, target_key)
│
└── Khi nhấn phím:
    │
    ├─► [last_transform.key == current_key?]
    │   └── YES ──► HOÀN TÁC
    │       ├── Xóa dấu phụ/thanh đã áp dụng
    │       ├── Thêm ký tự gốc vào output
    │       └── Clear last_transform
    │
    └── NO ──► Xử lý bình thường

VÍ DỤ:
┌─────────────┬──────────────────────────────────────┐
│   Input     │              Kết quả                 │
├─────────────┼──────────────────────────────────────┤
│ a + a       │ â (Transform::Tone saved)            │
│ â + a       │ aa (revert â → a, thêm 'a')          │
├─────────────┼──────────────────────────────────────┤
│ a + s       │ á (Transform::Mark saved)            │
│ á + s       │ as (revert á → a, thêm 's')          │
├─────────────┼──────────────────────────────────────┤
│ a + w       │ ă                                    │
│ ă + w       │ aw                                   │
└─────────────┴──────────────────────────────────────┘
```

### 6.2 Mark Repositioning (Di chuyển dấu thanh)

```
MARK REPOSITIONING
│
├── Trigger: Sau khi thêm dấu phụ (handle_tone)
│
├── Quy trình:
│   │
│   ├── 1. Tìm vị trí dấu thanh hiện tại
│   │      └── mark_info = find(c.mark > 0)
│   │
│   ├── 2. Thu thập nguyên âm MỚI (với dấu phụ mới)
│   │      └── vowels = collect_vowels()
│   │
│   ├── 3. Tính lại vị trí đúng
│   │      └── new_pos = Phonology::find_tone_position()
│   │
│   └── 4. Di chuyển nếu cần
│          ├── [new_pos != old_pos?]
│          │   ├── buffer[old_pos].mark = 0
│          │   └── buffer[new_pos].mark = mark_value
│          └── return Some(old_pos) để rebuild
│
└── Ví dụ:
    │
    │   Gõ "muois" (Telex):
    │   ├── m → u → o → i → s
    │   ├── Buffer: [m, u, o, i]
    │   ├── 's' → dấu sắc, vowels = [u, o, i]
    │   ├── find_tone_position → vị trí o (uoi → giữa)
    │   └── Kết quả: muói (?)
    │
    │   Tiếp tục gõ "w":
    │   ├── 'w' → uo thành ươ
    │   ├── Buffer: [m, ư, ớ, i] với dấu trên ơ
    │   ├── NHƯNG dấu đang trên o (chưa có móc)
    │   ├── Tính lại: ươi → dấu giữa (ơ)
    │   ├── old_pos=2 (o), new_pos=2 (ơ) → Cùng vị trí!
    │   └── Chỉ cần rebuild với tone mới
    │
    │   Thực tế:
    │   └── muối + w → mười
```

### 6.3 UO Compound (Nguyên âm kép ươ)

```
UO COMPOUND HANDLING
│
├── Trigger: Gõ 'w' (Telex) hoặc '7' (VNI) với mẫu uo trong buffer
│
├── Detection:
│   │
│   has_uo_compound()
│   ├── Duyệt buffer tìm nguyên âm liền kề
│   ├── [prev=U && curr=O?] ──► true (uo)
│   ├── [prev=O && curr=U?] ──► true (ou)
│   └── else ──► false
│
├── Processing:
│   │
│   find_eligible_vowels_for_tone(key, tone, target)
│   ├── [tone==2 && (key==W || key==7)?]
│   │   └── [has_uo_compound?]
│   │       └── YES ──► Áp dụng móc cho CẢ u VÀ o
│   │           ├── u → ư
│   │           └── o → ơ
│   │
│   └── else ──► Chỉ áp dụng cho target vowel
│
└── Ví dụ:
    │
    │   Gõ "truong" + "w":
    │   ├── Buffer: [t, r, u, o, n, g]
    │   ├── 'w' nhấn, tìm uo compound
    │   ├── Áp dụng tone=2 cho cả u và o
    │   │   ├── buffer[2].tone = 2 (u → ư)
    │   │   └── buffer[3].tone = 2 (o → ơ)
    │   └── Kết quả: "trương"
    │
    │   Gõ "nguoi" + "w" + "f":
    │   ├── nguoi + w → người (ư + ơ)
    │   ├── + f (huyền) → ngườì → ngườì
    │   └── Dấu huyền đặt trên ơ (giữa của ươi)
```

### 6.4 Qu Detection (Phân biệt qua vs mua)

```
QU DETECTION
│
├── Mục đích: Phân biệt vai trò của 'u'
│   │
│   ├── "qua" → q + u + a
│   │   └── u là ÂM ĐỆM → dấu trên 'a': quá
│   │
│   └── "mua" → m + u + a
│       └── u là NGUYÊN ÂM CHÍNH → dấu trên 'u': mùa
│
├── Algorithm:
│   │
│   has_qu_initial()
│   ├── Tìm 'u' đầu tiên trong buffer
│   ├── [i > 0?] ──► Kiểm tra ký tự trước
│   │   └── [prev.key == Q?]
│   │       ├── YES ──► return true
│   │       └── NO ──► return false
│   └── [i == 0?] ──► return false
│
└── Ảnh hưởng đến find_tone_position:
    │
    └── is_medial_pair(U, A, has_qu_initial)
        ├── has_qu=true ──► ua là âm đệm+âm chính → dấu trên a
        └── has_qu=false ──► ua là âm chính+bán âm → dấu trên u
```

---

## 7. CHARACTER COMPOSITION

### 7.1 Bảng Unicode Nguyên âm

```
VOWEL_TABLE
│
├── ('a', ['á', 'à', 'ả', 'ã', 'ạ'])
├── ('ă', ['ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'])
├── ('â', ['ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'])
├── ('e', ['é', 'è', 'ẻ', 'ẽ', 'ẹ'])
├── ('ê', ['ế', 'ề', 'ể', 'ễ', 'ệ'])
├── ('i', ['í', 'ì', 'ỉ', 'ĩ', 'ị'])
├── ('o', ['ó', 'ò', 'ỏ', 'õ', 'ọ'])
├── ('ô', ['ố', 'ồ', 'ổ', 'ỗ', 'ộ'])
├── ('ơ', ['ớ', 'ờ', 'ở', 'ỡ', 'ợ'])
├── ('u', ['ú', 'ù', 'ủ', 'ũ', 'ụ'])
├── ('ư', ['ứ', 'ừ', 'ử', 'ữ', 'ự'])
└── ('y', ['ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'])
          [0]  [1]  [2]  [3]  [4]
          sắc huyền hỏi  ngã nặng
```

### 7.2 Character Conversion Flow

```
to_char(key, caps, tone, mark)
│
├── 1. GET BASE CHAR
│   │
│   get_base_char(key, tone)
│   ├── key=A
│   │   ├── tone=0 → 'a'
│   │   ├── tone=1 → 'â' (mũ)
│   │   └── tone=2 → 'ă' (trăng)
│   ├── key=E
│   │   ├── tone=0 → 'e'
│   │   └── tone=1 → 'ê'
│   ├── key=I → 'i'
│   ├── key=O
│   │   ├── tone=0 → 'o'
│   │   ├── tone=1 → 'ô'
│   │   └── tone=2 → 'ơ'
│   ├── key=U
│   │   ├── tone=0 → 'u'
│   │   └── tone=2 → 'ư'
│   └── key=Y → 'y'
│
├── 2. APPLY MARK
│   │
│   apply_mark(base, mark)
│   ├── mark=0 → return base
│   └── mark>0 → lookup VOWEL_TABLE[base][mark-1]
│
└── 3. APPLY CASE
    │
    ├── caps=false → return as-is
    └── caps=true → return char.to_uppercase()

VÍ DỤ:
┌───────────────────────────────────────────────────┐
│  to_char(A, false, 1, 1)                          │
│  ├── get_base_char(A, 1) → 'â'                    │
│  ├── apply_mark('â', 1) → 'ấ' (sắc)               │
│  └── caps=false → 'ấ'                             │
└───────────────────────────────────────────────────┘
```

---

## 8. REBUILD OUTPUT

### 8.1 rebuild_from(pos) Algorithm

```
rebuild_from(pos)
│
├── Khởi tạo:
│   ├── output = []
│   └── backspace = 0
│
├── Duyệt buffer từ pos → cuối:
│   │
│   for i in pos..buffer.len()
│   │
│   ├── backspace += 1
│   │
│   ├── [char.key == D && char.stroke?]
│   │   └── output.push(đ hoặc Đ)
│   │
│   ├── [is_vowel(char.key)?]
│   │   └── output.push(to_char(key, caps, tone, mark))
│   │
│   └── [is_consonant(char.key)?]
│       └── output.push(key_to_char(key, caps))
│
└── Return Result::send(backspace, output)
```

### 8.2 Ví dụ: Gõ "Việt" (Telex)

```
GÕ "Việt" BẰNG TELEX
│
├── 'V' (caps)
│   ├── Stage 1-4: No match
│   ├── handle_normal_letter(V, true)
│   ├── Buffer: [V]
│   └── Output: "V"
│
├── 'i'
│   ├── Stage 1-4: No match
│   ├── handle_normal_letter(I, false)
│   ├── Buffer: [V, i]
│   └── Output: "Vi"
│
├── 'e'
│   ├── Stage 1-4: No match
│   ├── handle_normal_letter(E, false)
│   ├── Buffer: [V, i, e]
│   └── Output: "Vie"
│
├── 'e' (lần 2)
│   ├── Stage 2: is_tone_for(E, [i, e])?
│   │   └── Telex: ee → tone=1 (mũ), target=E
│   ├── handle_tone(E, 1, E)
│   │   ├── Tìm e tại pos=2
│   │   ├── buffer[2].tone = 1
│   │   └── Buffer: [V, i, ê]
│   ├── rebuild_from(2)
│   │   └── to_char(E, false, 1, 0) → 'ê'
│   └── Result: backspace=1, chars=['ê']
│   └── Output: "Viê"
│
├── 't'
│   ├── Stage 1-4: No match
│   ├── handle_normal_letter(T, false)
│   ├── Buffer: [V, i, ê, t]
│   └── Output: "Viêt"
│
└── 's'
    ├── Stage 3: is_mark(S) → Some(1) (sắc)
    ├── handle_mark(S, 1)
    │   ├── vowels = [i(pos=1), ê(pos=2)]
    │   ├── has_final_consonant(2) = true
    │   ├── ★ find_tone_position:
    │   │   ├── n=2, has_final=true
    │   │   └── return v2.pos = 2
    │   ├── buffer[2].mark = 1
    │   └── Buffer: [V, i, ế, t]
    ├── rebuild_from(2)
    │   ├── to_char(E, false, 1, 1) → 'ế'
    │   └── key_to_char(T, false) → 't'
    └── Result: backspace=2, chars=['ế', 't']
    └── Output: "Việt" ✓
```

---

## 9. VALIDATION ÂM TIẾT TIẾNG VIỆT

> **Tham khảo đầy đủ**: [vietnamese-language-system.md](./vietnamese-language-system.md) - Section 4.4, 6.5, và 12

### 9.1 Tại sao cần Validation?

```
MỤC ĐÍCH:
│
├── Xác định buffer hiện tại có phải là từ tiếng Việt hợp lệ
│   trước khi áp dụng transformation (dấu thanh/dấu phụ)
│
├── VÍ DỤ:
│   ├── "Duoc" + j → "Được" ✓ (tiếng Việt hợp lệ)
│   ├── "Clau" + s → "Claus" (không phải tiếng Việt - giữ nguyên)
│   ├── "HTTP" + s → "HTTPs" (không có nguyên âm - giữ nguyên)
│   └── "John" + s → "Johns" ("J" không có trong tiếng Việt)
│
└── LỢI ÍCH:
    ├── Tránh biến đổi từ tiếng Anh/từ mượn
    ├── Cho phép gõ code, email, URL không bị ảnh hưởng
    └── Tăng trải nghiệm người dùng
```

### 9.2 Decision Tree: Validation Pipeline

```
is_valid_vietnamese_syllable(buffer)
│
├─► STEP 1: Kiểm tra có nguyên âm không
│   ├── Không có nguyên âm → INVALID
│   └── Có nguyên âm → tiếp tục
│
├─► STEP 2: Xác định phụ âm đầu (C₁)
│   ├── Nếu có C₁:
│   │   ├── C₁ ∈ {b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x}? → OK
│   │   ├── C₁ ∈ {ch,gh,gi,kh,ng,nh,ph,qu,th,tr}? → OK
│   │   ├── C₁ = "ngh"? → OK
│   │   └── else → INVALID (vd: cl, bl, j, f, w, z)
│   │
│   └── Kiểm tra quy tắc chính tả:
│       ├── "c" trước e,ê,i,y? → INVALID (phải dùng "k")
│       ├── "k" trước a,ă,â,o,ô,ơ,u,ư? → INVALID (phải dùng "c")
│       ├── "g" trước e,ê,i? → INVALID (phải dùng "gh")
│       ├── "gh" trước a,ă,â,o,ô,ơ,u,ư? → INVALID
│       ├── "ng" trước e,ê,i? → INVALID (phải dùng "ngh")
│       └── "ngh" trước a,ă,â,o,ô,ơ,u,ư? → INVALID
│
├─► STEP 3: Xác định nguyên âm (V)
│   ├── Nguyên âm đơn: a,ă,â,e,ê,i,o,ô,ơ,u,ư,y
│   ├── Nguyên âm đôi: ai,ao,au,âu,ây,eo,êu,ia,iê,iu,oa,oă,oe...
│   └── Nguyên âm ba: iêu,yêu,ươi,ươu,uôi,oai,oay,oeo,uây,uyê
│
├─► STEP 4: Xác định âm cuối (C₂)
│   ├── Phụ âm cuối hợp lệ: c,ch,m,n,ng,nh,p,t
│   ├── Bán nguyên âm cuối: i,y,o,u
│   └── Kiểm tra kết hợp:
│       ├── -ch chỉ sau a,ă,ê,i
│       ├── -nh chỉ sau a,ă,ê,i,y
│       └── -ng không sau e,ê
│
└─► STEP 5: Kiểm tra quy tắc thanh điệu + âm cuối
    │
    └── Nếu có âm cuối tắc (p,t,c,ch):
        └── Chỉ cho phép thanh sắc hoặc nặng
            ├── ✓ cấp, cập, mát, mạt
            └── ✗ cảp, cãp, cap, càp (không tồn tại)
```

### 9.3 Danh sách Phụ âm đầu KHÔNG HỢP LỆ

```
INVALID_INITIALS - Reject ngay khi gặp:
│
├── Chữ cái không có trong tiếng Việt:
│   └── f, j, w, z
│
├── Cụm phụ âm (consonant clusters):
│   ├── *l: bl, cl, fl, gl, pl, sl
│   ├── *r: br, cr, dr, fr, gr, pr, str
│   ├── s*: sc, sk, sm, sn, sp, st, sw
│   └── *w: dw, tw, sw
│
└── Vi phạm quy tắc chính tả:
    ├── ce, ci (phải là ke, ki)
    ├── ka, ko (phải là ca, co)
    ├── nge, ngi (phải là nghe, nghi)
    └── gha, ngha (phải là ga, nga)
```

### 9.4 Quy tắc Thanh điệu + Âm cuối Tắc

```
TONE + FINAL STOP CONSONANT RULE
│
├── Âm cuối tắc: p, t, c, ch
│
├── CHỈ ĐƯỢC mang thanh sắc (1) hoặc nặng (5)
│   │
│   ├── ✓ Hợp lệ:
│   │   ├── cấp, cập (sắc, nặng + p)
│   │   ├── mát, mạt (sắc, nặng + t)
│   │   ├── các, cạc (sắc, nặng + c)
│   │   └── ách, ạch (sắc, nặng + ch)
│   │
│   └── ✗ KHÔNG hợp lệ:
│       ├── *cảp, *cãp, *cap, *càp (hỏi, ngã, ngang, huyền + p)
│       ├── *mảt, *mãt, *mat, *màt
│       ├── *cảc, *cãc, *cac, *càc
│       └── *ảch, *ãch, *ach, *àch
│
└── ÁP DỤNG:
    ├── Khi user gõ dấu thanh không hợp lệ với âm cuối tắc:
    │   ├── Không apply dấu
    │   └── Hoặc thông báo/ignore
    │
    └── VÍ DỤ:
        └── "cap" + r (hỏi) → không apply (không tồn tại *cảp)
```

### 9.5 Implementation Notes

```rust
// Suggested validation check before transformation

fn should_apply_transformation(buffer: &[Char], mark: Option<u8>) -> bool {
    // 1. Check if buffer is valid Vietnamese
    if !is_valid_vietnamese_syllable(buffer) {
        return false;
    }

    // 2. If applying mark (dấu thanh), check tone+final rule
    if let Some(mark_value) = mark {
        if let Some(final_c) = get_final_consonant(buffer) {
            if is_stop_consonant(final_c) {
                // Only allow sắc (1) or nặng (5)
                return matches!(mark_value, 1 | 5);
            }
        }
    }

    true
}

fn is_stop_consonant(c: &str) -> bool {
    matches!(c, "p" | "t" | "c" | "ch")
}
```

---

## 10. TÓM TẮT

```
GONHANH ENGINE SUMMARY
│
├── KIẾN TRÚC
│   ├── Phonology-based (không dùng lookup table)
│   ├── 4-stage pipeline (đ → tone → mark → remove)
│   └── Fixed buffer 32 chars
│
├── THUẬT TOÁN ĐẶT DẤU
│   ├── 1 nguyên âm → đặt trực tiếp
│   ├── 2 nguyên âm → 7+ quy tắc ngữ âm
│   ├── 3 nguyên âm → 5 priority rules
│   └── Qu detection cho qua vs mua
│
├── INPUT METHODS
│   ├── Telex: letters as modifiers (s, f, r, x, j, aa, aw)
│   └── VNI: numbers as modifiers (1-5, 6-9, 0)
│
├── CƠ CHẾ ĐẶC BIỆT
│   ├── Double-key revert (aaa → aa)
│   ├── Mark repositioning (di chuyển dấu thanh)
│   ├── UO compound (uo → ươ với cả u và o)
│   └── Delayed mode (VNI: dung9 → đung)
│
├── VALIDATION (ĐỀ XUẤT)
│   ├── Kiểm tra buffer có phải tiếng Việt hợp lệ
│   ├── Áp dụng quy tắc chính tả (c/k, g/gh, ng/ngh)
│   ├── Áp dụng quy tắc thanh điệu + âm cuối tắc
│   └── Tránh biến đổi từ tiếng Anh/code/URL
│
└── OUTPUT
    ├── Unicode precomposed characters
    ├── Backspace count + new chars
    └── Rebuild từ vị trí thay đổi
```

---

## Changelog

- **2025-12-08**: Bổ sung Section 9 - Validation Âm tiết Tiếng Việt
  - Thêm decision tree cho validation pipeline
  - Danh sách phụ âm đầu không hợp lệ
  - Quy tắc thanh điệu + âm cuối tắc
  - Implementation notes với pseudo-code
  - Liên kết đến vietnamese-language-system.md

- **2025-12-08**: Tạo tài liệu Decision Tree
  - Tổng quan cấu trúc engine
  - Cấu trúc dữ liệu (Char, Result)
  - 4-stage pipeline xử lý phím
  - Input method rules (Telex, VNI)
  - Thuật toán đặt dấu thanh (Phonology)
  - Các cơ chế đặc biệt (double-key revert, mark repositioning, UO compound, Qu detection)
  - Character composition và rebuild output

---

*Tài liệu được tạo từ phân tích source code GoNhanh Core Engine*
