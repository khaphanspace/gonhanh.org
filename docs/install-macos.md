# Gõ Nhanh trên macOS

## Cài đặt

### Cách 1: Homebrew (Khuyến nghị)

```bash
brew install --cask gonhanh
```

Sau đó cấp quyền: System Settings → Privacy & Security → Accessibility → Bật **Gõ Nhanh**

### Cách 2: Tải thủ công

1. **Tải về:** [GoNhanh.dmg](https://github.com/khaphanspace/gonhanh.org/releases/latest/download/GoNhanh.dmg)

2. **Cài đặt:** Mở file `.dmg` → Kéo vào **Applications**

3. **Cho phép chạy:** (Lưu ý phải tắt app trước khi chạy)
   ```bash
   xattr -cr /Applications/GoNhanh.app
   ```

4. **Cấp quyền:** Mở app → System Settings → Privacy & Security → Accessibility → Bật **GoNhanh**

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

Menu Bar → Cài đặt → Kiểu gõ → **VNI**

| Gõ | Kết quả |
|----|---------|
| `a1`, `a2`, `a3`, `a4`, `a5` | á, à, ả, ã, ạ |
| `a6`, `a8`, `o6`, `e6` | â, ă, ô, ê |
| `o7`, `u7`, `d9` | ơ, ư, đ |

### Tính năng khác

- **Gõ tắt:** Menu Bar → Cài đặt → Gõ tắt → Thêm từ viết tắt
- **Ngoại lệ:** Menu Bar → Cài đặt → Ngoại lệ → Thêm app không muốn gõ tiếng Việt

---

## Điều khiển qua dòng lệnh (CLI & Vim Mode)

`GoNhanh.app` đi kèm công cụ dòng lệnh `gonhanh` (bí danh `gn`) tương thích quy ước `im-select` / `macism`, đồng nhất giữa macOS và Linux.

### Thiết lập `PATH`

Nếu cài thủ công qua `.dmg` (hoặc cask chưa tự tạo symlink), tạo liên kết vào `~/.local/bin` (hoặc `/opt/homebrew/bin`):

```bash
mkdir -p ~/.local/bin
ln -sf /Applications/GoNhanh.app/Contents/SharedSupport/bin/gonhanh ~/.local/bin/gonhanh
ln -sf /Applications/GoNhanh.app/Contents/SharedSupport/bin/gn ~/.local/bin/gn
```

### Các lệnh hỗ trợ

| Lệnh | Chức năng |
|------|-----------|
| `gonhanh` (hoặc `gn`) | In trạng thái hiệu lực hiện tại: `vi` hoặc `en` |
| `gonhanh vi` | Bật gõ tiếng Việt (im lặng trên `stdout`, dùng cho Vim/script) |
| `gonhanh en` | Tắt gõ tiếng Việt (im lặng trên `stdout`, dùng cho Vim/script) |
| `gonhanh on` | Bật Gõ Nhanh và hiện trạng thái |
| `gonhanh off` | Tắt Gõ Nhanh và hiện trạng thái |
| `gonhanh toggle` | Đảo trạng thái bật/tắt |
| `gonhanh telex` | Chuyển sang Telex và bật Gõ Nhanh |
| `gonhanh vni` | Chuyển sang VNI và bật Gõ Nhanh |
| `gonhanh status` | Xem trạng thái hiện tại |
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
  "vim.autoSwitchInputMethod.obtainIMCmd": "/Applications/GoNhanh.app/Contents/SharedSupport/bin/gonhanh",
  "vim.autoSwitchInputMethod.switchIMCmd": "/Applications/GoNhanh.app/Contents/SharedSupport/bin/gonhanh {im}"
}
```

**Obsidian (Vim Motions → Input method switching):**

| Trường | Giá trị |
|--------|---------|
| IM binary path | `/opt/homebrew/bin/gonhanh` *(hoặc `/Applications/GoNhanh.app/Contents/SharedSupport/bin/gonhanh`)* |
| Obtain IM arguments | *(để trống)* |
| Switch IM arguments | `{im}` |
| Normal mode IM | `en` |
| Insert mode IM behavior | `Restore` |

---

## Nâng cấp

**Homebrew:**
```bash
brew upgrade --cask gonhanh
```

**Thủ công:** Menu Bar → Cài đặt → Cập nhật → **Tải và cài đặt**

---

## Gỡ cài đặt

**Homebrew:**
```bash
brew uninstall --cask gonhanh
```

**Thủ công:**
1. Menu Bar → **Thoát**
2. Xóa app từ Applications
3. (Tùy chọn) Xóa cấu hình:
   ```bash
   rm -rf ~/.config/gonhanh ~/Library/Preferences/org.gonhanh.*
   ```

---

## Xử lý sự cố

**App không mở được?**
```bash
xattr -cr /Applications/GoNhanh.app
```

**Không gõ được tiếng Việt?**
1. Kiểm tra icon Menu Bar hiển thị "VN"
2. Nhấn `Ctrl + Space`
3. Kiểm tra quyền Accessibility đã bật

**Sau khi cập nhật macOS:**

System Settings → Privacy & Security → Accessibility → Tắt/bật lại GoNhanh

---

## Nâng cao

<details>
<summary>Khởi động cùng hệ thống</summary>

System Settings → General → Login Items → Thêm **GoNhanh**
</details>

<details>
<summary>Yêu cầu hệ thống</summary>

- macOS 13.0 (Ventura) trở lên
- Apple Silicon hoặc Intel
</details>
