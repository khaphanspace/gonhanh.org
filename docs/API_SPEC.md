# API Specification

> Auto-generated from FFI interface in `core/src/lib.rs`.

**Type:** FFI (Foreign Function Interface) - C ABI
**Source:** `core/src/lib.rs`

---

## 1. Overview

Gõ Nhanh exposes a C ABI interface for platform integration. The core engine is written in Rust and compiled as a static/dynamic library.

### 1.1 Library Outputs

| Type | File | Usage |
|------|------|-------|
| staticlib | `libgonhanh_core.a` | macOS, Linux |
| cdylib | `libgonhanh_core.dylib/dll/so` | Dynamic linking |
| rlib | Rust library | Testing |

---

## 2. Data Types

### 2.1 ImeResult

```c
typedef struct {
    uint32_t chars[32];      // UTF-32 output characters
    uint8_t action;          // 0=None, 1=Send, 2=Restore
    uint8_t backspace;       // Number of chars to delete
    uint8_t count;           // Number of valid chars
    uint8_t _pad;            // Padding for alignment
} ImeResult;
```

### 2.2 Action Enum

| Value | Name | Description |
|-------|------|-------------|
| 0 | None | Pass key through, no transformation |
| 1 | Send | Replace text: backspace + chars |
| 2 | Restore | Restore raw input (ESC) |

### 2.3 Method Enum

| Value | Name |
|-------|------|
| 0 | Telex |
| 1 | VNI |

---

## 3. Endpoint Matrix

### 3.1 Core Functions

| Function | Signature | Description | Status |
|----------|-----------|-------------|--------|
| `ime_init` | `void ime_init(void)` | Initialize engine (call once) | 🔄 |
| `ime_key` | `ImeResult* ime_key(uint16_t, bool, bool)` | Process keystroke | 🔄 |
| `ime_key_ext` | `ImeResult* ime_key_ext(uint16_t, bool, bool, bool)` | Process with shift flag | 🔄 |
| `ime_free` | `void ime_free(ImeResult*)` | Free result memory | 🔄 |

### 3.2 Configuration Functions

| Function | Signature | Description | Status |
|----------|-----------|-------------|--------|
| `ime_method` | `void ime_method(uint8_t)` | Set input method (0=Telex, 1=VNI) | 🔄 |
| `ime_enabled` | `void ime_enabled(bool)` | Enable/disable engine | 🔄 |
| `ime_clear` | `void ime_clear(void)` | Clear buffer (word boundary) | 🔄 |
| `ime_clear_all` | `void ime_clear_all(void)` | Clear all including history | 🔄 |

### 3.3 Feature Toggles

| Function | Signature | Description | Default | Status |
|----------|-----------|-------------|---------|--------|
| `ime_skip_w_shortcut` | `void ime_skip_w_shortcut(bool)` | Skip w→ư shortcut | false | 🔄 |
| `ime_bracket_shortcut` | `void ime_bracket_shortcut(bool)` | Enable ]→ư, [→ơ | true | 🔄 |
| `ime_esc_restore` | `void ime_esc_restore(bool)` | Enable ESC restore | true | 🔄 |
| `ime_free_tone` | `void ime_free_tone(bool)` | Skip validation | false | 🔄 |
| `ime_modern` | `void ime_modern(bool)` | Modern tone placement | false | 🔄 |
| `ime_english_auto_restore` | `void ime_english_auto_restore(bool)` | Auto-restore English | false | 🔄 |
| `ime_auto_capitalize` | `void ime_auto_capitalize(bool)` | Auto-capitalize sentences | false | 🔄 |

### 3.4 Shortcut Functions

| Function | Signature | Description | Status |
|----------|-----------|-------------|--------|
| `ime_add_shortcut` | `void ime_add_shortcut(char*, char*)` | Add abbreviation | 🔄 |
| `ime_remove_shortcut` | `void ime_remove_shortcut(char*)` | Remove abbreviation | 🔄 |
| `ime_clear_shortcuts` | `void ime_clear_shortcuts(void)` | Clear all shortcuts | 🔄 |

### 3.5 Advanced Functions

| Function | Signature | Description | Status |
|----------|-----------|-------------|--------|
| `ime_get_buffer` | `int64_t ime_get_buffer(uint32_t*, int64_t)` | Get buffer as UTF-32 | 🔄 |
| `ime_restore_word` | `void ime_restore_word(char*)` | Restore word to buffer | 🔄 |

---

## 4. Function Details

### 4.1 ime_init

```c
void ime_init(void);
```

**Description:** Initialize the IME engine. Must be called exactly once before any other `ime_*` functions.

**Thread Safety:** Uses internal mutex.

**Example:**
```c
// At app start
ime_init();
ime_method(0);  // Telex
```

### 4.2 ime_key

```c
ImeResult* ime_key(uint16_t keycode, bool caps, bool ctrl);
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| keycode | uint16_t | macOS virtual keycode (0-127) |
| caps | bool | CapsLock pressed |
| ctrl | bool | Cmd/Ctrl/Alt pressed (bypasses IME) |

**Returns:** Pointer to `ImeResult` (caller must free with `ime_free`)

**Example:**
```c
ImeResult* r = ime_key(0x00, false, false);  // 'a' key
if (r && r->action == 1) {
    // Send r->backspace deletes, then r->chars
}
ime_free(r);
```

### 4.3 ime_key_ext

```c
ImeResult* ime_key_ext(uint16_t keycode, bool caps, bool ctrl, bool shift);
```

**Description:** Extended version with Shift flag. In VNI mode, Shift+number types symbols instead of marks.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| shift | bool | Shift key pressed |

### 4.4 ime_add_shortcut

```c
void ime_add_shortcut(const char* trigger, const char* replacement);
```

**Description:** Add a text abbreviation. Auto-detects trigger type:
- Symbol-only triggers (`->`, `=>`) → immediate trigger
- Letter triggers (`vn`, `hcm`) → word boundary trigger

**Example:**
```c
ime_add_shortcut("vn", "Việt Nam");   // Word boundary
ime_add_shortcut("->", "→");           // Immediate
```

### 4.5 ime_get_buffer

```c
int64_t ime_get_buffer(uint32_t* out, int64_t max_len);
```

**Description:** Get full buffer as UTF-32 for "Select All + Replace" method.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| out | uint32_t* | Output buffer |
| max_len | int64_t | Maximum codepoints to write |

**Returns:** Number of codepoints written.

---

## 5. Usage Flow

### 5.1 Initialization

```
1. App launch
   └── ime_init()

2. Configure
   ├── ime_method(0)           // Telex
   ├── ime_enabled(true)
   ├── ime_english_auto_restore(true)
   └── ime_add_shortcut(...)
```

### 5.2 Keystroke Processing

```
1. Key down event
   └── ime_key(keycode, caps, ctrl)

2. Check result
   ├── action == 0 → pass through
   ├── action == 1 → backspace + insert chars
   └── action == 2 → restore raw input

3. Free result
   └── ime_free(result)
```

### 5.3 Word Boundary

```
1. Space/punctuation/Enter
   └── ime_clear()

2. Cursor move/focus change
   └── ime_clear_all()
```

---

## 6. Platform Integration

### 6.1 macOS (Swift)

```swift
// RustBridge.swift
@_silgen_name("ime_init")
func ime_init()

@_silgen_name("ime_key")
func ime_key(_ key: UInt16, _ caps: Bool, _ ctrl: Bool) -> UnsafeMutablePointer<ImeResult>?
```

### 6.2 Windows (C#)

```csharp
// RustBridge.cs
[DllImport("gonhanh_core")]
public static extern void ime_init();

[DllImport("gonhanh_core")]
public static extern IntPtr ime_key(ushort key, bool caps, bool ctrl);
```

### 6.3 Linux (C++)

```cpp
// RustBridge.h
extern "C" {
    void ime_init();
    ImeResult* ime_key(uint16_t key, bool caps, bool ctrl);
}
```

---

## 7. Memory Management

| Function | Allocation | Deallocation |
|----------|------------|--------------|
| `ime_key` | Rust `Box::into_raw` | `ime_free` required |
| `ime_key_ext` | Rust `Box::into_raw` | `ime_free` required |
| `ime_add_shortcut` | Internal | Automatic |

**Important:** Always call `ime_free()` for each non-null `ime_key` return.

---

## 8. Error Handling

| Scenario | Behavior |
|----------|----------|
| Engine not initialized | `ime_key` returns null |
| Null pointer to `ime_free` | Safe no-op |
| Null pointer to shortcuts | Safe no-op |
| Invalid UTF-8 string | Function returns without action |

---

## Changelog

- **2026-01-11**: Initial generation from `core/src/lib.rs`
