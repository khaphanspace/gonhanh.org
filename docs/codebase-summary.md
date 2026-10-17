# Gõ Nhanh: Codebase Summary

Complete directory structure, module responsibilities, and development entry points for the Gõ Nhanh Vietnamese Input Method Engine.

## Directory Structure

```
gonhanh.org/
├── core/                          # Rust engine (100% platform-agnostic)
│   ├── build.rs                  # Packs data/dictionaries/* into static word lists
│   ├── src/
│   │   ├── lib.rs                # FFI exports (22 functions: ime_init, ime_key, ime_method, ...)
│   │   ├── utils.rs              # Utility functions (char conversions, etc.)
│   │   │
│   │   ├── phonology/            # "Is this Vietnamese? Where does the tone go?"
│   │   │   ├── letters.rs        # Unit / Tone / Mod: the atoms of a syllable
│   │   │   ├── validity.rs       # validate(): Invalid / Prefix / Complete
│   │   │   ├── tone_place.rs     # tone_index(): which vowel carries the tone
│   │   │   └── tables.rs         # Generated nucleus × coda tables (scripts/gen/phonology_tables.py)
│   │   │
│   │   ├── compose/              # "What do these keys type?" (beam of readings)
│   │   │   ├── method.rs         # Telex/VNI as a table: key → intent
│   │   │   ├── parse.rs          # Readings of the keys typed so far
│   │   │   ├── lattice.rs        # Compose: beam, pruning by phonology, ranking
│   │   │   ├── render.rs         # Letters → text (tone placed on the finished syllable)
│   │   │   └── diff.rs           # Text change → backspaces + new chars
│   │   │
│   │   ├── engine/               # "What happens when the word ends?"
│   │   │   ├── mod.rs            # Engine struct, settings, key entry points, FFI Result
│   │   │   ├── word.rs           # Letter / digit / Backspace keys
│   │   │   ├── boundary.rs       # Space, punctuation, Enter, ESC
│   │   │   ├── restore.rs        # English restore decision table
│   │   │   ├── history.rs        # Committed words (Backspace after space)
│   │   │   ├── keymap.rs         # Platform keycode ↔ core key, word → keys
│   │   │   ├── out.rs            # Internal edit instruction before packing into Result
│   │   │   ├── autocap.rs        # Auto-capitalize helpers
│   │   │   ├── disabled.rs       # IME off: shortcuts still work
│   │   │   └── shortcut.rs       # User-defined abbreviations with priority
│   │   │
│   │   ├── data/                 # Keycodes + word lists
│   │   │   ├── mod.rs            # Data module exports
│   │   │   ├── keys.rs           # Virtual keycode constants and classifiers
│   │   │   ├── lexicon.rs        # Static word lists (EN, REF, VI, KEEP, DOUBLES)
│   │   │   ├── english_dict.rs   # English lookups (inflections)
│   │   │   ├── telex_doubles.rs  # English words containing Telex doubles
│   │   │   └── dictionaries/     # vi.dic, names.dic, keep.dic, en/, en-ref/, ...
│   │   │
│   │   └── updater/mod.rs        # Version comparison for platform update checks
│   │
│   ├── tests/
│   │   ├── data/                 # Word corpora (english_100k, vietnamese_22k, ...)
│   │   └── suite/                # One test binary: main.rs + modules
│   │       ├── common/mod.rs     # Test utilities (IME helper, test setup)
│   │       ├── typing_test.rs    # Full keystroke sequences (Telex + VNI)
│   │       ├── engine_test.rs    # Engine state + settings
│   │       ├── integration_test.rs # End-to-end keystroke→output tests
│   │       ├── paragraph_test.rs # Multi-word paragraph typing tests
│   │       ├── issue_regressions.rs, bug_reports_test.rs # Cases reported by users
│   │       └── phonology_audit.rs, english_*_test.rs, vietnamese_*_test.rs, ...
│   ├── benches/engine_bench.rs   # Latency / allocation benchmark
│   ├── examples/                 # Dev tools: try, trace, prof, audit, dbg, wire
│   └── Cargo.toml               # Rust manifest (zero production deps)
│
├── platforms/                    # Platform-specific implementations
│   │
│   ├── macos/                   # Production: SwiftUI app (~1700 LOC)
│   │   ├── App.swift            # AppDelegate + main application setup
│   │   ├── RustBridge.swift     # FFI bridge to Rust engine (CRITICAL)
│   │   ├── MenuBar.swift        # Status bar UI + menu items
│   │   │
│   │   ├── SettingsView.swift   # Input method selection + preferences
│   │   ├── OnboardingView.swift # Accessibility permission setup wizard
│   │   ├── AboutView.swift      # About window + version info
│   │   ├── UpdateView.swift     # Update notification UI
│   │   │
│   │   ├── LaunchAtLogin.swift  # SMAppService integration (auto-launch)
│   │   ├── UpdateManager.swift  # DMG download + version tracking
│   │   ├── UpdateChecker.swift  # GitHub API integration (version checking)
│   │   ├── AppMetadata.swift    # Shared app constants (version, names)
│   │   │
│   │   ├── libgonhanh_core.a    # Compiled universal Rust library (arm64 + x86_64)
│   │   ├── GoNhanh.xcodeproj/   # Xcode project + build settings
│   │   ├── Assets.xcassets/     # App icons (1024×1024 down to 16×16)
│   │   ├── dmg-resources/       # DMG installer background + resources
│   │   └── Tests/               # Swift unit tests (LaunchAtLoginTests.swift)
│   │
│   ├── windows/                 # Production: WPF/.NET 8 app (~1400 LOC)
│   │   ├── App.xaml.cs          # Application entry point + setup
│   │   ├── Core/
│   │   │   ├── RustBridge.cs    # FFI bridge to Rust engine
│   │   │   ├── KeyboardHook.cs  # SetWindowsHookEx keyboard interception
│   │   │   ├── KeyCodes.cs      # Windows virtual keycodes mapping
│   │   │   └── TextSender.cs    # Text input simulation (SendInput)
│   │   ├── Services/
│   │   │   ├── SettingsService.cs # Registry-based settings persistence
│   │   │   └── UpdateService.cs   # Windows update checker
│   │   ├── Views/
│   │   │   ├── TrayIcon.cs      # System tray icon UI + menu
│   │   │   ├── OnboardingWindow.xaml.cs # Setup wizard
│   │   │   ├── AboutWindow.xaml.cs      # About dialog
│   │   │   └── SettingsWindow.xaml.cs   # Preferences window
│   │   └── libgonhanh_core.dll  # Compiled Rust DLL
│   │
│   └── linux/                   # Beta: Fcitx5 addon (~500 LOC)
│       ├── src/
│       │   ├── Engine.h/cpp      # Fcitx5 InputMethodEngine implementation
│       │   ├── RustBridge.h/cpp  # C++ FFI wrapper to Rust core
│       │   └── KeycodeMap.h      # X11/Wayland keysym → keycode mapping
│       ├── data/
│       │   ├── gonhanh-addon.conf # Fcitx5 addon registration
│       │   └── gonhanh.conf      # Input method configuration
│       ├── scripts/
│       │   ├── build.sh          # CMake build script
│       │   └── install.sh        # User-local installation script
│       └── libgonhanh_core.so    # Compiled Rust shared library (x86_64)
│
├── scripts/                     # Build automation
│   ├── build/                  # Build scripts (core.sh, macos.sh, windows.sh)
│   ├── setup/                  # Setup scripts (macos.sh, windows.ps1, linux.sh)
│   ├── release/                # Release (dmg.sh, notes.sh, contributors.js)
│   └── test/                   # Test scripts (benchmark.sh, typing.swift)
│
├── Makefile                    # Main build targets
├── .github/workflows/          # CI/CD automation
│   ├── ci.yml                 # Run on push/PR: format, clippy, tests
│   └── release.yml            # Run on tags: build, create GitHub release
│
├── CLAUDE.md                   # Developer guidance (architecture, patterns, commands)
├── README.md                   # Project overview + quick start
└── docs/                       # Documentation (this folder)
```

## Core Module Responsibilities

Dependencies only go downward: `engine` → `compose` → `phonology`. Full picture: [core-architecture.md](core-architecture.md).

### Phonology (core/src/phonology/)

Pure, stateless, allocation-free. Knows nothing about keycodes, buffers, English dictionaries or the engine.

- `letters.rs` (~160 lines): `Unit` (base letter + modifier + stroke + tone), `Tone`, `Mod`.
- `validity.rs` (~570 lines): `validate()` answers `Invalid` (no completion is Vietnamese), `Prefix` (valid so far) or `Complete`. Rules: structure (C₁)(G)V(C₂)+T, c/k/g/gh/ng/ngh spelling, nucleus whitelist, stop finals take only sắc/nặng, nucleus × coda matrix. See [validation-algorithm.md](validation-algorithm.md).
- `tone_place.rs` (~100 lines): `tone_index()` picks the vowel that carries the tone from the finished syllable, never from typing order (`hoaf`, `hofa` agree). Modern (oà/uý) or old (òa/úy) style.
- `tables.rs`: generated from `vi.dic` + `names.dic` by `scripts/gen/phonology_tables.py`. Do not edit by hand.

### Compose (core/src/compose/)

Raw keys in, display text out. Every ambiguous key (a e o w d s f r x j z, VNI digits) is a letter or a modifier; `Compose` keeps a small beam of live readings and lets `phonology::validate` prune those that can never become Vietnamese. The display is the best surviving reading, so typing order never matters and nothing is reverted.

- `method.rs`: Telex and VNI as one key → intent table (Telex: s/f/r/x/j tones, aa/ee/oo/ow/aw/w modifiers, dd → đ; VNI: 1-5 tones, 6-8 modifiers, 9 → đ).
- `parse.rs`: `extend()` returns every sensible reading of the next key.
- `lattice.rs`: `Compose` (beam ≤ 8), ranking `Complete > Prefix > Loose > NamePrefix`, "keep" fallback when no reading is Vietnamese.
- `render.rs`: letters → `Display`, tone placed by `tone_index`.
- `diff.rs`: old display → new display = (backspaces, chars).

### Engine (core/src/engine/)

#### `engine/mod.rs` - `Engine`
**Lines**: ~400 | **Source**: `core/src/engine/mod.rs`

Owns the current word (`Compose`), settings and committed-word history. Public entry points: `on_key`, `on_key_ext`, `on_key_with_char`. Returns `Result` (action None/Send/Restore, backspace count, output chars).

Key flow:
- Ctrl → clear; IME off → `disabled.rs` (shortcuts only)
- Space / ESC / punctuation / Enter → `boundary.rs`: shortcut expansion → English restore table → history
- Letter / digit / Backspace → `word.rs`: `Compose::push` → render → diff → `Out` → `Result`

**Key Functions**:
- `Engine::new()`, `clear()`, `clear_all()`
- `set_method(u8)` (0=Telex, 1=VNI), `set_enabled(bool)`
- settings: `set_esc_restore`, `set_free_tone`, `set_modern_tone`, `set_english_auto_restore`, `set_auto_capitalize`, `set_allow_foreign_consonants`, ...
- `shortcuts_mut()` - user abbreviations
- `restore_word(&str)` - resume editing a word already on screen

#### Word-level modules
- `word.rs`, `boundary.rs`: key handlers for the word being typed and for word ends.
- `restore.rs`: English restore as one ordered decision table (first matching row wins). Never returns raw letters for a valid Vietnamese word without strong English evidence.
- `history.rs`: last 10 committed words so Backspace after a space can resume editing.
- `keymap.rs`: platform keycode ↔ core key; word → typing keys.
- `out.rs`: small internal edit value, packed once into the 1 KB `Result`.
- `autocap.rs`, `disabled.rs`: auto-capitalize helpers; IME-off behavior.

#### `engine/shortcut.rs` - User-Defined Abbreviations
**Lines**: ~950 | **Complexity**: Medium | **Source**: `core/src/engine/shortcut.rs`

Priority-based matching system. Supports arbitrary abbreviation → expansion (e.g., "hv" → "không"), per input method or for all. Longest-match-first strategy to avoid conflicts.

### Data (core/src/data/)

- `keys.rs`: virtual keycodes and classifiers (`is_letter`, `is_vowel`, `is_break`).
- `lexicon.rs`: static sorted word lists generated by `core/build.rs` from `data/dictionaries/` (`VI`, `KEEP`, `EN`, `REF`, `DOUBLES`); in-place binary search, no heap.
- `english_dict.rs`, `telex_doubles.rs`: thin lookups on those lists.
- `dictionaries/`: `vi.dic`, `names.dic`, `vi-non-syllables.txt`, `keep.dic`, `en/*.txt`, `en-ref/*.txt`.

### Updater (core/src/updater/mod.rs)

Semantic version comparison reused by platform update checkers.

### FFI Layer (core/src/lib.rs)

**Lines**: ~900 | **Complexity**: High (unsafe) | **Source**: `core/src/lib.rs`

Exports 22 C ABI functions (thread-safe via Mutex). Critical: Must maintain `#[repr(C)]` struct layout exactly.

**Main functions**:
```rust
ime_init()                                                  // Initialize
ime_key(key: u16, caps: bool, ctrl: bool) -> *mut Result   // Process keystroke
ime_key_ext(key, caps, ctrl, shift) -> *mut Result         // With shift state
ime_key_with_char(...) -> *mut Result                      // With the real character (Option keys)
ime_method(method: u8)                                      // Switch input method (0=Telex, 1=VNI)
ime_enabled(enabled: bool)                                  // Toggle on/off
ime_clear() / ime_clear_all()                               // Reset word / word + history
ime_restore_word(word)                                      // Resume editing a word on screen
ime_add_shortcut / ime_remove_shortcut / ime_clear_shortcuts
ime_free(result)                                            // Deallocate Result
```
Plus setting toggles (`ime_modern`, `ime_free_tone`, `ime_esc_restore`, `ime_english_auto_restore`, `ime_auto_capitalize`, `ime_allow_foreign_consonants`, `ime_skip_w_shortcut`, `ime_bracket_shortcut`).

**Result Struct** (matches Swift exactly):
```rust
#[repr(C)]
pub struct Result {
    pub chars: [u32; 256],   // UTF-32 output (1024 bytes)
    pub action: u8,          // 0=None, 1=Send, 2=Restore
    pub backspace: u8,       // Characters to delete
    pub count: u8,           // Valid output chars
    pub flags: u8,           // bit 0: key consumed (shortcut)
}
```

## Platform-Specific Modules

### macOS Platform (platforms/macos/)

#### `RustBridge.swift` - FFI Bridge (CRITICAL)
**Lines**: ~250 | **Responsibility**: Bridge Rust ↔ Swift | **Source**: `platforms/macos/RustBridge.swift`

Must declare `ImeResult` struct matching Rust `Result` byte-for-byte. Wraps the Rust FFI functions. Handles pointer safety with `defer { ime_free(ptr) }`.

#### `MenuBar.swift` - Status Bar UI
**Lines**: ~350 | **Responsibility**: Main app UI | **Source**: `platforms/macos/MenuBar.swift`

Creates NSStatusBar, manages menu items: Enable/Disable, Input Method, Settings, About, Quit. Handles global Ctrl+Space hotkey.

#### `App.swift` - Application Delegate
**Source**: `platforms/macos/App.swift`

AppDelegate for NSApplication. First-run detection, MenuBarController initialization, accessibility permission checking.

#### Other Swift Files
- `OnboardingView.swift` - Permission setup wizard
- `LaunchAtLogin.swift` - SMAppService integration
- `UpdateManager.swift` - DMG download + mounting
- `UpdateChecker.swift` - GitHub release checking
- `SettingsView.swift`, `AboutView.swift`, `UpdateView.swift` - UI components
- `AppMetadata.swift` - Shared app constants

### Windows Platform (platforms/windows/)

#### `Core/RustBridge.cs` - FFI Bridge
**Source**: `platforms/windows/Core/RustBridge.cs`

P/Invoke signatures matching Rust FFI, UTF-32 interop, memory management.

#### `Core/KeyboardHook.cs` - Keyboard Interception
**Source**: `platforms/windows/Core/KeyboardHook.cs`

SetWindowsHookEx for system-wide WH_KEYBOARD_LL hook, WM_KEYDOWN processing.

#### `Services/SettingsService.cs` - Registry Persistence
**Source**: `platforms/windows/Services/SettingsService.cs`

Stores user preferences, input method selection, enable/disable state.

#### `Views/TrayIcon.cs` - System Tray UI
**Source**: `platforms/windows/Views/TrayIcon.cs`

NotifyIcon creation, context menu: Enable/Disable, Input Method, Settings, About.

### Linux Platform (platforms/linux/)

#### `src/Engine.h/cpp` - Fcitx5 Integration
**Lines**: ~200 | **Responsibility**: Input method engine | **Source**: `platforms/linux/src/Engine.h/cpp`

Implements Fcitx5 `InputMethodEngine` interface. Handles input method registration, key processing, and candidate list management.

#### `src/RustBridge.h/cpp` - C++ FFI Wrapper
**Lines**: ~150 | **Responsibility**: Bridge C++ ↔ Rust | **Source**: `platforms/linux/src/RustBridge.h/cpp`

C++ wrapper around Rust FFI, handles UTF-32 conversion and memory safety.

#### `src/KeycodeMap.h` - Keycode Mapping
**Source**: `platforms/linux/src/KeycodeMap.h`

Maps X11/Wayland keysyms to internal keycode representation for compatibility with macOS keycode space.

## Test Coverage

### Test Files (core/tests/suite/)

One test binary (`main.rs` + modules); corpora in `core/tests/data/`. Library unit tests live next to the code (`make t`).

| File | Purpose | Source |
|------|---------|--------|
| `typing_test.rs` | Full keystroke sequences (Telex + VNI) | `core/tests/suite/typing_test.rs` |
| `engine_test.rs` | Engine state + settings | `core/tests/suite/engine_test.rs` |
| `integration_test.rs` | End-to-end keystroke→output | `core/tests/suite/integration_test.rs` |
| `paragraph_test.rs` | Multi-word paragraphs | `core/tests/suite/paragraph_test.rs` |
| `issue_regressions.rs`, `bug_reports_test.rs` | Cases reported by users | `core/tests/suite/` |
| `phonology_audit.rs` | Syllable validity + tone placement | `core/tests/suite/phonology_audit.rs` |
| `english_*_test.rs`, `vietnamese_*_test.rs` | Corpus pass rates | `core/tests/suite/` |

**Test Utilities** (core/tests/suite/common/mod.rs): IME helper and assertion helpers.

## Entry Points for Common Development Tasks

### Adding a New Input Method
1. Add the method to `Method` and its key → intent table in `core/src/compose/method.rs`
2. Map the platform setting in `Engine::set_method()` (`core/src/engine/mod.rs`) and `ime_method()` (`core/src/lib.rs`)
3. Add test cases in `core/tests/suite/typing_test.rs`
4. Update UI in `platforms/macos/SettingsView.swift`

### Fixing a Typing Bug
1. Add the user's case (keys, expected text) to `core/tests/suite/issue_regressions.rs`
2. Fix at the right layer: grammar in `core/src/phonology/`, readings in `core/src/compose/parse.rs`, end-of-word decision in `core/src/engine/restore.rs`
3. If `vi.dic` / `names.dic` changed, run `python3 scripts/gen/phonology_tables.py`
4. Run `make t`, then `make gate`

### Optimizing Engine Performance
1. Measure with `cd core && cargo run --release --example prof` (CPU time per key, per layer) and `make bench`
2. Avoid allocations in the `ime_key()` path (beam and buffers are fixed-size arrays)

### Adding Shortcut Support UI
1. Design shortcut edit dialog in `platforms/macos/ShortcutsView.swift`
2. Store in UserDefaults as JSON
3. Parse in `RustBridge.swift` and call `ime_add_shortcut()` / `ime_clear_shortcuts()`
4. Test with `core/tests/suite/disabled_shortcut_test.rs`

### Cross-Platform Port (Windows/Linux)
1. **Core** (core/src/): Already platform-agnostic ✓
2. **Platform wrapper**:
   - Windows: `platforms/windows/` (WPF + P/Invoke) - DONE
   - Linux: `platforms/linux/` (Qt + FFI) - TODO
3. **Build automation**: Add scripts pattern
4. **Testing**: Adapt platform-specific tests

## Module Dependency Graph

```
lib.rs (FFI boundary)
  ↓
engine/ (word end: shortcuts, English restore, history)
  ├─→ engine/shortcut.rs
  ├─→ engine/restore.rs ──→ data/lexicon.rs (EN, REF, VI, KEEP, DOUBLES)
  ├─→ data/keys.rs
  └─→ compose/ (beam of readings, render, diff)
        ├─→ compose/method.rs (Telex/VNI key → intent)
        └─→ phonology/ (validate, tone_index; tables.rs generated)

RustBridge.swift (macOS)
  ├─→ lib.rs exports
  └─→ CGEventTap keyboard hook

RustBridge.cs (Windows)
  ├─→ libgonhanh_core.dll exports
  └─→ SetWindowsHookEx keyboard hook
```

## Module Characteristics

| Module | LOC | Responsibility | Stability | Complexity |
|--------|-----|-----------------|-----------|------------|
| engine/mod.rs + word/boundary/restore | 1300 | Word lifecycle | High | High |
| engine/shortcut.rs | 950 | User abbreviations | Medium | Medium |
| compose/* | 1550 | Beam, render, diff | High | High |
| phonology/* | 1000 | Grammar, tone placement | Very High | High |
| data/*.rs | 500 | Keycodes, lookups | Very High | Low |
| lib.rs | 900 | FFI | Very High | High |
| RustBridge.swift | 250 | FFI Bridge | High | High |
| MenuBar.swift | 350 | UI | Medium | Medium |
| Other Swift | 1100 | Platform | Medium | Low-Medium |

## Performance Characteristics

### Critical Path (ime_key execution)
1. Lock ENGINE mutex
2. `Compose::push`: extend readings, prune with `phonology::validate`, rank
3. Render + diff against the previous display
4. Pack `Result`
5. Unlock mutex

Target: well under 1ms per key. No heap allocation in the typing core; measure with `examples/prof` and `make bench`.

### Memory Usage
- Static data: word lists (`data/lexicon.rs`), phonology tables, keycodes
- ENGINE global: fixed-size struct (beam, history of 10 words)
- Per keystroke: Stack-allocated arrays only (no heap)
- SwiftUI overhead: ~4.5MB (standard)

**Total app**: ~5MB resident

---

**Last Updated**: 2026-10-09
**Total Lines**: ~16,000 (Rust + Swift + Windows + Linux)
**Total Tokens**: 99,444 (per repomix analysis)
**Coverage**: 100% of directories documented
**Platforms**: macOS (v1.0.21+), Windows (production), Linux (beta)
