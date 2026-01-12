# System Requirement Definition (SRD)

> Auto-generated from existing codebase. Review and enhance with business context.

**Project:** Gõ Nhanh - Vietnamese Input Method Engine
**Version:** 1.0.x
**Status:** Production (macOS), Beta (Windows/Linux)

---

## 1. Overview

Gõ Nhanh is a high-performance Vietnamese input method engine for macOS, Windows, and Linux. Uses phonology-based processing (not lookup tables) with <1ms latency.

### 1.1 Vision

Free, fast, stable Vietnamese input for all platforms. No ads. No data collection.

### 1.2 Goals

- G-01: Provide accurate Vietnamese input with Telex/VNI support
- G-02: Auto-restore English words accidentally transformed
- G-03: Per-app mode memory (remember ON/OFF per application)
- G-04: Sub-millisecond latency, <10MB memory footprint
- G-05: Cross-platform support with shared core engine

---

## 2. Entities

| ID | Entity | Source | Description |
|----|--------|--------|-------------|
| E-01 | Syllable | `engine/syllable.rs` | Vietnamese syllable structure (C₁GVC₂) |
| E-02 | Buffer | `engine/buffer.rs` | Circular input buffer (32 chars max) |
| E-03 | Char | `engine/buffer.rs` | Single character with tone/mark/stroke |
| E-04 | Result | `engine/mod.rs` | FFI result struct (action, backspace, chars) |
| E-05 | Shortcut | `engine/shortcut.rs` | Text abbreviation mapping |
| E-06 | AppState | `MainSettingsView.swift` | Application state (macOS) |
| E-07 | Settings | UserDefaults/Registry | User preferences storage |

### 2.1 Entity Details

#### E-01: Syllable

```
Syllable = (C₁)(G)V(C₂)
├── C₁ = Initial consonant (optional, 28 valid)
├── G  = Glide (optional: o, u)
├── V  = Vowel nucleus (required, 12 bases)
└── C₂ = Final consonant (optional, 13 valid)
```

#### E-03: Char

```rust
struct Char {
    key: u16,     // Virtual keycode
    caps: bool,   // Uppercase flag
    tone: u8,     // 0=none, 1=circumflex, 2=horn
    mark: u8,     // 0=none, 1-5=sắc/huyền/hỏi/ngã/nặng
    stroke: bool, // d → đ
}
```

#### E-04: Result

```rust
struct Result {
    chars: [u32; 32],  // UTF-32 codepoints
    action: u8,        // 0=None, 1=Send, 2=Restore
    backspace: u8,     // Characters to delete
    count: u8,         // Valid chars count
}
```

---

## 3. Features

| ID | Feature | Priority | Status | Source |
|----|---------|----------|--------|--------|
| FR-01 | Core Engine Processing | P0 | 🔄 | `engine/mod.rs` |
| FR-02 | Telex Input Method | P0 | 🔄 | `input/telex.rs` |
| FR-03 | VNI Input Method | P0 | 🔄 | `input/vni.rs` |
| FR-04 | Syllable Validation | P0 | 🔄 | `engine/validation.rs` |
| FR-05 | Text Shortcuts | P1 | 🔄 | `engine/shortcut.rs` |
| FR-06 | Auto-Restore English | P1 | 🔄 | `engine/mod.rs` |
| FR-07 | ESC Restore | P1 | 🔄 | `engine/mod.rs` |
| FR-08 | Double-Key Revert | P1 | 🔄 | `engine/mod.rs` |
| FR-09 | Per-App Mode Memory | P2 | 🔄 | `MainSettingsView.swift` |
| FR-10 | Auto-Capitalize | P2 | 🔄 | `engine/mod.rs` |
| FR-11 | Modern Tone Placement | P2 | 🔄 | `engine/transform.rs` |
| FR-12 | W-Vowel Shortcut | P2 | 🔄 | `engine/mod.rs` |
| FR-13 | Bracket Shortcuts | P3 | 🔄 | `engine/mod.rs` |
| FR-14 | Free Tone Mode | P3 | 🔄 | `engine/mod.rs` |
| FR-15 | Auto-Update Check | P2 | 🔄 | `UpdateChecker.swift` |
| FR-16 | Input Source Sync | P2 | 🔄 | `InputSourceManager.swift` |

### 3.1 Feature Details

#### FR-01: Core Engine Processing

7-stage pipeline:
1. Stroke (d → đ)
2. Tone Marks (circumflex/horn/breve)
3. Mark Modifiers (sắc/huyền/hỏi/ngã/nặng)
4. Mark Removal
5. W-Vowel (Telex)
6. Normal Letter
7. Shortcut Expansion

#### FR-04: Syllable Validation

6 validation rules (fail-fast):
1. Has vowel
2. Valid initial consonant
3. All chars parsed
4. Spelling rules (c/k, g/gh, ng/ngh)
5. Valid final consonant
6. Valid vowel pattern

#### FR-06: Auto-Restore English

Patterns detected:
- Modifier + consonant: `text`, `next`, `expect`
- EI + modifier: `their`, `weird`
- W-prefix: `window`, `water`, `write`
- F-prefix: `file`, `fix`, `function`

---

## 4. Screens

| ID | Screen | Route/Entry | Source | Platform |
|----|--------|-------------|--------|----------|
| S-01 | Menu Bar | System tray | `MenuBar.swift` | macOS |
| S-02 | Settings | Menu → Settings | `MainSettingsView.swift` | macOS |
| S-03 | Onboarding | First run | `OnboardingView.swift` | macOS |
| S-04 | About | Menu → About | `AboutView.swift` | macOS |
| S-05 | Update Dialog | Auto/Manual | `UpdateView.swift` | macOS |
| S-06 | System Tray | Taskbar | `TrayIcon.cs` | Windows |
| S-07 | Settings | Tray → Settings | `SettingsWindow.xaml` | Windows |

### 4.1 Screen Components (macOS)

#### S-02: Settings (MainSettingsView)

Sections:
- Input Method selector (Telex/VNI)
- Enable/Disable toggle
- Per-app mode toggle
- Auto-capitalize toggle
- Modern tone placement toggle
- Shortcuts editor
- Sound toggle
- Launch at login

---

## 5. Non-Functional Requirements

| ID | Requirement | Target | Status |
|----|-------------|--------|--------|
| NFR-01 | Latency | <1ms | ✅ ~0.3-0.5ms |
| NFR-02 | Memory | <10MB | ✅ ~5MB |
| NFR-03 | Test Coverage | High | ✅ 600+ tests |
| NFR-04 | Accessibility | macOS trusted | ✅ |
| NFR-05 | Privacy | No telemetry | ✅ Offline 100% |
| NFR-06 | Platforms | macOS/Windows/Linux | 🔄 macOS=prod |

---

## 6. Business Rules

| ID | Rule | Source |
|----|------|--------|
| BR-01 | Invalid syllable → no transform | `validation.rs` |
| BR-02 | Double-key → revert transformation | `engine/mod.rs` |
| BR-03 | ESC → restore raw input | `engine/mod.rs` |
| BR-04 | Space after English pattern → auto-restore | `engine/mod.rs` |
| BR-05 | Per-app mode saves ON/OFF per bundle ID | `AppState` |
| BR-06 | Ctrl/Cmd modifier → pass through | `engine/mod.rs` |
| BR-07 | Free tone mode → skip validation | `engine/mod.rs` |

---

## 7. Integration Points

| ID | System | Type | Description |
|----|--------|------|-------------|
| INT-01 | macOS CGEventTap | System API | Keyboard hook |
| INT-02 | Windows SetWindowsHookEx | System API | Keyboard hook |
| INT-03 | Linux Fcitx5 | IME Framework | Input method |
| INT-04 | macOS Accessibility | Permission | Required for keyboard |
| INT-05 | GitHub Releases | External | Auto-update source |
| INT-06 | Homebrew | Distribution | macOS install |

---

## 8. Traceability Matrix

| Feature | Entities | Screens | Rules |
|---------|----------|---------|-------|
| FR-01 | E-01,E-02,E-03,E-04 | - | BR-01,BR-06 |
| FR-02 | E-03 | S-02 | - |
| FR-03 | E-03 | S-02 | - |
| FR-04 | E-01 | - | BR-01 |
| FR-05 | E-05 | S-02 | - |
| FR-06 | E-02 | - | BR-04 |
| FR-07 | E-02 | - | BR-03 |
| FR-08 | E-03 | - | BR-02 |
| FR-09 | E-06,E-07 | S-02 | BR-05 |

---

## 9. IPA Checklist

- [x] Entities extracted from code
- [x] Features inferred from code
- [x] Screens extracted from UI files
- [x] Non-functional requirements defined
- [x] Business rules documented
- [ ] User stories (needs manual input)
- [ ] Acceptance criteria (needs manual input)

---

## Changelog

- **2026-01-11**: Initial generation from codebase analysis
