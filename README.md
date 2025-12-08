<p align="center">
  <img src="assets/logo.png" alt="Gõ Nhanh Logo" width="128" height="128">
</p>

<h1 align="center">Gõ Nhanh</h1>

<p align="center">
  <img src="https://img.shields.io/github/downloads/khaphanspace/gonhanh.org/total?label=Downloads" />
  <img src="https://img.shields.io/github/v/release/khaphanspace/gonhanh.org?label=Latest%20Release" />
  <img src="https://img.shields.io/github/last-commit/khaphanspace/gonhanh.org" />
</p>
<p align="center">
  <img src="https://img.shields.io/badge/Core-Rust-000000?logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/UI-SwiftUI-F05138?logo=swift&logoColor=white" />
  <img src="https://img.shields.io/badge/Platform-macOS-000000?logo=apple&logoColor=white" />
  <img src="https://img.shields.io/badge/License-GPL--3.0-blue.svg" alt="License: GPL-3.0">
  <img src="https://github.com/khaphanspace/gonhanh.org/actions/workflows/ci.yml/badge.svg" alt="CI">
</p>

<p align="center"><strong>Gõ Nhanh</strong> - Bộ gõ tiếng Việt hiện đại, hiệu suất cao dành cho macOS. <br>
Kết hợp sức mạnh của <strong>Rust</strong> (Core Engine) và <strong>SwiftUI</strong> (Native UI) để mang lại trải nghiệm gõ phím mượt mà, ổn định và bảo mật.</p>

## 📥 Cài đặt

| Nền tảng    | Trạng thái  |                                                   Tải xuống                                                   | Hướng dẫn                               |
| :---------- | :---------: | :-----------------------------------------------------------------------------------------------------------: | :-------------------------------------- |
| **macOS**   | ✅ Sẵn sàng | [📥 **Tải về GoNhanh.dmg**](https://github.com/khaphanspace/gonhanh.org/releases/latest/download/GoNhanh.dmg) | [Xem hướng dẫn](docs/install-macos.md)  |
| **Windows** | 🗓️ Planned  |                                                       —                                                       | [Xem lộ trình](docs/install-windows.md) |
| **Linux**   | 🗓️ Planned  |                                                       —                                                       | [Xem lộ trình](docs/install-linux.md)   |

## 🚀 Tính năng nổi bật

Gõ Nhanh hướng tới sự **Chuẩn hóa - Hiệu suất - Tiện dụng**:

- **Core Engine (Rust)**: Xử lý dấu thông minh, độ trễ < 1ms, bộ nhớ cực thấp (~5MB).
- **Native UI (SwiftUI)**: Giao diện tối giản trên Menu Bar, hỗ trợ Light/Dark mode.
- **Hook cấp thấp**: Tương thích tốt với Terminal, IDE (VS Code, IntelliJ) và các ứng dụng đồ họa.
- **Smart Defaults**: Cài là dùng, không cần cấu hình phức tạp.

### Tính năng chi tiết

#### 🧠 Core Engine

- **Kiểu gõ**: Hỗ trợ đầy đủ **Telex** và **VNI**.
- **Xử lý dấu thông minh**:
  - Tự động đặt dấu đúng vị trí ngữ âm (Smart Tone Placement).
  - Tùy chọn kiểu bỏ dấu: Cổ điển (`oà`) hoặc Hiện đại (`òa`).
  - Kiểm tra tính hợp lệ của âm tiết (Phonology Check) để tránh gõ sai.
- **Hiệu suất tối thượng**: Độ trễ xử lý < 1ms, bộ nhớ sử dụng cực thấp (~5MB), không gây nóng máy.

#### 🖥️ Native App (macOS)

- **Tối giản**: Ứng dụng chạy trên Menu Bar, không chiếm Dock, không làm phiền.
- **Giao diện hiện đại**: Viết bằng **SwiftUI**, tự động thích ứng Light/Dark mode.
- **Tiện ích**:
  - Phím tắt chuyển đổi Anh/Việt toàn cục.
  - Tự động khởi động cùng hệ thống.
  - Cơ chế Hook bàn phím cấp thấp (CGEventTap) đảm bảo độ ổn định cao trên mọi ứng dụng (Terminal, Claude, IDE...).

### Cam kết "Ba Không"

- 🚫 **Không thu phí**: Miễn phí trọn đời, không có bản "Premium".
- 🚫 **Không rác**: Không quảng cáo, không popup, không tính năng thừa thãi.
- 🚫 **Không theo dõi**: Offline 100%, không thu thập dữ liệu, mã nguồn minh bạch.

## Động lực

Tôi (**Kha Phan**) bắt đầu dự án này vì các bộ gõ hiện tại thường xuyên gặp lỗi khi tôi làm việc với **Claude Code**.

Từ nhu cầu giải quyết vấn đề cá nhân, Gõ Nhanh được phát triển thành một sản phẩm hoàn thiện dành tặng cộng đồng. Đây cũng là sự tiếp nối và kế thừa từ **UniKey**, **OpenKey** và **EVKey**.

## So sánh

|                |      Gõ Nhanh      |        EVKey        |    OpenKey     |    GoTiengViet    |     UniKey     |
| :------------- | :----------------: | :-----------------: | :------------: | :---------------: | :------------: |
| **Trạng thái** | 🟢 **Phát triển**  | 🔴 Ngừng phát triển |   🟡 Bảo trì   | 🟡 Ngừng cập nhật |   🟢 Ổn định   |
| macOS          |         ✅         |         ✅          |       ✅       |        ✅         |       ❌       |
| Windows        |     🗓️ Planned     |         ✅          |       ✅       |        ✅         |       ✅       |
| Linux          |     🗓️ Planned     |         ❌          |       ✅       |        ❌         |  ✅ (Engine)   |
| **Mã nguồn**   | ✅ **Open Source** |   ✅ Open Source    | ✅ Open Source |     🚫 Closed     | ✅ Core Engine |
| Công nghệ      | **Rust + Native**  |      C++ + Qt       |    C++ + Qt    |    Obj-C / C++    |      C++       |
| Bảng mã        |    **Unicode**     |     Đa bảng mã      |   Đa bảng mã   |    Đa bảng mã     |   Đa bảng mã   |
| Chi phí        |    ✅ Miễn phí     |     ✅ Miễn phí     |  ✅ Miễn phí   |   Miễn phí/Pro    |  ✅ Miễn phí   |
| Năm ra mắt     |        2025        |        2018         |      2019      |       2008        |      1999      |

Nếu cần chuyển mã hay dùng bảng mã cũ, dùng UniKey/EVKey/OpenKey.

### Tại sao chọn Gõ Nhanh?

| Vấn đề                                     |    Bộ gõ khác / Mặc định     |        Gõ Nhanh          |
| :----------------------------------------- | :--------------------------: | :----------------------: |
| **Dính chữ Chrome/Edge** (`aa` → `aâ`)     | ⚠️ Tắt autocomplete thủ công |      ✅ Tự động fix      |
| **Lặp chữ Google Docs** (`được` → `đđược`) |  ⚠️ Bật "Sửa lỗi" thủ công   |      ✅ Tự động fix      |
| **Mất dấu Excel** (`trường` → `trương`)    |       ⚠️ Không ổn định       |      ✅ Tự động fix      |
| **Nhảy chữ Terminal/CLI**                  |     ❌ Không hỗ trợ tốt      |    ✅ Smart detection    |
| **Hộp đen che chữ (macOS)**                |    ❌ Che mất nội dung gõ    |     ✅ Edit-in-place     |
| **Gạch chân khó chịu (macOS)**             |       ❌ Luôn hiển thị       |    ✅ Không gạch chân    |
| **Lỗi lặp chữ Discord (Windows)**          |     ❌ Thường xuyên gặp      |  ✅ Fix triệt để (plan)  |
| **Xung đột phím tắt IDE**                  |     ⚠️ Cần map lại phím      |    ✅ Hook thông minh    |
| **Chọn bảng mã**                           |  ⚠️ Nhiều lựa chọn gây rối   |   ✅ Mặc định Unicode    |
| **Chọn kiểu gõ**                           |    ⚠️ Telex/VNI/VIQR/...     |    ✅ Telex hoặc VNI     |
| **Cấu hình phức tạp**                      |       ⚠️ 10+ tùy chọn        |      ✅ Cài là dùng      |
| **Chạy quyền Admin (Windows)**             |     ⚠️ Cần bật thủ công      |   🗓️ Tự động (planned)   |
| **Quyền Accessibility (macOS)**            |    ⚠️ Hướng dẫn phức tạp     |    ✅ Prompt tự động     |
| **Gõ trong Password field**                |  ❌ Bị chặn (Secure Input)   | ✅ Hoạt động bình thường |
| **Khởi động cùng hệ thống**                |     ⚠️ Cấu hình thủ công     |     ✅ Mặc định bật      |
| **Cập nhật phiên bản**                     |      ⚠️ Tải về thủ công      | 🗓️ Auto-update (planned) |

Chi tiết các lỗi thường gặp ở các bộ gõ khác: [docs/common-issues.md](docs/common-issues.md)

## Cách hoạt động

Engine dựa trên ngữ âm học tiếng Việt thay vì bảng tra cứu:

```
Âm tiết = [Phụ âm đầu] + [Âm đệm] + Nguyên âm chính + [Âm cuối] + Thanh điệu
```

Thuật toán đặt dấu theo quy tắc ngữ âm. Hỗ trợ cả kiểu cũ (`hoà`) và kiểu mới (`hòa`).

Chi tiết: [docs/vietnamese-language-system.md](docs/vietnamese-language-system.md)

## Kiến trúc

```
┌────────────────────────────────────────────────────────┐
│               Platform UI Layer (Native)               │
│    ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │
│    │    macOS     │  │   Windows    │  │   Linux    │  │
│    │   SwiftUI    │  │  WPF (Plan)  │  │ IBus/Fcitx │  │
│    └──────┬───────┘  └──────┬───────┘  └──────┬─────┘  │
└───────────┼─────────────────┼─────────────────┼────────┘
            │           FFI Bridge (C ABI)      │
┌───────────▼─────────────────▼─────────────────▼────────┐
│                 Rust Core Library                      │
│      (Engine, Logic, State, Phonology Rules)           │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Input Processing Pipeline (Telex / VNI)         │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

- **Platform UI**: Native code (Swift/C#/C++) sử dụng **Low-level Hook (CGEventTap)** để bắt sự kiện phím tầng hệ thống, đảm bảo độ trễ < 1ms và tương thích mọi ứng dụng.
- **Rust Core**: Logic xử lý tiếng Việt dựa trên **thuật toán ngữ âm** (Phonology Engine) thay vì tra bảng, giúp xử lý dấu thông minh và chỉ tốn ~5MB bộ nhớ.
- **FFI**: Giao tiếp giữa UI và Core thông qua C ABI, đảm bảo tính tương thích và tốc độ tối đa.

## Tài liệu

| Tài liệu                                                                 | Mô tả                                                          |
| ------------------------------------------------------------------------ | -------------------------------------------------------------- |
| [Hệ thống chữ viết & Phương pháp gõ](docs/vietnamese-language-system.md) | Cơ sở lý thuyết ngữ âm và quy tắc đặt dấu.                     |
| [Architecture](docs/architecture.md)                                     | Kiến trúc hệ thống, FFI, và luồng dữ liệu.                     |
| [Development Guide](docs/development.md)                                 | Hướng dẫn build, test, và đóng góp mã nguồn.                   |
| [Các lỗi thường gặp](docs/common-issues.md)                              | Tổng hợp lỗi bộ gõ (Chrome, Word, Terminal) và cách khắc phục. |

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=khaphanspace/gonhanh.org&type=Timeline&legend=bottom-right)](https://www.star-history.com/#khaphanspace/gonhanh.org&type=Timeline&legend=bottom-right)

## License

Copyright © 2025 Gõ Nhanh Contributors. [GNU GPLv3](LICENSE).
