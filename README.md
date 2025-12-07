# GoNhanh (Gõ Nhanh)

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![CI](https://github.com/khaphanspace/gonhanh.org/actions/workflows/ci.yml/badge.svg)](https://github.com/khaphanspace/gonhanh.org/actions/workflows/ci.yml)

**GoNhanh** (đọc là **Gõ Nhanh**) - Bộ gõ tiếng Việt hiệu suất cao, hiện đại và đa nền tảng.

Sức mạnh của **Rust** (Core Engine) kết hợp với **Native UI** (SwiftUI) mang lại trải nghiệm gõ phím mượt mà, ổn định và bảo mật tuyệt đối.

## 🚀 Về GoNhanh

GoNhanh được xây dựng với mục tiêu trở thành bộ gõ tiếng Việt **hoàn thiện nhất**, dựa trên các trụ cột: **Chuẩn hóa - Hiện đại - Tương lai**.

- **Chuẩn hóa**: Tuyệt đối tuân thủ quy tắc chính tả chữ Quốc ngữ (theo chuẩn BGD&ĐT).
- **Hiệu suất & Công nghệ**: Core engine viết bằng Rust kết hợp Native UI giúp xử lý tức thì, bỏ qua gánh nặng lịch sử (legacy code).
- **Đa nền tảng & Tương lai**: Kiến trúc Hybrid sẵn sàng cho macOS, Windows, Linux và các hệ thống thế hệ mới (Wayland).
- **Trải nghiệm mượt mà**: Giao diện thích ứng (Light/Dark mode), hoạt ảnh mượt mà, phản hồi lập tức.
- **Cài là dùng**: Cấu hình tối ưu sẵn (Smart Defaults), không cần thiết lập phức tạp.

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

Từ nhu cầu giải quyết vấn đề cá nhân, GoNhanh được phát triển thành một sản phẩm hoàn thiện dành tặng cộng đồng. Đây cũng là sự tiếp nối di sản của các tượng đài **VietKey**, **UniKey**, **OpenKey** và **EVKey**.

## So sánh

|                |      GoNhanh       |        EVKey        |    OpenKey     |    GoTiengViet    |     UniKey     |
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

_\* Windows: đang trong lộ trình phát triển (Roadmap)._

Nếu cần chuyển mã hay dùng bảng mã cũ, dùng UniKey/EVKey/OpenKey.

### Tại sao chọn GoNhanh?

| Vấn đề                                     |          Bộ gõ khác          |         GoNhanh          |
| :----------------------------------------- | :--------------------------: | :----------------------: |
| **Dính chữ Chrome/Edge** (`aa` → `aâ`)     | ⚠️ Tắt autocomplete thủ công |      ✅ Tự động fix      |
| **Lặp chữ Google Docs** (`được` → `đđược`) |  ⚠️ Bật "Sửa lỗi" thủ công   |      ✅ Tự động fix      |
| **Mất dấu Excel** (`trường` → `trương`)    |       ⚠️ Không ổn định       |      ✅ Tự động fix      |
| **Nhảy chữ Terminal/CLI**                  |     ❌ Không hỗ trợ tốt      |    ✅ Smart detection    |
| **Xung đột bộ gõ macOS**                   |      ⚠️ Config thủ công      |     ✅ Hook cấp thấp     |
| **Chọn bảng mã**                           |  ⚠️ Nhiều lựa chọn gây rối   |   ✅ Mặc định Unicode    |
| **Chọn kiểu gõ**                           |    ⚠️ Telex/VNI/VIQR/...     |    ✅ Telex hoặc VNI     |
| **Cấu hình phức tạp**                      |       ⚠️ 10+ tùy chọn        |      ✅ Cài là dùng      |
| **Chạy quyền Admin (Windows)**             |     ⚠️ Cần bật thủ công      |   ✅ Tự động (planned)   |
| **Quyền Accessibility (macOS)**            |    ⚠️ Hướng dẫn phức tạp     |    ✅ Prompt tự động     |
| **Gõ trong Password field**                |  ❌ Bị chặn (Secure Input)   | ✅ Hoạt động bình thường |
| **Khởi động cùng hệ thống**                |     ⚠️ Cấu hình thủ công     |     ✅ Mặc định bật      |
| **Cập nhật phiên bản**                     |      ⚠️ Tải về thủ công      | ✅ Auto-update (planned) |

**Triết lý GoNhanh: Cài là dùng, không cần cấu hình.**

Chi tiết các lỗi và cách khắc phục: [docs/common-issues.md](docs/common-issues.md)

## Cách hoạt động

Engine dựa trên ngữ âm học tiếng Việt thay vì bảng tra cứu:

```
Âm tiết = [Phụ âm đầu] + [Âm đệm] + Nguyên âm chính + [Âm cuối] + Thanh điệu
```

Thuật toán đặt dấu theo quy tắc ngữ âm. Hỗ trợ cả kiểu cũ (`hoà`) và kiểu mới (`hòa`).

Chi tiết: [docs/vietnamese-language-system.md](docs/vietnamese-language-system.md)

## Kiến trúc

```
┌─────────────────────────────────────┐
│         Platform UI Layer           │
│  ┌──────────┐      ┌──────────┐    │
│  │  macOS   │      │ Windows  │    │
│  │ SwiftUI  │      │   WPF    │    │
│  └─────┬────┘      └────┬─────┘    │
└────────┼────────────────┼──────────┘
         │    FFI (C ABI) │
┌────────▼────────────────▼──────────┐
│         Rust Core Library          │
│  ┌─────────────────────────────┐   │
│  │  Engine (Telex/VNI)         │   │
│  │  - Buffer management        │   │
│  │  - Phonology-based rules    │   │
│  │  - Unicode output           │   │
│  └─────────────────────────────┘   │
└────────────────────────────────────┘
```

- macOS: SwiftUI (done)
- Windows: WPF (planned)
- Linux: IBus/Wayland (planned)

## Tài liệu

| Tài liệu                                      | Mô tả                                                          | Link                                           |
| --------------------------------------------- | -------------------------------------------------------------- | ---------------------------------------------- |
| Hệ thống chữ viết tiếng Việt & Phương pháp gõ | Cơ sở lý thuyết ngữ âm và quy tắc đặt dấu.                     | [Tài liệu](docs/vietnamese-language-system.md) |
| Architecture                                  | Kiến trúc hệ thống, FFI, và luồng dữ liệu.                     | [Tài liệu](docs/architecture.md)               |
| Development Guide                             | Hướng dẫn build, test, và đóng góp mã nguồn.                   | [Tài liệu](docs/development.md)                |
| Các lỗi thường gặp                            | Tổng hợp lỗi bộ gõ (Chrome, Word, Terminal) và cách khắc phục. | [Tài liệu](docs/common-issues.md)              |

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=khaphanspace/gonhanh.org&type=Date)](https://star-history.com/#khaphanspace/gonhanh.org&Date)

## License

[GPL-3.0-or-later](LICENSE)
