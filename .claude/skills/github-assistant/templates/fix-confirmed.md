# Fix Confirmed Template

Use when a bug has been fixed and released.

## Template

```markdown
Cảm ơn bạn đã báo lỗi và sử dụng Gõ Nhanh!

## ✅ Đã fix trong v[version]

**Commit:** [`[commit_hash]`]([commit_url])

**Files changed:**
- [`[file_path]`]([file_url]#L[start]-L[end])

**Nguyên nhân:** [root_cause]

**Cách fix:** [fix_description]

---

📥 **Vui lòng cập nhật lên v[version]** để sử dụng bản fix này.
```

## Get Commit Info

```bash
# Get commit hash and URL
gh api repos/{owner}/{repo}/commits/{sha} --jq '.html_url'

# Get files changed with line numbers
git show --stat {sha}
git diff {sha}~1 {sha} --name-only

# Get file URL with lines
# Format: https://github.com/{owner}/{repo}/blob/{sha}/{file}#L{start}-L{end}
```

## Example

```markdown
Cảm ơn bạn đã báo lỗi và sử dụng Gõ Nhanh!

## ✅ Đã fix trong v1.0.85

**Commit:** [`4f79b1c`](https://github.com/aspect-build/aspect-cli/commit/4f79b1c)

**Files changed:**
- [`core/src/engine/mod.rs`](https://github.com/aspect-build/aspect-cli/blob/4f79b1c/core/src/engine/mod.rs#L123-L145)
- [`core/src/engine/buffer.rs`](https://github.com/aspect-build/aspect-cli/blob/4f79b1c/core/src/engine/buffer.rs#L67-L89)

**Nguyên nhân:** Buffer không được reset đúng cách sau thao tác DELETE.

**Cách fix:** Thêm cơ chế `restored_pending_clear` để đảm bảo buffer được clear đúng.

**Liên quan:** #98, #106

---

📥 **Vui lòng cập nhật lên v1.0.85** để sử dụng bản fix này.
```
