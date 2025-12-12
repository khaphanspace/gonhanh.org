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
  <img src="https://img.shields.io/badge/License-GPL--3.0-blue.svg" alt="License: GPL-3.0">
  <img src="https://github.com/khaphanspace/gonhanh.org/actions/workflows/ci.yml/badge.svg" alt="CI">
</p>

<p align="center">
  <strong>Bộ gõ tiếng Việt miễn phí, nhanh, ổn định cho macOS.</strong><br>
  Cài là dùng. Không quảng cáo. Không thu thập dữ liệu.
</p>

---

## 📥 Tải về & Cài đặt

| Nền tảng | Trạng thái | Tải xuống | Hướng dẫn |
|:---------|:----------:|:---------:|:----------|
| **macOS** | ✅ Sẵn sàng | [📥 **Tải GoNhanh.dmg**](https://github.com/khaphanspace/gonhanh.org/releases/latest/download/GoNhanh.dmg) | [Xem hướng dẫn](docs/install-macos.md) |
| **Windows** | 🗓️ Sắp ra mắt | — | [Xem lộ trình](docs/install-windows.md) |
| **Linux** | 🗓️ Sắp ra mắt | — | [Xem lộ trình](docs/install-linux.md) |

## ✨ Tính năng chính

| Tính năng | Mô tả |
|-----------|-------|
| **Telex & VNI** | Chọn kiểu gõ quen thuộc của bạn |
| **Đặt dấu thông minh** | Tự động đặt dấu đúng vị trí theo quy tắc ngữ âm (`hoà` hoặc `hòa`) |
| **Tự nhận diện tiếng Anh** | Gõ `Claus`, `Windows`, `Developer` không bị thêm dấu — engine nhận diện từ không hợp lệ |
| **Gõ tắt** | Tạo phím tắt riêng (`vn` → `Việt Nam`, `ko` → `không`) |
| **Gõ linh hoạt** | `Dod` → `Đo`, `duocwj` → `được` — thứ tự phím không quan trọng |
| **Nhấn đúp để hủy** | `aa` → `â`, nhấn `a` thêm lần nữa → `aa` |
| **Nhanh & Nhẹ** | Độ trễ <1ms, chỉ dùng ~5MB RAM |
| **Hoạt động mọi nơi** | Terminal, VS Code, Chrome, Word, Excel... |
| **Ctrl+Space** | Chuyển đổi Anh/Việt nhanh chóng |
| **Tự khởi động** | Chạy cùng hệ thống, không cần bật thủ công |

### Cam kết "Ba Không"

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
make test      # Chạy 185+ tests
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
| **Test-driven** | 185+ tests. Mọi PR phải pass CI. Coverage cho edge cases tiếng Việt |
| **Validation-first** | Reject invalid input sớm. Validate trước khi transform |
| **Platform-agnostic core** | Core = pure Rust, no OS-specific code. UI layer riêng cho mỗi platform |

### Tài liệu kỹ thuật

| Tài liệu | Mô tả |
|----------|-------|
| [Kiến trúc hệ thống](docs/system-architecture.md) | FFI, luồng dữ liệu, app compatibility |
| [Validation](docs/validation-algorithm.md) | 5 quy tắc kiểm tra âm tiết |
| [Ngữ âm tiếng Việt](docs/vietnamese-language-system.md) | Cơ sở lý thuyết |
| [Hướng dẫn phát triển](docs/development.md) | Build, test, contribute |

---

## ⭐ Star History

[![Star History Chart](https://api.star-history.com/svg?repos=khaphanspace/gonhanh.org&type=Timeline&legend=bottom-right)](https://www.star-history.com/#khaphanspace/gonhanh.org&type=Timeline&legend=bottom-right)

---

## 📄 License

Copyright © 2025 Gõ Nhanh Contributors. [GNU GPLv3](LICENSE).
