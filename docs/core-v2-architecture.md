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
   **Gõ tự do chỉ nới phụ âm đầu *không phải tiếng Việt*, không nới vần và không nới phụ âm đầu hợp lệ** (`nginx` không thành `ngĩn`: `ng` đã là phụ âm đầu tiếng Việt nên lỗi nằm ở chính tả, không phải ở phụ âm đầu): `khphá`, `qcáo`, `zị`, `wé` (vần `á`, `áo`, `ị`, `é` đúng) nhưng `ads`, `expect`, `haaấ` không có cách đọc tự do vì coda `d`/`xpct` hay cụm nguyên âm `aaa` không phải tiếng Việt.
4. **Không cấp phát trong lõi gõ**: Beam, `Children`, `keeps` cấp một lần lúc tạo `Compose`. Chạy lại `examples/prof` khi sửa `compose/`.
5. **Backspace ≡ quay về trạng thái phím trước** (hoặc phát lại phím trừ chữ vừa xoá). Khi từ đang hiện chữ thô, xoá một chữ là xoá đúng một phím (`serv⌫` → `ser`).
6. **Khôi phục tiếng Anh là một bảng thứ tự cố định** (`restore.rs`, ~12 hàng, dừng ở hàng đầu khớp). Không thêm luật theo từng từ; thêm bằng chứng chung hoặc dữ liệu.
8. **Dấu không nhảy trong từ không còn là tiếng Việt**: khi không còn cách đọc đúng, dấu giữ nguyên trên nguyên âm nó đang đứng (`háaa` + `e` → `háaae`, không thành `haáae`). Còn là tiếng Việt thì ngữ pháp đặt dấu như thường (`hoa`+`i` → `hoái`). Vị trí lưu ở `Parse::tone_at`. Hoàn tác một biến đổi (dấu mũ, móc) đưa từ về trạng thái trước biến đổi đó, nên dấu cũng về lại chỗ cũ: `mùa` + `a` → `muầ`, + `a` → `mùaa` (dấu ở `u`, không ở chữ `a` đầu).
   **Kéo dài chữ**: gõ lại nguyên âm cuối của nguyên âm đôi trong từ đã có dấu (`mùa` + `a`) chỉ làm chữ dài thêm (`mùaa`, `mùaaa`), không hiện `muầ` chen giữa. Cách đọc dấu mũ vẫn sống trong beam nên `bafan` → `bần`; nếu cách đọc dấu mũ đã là một từ (`bồ`, `cố`) hoặc nguyên âm đơn (`hara` → `hẩ`, `afa` → `ầ`, quyết định ở #211) thì dấu mũ thắng như trước.
9. **Gõ liền nhiều âm tiết (chỉ khi bật Gõ tự do)**: khi từ không còn cách đọc tiếng Việt, session tách nó thành các âm tiết hoàn chỉnh (`xinchaof` → `xin` + `chào`, `thuwrgoxTieengsVieetj` → `thửgõTiếngViệt`) rồi viết lại phần đang hiện. Từ đó dấu và phụ âm chỉ tác động lên âm tiết cuối. Không cắt khi (a) chuỗi phím là tiếng Anh: một từ, tiền tố của một từ, hoặc nhiều từ (mỗi từ ≥ 3 chữ) nối nhau (`expect`, `thanks`, `helloworld`, `popover` = `pop` + `over`), bất kể auto-restore; (b) không tách trọn được thành âm tiết. Chỉ đoạn đầu được nới phụ âm đầu (`khphá`); các đoạn sau phải là âm tiết tiếng Việt thật, và đoạn bị đóng phải có ít nhất 2 chữ. Mỗi đoạn bị đóng nhận cùng quyết định của bảng restore như lúc gặp dấu cách. Giới hạn của Telex: âm tiết sau không thể bắt đầu bằng phím dấu `s r x f j` (`hocsinh` ra `hóc` + `inh`).
10. **Hoàn tác thắng hiện chữ thô**: gõ đôi phím dấu là ý định rõ ràng nên từ hiện ngay dạng đã hoàn tác, không đợi dấu cách (`tesst` → `test`, `tess` → `tes`), kể cả khi chuỗi phím trông như từ tiếng Anh. **Phím vừa hoàn tác, gõ lại ngay thì chỉ là chữ** (`aaa` → `aa`, `aaaa` → `aaa`), và ở chế độ tự do cách đọc đã hoàn tác thắng cách đọc thô cùng hạng (`wws` → `ws`, `dddd` → `ddd`).
7. **Từ tiếng Việt hợp lệ có trong `vi.dic` không bao giờ bị trả về chữ thô** trừ khi có bằng chứng tiếng Anh mạnh (xem hàng 6).

## Dữ liệu

| File | Vai trò | Ai đọc |
|---|---|---|
| `data/dictionaries/vi.dic` | âm tiết có thật | `build.rs` → `VI`; sinh ngữ pháp |
| `data/dictionaries/names.dic` | tên riêng ngoài luật (kạn, kông, krông…) | chỉ lúc sinh `tables.rs` |
| `data/dictionaries/vi-non-syllables.txt` | mục của vi.dic không phải âm tiết | sinh bảng, audit |
| `data/dictionaries/keep.dic` | từ luôn giữ nguyên | `build.rs` → `KEEP` |
| `data/dictionaries/en/*.txt` | từ tiếng Anh, **mỗi loại một file**: `general` (từ thông dụng), `tech` (thuật ngữ lập trình/UI/hạ tầng), `brands` (tên sản phẩm, công ty), `chat` (từ hay gặp khi chat/làm việc) | `build.rs` gộp mọi `*.txt` (chữ thường, bỏ trùng) → `EN` |
| `data/telex_doubles.txt` | từ Anh chứa chữ đôi kiểu Telex (danh sách lịch sử của `general`); từ có chữ đôi trong các file `en/` khác được `build.rs` tự thêm | `build.rs` → `DOUBLES` |

### Thêm từ tiếng Anh

Muốn engine nhận ra thêm từ (không đoán theo từng từ), thêm vào đúng file trong `core/src/data/dictionaries/en/` hoặc tạo file `*.txt` mới cho một loại mới (`#` bắt đầu ghi chú, mỗi dòng một từ chữ thường a–z). Không phải sửa code, không phải sửa chỗ thứ hai. Kiểm tra:

| Lệnh | Cho biết |
|---|---|
| `cargo test --features engine_v2 --test suite dictionary_sources -- --nocapture` | mỗi file đúng định dạng, không trùng giữa các file; **độ phủ từng file**: bao nhiêu từ gõ ra đúng nguyên dạng (tối đa 10% đổi) và liệt kê từ bị đổi (thường do trùng âm tiết `vi.dic`: `toast`→`toát`) |
| `make gate-v2`, `scripts/test/dict.sh` | từ điển tiếng Việt vẫn 100%, tiếng Anh không tụt |

Từ trùng âm tiết tiếng Việt trong `vi.dic` vẫn ưu tiên tiếng Việt (quy tắc của bảng restore); thêm từ vào từ điển không đổi điều đó.

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
| `cargo run --release --features engine_v2 --example trace -- telex,free,ar hasaaaaaae` | màn hình sau từng phím (cờ `vni`, `ar`, `free`, `nw`): xem đúng trải nghiệm lúc gõ |
| `cargo run --release --features engine_v2 --example audit -- stretch < core/tests/data/vietnamese_telex_pairs.txt` | kéo dài nguyên âm cuối của từng âm tiết có dấu (`mùa` + `aaaa`), kiểm từng phím: chữ dài thêm một, dấu đứng yên, không dấu lạ chen giữa |
| `... --example audit -- jumps ar free < core/src/data/english_dict_merged.txt` | đếm từ tiếng Anh có dấu bị đổi vị trí khi gõ tiếp |
| `... --example audit -- runon free ar < core/tests/data/vietnamese_telex_pairs.txt` | ghép ngẫu nhiên 2 từ tiếng Việt gõ liền không dấu cách, đo tỉ lệ ra đúng |
| `... --example audit -- en_compound` | ghép 2 từ tiếng Anh gõ liền, đếm từ bị Gõ tự do làm đổi |
| `... --example audit -- free_cost` | CPU mỗi phím, số lần viết lại chữ, phím chậm nhất: gõ thường và Gõ tự do trên từ tiếng Anh |
| `... --example audit -- spin [free] [vn]` | vòng gõ 12 giây để gắn trình đo (`sample`, Instruments) |

Đo hiệu năng: máy dùng chung thì wall-clock vô nghĩa; `prof` dùng CPU time, lấy tốt nhất trong 25 lượt. So hai bản bằng cách chạy xen kẽ ≥ 20 lượt và lấy giá trị nhỏ nhất mỗi chỉ số; luôn chạy thêm một lần bản cũ lần hai để biết nhiễu (≈ 2%). Build hai binary (có/không `engine_v2`) vào hai thư mục riêng rồi chạy xen kẽ.

## Thêm một hành vi mới

1. Tìm ca người dùng đã báo (chuỗi phím, mong đợi) và thêm một dòng vào `core/tests/suite/issue_regressions.rs`.
2. Sửa ở tầng đúng: ngữ pháp (`phonology`), cách đọc (`compose/parse.rs`), hay quyết định cuối từ (`session/restore.rs`).
3. Chạy `make gate`, `make gate-v2`; cập nhật `docs/behavior-changes.md` nếu khác V1.
