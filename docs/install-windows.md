# Gõ Nhanh trên Windows

**Trạng thái:** Beta · Windows 10 (19041+) và Windows 11 · x64

## Cài đặt

1. Tải `GoNhanh-<phiên bản>-win-x64.zip` tại [Releases](https://github.com/khaphanspace/gonhanh.org/releases).
2. Giải nén và chạy `GoNhanh.exe` (~2 MB, không cần cài thêm runtime nào). Thư mục `bin` chứa công cụ [dòng lệnh](#dòng-lệnh-vim-script), không bắt buộc.
3. Icon ⚡ xuất hiện ở khay hệ thống (góc phải taskbar). Bấm mũi tên **^** nếu không thấy, có thể kéo icon ra taskbar để luôn hiển thị.

> File chưa được ký số nên lần đầu Windows SmartScreen có thể cảnh báo: chọn **More info → Run anyway**.

Muốn Gõ Nhanh tự chạy khi mở máy: mở Cài đặt → bật **Khởi động cùng Windows**. Gõ Nhanh tạo một tác vụ trong Task Scheduler (tên `GoNhanh autostart …`) chạy ngay khi đăng nhập, nên có bộ gõ trước các ứng dụng khởi động khác thay vì phải đợi chúng lần lượt chạy xong. Nếu chuyển Gõ Nhanh sang thư mục khác, chỉ cần mở bản mới một lần là tác vụ được cập nhật.

## Sử dụng

| Thao tác | Kết quả |
|----------|---------|
| Click icon ⚡ ở khay | Chuyển tiếng Việt ↔ tiếng Anh (icon có màu = tiếng Việt, xám = tiếng Anh) |
| `Ctrl` + `Space` | Chuyển tiếng Việt ↔ tiếng Anh |
| Click phải icon | Menu: bật tiếng Việt, kiểu gõ Telex/VNI, Cài đặt, Thoát |
| Rê chuột lên icon | Xem chế độ hiện tại, ví dụ "Tiếng Việt (Telex)" |

**Cài đặt** gồm: kiểu gõ, gõ W thành Ư, gõ `[ ]` thành Ơ Ư, tự khôi phục tiếng Anh, đặt dấu kiểu mới, phụ âm ngoại, tự viết hoa đầu câu, nhớ trạng thái theo ứng dụng, âm thanh, khởi động cùng Windows và **Bảng gõ tắt** (thêm, sửa, bật/tắt, nhập/xuất file).

### Quyền riêng tư

Gõ Nhanh hoạt động hoàn toàn offline: không quảng cáo, không thu thập dữ liệu. Ứng dụng chỉ kết nối GitHub để kiểm tra bản mới nếu bạn tự bật **Tự kiểm tra bản mới** trong Cài đặt (mặc định tắt).

## Dòng lệnh (Vim, script)

`bin\gonhanh.exe` (hoặc `bin\gn.exe`) điều khiển Gõ Nhanh đang chạy, cùng cú pháp với bản macOS/Linux và theo quy ước `im-select`, nên dùng được cho các plugin Vim tự chuyển bộ gõ khi đổi mode.

Để gõ `gn` ở mọi nơi: mở Cài đặt → **Hệ thống** → bật **Dòng lệnh gn**. Gõ Nhanh thêm thư mục `bin` vào `PATH` của tài khoản (tắt để gỡ ra), sau đó mở terminal mới là dùng được. Nếu chuyển Gõ Nhanh sang thư mục khác, tắt rồi bật lại mục này.

| Lệnh | Kết quả |
|------|---------|
| `gonhanh` | In trạng thái hiện tại: `vi` hoặc `en` |
| `gonhanh vi` / `gonhanh en` | Bật / tắt tiếng Việt, không in gì (dùng cho Vim/script) |
| `gonhanh on` / `off` / `toggle` | Bật / tắt / đảo trạng thái, in dòng trạng thái |
| `gonhanh telex` / `vni` | Đổi kiểu gõ và bật tiếng Việt |
| `gonhanh status` | Ví dụ `● BẬT │ telex │ gonhanh` |
| `gonhanh start` / `quit` | Mở Gõ Nhanh (đợi đến khi sẵn sàng, nên dùng được `gn start; gn vi`) / thoát Gõ Nhanh |
| `gonhanh completion` | In script gợi ý lệnh khi bấm `Tab` trong PowerShell |
| `gonhanh version` / `help` | Phiên bản / trợ giúp |

**Gợi ý lệnh bằng Tab (PowerShell):** thêm dòng sau vào `$PROFILE` (mở bằng `notepad $PROFILE`), rồi mở terminal mới. Gõ `gn ` rồi bấm `Tab` để lần lượt hiện các lệnh (`gn t` + `Tab` → `toggle`, `telex`):

```powershell
gn completion | Out-String | Invoke-Expression
```

Lệnh trả mã thoát `0` khi thành công, `1` kèm thông báo lỗi nếu Gõ Nhanh chưa chạy hoặc lệnh sai. Riêng `gn en` vẫn thành công khi Gõ Nhanh chưa chạy (lúc đó gõ phím vốn đã là tiếng Anh), nên Vim không báo lỗi mỗi lần nhấn `Esc`. Mỗi lệnh mất khoảng 10 ms. Lệnh `vi`/`en` không làm gì (không phát âm thanh) nếu trạng thái đã đúng. Dòng lệnh vẫn dùng được khi Gõ Nhanh chạy với quyền Administrator.

**Neovim** ([im-select.nvim](https://github.com/keaising/im-select.nvim)):

```lua
require("im_select").setup({
  default_command = "gonhanh",
  default_im_select = "en",
})
```

**VS Code** (VSCodeVim, `settings.json`):

```json
"vim.autoSwitchInputMethod.enable": true,
"vim.autoSwitchInputMethod.defaultIM": "en",
"vim.autoSwitchInputMethod.obtainIMCmd": "gonhanh",
"vim.autoSwitchInputMethod.switchIMCmd": "gonhanh {im}"
```

**Obsidian** (Vim Motions → Input method switching): IM binary path là đường dẫn tới `gonhanh.exe`, *Obtain IM arguments* để trống, *Switch IM arguments* là `{im}`, *Normal mode IM* là `en`.

## Quy tắc gõ

### Telex

| Gõ | Kết quả | Mô tả |
|----|---------|-------|
| `as`, `af`, `ar`, `ax`, `aj` | á, à, ả, ã, ạ | Dấu thanh |
| `aa`, `aw` | â, ă | Dấu mũ / trăng |
| `ee`, `oo` | ê, ô | Dấu mũ |
| `ow`, `uw` | ơ, ư | Dấu móc |
| `dd` | đ | Gạch ngang |

### VNI

| Gõ | Kết quả | Mô tả |
|----|---------|-------|
| `a1`-`a5` | á, à, ả, ã, ạ | Dấu thanh (1 sắc, 2 huyền, 3 hỏi, 4 ngã, 5 nặng) |
| `a6`, `a8` | â, ă | Dấu mũ (6) / trăng (8) |
| `o6`, `e6` | ô, ê | Dấu mũ |
| `o7`, `u7` | ơ, ư | Dấu móc (7) |
| `d9` | đ | Gạch ngang (9) |

## Hiệu năng

| Chỉ số | Kết quả đo |
|--------|-----------|
| Thời gian engine xử lý một phím | ~1–2 µs trung bình |
| Bộ nhớ khi chạy nền | ~0,2 MB (cột Memory trong Task Manager) |
| Kích thước | ~2 MB, một file exe |

## Xử lý sự cố

- **Không gõ được trong một ứng dụng:** ứng dụng đó có thể đang chạy với quyền Administrator; Windows chặn ứng dụng thường gửi phím vào ứng dụng quyền cao hơn.
- **Gõ ra chữ không dấu:** kiểm tra kiểu gõ (rê chuột lên icon); gõ Telex khi đang chọn VNI sẽ không ra dấu.
- **Trò chơi toàn màn hình:** một số game chặn keyboard hook; chuyển sang tiếng Anh khi chơi.

## Build từ mã nguồn

Xem [platforms/windows/README.md](../platforms/windows/README.md). Tóm tắt:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/setup/windows.ps1   # kiểm tra Rust, CMake, VS Build Tools
powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1   # → platforms/windows/publish/GoNhanh.exe
```
