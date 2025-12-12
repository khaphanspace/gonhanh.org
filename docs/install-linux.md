# Hướng dẫn cài đặt Gõ Nhanh trên Linux

## Yêu cầu hệ thống

- Ubuntu 22.04+ / Fedora 38+ / Arch Linux
- Fcitx5 input method framework
- Wayland hoặc X11

## Cài đặt nhanh

### 1. Cài đặt Fcitx5 (nếu chưa có)

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

### 2. Tải và cài đặt Gõ Nhanh

```bash
# Tải bản release mới nhất
curl -LO https://github.com/user/gonhanh/releases/latest/download/gonhanh-linux-x64.tar.gz

# Giải nén và cài đặt
tar xzf gonhanh-linux-x64.tar.gz
cd gonhanh-*-linux-x64
./install.sh
```

### 3. Cấu hình Fcitx5

```bash
# Khởi động lại Fcitx5
fcitx5 -r &

# Mở cấu hình
fcitx5-configtool
```

Trong Fcitx5 Configuration:
1. Vào **Input Method** tab
2. Click **Add Input Method**
3. Tìm và chọn **GoNhanh**
4. Click **OK**

### 4. Thiết lập biến môi trường

Thêm vào `~/.bashrc` hoặc `~/.zshrc`:

```bash
export GTK_IM_MODULE=fcitx
export QT_IM_MODULE=fcitx
export XMODIFIERS=@im=fcitx
```

Sau đó logout/login hoặc chạy:
```bash
source ~/.bashrc
```

## Sử dụng

- **Ctrl+Space** - Bật/tắt bộ gõ
- Mặc định sử dụng kiểu gõ **Telex**
- Hỗ trợ cả **VNI** (cấu hình trong settings)

## Gỡ cài đặt

```bash
cd gonhanh-*-linux-x64
./install.sh --uninstall
```

Hoặc xóa thủ công:
```bash
rm -f ~/.local/lib/fcitx5/gonhanh.so
rm -f ~/.local/lib/libgonhanh_core.so
rm -f ~/.local/share/fcitx5/addon/gonhanh.conf
rm -f ~/.local/share/fcitx5/inputmethod/gonhanh.conf
```

## Xử lý sự cố

### Bộ gõ không xuất hiện trong danh sách

```bash
# Kiểm tra addon đã cài đặt
ls -la ~/.local/lib/fcitx5/

# Khởi động lại Fcitx5
fcitx5 -r &
```

### Không gõ được tiếng Việt

1. Kiểm tra biến môi trường:
   ```bash
   echo $GTK_IM_MODULE  # Phải là "fcitx"
   ```

2. Kiểm tra Fcitx5 đang chạy:
   ```bash
   pgrep fcitx5
   ```

### Build từ source

Xem hướng dẫn chi tiết tại [platforms/linux/README.md](../platforms/linux/README.md).
