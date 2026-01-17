# Tech Stack

> Auto-generated from codebase on 2026-01-12

## Project Architecture

**Type:** Cross-platform Desktop Application (Vietnamese IME)

**Components:**
| Component | Path | Type |
|-----------|------|------|
| Core Engine | `core/` | Rust Library (FFI exports) |
| macOS UI | `platforms/macos/` | SwiftUI |
| Linux Integration | `platforms/linux/` | Fcitx5 + C++ |
| Windows UI | (planned) | WPF/.NET 8 |

---

## Core Engine (`core/`)

| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| Language | **Rust** | 2021 edition | Core IME logic |
| Dependencies | **std only** | - | Zero runtime deps |
| Build Type | `staticlib`, `cdylib`, `rlib` | - | FFI + testing |
| Testing | `rstest`, `serial_test` | - | 600+ tests |
| Optimization | LTO, strip, size opt | - | <1ms latency, ~5MB RAM |

**Key Modules:**
- `src/engine/` - Main processing pipeline (7 stages)
- `src/data/` - Linguistic data (validation matrices)
- `src/input/` - Telex/VNI input methods
- `src/lib.rs` - FFI interface (C-compatible exports)

---

## macOS Platform (`platforms/macos/`)

| Component | Technology | Purpose |
|-----------|------------|---------|
| UI Framework | **SwiftUI** | Native UI (settings, about, onboarding) |
| Keyboard Hook | **CGEventTap** | Intercept keystrokes system-wide |
| FFI Bridge | **C bridging** | Call Rust core functions |
| Update System | Custom update checker | Auto-update via GitHub releases |
| Permissions | **Accessibility API** | Required for keyboard monitoring |
| Launch Agent | **ServiceManagement** | Auto-start on login |

**Key Files:**
- `App.swift` - App entry point, lifecycle
- `MenuBar.swift` - Menu bar controller, notifications
- `MainSettingsView.swift` - Settings UI (700+ lines)
- `RustBridge.swift` - FFI calls to Rust core
- `InputSourceManager.swift` - Detect input source changes
- `SpecialPanelAppDetector.swift` - Handle special apps (Spotlight, etc.)

---

## Linux Platform (`platforms/linux/`)

| Component | Technology | Purpose |
|-----------|------------|---------|
| IME Framework | **Fcitx5** | Linux input method framework |
| Language | **C++** | Fcitx5 plugin API |
| FFI Bridge | **C bridging** | Call Rust core functions |
| Build System | **CMake** | Compile and install |

---

## Development Tools

| Tool | Purpose |
|------|---------|
| **Makefile** | Build automation (test, format, build, release) |
| **GitHub Actions** | CI/CD (test on PR, auto-release) |
| **cargo fmt** | Code formatting |
| **cargo clippy** | Linting |
| **Claude Code** | Primary development tool |
| **ClaudeKit** | Development workflows |

---

## Build & Distribution

| Platform | Build Tool | Package Format | Distribution |
|----------|------------|----------------|--------------|
| **macOS** | Xcode + cargo | `.dmg` + `.app` | GitHub Releases + Homebrew |
| **Linux** | CMake + cargo | Fcitx5 addon | Manual install |
| **Windows** | (planned) | `.exe` installer | GitHub Releases |

---

## Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Anti-over-engineering** | No unnecessary abstractions, inline code when used once |
| **Performance-first** | Target: <1ms latency, <10MB RAM, no allocation in hot path |
| **Zero dependency** | Core uses only Rust `std`, no external crates |
| **Validation-first** | Reject invalid input early before transformation |
| **Platform-agnostic core** | Pure Rust core, OS-specific code only in `platforms/` |

---

## FFI Interface (C ABI)

**Exported Functions:**
```c
void ime_init(void);                              // Initialize engine
ImeResult* ime_key(u16 key, bool caps, bool ctrl); // Process keystroke
void ime_method(u8 method);                       // 0=Telex, 1=VNI
void ime_enabled(bool enabled);                   // Toggle on/off
void ime_clear(void);                             // Clear buffer
void ime_free(ImeResult* result);                 // Free result
// + 10 more configuration functions
```

**Result Struct:**
```c
struct ImeResult {
    u8 action;          // 0=None, 1=Send, 2=Restore
    u8 backspace;       // Chars to delete
    u32 chars[16];      // UTF-32 codepoints to insert
    i64 count;          // Number of valid chars
}
```

---

## Performance Targets

| Metric | Target | Achieved |
|--------|--------|----------|
| Latency | <1ms | ✅ <1ms |
| Memory | <10MB | ✅ ~5MB |
| Binary Size | <5MB (core) | ✅ ~2MB |
| Test Coverage | 100% (critical paths) | ✅ 600+ tests |
| Startup Time | <100ms | ✅ ~50ms |

---

## References

- [System Architecture](./system-architecture.md) - FFI flow, data structures
- [Core Engine Algorithm](./core-engine-algorithm.md) - Processing pipeline
- [Validation Algorithm](./validation-algorithm.md) - Vietnamese syllable rules
- [Vietnamese Language System](./vietnamese-language-system.md) - Linguistic foundation
