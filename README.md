<h1 align="center">
  <img src="assets/logo.png" alt="Gõ Nhanh Logo" width="128" height="128"><br>
  Gõ Nhanh
</h1>

<p align="center">
  <img src="https://img.shields.io/github/downloads/khaphanspace/gonhanh.org/total?label=Downloads" />
  <img src="https://img.shields.io/github/last-commit/khaphanspace/gonhanh.org" />
</p>
<p align="center">
  <img src="https://img.shields.io/badge/Platform-macOS-000000?logo=apple&logoColor=white" />
  <img src="https://img.shields.io/badge/Platform-Linux-FCC624?logo=linux&logoColor=black" />
  <img src="https://img.shields.io/badge/License-GPL--3.0-blue.svg" alt="License: GPL-3.0">
  <img src="https://github.com/khaphanspace/gonhanh.org/actions/workflows/ci.yml/badge.svg" alt="CI">
</p>

<p align="center">
  <strong>Bộ gõ tiếng Việt miễn phí, nhanh, ổn định cho macOS.</strong><br>
  Cài là dùng. Không quảng cáo. Không thu thập dữ liệu.
</p>

<p align="center">
  <img src="assets/screenshot.png" alt="Gõ Nhanh Light Mode" width="100%">
</p>

---

## 📥 Tải về & Cài đặt

| Nền tảng | Trạng thái | Tải xuống | Hướng dẫn |
|:--------:|:----------:|:---------:|:----------|
| **macOS** | ✅ Sẵn sàng | [📥 Tải GoNhanh.dmg](https://github.com/khaphanspace/gonhanh.org/releases/latest/download/GoNhanh.dmg) | [Xem hướng dẫn](docs/install-macos.md) |
| **Linux** | 🧪 Beta | — | [Xem hướng dẫn](docs/install-linux.md) |
| **Windows** | 🧪 Beta | — | [Xem hướng dẫn](docs/install-windows.md) |

## ✨ Tính năng

- ⌨️ **Telex & VNI** — Chọn kiểu gõ quen thuộc
- 🎯 **Đặt dấu đúng vị trí** — `hoà` hoặc `hòa` tùy bạn chọn
- 🔤 **Nhận diện tiếng Anh** — `Windows`, `Chrome` không bị thêm dấu
- ✂️ **Gõ tắt** — `vn` → `Việt Nam`, `ko` → `không`
- ⚡ **Nhanh & Nhẹ** — <1ms · ~5MB RAM
- 🌗 **Dark/Light** — Tự động theo hệ thống
- 🔌 **Tương thích mọi app** — Terminal, VS Code, Chrome, Word, Excel...
- 💻 **Đa nền tảng** — macOS, Linux, Windows (sắp ra mắt)

### 🛡️ Cam kết "Ba Không"

- 🚫 **Không thu phí** — Miễn phí mãi mãi, không bản Pro
- 🚫 **Không quảng cáo** — Không popup, không làm phiền
- 🚫 **Không theo dõi** — Offline 100%, mã nguồn mở

## 🆚 So sánh với bộ gõ khác

| Vấn đề thường gặp | Bộ gõ khác | Gõ Nhanh |
|:------------------|:----------:|:--------:|
| Dính chữ trên Chrome/Edge | ⚠️ Phải tắt autocomplete | ✅ Tự động fix |
| Lặp chữ trên Google Docs | ⚠️ Phải bật "Sửa lỗi" | ✅ Tự động fix |
| Nhảy chữ trên Terminal | ❌ Không hỗ trợ tốt | ✅ Hoạt động tốt |
| Gạch chân khó chịu (macOS) | ❌ Luôn hiển thị | ✅ Không gạch chân |
| Cấu hình phức tạp | ⚠️ 10+ tùy chọn | ✅ Cài là dùng |
| Gõ trong ô mật khẩu | ❌ Bị chặn | ✅ Hoạt động bình thường |

> 💡 **Khi nào dùng bộ gõ khác?** Nếu bạn cần chuyển đổi bảng mã cũ (VNI, TCVN3...), hãy dùng UniKey/EVKey/OpenKey.

Chi tiết: [Các lỗi thường gặp](docs/common-issues.md)

## ❤️‍🔥 Động lực

Tôi (**Kha Phan**) bắt đầu dự án này vì các bộ gõ hiện tại thường xuyên gặp lỗi khi tôi làm việc với **Claude Code**.

Từ nhu cầu giải quyết vấn đề cá nhân, Gõ Nhanh được phát triển thành một sản phẩm hoàn thiện dành tặng cộng đồng. Đây cũng là sự tiếp nối và kế thừa từ **UniKey**, **OpenKey** và **EVKey**.

Hy vọng Gõ Nhanh góp phần truyền cảm hứng cho cộng đồng mã nguồn mở tại Việt Nam.

---

## 🔧 Dành cho Developer

### Cách hoạt động

Engine dựa trên **ngữ âm học tiếng Việt** thay vì bảng tra cứu:

```
Âm tiết = [Phụ âm đầu] + [Âm đệm] + Nguyên âm chính + [Âm cuối] + Thanh điệu
          (b,c,d,g...)   (o,u)      (a,ă,â,e,ê...)    (c,m,n,p,t)  (sắc,huyền...)
```

Chi tiết: [docs/core-engine-algorithm.md](docs/core-engine-algorithm.md) | [docs/vietnamese-language-system.md](docs/vietnamese-language-system.md)

### Build & Test

```bash
# Setup (chạy 1 lần)
./scripts/setup.sh

# Development
make test      # Chạy tests
make format    # Format + lint
make build     # Build full app
make install   # Copy vào /Applications
```

### Nguyên tắc thiết kế

| Nguyên tắc | Chi tiết |
|------------|----------|
| **Anti-over-engineering** | Không abstraction layer thừa. Inline code khi chỉ dùng 1 chỗ |
| **Performance-first** | Target: <1ms latency, <10MB RAM. Không allocation trong hot path |
| **Zero dependency** | Rust core chỉ dùng `std`. Không crates ngoài |
| **Test-driven** | 200+ tests với test coverage 100%. Coverage cho edge cases tiếng Việt |
| **Validation-first** | Reject invalid input sớm. Validate trước khi transform |
| **Platform-agnostic core** | Core = pure Rust, no OS-specific code. UI layer riêng cho mỗi platform |

### Tài liệu kỹ thuật

| Tài liệu | Mô tả |
|----------|-------|
| [Kiến trúc hệ thống](docs/system-architecture.md) | FFI, luồng dữ liệu, app compatibility |
| [Validation Algorithm](docs/validation-algorithm.md) | 5 quy tắc kiểm tra âm tiết |
| [Hệ thống chữ viết tiếng Việt & Phương pháp gõ](docs/vietnamese-language-system.md) | Cơ sở lý thuyết |
| [Hướng dẫn phát triển](docs/development.md) | Build, test, contribute |

---

## ⭐ Star History

[![Star History Chart](https://api.star-history.com/svg?repos=khaphanspace/gonhanh.org&type=Timeline&legend=bottom-right)](https://www.star-history.com/#khaphanspace/gonhanh.org&type=Timeline&legend=bottom-right)

---

## 📄 License

Copyright © 2025 Gõ Nhanh Contributors. [GNU GPLv3](LICENSE).
