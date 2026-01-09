<h1 align="center">
  <img src="assets/logo.png" alt="Gõ Nhanh Logo" width="128" height="128"><br>
  Gõ Nhanh
</h1>

<p align="center">
  <img src="https://img.shields.io/github/downloads/khaphanspace/gonhanh.org/total?label=Downloads" />
  <img src="https://img.shields.io/github/contributors/khaphanspace/gonhanh.org" />
  <img src="https://img.shields.io/github/last-commit/khaphanspace/gonhanh.org" />
</p>
<p align="center">
  <img src="https://img.shields.io/badge/Platform-macOS-000000?logo=apple&logoColor=white" />
  <img src="https://img.shields.io/badge/Platform-Linux-FCC624?logo=linux&logoColor=black" />
  <img src="https://img.shields.io/badge/License-BSD--3--Clause-blue.svg" alt="License: BSD-3-Clause">
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

### 🍺 Homebrew (macOS - Khuyến nghị)

```bash
brew install --cask gonhanh
```

> **Cập nhật:** App tự động kiểm tra phiên bản mới mỗi 24h. Hoặc dùng `brew upgrade --cask gonhanh`.

### 📦 Tải thủ công

| Nền tảng | Trạng thái | Tải xuống | Hướng dẫn |
|:--------:|:----------:|:---------:|:----------|
| **macOS** | ✅ Sẵn sàng | [📥 Tải GoNhanh.dmg](https://github.com/khaphanspace/gonhanh.org/releases/latest/download/GoNhanh.dmg) | [Xem hướng dẫn](docs/install-macos.md) |
| **Linux** | 🧪 Beta | — | [Xem hướng dẫn](docs/install-linux.md) |
| **Windows** | 🧪 Beta | — | [Xem hướng dẫn](docs/install-windows.md) |

<details>
<summary><strong>⚠️ macOS: App không mở được?</strong></summary>

```bash
# Chạy lệnh này trong Terminal sau khi kéo app vào Applications
xattr -cr /Applications/GoNhanh.app
```

Sau đó: **System Settings → Privacy & Security → Accessibility → Bật GoNhanh**

</details>

## ✨ Tính năng

### 🔥 Highlight

- 🔍 **Fix lỗi Chrome/Spotlight/Arc/Claude Code/JetBrains** - Tự động sửa dính chữ trong address bar, thanh tìm kiếm, Arc history
- 🔤 **Auto-restore tiếng Anh** — Gõ `text` `expect` `user` `push` `sort` → tự khôi phục khi nhấn space. [Xem chi tiết ↓](#-auto-restore-tiếng-anh)
- ⎋ **Gõ ESC tự khôi phục** — Gõ `user` → `úẻ` → nhấn **ESC** → `user`. Không cần tắt bộ gõ khi gõ tiếng Anh!
- 🧠 **Chuyển chế độ thông minh** — Tự nhớ ON/OFF cho từng app. Code trên VS Code (tắt) → Chat trên Slack (bật) → tự chuyển
- 🔄 **Tự động theo input source** — Dùng tiếng Nhật, Hàn, Trung... → Gõ Nhanh tự tắt. Chuyển về tiếng Anh → tự bật lại
- ⚡ **Siêu nhanh** — <1ms latency · ~5MB RAM. Hỗ trợ đa nền tảng trên cùng một engine


<p align="center">
  <video src="https://github.com/user-attachments/assets/957ec6c6-d6df-4ca9-9161-0a1bb6cf98ce" width="100%"></video>
</p>

### 📋 Đầy đủ

- ⌨️ **Telex & VNI** — Chọn kiểu gõ quen thuộc
- 🎯 **Đặt dấu chuẩn** — Tự động theo [quy tắc mới](https://vi.wikipedia.org/wiki/Quy_t%E1%BA%AFc_%C4%91%E1%BA%B7t_d%E1%BA%A5u_thanh_c%E1%BB%A7a_ch%E1%BB%AF_Qu%E1%BB%91c_ng%E1%BB%AF): `hoà`, `khoẻ`, `thuỷ`
- 🔠 **Tự viết hoa đầu câu** — Gõ `ok.` Space `b` → `B` hoa. Hỗ trợ `.` `!` `?` và Enter
- ✂️ **Gõ tắt** — `vn` → `Việt Nam`, `ko` → `không`
- 🔌 **Mọi app** — VS Code, Zed, Chrome, Notion, Terminal, Ghostty...
- 🌗 **Dark/Light** — Theo hệ thống
- 💻 **Đa nền tảng** — macOS, Linux, Windows (beta)

### 🛡️ Cam kết "Ba Không"

- 🚫 **Không thu phí** — Miễn phí mãi mãi, không bản Pro
- 🚫 **Không quảng cáo** — Không popup, không làm phiền
- 🚫 **Không theo dõi** — Offline 100%, mã nguồn mở

---

## 🔤 Auto-restore tiếng Anh

Khi gõ tiếng Anh bằng Telex, một số chữ cái bị nhận nhầm thành modifier tiếng Việt:
- `s` → sắc, `f` → huyền, `r` → hỏi, `x` → ngã, `j` → nặng
- `w` → dấu móc (ư, ơ)

**Gõ Nhanh tự động khôi phục** khi nhấn **Space** nếu phát hiện pattern tiếng Anh.

### ✅ Các pattern được nhận diện

| Pattern | Ví dụ | Giải thích |
|:--------|:------|:-----------|
| **Modifier + phụ âm** | `text` `next` `test` `expect` `express` | x/s theo sau bởi phụ âm (t, p, c...) |
| **EI + modifier** | `their` `weird` | Cặp nguyên âm "ei" + r/s/f... |
| **P + AI + modifier** | `pair` | P đầu (hiếm trong tiếng Việt) + ai |
| **Nguyên âm + modifier + nguyên âm** | `use` `user` | Không có phụ âm đầu |
| **W đầu + phụ âm** | `window` `water` `write` `what` | W không phải phụ âm đầu tiếng Việt |
| **W + nguyên + W** | `wow` | Pattern "wow" đặc trưng tiếng Anh |
| **F đầu** | `file` `fix` `function` `firebase` | F không tồn tại trong tiếng Việt |

### 📝 So sánh: macOS Telex vs Gõ Nhanh

| Gõ | macOS Telex | Gõ Nhanh |
|:---|:------------|:---------|
| `text ` | `têt ` ❌ | `text ` ✅ |
| `expect ` | `ễpct ` ❌ | `expect ` ✅ |
| `perfect ` | `pềct ` ❌ | `perfect ` ✅ |
| `window ` | `ưindow ` ❌ | `window ` ✅ |
| `with ` | `ưith ` ❌ | `with ` ✅ |
| `tesla ` | `téla ` ❌ | `tesla ` ✅ |
| `luxury ` | `lủuy ` ❌ | `luxury ` ✅ |
| `case ` | `cáe ` ❌ | `case ` ✅ |
| `wow ` | `ươ ` ❌ | `wow ` ✅ |
| `file ` | `file ` ✅ | `file ` ✅ |
| `fix ` | `fix ` ✅ | `fix ` ✅ |

### ⚠️ Giới hạn

Một số từ tiếng Anh tạo ra cấu trúc **hợp lệ tiếng Việt** nên không thể tự nhận diện:

| Từ Anh | Kết quả Telex | Lý do |
|:-------|:--------------|:------|
| `mix` | `mĩ` | M + ĩ hợp lệ |
| `box` | `bõ` | B + õ hợp lệ |
| `six` | `sĩ` | S + ĩ hợp lệ |

**Giải pháp:**
- Nhấn **ESC** sau khi gõ để khôi phục
- Tắt bộ gõ tạm thời (⌘+Space hoặc click menu bar)

---

## ❤️‍🔥 Động lực

Tôi (**Kha Phan**) bắt đầu dự án này vì các bộ gõ hiện tại thường xuyên gặp lỗi khi tôi làm việc với **Claude Code**.

Từ nhu cầu giải quyết vấn đề cá nhân, Gõ Nhanh được phát triển thành một sản phẩm hoàn thiện dành tặng cộng đồng. Đây cũng là sự tiếp nối và kế thừa từ **UniKey**, **OpenKey** và **EVKey**.

Hy vọng Gõ Nhanh góp phần truyền cảm hứng cho cộng đồng mã nguồn mở tại Việt Nam.

Nếu các bạn thấy phần mềm hữu ích có thể [ủng hộ tôi tại đây](https://github.com/sponsors/khaphanspace).

---

## 🔧 Dành cho Developer

### Tech Stack

| Layer | Công nghệ |
|-------|-----------|
| **Core Engine** | Rust 2021 (pure `std`, zero runtime deps) |
| **macOS** | SwiftUI + CGEventTap + C FFI |
| **Windows** | WPF/.NET 8 + SetWindowsHookEx + P/Invoke |
| **Linux** | Fcitx5 + C++ + C FFI |
| **Testing** | rstest + serial_test (600+ tests) |
| **CI/CD** | GitHub Actions + auto-versioning + AI-generated release notes |
| **Dev Tools** | [Claude Code](https://claude.ai/code) with [ClaudeKit](https://claudekit.cc/) |

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
| **Test-driven** | 600+ tests với coverage 100%. Bao gồm edge cases tiếng Việt và auto-restore tiếng Anh |
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

Copyright © 2025 Gõ Nhanh Contributors. [BSD-3-Clause](LICENSE).
