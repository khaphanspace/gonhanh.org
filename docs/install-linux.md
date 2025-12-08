# Hướng dẫn cài đặt Gõ Nhanh trên Linux

> **Lưu ý:** Phiên bản Linux hiện đang trong giai đoạn phát triển (Planned).

## Trạng thái hiện tại

Gõ Nhanh cho Linux dự kiến sẽ hỗ trợ **IBus** engine để tương thích tốt với GNOME, KDE và các môi trường Desktop khác (bao gồm cả Wayland).

## Lộ trình phát triển

Tôi đang tập trung hoàn thiện Core Engine và phiên bản macOS trước khi port sang Linux.

## Dành cho Developers

Core Engine viết bằng Rust có thể compile và chạy trên Linux. Bạn có thể tham khảo thư mục `core/` để build và test logic bộ gõ.

```bash
cd core
cargo test
```

Hãy theo dõi [Roadmap](../README.md) để cập nhật thông tin mới nhất.
