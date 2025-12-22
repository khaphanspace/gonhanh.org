# Multi-Report Response Template

Use when responding to multiple reports in a single issue.

## Template

```markdown
Cảm ơn các bạn đã góp ý!

### Phản hồi:

**@[username1]** - [issue_description]: [status] ([`commit`](commit_url))

**@[username2]** - [issue_description]: [status] ([`commit`](commit_url))

**Files changed:**
- [`[file]`](file_url#L[start]-L[end])

---

📥 **Vui lòng cập nhật lên v[version]** để sử dụng các bản fix mới.

Cảm ơn đã đồng hành cùng Gõ Nhanh! 🙏
```

## Status Options

- ✅ **Đã fix** - Bug fixed
- ⏳ Đang xử lý - In progress
- ❓ Cần thêm thông tin - Need more info
- ℹ️ Hạn chế hiện tại - Known limitation

## Example

```markdown
Cảm ơn các bạn đã góp ý!

### Phản hồi:

**@khangbinhdl** - Lỗi shortcut mất space: ✅ **Đã fix** ([`b5c0401`](https://github.com/aspect/repo/commit/b5c0401))

**@binhgiap** - "tieeps" không ra "tiếp": ✅ **Đã fix** ([`4147a1f`](https://github.com/aspect/repo/commit/4147a1f))

**@linhnhatnguyenepita** - Từ vùng miền "chơ": ✅ **Đã fix** ([`9b67ae3`](https://github.com/aspect/repo/commit/9b67ae3))

**Files changed:**
- [`core/src/engine/mod.rs`](https://github.com/aspect/repo/blob/main/core/src/engine/mod.rs#L123-L145)
- [`core/src/engine/buffer.rs`](https://github.com/aspect/repo/blob/main/core/src/engine/buffer.rs#L67-L89)

**Liên quan:** #98, #106, PR #103

---

📥 **Vui lòng cập nhật lên v1.0.85** để sử dụng các bản fix mới.

Cảm ơn đã đồng hành cùng Gõ Nhanh! 🙏
```
