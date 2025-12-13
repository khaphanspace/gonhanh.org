# Hướng dẫn cài đặt Gõ Nhanh trên Linux

## Cài đặt nhanh (1 lệnh)

```bash
curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/install-linux.sh | bash
```

Script sẽ tự động:
- Cài đặt Fcitx5 (nếu chưa có)
- Tải và cài đặt Gõ Nhanh
- Cấu hình biến môi trường
- Thêm GoNhanh vào Fcitx5
- Khởi động Fcitx5

**Sau khi cài xong**, bạn có thể cần đăng xuất/đăng nhập lại để áp dụng đầy đủ.

---

## Yêu cầu hệ thống

- **Hệ điều hành**: Ubuntu 22.04+, Fedora 38+, Arch Linux, hoặc các distro có hỗ trợ Fcitx5
- **Quyền**: sudo (để cài Fcitx5 nếu chưa có)

---

## Nâng cấp (Upgrade)

Chạy lại script cài đặt:

```bash
curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/install-linux.sh | bash
```

Script sẽ tự động ghi đè các file cũ bằng phiên bản mới.

---

## Gỡ cài đặt (Uninstall)

```bash
# Xóa các file thư viện
rm -f ~/.local/lib/fcitx5/gonhanh.so
rm -f ~/.local/lib/libgonhanh_core.so

# Xóa các file cấu hình addon
rm -f ~/.local/share/fcitx5/addon/gonhanh.conf
rm -f ~/.local/share/fcitx5/inputmethod/gonhanh.conf

# Xóa file cấu hình người dùng (tùy chọn)
rm -rf ~/.config/gonhanh

# Khởi động lại Fcitx5
fcitx5 -r
```

---

## Hướng dẫn sử dụng

### Bật/Tắt bộ gõ

- **Phím tắt**: `Ctrl + Space` - Chuyển đổi giữa gõ tiếng Việt và tiếng Anh

### Chuyển đổi kiểu gõ

**Chuyển sang VNI:**
```bash
mkdir -p ~/.config/gonhanh && echo "vni" > ~/.config/gonhanh/method
fcitx5 -r
```

**Chuyển về Telex (mặc định):**
```bash
echo "telex" > ~/.config/gonhanh/method
fcitx5 -r
```

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

### Không gõ được tiếng Việt sau khi cài

1. **Đăng xuất và đăng nhập lại** để áp dụng biến môi trường

2. Kiểm tra Fcitx5 đang chạy:
   ```bash
   pgrep fcitx5 || fcitx5 -d
   ```

3. Kiểm tra biến môi trường:
   ```bash
   echo $GTK_IM_MODULE  # Phải là "fcitx"
   ```

### Bộ gõ không xuất hiện trong danh sách Fcitx5

1. Kiểm tra file addon đã được cài:
   ```bash
   ls ~/.local/share/fcitx5/addon/gonhanh.conf
   ```

2. Mở Fcitx5 Config Tool và thêm GoNhanh thủ công:
   ```bash
   fcitx5-configtool
   ```
   → Input Method → Add → **GoNhanh**

### Fcitx5 không tự khởi động

Thêm vào autostart:
```bash
mkdir -p ~/.config/autostart
cat > ~/.config/autostart/fcitx5.desktop << 'EOF'
[Desktop Entry]
Name=Fcitx5
Exec=fcitx5 -d
Type=Application
EOF
```

---

## Build từ source

Xem hướng dẫn chi tiết tại [platforms/linux/README.md](../platforms/linux/README.md)

---

## Liên kết hữu ích

- [Mã nguồn Linux](../platforms/linux/)
- [Báo lỗi](https://github.com/khaphanspace/gonhanh.org/issues)
