# Lõi gõ: kiến trúc

Tài liệu bắt buộc đọc trước khi sửa engine. FFI: 22 hàm, `Result` 1028 B (`MAX` = 256). Lý do từng quyết định theo issue: [issue-coverage.md](issue-coverage.md). Ngữ pháp tiếng Việt: [vietnamese-language-system.md](vietnamese-language-system.md), [validation-algorithm.md](validation-algorithm.md).

## Ba tầng, mỗi tầng một câu hỏi

```
core/src/
├─ phonology/   "Đây có phải tiếng Việt không? Dấu đặt ở đâu?"      thuần, không cấp phát, không biết bàn phím
├─ compose/     "Chuỗi phím này đang gõ ra chữ gì?"                  beam các cách đọc, không biết từ điển Anh
└─ engine/      "Từ này kết thúc thì làm gì?"                         dấu cách, ngắt, ESC, gõ tắt, lịch sử, restore
```

Chiều phụ thuộc chỉ đi xuống: `engine` → `compose` → `phonology`. Từ điển tiếng Anh chỉ `engine/restore.rs` dùng, và chỉ ở ranh giới từ.

## Luồng một phím

```
phím ─ Engine::on_key_ext
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
9. **Gõ liền nhiều âm tiết (chỉ khi bật Gõ tự do)**: khi từ không còn cách đọc tiếng Việt, engine tách nó thành các âm tiết hoàn chỉnh (`xinchaof` → `xin` + `chào`, `thuwrgoxTieengsVieetj` → `thửgõTiếngViệt`) rồi viết lại phần đang hiện. Từ đó dấu và phụ âm chỉ tác động lên âm tiết cuối. Không cắt khi (a) chuỗi phím là tiếng Anh: một từ, tiền tố của một từ, hoặc nhiều từ (mỗi từ ≥ 3 chữ) nối nhau (`expect`, `thanks`, `helloworld`, `popover` = `pop` + `over`), bất kể auto-restore; (b) không tách trọn được thành âm tiết. Chỉ đoạn đầu được nới phụ âm đầu (`khphá`); các đoạn sau phải là âm tiết tiếng Việt thật, và đoạn bị đóng phải có ít nhất 2 chữ. Mỗi đoạn bị đóng nhận cùng quyết định của bảng restore như lúc gặp dấu cách. Giới hạn của Telex: âm tiết sau không thể bắt đầu bằng phím dấu `s r x f j` (`hocsinh` ra `hóc` + `inh`).
10. **Gõ đôi để hoàn tác luôn về dạng thường**, với mọi âm tiết, mọi chế độ, ngay lúc gõ và cả ở dấu cách (`perr` → `per`, `baaa` → `baa`, `ddd` → `dd`, `ww` → `w`, VNI `ba11` → `ba1`). Từ điển chỉ đổi điều đó ở dấu cách khi chữ gõ **chính là** một từ có chữ đôi (`perry`, `class`), không bao giờ bằng cách đoán theo tiền tố hay danh sách mục không phải từ (`terr`). Danh sách chữ đôi (`DOUBLES`) chỉ giữ từ có trong từ điển tiếng Anh. Bộ test tính chất `a_doubled_key_goes_back_to_the_plain_letters_for_any_syllable` và bản VNI kiểm hơn 13 nghìn tổ hợp.
10b. **Hoàn tác thắng hiện chữ thô**: gõ đôi phím dấu là ý định rõ ràng nên từ hiện ngay dạng đã hoàn tác, không đợi dấu cách (`tesst` → `test`, `tess` → `tes`), kể cả khi chuỗi phím trông như từ tiếng Anh. **Phím vừa hoàn tác, gõ lại ngay thì chỉ là chữ** (`aaa` → `aa`, `aaaa` → `aaa`), và ở chế độ tự do cách đọc đã hoàn tác thắng cách đọc thô cùng hạng (`wws` → `ws`, `dddd` → `ddd`).
7. **Từ tiếng Việt hợp lệ có trong `vi.dic` không bao giờ bị trả về chữ thô** trừ khi có bằng chứng tiếng Anh mạnh (xem hàng 6).

## Dữ liệu

| File | Vai trò | Ai đọc |
|---|---|---|
| `data/dictionaries/vi.dic` | âm tiết có thật | `build.rs` → `VI`; sinh ngữ pháp |
| `data/dictionaries/names.dic` | tên riêng ngoài luật (kạn, kông, krông…) | chỉ lúc sinh `tables.rs` |
| `data/dictionaries/vi-non-syllables.txt` | mục của vi.dic không phải âm tiết | sinh bảng, audit |
| `data/dictionaries/keep.dic` | từ luôn giữ nguyên | `build.rs` → `KEEP` |
| `data/dictionaries/en/*.txt` | từ tiếng Anh, **mỗi loại một file**: `general` (từ thông dụng), `tech` (thuật ngữ lập trình/UI/hạ tầng), `brands` (tên sản phẩm, công ty), `chat` (từ hay gặp khi chat/làm việc) | `build.rs` gộp mọi `*.txt` (chữ thường, bỏ trùng) → `EN` |
| `data/dictionaries/en-ref/*.txt` | danh sách tham khảo từ hiếm (Webster 2nd, miền công cộng, 4–15 chữ: `revert`, `popover`…), **chỉ** dùng nơi bằng chứng tiếng Việt yếu: không hiện dấu cho cách đọc chỉ do Gõ tự do cho phép khi chữ gõ còn là (tiền tố của) từ tiếng Anh, kể cả khi tắt auto-restore | `build.rs` → `REF` |
| `data/telex_doubles.txt` | từ Anh chứa chữ đôi kiểu Telex (danh sách lịch sử của `general`); từ có chữ đôi trong các file `en/` khác được `build.rs` tự thêm | `build.rs` → `DOUBLES` |

### Nhận biết từ tiếng Anh: hai danh sách, một quy tắc

- `EN` (thư mục `en/`, tuyển chọn) là **bằng chứng mạnh**: chữ gõ là một từ trong đó thì bảng restore trả về chữ thô; dạng đã huỷ cũng là từ trong đó thì giữ dạng huỷ (`lissa` → `lisa`). Từ hiếm trong đây làm hỏng các quyết định này (`sory`, `teet` của Webster biến `sorry`, `têt` thành sai), nên không nạp danh sách lớn vào `EN`.
- `REF` (thư mục `en-ref/`, danh sách lớn) là **bằng chứng yếu**: chỉ quyết định khi cách đọc tiếng Việt chính nó yếu (chỉ hợp lệ nhờ Gõ tự do).
- Cả hai nhận dạng biến cách có quy tắc (`knows_inflected`, `begins_inflected`: số nhiều, `-ed`, `-ing`, `-er`, `-ly`, gốc ≥ 4 chữ): danh sách chỉ chứa dạng gốc.
- Không để chuỗi một chữ lặp (`ww`, `ss`, `aaaa`) hay cặp 2 chữ vô nghĩa (`qc`, `wm`) trong từ điển: chúng thắng thao tác huỷ phím và đụng các viết tắt của Gõ tự do (`dictionary_sources` kiểm).

### Thêm từ tiếng Anh

Muốn engine nhận ra thêm từ (không đoán theo từng từ), thêm vào đúng file trong `core/src/data/dictionaries/en/` hoặc tạo file `*.txt` mới cho một loại mới (`#` bắt đầu ghi chú, mỗi dòng một từ chữ thường a–z). Không phải sửa code, không phải sửa chỗ thứ hai. Kiểm tra:

| Lệnh | Cho biết |
|---|---|
| `cargo test --test suite dictionary_sources -- --nocapture` | mỗi file đúng định dạng, không trùng giữa các file; **độ phủ từng file**: bao nhiêu từ gõ ra đúng nguyên dạng (tối đa 10% đổi) và liệt kê từ bị đổi (thường do trùng âm tiết `vi.dic`: `toast`→`toát`) |
| `make gate`, `scripts/test/dict.sh` | từ điển tiếng Việt vẫn 100%, tiếng Anh không tụt |

Từ trùng âm tiết tiếng Việt trong `vi.dic` vẫn ưu tiên tiếng Việt (quy tắc của bảng restore); thêm từ vào từ điển không đổi điều đó.

Sửa `vi.dic` hoặc `names.dic`: chạy `python3 scripts/gen/phonology_tables.py`; `make gate` kiểm bằng `--check`.

## Kiểm thử và đo

| Lệnh | Việc |
|---|---|
| `make t` | test đơn vị (giây) |
| `make gate` | lib + suite + từ điển + bảng sinh còn mới |
| `cargo test --profile gate --test suite issue_regressions` | 150+ ca rút từ issue |
| `GN_FUZZ_KEYS=2000000 cargo test --profile gate --test suite engine_fuzz` | bão phím ngẫu nhiên |
| `cargo run --release --example prof` | CPU time/phím theo từng lớp |
| `cargo run --release --example try -- telex_ar "vieetj_nam_"` | thử nhanh (`_` dấu cách, `<` backspace) |
| `cargo run --release --example dbg -- repea` | cách đọc thắng ở từng phím |
| `cargo run --release --example trace -- telex,free,ar hasaaaaaae` | màn hình sau từng phím (cờ `vni`, `ar`, `free`, `nw`): xem đúng trải nghiệm lúc gõ |
| `cargo run --release --example audit -- stretch < core/tests/data/vietnamese_telex_pairs.txt` | kéo dài nguyên âm cuối của từng âm tiết có dấu (`mùa` + `aaaa`), kiểm từng phím: chữ dài thêm một, dấu đứng yên, không dấu lạ chen giữa |
| `... --example audit -- jumps ar free < core/src/data/english_dict_merged.txt` | đếm từ tiếng Anh có dấu bị đổi vị trí khi gõ tiếp |
| `... --example audit -- runon free ar < core/tests/data/vietnamese_telex_pairs.txt` | ghép ngẫu nhiên 2 từ tiếng Việt gõ liền không dấu cách, đo tỉ lệ ra đúng |
| `cargo run --release --example cancel_audit -- [free] < words.txt` | gõ từng từ tiếng Anh với phím dấu gõ đôi ngay sau nguyên âm đã bị đặt dấu (`dis` + `s` + `connect`), đếm từ không trở về đúng chính tả |
| `... --example audit -- en_compound` | ghép 2 từ tiếng Anh gõ liền, đếm từ bị Gõ tự do làm đổi |
| `... --example audit -- free_cost` | CPU mỗi phím, số lần viết lại chữ, phím chậm nhất: gõ thường và Gõ tự do trên từ tiếng Anh |
| `... --example audit -- spin [free] [vn]` | vòng gõ 12 giây để gắn trình đo (`sample`, Instruments) |

Đo hiệu năng: máy dùng chung thì wall-clock vô nghĩa; `prof` dùng CPU time, lấy tốt nhất trong 25 lượt. So hai bản bằng cách chạy xen kẽ ≥ 20 lượt và lấy giá trị nhỏ nhất mỗi chỉ số; luôn chạy thêm một lần bản cũ lần hai để biết nhiễu (≈ 2%). Build hai binary (trước/sau thay đổi) vào hai thư mục riêng rồi chạy xen kẽ.

## Thêm một hành vi mới

1. Tìm ca người dùng đã báo (chuỗi phím, mong đợi) và thêm một dòng vào `core/tests/suite/issue_regressions.rs`.
2. Sửa ở tầng đúng: ngữ pháp (`phonology`), cách đọc (`compose/parse.rs`), hay quyết định cuối từ (`engine/restore.rs`).
3. Chạy `make gate`; cập nhật tài liệu này nếu đổi một bất biến.
