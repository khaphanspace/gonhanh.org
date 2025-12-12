# Gõ Nhanh: Project Overview & Product Development Requirements

## Project Vision

Gõ Nhanh is a **high-performance Vietnamese input method engine** (IME) for macOS and Windows with a platform-agnostic Rust core. It enables fast, accurate Vietnamese text input with minimal system overhead. The project demonstrates production-grade system software design: Rust core for performance and safety, native UI (SwiftUI on macOS, WPF on Windows), and validation-first transformation pipeline for Vietnamese phonology.

## Product Goals

1. **Performance**: Sub-millisecond keystroke latency (<1ms)
2. **Reliability**: Validation-first architecture (phonology rules checked BEFORE transformation)
3. **Cross-Platform**: macOS + Windows + Linux (planned) with consistent core engine
4. **User Experience**: Seamless platform integration (CGEventTap on macOS, SetWindowsHookEx on Windows)
5. **Memory Efficiency**: ~5MB memory footprint with optimized binary packaging

## Target Users

- **Primary**: Vietnamese professionals and students on macOS who type Vietnamese daily
- **Secondary**: Vietnamese diaspora, bilingual professionals
- **Requirement**: macOS 10.15+ with Accessibility permissions enabled

## Core Functional Requirements

### Input Methods
- **Telex**: Vietnamese keyboard layout (VIQR-style: a's → á)
- **VNI**: Alternative numeric layout support
- **Shortcuts**: User-defined abbreviations with priority matching

### Keystroke Processing
1. Buffer management: Maintain context for multi-keystroke transforms
2. Validation: Check syllable against Vietnamese phonology rules
3. Transformation: Apply diacritics (sắc, huyền, hỏi, ngã, nặng) and tone modifiers (circumflex, horn)
4. Output: Send backspace + replacement characters or pass through

### Platform Integration
- CGEventTap keyboard hook intercepts keyDown events system-wide
- Smart text replacement: Backspace method (Terminal) + Selection method (Chrome/Excel)
- Ctrl+Space global hotkey for Vietnamese/English toggle
- Application detection: Specialized handling for autocomplete apps

## Non-Functional Requirements

### Performance
- Keystroke latency: <1ms measured end-to-end
- CPU usage: <2% during normal typing
- Memory: ~5MB resident set size
- No input delay under sustained high-speed typing

### Reliability
- 160+ integration tests covering edge cases
- Validation-first pattern: Reject invalid Vietnamese before transforming
- Graceful fallback: Pass through on disable or invalid input
- Thread-safe global engine instance via Mutex

### Compatibility
- macOS 10.15 Catalina and later
- Apple Silicon (arm64) and Intel (x86_64) universal binaries
- Works with all major applications: Terminal, VS Code, Chrome, Safari, Office

### Security
- No internet access required (offline-first)
- GPL-3.0-or-later license (free and open source)
- Accessibility permission: Required for keyboard hook (transparent to user)
- No telemetry or analytics

## Architecture Overview

```
User Keystroke (CGEventTap/SetWindowsHookEx)
        ↓
   Platform Bridge (RustBridge.swift / RustBridge.cs)
        ↓
   Rust Engine (ime_key) - Validation-First 7-Stage Pipeline
    ├─ Stage 1: Stroke Detection (đ/Đ)
    ├─ Stage 2: Tone Mark Detection (sắc/huyền/hỏi/ngã/nặng)
    ├─ Stage 3: Vowel Mark Detection (circumflex/horn/breve)
    ├─ Stage 4: Mark Removal (revert previous marks)
    ├─ Stage 5: W-Vowel Handling (Telex-specific, "w"→"ư")
    ├─ Stage 6: Normal Letter Processing (pass-through)
    └─ Stage 7: Shortcut Expansion (user-defined abbreviations)
        ↓
   Validation (5 Vietnamese Phonology Rules) - Applied BEFORE Transform
    ├─ Rule 1: Must have vowel
    ├─ Rule 2: Valid initial consonants only
    ├─ Rule 3: All characters parsed
    ├─ Rule 4: Spelling rules (c/k/g restrictions)
    └─ Rule 5: Valid final consonants only
        ↓
   Transform & Output (apply diacritics, tone marks)
        ↓
   Result (action, backspace count, output chars)
        ↓
   Platform UI (SwiftUI on macOS, WPF on Windows - Send text or pass through)
```

## Success Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Keystroke latency | <1ms | ~0.2-0.5ms | ✓ Exceeds |
| Memory usage | <10MB | ~5MB | ✓ Exceeds |
| Test coverage | >90% | 160+ tests, 2100+ lines | ✓ Exceeds |
| macOS compatibility | 10.15+ | 10.15+ universal binary | ✓ Met |
| Code quality | Zero warnings | `cargo clippy -D warnings` | ✓ Met |
| Cross-platform | macOS + Windows | Both production-ready | ✓ Met |

## Roadmap

### Phase 1: macOS (Complete - v1.0.21+)
- Telex + VNI input methods
- Menu bar app with settings
- Auto-launch on login
- Update checker via GitHub releases
- Validation-first architecture
- Shortcut system with priority matching

### Phase 2: Cross-Platform (Partial - Windows Complete)

**Windows 10/11 (Complete)**
- SetWindowsHookEx keyboard hook
- WPF/.NET 8 UI with system tray
- Registry-based settings persistence
- Feature parity with macOS version
- Compiled DLL shared with macOS core

**Linux (Planned)**
- X11/Wayland event hook integration
- Qt or GTK UI
- XDG config for settings persistence
- Feature parity with macOS/Windows

### Phase 3: Enhanced Features (Future)
- Cloud sync for user preferences
- Machine learning for shortcut suggestions
- Dictionary lookup integration
- Advanced diacritics editor
- Mobile support (iOS/Android)

## Development Standards

### Code Organization
- **Core** (`core/src/`): Rust engine, pure logic, zero platform dependencies
- **Platform** (`platforms/macos/`): SwiftUI UI, platform integration, FFI bridge
- **Scripts** (`scripts/`): Build automation for universal binaries
- **Tests** (`core/tests/`): Integration tests + unit tests

### Quality Gates
- Format: `cargo fmt` (automatic formatting)
- Lint: `cargo clippy -- -D warnings` (no warnings allowed)
- Tests: `cargo test` (160+ tests must pass)
- Build: Universal binary creation (arm64 + x86_64)

### Commit Message Format
Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): subject

body

footer
```

Examples:
- `feat(engine): add shortcut expansion for common abbreviations`
- `fix(transform): correct diacritic placement for ư vowel`
- `docs(ffi): update RustBridge interface documentation`
- `test(validation): add edge cases for invalid syllables`

## Dependencies

### Rust
- Zero production dependencies (pure stdlib)
- Dev: `rstest` for parametrized tests

### Swift/macOS
- Foundation: URLSession, UserDefaults, FileHandle
- AppKit: NSApplication, NSStatusBar, CGEventTap
- SwiftUI: Standard UI components, macOS 11+ features

### Build Tools
- `cargo` (Rust toolchain)
- `xcodebuild` (macOS app build)
- GNU Make (build automation)

## Maintenance & Support

### Release Schedule
- Patch releases: Bug fixes and small improvements (monthly)
- Minor releases: New features (quarterly)
- Major releases: Breaking changes (annually or as needed)

### Community
- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: Questions and community support
- Contributing: GPL-3.0 requires contributor agreement

## Success Criteria for Milestones

**v1.0 Release**
- All core input methods working reliably
- Sub-1ms latency confirmed
- 160+ tests passing
- macOS app in official release

**v1.1+ Releases**
- Cross-platform support (Windows/Linux)
- User-customizable shortcuts
- Enhanced documentation
- Community contribution guidelines

---

**Last Updated**: 2025-12-10
**Status**: Active Development
**Platforms**: macOS (v1.0.21+), Windows (production), Linux (planned)
**Repository**: https://github.com/khaphanspace/gonhanh.org
