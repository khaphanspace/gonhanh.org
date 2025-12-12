# Hướng dẫn cài đặt Gõ Nhanh trên macOS

## Yêu cầu hệ thống

- **macOS**: 13.0 (Ventura) trở lên
- **Chip**: Apple Silicon (M1/M2/M3/M4) hoặc Intel

---

## Cài đặt

### 1. Tải ứng dụng

Tải xuống file `.dmg` phiên bản mới nhất tại: **[Tải Gõ Nhanh](https://github.com/khaphanspace/gonhanh.org/releases/latest/download/GoNhanh.dmg)**

_(Nếu bạn muốn chọn phiên bản cũ hơn, hãy truy cập [Danh sách Releases](https://github.com/khaphanspace/gonhanh.org/releases))_

### 2. Cài đặt vào Applications

1. Mở file `.dmg` vừa tải
2. Kéo biểu tượng **Gõ Nhanh** vào thư mục **Applications**

### 3. Mở ứng dụng lần đầu (Quan trọng)

Do Gõ Nhanh chưa được ký số bởi Apple, bạn cần chạy lệnh sau trong **Terminal** để cho phép ứng dụng khởi chạy (chỉ cần làm 1 lần):

```bash
xattr -cr /Applications/GoNhanh.app
```

### 4. Cấp quyền Accessibility

Khi mở lần đầu, macOS sẽ yêu cầu cấp quyền **Accessibility** để Gõ Nhanh có thể đọc phím bạn gõ:

1. Mở **System Settings** → **Privacy & Security** → **Accessibility**
2. Bật công tắc cho **GoNhanh**
3. Khởi động lại ứng dụng nếu cần

---

## Nâng cấp (Upgrade)

### Cách 1: Tự động cập nhật (Khuyến nghị)

1. Click icon **Gõ Nhanh** trên Menu Bar
2. Chọn **Cài đặt** → **Cập nhật**
3. Nếu có phiên bản mới, click **Tải và cài đặt**

### Cách 2: Cập nhật thủ công

1. Tải file `.dmg` phiên bản mới từ [Releases](https://github.com/khaphanspace/gonhanh.org/releases)
2. Thoát ứng dụng Gõ Nhanh đang chạy
3. Kéo ứng dụng mới vào Applications, chọn **Replace** khi được hỏi

---

## Gỡ cài đặt (Uninstall)

1. Thoát ứng dụng: Click icon trên Menu Bar → **Thoát** (hoặc **Quit**)
2. Xóa ứng dụng: Kéo **GoNhanh** từ thư mục Applications vào Trash
3. (Tùy chọn) Xóa file cấu hình:
   ```bash
   rm -rf ~/.config/gonhanh
   rm -rf ~/Library/Preferences/org.gonhanh.*
   ```

---

## Hướng dẫn sử dụng

### Bật/Tắt bộ gõ

- **Phím tắt mặc định**: `Ctrl + Space` - Bật/tắt gõ tiếng Việt
- **Hoặc**: Click icon trên Menu Bar → Toggle công tắc

Khi bộ gõ **bật**: Icon hiển thị "VN"
Khi bộ gõ **tắt**: Icon mờ đi

### Cài đặt (Settings)

Click icon trên Menu Bar → **Cài đặt** để mở cửa sổ cài đặt với các tab:

#### Tab "Kiểu gõ" (Method)

Chọn kiểu gõ bạn muốn sử dụng:
- **Telex** (mặc định): Sử dụng các chữ cái để tạo dấu
- **VNI**: Sử dụng số để tạo dấu

#### Tab "Gõ tắt" (Shortcut)

Tạo các từ viết tắt để gõ nhanh hơn:
- Click **+** để thêm gõ tắt mới
- Nhập **Từ khóa** và **Nội dung thay thế**
- Ví dụ: `vn` → `Việt Nam`, `hcm` → `Hồ Chí Minh`

#### Tab "Ngoại lệ" (Exclude)

Thêm các ứng dụng mà bạn không muốn Gõ Nhanh hoạt động:
- Click **+** để thêm ứng dụng
- Gõ Nhanh sẽ tự động tắt khi bạn đang dùng các ứng dụng này
- Hữu ích cho: Terminal, IDE, các ứng dụng cần gõ code

### Quy tắc gõ Telex

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

### Quy tắc gõ VNI

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

## Xử lý sự cố

### Ứng dụng không mở được

**Lỗi "App is damaged":**
```bash
xattr -cr /Applications/GoNhanh.app
```

**Lỗi "Cannot be opened because the developer cannot be verified":**
1. Mở **System Settings** → **Privacy & Security**
2. Cuộn xuống tìm thông báo về GoNhanh
3. Click **Open Anyway**

### Không gõ được tiếng Việt

1. Kiểm tra icon trên Menu Bar có hiển thị "VN" không
2. Thử nhấn `Ctrl + Space` để bật bộ gõ
3. Kiểm tra quyền Accessibility đã được cấp chưa
4. Kiểm tra ứng dụng hiện tại có trong danh sách Ngoại lệ không

### Bộ gõ không hoạt động sau khi cập nhật macOS

1. Mở **System Settings** → **Privacy & Security** → **Accessibility**
2. Tắt rồi bật lại quyền cho GoNhanh
3. Khởi động lại ứng dụng

---

## Khởi động cùng hệ thống

Để Gõ Nhanh tự động chạy khi khởi động máy:

1. Mở **System Settings** → **General** → **Login Items**
2. Click **+** và chọn **GoNhanh** từ Applications

---

## Liên kết hữu ích

- [Mã nguồn macOS](../platforms/macos/)
- [Báo lỗi](https://github.com/khaphanspace/gonhanh.org/issues)
