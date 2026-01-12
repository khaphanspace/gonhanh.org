# Tech Stack

> Auto-generated from codebase analysis.

## Architecture Type

**Multi-Platform Native App** with shared core engine.

```
┌─────────────────────────────────────────────────────────────────┐
│ Platform UI (SwiftUI/WPF/Fcitx5) → FFI Bridge → Rust Core      │
└─────────────────────────────────────────────────────────────────┘
```

## Project Structure

| Component | Path | Type | Status |
|-----------|------|------|--------|
| Core Engine | `core/` | Rust Library | ✅ Production |
| macOS App | `platforms/macos/` | SwiftUI | ✅ Production |
| Windows App | `platforms/windows/` | WPF/.NET 8 | 🧪 Beta |
| Linux IME | `platforms/linux/` | Fcitx5 + C++ | 🧪 Beta |

## Core Engine (`core/`)

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | 2021 Edition |
| Dependencies | None (pure `std`) | - |
| Build Output | staticlib, cdylib, rlib | - |
| Testing | rstest + serial_test | 0.18 / 3.0 |

### Core Modules

| Module | Path | Description |
|--------|------|-------------|
| Engine | `src/engine/` | 7-stage processing pipeline |
| Buffer | `src/engine/buffer.rs` | Circular input buffer (32 chars) |
| Syllable | `src/engine/syllable.rs` | Vietnamese syllable parsing |
| Transform | `src/engine/transform.rs` | Character transformations |
| Validation | `src/engine/validation.rs` | 6 validation rules |
| Shortcut | `src/engine/shortcut.rs` | Text abbreviation expansion |
| Data | `src/data/` | Vowel tables, constants, keycodes |
| Input | `src/input/` | Telex/VNI input methods |
| FFI | `src/lib.rs` | C ABI exports |

## macOS Platform (`platforms/macos/`)

| Component | Technology |
|-----------|------------|
| UI Framework | SwiftUI |
| Language | Swift |
| Keyboard Hook | CGEventTap |
| FFI Bridge | C ABI + Swift |
| Build System | Xcode |
| Min Deployment | macOS 13+ |

### macOS Modules

| File | Description |
|------|-------------|
| `App.swift` | Application entry point |
| `MenuBar.swift` | Status bar menu controller |
| `MainSettingsView.swift` | Settings UI (1000+ lines) |
| `OnboardingView.swift` | First-run wizard |
| `AboutView.swift` | About dialog |
| `RustBridge.swift` | FFI wrapper for Rust core |
| `InputSourceManager.swift` | macOS input source integration |
| `SpecialPanelAppDetector.swift` | Spotlight/Raycast detection |
| `UpdateChecker.swift` | Auto-update logic |

## Windows Platform (`platforms/windows/`)

| Component | Technology |
|-----------|------------|
| UI Framework | WPF |
| Language | C#/.NET 8 |
| Keyboard Hook | SetWindowsHookEx |
| FFI Bridge | P/Invoke |
| Build System | MSBuild |

> **Note:** Windows implementation in beta, see `platforms/windows/README.md`

## Linux Platform (`platforms/linux/`)

| Component | Technology |
|-----------|------------|
| IME Framework | Fcitx5 |
| Language | C++ |
| FFI Bridge | C ABI |
| Build System | CMake |

### Linux Modules

| File | Description |
|------|-------------|
| `src/Engine.cpp` | Fcitx5 engine adapter |
| `src/RustBridge.cpp` | FFI wrapper |
| `src/KeycodeMap.h` | X11/Wayland keycode mapping |

## CI/CD & Tooling

| Tool | Purpose |
|------|---------|
| GitHub Actions | CI/CD pipeline |
| semantic-release | Auto versioning |
| Homebrew | macOS distribution |
| Claude Code + ClaudeKit | AI-assisted development |

## Performance Targets

| Metric | Target | Actual |
|--------|--------|--------|
| Latency | <1ms | ~0.3-0.5ms |
| Memory | <10MB | ~5MB |
| Test Count | - | 600+ tests |

## Design Principles

1. **Zero Dependencies** - Core uses only Rust `std`
2. **Validation-First** - Reject invalid input before transform
3. **Platform-Agnostic Core** - No OS code in `core/`
4. **Performance-First** - No allocation in hot path
5. **Anti-Over-Engineering** - Inline code when used once
