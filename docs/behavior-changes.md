# Behavior changes V1 → V2

Chạy bộ test V1 (866 test) trên engine V2: `cargo test --profile gate --features engine_v2 --test suite -- --skip golden_digest`.
Kết quả: cổng `make gate-v2` (`scripts/test/v2.sh`) chạy toàn bộ test V1 trên V2; **chỉ 20 test được phép fail**, liệt kê ở `core/tests/v2-known-differences.txt` (bảng dưới; `aumf` và `sax` mỗi cái làm hỏng 2 test). Test mới ngoài danh sách fail, hoặc test trong danh sách bỗng qua, đều làm cổng đỏ. Mọi test còn lại — gõ Telex/VNI, dấu, gõ tắt, auto-capitalize, ESC, backspace/lịch sử từ, phím ngắt, `[ ]`, auto-restore, từ điển 22k, 100k EN — giống V1.

Mỗi dòng: V1 làm gì, V2 làm gì, vì sao V2 đúng hơn hoặc không thể giữ. Khi cutover (P5) các test mã hoá hành vi V1 ở cột "Test" được sửa kỳ vọng theo cột V2.

## Khác biệt do V2 theo ngữ pháp (docs là spec)

| Gõ | V1 | V2 | Lý do | Test |
|---|---|---|---|---|
| `aia` | `âi` | `aia` | `âi` không phải vần tiếng Việt (§7.6 whitelist) | `engine_test` |
| `buong67` (VNI) | `buơng` | `bương` | `ơng` không tồn tại; `uo`+7 luôn là ươ khi có coda | `typing_test` |
| `wo ` (AR) | `ươ` | `wo` | `ươ` không có coda không kết thúc được từ (vi.dic không có) | `auto_restore_dynamic` |
| `uwis ` (AR) | `ứi` | `uwis` | `ưi` chỉ đứng sau phụ âm đầu (gửi, ngửi); khớp `wi → wi` | `engine_test` |
| `aumf` | `àum` | `aumf` | `aum` không hợp lệ nên không đặt dấu (V1 chỉ kiểm cấu trúc) | `integration_test` ×2 |
| `kanjz` | `kan` | `kanjz` | `kạn` chỉ là tên riêng; dấu chỉ được giữ khi cả tên khớp | `bug_reports_test` |
| `cusor ` (AR) | `cuỏ` | `cusor` | `cuo` chờ ô/ơ + coda; kết thúc từ ở đó không thể là âm tiết VN. `mire`, `lire`, `ire`… cùng dạng và V1 restore | `revert_auto_restore_test` |
| `lisa ` (AR) | `lía` | `lisa` | `lisa` là từ tiếng Anh/tên có trong từ điển, `lía` không có trong vi.dic (cùng nhóm `bore`, `pair`) | `engine::tests::test_interleaved_diphthong_auto_restore` |
| `booos ` | `boó` | `bóo` | dấu là hàm của chữ cuối: sau khi hoàn tác `ooo`→`oo`, cặp `oo` mở đặt dấu ở chữ đầu | `engine::tests::test_literal_after_circumflex_revert` |
| `muafaa ` (AR) | `muàa` | `mùaa` | V1: `muafaa`→`muàa` nhưng `mufaaa`→`mùaa` (phụ thuộc thứ tự gõ). V2: dấu là hàm của chữ cuối (I7) | `english_auto_restore_test` |

## Khác biệt do V1 vá theo từng từ (CLAUDE.md cấm), V2 dùng từ điển + dấu hiệu chung

| Gõ (AR bật) | V1 | V2 | Lý do | Test |
|---|---|---|---|---|
| `sax `, `rims ` | `sax`, `rims` | `sã`, `rím` | V1 có pattern riêng cho từng từ (`has_english_modifier_pattern`: `sax`, `sims`); `sã`, `rím` là âm tiết hợp lệ và `sax/rims` không có trong từ điển EN | `english_auto_restore_test`, `paragraph_test` |
| `post `, `queue ` | `pót`, `quêu` | `post`, `queue` | Âm tiết VN không có trong vi.dic và (từ EN có trong từ điển, hoặc phụ âm đầu `p` vay mượn, hoặc phải đổi dấu giữa chừng) → trả về EN (cùng nhóm `bore`, `core`, `pair`, `pais`, `arts`) | `auto_restore_dynamic` |
| `tafoo `, `chaofo ` | `tàoo`, `chàoo` | `tafoo`, `chaofo` | Vần không hợp lệ, V2 trả chuỗi thô; V1 giữ rác do luật "dấu rồi nguyên âm đôi khác" | `english_auto_restore_test` |
| `abc ook` | `abc ôk` | `abc ook` | `ôk` không hợp lệ (chỉ `ôc`), không có dấu hiệu VN mạnh (đ, ư, ơ, ă) | `integration_test` |
| `theref ` (r rồi f) | `theref` | `thề` | `thề` là âm tiết hợp lệ, người gõ đổi dấu giữa chừng là hành vi VN bình thường | `bug_reports_test` (issue230) |

## Không phải khác biệt

- Hiển thị khi từ không còn cách đọc tiếng Việt: V1 giữ mọi dấu đã áp dụng (`tẽt`, `ăi`); V2 giữ đúng như vậy khi auto-restore tắt. Khi auto-restore bật V2 hiện chữ gõ thô (`text`) trừ khi từ có đ/ư/ơ/ă gõ chủ động (`đc`, `chứcc`): giảm nhấp nháy, không đổi kết quả sau dấu cách.
- Backspace khi chữ vừa xoá không khớp trạng thái nào trước đó (`ươ` → `ư`): V2 phát lại phím trừ phím tạo chữ đó; V1 pop buffer.

## Hiệu năng và độ tin cậy (đo 2026-10-08, CPU time, tốt nhất trong 25 lượt, máy tải cao)

| | V1 | V2 |
|---|---|---|
| ns/phím, tiếng Việt, auto-restore tắt / bật | 728 / 1849 | **424 / 709** |
| ns/phím, tiếng Anh, auto-restore tắt / bật | 610 / 1580 | **510 / 718** |
| Cấp phát/phím (bench `vi_auto_off`) | 7,0 (baseline) / 5,1 | **0,37** (chỉ ở ranh giới từ; lõi gõ 0) |
| Phím đầu tiên | 30 ms (baseline) | **8–30 µs** |
| Backspace/từ (EN, bật restore) | 1,96 | **1,64** |
| Lỗi tiếng Anh 100k (bật restore) | 2.607 | **1.869** (V2 sai mà V1 đúng: 68 từ; V1 sai mà V2 đúng: 806) |
| Từ điển 22k VN/Telex/VNI | 100% | 100% |
| Bão phím ngẫu nhiên 900k phím (`engine_fuzz`) | yêu cầu xoá 60 ký tự ở phím 4.385 (đã tái hiện) | không lỗi |

Nguồn tối ưu V2: không khởi tạo mảng lớn mỗi phím (đọc/ghi beam tại chỗ), chuỗi `keep` tính lười, bảng vần → tra 1 lần nhị phân (`NUCLEUS_INDEX`, sinh từ vi.dic), phụ âm đầu bằng `match`, kết quả của session là `Out` (≈140 B) rồi mới đóng gói `Result` (1 KB) một lần.

## Còn lại

- Các từ tiếng Anh đọc được thành âm tiết VN hợp lệ (`this`→`thí`, `rest`→`rét`, `giro`→`giỏ`) không phân biệt được nếu thiếu từ điển lớn; V2 giữ tiếng Việt vì từ điển EN đi kèm nhỏ. Mở rộng từ điển EN là việc dữ liệu, không đổi code.
- `Entry` lịch sử sau khi `Keep` thay đổi hiển thị: backspace-sau-cách phát lại theo `literal`; chưa có test.
