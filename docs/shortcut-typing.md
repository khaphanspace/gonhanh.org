# Gõ tắt Tiếng Việt (Shortcut Typing)

> Tài liệu tổng hợp các tính năng gõ tắt từ các bộ gõ phổ biến và đề xuất xử lý trong GoNhanh Engine.

**Tài liệu liên quan**:
- [core-engine-algorithm-v2.md](./core-engine-algorithm-v2.md) - Thuật toán engine V2
- [validation-algorithm.md](./validation-algorithm.md) - Thuật toán validation

**Nguồn tham khảo**:
- [UniKey](https://www.unikey.org/) - Bộ gõ tiếng Việt phổ biến nhất
- [OpenKey](https://open-key.org/) - Bộ gõ tiếng Việt mã nguồn mở
- [EVKey](https://evkey.vn/) - Bộ gõ tiếng Việt
- [Laban Key](https://laban.vn/mlabankey/) - Bộ gõ tiếng Việt mobile
- [Wikipedia - Telex](https://vi.wikipedia.org/wiki/Telex_(kiểu_gõ)) - Kiểu gõ Telex

---

## 1. TỔNG QUAN CÁC LOẠI GÕ TẮT

```
CÁC LOẠI GÕ TẮT:
│
├── 1. GÕ TẮT NGUYÊN ÂM ĐƠN (Single-key Vowel Shortcuts) ★ MỚI
│   └── w→ư, [→ơ, ]→ư, {→Ơ, }→Ư
│
├── 2. GÕ TẮT PHỤ ÂM ĐẦU (Initial Shortcuts)
│   └── f→ph, j→gi, w→qu
│
├── 3. GÕ TẮT PHỤ ÂM CUỐI (Final Shortcuts)
│   └── g→ng, h→nh, k→ch
│
├── 4. QUICK TELEX (Double-key Shortcuts)
│   └── cc→ch, gg→gi, kk→kh, nn→ng, qq→qu, pp→ph, tt→th, uu→ươ
│
├── 5. SIMPLE TELEX (Telex giản lược) ★ MỚI
│   └── w đầu từ KHÔNG thành ư, chỉ uw/ow/aw mới có dấu
│
├── 6. MACRO / TỪ VIẾT TẮT (User-defined)
│   └── User tự định nghĩa: "ko"→"không", "dc"→"được"...
│
└── 7. AUTOCORRECT (Sửa lỗi chính tả)
    └── Tự động sửa lỗi phổ biến
```

---

## 2. GÕ TẮT NGUYÊN ÂM ĐƠN (Single-key Vowel Shortcuts)

> Đây là tính năng từ Telex mở rộng (Extended Telex) của BKED, UniKey và các bộ gõ khác.

### 2.1 Bảng gõ tắt nguyên âm đơn

```
┌───────────────────────────────────────────────────────────────────────┐
│              TELEX STANDARD - GÕ TẮT NGUYÊN ÂM                        │
├──────────┬──────────┬─────────────────────────────────────────────────┤
│  Input   │  Output  │  Ghi chú                                        │
├──────────┼──────────┼─────────────────────────────────────────────────┤
│    w     │    ư     │  w đơn lẻ (đầu từ hoặc sau phụ âm) → ư          │
│    W     │    Ư     │  W hoa → Ư hoa                                  │
│    [     │    ơ     │  Dấu ngoặc vuông mở → ơ                         │
│    ]     │    ư     │  Dấu ngoặc vuông đóng → ư                       │
│    {     │    Ơ     │  Shift + [ → Ơ hoa                              │
│    }     │    Ư     │  Shift + ] → Ư hoa                              │
└──────────┴──────────┴─────────────────────────────────────────────────┘

Nguồn: BKED (Quách Tuấn Ngọc), UniKey, OpenKey
```

### 2.2 So sánh các kiểu gõ

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    SO SÁNH CÁCH GÕ "ư" VÀ "ơ"                           │
├─────────────┬─────────────┬─────────────┬───────────────────────────────┤
│  Ký tự      │  Telex std  │  Telex ext  │  VNI                          │
├─────────────┼─────────────┼─────────────┼───────────────────────────────┤
│     ư       │     uw      │   w hoặc ]  │    u7                         │
│     Ư       │     Uw      │   W hoặc }  │    U7                         │
│     ơ       │     ow      │      [      │    o7                         │
│     Ơ       │     Ow      │      {      │    O7                         │
│     ă       │     aw      │      aw     │    a8                         │
│     â       │     aa      │      aa     │    a6                         │
│     ê       │     ee      │      ee     │    e6                         │
│     ô       │     oo      │      oo     │    o6                         │
│     đ       │     dd      │      dd     │    d9                         │
└─────────────┴─────────────┴─────────────┴───────────────────────────────┘
```

### 2.3 Ví dụ sử dụng

```
VÍ DỤ GÕ "Người Việt" VỚI CÁC KIỂU KHÁC NHAU:
│
├── Telex chuẩn:  "Nguwowif Vieetj"
│
├── Telex mở rộng (với [ ] w):  "Ng][fi Vieejt"
│   └── Ng + ] + [ + f + i = Người
│
└── VNI: "Nguo71telexi72 Vie65tj5"
```

### 2.4 Vấn đề với "w" đơn lẻ

```
VẤN ĐỀ: "w" đơn lẻ tự động thành "ư"
│
├── MONG MUỐN: Gõ "www" → "www" (địa chỉ web)
│   └── THỰC TẾ: "www" → "ưưư" (với Telex mặc định)
│
├── MONG MUỐN: Gõ "wifi" → "wifi"
│   └── THỰC TẾ: "wifi" → "ưifi" (với Telex mặc định)
│
├── GIẢI PHÁP 1: Double-key revert
│   └── "ww" → "w" (gõ w hai lần để có w thường)
│
├── GIẢI PHÁP 2: Simple Telex (UniKey 4.6 RC2+)
│   └── w đầu từ KHÔNG đổi thành ư
│   └── Chỉ "uw", "ow", "aw" mới có dấu móc/trăng
│
└── GIẢI PHÁP 3: Validation
    └── "wifi" không phải tiếng Việt → không transform
```

---

## 3. SIMPLE TELEX (Telex Giản lược)

> Có từ UniKey 4.6 RC2 (2023) - giải quyết vấn đề w→ư gây khó chịu.

### 3.1 Nguyên tắc Simple Telex

```
SIMPLE TELEX - KHÁC BIỆT VỚI TELEX CHUẨN:
│
├── w ĐẦU TỪ → KHÔNG đổi thành ư
│   ├── "wifi" → "wifi" ✓ (không thành "ưifi")
│   ├── "www" → "www" ✓
│   └── "word" → "word" ✓
│
├── w SAU NGUYÊN ÂM → VẪN có dấu móc/trăng
│   ├── "uw" → "ư" ✓
│   ├── "ow" → "ơ" ✓
│   └── "aw" → "ă" ✓
│
├── [ ] { } → KHÔNG tạo ơ ư
│   └── Gõ dấu ngoặc bình thường
│
└── Các phím khác → Như Telex chuẩn
    ├── aa → â
    ├── ee → ê
    ├── oo → ô
    ├── dd → đ
    └── s,f,r,x,j → dấu thanh
```

### 3.2 Bảng so sánh Telex vs Simple Telex

```
┌─────────────────────────────────────────────────────────────────────────┐
│              TELEX CHUẨN vs SIMPLE TELEX                                │
├─────────────┬─────────────────────┬─────────────────────────────────────┤
│   Input     │   Telex chuẩn       │   Simple Telex                      │
├─────────────┼─────────────────────┼─────────────────────────────────────┤
│   w         │   ư                 │   w (giữ nguyên nếu đầu từ)         │
│   W         │   Ư                 │   W (giữ nguyên nếu đầu từ)         │
│   [         │   ơ                 │   [ (dấu ngoặc thường)              │
│   ]         │   ư                 │   ] (dấu ngoặc thường)              │
│   {         │   Ơ                 │   { (dấu ngoặc thường)              │
│   }         │   Ư                 │   } (dấu ngoặc thường)              │
├─────────────┼─────────────────────┼─────────────────────────────────────┤
│   uw        │   ư                 │   ư ✓ (vẫn hoạt động)               │
│   ow        │   ơ                 │   ơ ✓ (vẫn hoạt động)               │
│   aw        │   ă                 │   ă ✓ (vẫn hoạt động)               │
├─────────────┼─────────────────────┼─────────────────────────────────────┤
│   wifi      │   ưifi (sai!)       │   wifi ✓                            │
│   www       │   ưưư (sai!)        │   www ✓                             │
│   tư        │   tư ✓              │   tư ✓                              │
│   mưa       │   mưa ✓             │   mưa ✓                             │
└─────────────┴─────────────────────┴─────────────────────────────────────┘
```

---

## 4. GÕ TẮT PHỤ ÂM ĐẦU (Initial Shortcuts)

### 4.1 Bảng gõ tắt

```
┌──────────┬──────────┬─────────────────────────────────────┐
│  Input   │  Output  │              Ví dụ                  │
├──────────┼──────────┼─────────────────────────────────────┤
│    f     │    ph    │ fa → pha, fo → pho, fong → phong   │
│    j     │    gi    │ ja → gia, jo → gio, jong → giông   │
│    w     │    qu    │ wa → qua, we → quê, wy → quy       │
└──────────┴──────────┴─────────────────────────────────────┘

Nguồn: OpenKey (từ bản 1.6)
```

### 4.2 Lý do và lợi ích

```
TẠI SAO CẦN GÕ TẮT PHỤ ÂM ĐẦU?
│
├── f, j, w KHÔNG có trong tiếng Việt chuẩn
│   └── Có thể tận dụng làm shortcut
│
├── Giảm số phím bấm:
│   ├── "ph" (2 phím) → "f" (1 phím)
│   ├── "gi" (2 phím) → "j" (1 phím)
│   └── "qu" (2 phím) → "w" (1 phím)
│
└── Ví dụ thực tế:
    ├── "phương" (6 phím) → "fuong" (5 phím) + w → "fương"
    ├── "giống" (5 phím) → "jong" (4 phím) + s → "jóng"
    └── "quên" (4 phím) → "wen" (3 phím) + "wên"
```

### 4.3 Quy tắc chi tiết

```
QUY TẮC GÕ TẮT PHỤ ÂM ĐẦU:
│
├── f → ph
│   ├── Trigger: f + nguyên âm
│   ├── fa → pha, fe → phe, fi → phi, fo → pho, fu → phu
│   ├── fai → phai, fong → phong, fung → phung
│   └── Không áp dụng: f đứng một mình hoặc cuối từ
│
├── j → gi
│   ├── Trigger: j + nguyên âm
│   ├── ja → gia, je → gie, jo → gio, ju → giu
│   ├── jang → giang, jong → giông
│   └── Lưu ý: "gi" + i,ê đặc biệt (giì, giê)
│
└── w → qu
    ├── Trigger: w + nguyên âm (thường là a, e, y, ê)
    ├── wa → qua, we → quê, wy → quy
    ├── wan → quan, wyen → quyên
    └── Lưu ý: w còn dùng cho dấu móc (ư, ơ) → cần phân biệt
```

---

## 5. GÕ TẮT PHỤ ÂM CUỐI (Final Shortcuts)

### 5.1 Bảng gõ tắt

```
┌──────────┬──────────┬─────────────────────────────────────┐
│  Input   │  Output  │              Ví dụ                  │
├──────────┼──────────┼─────────────────────────────────────┤
│    g     │    ng    │ ag → ang, og → ong, ug → ung       │
│    h     │    nh    │ ah → anh, eh → ênh, ih → inh       │
│    k     │    ch    │ ak → ach, ek → êch, ik → ich       │
└──────────┴──────────┴─────────────────────────────────────┘

Nguồn: OpenKey (từ bản 1.6)
```

### 5.2 Quy tắc chi tiết

```
QUY TẮC GÕ TẮT PHỤ ÂM CUỐI:
│
├── g → ng (ở cuối âm tiết)
│   ├── Trigger: nguyên âm + g (không có ký tự sau)
│   ├── ag → ang, ăg → ăng, âg → âng
│   ├── og → ong, ôg → ông, ơg → ơng
│   ├── ug → ung, ưg → ưng
│   ├── Ví dụ: "bag" → "bang", "sog" → "song"
│   └── KHÔNG áp dụng nếu g là phụ âm đầu: "ga", "go"
│
├── h → nh (ở cuối âm tiết)
│   ├── Trigger: nguyên âm hợp lệ + h
│   ├── CHỈ sau: a, ă, ê, i, y (theo quy tắc -nh)
│   ├── ah → anh, ăh → ănh, êh → ênh, ih → inh
│   ├── Ví dụ: "bah" → "banh", "mih" → "minh"
│   └── KHÔNG áp dụng: "oh", "uh" (không hợp lệ)
│
└── k → ch (ở cuối âm tiết)
    ├── Trigger: nguyên âm hợp lệ + k
    ├── CHỈ sau: a, ă, ê, i (theo quy tắc -ch)
    ├── ak → ach, ăk → ăch, êk → êch, ik → ich
    ├── Ví dụ: "sak" → "sách", "mêk" → "mêch"
    └── KHÔNG áp dụng: "ok", "uk" (không hợp lệ → giữ nguyên)
```

### 5.3 Validation cho Gõ tắt Cuối

```
PHẢI VALIDATE TRƯỚC KHI APPLY:
│
├── g → ng: Kiểm tra nguyên âm trước có hợp lệ với -ng không
│   ├── ✓ a, ă, â, o, ô, ơ, u, ư → OK
│   └── ✗ e, ê → KHÔNG (e, ê + ng không hợp lệ, dùng -nh)
│
├── h → nh: Kiểm tra nguyên âm trước có hợp lệ với -nh không
│   ├── ✓ a, ă, ê, i, y → OK
│   └── ✗ o, ô, ơ, u, ư → KHÔNG (giữ nguyên h)
│
└── k → ch: Kiểm tra nguyên âm trước có hợp lệ với -ch không
    ├── ✓ a, ă, ê, i → OK
    └── ✗ o, ô, ơ, u, ư, y → KHÔNG (giữ nguyên k)
```

---

## 6. QUICK TELEX (Double-key Shortcuts)

> Có trong OpenKey, Laban Key và một số bộ gõ mobile.

### 6.1 Bảng Quick Telex đầy đủ

```
┌───────────────────────────────────────────────────────────────────────┐
│              QUICK TELEX - BẢNG GÕ TẮT ĐẦY ĐỦ                         │
├──────────┬──────────┬─────────────────────────────────────────────────┤
│  Input   │  Output  │              Ví dụ                              │
├──────────┼──────────┼─────────────────────────────────────────────────┤
│    cc    │    ch    │ cca → cha, cco → cho, ccu → chu                │
│    gg    │    gi    │ gga → gia, ggo → gio                           │
│    kk    │    kh    │ kka → kha, kko → kho, kki → khi                │
│    nn    │    ng    │ nna → nga, nno → ngo, nnu → ngu                │
│    qq    │    qu    │ qqa → qua, qqe → quê                           │
│    pp    │    ph    │ ppa → pha, ppo → pho                           │
│    tt    │    th    │ tta → tha, tto → tho, tti → thi                │
├──────────┼──────────┼─────────────────────────────────────────────────┤
│    uu    │    ươ    │ Laban Key: uua → ươa, muu → mươ ★ ĐẶC BIỆT    │
└──────────┴──────────┴─────────────────────────────────────────────────┘

Nguồn: OpenKey, Laban Key
```

### 6.2 So sánh với Double-key Revert

```
QUAN TRỌNG: PHÂN BIỆT VỚI DOUBLE-KEY REVERT
│
├── QUICK TELEX (phụ âm):
│   ├── cc → ch (chuyển đổi phụ âm)
│   ├── gg → gi
│   └── Mục đích: Gõ nhanh phụ âm đôi
│
├── DOUBLE-KEY REVERT (nguyên âm - đã có trong Telex):
│   ├── aa → â (dấu mũ)
│   ├── ee → ê
│   ├── oo → ô
│   ├── aaa → aa (hoàn tác)
│   └── Mục đích: Thêm dấu phụ cho nguyên âm
│
└── KHÔNG XUNG ĐỘT vì:
    ├── Quick Telex: chỉ cho PHỤ ÂM (c, g, k, n, q, p, t)
    └── Double-key: chỉ cho NGUYÊN ÂM (a, e, o)
```

---

## 7. MACRO / TỪ VIẾT TẮT (User-defined)

### 7.1 Cơ chế Macro

```
MACRO DEFINITION:
│
├── User tự định nghĩa: keyword → replacement
│
├── Ví dụ phổ biến:
│   ├── "ko"  → "không"
│   ├── "dc"  → "được"
│   ├── "vs"  → "với"
│   ├── "ng"  → "người"
│   ├── "tnao" → "thế nào"
│   ├── "vd"  → "ví dụ"
│   ├── "tks" → "thanks"
│   ├── "CHXHCNVN" → "Cộng hòa Xã hội Chủ nghĩa Việt Nam"
│   └── ...
│
├── Trigger:
│   ├── UniKey/EVKey: Gõ keyword + space/enter
│   └── Hoặc ngay sau khi match
│
└── Lưu trữ:
    ├── File text: macro.txt
    └── Format: keyword=replacement (mỗi dòng 1 entry)
```

### 7.2 Phân biệt với Gõ tắt Phụ âm

```
MACRO vs GÕ TẮT PHỤ ÂM:
│
├── GÕ TẮT PHỤ ÂM (Built-in):
│   ├── Tự động, không cần space
│   ├── Dựa trên quy tắc ngôn ngữ
│   ├── f→ph, j→gi, w→qu
│   └── Hoạt động trong quá trình gõ
│
└── MACRO (User-defined):
    ├── User tự định nghĩa
    ├── Cần trigger (space/enter)
    ├── Từ viết tắt tùy ý
    └── Replace toàn bộ keyword
```

---

## 8. ĐỀ XUẤT XỬ LÝ TRONG GONHANH ENGINE

### 8.1 Vị trí trong Pipeline V2

```
PIPELINE V2 VỚI GÕ TẮT:
│
on_key(key, caps)
│
├─► [is_break(key)?] ──► check_macro() ──► clear buffer ──► return
│                        ↑
│                        Trigger macro khi space/enter
│
├─► [key == DELETE?] ──► pop buffer ──► return NONE
│
├─► ★ SHORTCUT LAYER (MỚI) ★
│   │
│   ├── [is_initial_shortcut(key)?]
│   │   ├── f, j, w ở đầu buffer?
│   │   └── Đánh dấu để transform sau
│   │
│   └── [is_final_shortcut(key)?]
│       ├── g, h, k sau nguyên âm?
│       └── Validate + transform nếu hợp lệ
│
├─► [is_modifier(key)?]
│   ├── Validate buffer
│   ├── Apply pattern replacement
│   └── Output
│
└─► [is_letter(key)?] ──► push to buffer ──► return NONE
```

### 8.2 Đề xuất: Shortcut Expansion Stage

```
SHORTCUT EXPANSION - Stage riêng TRƯỚC validation
│
├── TIMING: Khi nào expand?
│   │
│   ├── OPTION A: Expand ngay khi gõ (Eager)
│   │   ├── f + a → buffer = ['p', 'h', 'a']
│   │   ├── Ưu: Đơn giản, buffer luôn chuẩn
│   │   └── Nhược: Không revert được
│   │
│   ├── OPTION B: Expand khi gặp modifier (Lazy) ★ ĐỀ XUẤT
│   │   ├── f + a → buffer = ['f', 'a'] (giữ nguyên)
│   │   ├── f + a + s → expand 'f'→'ph' → validate "pha" → add sắc
│   │   ├── Ưu: Có thể revert, linh hoạt
│   │   └── Nhược: Phức tạp hơn
│   │
│   └── OPTION C: Expand khi kết thúc từ (Deferred)
│       ├── Chờ space/break mới expand
│       ├── Ưu: Toàn quyền kiểm soát
│       └── Nhược: User không thấy ngay kết quả
│
└── ĐỀ XUẤT: OPTION B (Lazy Expansion)
```

### 8.3 Thuật toán Shortcut Expansion

```rust
/// Shortcut types
enum ShortcutType {
    InitialF,   // f → ph
    InitialJ,   // j → gi
    InitialW,   // w → qu (cần phân biệt với dấu móc)
    FinalG,     // g → ng
    FinalH,     // h → nh
    FinalK,     // k → ch
    QuickTelex, // cc→ch, gg→gi, etc.
}

/// Expand shortcuts in buffer before validation
fn expand_shortcuts(buffer: &mut Vec<Char>) -> bool {
    let mut changed = false;

    // 1. Expand initial shortcuts (f, j, w at start)
    if let Some(first) = buffer.first() {
        match first.key {
            'f' => {
                // f → ph
                buffer[0].key = 'p';
                buffer.insert(1, Char::new('h'));
                changed = true;
            }
            'j' => {
                // j → gi
                buffer[0].key = 'g';
                buffer.insert(1, Char::new('i'));
                changed = true;
            }
            'w' => {
                // w → qu (chỉ khi theo sau là nguyên âm thích hợp)
                if buffer.len() > 1 && is_qu_compatible_vowel(buffer[1].key) {
                    buffer[0].key = 'q';
                    buffer.insert(1, Char::new('u'));
                    changed = true;
                }
            }
            _ => {}
        }
    }

    // 2. Expand final shortcuts (g, h, k at end)
    if let Some(last) = buffer.last() {
        let prev_vowel = get_last_vowel(buffer);
        match last.key {
            'g' if is_valid_for_ng(prev_vowel) => {
                // g → ng
                let pos = buffer.len() - 1;
                buffer[pos].key = 'n';
                buffer.push(Char::new('g'));
                changed = true;
            }
            'h' if is_valid_for_nh(prev_vowel) => {
                // h → nh
                let pos = buffer.len() - 1;
                buffer[pos].key = 'n';
                buffer.push(Char::new('h'));
                changed = true;
            }
            'k' if is_valid_for_ch(prev_vowel) => {
                // k → ch
                let pos = buffer.len() - 1;
                buffer[pos].key = 'c';
                buffer.push(Char::new('h'));
                changed = true;
            }
            _ => {}
        }
    }

    // 3. Expand Quick Telex (cc, gg, kk, nn, qq, pp, tt)
    changed |= expand_quick_telex(buffer);

    changed
}

/// Validate vowel for final shortcuts
fn is_valid_for_ng(vowel: char) -> bool {
    // -ng hợp lệ sau: a, ă, â, o, ô, ơ, u, ư
    // KHÔNG hợp lệ sau: e, ê
    !matches!(vowel, 'e' | 'ê')
}

fn is_valid_for_nh(vowel: char) -> bool {
    // -nh chỉ hợp lệ sau: a, ă, ê, i, y
    matches!(vowel, 'a' | 'ă' | 'ê' | 'i' | 'y')
}

fn is_valid_for_ch(vowel: char) -> bool {
    // -ch chỉ hợp lệ sau: a, ă, ê, i
    matches!(vowel, 'a' | 'ă' | 'ê' | 'i')
}
```

### 8.4 Xử lý xung đột "w"

```
VẤN ĐỀ: "w" có 2 nghĩa trong Telex
│
├── 1. Dấu móc: aw→ă, ow→ơ, uw→ư
└── 2. Gõ tắt: w→qu (wa→qua)

GIẢI PHÁP: Context-based Decision
│
├── RULE 1: Nếu "w" đứng SAU nguyên âm → dấu móc
│   ├── "aw" → "ă"
│   ├── "ow" → "ơ"
│   └── "uw" → "ư"
│
├── RULE 2: Nếu "w" đứng ĐẦU + theo sau là nguyên âm qu-compatible → gõ tắt
│   ├── "wa" → "qua"
│   ├── "we" → "que"
│   └── "wy" → "quy"
│
├── RULE 3: Nếu "w" trong pattern "uow" → compound ươ
│   └── "truongw" → u+o thành ươ (không phải qu)
│
└── PRIORITY:
    1. Kiểm tra dấu móc trước (sau nguyên âm)
    2. Kiểm tra gõ tắt qu (đầu từ)
    3. Default: thêm vào buffer như bình thường

ALGORITHM:
fn handle_w(buffer, key='w') {
    if buffer.is_empty() {
        // w đầu từ → có thể là gõ tắt qu
        mark_potential_qu_shortcut();
        push_to_buffer('w');
    } else if last_is_vowel_for_hook() {
        // w sau a, o, u → dấu móc
        apply_hook_mark();
    } else if has_uo_compound() {
        // uo + w → ươ
        apply_uo_compound();
    } else {
        push_to_buffer('w');
    }
}
```

### 8.5 Cấu hình On/Off

```
SHORTCUTS CONFIG:
│
├── initial_shortcuts: bool  // f→ph, j→gi, w→qu
├── final_shortcuts: bool    // g→ng, h→nh, k→ch
├── quick_telex: bool        // cc→ch, gg→gi, etc.
├── allow_fjwz_as_is: bool   // Cho phép f,j,w,z như chữ cái
└── macro_enabled: bool      // Bật/tắt macro

DEFAULT:
├── initial_shortcuts = true
├── final_shortcuts = true
├── quick_telex = false (optional, user bật nếu muốn)
├── allow_fjwz_as_is = true (cho từ mượn)
└── macro_enabled = true
```

---

## 9. INTEGRATION FLOW

### 9.1 Complete Pipeline với Shortcuts

```
COMPLETE PIPELINE V2 + SHORTCUTS:
│
on_key(key, caps)
│
├─► STAGE 0: Break/Special Keys
│   ├── [is_break?] → trigger macro check → clear buffer
│   └── [DELETE?] → pop buffer
│
├─► STAGE 1: Shortcut Detection (NEW)
│   │
│   ├── [Initial shortcut? (f/j/w at start)]
│   │   └── Mark for lazy expansion
│   │
│   ├── [Final shortcut? (g/h/k after vowel)]
│   │   └── Validate + expand if valid
│   │
│   └── [Quick Telex? (cc/gg/kk/nn/qq/pp/tt)]
│       └── Expand double consonants
│
├─► STAGE 2: Modifier Detection
│   │
│   ├── [is_modifier(key)?]
│   │   │
│   │   ├── EXPAND SHORTCUTS (lazy expansion point)
│   │   │
│   │   ├── VALIDATE buffer (with expanded shortcuts)
│   │   │   └── Invalid → return NONE, add key normally
│   │   │
│   │   ├── Apply tone/mark transformation
│   │   │
│   │   └── Validate tone + stop final rule
│   │
│   └── Continue if not modifier
│
├─► STAGE 3: Normal Letter
│   └── Push to buffer
│
└─► RETURN result
```

### 9.2 Ví dụ Luồng xử lý

```
VÍ DỤ 1: "fas" → "phá"
│
├── 'f' → push buffer=['f'], mark initial shortcut
├── 'a' → push buffer=['f','a']
├── 's' → is_modifier
│   ├── Expand shortcuts: ['f','a'] → ['p','h','a']
│   ├── Validate "pha" → VALID
│   ├── Apply sắc → "phá"
│   └── Output: "phá"
│
└── Result: "phá" ✓

VÍ DỤ 2: "bag" → "bang"
│
├── 'b' → push buffer=['b']
├── 'a' → push buffer=['b','a']
├── 'g' → is final shortcut?
│   ├── prev vowel = 'a' → valid for ng
│   ├── Expand: ['b','a','g'] → ['b','a','n','g']
│   └── Output: "bang"
│
└── Result: "bang" ✓

VÍ DỤ 3: "wes" → "qués" hoặc "qué"
│
├── 'w' → push buffer=['w'], mark potential qu
├── 'e' → push buffer=['w','e']
├── 's' → is_modifier
│   ├── Expand shortcuts: ['w','e'] → ['q','u','e']
│   │   └── Wait, 'e' after 'qu' should be 'ê'?
│   │       Actually "que" is valid, becomes "qué"
│   ├── Validate "que" → VALID
│   ├── Apply sắc → "qué"
│   └── Output: "qué"
│
└── Result: "qué" ✓

VÍ DỤ 4: "aws" → "ăs" (không phải "qua" + s)
│
├── 'a' → push buffer=['a']
├── 'w' → is after vowel 'a' → dấu móc (ă)
│   └── buffer=['ă']
├── 's' → is_modifier
│   ├── Validate "ă" → VALID
│   ├── Apply sắc → "ắ"
│   └── Output: "ắ"
│
└── Result: "ắ" ✓ (w là dấu móc, không phải gõ tắt qu)
```

---

## 10. EDGE CASES VÀ CONFLICT RESOLUTION

### 10.1 Xung đột có thể xảy ra

```
CONFLICTS:
│
├── w: dấu móc vs gõ tắt qu
│   ├── "aw" → ă (móc)
│   ├── "wa" → qua (gõ tắt)
│   └── Resolution: Vị trí quyết định (sau vowel = móc, đầu = qu)
│
├── g: âm cuối ng vs phụ âm đầu g
│   ├── "bag" → bang (cuối)
│   ├── "ga" → ga (đầu, không đổi)
│   └── Resolution: Chỉ expand ở cuối từ (sau nguyên âm)
│
├── h: âm cuối nh vs phụ âm đầu h
│   ├── "ah" → anh (cuối)
│   ├── "ha" → ha (đầu, không đổi)
│   └── Resolution: Chỉ expand ở cuối từ
│
├── Quick Telex vs Double-key Revert
│   ├── "cc" → ch (Quick Telex cho phụ âm)
│   ├── "aa" → â (Double-key cho nguyên âm)
│   └── Resolution: Không xung đột (phụ âm vs nguyên âm)
│
└── Gõ tắt vs Từ mượn
    ├── "file" - muốn giữ nguyên, không thành "phile"
    ├── "jazz" - muốn giữ nguyên
    └── Resolution: Config allow_fjwz_as_is + validation
```

### 10.2 Quy tắc ưu tiên

```
PRIORITY ORDER:
│
1. Dấu móc (w sau nguyên âm) - HIGHEST
2. UO compound (w với pattern uo)
3. Gõ tắt phụ âm đầu (f, j, w đầu từ)
4. Gõ tắt phụ âm cuối (g, h, k cuối từ)
5. Quick Telex (double consonants)
6. Macro (keyword matching)
7. Normal letter - LOWEST
```

---

## 11. BẢNG TỔNG HỢP SO SÁNH CÁC KIỂU GÕ

> So sánh chi tiết giữa **Telex chuẩn**, **Simple Telex** (UniKey 4.6+), và **GoNhanh Telex** (đề xuất).

### 11.1 Bảng so sánh tổng hợp

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                           BẢNG SO SÁNH: TELEX vs SIMPLE TELEX vs GONHANH TELEX                          │
├─────────────┬─────────────────────┬─────────────────────┬─────────────────────┬─────────────────────────┤
│   Input     │   Telex chuẩn       │   Simple Telex      │   GoNhanh Telex     │   Ghi chú               │
├─────────────┴─────────────────────┴─────────────────────┴─────────────────────┴─────────────────────────┤
│                                     NGUYÊN ÂM CÓ DẤU PHỤ                                                │
├─────────────┬─────────────────────┬─────────────────────┬─────────────────────┬─────────────────────────┤
│   aa        │         â           │         â           │         â           │ Double-key              │
│   aw        │         ă           │         ă           │         ă           │ Hook mark               │
│   ee        │         ê           │         ê           │         ê           │ Double-key              │
│   oo        │         ô           │         ô           │         ô           │ Double-key              │
│   ow        │         ơ           │         ơ           │         ơ           │ Hook mark               │
│   uw        │         ư           │         ư           │         ư           │ Hook mark               │
│   dd        │         đ           │         đ           │         đ           │ Double-key              │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│                                     NGUYÊN ÂM ĐƠN TẮT (w, [, ])                                         │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│   w (đầu)   │         ư           │         w           │    ư hoặc qu ★      │ GoNhanh: context-aware  │
│   w (giữa)  │         ư           │         w           │         ư           │ Sau phụ âm → ư          │
│   [         │         ơ           │         [           │         ơ           │ Extended Telex          │
│   ]         │         ư           │         ]           │         ư           │ Extended Telex          │
│   {         │         Ơ           │         {           │         Ơ           │ Extended Telex (hoa)    │
│   }         │         Ư           │         }           │         Ư           │ Extended Telex (hoa)    │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│                                     DẤU THANH (TONE MARKS)                                              │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│   s         │       sắc (´)       │       sắc (´)       │       sắc (´)       │                         │
│   f         │       huyền (`)     │       huyền (`)     │       huyền (`)     │                         │
│   r         │       hỏi (ˀ)       │       hỏi (ˀ)       │       hỏi (ˀ)       │                         │
│   x         │       ngã (~)       │       ngã (~)       │       ngã (~)       │                         │
│   j         │       nặng (.)      │       nặng (.)      │       nặng (.)      │                         │
│   z         │       xóa dấu       │       xóa dấu       │       xóa dấu       │ Remove tone             │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│                                  ★ GÕ TẮT PHỤ ÂM ĐẦU (Initial Shortcuts)                               │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│   f + V     │         -           │         -           │        ph           │ fa→pha, fo→pho         │
│   j + V     │         -           │         -           │        gi           │ ja→gia, jo→gio         │
│   w + V     │         -           │         -           │        qu           │ wa→qua, wy→quy         │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│                                  ★ GÕ TẮT PHỤ ÂM CUỐI (Final Shortcuts)                                │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│   V + g     │         -           │         -           │        ng           │ ag→ang, og→ong         │
│   V + h     │         -           │         -           │        nh           │ ah→anh, ih→inh         │
│   V + k     │         -           │         -           │        ch           │ ak→ach, ik→ich         │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│                                  ★ QUICK TELEX (Double Consonants)                                      │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│   cc        │         -           │         -           │        ch           │ cca→cha                │
│   gg        │         -           │         -           │        gi           │ gga→gia                │
│   kk        │         -           │         -           │        kh           │ kka→kha                │
│   nn        │         -           │         -           │        ng           │ nna→nga                │
│   qq        │         -           │         -           │        qu           │ qqa→qua                │
│   pp        │         -           │         -           │        ph           │ ppa→pha                │
│   tt        │         -           │         -           │        th           │ tta→tha                │
│   uu        │         -           │         -           │        ươ           │ Laban Key style         │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│                                  REVERT / UNDO (Hoàn tác)                                               │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│   aaa       │        aa           │        aa           │        aa           │ â → aa                  │
│   eee       │        ee           │        ee           │        ee           │ ê → ee                  │
│   ooo       │        oo           │        oo           │        oo           │ ô → oo                  │
│   ddd       │        dd           │        dd           │        dd           │ đ → dd                  │
│   ww        │        w            │        ww           │        w            │ ư → w (Telex chuẩn)     │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│                                  TỪ TIẾNG ANH / TỪ MƯỢN                                                 │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│   wifi      │       ưifi ✗        │       wifi ✓        │       wifi ✓        │ Validation-based        │
│   www       │       ưưư ✗         │       www ✓         │       www ✓         │ Validation-based        │
│   file      │       phile ✗       │       file ✓        │       file ✓        │ Validation-based        │
│   jazz      │       giazz ✗       │       jazz ✓        │       jazz ✓        │ Validation-based        │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│                                  VÍ DỤ THỰC TẾ                                                          │
├─────────────┼─────────────────────┼─────────────────────┼─────────────────────┼─────────────────────────┤
│ Việt Nam    │   Vieetj Namn       │   Vieetj Namn       │   Vieetj Namn       │ Giống nhau              │
│ người       │   nguwofi           │   nguwofi           │   ng][fi hoặc njfi  │ GoNhanh: nhiều cách     │
│ phương      │   phuowng           │   phuowng           │   f]g hoặc fuowng   │ GoNhanh: gõ tắt f       │
│ quốc        │   quoocs            │   quoocs            │   wocs hoặc quoocs  │ GoNhanh: gõ tắt w       │
│ giống       │   gioongs           │   gioongs           │   jogs hoặc gioongs │ GoNhanh: gõ tắt j       │
│ khoảng      │   khoangr           │   khoangr           │   kkagr hoặc khoagr │ GoNhanh: Quick Telex    │
│ thành       │   thanhf            │   thanhf            │   ttahf hoặc thahf  │ GoNhanh: Quick Telex    │
│ nhanh       │   nhanhh            │   nhanhh            │   nnahh hoặc nhahh  │ GoNhanh: Quick Telex    │
└─────────────┴─────────────────────┴─────────────────────┴─────────────────────┴─────────────────────────┘

Chú thích:
- V = nguyên âm (vowel)
- ★ = Tính năng mới của GoNhanh
- ✓ = Kết quả đúng
- ✗ = Kết quả sai/không mong muốn
- "-" = Không hỗ trợ
```

### 11.2 Bảng tổng hợp số phím bấm

```
┌────────────────────────────────────────────────────────────────────────────────────────────────┐
│                           SO SÁNH SỐ PHÍM BẤM (KEYSTROKE COUNT)                                │
├─────────────────┬──────────────────┬──────────────────┬──────────────────┬─────────────────────┤
│   Từ cần gõ     │   Telex chuẩn    │   Simple Telex   │   GoNhanh Telex  │   Tiết kiệm         │
├─────────────────┼──────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│   phương        │   phuowng (7)    │   phuowng (7)    │   f]g (3)        │   -4 phím (57%)     │
│   người         │   nguwofi (7)    │   nguwofi (7)    │   njfi (4)       │   -3 phím (43%)     │
│   quốc          │   quoocs (6)     │   quoocs (6)     │   wocs (4)       │   -2 phím (33%)     │
│   giống         │   gioongs (7)    │   gioongs (7)    │   jogs (4)       │   -3 phím (43%)     │
│   không         │   khoong (6)     │   khoong (6)     │   kkog (4)       │   -2 phím (33%)     │
│   thanh         │   thanhh (6)     │   thanhh (6)     │   ttahh (5)      │   -1 phím (17%)     │
│   ngành         │   nganhf (6)     │   nganhf (6)     │   nnahf (5)      │   -1 phím (17%)     │
│   chương        │   chuowng (7)    │   chuowng (7)    │   cc]g (4)       │   -3 phím (43%)     │
│   nghiệp        │   nghieepj (8)   │   nghieepj (8)   │   nnjepj (6)     │   -2 phím (25%)     │
├─────────────────┼──────────────────┼──────────────────┼──────────────────┼─────────────────────┤
│   TRUNG BÌNH    │      6.7         │      6.7         │      4.3         │   -2.4 phím (36%)   │
└─────────────────┴──────────────────┴──────────────────┴──────────────────┴─────────────────────┘
```

### 11.3 Bảng chi tiết GoNhanh Telex shortcuts

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                              GONHANH TELEX - BẢNG GÕ TẮT CHI TIẾT                               │
├─────────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────┐   │
│  │                           PHỤ ÂM ĐẦU (Initial Shortcuts)                                │   │
│  ├──────────┬──────────┬───────────────────────────────────────────────────────────────────┤   │
│  │  Input   │  Output  │  Điều kiện & Ví dụ                                                │   │
│  ├──────────┼──────────┼───────────────────────────────────────────────────────────────────┤   │
│  │    f     │    ph    │  f + nguyên âm: fa→pha, fo→pho, fong→phong, fuowng→phương        │   │
│  │    j     │    gi    │  j + nguyên âm: ja→gia, jo→gio, jang→giang, jong→giông          │   │
│  │    w     │    qu    │  w + (a,e,y,ê) ở ĐẦU: wa→qua, we→quê, wy→quy, wan→quan          │   │
│  └──────────┴──────────┴───────────────────────────────────────────────────────────────────┘   │
│                                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────┐   │
│  │                           PHỤ ÂM CUỐI (Final Shortcuts)                                 │   │
│  ├──────────┬──────────┬───────────────────────────────────────────────────────────────────┤   │
│  │  Input   │  Output  │  Điều kiện (nguyên âm trước) & Ví dụ                              │   │
│  ├──────────┼──────────┼───────────────────────────────────────────────────────────────────┤   │
│  │    g     │    ng    │  Sau a,ă,â,o,ô,ơ,u,ư: ag→ang, og→ong, ug→ung, bag→bang          │   │
│  │    h     │    nh    │  Sau a,ă,ê,i,y: ah→anh, êh→ênh, ih→inh, bah→banh                │   │
│  │    k     │    ch    │  Sau a,ă,ê,i: ak→ach, êk→êch, ik→ich, sak→sách                  │   │
│  └──────────┴──────────┴───────────────────────────────────────────────────────────────────┘   │
│                                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────┐   │
│  │                           QUICK TELEX (Double Consonants)                               │   │
│  ├──────────┬──────────┬───────────────────────────────────────────────────────────────────┤   │
│  │  Input   │  Output  │  Ví dụ                                                            │   │
│  ├──────────┼──────────┼───────────────────────────────────────────────────────────────────┤   │
│  │    cc    │    ch    │  cca→cha, cco→cho, ccu→chu, ccuowng→chương                       │   │
│  │    gg    │    gi    │  gga→gia, ggo→gio, ggawng→giăng                                  │   │
│  │    kk    │    kh    │  kka→kha, kko→kho, kkoang→khoang                                 │   │
│  │    nn    │    ng    │  nna→nga, nno→ngo, nnahf→ngành                                   │   │
│  │    qq    │    qu    │  qqa→qua, qqe→quê, qqocs→quốc                                    │   │
│  │    pp    │    ph    │  ppa→pha, ppo→pho, ppuowng→phương                                │   │
│  │    tt    │    th    │  tta→tha, tto→tho, ttahf→thành                                   │   │
│  │    uu    │    ươ    │  muu→mươ, luuwng→lượng, duua→dừa (Laban Key style)               │   │
│  └──────────┴──────────┴───────────────────────────────────────────────────────────────────┘   │
│                                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────────────────────┐   │
│  │                           NGUYÊN ÂM TẮT (Single-key Vowels)                             │   │
│  ├──────────┬──────────┬───────────────────────────────────────────────────────────────────┤   │
│  │  Input   │  Output  │  Ghi chú                                                          │   │
│  ├──────────┼──────────┼───────────────────────────────────────────────────────────────────┤   │
│  │    [     │    ơ     │  Dấu ngoặc vuông mở → ơ                                           │   │
│  │    ]     │    ư     │  Dấu ngoặc vuông đóng → ư                                         │   │
│  │    {     │    Ơ     │  Shift + [ → Ơ hoa                                                │   │
│  │    }     │    Ư     │  Shift + ] → Ư hoa                                                │   │
│  │  w(giữa) │    ư     │  w sau phụ âm (không phải đầu từ) → ư                             │   │
│  └──────────┴──────────┴───────────────────────────────────────────────────────────────────┘   │
│                                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### 11.4 Không áp dụng Shortcuts khi

```
KHÔNG EXPAND KHI:
│
├── Validation fail (không phải tiếng Việt hợp lệ)
│   └── "file" → giữ "file", không thành "phile"
│
├── User config tắt shortcuts
│
├── Chế độ tiếng Anh (Vietnamese mode = off)
│
├── Từ mượn với allow_fjwz_as_is = true
│   └── "wifi", "zone", "jazz"
│
└── Nguyên âm sau không hợp lệ cho shortcut đó
    └── "wng" → không expand (ng không đi sau qu)
```

---

## 12. IMPLEMENTATION CHECKLIST

```
□ Data structures cho shortcuts
  □ ShortcutType enum
  □ Shortcut config struct

□ Detection functions
  □ is_initial_shortcut(key, buffer)
  □ is_final_shortcut(key, buffer)
  □ is_quick_telex(buffer)

□ Expansion functions
  □ expand_initial_shortcut(buffer)
  □ expand_final_shortcut(buffer)
  □ expand_quick_telex(buffer)

□ Validation integration
  □ Validate AFTER expansion
  □ Rollback if invalid

□ W conflict resolution
  □ Detect position (start vs after vowel)
  □ Priority: hook > qu shortcut

□ Config options
  □ initial_shortcuts on/off
  □ final_shortcuts on/off
  □ quick_telex on/off
  □ allow_fjwz_as_is

□ Macro system (separate module)
  □ Load macro definitions
  □ Trigger on break keys
  □ Replace keyword with value
```

---

## Changelog

- **2025-12-08**: Tạo tài liệu Shortcut Typing
  - Tổng hợp từ UniKey, EVKey, OpenKey
  - Gõ tắt phụ âm đầu (f→ph, j→gi, w→qu)
  - Gõ tắt phụ âm cuối (g→ng, h→nh, k→ch)
  - Quick Telex (cc→ch, gg→gi, etc.)
  - Đề xuất xử lý trong pipeline V2
  - Xử lý xung đột w (móc vs qu)
  - Integration flow và examples
  - Implementation checklist

---

*Tài liệu gõ tắt tiếng Việt cho GoNhanh Core Engine*
