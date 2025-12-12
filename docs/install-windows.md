# Hướng dẫn cài đặt Gõ Nhanh trên Windows

> **Trạng thái**: Đang phát triển (Coming Soon)

---

## Thông tin

Gõ Nhanh cho Windows đang được phát triển tích cực với các công nghệ:

- **UI Framework**: WPF (Windows Presentation Foundation)
- **Core Engine**: Rust (chia sẻ với macOS và Linux)
- **Hỗ trợ**: Windows 10 và Windows 11

### Tính năng dự kiến

- Gõ tiếng Việt với kiểu **Telex** và **VNI**
- Gõ tắt tùy chỉnh
- Danh sách ứng dụng ngoại lệ
- System tray icon với menu nhanh
- Tự động khởi động cùng Windows

---

## Cài đặt (Khi phát hành)

_Hướng dẫn cài đặt sẽ được cập nhật khi phiên bản Windows chính thức phát hành._

### Yêu cầu hệ thống (Dự kiến)

- **Hệ điều hành**: Windows 10 version 1903+ hoặc Windows 11
- **Kiến trúc**: x64

---

## Nâng cấp (Upgrade)

_Sẽ cập nhật sau khi phát hành._

---

## Gỡ cài đặt (Uninstall)

_Sẽ cập nhật sau khi phát hành._

---

## Hướng dẫn sử dụng

_Sẽ cập nhật sau khi phát hành._

### Quy tắc gõ Telex (Tham khảo)

| Phím | Dấu | Ví dụ |
|------|-----|-------|
| `s` | Sắc | `as` → á |
| `f` | Huyền | `af` → à |
| `r` | Hỏi | `ar` → ả |
| `x` | Ngã | `ax` → ã |
| `j` | Nặng | `aj` → ạ |
| `aa` | Â | `aa` → â |
| `aw` | Ă | `aw` → ă |
| `ee` | Ê | `ee` → ê |
| `oo` | Ô | `oo` → ô |
| `ow` | Ơ | `ow` → ơ |
| `uw` | Ư | `uw` → ư |
| `dd` | Đ | `dd` → đ |

### Quy tắc gõ VNI (Tham khảo)

| Phím | Dấu | Ví dụ |
|------|-----|-------|
| `1` | Sắc | `a1` → á |
| `2` | Huyền | `a2` → à |
| `3` | Hỏi | `a3` → ả |
| `4` | Ngã | `a4` → ã |
| `5` | Nặng | `a5` → ạ |
| `6` | Mũ (Â, Ê, Ô) | `a6` → â |
| `7` | Móc (Ư, Ơ) | `u7` → ư |
| `8` | Trăng (Ă) | `a8` → ă |
| `9` | Đ | `d9` → đ |

---

## Dành cho Developers

Nếu bạn muốn đóng góp hoặc thử nghiệm phiên bản dev:

### Yêu cầu

- Windows 10/11
- [Rust toolchain](https://rustup.rs/)
- [Visual Studio 2022](https://visualstudio.microsoft.com/) với workload:
  - Desktop development with C++
  - .NET desktop development

### Build từ source

```powershell
git clone https://github.com/khaphanspace/gonhanh.org.git
cd gonhanh.org/platforms/windows
cargo build --release
```

### Chạy tests

```powershell
cargo test
```

---

## Theo dõi tiến độ

- [Roadmap](../README.md)
- [GitHub Issues](https://github.com/khaphanspace/gonhanh.org/issues)
- [Releases](https://github.com/khaphanspace/gonhanh.org/releases)

Hãy theo dõi để cập nhật khi phiên bản Windows chính thức phát hành!
