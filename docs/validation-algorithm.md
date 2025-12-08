# Thuật toán Validation Âm tiết Tiếng Việt

> Thuật toán chi tiết để xác định một chuỗi ký tự có phải là âm tiết tiếng Việt hợp lệ hay không.

**Tài liệu liên quan**:
- [vietnamese-language-system.md](./vietnamese-language-system.md) - Hệ thống chữ viết tiếng Việt
- [core-engine-algorithm-v2.md](./core-engine-algorithm-v2.md) - Thuật toán engine V2

---

## 1. MỤC ĐÍCH

```
VALIDATION ĐỂ LÀM GÌ?
│
├── Xác định buffer có phải tiếng Việt TRƯỚC khi transform
│   ├── "duoc" + j → VALID → transform → "được" ✓
│   ├── "claus" + s → INVALID → không transform → "clauss" ✓
│   └── "http" + s → INVALID → không transform → "https" ✓
│
├── Bảo vệ:
│   ├── Code: function, const, let, HTTP, API
│   ├── Tên riêng nước ngoài: John, Mary, Claude
│   ├── Từ mượn: pizza, coffee, jazz
│   └── URL/Email: không bị transform
│
└── Tăng UX: Chỉ transform khi chắc chắn là tiếng Việt
```

---

## 2. CẤU TRÚC ÂM TIẾT TIẾNG VIỆT

```
Syllable = (C₁)(G)V(C₂)

Trong đó:
├── C₁ = Phụ âm đầu (Initial)     - TÙY CHỌN
├── G  = Âm đệm (Glide/Medial)    - TÙY CHỌN
├── V  = Nguyên âm (Vowel Nucleus) - BẮT BUỘC
└── C₂ = Âm cuối (Final)          - TÙY CHỌN

VÍ DỤ:
├── "a"      → V=a
├── "an"     → V=a, C₂=n
├── "ban"    → C₁=b, V=a, C₂=n
├── "hoa"    → C₁=h, G=o, V=a
├── "hoàn"   → C₁=h, G=o, V=a, C₂=n
├── "người"  → C₁=ng, V=ươ, C₂=i
├── "trường" → C₁=tr, V=ươ, C₂=ng
└── "nghiêng"→ C₁=ngh, V=iê, C₂=ng
```

---

## 3. DATA CONSTANTS

### 3.1 Phụ âm đầu hợp lệ (C₁)

```rust
/// Phụ âm đơn (17)
const INITIAL_SINGLE: &[&str] = &[
    "b", "c", "d", "g", "h", "k", "l", "m",
    "n", "p", "q", "r", "s", "t", "v", "x"
];
// Lưu ý: "đ" xử lý riêng vì là Unicode đặc biệt

/// Phụ âm đôi (10)
const INITIAL_DOUBLE: &[&str] = &[
    "ch", "gh", "gi", "kh", "ng", "nh", "ph", "qu", "th", "tr"
];

/// Phụ âm ba (1)
const INITIAL_TRIPLE: &[&str] = &["ngh"];

/// Tất cả phụ âm đầu (sắp xếp theo độ dài giảm dần để match longest-first)
const ALL_INITIALS: &[&str] = &[
    // 3 chars
    "ngh",
    // 2 chars
    "ch", "gh", "gi", "kh", "ng", "nh", "ph", "qu", "th", "tr",
    // 1 char
    "b", "c", "d", "g", "h", "k", "l", "m", "n", "p", "q", "r", "s", "t", "v", "x"
];
```

### 3.2 Nguyên âm hợp lệ (V)

```rust
/// Nguyên âm đơn (12) - base form (không dấu thanh)
const VOWEL_SINGLE: &[&str] = &[
    "a", "ă", "â", "e", "ê", "i", "o", "ô", "ơ", "u", "ư", "y"
];

/// Nguyên âm đôi (26)
const VOWEL_DOUBLE: &[&str] = &[
    // Âm đệm + âm chính
    "oa", "oă", "oe", "oo", "ua", "uâ", "uê", "uy", "uơ",
    // Âm chính + bán nguyên âm
    "ai", "ao", "au", "ay", "âu", "ây",
    "eo", "êu",
    "ia", "iê", "iu",
    "oi", "ôi", "ơi",
    "ui", "ưi", "uo", "uô", "ươ", "ưa", "ya", "yê"
];

/// Nguyên âm ba (10)
const VOWEL_TRIPLE: &[&str] = &[
    "iêu", "yêu", "ươi", "ươu", "uôi",
    "oai", "oay", "oeo", "uây", "uyê", "uya"
];

/// Tất cả nguyên âm (sắp xếp longest-first)
const ALL_VOWELS: &[&str] = &[
    // 3 chars - nguyên âm ba
    "iêu", "yêu", "ươi", "ươu", "uôi", "oai", "oay", "oeo", "uây", "uyê", "uya",
    // 2 chars - nguyên âm đôi
    "oa", "oă", "oe", "oo", "ua", "uâ", "uê", "uy", "uơ",
    "ai", "ao", "au", "ay", "âu", "ây", "eo", "êu",
    "ia", "iê", "iu", "oi", "ôi", "ơi", "ui", "ưi",
    "uo", "uô", "ươ", "ưa", "ya", "yê",
    // 1 char - nguyên âm đơn
    "a", "ă", "â", "e", "ê", "i", "o", "ô", "ơ", "u", "ư", "y"
];
```

### 3.3 Âm cuối hợp lệ (C₂)

```rust
/// Phụ âm cuối (8)
const FINAL_CONSONANT: &[&str] = &[
    "ch", "ng", "nh",  // 2 chars first
    "c", "m", "n", "p", "t"  // 1 char
];

/// Bán nguyên âm cuối (4)
const FINAL_SEMIVOWEL: &[&str] = &["i", "y", "o", "u"];

/// Tất cả âm cuối (longest-first)
const ALL_FINALS: &[&str] = &[
    // 2 chars
    "ch", "ng", "nh",
    // 1 char
    "c", "m", "n", "p", "t", "i", "y", "o", "u"
];

/// Âm cuối tắc (chỉ cho thanh sắc/nặng)
const STOP_FINALS: &[&str] = &["p", "t", "c", "ch"];
```

### 3.4 Ký tự KHÔNG có trong tiếng Việt

```rust
/// Chữ cái không có trong tiếng Việt
const INVALID_CHARS: &[char] = &['f', 'j', 'w', 'z'];

/// Cụm phụ âm không hợp lệ (consonant clusters)
const INVALID_CLUSTERS: &[&str] = &[
    // *l combinations
    "bl", "cl", "fl", "gl", "pl", "sl",
    // *r combinations
    "br", "cr", "dr", "fr", "gr", "pr", "str",
    // s* combinations
    "sc", "sk", "sm", "sn", "sp", "st", "sw",
    // *w combinations
    "dw", "tw",
    // Khác
    "ck", "qu" // qu xử lý riêng như một đơn vị
];
```

---

## 4. THUẬT TOÁN PARSE SYLLABLE

### 4.1 Tổng quan

```
parse_syllable(input: &str) -> Option<Syllable>
│
├── STEP 1: Normalize input (lowercase, NFC)
├── STEP 2: Extract initial consonant (C₁) - longest match first
├── STEP 3: Extract vowel nucleus (V) - longest match first
├── STEP 4: Extract final (C₂) - longest match first
├── STEP 5: Validate structure
└── RETURN Syllable { initial, vowel, final }
```

### 4.2 Chi tiết thuật toán

```rust
struct Syllable {
    initial: Option<String>,  // C₁ - phụ âm đầu
    vowel: String,            // V  - nguyên âm (bắt buộc)
    final_c: Option<String>,  // C₂ - âm cuối
}

fn parse_syllable(input: &str) -> Option<Syllable> {
    // STEP 1: Normalize
    let s = normalize(input);  // lowercase, remove tone marks, NFC

    if s.is_empty() {
        return None;
    }

    let mut pos = 0;

    // STEP 2: Extract initial consonant (longest-first)
    let initial = extract_initial(&s, &mut pos);

    // STEP 3: Extract vowel (bắt buộc, longest-first)
    let vowel = extract_vowel(&s, &mut pos)?;  // None nếu không có

    // STEP 4: Extract final (nếu còn)
    let final_c = extract_final(&s, &mut pos);

    // STEP 5: Kiểm tra đã hết chuỗi chưa
    if pos != s.len() {
        return None;  // Còn ký tự thừa → invalid
    }

    Some(Syllable { initial, vowel, final_c })
}
```

### 4.3 Extract Initial (Phụ âm đầu)

```rust
fn extract_initial(s: &str, pos: &mut usize) -> Option<String> {
    let remaining = &s[*pos..];

    // Try longest first: 3 chars → 2 chars → 1 char

    // 3 chars: "ngh"
    if remaining.len() >= 3 {
        let candidate = &remaining[0..3];
        if INITIAL_TRIPLE.contains(&candidate) {
            *pos += 3;
            return Some(candidate.to_string());
        }
    }

    // 2 chars: "ch", "gh", "gi", "kh", "ng", "nh", "ph", "qu", "th", "tr"
    if remaining.len() >= 2 {
        let candidate = &remaining[0..2];
        if INITIAL_DOUBLE.contains(&candidate) {
            *pos += 2;
            return Some(candidate.to_string());
        }
    }

    // 1 char: "b", "c", "d", ...
    if remaining.len() >= 1 {
        let candidate = &remaining[0..1];
        if INITIAL_SINGLE.contains(&candidate) {
            *pos += 1;
            return Some(candidate.to_string());
        }
        // Check for đ (Unicode)
        if remaining.starts_with('đ') {
            *pos += 'đ'.len_utf8();
            return Some("đ".to_string());
        }
    }

    None  // Không có phụ âm đầu (âm tiết bắt đầu bằng nguyên âm)
}
```

### 4.4 Extract Vowel (Nguyên âm)

```rust
fn extract_vowel(s: &str, pos: &mut usize) -> Option<String> {
    let remaining = &s[*pos..];

    if remaining.is_empty() {
        return None;
    }

    // Try longest first: 3 chars → 2 chars → 1 char

    // 3 chars: nguyên âm ba
    if remaining.len() >= 3 {
        let candidate = &remaining[0..3];
        if VOWEL_TRIPLE.contains(&candidate) {
            *pos += 3;
            return Some(candidate.to_string());
        }
    }

    // 2 chars: nguyên âm đôi
    if remaining.len() >= 2 {
        let candidate = &remaining[0..2];
        if VOWEL_DOUBLE.contains(&candidate) {
            *pos += 2;
            return Some(candidate.to_string());
        }
    }

    // 1 char: nguyên âm đơn
    if remaining.len() >= 1 {
        // Handle multi-byte Vietnamese vowels
        let first_char = remaining.chars().next()?;
        let char_str = first_char.to_string();

        if VOWEL_SINGLE.contains(&char_str.as_str()) {
            *pos += first_char.len_utf8();
            return Some(char_str);
        }
    }

    None  // Không tìm thấy nguyên âm
}
```

### 4.5 Extract Final (Âm cuối)

```rust
fn extract_final(s: &str, pos: &mut usize) -> Option<String> {
    let remaining = &s[*pos..];

    if remaining.is_empty() {
        return None;
    }

    // Try longest first: 2 chars → 1 char

    // 2 chars: "ch", "ng", "nh"
    if remaining.len() >= 2 {
        let candidate = &remaining[0..2];
        if ["ch", "ng", "nh"].contains(&candidate) {
            *pos += 2;
            return Some(candidate.to_string());
        }
    }

    // 1 char: "c", "m", "n", "p", "t", "i", "y", "o", "u"
    if remaining.len() >= 1 {
        let candidate = &remaining[0..1];
        if ALL_FINALS.contains(&candidate) {
            *pos += 1;
            return Some(candidate.to_string());
        }
    }

    None
}
```

---

## 5. THUẬT TOÁN VALIDATION

### 5.1 Main Validation Flow

```
is_valid_vietnamese(buffer: &str) -> bool
│
├── STEP 1: Quick reject - có ký tự không hợp lệ?
│   └── Chứa f, j, w, z → INVALID
│
├── STEP 2: Parse syllable
│   └── parse_syllable() → None → INVALID
│
├── STEP 3: Validate initial consonant rules
│   ├── Spelling rules (c/k, g/gh, ng/ngh)
│   └── Invalid clusters check
│
├── STEP 4: Validate vowel + final combination
│   ├── -ch chỉ sau a, ă, ê, i
│   ├── -nh chỉ sau a, ă, ê, i, y
│   └── -ng không sau e, ê (dùng -nh)
│
├── STEP 5: Validate tone + stop final rule
│   └── p, t, c, ch → chỉ sắc hoặc nặng
│
└── RETURN true (VALID)
```

### 5.2 Implementation

```rust
fn is_valid_vietnamese(input: &str) -> bool {
    // STEP 1: Quick reject - ký tự không hợp lệ
    let lower = input.to_lowercase();
    for c in lower.chars() {
        if INVALID_CHARS.contains(&c) {
            return false;
        }
    }

    // STEP 2: Parse syllable
    let syllable = match parse_syllable(&lower) {
        Some(s) => s,
        None => return false,
    };

    // STEP 3: Validate initial consonant rules
    if let Some(ref initial) = syllable.initial {
        if !validate_initial_rules(initial, &syllable.vowel) {
            return false;
        }
    }

    // STEP 4: Validate vowel + final combination
    if let Some(ref final_c) = syllable.final_c {
        if !validate_vowel_final(&syllable.vowel, final_c) {
            return false;
        }
    }

    true
}

/// Validate tone với âm cuối tắc
fn is_valid_tone_for_final(tone: u8, final_c: Option<&str>) -> bool {
    match final_c {
        Some(f) if STOP_FINALS.contains(&f) => {
            // Âm cuối tắc: chỉ sắc (1) hoặc nặng (5)
            matches!(tone, 1 | 5)
        }
        _ => true  // Các âm cuối khác: tất cả thanh OK
    }
}
```

### 5.3 Validate Initial Rules (Quy tắc chính tả)

```rust
fn validate_initial_rules(initial: &str, vowel: &str) -> bool {
    let first_vowel_char = vowel.chars().next().unwrap_or(' ');

    // Nguyên âm hàng trước: e, ê, i, y
    let is_front_vowel = matches!(first_vowel_char, 'e' | 'ê' | 'i' | 'y');

    // Nguyên âm hàng sau: a, ă, â, o, ô, ơ, u, ư
    let is_back_vowel = matches!(first_vowel_char,
        'a' | 'ă' | 'â' | 'o' | 'ô' | 'ơ' | 'u' | 'ư');

    match initial {
        // C trước nguyên âm hàng sau, K trước nguyên âm hàng trước
        "c" => is_back_vowel,   // ca, co, cu ✓ | ce, ci ✗
        "k" => is_front_vowel,  // ke, ki ✓ | ka, ko ✗

        // G trước nguyên âm hàng sau, GH trước nguyên âm hàng trước
        "g" => is_back_vowel,   // ga, go ✓ | ge, gi ✗ (gi là phụ âm riêng)
        "gh" => is_front_vowel, // ghe, ghi ✓ | gha, gho ✗

        // NG trước nguyên âm hàng sau, NGH trước nguyên âm hàng trước
        "ng" => is_back_vowel,  // nga, ngo ✓ | nge, ngi ✗
        "ngh" => is_front_vowel, // nghe, nghi ✓ | ngha ✗

        // Các phụ âm khác: OK
        _ => true,
    }
}
```

### 5.4 Validate Vowel + Final Combination

```rust
fn validate_vowel_final(vowel: &str, final_c: &str) -> bool {
    // Lấy nguyên âm cuối cùng (để check với âm cuối)
    let last_vowel = vowel.chars().last().unwrap_or(' ');

    match final_c {
        // -ch chỉ sau a, ă, ê, i
        "ch" => matches!(last_vowel, 'a' | 'ă' | 'ê' | 'i'),

        // -nh chỉ sau a, ă, ê, i, y
        "nh" => matches!(last_vowel, 'a' | 'ă' | 'ê' | 'i' | 'y'),

        // -ng không sau e, ê (dùng -nh thay thế)
        "ng" => !matches!(last_vowel, 'e' | 'ê'),

        // Các âm cuối khác: OK với hầu hết nguyên âm
        _ => true,
    }
}
```

---

## 6. NORMALIZE FUNCTION

### 6.1 Remove Tone Marks

```rust
/// Bảng mapping nguyên âm có dấu → không dấu
const TONE_MAP: &[(char, char)] = &[
    // a variants
    ('á', 'a'), ('à', 'a'), ('ả', 'a'), ('ã', 'a'), ('ạ', 'a'),
    ('ắ', 'ă'), ('ằ', 'ă'), ('ẳ', 'ă'), ('ẵ', 'ă'), ('ặ', 'ă'),
    ('ấ', 'â'), ('ầ', 'â'), ('ẩ', 'â'), ('ẫ', 'â'), ('ậ', 'â'),
    // e variants
    ('é', 'e'), ('è', 'e'), ('ẻ', 'e'), ('ẽ', 'e'), ('ẹ', 'e'),
    ('ế', 'ê'), ('ề', 'ê'), ('ể', 'ê'), ('ễ', 'ê'), ('ệ', 'ê'),
    // i variants
    ('í', 'i'), ('ì', 'i'), ('ỉ', 'i'), ('ĩ', 'i'), ('ị', 'i'),
    // o variants
    ('ó', 'o'), ('ò', 'o'), ('ỏ', 'o'), ('õ', 'o'), ('ọ', 'o'),
    ('ố', 'ô'), ('ồ', 'ô'), ('ổ', 'ô'), ('ỗ', 'ô'), ('ộ', 'ô'),
    ('ớ', 'ơ'), ('ờ', 'ơ'), ('ở', 'ơ'), ('ỡ', 'ơ'), ('ợ', 'ơ'),
    // u variants
    ('ú', 'u'), ('ù', 'u'), ('ủ', 'u'), ('ũ', 'u'), ('ụ', 'u'),
    ('ứ', 'ư'), ('ừ', 'ư'), ('ử', 'ư'), ('ữ', 'ư'), ('ự', 'ư'),
    // y variants
    ('ý', 'y'), ('ỳ', 'y'), ('ỷ', 'y'), ('ỹ', 'y'), ('ỵ', 'y'),
];

fn remove_tone_marks(s: &str) -> String {
    s.chars().map(|c| {
        TONE_MAP.iter()
            .find(|(from, _)| *from == c)
            .map(|(_, to)| *to)
            .unwrap_or(c)
    }).collect()
}

fn normalize(input: &str) -> String {
    let lower = input.to_lowercase();
    remove_tone_marks(&lower)
}
```

---

## 7. VÍ DỤ VÀ TEST CASES

### 7.1 Valid Cases

```
┌─────────────┬────────────────────────────────────────┐
│   Input     │              Parsing                   │
├─────────────┼────────────────────────────────────────┤
│ "a"         │ C₁=∅, V="a", C₂=∅           → VALID   │
│ "an"        │ C₁=∅, V="a", C₂="n"         → VALID   │
│ "ba"        │ C₁="b", V="a", C₂=∅         → VALID   │
│ "ban"       │ C₁="b", V="a", C₂="n"       → VALID   │
│ "hoa"       │ C₁="h", V="oa", C₂=∅        → VALID   │
│ "hoàn"      │ C₁="h", V="oa", C₂="n"      → VALID   │
│ "toán"      │ C₁="t", V="oa", C₂="n"      → VALID   │
│ "người"     │ C₁="ng", V="ươ", C₂="i"     → VALID   │
│ "trường"    │ C₁="tr", V="ươ", C₂="ng"    → VALID   │
│ "nghiêng"   │ C₁="ngh", V="iê", C₂="ng"   → VALID   │
│ "qua"       │ C₁="qu", V="a", C₂=∅        → VALID   │
│ "khuyên"    │ C₁="kh", V="uyê", C₂="n"    → VALID   │
│ "được"      │ C₁="đ", V="ươ", C₂="c"      → VALID   │
│ "giúp"      │ C₁="gi", V="u", C₂="p"      → VALID   │
│ "kẻ"        │ C₁="k", V="e", C₂=∅         → VALID   │
│ "ghế"       │ C₁="gh", V="ê", C₂=∅        → VALID   │
└─────────────┴────────────────────────────────────────┘
```

### 7.2 Invalid Cases

```
┌─────────────┬────────────────────────────────────────┐
│   Input     │              Reason                    │
├─────────────┼────────────────────────────────────────┤
│ "clau"      │ "cl" không phải phụ âm đầu hợp lệ     │
│ "john"      │ "j" không có trong tiếng Việt          │
│ "http"      │ Không có nguyên âm                     │
│ "pizza"     │ "zz" không hợp lệ                      │
│ "black"     │ "bl" là consonant cluster              │
│ "stop"      │ "st" là consonant cluster              │
│ "ce"        │ "c" + "e" vi phạm (phải là "ke")      │
│ "ka"        │ "k" + "a" vi phạm (phải là "ca")      │
│ "ghe"       │ ✓ VALID (gh + e)                       │
│ "ge"        │ ✗ "g" + "e" vi phạm (phải là "ghe")   │
│ "nghe"      │ ✓ VALID (ngh + e)                      │
│ "nge"       │ ✗ "ng" + "e" vi phạm (phải là "nghe") │
│ "ôch"       │ ✗ "ô" + "ch" không hợp lệ             │
│ "ung"       │ ✓ VALID                                │
│ "êng"       │ ✗ "ê" + "ng" không hợp lệ (dùng ênh)  │
└─────────────┴────────────────────────────────────────┘
```

### 7.3 Tone + Stop Final Cases

```
┌───────────────┬────────────────────────────────────────┐
│   Input       │              Validation                │
├───────────────┼────────────────────────────────────────┤
│ "cấp" (sắc)   │ final="p" + tone=sắc     → VALID ✓   │
│ "cập" (nặng)  │ final="p" + tone=nặng    → VALID ✓   │
│ "cảp" (hỏi)   │ final="p" + tone=hỏi     → INVALID ✗ │
│ "cãp" (ngã)   │ final="p" + tone=ngã     → INVALID ✗ │
│ "cap" (ngang) │ final="p" + tone=ngang   → INVALID ✗ │
│ "càp" (huyền) │ final="p" + tone=huyền   → INVALID ✗ │
├───────────────┼────────────────────────────────────────┤
│ "mát" (sắc)   │ final="t" + tone=sắc     → VALID ✓   │
│ "mạt" (nặng)  │ final="t" + tone=nặng    → VALID ✓   │
│ "mat" (ngang) │ final="t" + tone=ngang   → INVALID ✗ │
├───────────────┼────────────────────────────────────────┤
│ "các" (sắc)   │ final="c" + tone=sắc     → VALID ✓   │
│ "cạc" (nặng)  │ final="c" + tone=nặng    → VALID ✓   │
│ "cac" (ngang) │ final="c" + tone=ngang   → INVALID ✗ │
├───────────────┼────────────────────────────────────────┤
│ "ách" (sắc)   │ final="ch" + tone=sắc    → VALID ✓   │
│ "ạch" (nặng)  │ final="ch" + tone=nặng   → VALID ✓   │
│ "ach" (ngang) │ final="ch" + tone=ngang  → INVALID ✗ │
├───────────────┼────────────────────────────────────────┤
│ "cam" (ngang) │ final="m" (non-stop)     → VALID ✓   │
│ "cảm" (hỏi)   │ final="m" (non-stop)     → VALID ✓   │
│ "cãm" (ngã)   │ final="m" (non-stop)     → VALID ✓   │
└───────────────┴────────────────────────────────────────┘
```

---

## 8. EDGE CASES VÀ SPECIAL HANDLING

### 8.1 "qu" as Single Unit

```
"qu" được xử lý như MỘT phụ âm đầu:

├── "qua" → C₁="qu", V="a"      (không phải C₁="q", G="u", V="a")
├── "quê" → C₁="qu", V="ê"
├── "quy" → C₁="qu", V="y"
└── "quân" → C₁="qu", V="â", C₂="n"

LƯU Ý: "u" trong "qu" KHÔNG phải âm đệm
```

### 8.2 "gi" as Single Unit

```
"gi" là MỘT phụ âm đầu (phát âm /z/ hoặc /j/):

├── "gia" → C₁="gi", V="a"
├── "giờ" → C₁="gi", V="ơ"
├── "giúp" → C₁="gi", V="u", C₂="p"
└── "giêng" → C₁="gi", V="ê", C₂="ng"

LƯU Ý: Không nhầm với "g" + "i" riêng
```

### 8.3 Handling "ươ" compound

```
"ươ" là nguyên âm đôi đặc biệt:

├── "mươi" → C₁="m", V="ươi"
├── "người" → C₁="ng", V="ươ", C₂="i"
├── "trường" → C₁="tr", V="ươ", C₂="ng"
└── "được" → C₁="đ", V="ươ", C₂="c"

Khi gõ: "truong" + w → "trương" (u→ư, o→ơ cùng lúc)
```

### 8.4 "y" vs "i" at word start

```
Khi đứng đầu âm tiết một mình hoặc với final:

├── "y" đứng đầu: y, yêu, yến, yểng
├── "i" ít khi đứng đầu một mình (thường viết "y")

Quy tắc:
├── Đứng một mình → dùng "y": y tá, ý kiến
└── Trong từ → cả hai OK: lí/lý, kĩ/kỹ
```

---

## 9. FULL IMPLEMENTATION EXAMPLE

```rust
/// Complete validation module
mod validation {
    use std::collections::HashSet;

    // Constants (as defined above)
    // ...

    pub struct Syllable {
        pub initial: Option<String>,
        pub vowel: String,
        pub final_c: Option<String>,
    }

    pub fn is_valid_vietnamese(input: &str) -> bool {
        // Step 1: Quick reject
        let lower = input.to_lowercase();
        if lower.chars().any(|c| INVALID_CHARS.contains(&c)) {
            return false;
        }

        // Step 2: Parse
        let syllable = match parse_syllable(&lower) {
            Some(s) => s,
            None => return false,
        };

        // Step 3: Validate initial rules
        if let Some(ref init) = syllable.initial {
            if !validate_initial_rules(init, &syllable.vowel) {
                return false;
            }
        }

        // Step 4: Validate vowel + final
        if let Some(ref final_c) = syllable.final_c {
            if !validate_vowel_final(&syllable.vowel, final_c) {
                return false;
            }
        }

        true
    }

    /// Check if transformation should apply
    pub fn should_transform(buffer: &str, tone: Option<u8>) -> bool {
        if !is_valid_vietnamese(buffer) {
            return false;
        }

        // Check tone + stop final rule
        if let Some(t) = tone {
            let syllable = parse_syllable(&buffer.to_lowercase()).unwrap();
            if let Some(ref f) = syllable.final_c {
                if STOP_FINALS.contains(&f.as_str()) {
                    return matches!(t, 1 | 5);  // Only sắc or nặng
                }
            }
        }

        true
    }
}
```

---

## 10. INTEGRATION VỚI ENGINE

### 10.1 Khi nào gọi Validation?

```
ENGINE V2 INTEGRATION:
│
on_key(key)
│
├── [is_modifier(key)?]
│   │
│   ├── ★ VALIDATION POINT 1: Trước khi transform
│   │   └── is_valid_vietnamese(buffer)?
│   │       ├── NO → return NONE (không transform)
│   │       └── YES → tiếp tục
│   │
│   ├── Apply transformation
│   │
│   └── ★ VALIDATION POINT 2: Nếu có tone + stop final
│       └── is_valid_tone_for_final(tone, final)?
│           ├── NO → reject transformation
│           └── YES → accept
│
└── [is_letter(key)?] → push to buffer
```

### 10.2 Performance Considerations

```
OPTIMIZATION:
│
├── Cache syllable parsing: Chỉ parse lại khi buffer thay đổi
│
├── Early exit: Kiểm tra INVALID_CHARS trước
│
├── Lazy validation: Chỉ validate khi gặp modifier key
│
└── Incremental: Khi thêm 1 char, chỉ cần validate phần mới
```

---

## Changelog

- **2025-12-08**: Tạo tài liệu Validation Algorithm
  - Chi tiết cấu trúc âm tiết tiếng Việt
  - Data constants cho phụ âm đầu, nguyên âm, âm cuối
  - Thuật toán parse_syllable (longest-first)
  - Thuật toán validation với các quy tắc chính tả
  - Quy tắc tone + stop final
  - Normalize function (remove tone marks)
  - Ví dụ và test cases
  - Edge cases handling
  - Integration guide với engine

---

*Tài liệu thuật toán validation cho GoNhanh Core Engine*
