# Hướng dẫn cài đặt Gõ Nhanh trên macOS

## Yêu cầu hệ thống

- macOS 13.0 (Ventura) trở lên.
- Chip Apple Silicon (M1/M2/M3) hoặc Intel.

## Các bước cài đặt

### 1. Tải ứng dụng

Tải xuống file `.dmg` phiên bản mới nhất tại: **[Tải Gõ Nhanh mới nhất](https://github.com/khaphanspace/gonhanh.org/releases/latest/download/GoNhanh.dmg)**

_(Nếu bạn muốn chọn phiên bản cũ hơn, hãy truy cập [Danh sách Releases](https://github.com/khaphanspace/gonhanh.org/releases))_

### 2. Cài đặt

1. Mở file `.dmg` vừa tải.
2. Kéo biểu tượng **Gõ Nhanh** vào thư mục **Applications**.

### 2.1. Mở ứng dụng lần đầu (Quan trọng)

Do Gõ Nhanh chưa được ký số bởi Apple, bạn cần chạy lệnh sau trong **Terminal** để cho phép ứng dụng khởi chạy (chỉ cần làm 1 lần):

```bash
xattr -cr /Applications/GoNhanh.app
```

Sau đó bạn có thể mở Gõ Nhanh từ Applications như bình thường.

## Gỡ cài đặt

Để xóa hoàn toàn Gõ Nhanh:

1. Thoát ứng dụng (Click icon trên Menu Bar -> Quit).
2. Xóa Gõ Nhanh khỏi thư mục Applications.
3. (Tùy chọn) Xóa file cấu hình tại `~/.config/gonhanh`.
