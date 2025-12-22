# Need More Info Template

Use when additional information is required from the reporter.

## Template

```markdown
Cảm ơn bạn đã báo lỗi!

## ❓ Cần thêm thông tin

Bạn có thể cung cấp thêm:
1. [question_1]
2. [question_2]

**Bật debug log:**
```bash
touch /tmp/gonhanh_debug.log && tail -f /tmp/gonhanh_debug.log
```

Gõ lại để tái hiện lỗi, sau đó gửi log giúp mình nhé!
```

## Placeholders

- `[question_1]`, `[question_2]` - Specific questions to ask

## Example

```markdown
Cảm ơn bạn đã báo lỗi!

## ❓ Cần thêm thông tin

Bạn có thể cung cấp thêm:
1. Phiên bản Gõ Nhanh đang dùng?
2. Lỗi xảy ra ở app nào (Chrome, VS Code, ...)?

**Bật debug log:**
```bash
touch /tmp/gonhanh_debug.log && tail -f /tmp/gonhanh_debug.log
```

Gõ lại để tái hiện lỗi, sau đó gửi log giúp mình nhé!
```
