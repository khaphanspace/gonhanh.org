# Issue coverage: V2 so với 251 issue của dự án

Nguồn: `gh issue list` (khaphanspace/gonhanh.org, 2026-10-08): **251 issue** (240 đóng, 11 mở). Phân nhóm theo **tầng gây lỗi**, vì mỗi tầng được bảo vệ bằng một cách khác nhau.

| Nhóm | Số issue | Tầng | V2 bảo đảm bằng | Kết luận |
|---|---|---|---|---|
| A. Gõ tiếng Việt sai (thứ tự phím, dấu, `đ`, `ươ`, chữ hoa) | 46 | lõi gõ | dấu là hàm của chữ cuối cùng (không phụ thuộc thứ tự), ngữ pháp sinh từ `vi.dic`; `issue_regressions` + từ điển 22k mọi thứ tự + `typing_order_permutation` | **Đã chặn** |
| B. Auto-restore tiếng Anh sai (mất/thừa chữ: `simss`, `conssole`, `Therere`, `mussic`, `momoo`) | 34 | lõi gõ (restore) | một bảng quyết định ở ranh giới từ; chữ gõ được tính từ phím thô nên không thể nhân đôi chữ; EN 100k, `issue_regressions` | **Đã chặn** các ca đã báo; còn từ Anh đọc ra âm tiết VN hợp lệ (`this`→`thí`) |
| C. Backspace / sửa từ / dán lại | 12 | lõi gõ + app | backspace ≡ quay về trạng thái phím trước; trạng thái ghim khi từ đang hiện chữ thô; bão phím 900k không lỗi | **Đã chặn** phần lõi; phần tiêm phím (Cmd+A, Option+Backspace) thuộc app |
| D. Gõ tắt | 22 | session | dùng lại `shortcut.rs` của V1 (không đổi), vòng đời mới có test (tắt IME, dấu câu, sau backspace, `->`) | **Giữ nguyên hành vi V1**; không có test riêng cho từng issue UI |
| E. Tự viết hoa | 6 | session | `autocap.rs` + `auto_capitalize_test` | Đã chặn phần engine; loại trừ app thuộc app |
| F. Phím tắt bật/tắt, input source, âm thanh | 28 | Swift (macOS) | không đổi bởi V2 | **Ngoài phạm vi engine** |
| G. Tương thích app (Terminal, Claude Code, Firefox URL, Spotlight, Notion, Excel, trình duyệt…) | 60 | Swift: cách tiêm phím, AX | không đổi bởi V2 | **Ngoài phạm vi engine**; V2 làm kết quả của lõi nhanh hơn và đúng, nhưng lỗi mất chữ do tiêm phím vẫn tồn tại |
| H. Cài đặt / cập nhật / ký code / quyền / Linux / hiệu năng hệ thống | 29 | build, hệ điều hành | không đổi | **Ngoài phạm vi engine** |
| I. Yêu cầu tính năng | 14 | nhiều tầng | xem bảng dưới | Một phần |

## Nhóm A: các nguyên nhân gốc và cách V2 chặn

| Nguyên nhân gốc (V1) | Issue | V2 |
|---|---|---|
| Dấu/mũ phụ thuộc thứ tự gõ (`ddwocj` ra `đưọc`, `duod`, `xuatas`, `neues`, `hieuer`, `vietes`) | #14 #24 #29 #124 #136 #172 #182 #183 #259 | Mỗi phím là chữ hoặc dấu; ngữ pháp loại cách đọc sai; dấu đặt trên âm tiết hoàn chỉnh. `compose_diff` kiểm mọi thứ tự trên 22k từ |
| `ươ`, `uơ`, `ưa` (`twong`, `giow`, `muwa`, `Quoiws`, `giuaw`) | #29 #74 #99 #106 #151 #243 | `ư`+`o` thành `ươ`; `uô`+`w` thành `ươ` |
| `w` đầu từ (`Ừm`, `Wf`) | #122 #199 #252 #384 | `w` đơn thành `ư`; `ww` trả `w` (chữ hoa theo phím đầu) |
| `đ` đơn/đầu từ, chữ hoa (`Dd`, `DDAAU`, `Đa`) | #166 #247 #333 | stroke áp dụng trên `d` đứng ngay trước; test chữ hoa |
| Đặt dấu `oà/òa`, `uý/úy` | #54 #64 | `tone_index` theo tuỳ chọn, kể cả khi dấu gõ sát nguyên âm (`xosa`, `tufy`) |
| Dấu mũ trễ (`thoio`→`thôi`, `viejc`) | #196 #371 | mũ trễ hợp lệ khi âm tiết hợp lệ |
| Dấu sau phụ âm cuối (`hangf nganf`, `khoangr`, `banwfg`) | #200 #253 #262 | cùng đường với mọi thứ tự phím |

## Nhóm B: nguyên nhân gốc

| Nguyên nhân gốc (V1) | Issue | V2 |
|---|---|---|
| Hoàn tác dấu kép rồi còn thừa chữ (`sims`→`simss`, `console`→`conssole`, `Therere`, `mussic`, `bussiness`, `momo`→`momoo`, `fomo`) | #131 #142 #193 #230 #296 #337 #348 #355 #356 #367 #427 | Chữ hiển thị tính lại từ chuỗi phím thô, không giữ danh sách chữ riêng; `momo` ra `môm` (không nhân đôi `o`) |
| Mất chữ khi từ tiếng Anh bị đọc như tiếng Việt (`respect`, `await`, `metric`, `cursor`, `view`, `useEffect`) | #15 #115 #116 #145 #147 #410 | bảng quyết định: từ không đọc được thành VN trả về chữ thô; mất ≥ 2 chữ do dấu bị ăn thì trả thô |
| `sếp`, `bếp` bị coi là tiếng Anh theo thứ tự gõ | #154 #163 | VN hợp lệ và có trong `vi.dic` thì giữ |
| Backspace rồi gõ lại (`serv⌫r`) | #197 | trạng thái ghim: hiện chữ thô thì xoá 1 phím = xoá 1 chữ |
| Từ không hợp lệ (`OTR`→`ỎT`) | #403 | `at_end`: âm tiết thiếu dấu/coda khi kết thúc từ là Invalid |

## Việc còn mở ở tầng engine

| Issue | Tình trạng |
|---|---|
| #356 `mos`+space | V2 ra `mó` (đúng VN). Báo cáo là `moss` do hoàn tác sai, đã hết |
| #211 chữ kéo dài `áaaa`, `nhéee` | V2 giữ chữ kéo dài khi từ VN hợp lệ sau khi thu gọn; ca `a`+`s`+`aaa` chưa có test, V1 cũng không đạt |
| #359 gõ tự do không theo chính tả (`khphá`, `qcáo`, `wé`) | **đã có** (`free_tone`): ngữ pháp thắng trước, tự do chỉ là dự phòng nên `wes`→`wé` khi bật phụ âm ngoại và từ VN đúng không đổi; test `issue_regressions`. App macOS chưa có nút bật (chỉ có cầu FFI `ime_free_tone`). Còn thiếu phím tạm tắt bằng Ctrl/Cmd (#360) |
| #393 `kông`, `kưng`, `kăng`, `zạ` | `kông` có (names.dic); `Kơ/Kư/Kă` chưa đưa vào vì chưa xác minh; `z j f w` bằng tuỳ chọn phụ âm ngoại |
| #180 #316 Simple Telex | **đã có**: tắt "Gõ W thành Ư" (`skip_w_shortcut`); V2 chạy cùng test |
| #232 `zị` | có với tuỳ chọn phụ âm ngoại |
| Từ Anh = âm tiết VN hợp lệ (`this`, `rest`, `giro`) | cần từ điển EN lớn hơn, xem `docs/behavior-changes.md` |


## Gõ tự do: các bộ gõ lâu đời làm gì, Gõ Nhanh làm gì

Đọc mã nguồn UniKey (ukengine, qua ibus-unikey) và OpenKey; ba khái niệm khác nhau thường bị gộp thành "gõ tự do":

| Khái niệm | UniKey / OpenKey | Gõ Nhanh |
|---|---|---|
| **Bỏ dấu tự do** (free marking): phím dấu/mũ/`đ` được gõ sau khi đã gõ thêm chữ, không cần ngay sau nguyên âm | tuỳ chọn; tắt thì dấu chỉ ăn khi nó là phím liền trước | **luôn bật** (`toanf`, `ddojc`, `dojc` đều ra chữ đúng, issue #32) |
| **Kiểm tra chính tả**: chữ không đúng ngữ pháp thì không áp dấu; tắt thì từ nào hết là tiếng Việt, chữ kế tiếp được coi là *bắt đầu một từ mới* và gõ tiếp bình thường (`khphas` → `kh` + `phá`) | tuỳ chọn, bật mặc định; OpenKey có phím tạm tắt (Control) | bật mặc định; tắt = "gõ tự do": ngữ pháp vẫn thắng trước, chỗ nào không có cách đọc đúng thì áp dấu đã gõ (`khphá`, `qcáo`, `wé`) |
| **Ngắt nhịp**: dừng áp dấu giữa từ mà không cần dấu cách (`kh` ⌃ `phá`; `q` ⌃ `c` không bung gõ tắt `qc`) | OpenKey: Control tạm tắt kiểm tra chính tả | **đã có từ #150**: chạm Control xoá bộ đệm từ; lõi V2 có test (`control_tap_breaks_the_word_and_the_shortcut`) |
| **Phụ âm ngoại** `z w j f` | OpenKey có (chỉ nới bộ kiểm tra; phím `w` vẫn ra `ư` đầu từ); UniKey không | gộp vào **Gõ tự do** (không còn công tắc riêng): `zij`→`zị`, `wes`→`wé`, nhưng `wa`→`ưa`, `wm`→`ưm` (có cách đọc đúng thì thắng); ai đã bật tuỳ chọn cũ được chuyển sang Gõ tự do |
| **Khôi phục từ sai** khi hết từ | UniKey/OpenKey: nếu từ không hợp lệ thì trả phím thô | có, dùng từ điển tiếng Anh (chính xác hơn: không trả những từ chưa chắc là tiếng Anh) |

Khác biệt chính với UniKey khi tắt kiểm tra chính tả: UniKey *cắt từ* tại chỗ hết hợp lệ rồi áp luật Việt cho từng đoạn; Gõ Nhanh giữ một từ nhưng đặt dấu trên nguyên âm của đoạn có nguyên âm (`khphas` → `khphá`, cùng kết quả cho các ca trong issue). Cùng thứ tự gõ, hai cách ra cùng chữ vì dấu luôn đặt trên cụm nguyên âm.

### Nguyên lý Gõ tự do (rút từ cách người dùng thực sự gõ)

Người gõ tự do viết tắt hoặc tên riêng có **phụ âm đầu lạ** (`khph`, `qc`, `z`, `w`, `f`, `j`) rồi một vần đúng. Họ không gõ vần sai chủ ý. Nên:

| Nguyên lý | Ví dụ |
|---|---|
| Nới đúng một chỗ: phụ âm đầu. Vần (nguyên âm + coda + dấu) vẫn phải là tiếng Việt | `khphas`→`khphá`, `qcaos`→`qcáo`, `zij`→`zị`, `wes`→`wé` |
| Coda lạ hoặc cụm nguyên âm lạ không có cách đọc tự do: giữ như đã gõ | `ads`→`ads`, `expect`→`expect`, `haaas` |
| Có cách đọc đúng ngữ pháp thì thắng cách đọc tự do | `wa`→`ưa`, `wm`→`ưm` |
| Từ tiếng Anh đúng từ điển thắng cách đọc có phụ âm đầu lạ (bật khôi phục) | `west`, `were`, `warm`, `foresee` giữ nguyên; `wes`→`wé` vì không phải từ Anh |
| Dấu không nhảy trong từ không còn là tiếng Việt | `hasaaaaaae`: `háaaaaae`, dấu đứng yên trên `a` đầu |

Đo trên 17.641 từ Anh khi bật Gõ tự do + khôi phục: từ bị đổi 397 → 405 so với không bật Gõ tự do (trước khi sửa: dấu nhảy ở 629 từ, giờ 151; phần còn lại là chuyển vần Việt hợp lệ như `lìe`→`liè`).

Cài đặt trên macOS gọi công tắc này là **Gõ tự do** (tắt mặc định), cách người dùng thường gọi; lưu trong khoá `gonhanh.freeTone`. Tên "Kiểm tra chính tả" của UniKey/OpenKey bị bỏ vì khó hiểu với người dùng mới.

## Issue mở (11): #349, #359, #360, #374, #375, #381, #384, #400, #406, #417, #426

Không issue mở nào là lỗi của lõi: #349 (Dvorak), #374 (hai user macOS), #375 (cài Linux), #381 (nguồn nhập Unicode Hex), #384 (Atlas/Facebook comment, đã thêm test chữ hoa: lõi đúng), #400 (mất chữ ở Telegram: lõi đúng `quets`→`quét`, lỗi do tiêm phím), #406 (lag), #417 (Playwright), #426 (thanh địa chỉ Firefox) thuộc app/nền tảng.

## Danh sách đầy đủ theo nhóm

**A. Gõ tiếng Việt** (46): #14, #24, #29, #32, #44, #48, #54, #64, #74, #87, #88, #99, #105, #106, #110, #111, #122, #124, #125, #127, #136, #151, #159, #162, #166, #169, #172, #182, #183, #196, #200, #236, #243, #247, #252, #253, #259, #262, #272, #303, #318, #333, #340, #356, #371, #400

**B. Auto-restore tiếng Anh** (34): #15, #26, #39, #51, #57, #115, #116, #118, #131, #142, #145, #146, #147, #149, #154, #163, #184, #193, #197, #229, #230, #246, #296, #319, #321, #337, #339, #348, #355, #367, #402, #403, #410, #427

**C. Backspace / sửa từ** (12): #98, #100, #194, #212, #217, #251, #293, #294, #298, #311, #361, #380

**D. Gõ tắt** (22): #23, #25, #52, #58, #86, #107, #123, #128, #129, #130, #161, #167, #178, #275, #329, #343, #352, #363, #376, #382, #383, #387

**E. Tự viết hoa** (6): #27, #133, #185, #245, #274, #302

**F. Phím tắt / input source** (28): #19, #40, #41, #46, #50, #53, #92, #95, #119, #150, #157, #165, #168, #171, #173, #181, #198, #216, #220, #268, #292, #307, #381, #399, #401, #407, #416, #424

**G. Tương thích app** (60): #13, #16, #21, #30, #34, #35, #36, #38, #55, #94, #109, #132, #140, #155, #160, #164, #170, #175, #190, #191, #192, #195, #199, #218, #225, #238, #255, #258, #261, #264, #271, #282, #283, #297, #323, #326, #327, #336, #345, #362, #364, #365, #369, #373, #378, #384, #385, #388, #391, #392, #395, #396, #404, #408, #411, #413, #417, #418, #425, #426

**H. Cài đặt / hệ thống** (29): #18, #20, #33, #42, #43, #62, #90, #93, #104, #134, #137, #138, #143, #148, #186, #207, #224, #347, #351, #354, #366, #368, #374, #375, #386, #390, #405, #406, #412

**I. Yêu cầu tính năng** (14): #5, #56, #65, #158, #180, #211, #232, #235, #286, #316, #349, #359, #360, #393

## Bộ test chặn tái phát

- `core/tests/suite/issue_regressions.rs`: 100 ca, mỗi ca ghi số issue; chỉ chạy với `--features engine_v2` (V1 sai 2 ca: #197, #410; V2 sai 0).
- `typing_order_permutation_test`, `compose_diff`, `phonology_audit`, từ điển 22k và EN 100k: các lớp lỗi theo thứ tự phím và ngữ pháp.
- `engine_fuzz`: 900k phím ngẫu nhiên, không panic và không xin xoá quá một từ.

Muốn thêm một lỗi mới: thêm một dòng `(issue, chế độ, gõ gì, kỳ vọng)` vào `CASES`.
