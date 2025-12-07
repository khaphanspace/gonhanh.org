# GoNhanh (Gõ Nhanh)

[![CI](https://github.com/khaphanspace/gonhanh.org/actions/workflows/ci.yml/badge.svg)](https://github.com/khaphanspace/gonhanh.org/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)

**GoNhanh** (đọc là **Gõ Nhanh**) - Bộ gõ tiếng Việt hiệu suất cao, hiện đại và đa nền tảng.

Sức mạnh của **Rust** (Core Engine) kết hợp với **Native UI** (SwiftUI) mang lại trải nghiệm gõ phím mượt mà, ổn định và bảo mật tuyệt đối.

## 🚀 Về GoNhanh

GoNhanh được xây dựng với mục tiêu trở thành bộ gõ tiếng Việt **hoàn thiện nhất**, dựa trên các trụ cột: **Chuẩn hóa - Hiện đại - Tương lai**.

- **Chuẩn hóa**: Tuyệt đối tuân thủ quy tắc chính tả chữ Quốc ngữ (theo chuẩn BGD&ĐT).
- **Hiệu suất & Công nghệ**: Core engine viết bằng Rust kết hợp Native UI giúp xử lý tức thì, bỏ qua gánh nặng lịch sử (legacy code).
- **Đa nền tảng & Tương lai**: Kiến trúc Hybrid sẵn sàng cho macOS, Windows, Linux và các hệ thống thế hệ mới (Wayland).
- **Trải nghiệm mượt mà**: Giao diện thích ứng (Light/Dark mode), hoạt ảnh mượt mà, phản hồi lập tức.
- **Cài là dùng**: Cấu hình tối ưu sẵn (Smart Defaults), không cần thiết lập phức tạp.

### Tính năng nổi bật

- ✅ **Đa kiểu gõ**: Hỗ trợ đầy đủ Telex và VNI.
- ✅ **Xử lý thông minh**: Tự động đặt dấu đúng vị trí và kiểm tra ngữ âm.
- ✅ **Phím tắt**: Chuyển đổi Anh/Việt tức thì, có thể tùy biến.
- ✅ **Tối giản**: Chạy ngầm dưới khay hệ thống, không chiếm không gian làm việc.

## Động lực

Tiếp nối di sản của **VietKey**, **UniKey**, **OpenKey** và **EVKey**, tôi (**Kha Phan**) phát triển GoNhanh để giải quyết nhu cầu về sự ổn định tuyệt đối trên các công cụ lập trình hiện đại.

Dự án **phi lợi nhuận** với cam kết: **Mã nguồn mở - Miễn phí - Của cộng đồng**.

## Ba Không

- 🚫 **Không thu phí**: Miễn phí trọn đời, không có bản "Premium".
- 🚫 **Không rác**: Không quảng cáo, không popup, không tính năng thừa thãi.
- 🚫 **Không theo dõi**: Offline 100%, không thu thập dữ liệu, mã nguồn minh bạch.

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

| Tài liệu                                      | Mô tả                                        | Link                                           |
| --------------------------------------------- | -------------------------------------------- | ---------------------------------------------- |
| Hệ thống chữ viết tiếng Việt & Phương pháp gõ | Cơ sở lý thuyết ngữ âm và quy tắc đặt dấu.   | [Tài liệu](docs/vietnamese-language-system.md) |
| Architecture                                  | Kiến trúc hệ thống, FFI, và luồng dữ liệu.   | [Tài liệu](docs/architecture.md)               |
| Development Guide                             | Hướng dẫn build, test, và đóng góp mã nguồn. | [Tài liệu](docs/development.md)                |

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=khaphanspace/gonhanh.org&type=Date)](https://star-history.com/#khaphanspace/gonhanh.org&Date)

## License

[GPL-3.0-or-later](LICENSE)
