# Cài đặt Gõ Nhanh trên Linux

```bash
curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/install-linux.sh | bash
```

Sau đó:
```bash
fcitx5 -r && fcitx5-configtool
```
→ Input Method → Add → **GoNhanh**

---

## Phím tắt

- **Ctrl+Space** - Bật/tắt bộ gõ
- Mặc định: Telex

## Gỡ cài đặt

```bash
rm -f ~/.local/lib/fcitx5/gonhanh.so ~/.local/lib/libgonhanh_core.so
rm -f ~/.local/share/fcitx5/addon/gonhanh.conf ~/.local/share/fcitx5/inputmethod/gonhanh.conf
fcitx5 -r
```

## Build từ source

Xem [platforms/linux/README.md](../platforms/linux/README.md)
