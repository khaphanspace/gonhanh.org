# Gõ Nhanh trên Windows

Gõ tiếng Việt cực nhanh với Telex/VNI. Hỗ trợ Windows 10/11, tích hợp system tray, bật/tắt nhanh với Ctrl+Space.

---

## Cài đặt

### Cách 1: Tải về (Khuyến nghị)

1. **Tải DMG/Installer:** [GoNhanh releases](https://github.com/khaphanspace/gonhanh.org/releases/latest)

2. **Chạy installer** và hoàn thành setup wizard

3. **Khởi động:** App sẽ tự chạy ở system tray (biểu tượng "GN")

4. **Cấp quyền:** Nếu Windows hỏi, nhấp "Yes" để cho phép keyboard access

---

## Sử dụng

| Phím tắt | Chức năng |
|----------|-----------|
| `Ctrl + Space` | Bật/tắt tiếng Việt (Telex) |
| Tray icon menu | Đổi gõ tắt, cài đặt, tắt app |

### Telex (mặc định)

| Gõ | Kết quả |
|----|---------|
| `as`, `af`, `ar`, `ax`, `aj` | á, à, ả, ã, ạ |
| `aa`, `aw`, `ee`, `oo` | â, ă, ê, ô |
| `ow`, `uw`, `dd` | ơ, ư, đ |

### VNI (chuyển từ tray menu)

| Gõ | Kết quả |
|----|---------|
| `a1`, `a2`, `a3`, `a4`, `a5` | á, à, ả, ã, ạ |
| `a6`, `a8`, `o6`, `e6` | â, ă, ô, ê |
| `o7`, `u7`, `d9` | ơ, ư, đ |

---

## Tray Menu

Nhấp vào biểu tượng "GN" ở system tray:

- **Enable/Disable** - Bật/tắt gõ tiếng Việt
- **Input Method** - Chọn Telex hoặc VNI
- **Settings** - Cấu hình ưa thích
- **About** - Xem phiên bản và thông tin

---

## Khắc phục sự cố

### App không xuất hiện
1. Tìm biểu tượng "GN" ở system tray (góc phải taskbar)
2. Nếu không thấy, khởi động lại app từ menu Start

### Ctrl+Space không hoạt động
1. Mở Settings (từ tray menu)
2. Đảm bảo "Enable on startup" được bật
3. Thử restart app

### Text không xuất hiện đúng
1. Kiểm tra input method trong tray menu
2. Tắt app khác có thể can thiệp (antivirus, macro tools)

---

## Theo dõi & Phản hồi

- [Releases](https://github.com/khaphanspace/gonhanh.org/releases)
- [GitHub Issues](https://github.com/khaphanspace/gonhanh.org/issues)
- Báo cáo lỗi: Bao gồm phiên bản + bước tái hiện

---

## Dành cho Developer

<details>
<summary>Build từ source</summary>

**Yêu cầu:**
- Windows 10/11
- [Rust](https://rustup.rs/) (x86_64-pc-windows-msvc + aarch64-pc-windows-msvc targets)
- [Visual Studio 2022](https://visualstudio.microsoft.com/) (C++ & .NET 8+ workload)
- [.NET 8 SDK](https://dotnet.microsoft.com/download/dotnet/8.0)

**Các bước:**

```powershell
# Clone repository
git clone https://github.com/khaphanspace/gonhanh.org.git
cd gonhanh.org

# Build Rust core
pwsh scripts/build-windows.ps1

# Build .NET WPF app
cd platforms/windows/GoNhanh.Windows
dotnet build -c Release
dotnet publish -c Release
```

**Output:** `platforms/windows/GoNhanh.Windows/bin/Release/net8.0-windows10.0.17763.0/win-x64/publish/`

**Architecture:**
- Rust core: `libgonhanh_core.dll` (P/Invoke FFI)
- WPF UI: `GoNhanh.exe` (system tray app)
- Keyboard hook: `SetWindowsHookEx` (WH_KEYBOARD_LL)

Xem [system-architecture.md](./system-architecture.md) cho chi tiết kiến trúc.

</details>
