# Hướng dẫn cài đặt Gõ Nhanh trên Linux

## Yêu cầu hệ thống

- **Hệ điều hành**: Ubuntu 22.04+, Fedora 38+, Arch Linux, hoặc các distro có hỗ trợ Fcitx5
- **Input Method Framework**: Fcitx5
- **Thư viện**: libfcitx5core, cmake, g++ (để build từ source)

### Cài đặt Fcitx5 (nếu chưa có)

**Ubuntu/Debian:**
```bash
sudo apt install fcitx5 fcitx5-configtool
```

**Fedora:**
```bash
sudo dnf install fcitx5 fcitx5-configtool
```

**Arch Linux:**
```bash
sudo pacman -S fcitx5 fcitx5-configtool
```

---

## Cài đặt

### Cách 1: Cài đặt tự động (Khuyến nghị)

```bash
curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/install-linux.sh | bash
```

Sau khi cài xong, thêm Gõ Nhanh vào Fcitx5:
```bash
fcitx5 -r && fcitx5-configtool
```
→ Input Method → Add → **GoNhanh**

### Cách 2: Build từ source

Xem hướng dẫn chi tiết tại [platforms/linux/README.md](../platforms/linux/README.md)

---

## Nâng cấp (Upgrade)

Chạy lại script cài đặt để cập nhật lên phiên bản mới nhất:

```bash
curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/install-linux.sh | bash
fcitx5 -r
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

- **Phím tắt**: `Ctrl + Space` - Chuyển đổi giữa bộ gõ tiếng Việt và tiếng Anh

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

### Bộ gõ không xuất hiện trong danh sách

1. Kiểm tra Fcitx5 đang chạy:
   ```bash
   pgrep fcitx5
   ```

2. Kiểm tra file addon đã được cài:
   ```bash
   ls ~/.local/share/fcitx5/addon/gonhanh.conf
   ```

3. Khởi động lại Fcitx5:
   ```bash
   fcitx5 -r
   ```

### Không gõ được tiếng Việt

1. Đảm bảo đã chọn GoNhanh trong Input Method
2. Thử nhấn `Ctrl + Space` để bật bộ gõ
3. Kiểm tra biến môi trường:
   ```bash
   echo $INPUT_METHOD
   echo $GTK_IM_MODULE
   echo $QT_IM_MODULE
   ```

   Nếu chưa có, thêm vào `~/.profile` hoặc `~/.bashrc`:
   ```bash
   export INPUT_METHOD=fcitx
   export GTK_IM_MODULE=fcitx
   export QT_IM_MODULE=fcitx
   export XMODIFIERS=@im=fcitx
   ```

---

## Liên kết hữu ích

- [Mã nguồn Linux](../platforms/linux/)
- [Báo lỗi](https://github.com/khaphanspace/gonhanh.org/issues)
