# GoNhanh Architecture

Comprehensive system architecture, module breakdown, FFI contract, and implementation details for GoNhanh Vietnamese IME.

## System Overview

GoNhanh uses a **Rust core + native UI** hybrid architecture enabling cross-platform support with platform-native UI/UX:

```
┌──────────────────────────────────────────┐
│        Platform UI Layer (Native)        │
│   ┌──────────────┐    ┌──────────────┐  │
│   │   macOS      │    │  Windows     │  │
│   │   SwiftUI    │    │  WPF (TODO)  │  │
│   │  (765 lines) │    │              │  │
│   └──────┬───────┘    └──────┬───────┘  │
└──────────┼────────────────────┼─────────┘
           │    FFI Bridge      │
           │    (C ABI)         │
┌──────────▼────────────────────▼─────────┐
│       Rust Core Library                 │
│       (2068 lines, thread-safe)         │
│  ┌────────────────────────────────┐    │
│  │  FFI Layer (lib.rs)            │    │
│  │  - 7 main functions (ime_*)    │    │
│  │  - C-compatible Result struct  │    │
│  │  - Thread-safe Mutex engine    │    │
│  └────────────────────────────────┘    │
│  ┌────────────────────────────────┐    │
│  │  Engine (engine/mod.rs)        │    │
│  │  - Key processing pipeline     │    │
│  │  - 4-stage transformation      │    │
│  │  - Vietnamese phonology rules  │    │
│  │  - 551 lines core logic        │    │
│  └────────────────────────────────┘    │
│  ┌────────────────────────────────┐    │
│  │  Data & Rules (data/, input/)  │    │
│  │  - Vowel phonology algorithm   │    │
│  │  - Telex/VNI input methods     │    │
│  │  - Unicode character mappings  │    │
│  │  - Virtual key codes           │    │
│  └────────────────────────────────┘    │
│  ┌────────────────────────────────┐    │
│  │  Buffer Management             │    │
│  │  - Typing buffer (max 32 chars)│    │
│  │  - Character state tracking    │    │
│  │  - Diacritic repositories      │    │
│  └────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

## Core Library Structure

### Module Breakdown

```
core/src/
├── lib.rs                 # FFI exports (265 lines)
├── data/
│   ├── mod.rs            # Module exports
│   ├── keys.rs           # Virtual key codes (240 lines)
│   ├── chars.rs          # Unicode mappings
│   └── vowel.rs          # Phonology algorithm (350+ lines)
├── engine/
│   ├── mod.rs            # Main engine (551 lines)
│   └── buffer.rs         # Typing buffer (max 32 chars)
└── input/
    ├── mod.rs            # Input method trait
    ├── telex.rs          # Telex rules
    └── vni.rs            # VNI rules
└── tests/                # 160+ test cases
    ├── basic_test.rs
    ├── word_test.rs
    ├── sentence_test.rs
    ├── behavior_test.rs
    ├── common_issues_test.rs
    └── edge_cases_test.rs
```

**Total: 2068 lines of Rust code**

### Line Count by Module

| Module | Lines | Purpose |
|--------|-------|---------|
| `engine/mod.rs` | 551 | Key processing pipeline (d, tone, mark, remove) |
| `data/vowel.rs` | 350+ | Vietnamese phonology rules |
| `platforms/macos/RustBridge.swift` | 443 | FFI bridge, keyboard hook, smart text replacement |
| `lib.rs` | 265 | FFI interface (7 functions) |
| `data/keys.rs` | 240 | Virtual key code constants |
| `tests/*.rs` | 160+ | Integration test suite |
| `input/telex.rs` | 80+ | Telex input method |
| `input/vni.rs` | 80+ | VNI input method |
| `platforms/macos/MenuBar.swift` | 192 | System tray, settings persistence |
| `engine/buffer.rs` | ~100 | Typing buffer management |

## FFI Interface (C ABI)

Complete FFI contract between Rust core and platform layers. All functions use C calling convention for maximum compatibility.

### Initialization & Cleanup

```c
/// Initialize IME engine (thread-safe Mutex)
void ime_init();

/// Clear current buffer (useful between words)
void ime_clear();

/// Free memory allocated by Result
/// Must call for every Result returned by ime_key()
void ime_free(Result* r);
```

### Configuration Functions

```c
/// Set input method: 0=Telex (default), 1=VNI
void ime_method(uint8_t method);

/// Enable/disable IME: true=enabled, false=disabled
void ime_enabled(bool enabled);

/// Tone placement style:
///   true=modern style  (hoà, hoả, hoã, hoạ)
///   false=old style    (hòa, hỏa, hõa, họa)
void ime_modern(bool modern);
```

### Key Processing

```c
/// Process single keystroke
/// Returns: dynamically allocated Result* (must call ime_free)
/// Returns NULL only on memory error (extremely rare)
Result* ime_key(
    uint16_t key,   // macOS virtual keycode
    bool caps,      // Caps Lock state
    bool ctrl       // Control key state (currently unused)
);
```

### Result Structure (C-compatible)

```rust
#[repr(C)]
pub struct Result {
    /// Unicode codepoints to output (up to 32 characters)
    /// Each u32 is a single Unicode codepoint (not UTF-16 surrogate pairs)
    pub chars: [u32; 32],

    /// Action type:
    ///   0 = NONE (buffer modified, no output yet)
    ///   1 = SEND (output ready, send chars[0..count])
    ///   2 = RESTORE (delete backspace chars, then output)
    pub action: u8,

    /// Number of Unicode characters to backspace before output
    /// Used for corrections (e.g., wrong tone placement)
    pub backspace: u8,

    /// Number of valid characters in chars[] array
    pub count: u8,

    /// Padding for alignment (reserved, do not use)
    pub _pad: u8,
}
```

**Memory Layout**: 32*4 + 1 + 1 + 1 + 1 = 136 bytes

### Action Semantics

1. **NONE (0)** - Internal state updated, nothing to send
   - Used when buffering keystrokes for tone placement
   - Example: typing "a" in Telex waits for next key

2. **SEND (1)** - Output ready, send characters immediately
   - Example: typing "as" → output "á" immediately
   - No deletion needed before inserting

3. **RESTORE (2)** - Correct previous input before outputting
   - Example: mark repositioning (ua2+7 → ừa not ưà)
   - Delete `backspace` characters, then insert new `count` characters

## Key Processing Pipeline

The engine processes Vietnamese text through 4 sequential transformation stages:

### 1. Đ Transformation (`try_handle_d()`)

Handles Vietnamese letter đ using different strategies per input method:

**Telex**: `dd` → đ (immediate double-key)
- Type 'd' twice in quick succession
- Reverts to single 'd' if next character is not 'd'

**VNI**: Delayed đ for consonant clusters (dung + 9)
- `d` + `u` + `n` + `g` + `9` → `đung`
- Key 9 triggers immediate đ transformation on any buffered 'd'

### 2. Tone Modifiers (`try_handle_tone()`)

Adds circumflex (^), breve (˘), horn (ơ/ư) marks to vowels:

**Telex**:
- `aa` → â (a + circumflex)
- `aw` → ă (a + breve)
- `ow` → ơ (o + horn)
- `uw` → ư (u + horn)
- Double-key revert: `aaa` → aa (undo circumflex)

**VNI**:
- `a6` → â (a + circumflex)
- `a8` → ă (a + breve)
- `o7` → ơ (o + horn)
- `u7` → ư (u + horn)
- Double-key revert: `a66` → a6 (undo circumflex)

### 3. Diacritical Marks (`try_handle_mark()`)

Applies tone marks (sắc, huyền, hỏi, ngã, nặng) with smart repositioning:

**Telex** (single-key mark):
- `s` = sắc (acute accent)
- `f` = huyền (grave accent)
- `r` = hỏi (tilde hook)
- `x` = ngã (circumflex above)
- `j` = nặng (dot below)

**VNI** (numeric mark):
- `1` = sắc
- `2` = huyền
- `3` = hỏi
- `4` = ngã
- `5` = nặng

**Smart Repositioning**: When adding a mark to multi-vowel syllables, engine automatically moves mark from incorrect position to phonologically correct position:
- `ua2` → ừa (not ưà) - mark follows vowel contraction
- `oai1` → ngoái (not ngói) - mark on primary vowel

### 4. Mark Removal (`handle_remove()`)

Removes diacritical marks in reverse order:

**Telex**: `z` (any vowel with mark + z removes mark)
**VNI**: `0` (any vowel with mark + 0 removes mark)

Multiple presses remove marks progressively:
- `á` + `z` → `a`
- `ấ` + `z` → `â` (remove tone)
- `â` + `z` → `a` (remove circumflex)

## Sophisticated Features

### Mark Repositioning Algorithm

Vietnamese phonology requires marks on specific vowel positions. When typing builds complex vowel structures, engine automatically corrects mark placement:

```
Example: Typing "muỗ" (strange)
  m + u + ô + i + 5 (nặng)

Without repositioning: "muội" (wrong: mark on ô)
With repositioning:    "muỗ" (correct: mark moved to ơ)
```

**Implementation** (`vowel.rs::find_tone_position()`):
- Analyzes vowel composition (single/double/triple)
- Considers final consonant presence
- Detects qu- special case (phonologically different from u)
- Determines primary vowel position
- Applies 8+ Vietnamese phonology rules
- Repositions mark if needed before output

### Qu- Detection

**Problem**: How to distinguish?
- `qua` - "qu" as single consonant cluster (not "q" + "ua")
- `mua` - simple "m" + "ua" diphthong

**Solution**: Qu-detection prevents phonology collision:
- `qu` + vowels are treated as consonant cluster
- Mark positioning rules change for qu- (different stress)
- Prevents tone marks on "u" in qua/que/qui sequences

### Delayed Đ (VNI Only)

**Problem**: Vietnamese "dung" (ordinary) vs "đung" (stand up)
- VNI can't distinguish immediate (like Telex dd)
- Both start with 'd'

**Solution**: Delay mark application until context clears:
- Buffer keys: d, u, n, g
- On key 9: check if 'd' exists in buffer
- Apply đ transformation to first 'd' when 9 is pressed
- Example: `d` → `u` → `n` → `g` → `9` → ✓ sends "đung"

### Double-Key Revert

Users can undo transformations by repeating keys:

```
Telex:  a + a + a → revert circumflex (aa → a)
        a + w + w → revert breve (aw → a)
VNI:    a + 6 + 6 → revert circumflex (a6 → a)
        a + 8 + 8 → revert breve (a8 → a)
```

## Vietnamese Phonology Algorithm

**File**: `core/src/data/vowel.rs`

Core algorithm determining correct tone mark placement using Vietnamese linguistic rules, not lookup tables.

### Function Signature

```rust
pub fn find_tone_position(
    vowels: &[u16],           // Buffer vowel keys
    has_final_consonant: bool, // Syllable structure
    modern: bool,             // Tone style (new vs old)
    has_qu_initial: bool      // Qu- cluster prefix
) -> Option<usize>            // Index in vowels[] to mark
```

### Algorithm Overview

1. **Classify vowel structure**
   - Single vowel (a, e, i, o, u)
   - Double vowel (ai, ao, ia, uo, etc.)
   - Triple vowel (iêu, oai, ươi, etc.)
   - Special compounds (ươ, uô, iê)

2. **Apply Vietnamese tone rules**
   - For final consonants: different rules than open syllables
   - For qu- syllables: shifted mark positions
   - For compound vowels: primary vs secondary vowel distinction
   - For modern vs old style: alternative positioning

3. **Return mark position**
   - Index into vowels array (0 = first vowel)
   - Used by engine to apply mark to correct vowel

### Rule Categories (8+ distinct)

| Vowel Pattern | Has Final | Rule | Mark Position |
|---------------|-----------|------|---------------|
| Single (a, o, u) | Any | Always on vowel | vowel position |
| ai, ao, au, etc. | Any | On first vowel | position[0] |
| ia, io, ua, etc. | Any | On second vowel | position[1] |
| oa, oe | Any | On second vowel (modern) or first (old) | varies |
| uy | Any | On second vowel (modern) or first (old) | varies |
| ươ, uô, iê | Yes | Special compound rules | compound-specific |
| qu + vowel | Any | Shifted rules (qu is cluster) | adjusted |
| Triple vowels | Any | Usually on middle vowel | middle index |

**Key insight**: Algorithm uses Vietnamese phonological structure (consonant clusters, vowel types, stress patterns) instead of naive lookup table.

## macOS Platform Layer

### Components Overview

| File | Lines | Purpose |
|------|-------|---------|
| `RustBridge.swift` | 443 | FFI bridge, keyboard hook manager, smart text replacement |
| `MenuBar.swift` | 192 | System tray icon, settings menu, state persistence |
| `SettingsView.swift` | 102 | SwiftUI settings UI (Telex/VNI, modern/old tone style) |
| `App.swift` | 28 | Entry point, LSUIElement configuration |

**Total: 765 lines of Swift**

### Keyboard Hook Architecture (`RustBridge.swift`)

**3-Tier Fallback Strategy** for maximum compatibility:

```swift
┌─────────────────────────────┐
│    CGEventTap (Primary)     │ ← Works in Terminal, most apps
│   Accessibility API         │   Requires System Settings permission
└──────────┬──────────────────┘
           │ (if fails)
           ↓
┌─────────────────────────────┐
│   NSLocalEventMonitor       │ ← Fallback: app-only events
│   (Secondary - not used)    │
└──────────┬──────────────────┘
           │ (if fails)
           ↓
┌─────────────────────────────┐
│   Manual Key Mapping        │ ← Final fallback
│   (Tertiary - not used)     │
└─────────────────────────────┘
```

**Primary**: CGEventTap at system level captures ALL keyboard events before apps see them.

### Smart Text Replacement Logic

Engine outputs Result, but platform must handle replacement correctly depending on app context:

**Strategy 1: Terminal/CLI** (no selection support)
```swift
// Delete using backspace key (sends Delete ASCII)
for _ in 0..<result.backspace {
    CGEvent(keyboardEventSource: source, virtualKey: 0x33, keyDown: true)
}
// Type new characters
for char in newChars {
    // Convert Unicode to keyboard events
}
```

**Strategy 2: Chrome/Safari** (autocomplete conflict)
- Detects autocomplete dropdown state
- Overrides with manual text replacement instead of typing
- Prevents "aa" → "aâ" double-input bug

**Strategy 3: Excel/Google Docs** (formula/context-aware)
- Checks if cursor is in formula bar
- Handles special paste behavior for complex diacritics
- Prevents mark loss on concurrent updates

### Settings Persistence

**UserDefaults Storage**:
```swift
"ime.enabled"    → Bool (default: true)
"ime.method"     → Int (0=Telex, 1=VNI)
"ime.modern"     → Bool (true=modern, false=old tone style)
```

Saved to: `~/Library/Preferences/com.khaphanspace.gonhanh.plist`

### App Lifecycle

**LSUIElement Configuration** (No Dock Icon):
```swift
// App.swift: UIElement mode
@main
struct GoNhanhApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    var body: some Scene {
        MenuBarExtra("GoNhanh", systemImage: "globe") {
            MenuBar()
        }
    }
}
```

- No window in dock
- Menu bar only interface
- Starts on login (via LaunchAgent)
- Accessibility permission request on first run

## Build System

### Scripts

| Script | Purpose |
|--------|---------|
| `scripts/setup.sh` | Install Rust targets (aarch64-apple-darwin, x86_64-apple-darwin) |
| `scripts/build-core.sh` | Build universal library (arm64 + x86_64 fat binary) |
| `scripts/build-macos.sh` | Build SwiftUI app with xcodebuild |

### Rust Build Optimization

**Cargo.toml `[profile.release]`**:
```toml
opt-level = "z"     # Size optimization (essential for ~3MB binary)
lto = true          # Link-time optimization (better inlining)
strip = true        # Remove debug symbols
codegen-units = 1   # Better optimization at cost of build time
```

**Result**:
- Binary size: ~3 MB (universal arm64 + x86_64)
- Memory overhead: ~25 MB at runtime
- Startup: ~200ms
- Per-keystroke latency: <1ms

### Build Artifacts

```
core/target/
├── aarch64-apple-darwin/release/
│   └── libgonhanh_core.a      (arm64)
├── x86_64-apple-darwin/release/
│   └── libgonhanh_core.a      (x86_64)
└── universal/
    └── libgonhanh_core.a      (lipo: merged fat binary)

platforms/macos/
├── build/Release/
│   └── GoNhanh.app
└── GoNhanh.xcodeproj/
```

## Testing Strategy

**160+ Integration Tests** covering real-world typing scenarios:

### Test Categories

| File | Tests | Coverage |
|------|-------|----------|
| `tests/basic_test.rs` | 40+ | Single keystrokes, character conversions |
| `tests/word_test.rs` | 50+ | Full Vietnamese words (Telex + VNI) |
| `tests/sentence_test.rs` | 20+ | Multi-word sentences |
| `tests/behavior_test.rs` | 20+ | User behaviors (backspace, corrections) |
| `tests/common_issues_test.rs` | 15+ | Real browser/office bugs (Chrome aa→aâ, Excel tone loss) |
| `tests/edge_cases_test.rs` | 15+ | Boundary conditions (buffer full, mixed input) |

### Test Execution

```bash
# Run all tests (cargo test)
make test

# Run specific test file
cargo test --test word_test

# Run specific test with output
cargo test vni_delayed_d_input -- --nocapture

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

## Performance Metrics

### Latency Breakdown (per keystroke)

| Component | Time | Notes |
|-----------|------|-------|
| Key capture (CGEventTap) | <0.1ms | System-level |
| FFI call overhead | <0.1ms | C function call |
| Engine processing | <0.5ms | Vowel rules, buffer management |
| Output generation | <0.1ms | Format Result struct |
| Text insertion (CGEvent.post) | <0.2ms | System event posting |
| **Total** | **<1ms** | Imperceptible to user |

### Memory Profile

| Component | Size | Notes |
|-----------|------|-------|
| Rust binary | ~3 MB | Optimized release build |
| Runtime heap | ~5 MB | Buffer + internal state |
| SwiftUI/Swift | ~15 MB | UI framework overhead |
| **Total** | ~25 MB | Very efficient for IME |

### Comparison

Traditional IME (e.g., EVKey): 50-100 MB
GoNhanh: 25 MB (50-75% smaller)

## Security Architecture

### Memory Safety

- **Rust guarantees**: No buffer overflows, no use-after-free
- **Buffer bounds**: Typing buffer limited to 32 characters (enforced by type system)
- **FFI safety**: All C pointers marked unsafe in Rust, no raw pointer arithmetic in hot path

### Permissions

- **Accessibility API**: Only permission requested (keyboard hook capability)
- **No network**: Completely offline, no telemetry
- **No file system**: No config files, all state in UserDefaults
- **No other APIs**: No camera, microphone, location, contacts, etc.

### Offline-Only

- No updates (manual download)
- No crash reporting
- No analytics
- No user tracking
- Source code publicly auditable on GitHub

## Architectural Decisions

### Why Rust Core?

1. **Memory Safety**: Prevents entire class of bugs (buffer overflow, use-after-free)
2. **Performance**: No garbage collector, zero-cost abstractions
3. **Cross-platform**: Same core for macOS, Windows, Linux
4. **FFI Simplicity**: C ABI is straightforward, works with any language

### Why Native UI?

1. **UX Quality**: SwiftUI on macOS provides native look/feel instantly
2. **Accessibility**: Native accessibility features (VoiceOver, etc.)
3. **Integration**: Seamless with system appearance (Light/Dark mode)
4. **Performance**: No runtime overhead from UI framework

### Why Phonology Algorithm?

1. **Correctness**: Follows Vietnamese linguistic rules, not hardcoded tables
2. **Flexibility**: Easy to support both old and modern tone styles
3. **Extensibility**: Can support dialects or related languages
4. **Maintainability**: Rules are documented, easier to debug

## Future Roadmap

### Planned Platforms

- **Windows**: WPF UI + same Rust core (library ready)
- **Linux**: IBus/Fcitx + Rust core (library ready)
- **Wayland**: Modern Linux display server support

### Planned Features

- Auto-update mechanism
- User custom dictionary
- Tone style persistence per app
- Telex variant support (VIQR)
- Advanced text selection handling

## Related Documentation

- Vietnamese language system: [`docs/vietnamese-language-system.md`](vietnamese-language-system.md)
- Development guide: [`docs/development.md`](development.md)
- Common issues & fixes: [`docs/common-issues.md`](common-issues.md)
