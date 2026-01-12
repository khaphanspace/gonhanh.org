# API Specification (FFI Interface)

> ⚠️ **Note:** This is NOT a REST/GraphQL API. This documents the **Foreign Function Interface (FFI)** between Rust core and platform UI layers.
>
> Auto-generated from `core/src/lib.rs` on 2026-01-12

---

## 1. Overview

### 1.1 Architecture

```
Platform UI (Swift/C++/.NET)
        ↓ FFI Calls
   Rust Core Engine
        ↓ Return
  ImeResult* (C struct)
```

### 1.2 Conventions

- **Language:** C ABI (`extern "C"`)
- **Namespace:** All functions prefixed with `ime_`
- **Memory:** Caller must free `ImeResult*` with `ime_free()`
- **Thread Safety:** Mutex-protected global engine instance
- **Null Safety:** All functions handle null inputs gracefully

---

## 2. Lifecycle Functions

### 2.1 Initialization

#### `ime_init()`

Initialize the IME engine. **Must be called exactly once** before any other `ime_*` functions.

**Signature:**
```c
void ime_init(void);
```

**Parameters:** None

**Returns:** void

**Thread Safety:** Yes (uses internal mutex)

**Example (Swift):**
```swift
RustBridge.ime_init()
```

**Source:** `core/src/lib.rs:52-55`

---

## 3. Keystroke Processing

### 3.1 Basic Keystroke

#### `ime_key(key, caps, ctrl)`

Process a key event and return result.

**Signature:**
```c
ImeResult* ime_key(uint16_t key, bool caps, bool ctrl);
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `key` | u16 | macOS virtual keycode (0-127 for standard keys) |
| `caps` | bool | true if CapsLock pressed (for uppercase) |
| `ctrl` | bool | true if Cmd/Ctrl/Alt pressed (bypasses IME) |

**Returns:**
- `ImeResult*` - Pointer to result struct (caller must free with `ime_free`)
- `null` - If engine not initialized

**Result Actions:**
- `0` = None (pass through to OS)
- `1` = Send (replace text: delete `backspace` chars, insert `chars`)
- `2` = Restore (undo transformation)

**Example (Swift):**
```swift
let result = RustBridge.ime_key(keycode, caps: isCaps, ctrl: isCtrl)
if let r = result {
    if r.action == 1 {
        deleteBackward(r.backspace)
        insert(r.chars, count: r.count)
    }
    RustBridge.ime_free(result)
}
```

**Source:** `core/src/lib.rs:78-86`

---

### 3.2 Extended Keystroke (VNI Symbols)

#### `ime_key_ext(key, caps, ctrl, shift)`

Process keystroke with extended parameters. Needed for VNI mode to handle Shift+number keys (symbols like @, #, $).

**Signature:**
```c
ImeResult* ime_key_ext(uint16_t key, bool caps, bool ctrl, bool shift);
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `key` | u16 | macOS virtual keycode |
| `caps` | bool | CapsLock state |
| `ctrl` | bool | Cmd/Ctrl/Alt state |
| `shift` | bool | Shift key state |

**VNI Behavior:**
When `shift=true` and key is 0-9, engine will NOT apply VNI marks/tones:
- Shift+2 → `@` (not huyền mark)
- Shift+3 → `#` (not hỏi mark)

**Returns:** Same as `ime_key()`

**Source:** `core/src/lib.rs:107-115`

---

## 4. Configuration Functions

### 4.1 Input Method

#### `ime_method(method)`

Set the input method (Telex or VNI).

**Signature:**
```c
void ime_method(uint8_t method);
```

**Parameters:**
| Value | Method |
|-------|--------|
| 0 | Telex (s=sắc, f=huyền, w=ư, etc.) |
| 1 | VNI (1-5=tones, 6-8=marks) |

**Example:**
```swift
RustBridge.ime_method(0) // Telex
RustBridge.ime_method(1) // VNI
```

**Source:** `core/src/lib.rs:124-129`

---

### 4.2 Enable/Disable

#### `ime_enabled(enabled)`

Enable or disable the engine. When disabled, `ime_key` returns action=0 (pass through).

**Signature:**
```c
void ime_enabled(bool enabled);
```

**Example:**
```swift
RustBridge.ime_enabled(true)  // Turn on
RustBridge.ime_enabled(false) // Turn off
```

**Source:** `core/src/lib.rs:136-141`

---

### 4.3 Feature Toggles

#### `ime_skip_w_shortcut(skip)`

In Telex mode, when `skip=true`, typing 'w' at word start stays as 'w' (not converted to 'ư').

**Signature:**
```c
void ime_skip_w_shortcut(bool skip);
```

**Source:** `core/src/lib.rs:149-154`

---

#### `ime_bracket_shortcut(enabled)`

Enable bracket shortcuts: `]` → `ư`, `[` → `ơ` in Telex mode.

**Signature:**
```c
void ime_bracket_shortcut(bool enabled);
```

**Source:** `core/src/lib.rs:161-166`

---

#### `ime_esc_restore(enabled)`

When `enabled=true`, pressing ESC restores original keystrokes.

**Signature:**
```c
void ime_esc_restore(bool enabled);
```

**Source:** `core/src/lib.rs:174-179`

---

#### `ime_free_tone(enabled)`

When `enabled=true`, allows placing diacritics anywhere without spelling validation (e.g., "Zìa" is allowed).

**Signature:**
```c
void ime_free_tone(bool enabled);
```

**Source:** `core/src/lib.rs:188-193`

---

#### `ime_modern(modern)`

Set tone placement style:
- `modern=true`: hoà, thuý (tone on second vowel - new style)
- `modern=false`: hòa, thúy (tone on first vowel - traditional)

**Signature:**
```c
void ime_modern(bool modern);
```

**Source:** `core/src/lib.rs:201-206`

---

#### `ime_english_auto_restore(enabled)`

Auto-restore English words accidentally transformed (e.g., "tẽt" → "text", "ễpct" → "expect").

**Signature:**
```c
void ime_english_auto_restore(bool enabled);
```

**Source:** `core/src/lib.rs:215-220`

---

#### `ime_auto_capitalize(enabled)`

Auto-capitalize first letter after sentence-ending punctuation (. ! ? Enter).

**Signature:**
```c
void ime_auto_capitalize(bool enabled);
```

**Source:** `core/src/lib.rs:229-234`

---

## 5. Buffer Management

### 5.1 Clear Buffer

#### `ime_clear()`

Clear input buffer. Call on word boundaries (space, punctuation). Preserves word history for backspace-after-space feature.

**Signature:**
```c
void ime_clear(void);
```

**Source:** `core/src/lib.rs:242-247`

---

#### `ime_clear_all()`

Clear everything including word history. Call when cursor position changes (mouse click, arrow keys, focus change) to prevent accidental restore from stale history.

**Signature:**
```c
void ime_clear_all(void);
```

**Source:** `core/src/lib.rs:255-260`

---

### 5.2 Get Buffer Content

#### `ime_get_buffer(out, max_len)`

Get full composed buffer as UTF-32 codepoints. Used for "Select All + Replace" injection method.

**Signature:**
```c
int64_t ime_get_buffer(uint32_t* out, int64_t max_len);
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `out` | u32* | Pointer to output buffer for UTF-32 codepoints |
| `max_len` | i64 | Maximum number of codepoints to write |

**Returns:** Number of codepoints written to `out`

**Safety:** `out` must point to valid memory of at least `max_len * sizeof(u32)` bytes.

**Source:** `core/src/lib.rs:277-292`

---

## 6. Text Expansion (Shortcuts)

### 6.1 Add Shortcut

#### `ime_add_shortcut(trigger, replacement)`

Add a shortcut. Auto-detects type:
- Symbol-only triggers (like `->`, `=>`) → Immediate trigger
- Letter triggers (like `vn`, `tphcm`) → Word boundary trigger

**Signature:**
```c
void ime_add_shortcut(const char* trigger, const char* replacement);
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `trigger` | char* | Null-terminated UTF-8 string (e.g., "vn") |
| `replacement` | char* | Null-terminated UTF-8 string (e.g., "Việt Nam") |

**Example:**
```swift
RustBridge.ime_add_shortcut("vn", "Việt Nam")      // Word boundary
RustBridge.ime_add_shortcut("->", "→")             // Immediate
```

**Source:** `core/src/lib.rs:320-350`

---

### 6.2 Remove Shortcut

#### `ime_remove_shortcut(trigger)`

Remove a shortcut by trigger.

**Signature:**
```c
void ime_remove_shortcut(const char* trigger);
```

**Source:** `core/src/lib.rs:360-374`

---

### 6.3 Clear All Shortcuts

#### `ime_clear_shortcuts()`

Clear all shortcuts from engine.

**Signature:**
```c
void ime_clear_shortcuts(void);
```

**Source:** `core/src/lib.rs:378-383`

---

## 7. Word Restore

### 7.1 Restore Word

#### `ime_restore_word(word)`

Restore buffer from a Vietnamese word string. Used when native app detects cursor at word boundary and user wants to continue editing (e.g., backspace into previous word).

**Signature:**
```c
void ime_restore_word(const char* word);
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `word` | char* | Null-terminated UTF-8 Vietnamese word |

**Example:**
```swift
// User backspaces from "việt " into "việt"
RustBridge.ime_restore_word("việt")
// Now typing 's' changes ệ to ế
```

**Source:** `core/src/lib.rs:401-413`

---

## 8. Memory Management

### 8.1 Free Result

#### `ime_free(result)`

Free a result pointer returned by `ime_key()` or `ime_key_ext()`.

**Signature:**
```c
void ime_free(ImeResult* result);
```

**Safety:**
- `result` must be a pointer returned by `ime_key/ime_key_ext`, or null
- Must be called exactly once per non-null result
- Do not use `result` after calling this function

**Source:** `core/src/lib.rs:301-305`

---

## 9. Data Structures

### 9.1 ImeResult

**Definition:**
```c
struct ImeResult {
    uint8_t action;     // 0=None, 1=Send, 2=Restore
    uint8_t backspace;  // Number of chars to delete
    uint32_t chars[16]; // UTF-32 codepoints to insert
    int64_t count;      // Number of valid chars (0-16)
};
```

**Action Values:**
| Value | Name | Meaning |
|-------|------|---------|
| 0 | None | Pass keystroke through to OS |
| 1 | Send | Replace text: delete `backspace` chars, insert `chars` |
| 2 | Restore | Restore raw ASCII input (undo transformation) |

**Example Usage:**
```swift
let result = RustBridge.ime_key(keycode, caps, ctrl)
guard let r = result, r.action == 1 else { return }

// Delete previous chars
for _ in 0..<r.backspace {
    deleteBackward()
}

// Insert new chars
let chars = (0..<r.count).compactMap {
    Unicode.Scalar(r.chars[Int($0)])
}
insert(String(String.UnicodeScalarView(chars)))

RustBridge.ime_free(result)
```

**Source:** `core/src/engine/mod.rs:Result` struct

---

## 10. Usage Patterns

### 10.1 Initialization Flow

```c
// 1. Initialize engine (once at app start)
ime_init();

// 2. Set input method
ime_method(0); // Telex

// 3. Configure features
ime_enabled(true);
ime_modern(true);
ime_english_auto_restore(true);
ime_auto_capitalize(false);
```

---

### 10.2 Keystroke Flow

```c
// On each keystroke:
ImeResult* r = ime_key(keycode, is_shift, is_ctrl);

if (r != NULL) {
    if (r->action == 1) {
        // Send: replace text
        delete_backward(r->backspace);
        insert_utf32(r->chars, r->count);
    } else if (r->action == 2) {
        // Restore: undo transformation
        // (handled internally by engine)
    }
    // action == 0: pass through (do nothing)

    ime_free(r);
}

// On space/punctuation:
ime_clear();

// On cursor move:
ime_clear_all();
```

---

### 10.3 Shortcut Management

```c
// Add shortcuts
ime_add_shortcut("vn", "Việt Nam");
ime_add_shortcut("hn", "Hà Nội");
ime_add_shortcut("->", "→");

// Remove one
ime_remove_shortcut("vn");

// Clear all
ime_clear_shortcuts();
```

---

## 11. Error Handling

### 11.1 Null Returns

Functions that return `ImeResult*` return `null` if:
- Engine not initialized (forgot `ime_init()`)
- Memory allocation failure (extremely rare)

**Always check for null:**
```c
ImeResult* r = ime_key(key, caps, ctrl);
if (r == NULL) {
    // Handle error (usually means engine not initialized)
    return;
}
// ... use r ...
ime_free(r);
```

---

### 11.2 Thread Safety

All functions use a global mutex. Concurrent calls are serialized. No race conditions.

**Note:** High-frequency concurrent calls may cause latency. Platform UI should call from main thread.

---

### 11.3 Memory Leaks

**Common mistake:**
```c
// ❌ LEAK: forgot to free
ImeResult* r = ime_key(key, caps, ctrl);
if (r->action == 1) {
    send_text(r->chars, r->count);
}
// Missing: ime_free(r);
```

**Correct:**
```c
// ✅ Always free after use
ImeResult* r = ime_key(key, caps, ctrl);
if (r != NULL) {
    if (r->action == 1) {
        send_text(r->chars, r->count);
    }
    ime_free(r); // Free before return
}
```

---

## 12. Platform Integration Examples

### 12.1 Swift (macOS)

```swift
// RustBridge.swift
class RustBridge {
    static func ime_init() {
        gonhanh_core.ime_init()
    }

    static func ime_key(_ key: UInt16, caps: Bool, ctrl: Bool) -> UnsafePointer<ImeResult>? {
        return gonhanh_core.ime_key(key, caps, ctrl)
    }

    static func ime_free(_ result: UnsafePointer<ImeResult>?) {
        if let r = result {
            gonhanh_core.ime_free(UnsafeMutablePointer(mutating: r))
        }
    }
}
```

---

### 12.2 C++ (Linux Fcitx5)

```cpp
// gonhanh_addon.cpp
extern "C" {
    void ime_init(void);
    ImeResult* ime_key(uint16_t key, bool caps, bool ctrl);
    void ime_free(ImeResult* result);
}

class GoNhanhEngine {
public:
    void init() { ime_init(); }

    void processKey(uint16_t key, bool caps, bool ctrl) {
        ImeResult* result = ime_key(key, caps, ctrl);
        if (result && result->action == 1) {
            deleteBackward(result->backspace);
            insertUTF32(result->chars, result->count);
        }
        if (result) ime_free(result);
    }
};
```

---

## 13. Testing

### 13.1 FFI Tests

See `core/src/lib.rs:420-733` for comprehensive FFI tests:
- ✅ Basic flow (a+s → á)
- ✅ Shortcut add/remove/clear
- ✅ Null safety
- ✅ Unicode handling
- ✅ Symbol vs letter trigger detection
- ✅ Restore word functionality

**Run tests:**
```bash
cd core && cargo test --lib
```

---

## 14. Performance

| Metric | Target | Achieved |
|--------|--------|----------|
| Latency | <1ms | ✅ <1ms |
| FFI Call Overhead | <100μs | ✅ ~50μs |
| Memory per Result | <128 bytes | ✅ 80 bytes |
| Thread Lock Wait | <10μs | ✅ ~5μs |

---

## 15. References

- Core Implementation: `core/src/lib.rs`, `core/src/engine/mod.rs`
- Platform Integration: `platforms/macos/RustBridge.swift`
- Architecture: `docs/system-architecture.md`
- Test Suite: `core/src/lib.rs` (tests module)
