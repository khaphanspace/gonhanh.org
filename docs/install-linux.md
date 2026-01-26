# Gõ Nhanh trên Linux

## Cài đặt

```bash
curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/setup/linux.sh | bash
```

Đăng xuất và đăng nhập lại để hoàn tất.

---

## Sử dụng

| Phím tắt | Chức năng |
|----------|-----------|
| `Ctrl + Space` hoặc `Super + Space` | Bật/tắt tiếng Việt (tùy desktop) |

| Lệnh | Chức năng |
|------|-----------|
| `gn` | Toggle bật/tắt |
| `gn on` | Bật tiếng Việt |
| `gn off` | Tắt tiếng Việt |
| `gn vni` | Chuyển sang VNI |
| `gn telex` | Chuyển sang Telex |
| `gn status` | Xem trạng thái |
| `gn update` | Cập nhật phiên bản mới |
| `gn uninstall` | Gỡ cài đặt |
| `gn version` | Xem phiên bản |
| `gn help` | Hiển thị trợ giúp |

---

## Gỡ cài đặt

```bash
gn uninstall
```

---

## Xử lý sự cố

**Lệnh `gn` không tìm thấy?**
```bash
source ~/.bashrc
```

**Không gõ được tiếng Việt?**
1. Đăng xuất/đăng nhập lại
2. Kiểm tra Fcitx5: `pgrep fcitx5 || fcitx5 -d`
3. Kiểm tra biến môi trường: `echo $GTK_IM_MODULE` (phải là `fcitx`)

**Thêm GoNhanh thủ công:**
```bash
fcitx5-configtool
```
→ Input Method → Add → GoNhanh

---

## Nâng cao

<details>
<summary>Cài Fcitx5 thủ công</summary>

```bash
# Ubuntu/Debian
sudo apt install fcitx5 fcitx5-configtool

# Fedora
sudo dnf install fcitx5 fcitx5-configtool

# Arch
sudo pacman -S fcitx5 fcitx5-configtool
```
</details>

<details>
<summary>Build từ source</summary>

Xem [platforms/linux/README.md](../platforms/linux/README.md)
</details>
