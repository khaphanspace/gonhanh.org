# Gõ Nhanh trên Windows

> **Coming Soon** - Đang phát triển

---

## Tính năng dự kiến

- Gõ tiếng Việt Telex/VNI
- Gõ tắt tùy chỉnh
- Danh sách app ngoại lệ
- System tray menu
- Tự khởi động cùng Windows

---

## Quy tắc gõ (Tham khảo)

### Telex

| Gõ | Kết quả |
|----|---------|
| `as`, `af`, `ar`, `ax`, `aj` | á, à, ả, ã, ạ |
| `aa`, `aw`, `ee`, `oo` | â, ă, ê, ô |
| `ow`, `uw`, `dd` | ơ, ư, đ |

### VNI

| Gõ | Kết quả |
|----|---------|
| `a1`, `a2`, `a3`, `a4`, `a5` | á, à, ả, ã, ạ |
| `a6`, `a8`, `o6`, `e6` | â, ă, ô, ê |
| `o7`, `u7`, `d9` | ơ, ư, đ |

---

## Theo dõi

- [Releases](https://github.com/khaphanspace/gonhanh.org/releases)
- [GitHub Issues](https://github.com/khaphanspace/gonhanh.org/issues)

---

## Dành cho Developer

<details>
<summary>Build từ source</summary>

**Yêu cầu:**
- Windows 10/11
- [Rust](https://rustup.rs/)
- [Visual Studio 2022](https://visualstudio.microsoft.com/) (C++ & .NET workload)

```powershell
git clone https://github.com/khaphanspace/gonhanh.org.git
cd gonhanh.org/platforms/windows
cargo build --release
```
</details>
