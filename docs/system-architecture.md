# Gõ Nhanh: System Architecture

## High-Level Architecture

```
┌──────────────────────────────────────────┐   ┌──────────────────────────────────────────┐
│         macOS Application                │   │      Windows Application (Phase 1)       │
│                                          │   │                                          │
│  ┌────────────────────────────────┐     │   │  ┌────────────────────────────────┐     │
│  │     SwiftUI Menu Bar           │     │   │  │   WPF/.NET 8 System Tray       │     │
│  │  • Input method selector       │     │   │  │  • Input method selector       │     │
│  │  • Enable/disable toggle       │     │   │  │  • Enable/disable toggle       │     │
│  │  • Settings, About, Update     │     │   │  │  • Settings, About, Update     │     │
│  └────────────┬────────────────────┘     │   │  └────────────┬────────────────────┘     │
│               │                          │   │               │                          │
│  ┌────────────▼────────────────────┐     │   │  ┌────────────▼────────────────────┐     │
│  │ CGEventTap Keyboard Hook        │     │   │  │ SetWindowsHookEx (WH_KEYBOARD_LL) │     │
│  │ • Intercepts keyDown events     │     │   │  │ • Global low-level keyboard hook│     │
│  │ • Smart text replacement        │     │   │  │ • SendInput for text injection  │     │
│  └────────────┬────────────────────┘     │   │  └────────────┬────────────────────┘     │
│               │                          │   │               │                          │
│  ┌────────────▼────────────────────┐     │   │  ┌────────────▼────────────────────┐     │
│  │    RustBridge (FFI Layer)       │     │   │  │   rust_bridge.cpp (C++/FFI)    │     │
│  │  • C ABI function calls         │     │   │  │  • UTF-32 ↔ UTF-16 conversion   │     │
│  │  • Swift memory safety          │     │   │  │  • RAII ImeResultGuard          │     │
│  └────────────┬────────────────────┘     │   │  │  • Windows API wrappers         │     │
│               │                          │   │  └────────────┬────────────────────┘     │
└───────────────┼──────────────────────────┘   └───────────────┼──────────────────────────┘
                │                                               │
                └───────────────────┬──────────────────────────┘
                                    │
                         extern "C" / C++20 FFI
                                    ↓
         ┌─────────────────────────────────────────────────────┐
         │  Rust Core Engine (Static Lib - Platform-Agnostic) │
         │  Compiled: staticlib + cdylib                       │
         │  CRT Linking: MSVC static (-C target-feature...)    │
         │  Size: ~150KB (release, stripped, LTO enabled)      │
         │  7-Stage Validation-First Pipeline                 │
         └─────────────────────────────────────────────────────┘
                            ↓
         ┌─────────────────────────────────────────────┐
         │          Input Method Layer                 │
         │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
         │  │ Telex    │  │ VNI      │  │ Shortcut │  │
         │  │ s/f/r/x/j   1-5/6-8/9  │  │ Priority │  │
         │  └──────────┘  └──────────┘  └──────────┘  │
         └─────────────────────────────────────────────┘
                            ↓
         ┌─────────────────────────────────────────────┐
         │     7-Stage Processing Pipeline             │
         │  1. Stroke (đ/Đ)                           │
         │  2. Tone Marks (sắc/huyền/hỏi/ngã/nặng)    │
         │  3. Vowel Marks (circumflex/horn/breve)    │
         │  4. Mark Removal (revert)                  │
         │  5. W-Vowel (Telex "w"→"ư")               │
         │  6. Normal Letter                          │
         │  7. Shortcut Expansion                     │
         └─────────────────────────────────────────────┘
                            ↓
         ┌─────────────────────────────────────────────┐
         │  Validation Rules (Before Transform)        │
         │  1. Must have vowel                         │
         │  2. Valid initials only                     │
         │  3. All chars parsed                        │
         │  4. Spelling rules (c/k/g restrictions)     │
         │  5. Valid finals only                       │
         └─────────────────────────────────────────────┘
                            ↓
         ┌─────────────────────────────────────────────┐
         │  Transform & Data Layer                     │
         │  • Vowel table: 72 entries                  │
         │  • Character mappings                       │
         │  • Phonology constants                      │
         └─────────────────────────────────────────────┘
                            ↓
         ┌─────────────────────────────────────────────┐
         │  Result Structure                           │
         │  • action: None/Send/Restore                │
         │  • backspace: N chars to delete             │
         │  • chars: [u32; 32] UTF-32 output          │
         │  • count: valid char count                  │
         └─────────────────────────────────────────────┘
```

## Data Flow: Keystroke to Output

### Example: Typing "á" in Telex

```
User types: [a] then [s]

Step 1: Key 'a' pressed
  ├─ CGEventTap captures keyDown
  ├─ RustBridge.processKey(keyCode=0x00, caps=false, ctrl=false)
  ├─ Rust: ime_key() called
  ├─ Engine:
  │  ├─ Append 'a' to buffer
  │  ├─ Validate: "a" is valid (vowel alone)
  │  ├─ No transform yet (single char, waiting for next)
  │  └─ Return Action::None (pass through)
  ├─ Swift: No action, let 'a' appear naturally
  └─ Output: User sees 'a' typed

Step 2: Key 's' pressed (sắc mark in Telex)
  ├─ CGEventTap captures keyDown
  ├─ RustBridge.processKey(keyCode=0x01, caps=false, ctrl=false)
  ├─ Rust: ime_key() called
  ├─ Engine:
  │  ├─ Check buffer context: "a" + "s" → sắc mark
  │  ├─ Validation: "á" is valid Vietnamese vowel
  │  ├─ Transform: Apply sắc mark to 'a' → 'á'
  │  ├─ Check shortcuts: No expansion needed
  │  ├─ Return Action::Send {
  │  │    backspace: 1,  // Delete 'a'
  │  │    chars: ['á']   // Insert 'á'
  │  └─ }
  ├─ Swift:
  │  ├─ Send 1 backspace (delete 'a')
  │  ├─ Send 'á' (via Unicode keyboard event)
  │  └─ 's' keystroke consumed (not passed through)
  └─ Output: User sees 'á' (exactly 1 character)

Result: "á" displayed instead of "as"
Latency: ~0.2-0.5ms total (Rust engine: <0.1ms)
```

### Example: Typing "không" with Shortcut

```
User types: [k] [h] [o] [n] [g] [space]  (or defined shortcut key)

Setup: User defined shortcut "khong" → "không"

Processing:
  Step 1-4: Build buffer "khon" → valid syllable, wait
  Step 5: Shortcut lookup
    ├─ Check if "khong" matches any user abbreviation
    ├─ Match found: "khong" → "không"
    └─ Return: backspace: 5, chars: ['k','h','ô','n','g']

  Swift execution:
    ├─ Delete 5 chars (k, h, o, n, g)
    ├─ Insert 5 chars (k, h, ô, n, g)
    └─ No change visible but ô is now correct diacritic
```

## FFI Interface Specification

### Function Signatures (C ABI)

```c
// Initialize engine (call once)
void ime_init(void);

// Process keystroke
typedef struct {
    uint32_t chars[32];      // UTF-32 output characters
    uint8_t action;          // 0=None, 1=Send, 2=Restore
    uint8_t backspace;       // Number of chars to delete
    uint8_t count;           // Number of valid chars
    uint8_t _pad;            // Padding for alignment
} ImeResult;

ImeResult* ime_key(uint16_t keycode, bool caps, bool ctrl);

// Set input method (0=Telex, 1=VNI)
void ime_method(uint8_t method);

// Enable/disable engine
void ime_enabled(bool enabled);

// Clear buffer (word boundary)
void ime_clear(void);

// Free result (caller must call this exactly once per ime_key)
void ime_free(ImeResult* result);
```

### Action Types

| Value | Name | Meaning | Response |
|-------|------|---------|----------|
| 0 | None | No transformation, pass key through | Send key to app |
| 1 | Send | Transform matched, replace text | Backspace + insert |
| 2 | Restore | Undo previous transform | Not currently used |

### Memory Ownership

- **FFI Responsibility**: Rust engine allocates Result struct
- **Caller Responsibility**: Swift must call `ime_free(result)` to deallocate
- **Safety**: Use `defer { ime_free(ptr) }` to guarantee cleanup even on early return

## Platform Integration Details

### macOS CGEventTap

#### Event Interception
```swift
// Tap into keyboard events system-wide
let eventMask: CGEventMask = (1 << CGEventType.keyDown.rawValue)

let tap = CGEvent.tapCreate(
    tap: .cghidEventTap,                    // Try HID event tap first
    place: .headInsertEventTap,             // Insert at head of chain
    options: .defaultTap,                   // Can modify/drop events
    eventsOfInterest: eventMask,            // Only keyDown
    callback: keyboardCallback,             // Our handler
    userInfo: nil
)
```

#### Fallback Strategy
```
1st attempt: CGEventTapType.cghidEventTap
   └─ If fails → 2nd attempt: cgSessionEventTap
      └─ If fails → 3rd attempt: cgAnnotatedSessionEventTap
         └─ If all fail → Accessibility permission required
```

#### Text Replacement Methods

**Method 1: Backspace (default)**
```
Send: BS BS ... BS (backspace count times)
      ↓ (0.8ms delay)
Send: Unicode input event with output chars
```

**Method 2: Selection (autocomplete apps)**
```
Send: Shift+Left Shift+Left ... Shift+Left (select chars)
      ↓
Send: Unicode input event (replaces selection)
```

#### Engine Result Cases

| Case | Action | Backspace | Output | Example (Telex) | Example (VNI) |
|------|--------|-----------|--------|-----------------|---------------|
| **Pass-through** | None | 0 | - | Normal letters, ctrl+key | Normal letters, ctrl+key |
| **Mark (dấu thanh)** | Send | 1 | vowel+mark | `as` → `á` | `a1` → `á` |
| **Tone (dấu mũ/móc)** | Send | 1+ | vowel+tone | `aa` → `â`, `ow` → `ơ` | `a6` → `â`, `o7` → `ơ` |
| **Stroke (đ)** | Send | 1+ | đ | `dd` → `đ` | `d9` → `đ` |
| **Compound ươ** | Send | 2 | ươ | `uow` → `ươ` | `u7o7` → `ươ` |
| **Mark reposition** | Send | 2+ | repositioned | `hoaf` → `hoà` | `hoa2` → `hoà` |
| **Revert (double key)** | Send | 1+ | original+key | `ass` → `as` | `a11` → `a1` |
| **Word shortcut** | Send | N | expanded | `vn ` → `Việt Nam ` | same |
| **W as ư (Telex)** | Send | 0 | ư | `w` → `ư`, `nhw` → `như` | N/A |

#### Text Replacement Strategy Matrix

| Backspace Count | Method | Reason | UX Impact |
|-----------------|--------|--------|-----------|
| **0** | None | No replacement needed (W→ư adds char) | ✅ Best - instant |
| **1** | Backspace | Single char, fast, no flicker | ✅ Good - imperceptible |
| **2-3** | Backspace | Compound vowels, still fast | ⚡ OK - minimal delay |
| **4+** | Backspace | Long shortcuts | ⚠️ May see brief flicker |

#### App Compatibility Matrix

**Legend:** ✅ OK | ⚠️ Sometimes issues | ❌ Known issues

##### Browsers

| App | Bundle ID | Body Text | Address Bar | Search Box |
|-----|-----------|-----------|-------------|------------|
| Chrome | `com.google.Chrome` | ✅ Backspace | ❌ Selection | ⚠️ Selection |
| Safari | `com.apple.Safari` | ✅ Backspace | ❌ Selection | ⚠️ Selection |
| Firefox | `org.mozilla.firefox` | ✅ Backspace | ❌ Selection | ⚠️ Selection |
| Edge | `com.microsoft.edgemac` | ✅ Backspace | ❌ Selection | ⚠️ Selection |
| Arc | `company.thebrowser.Browser` | ✅ Backspace | ❌ Selection | ⚠️ Selection |

##### Office & Productivity

| App | Bundle ID | Issue | Method |
|-----|-----------|-------|--------|
| Excel | `com.microsoft.Excel` | Cell autocomplete | Selection |
| Word | `com.microsoft.Word` | Suggestion popup | Selection |
| PowerPoint | `com.microsoft.Powerpoint` | Text box | Selection |
| Pages | `com.apple.iWork.Pages` | None (native) | Backspace |
| Numbers | `com.apple.iWork.Numbers` | None (native) | Backspace |
| Google Docs | (web) | Canvas-based | Backspace |

##### IDEs & Editors

| App | Bundle ID | Issue | Method |
|-----|-----------|-------|--------|
| VS Code | `com.microsoft.VSCode` | None | Backspace |
| Xcode | `com.apple.dt.Xcode` | None (native) | Backspace |
| Android Studio | `com.google.android.studio` | Autocomplete popup | Selection |
| IntelliJ | `com.jetbrains.intellij` | Autocomplete | Selection |
| WebStorm | `com.jetbrains.WebStorm` | Autocomplete | Selection |
| Sublime Text | `com.sublimetext.*` | None | Backspace |

##### Electron Apps

| App | Bundle ID | Issue | Method |
|-----|-----------|-------|--------|
| Slack | `com.tinyspeck.slackmacgap` | Sometimes lost char | Backspace |
| Discord | `com.hnc.Discord` | Electron IME bugs | Backspace |
| Notion | `notion.id` | Sometimes sticky | Backspace |
| Obsidian | `md.obsidian` | None | Backspace |
| Figma | `com.figma.Desktop` | Canvas text | Backspace |

##### Terminal & Chat

| App | Bundle ID | Issue | Method |
|-----|-----------|-------|--------|
| Terminal | `com.apple.Terminal` | None (native) | Backspace |
| iTerm2 | `com.googlecode.iterm2` | None | Backspace |
| Messages | `com.apple.MobileSMS` | None (native) | Backspace |
| Telegram | `ru.keepcoder.Telegram` | None (native) | Backspace |
| Zalo | `com.vng.zalo` | None | Backspace |

#### Detection Strategy

Instead of app-based detection, use **Accessibility API** to detect focused element type:

| AX Role | AX Subrole | Context | Method |
|---------|------------|---------|--------|
| `AXComboBox` | - | Address bar, dropdown | Selection |
| `AXTextField` | `AXSearchField` | Search with autocomplete | Selection |
| `AXTextField` | - | Form input | Backspace |
| `AXTextArea` | - | Multiline text | Backspace |
| `AXWebArea` | - | Web content editable | Backspace |

**Priority rules:**
1. `AXComboBox` → Always Selection (address bars, dropdowns)
2. `AXSearchField` subrole → Selection (search boxes)
3. JetBrains apps (`com.jetbrains.*`) → Selection (autocomplete)
4. Microsoft Excel → Selection (cell autocomplete)
5. **Everything else** → Backspace (default, ~90% of cases)

#### Current Implementation

```swift
// Accessibility-based detection (preferred)
func getReplacementMethod() -> ReplacementMethod {
    // Get focused element info
    guard let info = getFocusedElementInfo() else {
        return .backspace // Default
    }

    // Rule 1: ComboBox = address bar, dropdown
    if info.role == "AXComboBox" {
        return .selection
    }

    // Rule 2: Search field with autocomplete
    if info.role == "AXTextField" && info.subrole == "AXSearchField" {
        return .selection
    }

    // Rule 3: JetBrains IDEs
    if info.bundleId.hasPrefix("com.jetbrains") {
        return .selection
    }

    // Rule 4: Microsoft Excel
    if info.bundleId == "com.microsoft.Excel" {
        return .selection
    }

    // Default: Backspace (fast, no flicker)
    return .backspace
}
```

#### Known Issues & Trade-offs

| Issue | Cause | Solution | Status |
|-------|-------|----------|--------|
| **Dính chữ (address bar)** | Autocomplete intercepts backspace | AX detection → Selection | ✅ Fixed |
| **Flicker (selection)** | Multiple Shift+Left visible | Only use when needed | ✅ Minimized |
| **JetBrains autocomplete** | Code completion popup | Bundle ID detection | ✅ Fixed |
| **Excel cell autocomplete** | Cell suggestions | Bundle ID detection | ✅ Fixed |

### Windows SetWindowsHookEx Integration (Phase 2 Complete)

#### Build System: Corrosion + CMake

**Core Integration Chain:**
```
CMakeLists.txt (C++20 + MSVC)
    ↓
Corrosion v0.5 (git tag)
    ↓
Import gonhanh-core (Cargo.toml: staticlib crate-type)
    ↓
MSVC Static CRT linking: -C target-feature=+crt-static
    ↓
Result: Single .exe, zero DLL dependencies
```

**Key Files (Phase 2):**
- `platforms/windows/CMakeLists.txt` - Corrosion integration, MSVC CRT config (+1 line)
- `platforms/windows/src/keyboard_hook.h` - KeyboardHook class definition (29 LOC)
- `platforms/windows/src/keyboard_hook.cpp` - Hook implementation & VK→macOS mapping (222 LOC)
- `platforms/windows/src/rust_bridge.h` - C++ FFI wrapper (RAII guards)
- `platforms/windows/src/rust_bridge.cpp` - UTF-32 ↔ UTF-16 conversion
- `platforms/windows/src/main.cpp` - WinMain entry point (+37 LOC for message loop)
- `platforms/windows/resources/resources.rc` - Version metadata

#### FFI Memory Model

**Buffer Size Guarantee:**
```c
// Rust side (core/src/lib.rs)
const MAX_BUFFER_SIZE: usize = 256;

struct ImeResult {
    chars: [u32; 256],      // UTF-32 output
    action: u8,             // 0=None, 1=Send, 2=Restore
    backspace: u8,
    count: u8,
    flags: u8
}

// Windows side (rust_bridge.h)
struct ImeResult {
    uint32_t chars[256];    // Matches Rust MAX constant
    uint8_t action;
    uint8_t backspace;
    uint8_t count;
    uint8_t flags;
};
```

**RAII Memory Safety:**
```cpp
// C++ wrapper prevents leaks via scope
class ImeResultGuard {
    ImeResult* ptr_;
public:
    ~ImeResultGuard() { if (ptr_) ime_free(ptr_); }
    ImeResultGuard(const ImeResultGuard&) = delete;      // No copy
    ImeResultGuard(ImeResultGuard&&) = delete;            // No move
};

// Usage: scope-based cleanup
{
    ImeResultGuard result(ime_key(keycode, caps, ctrl));
    // ... process result
}  // automatic cleanup here
```

#### UTF Conversion Pipeline

**UTF-32 (Rust Engine) → UTF-16 (Windows API):**
```cpp
// Handles all Unicode ranges including surrogate pairs
void SendUnicodeText(const uint32_t* chars, uint8_t count) {
    for (uint8_t i = 0; i < count; ++i) {
        uint32_t cp = chars[i];

        if (cp <= 0xFFFF) {
            // BMP: single UTF-16 unit (Latin, Greek, Vietnamese)
            INPUT input = {};
            input.type = INPUT_KEYBOARD;
            input.ki.wVk = 0;              // Use Unicode, not VK
            input.ki.wScan = (WORD)cp;
            input.ki.dwFlags = KEYEVENTF_UNICODE;
            SendInput(1, &input, sizeof(INPUT));
        } else {
            // Supplementary planes: surrogate pair (emoji, rare chars)
            cp -= 0x10000;
            WORD high = 0xD800 + (WORD)(cp >> 10);
            WORD low = 0xDC00 + (WORD)(cp & 0x3FF);

            // Send both surrogates via SendInput
            SendInput(4, surrogatePair, sizeof(INPUT));
        }
    }
}
```

**Expected Codepoint Ranges (Vietnamese):**
- BMP (< 0x10000): ă, â, ê, ô, ơ, ư, đ (all native Vietnamese, no surrogates)
- Special: Zero-width joiners (0x200D) handled via direct mapping

#### Keyboard Hook Implementation (Phase 2)

**Global Message-Only Window Requirement:**
```cpp
// CRITICAL: WH_KEYBOARD_LL requires a message loop
HWND hwnd = CreateWindowEx(
    0, WINDOW_CLASS, L"GoNhanhMsg",
    0, 0, 0, 0, 0,
    HWND_MESSAGE,  // Message-only window (no desktop visible)
    NULL, hInstance, NULL
);

// Install hook AFTER window creation
auto& hook = gonhanh::KeyboardHook::Instance();
hook.Install();

// Message loop (REQUIRED for WH_KEYBOARD_LL to function)
MSG msg;
while (GetMessage(&msg, NULL, 0, 0)) {
    TranslateMessage(&msg);
    DispatchMessage(&msg);
}
```

**Why Message Loop Required:**
- `WH_KEYBOARD_LL` is a global low-level hook (system-wide interception)
- Windows requires a message queue to deliver hook notifications
- Message-only window (HWND_MESSAGE) is invisible but maintains queue
- Loop must run on same thread that called SetWindowsHookEx

**KeyboardHook Class Architecture:**

```cpp
class KeyboardHook {
private:
    static HHOOK hook_;
    bool enabled_ = true;
    bool processing_ = false;  // Reentrancy guard

    static LRESULT CALLBACK LowLevelKeyboardProc(...);
};
```

**Hook Callback Flow:**

```
LowLevelKeyboardProc(nCode, wParam, lParam)
    ↓
1. Check nCode == HC_ACTION (only process real events)
    ↓
2. Check LLKHF_INJECTED flag (skip injected keys to prevent loops)
    ↓
3. Only process WM_KEYDOWN & WM_SYSKEYDOWN (ignore repeats)
    ↓
4. Handle Ctrl+Space toggle (BEFORE enabled check, always works)
    ↓
5. Check enabled flag (pass through if off)
    ↓
6. Check reentrancy guard (prevent recursive processing)
    ↓
7. Convert VK → macOS keycode via VkToMacKeycode()
    ↓
8. Extract modifier states: Caps, Ctrl, Shift
    ↓
9. Call Rust engine: ime_key_ext(keycode, caps, ctrl, shift)
    ↓
10. If action == 1 (transform):
    - Send backspaces via SendInput (VK_BACK)
    - Send replacement chars via SendUnicodeText()
    - Return 1 (suppress original keystroke)
    ↓
11. Else: CallNextHookEx (pass through)
```

**VK→macOS Keycode Mapping (46 Keys):**

| Category | Count | Examples | macOS Codes |
|----------|-------|----------|------------|
| Letters A-Z | 26 | A→0x00, B→0x0B, ..., Z→0x06 | Verified vs core/src/data/keys.rs |
| Numbers 0-9 | 10 | 0→0x1D, 1→0x12, ..., 9→0x19 | Top row number keys |
| Special Keys | 10 | Space→0x31, Return→0x24, Back→0x33, Esc→0x35, []→0x21/0x1E | Common symbols |
| **Total** | **46** | - | Cross-referenced with Rust core |

**Reentrancy Guard & Injection Prevention:**

```cpp
// Check injected flag (prevents infinite loops)
if (kb->flags & LLKHF_INJECTED) {
    return CallNextHookEx(...);  // Pass through without processing
}

// Reentrancy guard (prevents concurrent processing of same engine)
if (g_instance->processing_) {
    return CallNextHookEx(...);  // Pass through
}

// Set flag during engine call
g_instance->processing_ = true;
ImeResultGuard result(ime_key_ext(keycode, caps, ctrl, shift));
g_instance->processing_ = false;
```

**Why Both Guards Needed:**
- `LLKHF_INJECTED`: Filters out our own SendInput calls (OS-level flag)
- `processing_`: Handles edge case where callback fires before our SendInput completes
- Combined: Eliminates infinite loops & concurrent Rust engine calls

**Backspace Implementation:**

```cpp
static void SendBackspaces(int count) {
    for (int i = 0; i < count; ++i) {
        INPUT input = {};
        input.type = INPUT_KEYBOARD;
        input.ki.wVk = VK_BACK;

        // Key down
        input.ki.dwFlags = 0;
        SendInput(1, &input, sizeof(INPUT));

        // Key up
        input.ki.dwFlags = KEYEVENTF_KEYUP;
        SendInput(1, &input, sizeof(INPUT));
    }
}
```

**Ctrl+Space Toggle (Global Hotkey):**

```cpp
// Special case: Ctrl+Space handled BEFORE enabled check
if (kb->vkCode == VK_SPACE && (GetKeyState(VK_CONTROL) & 0x8000)) {
    if (g_instance) {
        g_instance->Toggle();  // Toggle enabled_ state
    }
    return 1;  // Suppress key (don't pass to apps)
}

// This ordering ensures:
// - Toggle always works even if engine disabled
// - Toggle is global (works in any app)
// - Ctrl+Space never appears in text
```

**Advantages (vs Global Input Method):**
- Global keyboard interception (works before app input handling)
- Minimal overhead (kernel calls only for events)
- No app modification needed
- Persistent across app switches
- Works in secure contexts (lockscreen, elevated apps)

### Accessibility Permission (macOS)

#### macOS System Requirement
- **API**: `AXIsProcessTrusted()` checks if app has Accessibility permission
- **User Flow**:
  1. App requests permission on first run
  2. User goes to: System Settings → Privacy & Security → Accessibility
  3. User adds GoNhanh to the list
  4. App restart required to acquire permissions
  5. Once granted, app can create CGEventTap

#### Permission Checking
```swift
// Check permission before starting keyboard hook
let trusted = AXIsProcessTrusted()
if !trusted {
    // Prompt and open System Settings
    let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true]
    AXIsProcessTrustedWithOptions(options as CFDictionary)
}
```

### Global Hotkey: Ctrl+Space

```swift
// Virtual keycode 0x31 = Space
// Flag: maskControl, NOT maskCommand

func isToggleHotkey(_ keyCode: UInt16, _ flags: CGEventFlags) -> Bool {
    keyCode == 0x31 &&
    flags.contains(.maskControl) &&
    !flags.contains(.maskCommand)  // Exclude Cmd+Space (macOS Spotlight)
}

// When matched: Post NotificationCenter event
NotificationCenter.default.post(name: .toggleVietnamese, object: nil)

// Consume event (don't pass to app)
return nil
```

## Component Interactions

### Initialization Sequence
```
1. AppDelegate.applicationDidFinishLaunching
   ├─ Show OnboardingView (if first run)
   └─ On complete: MenuBarController.init()

2. MenuBarController.init()
   ├─ Create status bar icon
   ├─ Load settings from UserDefaults
   ├─ If accessibility trusted: startEngine()
   └─ Otherwise: show permission prompt

3. startEngine()
   ├─ RustBridge.initialize()
   │  └─ Call ime_init() (once, thread-safe)
   ├─ KeyboardHookManager.shared.start()
   │  └─ Create CGEventTap, enable listening
   ├─ RustBridge.setEnabled(true)
   └─ RustBridge.setMethod(userMethod)
```

### Runtime Flow
```
User types key
   ↓
CGEventTap callback fires
   ↓
Extract keycode + modifier flags
   ↓
Check global hotkey (Ctrl+Space) → Toggle Vietnamese
   ↓
Call RustBridge.processKey()
   ├─ Call ime_key(keycode, caps, ctrl)
   ├─ Receive ImeResult
   ├─ Extract UTF-32 chars → Character array
   └─ Return (backspaceCount, chars) tuple
   ↓
If transformation:
   ├─ Send backspaces (CGEvent)
   ├─ Send Unicode replacement
   └─ Consume original key (return nil)
   ↓
Else: Pass through (return unmodified event)
   ↓
Visible to user as transformed or original text
```

## Performance Characteristics

### Latency Budget
| Component | Time | Notes |
|-----------|------|-------|
| CGEventTap callback | ~50μs | System kernel time |
| Rust ime_key() | ~100-200μs | Engine processing |
| Swift RustBridge | ~50μs | FFI overhead + result conversion |
| CGEvent sending | ~100-200μs | Posting to event tap |
| **Total** | **~300-500μs** | <1ms requirement met |

### Memory Profile
| Component | Size | Notes |
|-----------|------|-------|
| Rust engine (static) | ~150KB | Tables + code |
| Swift runtime | ~4.5MB | Standard SwiftUI overhead |
| Buffer (64 chars) | ~200B | Circular buffer per engine instance |
| **Total** | **~5MB** | Matches requirement |

### Scalability
- **Multi-user**: App per user, each runs own engine instance
- **Concurrent**: Mutex-protected ENGINE global (thread-safe)
- **Continuous**: No memory leaks (tested with 160+ tests)
- **No limits**: Can type indefinitely without performance degradation

---

**Last Updated**: 2025-12-14
**Architecture Version**: 2.0 (Validation-First, Cross-Platform)
**Platforms**: macOS (v1.0.21+, CGEventTap), Windows (production, SetWindowsHookEx), Linux (beta, Fcitx5)
**Diagram Format**: ASCII (compatible with all documentation viewers)
**Codebase Metrics**: 99,444 tokens, 380,026 chars, 78 files (per repomix analysis)
