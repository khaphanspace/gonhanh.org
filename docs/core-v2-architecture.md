# Core V2: kiến trúc

Engine gõ V2 (`--features engine_v2`) thay `engine/mod.rs` (V1). Cùng API công khai và FFI (22 hàm, `Result` 1028 B). Khác biệt hành vi: [behavior-changes.md](behavior-changes.md). Lý do từng quyết định theo issue: [issue-coverage.md](issue-coverage.md).

## Ba tầng, mỗi tầng một câu hỏi

```
core/src/
├─ phonology/   "Đây có phải tiếng Việt không? Dấu đặt ở đâu?"      thuần, không cấp phát, không biết bàn phím
├─ compose/     "Chuỗi phím này đang gõ ra chữ gì?"                  beam các cách đọc, không biết từ điển Anh
└─ session/     "Từ này kết thúc thì làm gì?"                         dấu cách, ngắt, ESC, gõ tắt, lịch sử, restore
```

Chiều phụ thuộc chỉ đi xuống: `session` → `compose` → `phonology`. Từ điển tiếng Anh chỉ `session/restore.rs` dùng, và chỉ ở ranh giới từ.

## Luồng một phím

```
phím ─ session.on_key_ext
        ├─ Space / dấu ngắt / ESC → gõ tắt → bảng restore → lịch sử
        └─ chữ/số → compose.push
              ├─ parse.extend     mỗi phím là chữ hoặc dấu (≤ 12 cách đọc con)
              ├─ validate         phonology cắt cách đọc sai: Complete > Prefix > Loose > NamePrefix
              ├─ beam ≤ 8         sắp theo (hợp lệ, điểm bộ biến đổi, thứ tự tạo)
              └─ render + diff    đặt dấu trên âm tiết cuối → (backspace, chữ) → Out → Result
```

Hiển thị khi không còn cách đọc tiếng Việt: chuỗi `keep` (giữ mọi dấu đã hợp lệ lúc gõ, `tẽt`, `ăi`), hoặc chữ thô nếu bật auto-restore và không có bằng chứng ý định tiếng Việt (đ, ư, ơ, ă, `[ ]`).

## Bất biến (đổi là phải sửa test hồi quy và tài liệu này)

1. **Dấu là hàm của chữ cuối cùng**, không phụ thuộc thứ tự gõ (`tone_index` chạy trên âm tiết hoàn chỉnh).
2. **Ngữ pháp sinh từ dữ liệu**: ma trận vần × coda từ `vi.dic`, tên riêng từ `names.dic`, qua `scripts/gen/phonology_tables.py`. `tables.rs` không sửa tay.
3. **Cách đọc đúng ngữ pháp luôn thắng** cách đọc chỉ "tự do" (`Loose`): `wa` → `ưa`, `wes` → `wé` khi không có `ưe`.
4. **Không cấp phát trong lõi gõ**: Beam, `Children`, `keeps` cấp một lần lúc tạo `Compose`. Chạy lại `examples/prof` khi sửa `compose/`.
5. **Backspace ≡ quay về trạng thái phím trước** (hoặc phát lại phím trừ chữ vừa xoá). Khi từ đang hiện chữ thô, xoá một chữ là xoá đúng một phím (`serv⌫` → `ser`).
6. **Khôi phục tiếng Anh là một bảng thứ tự cố định** (`restore.rs`, ~12 hàng, dừng ở hàng đầu khớp). Không thêm luật theo từng từ; thêm bằng chứng chung hoặc dữ liệu.
7. **Từ tiếng Việt hợp lệ có trong `vi.dic` không bao giờ bị trả về chữ thô** trừ khi có bằng chứng tiếng Anh mạnh (xem hàng 6).

## Dữ liệu

| File | Vai trò | Ai đọc |
|---|---|---|
| `data/dictionaries/vi.dic` | âm tiết có thật | `build.rs` → `VI`; sinh ngữ pháp |
| `data/dictionaries/names.dic` | tên riêng ngoài luật (kạn, kông, krông…) | chỉ lúc sinh `tables.rs` |
| `data/dictionaries/vi-non-syllables.txt` | mục của vi.dic không phải âm tiết | sinh bảng, audit |
| `data/dictionaries/keep.dic` | từ luôn giữ nguyên | `build.rs` → `KEEP` |
| `data/english_dict_merged.txt` | từ tiếng Anh | `build.rs` → `EN` |
| `data/telex_doubles.txt` | từ Anh chứa chữ đôi kiểu Telex | `build.rs` → `DOUBLES` |

Sửa `vi.dic` hoặc `names.dic`: chạy `python3 scripts/gen/phonology_tables.py`; `make gate` kiểm bằng `--check`.

## Kiểm thử và đo

| Lệnh | Việc |
|---|---|
| `make t` | test đơn vị (giây) |
| `make gate` | V1 mặc định: lib + suite + từ điển + bảng sinh còn mới |
| `make gate-v2` | V2: toàn bộ test V1, chỉ `core/tests/v2-known-differences.txt` được fail |
| `cargo test --profile gate --features engine_v2 --test suite issue_regressions` | 150+ ca rút từ issue |
| `GN_FUZZ_KEYS=2000000 cargo test --profile gate --features engine_v2 --test suite engine_fuzz` | bão phím ngẫu nhiên |
| `cargo run --release --features engine_v2 --example prof` | CPU time/phím theo từng lớp |
| `cargo run --release --features engine_v2 --example try -- telex_ar "vieetj_nam_"` | thử nhanh (`_` dấu cách, `<` backspace) |
| `cargo run --release --features engine_v2 --example dbg -- repea` | cách đọc thắng ở từng phím |

Đo hiệu năng: máy dùng chung thì wall-clock vô nghĩa; `prof` dùng CPU time, lấy tốt nhất trong 25 lượt. Build hai binary (có/không `engine_v2`) vào hai thư mục riêng rồi chạy xen kẽ.

## Thêm một hành vi mới

1. Tìm ca người dùng đã báo (chuỗi phím, mong đợi) và thêm một dòng vào `core/tests/suite/issue_regressions.rs`.
2. Sửa ở tầng đúng: ngữ pháp (`phonology`), cách đọc (`compose/parse.rs`), hay quyết định cuối từ (`session/restore.rs`).
3. Chạy `make gate`, `make gate-v2`; cập nhật `docs/behavior-changes.md` nếu khác V1.
