# Gõ Nhanh trên Windows

**Status**: Phase 1 Complete - Core Integration Ready

## Cài đặt & Sử dụng

### Phiên bản hoàn thiện (v1.0.0+)

> Tới sắp - Installer sẽ khả dụng tại [Releases](https://github.com/khaphanspace/gonhanh.org/releases)

**Yêu cầu hệ thống:**
- Windows 10 Build 19041+ hoặc Windows 11
- .NET 8 Runtime (tự động với installer)
- ~50MB disk space

### Build từ Source (Developer)

Xem chi tiết tại [Developer Build Guide](#build-từ-source-windows).

---

## Tính năng & Hỗ trợ

### Cốt lõi (Phase 1)
- ✅ Telex gõ tiếng Việt
- ✅ VNI input method
- ✅ Global keyboard hook (SetWindowsHookEx WH_KEYBOARD_LL)
- ✅ Gõ tắt (shortcut expansion)
- ✅ UTF-32 ↔ UTF-16 conversion

### Sắp tới (Phase 2+)
- 📋 System tray menu & status indicator
- 📋 Settings dialog
- 📋 Auto-restore tiếng Anh
- 📋 Per-app enable/disable
- 📋 Auto-capitalization

---

## Quy tắc gõ

### Telex

| Gõ | Kết quả | Mô tả |
|----|---------|-------|
| `as`, `af`, `ar`, `ax`, `aj` | á, à, ả, ã, ạ | Dấu thanh |
| `aa`, `aw` | â, ă | Dấu mũ/breve |
| `ee`, `oo` | ê, ô | Dấu mũ |
| `ow`, `uw` | ơ, ư | Dấu móc |
| `dd` | đ | Gạch ngang |

### VNI

| Gõ | Kết quả | Mô tả |
|----|---------|-------|
| `a1`-`a5` | á, à, ả, ã, ạ | Dấu thanh (1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng) |
| `a6`, `a8` | â, ă | Dấu mũ (6) / breve (8) |
| `o6`, `e6` | ô, ê | Dấu mũ |
| `o7`, `u7` | ơ, ư | Dấu móc (7) |
| `d9` | đ | Gạch ngang (9) |

**Chế độ mặc định:** Telex (chuyển qua Settings → Input Method)

---

## Architecture & Kỹ thuật

### Platform Integration (Phase 1)

**Technology Stack:**
| Layer | Công nghệ | Chi tiết |
|-------|-----------|---------|
| **Build** | CMake 3.22+ + Corrosion v0.5 | Rust ↔ C++20 integration |
| **UI** | WPF + .NET 8 (planned) | System tray icon + menu |
| **Keyboard Hook** | SetWindowsHookEx (WH_KEYBOARD_LL) | Global low-level hook |
| **FFI** | C++ with RAII guards | UTF-32 ↔ UTF-16 conversion |
| **Core Engine** | Rust (static lib) | Zero dependencies, <150KB |
| **Runtime** | MSVC CRT (static link) | Zero external DLLs |

### Quy trình xử lý (Data Flow)

```
User types key
    ↓
SetWindowsHookEx KeyboardHookProc fires
    ↓
Extract VK code + modifier flags
    ↓
Call Rust: ime_key(vkCode, caps, ctrl)
    ↓
Engine: 7-stage validation pipeline
    ├─ 1. Stroke (đ/Đ)
    ├─ 2. Tone marks (sắc/huyền...)
    ├─ 3. Vowel marks (circumflex/horn...)
    ├─ 4. Mark removal (revert)
    ├─ 5. W-vowel (Telex)
    ├─ 6. Normal letter
    └─ 7. Shortcut expansion
    ↓
Return ImeResult { chars[], action, backspace, count }
    ↓
C++ Bridge: Utf32ToUtf16(chars)
    ↓
If transformation:
    ├─ Send backspace count
    ├─ Send UTF-16 replacement text
    └─ Consume key (return 1)
    ↓
Else: Pass through (return 0)
    ↓
Output visible to user
```

**Latency Target:** <1ms (Rust engine: ~100-200μs)

---

## Troubleshooting

### Build Issues

**Lỗi: "Corrosion not found"**
```powershell
# Đảm bảo CMake >= 3.22
cmake --version

# Clean build
rm -r build
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release
```

**Lỗi: "MSVC runtime mismatch"**
```powershell
# Cargo.toml đã set static CRT
# Verify in CMakeLists.txt:
# set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded")
```

**Lỗi: "Rust compiler not found"**
```powershell
rustup install stable-msvc
rustup default stable-msvc
```

### Runtime Issues

**Hook không hoạt động:**
- Kiểm tra: admin privileges (SetWindowsHookEx yêu cầu)
- Khởi động lại ứng dụng
- Xem Event Viewer → Windows Logs → Application

**Text không hiển thị:**
- Verify UTF-16 conversion: Check OutputDebugString logs
- Test: Notepad (app không hỗ trợ Unicode hook)

---

## Theo dõi

- [Project Releases](https://github.com/khaphanspace/gonhanh.org/releases)
- [GitHub Issues](https://github.com/khaphanspace/gonhanh.org/issues)
- [Windows Platform Branch](https://github.com/khaphanspace/gonhanh.org/tree/platforms/windows)

---

## Build từ Source (Windows)

### Prerequisites

1. **Windows 10/11** (Build 19041+)
2. **Visual Studio 2022** Community+ with C++ workload
   ```powershell
   # Install via Visual Studio Installer
   # → Workloads: C++ Desktop Development
   # → Individual components: MSVC v143, Windows 11 SDK
   ```
3. **Rust** (MSVC toolchain)
   ```powershell
   rustup install stable-msvc
   rustup default stable-msvc
   ```
4. **CMake** 3.22+
   ```powershell
   cmake --version  # Should be >= 3.22
   ```

### Clone & Build

```powershell
# Clone repository
git clone https://github.com/khaphanspace/gonhanh.org.git
cd gonhanh.org

# Configure (first time)
cmake -B build -DCMAKE_BUILD_TYPE=Release -DCMAKE_GENERATOR="Visual Studio 17 2022"

# Build
cmake --build build --config Release --parallel

# Output
# → build/Release/gonhanh.exe (size ~5-8MB)

# Test
.\build\Release\gonhanh.exe
```

### Build Configuration

**Optimization Flags (CMakeLists.txt):**
- `/O1` - Size optimization
- `/GL` - Whole program optimization
- `/GS-` - Disable security checks (performance)
- `/LTCG` - Link-time code generation
- `/OPT:REF` - Remove unused references
- `/OPT:ICF` - Inline function folding

**Rust Profile (Cargo.toml):**
```toml
[profile.release]
opt-level = "z"          # Optimize for size
lto = true               # Link-time optimization
codegen-units = 1        # Better optimization
strip = true             # Strip symbols
panic = "abort"          # Smaller binary
```

**Result:** ~150KB Rust static library + ~50KB C++ binary

### FFI Architecture

**Memory Safety (RAII):**
```cpp
// Automatic cleanup via scope
{
    ImeResultGuard result(ime_key(vkCode, caps, ctrl));
    // Process result...
}  // ime_free() called automatically
```

**Buffer Guarantee:**
- Rust const: `MAX_BUFFER_SIZE = 256`
- C++ struct: `uint32_t chars[256]`
- Sync: Both verified during build

**UTF Conversion:**
```cpp
// UTF-32 (Rust) → UTF-16 (Windows)
std::wstring text = gonhanh::Utf32ToUtf16(result->chars, result->count);
// Handles: BMP chars + surrogate pairs
```

### Verify Build

```powershell
# Run test exe
.\build\Release\gonhanh.exe

# Check debug output
# → "FFI OK: ime_key returned valid result"
# → "Converted text: [output]"
# → MessageBox: "Gõ Nhanh - FFI Test OK"

# File info
.\build\Release\gonhanh.exe | fc /B  # Check binary size
```

---

**Last Updated:** 2026-01-12 | **Phase:** Core Integration (1/3)
