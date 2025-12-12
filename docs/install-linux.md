# Hướng dẫn cài đặt Gõ Nhanh trên Linux

## Cài đặt (1 lệnh)

```bash
curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/install-linux.sh | bash
```

Xong! Restart Fcitx5 và thêm Gõ Nhanh trong `fcitx5-configtool`.

---

## Cài đặt thủ công

### Yêu cầu

- Ubuntu 22.04+ / Fedora 38+ / Arch Linux
- Fcitx5 (`sudo apt install fcitx5 fcitx5-configtool`)

### Các bước

```bash
# 1. Tải và cài đặt
curl -LO https://github.com/khaphanspace/gonhanh.org/releases/latest/download/gonhanh-linux.tar.gz
tar xzf gonhanh-linux.tar.gz && cd gonhanh-linux && ./install.sh

# 2. Restart và cấu hình
fcitx5 -r && fcitx5-configtool  # Input Method → Add → GoNhanh
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
