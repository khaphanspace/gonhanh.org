# Gõ Nhanh: Codebase Summary

Complete directory structure, module responsibilities, and development entry points for the Gõ Nhanh Vietnamese Input Method Engine.

## Directory Structure

```
gonhanh.org/
├── core/                          # Rust engine (100% platform-agnostic)
│   ├── src/
│   │   ├── lib.rs                # FFI exports (ime_init, ime_key, ime_method, etc.)
│   │   ├── utils.rs              # Utility functions (char conversions, etc.)
│   │   │
│   │   ├── engine/               # Core processing pipeline
│   │   │   ├── mod.rs            # Main Engine struct + ime_key orchestration
│   │   │   ├── buffer.rs         # Circular typing buffer (64 chars)
│   │   │   ├── syllable.rs       # Syllable parsing (C+G+V+C pattern)
│   │   │   ├── validation.rs     # Vietnamese phonology rules (5 rules)
│   │   │   ├── transform.rs      # Diacritic + tone application (pattern-based)
│   │   │   └── shortcut.rs       # User-defined abbreviations with priority
│   │   │
│   │   ├── input/                # Input method strategies
│   │   │   ├── mod.rs            # Input trait + method registry
│   │   │   ├── telex.rs          # Telex method (a/e/o/w for tones, s/f/r/x/j for marks)
│   │   │   └── vni.rs            # VNI method (1-5 for marks, 6-8 for tones, 9 for đ)
│   │   │
│   │   ├── data/                 # Static Vietnamese linguistic data
│   │   │   ├── mod.rs            # Data module exports
│   │   │   ├── keys.rs           # Telex/VNI keycode to transformation mappings
│   │   │   ├── chars.rs          # Character data (UTF-32 constants, casing)
│   │   │   ├── vowel.rs          # Vowel table (72 entries: 12 bases × 6 marks)
│   │   │   └── constants.rs      # Constants (consonants, valid clusters, etc.)
│   │   │
│   │   └── v2/                   # Engine V2 (Spec v3.7) - Next-gen architecture
│   │       ├── mod.rs            # 7-step keystroke pipeline orchestration
│   │       ├── types.rs          # KeyType, MarkType, Tone, Action enums
│   │       ├── state.rs          # BufferState u16 bitmask, VnState enum
│   │       └── buffer.rs         # Dual-buffer (raw/transformed) tracking
│   │
│   ├── tests/                    # Integration + unit tests (2100+ lines)
│   │   ├── common/mod.rs         # Test utilities (IME helper, test setup)
│   │   ├── unit_test.rs          # Unit tests for individual modules
│   │   ├── typing_test.rs        # Full keystroke sequences (Telex + VNI)
│   │   ├── engine_test.rs        # Engine initialization + state tests
│   │   ├── integration_test.rs   # End-to-end keystroke→output tests
│   │   └── paragraph_test.rs     # Multi-word paragraph typing tests
│   │
│   └── Cargo.toml               # Rust dependencies (zero production deps)
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
│   ├── setup.sh                # Environment setup (installs Rust, arms cargo-nextest)
│   ├── build-core.sh           # Build universal Rust library (arm64 + x86_64)
│   ├── build-macos.sh          # Build macOS SwiftUI app + DMG
│   ├── build-windows.ps1       # PowerShell build script for Windows
│   └── generate-release-notes.sh # Release notes generator
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

### Engine Modules (core/src/engine/)

#### `engine/mod.rs` - Main Processing Pipeline
**Lines**: ~500 | **Complexity**: High | **Source**: `core/src/engine/mod.rs`

Central `Engine` struct orchestrating 7-stage keystroke processing:
1. **Stroke detection** (đ/Đ) - Single key transformation
2. **Tone mark detection** (sắc/huyền/hỏi/ngã/nặng) - Multi-key sequences
3. **Vowel mark detection** (circumflex/horn/breve) - Multi-key sequences
4. **Mark removal** (reverse vowel transformation) - Undo previous marks
5. **W-vowel handling** (Telex-specific "w"→"ư") - Context-aware substitution
6. **Normal letter processing** - Regular keystroke
7. **Shortcut expansion** (user-defined) - Abbreviation matching

**Result**: Returns `Result` struct with action (None/Send/Restore), backspace count, output chars

**Key Functions**:
- `pub fn new() -> Self` - Initialize with empty buffer
- `pub fn process_key(&mut self, key: u16, is_shift: bool) -> Result` - Main entry point
- `pub fn clear(&mut self)` - Reset buffer (word boundary)
- `pub fn set_enabled(&mut self, enabled: bool)` - Toggle on/off
- `pub fn set_method(&mut self, method: InputMethod)` - Switch Telex/VNI
- `pub fn set_shortcuts(&mut self, shortcuts: Vec<Shortcut>)` - User abbreviations

#### `engine/buffer.rs` - Circular Typing Buffer
**Lines**: ~300 | **Complexity**: Medium | **Source**: `core/src/engine/buffer.rs`

Fixed 64-character circular buffer for multi-keystroke context. Tracks tone mark, vowel mark, and stroke for each character. Implements tone/mark repositioning (e.g., "hoaf" → "hoà").

**Key Methods**:
- `append_char(&mut self, ch: char, tone: ToneType, mark: VowelMark)`
- `remove_last(&mut self)` - Backspace operation
- `get_context(&self, count: usize) -> String` - Get last N chars
- `last_vowel_pos(&self) -> Option<usize>` - Find tone anchor point

#### `engine/syllable.rs` - Vietnamese Syllable Parsing
**Lines**: ~400 | **Complexity**: Medium-High | **Source**: `core/src/engine/syllable.rs`

Parses buffer into syllable components: (C₁)(G)V(C₂)
- C₁ = initial consonant
- G = glide (y/w)
- V = vowel
- C₂ = final consonant

Identifies vowel position for tone/mark placement based on linguistic rules.

#### `engine/validation.rs` - Vietnamese Phonology Rules
**Lines**: ~350 | **Complexity**: High | **Source**: `core/src/engine/validation.rs`

**5 Validation Rules** (applied BEFORE transformation, validation-first approach):
1. **Must have vowel**: Every valid syllable contains at least one vowel
2. **Valid initials**: Only 16 single consonants + 10 pairs + ngh allowed at start
3. **All chars parsed**: Every character fits syllable pattern (C+G+V+C)
4. **Spelling rules**: Enforce c/k/g restrictions (c→e/i only, k→non-e/i only, etc.)
5. **Valid finals**: Only c,ch,m,n,ng,nh,p,t allowed at end

#### `engine/transform.rs` - Diacritic & Tone Application
**Lines**: ~600 | **Complexity**: Very High | **Source**: `core/src/engine/transform.rs`

Pattern-based transformation (not case-by-case). Applies tones and vowel marks with special handling:
- **UO Compound**: "duoc" + horn → "dươc" (both u,o receive horn)
- **Tone Repositioning**: Smart placement by vowel combinations
- **Revert Logic**: "ass" (double mark) → "as" (undo, keep mark+key)
- **Last-transform tracking**: For intelligent reverting

#### `engine/shortcut.rs` - User-Defined Abbreviations
**Lines**: ~500 | **Complexity**: Medium | **Source**: `core/src/engine/shortcut.rs`

Priority-based matching system. Supports arbitrary abbreviation → expansion (e.g., "hv" → "không"). Longest-match-first strategy to avoid conflicts.

### Input Method Modules (core/src/input/)

#### `input/telex.rs` - Telex Input Method
**Lines**: ~200 | **Complexity**: Medium | **Source**: `core/src/input/telex.rs`

Vietnamese VIQR-style: a+s → á, a+f → à, a+r → ả, a+x → ã, a+j → ạ

Tone marks: s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng
Vowel marks: w=ư (horn on u), a→â (circumflex), e→ê, o→ô
Special: dd → đ, w alone → ư, nhw → như

#### `input/vni.rs` - VNI Input Method
**Lines**: ~200 | **Complexity**: Medium | **Source**: `core/src/input/vni.rs`

Vietnamese numeric: a+1 → á, a+2 → à, etc.

Tone marks: 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
Vowel marks: 6=circumflex, 7=horn, 8=breve
Stroke: d+9 → đ
Symbol typing: Shift+number skips normal letter, triggers mark directly

### Data Modules (core/src/data/)

#### `data/vowel.rs` - Vowel Transformation Table
**Lines**: ~300 | **Complexity**: Low | **Source**: `core/src/data/vowel.rs`

Pre-computed 72-entry table: 12 base vowels × 6 tone marks
Maps (vowel_char, tone_type) → transformed_char
Supports case preservation (à ↔ À)

#### `data/keys.rs` - Input Method Keycode Mappings
**Source**: `core/src/data/keys.rs`

Maps virtual keycodes to character representation, handles shift/caps lock modifiers.

#### `data/chars.rs` - Character Constants
**Source**: `core/src/data/chars.rs`

Pre-computed UTF-32 codepoints for all Vietnamese characters, used for FFI output.

#### `data/constants.rs` - Vietnamese Phonology Constants
**Source**: `core/src/data/constants.rs`

Valid initial consonants, final consonants, consonant clusters, vowel groups.

### Engine V2 Modules (core/src/v2/) - Next-Generation Architecture

**Phase 1 (Spec v3.7)**: Foundation types, state machine, dual-buffer tracking

**Feature Flag**: `#[cfg(feature = "engine-v2")]` in `core/Cargo.toml`

#### `v2/mod.rs` - Pipeline Orchestration
**Lines**: ~50 | **Complexity**: High | **Source**: `core/src/v2/mod.rs`

Implements 7-step keystroke pipeline (Spec v3.7):
0. **Key Classification** - Classify keystroke to KeyType
1. **Pre-check (Foreign Mode)** - Skip processing if disabled
2. **Dispatch & Execute** - Route to appropriate handler
3. **Tone/Mark Placement** - Position diacritics intelligently
4. **Buffer Update** - Sync raw + transformed buffers
5. **9-Layer Validation** - Exhaustive Vietnamese phonology check
6. **Restore Decision** - Auto-restore to English if invalid
7. **Output Generation** - Produce final keystroke result

#### `v2/types.rs` - Core Type Definitions
**Lines**: ~120 | **Complexity**: Medium | **Source**: `core/src/v2/types.rs`

**KeyType enum**: Classifies input keystrokes
- `Letter(u8)` - Regular letter (a-z, A-Z)
- `Tone(u8)` - Tone key (Telex: s,f,r,x,j or VNI: 1-5)
- `Mark(MarkType)` - Vowel mark key (aa,oo,w or VNI: 6-9,0)
- `Terminator` - Word boundary (space, punctuation)
- `Special` - Backspace, delete, escape
- `Passthrough` - No processing needed

**MarkType enum**: Vietnamese diacritical marks
- `Stroke` - dd → đ
- `Circumflex` - aa,oo,ee → â,ô,ê
- `HornOrBreve` - w context-dependent (ư/ơ/ă)

**Tone enum**: Vietnamese tone marks (6 including neutral)
- `None` (0), `Sac` (1), `Huyen` (2), `Hoi` (3), `Nga` (4), `Nang` (5)

**Action enum**: Post-classification action
- `Continue` - Add to buffer
- `AddTone(Tone)` - Apply tone mark
- `RevertTone` - Undo tone (double-key)
- `AddMark(MarkType)` - Apply vowel mark
- `AddStroke` - Apply đ stroke
- `CheckRestore` - Evaluate auto-restore on terminator
- `Passthrough` - No modification

#### `v2/state.rs` - Buffer State Machine
**Lines**: ~450 | **Complexity**: High | **Source**: `core/src/v2/state.rs`

**BufferState**: Compact u16 bitmask replacing 7 separate booleans
```
Bits 0-4:    Transformation tracking (had_transform, has_stroke, has_tone, has_mark, had_revert)
Bits 5-6:    Revert type (0=none, 1=tone, 2=mark, 3=both)
Bits 7-8:    Pending marks (pending_breve, pending_horn)
Bits 9-11:   Vietnamese validation state (VnState)
Bits 12-15:  Reserved for future features
```

**VnState enum**: Phonological validation state
- `Unknown` (0) - Not yet determined
- `Complete` (1) - Valid complete syllable
- `Incomplete` (2) - Valid start, needs more input
- `Impossible` (3) - Definitely not Vietnamese

**Cache Optimization**: Single u16 enables O(1) state comparisons vs 7 boolean comparisons

#### `v2/buffer.rs` - Dual-String Buffer
**Lines**: ~290 | **Complexity**: Medium | **Source**: `core/src/v2/buffer.rs`

Maintains parallel tracking of raw input vs transformed output:
- **raw**: Original ASCII keystrokes as typed (e.g., "vieejt")
- **transformed**: Vietnamese output (e.g., "việt")

**Why Dual-Buffer**:
- Accurate auto-restore (restore to raw on English detection)
- Character consumption tracking (raw.len - transformed.len)
- Clean diff calculation for keystroke output

**Key Methods**:
- `raw()` / `transformed()` - Read-only accessors
- `push_raw()` / `push_transformed()` - Append characters
- `pop_raw()` / `pop_transformed()` - Backspace
- `char_consumption()` - (raw_len - transformed_len), detects English being typed as Vietnamese
- `is_unchanged()` - Check if raw == transformed
- `clear()` - Reset both buffers at word boundary

### FFI Layer (core/src/lib.rs)

**Lines**: ~300 | **Complexity**: High (unsafe) | **Source**: `core/src/lib.rs`

Exports 6 C ABI functions (thread-safe via Mutex). Critical: Must maintain `#[repr(C)]` struct layout exactly.

**Exported Functions**:
```rust
ime_init()                                                  // Initialize
ime_key(key: u16, is_shift: bool, is_ctrl: bool) -> Result // Process keystroke
ime_key_ext(key, is_shift, is_ctrl, special) -> Result     // Extended version
ime_method(method: u8)                                      // Switch input method (0=Telex, 1=VNI)
ime_enabled(enabled: bool)                                  // Toggle on/off
ime_clear()                                                 // Reset buffer
ime_free(result)                                            // Deallocate Result
```

**Result Struct** (matches Swift exactly):
```rust
#[repr(C)]
pub struct Result {
    pub chars: [u32; 32],    // UTF-32 output (128 bytes)
    pub action: u8,          // 0=None, 1=Send, 2=Restore
    pub backspace: u8,       // Characters to delete
    pub count: u8,           // Valid output chars
    pub _pad: u8,            // Alignment padding
}
```

## Platform-Specific Modules

### macOS Platform (platforms/macos/)

#### `RustBridge.swift` - FFI Bridge (CRITICAL)
**Lines**: ~250 | **Responsibility**: Bridge Rust ↔ Swift | **Source**: `platforms/macos/RustBridge.swift`

Must declare `ImeResult` struct matching Rust `Result` byte-for-byte. Wraps all 6 Rust FFI functions. Handles pointer safety with `defer { ime_free(ptr) }`.

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

### Test Files (core/tests/)

| File | Purpose | Test Count | Source |
|------|---------|-----------|--------|
| `unit_test.rs` | Module unit tests | ~30 | `core/tests/unit_test.rs` |
| `typing_test.rs` | Full keystroke sequences | ~60 | `core/tests/typing_test.rs` |
| `engine_test.rs` | Engine initialization + state | ~20 | `core/tests/engine_test.rs` |
| `integration_test.rs` | End-to-end keystroke→output | ~35 | `core/tests/integration_test.rs` |
| `paragraph_test.rs` | Multi-word paragraphs | ~15 | `core/tests/paragraph_test.rs` |

**Total**: 160+ test cases, 2100+ lines of test code

**Test Utilities** (core/tests/common/mod.rs):
- `ImeHelper` struct for convenient test setup
- `assert_output()` macro for comparing expected results
- Test data constants

## Entry Points for Common Development Tasks

### Adding a New Input Method
1. Create `core/src/input/mynewmethod.rs` implementing InputMethod trait
2. Export from `core/src/input/mod.rs`
3. Add pattern matching in `engine/mod.rs` process_key()
4. Add test cases in `core/tests/typing_test.rs`
5. Update UI in `platforms/macos/SettingsView.swift`

### Fixing a Transform Bug
1. Locate issue in `core/src/engine/transform.rs`
2. Check `core/src/data/vowel.rs` for vowel table correctness
3. Add failing test case in `core/tests/unit_test.rs` or `typing_test.rs`
4. Fix transform logic
5. Run `make test` to verify

### Optimizing Engine Performance
1. Profile with `cargo flamegraph` (optional dependency)
2. Most time spent in: transform (60%), validation (25%), buffer operations (15%)
3. Avoid allocations in `ime_key()` path (uses stack-allocated arrays)

### Adding Shortcut Support UI
1. Design shortcut edit dialog in `platforms/macos/ShortcutsView.swift`
2. Store in UserDefaults as JSON
3. Parse in `RustBridge.swift` and call `ime_set_shortcuts()`
4. Test with `core/tests/shortcut_test.rs`

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
engine/mod.rs (orchestration)
  ├─→ engine/buffer.rs
  ├─→ engine/syllable.rs
  ├─→ engine/validation.rs
  ├─→ engine/transform.rs
  │    └─→ data/vowel.rs
  ├─→ engine/shortcut.rs
  ├─→ input/telex.rs (or vni.rs)
  │    └─→ data/keys.rs
  └─→ data/constants.rs

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
| engine/mod.rs | 500 | Orchestration | High | High |
| engine/transform.rs | 600 | Diacritics | High | Very High |
| engine/shortcut.rs | 500 | User abbreviations | Medium | Medium |
| engine/validation.rs | 350 | Phonology rules | High | High |
| engine/buffer.rs | 300 | Circular buffer | High | Medium |
| engine/syllable.rs | 400 | Parsing | High | Medium-High |
| input/telex.rs | 200 | Input method | High | Medium |
| input/vni.rs | 200 | Input method | High | Medium |
| data/vowel.rs | 300 | Lookup table | Very High | Low |
| data/* | 550 | Constants | Very High | Low |
| lib.rs | 300 | FFI | Very High | High |
| RustBridge.swift | 250 | FFI Bridge | High | High |
| MenuBar.swift | 350 | UI | Medium | Medium |
| Other Swift | 1100 | Platform | Medium | Low-Medium |

## Performance Characteristics

### Critical Path (ime_key execution)
1. Lock ENGINE mutex (1-2μs)
2. Validate keystroke (100-150μs) - main bottleneck
3. Process transform (50-100μs)
4. Allocate + populate Result (20-30μs)
5. Unlock mutex (1-2μs)

**Total**: 170-285μs (0.17-0.28ms) - well under 1ms budget

### Memory Usage
- Static data: ~150KB (vowel table, keycodes, constants)
- ENGINE global: ~500B (struct only)
- Per keystroke: Stack-allocated arrays only (no heap)
- SwiftUI overhead: ~4.5MB (standard)

**Total app**: ~5MB resident

---

**Last Updated**: 2025-12-14
**Total Lines**: ~16,000 (Rust + Swift + Windows + Linux)
**Total Tokens**: 99,444 (per repomix analysis)
**Coverage**: 100% of directories documented
**Platforms**: macOS (v1.0.21+), Windows (production), Linux (beta)
