# System Requirement Definition (SRD)

> ⚠️ **Auto-generated from existing codebase** on 2026-01-12
> This SRD was inferred from code. Review and enhance with business context.

---

## 1. Overview

### 1.1 Purpose
Vietnamese input method engine (IME) for macOS/Linux/Windows providing fast, accurate Vietnamese text input with <1ms latency and <10MB RAM usage.

### 1.2 Scope
- **In Scope:** Telex/VNI input methods, auto-restore English words, text shortcuts, per-app modes, modern tone placement
- **Out of Scope:** Cloud sync, mobile platforms, handwriting input, voice input

### 1.3 Target Users
- Vietnamese speakers typing on macOS, Linux, or Windows
- Developers, writers, students, office workers
- Users frustrated with existing IME bugs (Chrome, Spotlight, Arc, Claude Code, JetBrains)

---

## 2. Core Entities

| ID | Entity | Source | Description |
|----|--------|--------|-------------|
| **E-01** | **Engine** | `core/src/engine/mod.rs` | Core IME processing engine with 7-stage pipeline |
| **E-02** | **Buffer** | `core/src/engine/buffer.rs` | Circular input buffer storing keystrokes and transformations |
| **E-03** | **Syllable** | `core/src/engine/syllable.rs` | Vietnamese syllable parser (onset, nucleus, coda, tone) |
| **E-04** | **Transform** | `core/src/engine/transform.rs` | Character transformation logic (marks, tones, strokes) |
| **E-05** | **Validation** | `core/src/engine/validation.rs` | Vietnamese spelling validation (5 rules) |
| **E-06** | **Shortcut** | `core/src/engine/shortcut.rs` | Text expansion system (trigger → replacement) |
| **E-07** | **Settings** | `platforms/macos/MainSettingsView.swift:60-375` | App configuration state |
| **E-08** | **KeyboardHook** | `platforms/macos/` (inferred from CGEventTap) | System-wide keyboard event interceptor |

---

## 3. Features

### 3.1 Core Input Features

| ID | Feature | Source | Status | Description |
|----|---------|--------|--------|-------------|
| **FR-01** | **Telex Input** | `core/src/input/telex.rs` | ✅ Implemented | Telex method (s=sắc, f=huyền, w=ư, etc.) |
| **FR-02** | **VNI Input** | `core/src/input/vni.rs` | ✅ Implemented | VNI method (1-5=tones, 6-8=marks) |
| **FR-03** | **Tone Placement** | `core/src/engine/transform.rs` | ✅ Implemented | Auto tone placement (modern/traditional) |
| **FR-04** | **Mark Removal** | `core/src/engine/mod.rs` (stage 4) | ✅ Implemented | Double-key removes mark (aa→a, ss→s) |
| **FR-05** | **Validation** | `core/src/engine/validation.rs` | ✅ Implemented | Reject invalid Vietnamese syllables |

### 3.2 Auto-Restore Features

| ID | Feature | Source | Status | Description |
|----|---------|--------|--------|-------------|
| **FR-06** | **English Auto-Restore** | `core/src/engine/mod.rs:should_auto_restore` | ✅ Implemented | Auto-restore English words (text, expect, user) on Space |
| **FR-07** | **ESC Restore** | `core/src/engine/mod.rs:on_key` (ESC key) | ✅ Implemented | Press ESC to restore raw ASCII input |
| **FR-08** | **Backspace Restore** | `core/src/engine/mod.rs:ime_restore_word` | ✅ Implemented | Restore word when backspacing into it |

### 3.3 Text Expansion

| ID | Feature | Source | Status | Description |
|----|---------|--------|--------|-------------|
| **FR-09** | **Shortcut System** | `core/src/engine/shortcut.rs` | ✅ Implemented | Trigger-based text expansion (vn→Việt Nam) |
| **FR-10** | **Symbol Shortcuts** | `core/src/engine/shortcut.rs:immediate` | ✅ Implemented | Immediate triggers for symbols (→, =>) |
| **FR-11** | **Shortcut Import/Export** | `MainSettingsView.swift:329-356` | ✅ Implemented | Import/export shortcuts from .txt files |

### 3.4 App Integration

| ID | Feature | Source | Status | Description |
|----|---------|--------|--------|-------------|
| **FR-12** | **Per-App Mode** | `MainSettingsView.swift:300-312` | ✅ Implemented | Remember on/off state per app (VS Code=off, Slack=on) |
| **FR-13** | **Special Panel Detection** | `SpecialPanelAppDetector.swift` | ✅ Implemented | Handle Spotlight, Raycast, Alfred, Arc panels |
| **FR-14** | **Input Source Tracking** | `InputSourceManager.swift` | ✅ Implemented | Auto-disable when switching to JP/KR/CN input |
| **FR-15** | **Keyboard Hook** | `platforms/macos/` (CGEventTap) | ✅ Implemented | System-wide keyboard interception |

### 3.5 User Experience

| ID | Feature | Source | Status | Description |
|----|---------|--------|--------|-------------|
| **FR-16** | **Toggle Shortcut** | `MainSettingsView.swift:142-147` | ✅ Implemented | Global hotkey to toggle Vietnamese on/off |
| **FR-17** | **Sound Feedback** | `MainSettingsView.swift:7-33` | ✅ Implemented | Play sound when toggling (Tink/Pop) |
| **FR-18** | **Auto-Capitalize** | `core/src/lib.rs:ime_auto_capitalize` | ✅ Implemented | Capitalize after sentence-ending punctuation |
| **FR-19** | **Launch at Login** | `MainSettingsView.swift:228-298` | ✅ Implemented | Auto-start on system boot |
| **FR-20** | **Auto-Update** | `UpdateChecker.swift`, `UpdateManager.swift` | ✅ Implemented | Check for updates every 24h, download/install |

---

## 4. UI Screens (macOS)

| ID | Screen | File | Description |
|----|--------|------|-------------|
| **S-01** | **Menu Bar** | `MenuBar.swift` | Status bar menu with on/off, method, settings |
| **S-02** | **Onboarding** | `OnboardingView.swift` | First-run guide (permissions, features) |
| **S-03** | **Settings** | `MainSettingsView.swift` | Main settings window (2 tabs) |
| **S-04** | **Settings Page** | `MainSettingsView.swift:681-772` | Configure input method, shortcuts, toggles |
| **S-05** | **Shortcuts Sheet** | `MainSettingsView.swift:776-996` | Manage text expansion shortcuts (table + form) |
| **S-06** | **About Page** | `MainSettingsView.swift:1000-1028` | App info, version, links (GitHub, sponsor) |
| **S-07** | **Update Window** | `UpdateView.swift` | Download/install updates |

---

## 5. FFI API (Rust Core ↔ Platform UI)

### 5.1 Initialization

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `ime_init()` | - | void | Initialize engine (call once) |
| `ime_method(u8)` | 0=Telex, 1=VNI | void | Set input method |
| `ime_enabled(bool)` | enabled | void | Enable/disable engine |

### 5.2 Keystroke Processing

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `ime_key(u16, bool, bool)` | keycode, caps, ctrl | ImeResult* | Process keystroke |
| `ime_key_ext(u16, bool, bool, bool)` | keycode, caps, ctrl, shift | ImeResult* | Extended processing (VNI symbols) |
| `ime_clear()` | - | void | Clear buffer (word boundary) |
| `ime_clear_all()` | - | void | Clear buffer + history (cursor move) |

### 5.3 Configuration

| Function | Parameters | Description |
|----------|------------|-------------|
| `ime_skip_w_shortcut(bool)` | Skip w→ư at word start |
| `ime_bracket_shortcut(bool)` | Enable ]→ư, [→ơ |
| `ime_esc_restore(bool)` | Enable ESC restore |
| `ime_modern(bool)` | Modern tone placement (hoà vs hòa) |
| `ime_english_auto_restore(bool)` | Enable English auto-restore |
| `ime_auto_capitalize(bool)` | Auto-capitalize after punctuation |

### 5.4 Text Expansion

| Function | Parameters | Description |
|----------|------------|-------------|
| `ime_add_shortcut(char*, char*)` | trigger, replacement | Add shortcut |
| `ime_remove_shortcut(char*)` | trigger | Remove shortcut |
| `ime_clear_shortcuts()` | - | Clear all shortcuts |

### 5.5 Result Structure

```c
struct ImeResult {
    u8 action;          // 0=None (pass through), 1=Send (replace), 2=Restore
    u8 backspace;       // Number of chars to delete
    u32 chars[16];      // UTF-32 codepoints to insert
    i64 count;          // Number of valid chars
}
```

---

## 6. Business Rules

| ID | Rule | Source | Description |
|----|------|--------|-------------|
| **BR-01** | **Validation First** | `core/src/engine/validation.rs` | Validate syllable before accepting transformation |
| **BR-02** | **English Detection** | `core/src/engine/mod.rs:should_auto_restore` | Restore if buffer invalid VN + raw valid EN |
| **BR-03** | **Stroke Priority** | Auto-restore logic | Never restore if word contains đ/Đ (indicates intentional VN) |
| **BR-04** | **Double-Key Removal** | `core/src/engine/mod.rs:stage4` | Typing mark key twice removes mark |
| **BR-05** | **Per-App Persistence** | `MainSettingsView.swift:300-312` | Save/load enabled state per bundle ID |
| **BR-06** | **Shortcut Triggers** | `core/src/engine/shortcut.rs` | Word boundary for letters, immediate for symbols |
| **BR-07** | **Update Frequency** | `UpdateChecker.swift` | Check for updates every 24h |
| **BR-08** | **Memory Safety** | FFI interface | All FFI functions null-safe, use mutex for thread safety |

---

## 7. Performance Requirements

| ID | Requirement | Target | Source |
|----|-------------|--------|--------|
| **NFR-01** | **Latency** | <1ms | CLAUDE.md, README.md |
| **NFR-02** | **Memory** | <10MB RAM | CLAUDE.md, README.md |
| **NFR-03** | **Binary Size** | <5MB (core) | core/Cargo.toml (opt-level=z) |
| **NFR-04** | **Test Coverage** | 100% (critical paths) | core/ (600+ tests) |
| **NFR-05** | **Startup Time** | <100ms | (inferred from performance goals) |

---

## 8. Quality Requirements

| ID | Requirement | Implementation |
|----|-------------|----------------|
| **NFR-06** | **Zero Deps** | Core uses only Rust std library |
| **NFR-07** | **Cross-Platform** | Platform-agnostic Rust core + native UI layers |
| **NFR-08** | **Privacy** | 100% offline, no tracking, no network calls (except updates) |
| **NFR-09** | **Accessibility** | Requires macOS Accessibility permission |
| **NFR-10** | **App Compatibility** | Works in all apps (Chrome, VS Code, Terminal, etc.) |

---

## 9. IPA Compliance Checklist

- [x] **Entities** - Extracted from core engine modules
- [x] **Features** - Inferred from FFI interface + UI code
- [x] **Screens** - Extracted from SwiftUI files
- [x] **API Spec** - FFI interface documented (see Section 5)
- [ ] **Business Context** - Needs manual addition (why these features?)
- [ ] **User Journeys** - Needs manual addition (typical workflows)
- [ ] **Test Scenarios** - Partially covered (600+ unit tests, need E2E)
- [ ] **Error Handling** - Partially documented (needs expansion)

---

## 10. Gaps & Next Steps

### Documentation Gaps
- [ ] Add user journey maps (onboarding → daily use → troubleshooting)
- [ ] Document error handling strategy (FFI failures, permission denials)
- [ ] Add E2E test scenarios (vs current unit tests)
- [ ] Document release process (versioning, changelog generation)

### Feature Context Missing
- [ ] Why these 12 English auto-restore patterns? (FR-06)
- [ ] Why default shortcuts (vn, hn, hcm)? User research?
- [ ] Per-app mode: Which apps are most problematic? User feedback?
- [ ] Update strategy: Why 24h check interval vs on-demand?

### Business Rules Needing Clarification
- [ ] BR-02 (English detection): How was algorithm validated? False positive rate?
- [ ] BR-07 (Update frequency): User preference vs hardcoded?
- [ ] Shortcut limit: Max number of shortcuts? Performance impact?

---

## 11. References

- Technical Docs: `docs/system-architecture.md`, `docs/core-engine-algorithm.md`
- Validation Rules: `docs/validation-algorithm.md`
- Linguistic Foundation: `docs/vietnamese-language-system.md`
- User-Facing Docs: `README.md`, `docs/install-*.md`
