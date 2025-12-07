# GoNhanh ⚡

[![CI](https://github.com/khaphanspace/gonhanh.org/actions/workflows/ci.yml/badge.svg)](https://github.com/khaphanspace/gonhanh.org/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)

**Bộ gõ tiếng Việt thế hệ mới** — nhanh, nhẹ, native.

## Tại sao cần GoNhanh?

Các bộ gõ tiếng Việt hiện tại đã phục vụ cộng đồng rất tốt:

- [**UniKey**](https://www.unikey.org/) — Bộ gõ huyền thoại, tiêu chuẩn de facto từ 2000
- [**EVKey**](https://evkeyvn.com/) — Kế thừa UniKey với nhiều cải tiến
- [**OpenKey**](https://github.com/tuyenvm/OpenKey) — Open source, hỗ trợ macOS/Windows/Linux

Tuy nhiên, chúng có những hạn chế chung:

| Vấn đề | Mô tả |
|--------|-------|
| **Kiến trúc cũ** | C/C++ từ thập niên 2000, khó bảo trì |
| **Không native** | Dùng chung UI framework (Qt) cho mọi platform |
| **Feature creep** | Tích hợp nhiều tính năng ít dùng (chuyển mã, macro, spelling...) |
| **Lookup-based** | Dựa trên bảng tra cứu, không theo quy tắc ngữ âm |

GoNhanh không thay thế các bộ gõ trên, mà là một **lựa chọn khác** cho những ai cần sự đơn giản và hiệu năng.

## Triết lý GoNhanh

### 1. Phonology-first

Engine được xây dựng dựa trên **ngữ âm học tiếng Việt**, không phải bảng tra cứu cứng:

- Phân loại nguyên âm theo vai trò: âm chính, âm đệm, bán nguyên âm
- Thuật toán đặt dấu thanh theo quy tắc ngữ âm (không hardcode)
- Hỗ trợ cả hai trường phái: kiểu cũ (`oà`) và kiểu mới (`òa`)

> Xem chi tiết: [docs/vietnamese-language-system.md](docs/vietnamese-language-system.md)

### 2. Native-first

- **macOS**: SwiftUI, tích hợp menu bar như app native
- **Windows**: WPF/WinUI (planned)
- Mỗi platform có UI riêng, trải nghiệm tự nhiên

### 3. Rust Core

- Memory-safe, không buffer overflow
- Cross-platform: cùng engine cho mọi OS
- FFI đơn giản, dễ tích hợp

### 4. Minimal

- Chỉ làm một việc: gõ tiếng Việt
- Không chuyển mã, không clipboard hook, không macro
- Binary nhỏ (~3MB), RAM thấp (~25MB)

## So sánh

| | GoNhanh | OpenKey | UniKey | EVKey |
|---|:---:|:---:|:---:|:---:|
| **Engine** | Rust | C++ | C++ | C++ |
| **macOS UI** | SwiftUI | Obj-C | Qt | Qt |
| **Platforms** | macOS, Windows* | macOS, Windows, Linux | Windows | Windows, macOS |
| **Memory** | ~25 MB | ~30 MB | ~50 MB | ~40 MB |
| **Open source** | ✅ Full | ✅ Full | ⚠️ Partial | ✅ Full |
| **Kiến trúc** | Phonology | Lookup | Lookup | Lookup |
| **Chuyển mã** | ❌ | ✅ | ✅ | ✅ |
| **Macro** | ❌ | ✅ | ✅ | ✅ |

*\* Windows: planned*

## Features

| | |
|---|---|
| ⌨️ **Input methods** | Telex, VNI |
| 🎯 **Tone placement** | Algorithmic (kiểu cũ/mới) |
| 🔤 **Full Unicode** | 89 ký tự có dấu |
| ⚡ **Performance** | <1ms latency |
| 🔒 **Privacy** | Offline, no telemetry |

## Installation

### macOS

```bash
# Build from source
git clone https://github.com/khaphanspace/gonhanh.org
cd gonhanh.org
make build

# Install
cp -r platforms/macos/build/Release/GoNhanh.app /Applications/
```

### Homebrew (coming soon)

```bash
brew install gonhanh
```

## Usage

1. Mở GoNhanh từ Applications
2. Click icon trên menu bar để bật/tắt
3. Right-click để mở Settings:
   - Chọn kiểu gõ (Telex/VNI)
   - Chọn kiểu đặt dấu (cũ/mới)

**Lần đầu chạy**: Cấp quyền Accessibility trong System Settings → Privacy & Security → Accessibility

## Development

```bash
make test    # Run tests
make build   # Build everything
make clean   # Clean artifacts
```

Xem thêm: [docs/development.md](docs/development.md)

## Architecture

```
┌─────────────────────────────────┐
│     Platform UI (Swift/WPF)    │
└───────────────┬─────────────────┘
                │ FFI
┌───────────────▼─────────────────┐
│         Rust Core Engine        │
│  • Buffer management            │
│  • Phonology-based rules        │
│  • Unicode output               │
└─────────────────────────────────┘
```

Xem thêm: [docs/architecture.md](docs/architecture.md)

## Documentation

| Document | Nội dung |
|----------|----------|
| [vietnamese-language-system.md](docs/vietnamese-language-system.md) | Hệ thống ngữ âm tiếng Việt, quy tắc đặt dấu |
| [architecture.md](docs/architecture.md) | Kiến trúc hệ thống, FFI interface |
| [development.md](docs/development.md) | Hướng dẫn phát triển |

## Contributing

Contributions welcome! Xem [CONTRIBUTING.md](CONTRIBUTING.md)

## License

[GPL-3.0-or-later](LICENSE) — Tự do sử dụng, sửa đổi, phân phối với cùng license.
