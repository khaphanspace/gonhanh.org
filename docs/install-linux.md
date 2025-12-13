# Gõ Nhanh trên Linux

## Cài đặt

```bash
curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/install-linux.sh | bash
```

Đăng xuất và đăng nhập lại để hoàn tất.

---

## Sử dụng

| Phím tắt | Chức năng |
|----------|-----------|
| `Ctrl + Space` | Bật/tắt tiếng Việt |

### Telex (mặc định)

| Gõ | Kết quả |
|----|---------|
| `as`, `af`, `ar`, `ax`, `aj` | á, à, ả, ã, ạ |
| `aa`, `aw`, `ee`, `oo` | â, ă, ê, ô |
| `ow`, `uw`, `dd` | ơ, ư, đ |

### Đổi sang VNI

```bash
mkdir -p ~/.config/gonhanh && echo "vni" > ~/.config/gonhanh/method && fcitx5 -r
```

| Gõ | Kết quả |
|----|---------|
| `a1`, `a2`, `a3`, `a4`, `a5` | á, à, ả, ã, ạ |
| `a6`, `a8`, `o6`, `e6` | â, ă, ô, ê |
| `o7`, `u7`, `d9` | ơ, ư, đ |

---

## Nâng cấp

```bash
curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/install-linux.sh | bash
```

---

## Gỡ cài đặt

```bash
rm -f ~/.local/lib/fcitx5/gonhanh.so ~/.local/lib/libgonhanh_core.so
rm -f ~/.local/share/fcitx5/addon/gonhanh.conf ~/.local/share/fcitx5/inputmethod/gonhanh.conf
rm -rf ~/.config/gonhanh
fcitx5 -r
```

---

## Xử lý sự cố

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
