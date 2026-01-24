# Windows Phase 2 Documentation Index

Quick navigation guide for Windows keyboard hook (Phase 2) implementation documentation.

---

## Quick Links

### For Architects & Designers
Start here for high-level understanding:
- **[System Architecture](system-architecture.md)** - Overall design with Windows section
  - FFI integration model
  - Message loop architecture
  - Hook callback flow
  - Unicode injection pipeline

### For Implementers & Debuggers
Detailed technical reference:
- **[Windows Keyboard Hook Reference](windows-keyboard-hook-reference.md)** - Complete technical guide
  - VK→macOS keycode mapping (46 keys)
  - Hook callback implementation walkthrough
  - Reentrancy guard explanation
  - Troubleshooting guide
  - Performance characteristics

### For Phase 3 UI Developers
Complete implementation reference:
- **[Windows Phase 3: System Tray & UI](windows-phase3-ui-system-tray.md)** - Implementation guide
  - System tray architecture (Shell_NotifyIcon)
  - Settings window with 10 feature flags
  - Shortcuts dialog management
  - About dialog
  - Registry persistence (HKCU)
  - UTF-16 ↔ UTF-8 conversion
  - All 16 macOS features ported

### For Project Management
Overall roadmap and status:
- **[Project Overview & PDR](project-overview-pdr.md)** - Roadmap and requirements
  - Phase 2 breakdown (2a, 2b, 2c)
  - Phase 2b completion status (Jan 12, 2025)
  - Next phase planning

---

## Implementation Status

| Phase | Component | Status | Docs |
|-------|-----------|--------|------|
| **2a** | Architecture & Build | ✓ Complete | system-architecture.md |
| **2b** | Keyboard Hook | ✓ Complete (Jan 12) | windows-keyboard-hook-reference.md |
| **3** | UI & Settings | ✅ Complete (Jan 12) | windows-phase3-ui-system-tray.md |
| **4** | QA & Release | → Next | TBD |

---

## Documentation Structure

```
windows-keyboard-hook-reference.md (702 LOC) - Technical Reference
├── Quick Reference Table
├── Message-Only Window Architecture
├── VK→macOS Mapping (46 keys)
├── Hook Callback Implementation
├── Text Injection Pipeline
├── Reentrancy & Loop Prevention
├── Ctrl+Space Global Toggle
├── Singleton & Thread Safety
├── Build Integration
├── Troubleshooting Guide
├── Performance Notes
└── Testing Checklist

windows-phase2c-ui-integration.md (509 LOC) - Implementation Guide
├── Hook Public Interface
├── System Tray Integration Points
├── Settings Dialog Implementation
├── P/Invoke Bridge
├── Application Lifecycle
├── Threading Model
├── Troubleshooting During Integration
├── Example: Complete Minimal UI
└── Next Steps for Phase 2c

system-architecture.md (754 LOC) - Overview
└── Windows SetWindowsHookEx Integration (Phase 2)
    ├── Build System (Corrosion + CMake)
    ├── FFI Memory Model
    ├── UTF Conversion Pipeline
    ├── Keyboard Hook Implementation ← Phase 2b details
    └── Accessibility Permission (macOS)
```

---

## Key Technical Concepts

### Message-Only Window
- Why: WH_KEYBOARD_LL requires message queue for hook delivery
- How: HWND_MESSAGE creates invisible window with queue
- When: Created before SetWindowsHookEx call
- File: [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md)

### VK→macOS Keycode Mapping
- What: 46 keys mapped between Windows VK and macOS keycodes
- Why: Rust engine uses macOS keycode format
- How: Verified vs core/src/data/keys.rs
- File: [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md)

### Reentrancy Guards (Dual Layer)
- OS-Level: LLKHF_INJECTED flag (Windows marks injected keys)
- App-Level: processing_ boolean (prevents concurrent engine calls)
- Why: Belt-and-suspenders approach prevents infinite loops
- File: [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md)

### Ctrl+Space Global Toggle
- Design: Checked BEFORE enabled flag (always callable)
- Function: Toggle Vietnamese/English mode
- Suppression: Key never reaches applications
- File: [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md)

### Thread Safety Model
- Mechanism: Single-threaded message loop + atomic boolean flags
- Safety: No concurrent state modification
- No Mutex: Simple design, no deadlock risk
- File: [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md)

---

## Finding Specific Information

### "How do I..."

| Question | Answer |
|----------|--------|
| Understand the overall Windows architecture? | [system-architecture.md](system-architecture.md) - Windows section |
| Map Windows VK codes to macOS keycodes? | [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md) - VK Mapping Table |
| Debug a keyboard hook issue? | [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md) - Troubleshooting Guide |
| Integrate hook with WPF UI? | [windows-phase2c-ui-integration.md](windows-phase2c-ui-integration.md) - Complete guide |
| Understand the message loop requirement? | [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md) - Message-Only Window section |
| Prevent infinite loops from SendInput? | [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md) - Reentrancy & Loop Prevention |
| Write P/Invoke bindings? | [windows-phase2c-ui-integration.md](windows-phase2c-ui-integration.md) - P/Invoke Bridge |
| Set up Registry persistence? | [windows-phase2c-ui-integration.md](windows-phase2c-ui-integration.md) - Settings Dialog Implementation |
| Get Unicode text injection working? | [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md) - Text Injection Pipeline |
| Verify Phase 2b completion? | [project-overview-pdr.md](project-overview-pdr.md) - Roadmap section |

---

## Code Reference

### Key Files

| File | Purpose | Documentation |
|------|---------|-----------------|
| platforms/windows/src/keyboard_hook.h | Hook class definition | system-architecture.md, windows-keyboard-hook-reference.md |
| platforms/windows/src/keyboard_hook.cpp | Hook implementation | windows-keyboard-hook-reference.md (complete walkthrough) |
| platforms/windows/src/main.cpp | WinMain & message loop | windows-keyboard-hook-reference.md (Message-Only Window section) |
| platforms/windows/CMakeLists.txt | Build integration | system-architecture.md, windows-keyboard-hook-reference.md |
| platforms/windows/src/rust_bridge.h | FFI wrapper | system-architecture.md (FFI Memory Model) |
| platforms/windows/src/rust_bridge.cpp | UTF conversion | system-architecture.md (UTF Conversion Pipeline) |

---

## Performance & Requirements

### Latency Budget
- Target: <1ms per keystroke
- Typical: 200-500μs
- Headroom: 2-5x safety margin
- Details: [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md) - Performance Notes

### Memory Usage
- Hook instance: ~64 bytes
- Message window: ~100 bytes
- Total: ~170 bytes (negligible)
- Details: [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md) - Performance Notes

### Testing Checklist
- 20+ verification points
- Coverage: Typing, toggle, methods, edge cases, cross-app
- File: [windows-keyboard-hook-reference.md](windows-keyboard-hook-reference.md) - Testing Checklist

---

## Phase 3 Implementation (Complete)

Reference documentation for Phase 3 UI completion:
- [windows-phase3-ui-system-tray.md](windows-phase3-ui-system-tray.md) - Complete implementation guide
- [system-architecture.md](system-architecture.md) - Updated with Phase 3 UI section
- Key files: system_tray.h/cpp, settings.h/cpp, settings_window.h/cpp, resource.h, main.cpp

**Completion Date:** Jan 12, 2025

---

## Phase 4 (Quality Assurance & Release) - Next

1. Integration testing with keyboard hook
2. Settings persistence verification
3. Auto-start functionality testing
4. Cross-app compatibility testing
5. Performance & memory profiling
6. Installer & distribution packaging
7. Release notes & documentation

**Estimated Timeline:** 1-2 weeks

---

## Version & Updates

- **Phase 3 Status:** ✅ Complete (Jan 12, 2025)
- **Documentation Version:** 2.0 (Phase 3 Complete)
- **Last Updated:** 2025-01-12
- **Audience:** All developers working on Windows platform
- **Next Update:** Phase 4 completion (QA & Release)

---

**Navigation:** Start with your role above, then follow cross-references as needed.
