# Gõ Nhanh trên Windows

**Trạng thái:** Beta · Windows 10 (19041+) và Windows 11 · x64

## Cài đặt

1. Tải `GoNhanh-<phiên bản>-win-x64.zip` tại [Releases](https://github.com/khaphanspace/gonhanh.org/releases).
2. Giải nén và chạy `GoNhanh.exe`. Không cần cài thêm runtime nào; chỉ một file ~2 MB.
3. Icon ⚡ xuất hiện ở khay hệ thống (góc phải taskbar). Bấm mũi tên **^** nếu không thấy, có thể kéo icon ra taskbar để luôn hiển thị.

> File chưa được ký số nên lần đầu Windows SmartScreen có thể cảnh báo: chọn **More info → Run anyway**.

Muốn Gõ Nhanh tự chạy khi mở máy: mở Cài đặt → bật **Khởi động cùng Windows**.

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
