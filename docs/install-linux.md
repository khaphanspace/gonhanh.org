# Gõ Nhanh trên Linux

Gõ Nhanh cho Linux là addon Fcitx5 beta, hiện phát hành binary cho **x86_64**.
Trình cài đặt thêm GoNhanh vào profile hiện có và không xoá Bamboo hoặc input
method khác.

## Cài đặt

```bash
curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/setup/linux.sh | bash
```

Sau khi cài:

1. Đăng xuất rồi đăng nhập lại để desktop nạp Fcitx5.
2. Chạy `source ~/.profile` để dùng lệnh `gonhanh` (hoặc bí danh `gn`) ngay trong terminal hiện tại.
3. Chạy `gonhanh on` (hoặc `gn on`) để chọn GoNhanh.

Có thể tải archive thủ công tại
[`gonhanh-linux.tar.gz`](https://github.com/khaphanspace/gonhanh.org/releases/latest/download/gonhanh-linux.tar.gz),
giải nén và chạy `gonhanh-linux/install.sh`.

## Sử dụng (CLI & Vim Mode)

CLI `gonhanh` / `gn` tương thích quy ước `im-select` / `macism` và đồng nhất với bản macOS:

| Lệnh | Chức năng |
|------|-----------|
| `gonhanh` (hoặc `gn`) | In trạng thái hiện tại: `vi` hoặc `en` |
| `gonhanh vi` | Bật GoNhanh (im lặng trên `stdout`, dùng cho Vim/script) |
| `gonhanh en` | Tắt GoNhanh (im lặng trên `stdout`, dùng cho Vim/script) |
| `gonhanh toggle` | Đảo trạng thái bật/tắt riêng GoNhanh |
| `gonhanh on` | Chọn và bật GoNhanh |
| `gonhanh off` | Tắt GoNhanh nếu đang được chọn |
| `gonhanh vni` | Chuyển sang VNI, tải lại Fcitx5 và bật GoNhanh |
| `gonhanh telex` | Chuyển sang Telex, tải lại Fcitx5 và bật GoNhanh |
| `gonhanh status` | Xem trạng thái riêng của GoNhanh và input method hiện tại |
| `gonhanh update` | Cài bản Linux mới nhất |
| `gonhanh uninstall` | Gỡ GoNhanh, giữ nguyên input method khác |
| `gonhanh version` | Xem phiên bản |
| `gonhanh help` | Hiển thị trợ giúp |

### Tích hợp Vim Mode

**Neovim ([`im-select.nvim`](https://github.com/keaising/im-select.nvim)):**
```lua
require("im_select").setup({
  default_command = "gonhanh",
  default_im_select = "en",
})
```

**VS Code ([VSCodeVim](https://marketplace.visualstudio.com/items?itemName=vscodevim.vim)):**
```json
{
  "vim.autoSwitchInputMethod.enable": true,
  "vim.autoSwitchInputMethod.defaultIM": "en",
  "vim.autoSwitchInputMethod.obtainIMCmd": "gonhanh",
  "vim.autoSwitchInputMethod.switchIMCmd": "gonhanh {im}"
}
```

Phím chuyển input method mặc định của Fcitx5 thường là `Ctrl + Space`; cấu hình
thực tế phụ thuộc desktop.

## Gỡ cài đặt

```bash
gn uninstall
```

Trình gỡ cài đặt chỉ xoá file và profile item của GoNhanh. Bản sao profile trước
lần chỉnh đầu tiên được giữ tại `~/.config/fcitx5/profile.gonhanh.bak`.

## Xử lý sự cố

### Không tìm thấy `gn`

```bash
source ~/.profile
```

Hoặc mở terminal mới sau khi đăng nhập lại.

### GoNhanh chưa xuất hiện

```bash
fcitx5 -r
fcitx5-configtool
```

Kiểm tra các file:

```bash
ls ~/.local/lib/fcitx5/gonhanh.so
ls ~/.local/share/fcitx5/inputmethod/gonhanh.conf
```

### Addon không tải được

```bash
ldd ~/.local/lib/fcitx5/gonhanh.so
fcitx5 --verbose=*:4
```

### Kiến trúc ARM64 hoặc distro khác

Binary phát hành hiện chỉ hỗ trợ x86_64. Xem hướng dẫn build trong
[`platforms/linux/README.md`](../platforms/linux/README.md).

## Cài Fcitx5 thủ công

```bash
# Ubuntu/Debian
sudo apt install fcitx5 fcitx5-config-qt

# Fedora
sudo dnf install fcitx5 fcitx5-configtool

# Arch Linux
sudo pacman -S fcitx5 fcitx5-configtool
```
