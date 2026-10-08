# Typing Behavior Flow (V1, hiện trạng)

> Tài liệu bắt buộc đọc trước khi sửa engine (CLAUDE.md). Mô tả **hành vi thực tế** của `core/src/engine/mod.rs` trước refactor V2.
> Sẽ được thay bằng bản V2 ở phase P6 của [kế hoạch V2](../plans/261007-2250-core-v2-refactor/plan.md). Số dòng là của V1 tại thời điểm viết.

**Liên quan**: [core-engine-algorithm.md](./core-engine-algorithm.md) (thiết kế gốc) · [validation-algorithm.md](./validation-algorithm.md) · [vietnamese-language-system.md](./vietnamese-language-system.md) · [báo cáo audit](../plans/reports/core-refactor-current-logic-grammar-audit-refs-261007-2056-report.md) (drift giữa docs và code).

## 1. Luồng quyết định tổng quát

```
Validation flow:  VN(R) → VN(B) → RESTORE/FOREIGN → EN mode
   VN(R): raw_input có phải gõ tiếng Việt hợp lệ không?
   VN(B): buffer (đã biến đổi) có phải âm tiết VN hợp lệ không?
   RESTORE/FOREIGN: buffer invalid ∧ raw là EN  →  trả về chuỗi thô
   EN mode: giữ chuỗi thô
```

Nguyên tắc cốt lõi (CLAUDE.md): **không bao giờ restore khi buffer là VN hợp lệ**; sửa ở điểm quyết định (`should_auto_restore`), không thêm ngoại lệ theo từng pattern.

## 2. Chu trình một phím (`Engine::on_key_ext`, `mod.rs:638`)

```
on_key_ext(key, caps, ctrl, shift)
├─ ctrl                          → clear, xoá history, none
├─ IME tắt                       → chỉ gõ tắt (shortcut_prefix), không biến đổi
├─ SPACE                         → gõ tắt từ → try_auto_restore_on_space → history.push → clear
├─ ESC                           → esc_restore ? restore_to_raw : none ; clear
├─ [ ] (Telex, bật bracket)      → try_bracket_as_vowel
├─ phím ngắt (. , ! ? ...)       → gõ tắt → try_auto_restore_on_break → history → clear
├─ DELETE                        → pop buf+raw (+ heuristic raw "stale"); hết khoảng trắng → pop word_history
├─ auto-capitalize, Shift-segment (Telex)
├─ raw_input.push                (chỉ chữ cái và chữ số)
└─ process (1243)
   ├─ pre-guard: K-word kạn/kông · ShortPatternStroke undo · DelayedCircumflex undo
   ├─ 1 Stroke   try_stroke   d → đ (kề nhau, hoặc d..d trễ, VNI 9)
   ├─ 2 Tone     try_tone     mũ/móc/trăng (uo→ươ, circumflex Telex, breve)
   ├─ 3 Mark     try_mark     sắc/huyền/hỏi/ngã/nặng, đặt dấu theo Phonology
   ├─ 4 Remove   try_remove   z / 0
   ├─ 5 W→ư      try_w_as_vowel (Telex)
   └─ 6 Letter   handle_normal_letter  (revert trễ, reorder ia/ua, reposition dấu, restore giữa từ)
```

Mỗi stage trả `Option<Result>`; stage đầu tiên trả `Some` thắng. `Result{backspace, chars}` là **diff so với màn hình**: V1 tự tính `backspace` bằng `buf.len()`, nên phải giữ `buf` đồng bộ với màn hình.

## 3. Validation-first (vì sao phím không biến đổi)

Mỗi modifier (tone/mark/stroke) kiểm tra buffer trước khi áp dụng:
`is_valid_for_transform_with_options` (cấu trúc), `is_foreign_word_pattern` (ou, yo, cụm phụ âm EN), rhyme `-ing` + dấu, **thanh × coda tắc** (huyền/hỏi/ngã trên p/t/c/ch/k bị từ chối, `try_mark` ~3147).
Bỏ qua kiểm tra khi: `free_tone`, buffer đã có móc/đ (ý định VN rõ ràng), mẫu mũ trễ, từ ba-o (đoòng).

## 4. Hoàn tác phím đôi

| Gõ | Kết quả | Cách V1 làm |
|---|---|---|
| `aa` → `aaa` | â → aa | `revert_tone`, đặt `had_circumflex_revert`, `reverted_circumflex_key` |
| `as` → `ass` | á → as | `revert_mark`, đặt `had_mark_revert`, `telex_double_raw` |
| `dd` → `ddd` | đ → dd | `try_stroke`, `stroke_reverted` |
| `w` → `ww` | ư → w | `try_w_as_vowel`, `WShortcutSkipped` |

Sau hoàn tác mũ, mọi modifier còn lại của từ bị tắt (`skip_after_revert`), ngoại lệ từ ba-o.
Nguyên âm đã có dấu phụ + cùng nguyên âm gốc ⇒ thêm **chữ thô** (`chưa`+`a` → `chưaa`, issue #312).

## 5. Biến đổi trễ (nguồn phức tạp lớn nhất)

| Mẫu | Ví dụ | Cơ chế V1 |
|---|---|---|
| Mũ trễ cùng nguyên âm | `toto`→`tôt`, `xuata`→`xuât` | `try_tone` (mũ ngay nếu coda mở rộng được n/c), `Transform::DelayedCircumflex` (T/M/P), hoàn tác khi phụ âm kế tiếp vô lý |
| Gạch d trễ | `dod`→`đo`, `dods`→`đó` | `ShortPatternStroke`, hoàn tác khi chữ kế tạo âm tiết sai |
| Móc `uơ` trễ | `huow`→`huơ`, `duowc`→`dược` | `pending_u_horn_pos` |
| Tên K đặc biệt | `kanj`/`kajn`→`kạn`, `koong`/`kongo`→`kông` | **hardcode** buffer `[K,A,N]`/`[K,O,N,G]` (`mod.rs:5681–5737`); không phủ `konog`, `kon6g` |
| Đảo `ia/ua` + coda | `kisna`→`kían` | `reorder_diphthong_with_final` |

## 6. Restore tiếng Anh (`should_auto_restore`, `mod.rs:4756`)

Chỉ chạy khi `english_auto_restore` bật. Công thức:

```
restore = had_transform ∧ ( (buffer_invalid_VN ∧ raw_valid_EN)
                          ∨ (has_english_pattern ∧ raw_valid_EN ∧ !stroke) )
```

Thứ tự kiểm tra (rút gọn):
1. Cổng sớm: tắt/rỗng/chưa biến đổi · ≥3 phím giống nhau (kéo dài cảm xúc) · lặp coda sau âm tiết VN (`chứcc`).
2. **Ưu tiên VN**: có mũ/móc/đ, không có `w`, không có telex-double, buffer hợp lệ ⇒ giữ.
3. Whitelist telex-double (`telex_doubles.rs`, từ EN có aa/ee/oo/dd/ss/ff…) + `ss/ff` cuối từ.
4. **Quy tắc chính**: `buffer_invalid_VN ∧ raw_valid_EN` ⇒ restore (`raw_valid_EN` thực chất chỉ kiểm "toàn ASCII có nguyên âm").
5. W đầu từ, `ow` + ơ, `has_english_modifier_pattern` (~20 pattern), mũ ăn chữ, V1-V2-V1.
6. Mặc định giữ.

`is_buffer_invalid_vietnamese` (5772): từ điển `vi.dic` thắng đầu tiên → cấu trúc → `-ing`+dấu → nguyên âm đơn có dấu hợp lệ → mũ không coda (`sê`,`tê`) → diphthong mở + coda → `ơe` → `ươu` không initial → mũ + `k` cuối.

Restore chạy tại: SPACE, phím ngắt (`is_word_complete=true`) và giữa từ (consonant sau dấu, `4373`).

## 7. Vòng đời từ và lịch sử

- SPACE/ngắt: lưu buffer vào `word_history` (10 từ), `spaces_after_commit++`.
- DELETE khi buffer rỗng: giảm bộ đếm; về 0 thì khôi phục từ trước (`restore_raw_input_from_buffer`, `re_detect_pending_u_horn`, `re_detect_last_transform`) để gõ tiếp dấu.
- `clear()` đặt lại ~12 cờ bằng tay (**danh sách thứ hai và thứ ba** nằm ở nhánh Shift-segment `mod.rs:1208` và `restore_nonstandard_k_word` `5747`; thêm field phải sửa cả ba).
- Auto-capitalize: 4 field (`auto_capitalize`, `pending_capitalize`, `auto_capitalize_used`, `saw_sentence_ending`), đặt tại 7 chỗ.

## 8. Hợp đồng FFI (bất biến, không đổi ở V2)

`Result { chars[256]: u32, action: u8, backspace: u8, count: u8, flags: u8 }` (1028 byte; `flags` bit0 = `key_consumed`).
22 hàm `ime_*` trong `core/src/lib.rs`. `action`: 0 None · 1 Send · 2 Restore (không dùng).

## 9. Hành vi đã biết là lệch spec / chắp vá (để V2 xử lý)

| Hiện tượng | Vị trí | Ghi chú |
|---|---|---|
| Từ điển EN gọi ở **mọi chữ cái**, kể cả khi tắt auto-restore | `reposition_tone_if_needed` 3475, gọi từ 4326 | 7–10 cấp phát/phím, +~11ms phím đầu tiên (dựng HashSet) |
| `kạn/kông` chỉ khớp buffer chính xác | 5681–5737 | thiếu `konog`, `kon6g` |
| Thanh × coda tắc chỉ là cổng nhập phím, không ở validator | `try_mark` 3147 | `cap`/`mat` không dấu coi là hợp lệ |
| Vần × coda (§6.5.4) chưa được thực thi | `validation.rs` | `ôch`, `ônh`, `êng` lọt |
| Bug glide: mọi phụ âm đầu 2 chữ bị coi là `qu` | `syllable.rs:189` | `uya/uyu` hợp lệ sau `kh`, sai sau phụ âm đơn |
| `pending_breve_pos` luôn không chạy | `mod.rs:2808` | code chết |
| `has_english_modifier_pattern` có pattern theo từng từ (`sax`, `sims`) | 7878, 7983 | trái CLAUDE.md |
| Nhiều test cùng ghi một file `vietnamese_22k_failures.txt` | `tests/suite/*` | nội dung phụ thuộc thứ tự chạy |
