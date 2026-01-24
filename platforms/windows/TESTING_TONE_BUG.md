# Testing Tone Mark Bug - "chào" → "chaà"

## Bug Report

**Expected**: Typing "chaof" (Telex) → "chào"
**Actual**: "chaof" → "chaà"
**App Status**: Running in Task Manager ✓
**System Tray**: Icon not visible ❌

## Diagnosis Steps

### Step 1: Build Debug Version

```bash
cd platforms/windows

# Clean previous build
rm -rf build-debug

# Configure with debug console
cmake -B build-debug -G "Visual Studio 17 2022" -A x64 -DENABLE_DEBUG_CONSOLE=ON

# Build Debug (this will show console window)
cmake --build build-debug --config Debug
```

### Step 2: Run Debug Build

```bash
# Run from build directory
.\build-debug\Debug\gonhanh.exe
```

**Console window will appear** with startup logs like:
```
═══════════════════════════════════════════════════════════
  GoNhanh Debug Console
  Vietnamese Input Method - Windows C++ Implementation
═══════════════════════════════════════════════════════════

[STARTUP] GoNhanh starting...
[INFO] [2026-01-13 10:00:00] Rust engine initialized
[INFO] [2026-01-13 10:00:00] Keyboard hook installed
[INFO] [2026-01-13 10:00:00] System tray created
[INFO] [2026-01-13 10:00:00] GoNhanh started successfully
```

### Step 3: Test in Notepad

1. **Open Notepad**
2. **Press Ctrl+Space** to enable IME (if needed)
3. **Type slowly**: `c` → `h` → `a` → `o` → `f`
4. **Watch console output** for each keystroke

### Step 4: Analyze Console Output

For each key, you should see debug output like:

```
[HOOK] VK=0x43 keycode=8 backspace=0 count=1 chars=U+0063     (c)
[HOOK] VK=0x48 keycode=4 backspace=1 count=2 chars=U+0063 U+0068     (ch)
[HOOK] VK=0x41 keycode=0 backspace=2 count=3 chars=U+0063 U+0068 U+0061     (cha)
[HOOK] VK=0x4F keycode=31 backspace=3 count=4 chars=U+0063 U+0068 U+0061 U+006F     (chao)
[HOOK] VK=0x46 keycode=3 backspace=4 count=4 chars=U+0063 U+0068 U+00E0 U+006F     (chào) ← EXPECTED
```

**Key things to check**:
- **VK codes**: Are they correct? (C=0x43, H=0x48, A=0x41, O=0x4F, F=0x46)
- **keycodes**: Are mappings correct? (C=8, H=4, A=0, O=31, F=3)
- **backspace**: For 'f' key, should be 4 (delete "chao")
- **count**: For 'f' key, should be 4 (output "chào")
- **chars**: For 'f' key, should be `[U+0063, U+0068, U+00E0, U+006F]`
  - U+0063 = 'c'
  - U+0068 = 'h'
  - U+00E0 = 'à' (a with grave/huyền tone) ← THIS IS THE CRITICAL ONE
  - U+006F = 'o'

### Step 5: Identify Root Cause

#### If console shows CORRECT codepoints but Notepad shows wrong output:

→ **Bug is in SendUnicodeText** or SendBackspaces
   - Possible timing issue with SendInput
   - Possible backspace count error
   - Possible Unicode encoding issue

#### If console shows WRONG codepoints:

→ **Bug is in Rust engine** or keycode mapping
   - Check if keycode mapping is correct
   - Run Rust tests: `cd core && cargo test`
   - Check Phonology::find_tone_position logic

#### If no console output appears:

→ **Keyboard hook not working**
   - Check if hook was installed (should see "[INFO] Keyboard hook installed")
   - Try running as Administrator
   - Check for conflicting keyboard hooks (AutoHotkey, gaming tools)

## Expected Unicode Codepoints

| Character | Unicode | Name |
|-----------|---------|------|
| c | U+0063 | Latin Small Letter C |
| h | U+0068 | Latin Small Letter H |
| à | U+00E0 | Latin Small Letter A with Grave |
| o | U+006F | Latin Small Letter O |

**NOT**: U+0061 (a) + U+00E0 (à) separately!

## Common Issues

### Issue 1: "chaà" appears in Notepad (extra 'à')

**Symptoms**: User sees "c h a à" instead of "c h à o"

**Possible causes**:
1. Backspace count is wrong (deleting 3 instead of 4)
2. Character order is wrong in chars array
3. Tone mark applied to wrong vowel (o instead of a)

**Debug**: Check the chars array in console output for 'f' key.

### Issue 2: "chao" doesn't change at all

**Symptoms**: Typing 'f' doesn't apply tone mark

**Possible causes**:
1. IME is disabled (press Ctrl+Space)
2. Keyboard hook not catching 'f' key
3. Rust engine not recognizing 'f' as tone mark key

**Debug**: Check if [HOOK] line appears for 'f' key press.

### Issue 3: Console window doesn't appear

**Symptoms**: Debug build runs but no console

**Possible causes**:
1. Not built with ENABLE_DEBUG_CONSOLE=ON
2. Build configuration is Release instead of Debug

**Fix**: Re-run cmake with `-DENABLE_DEBUG_CONSOLE=ON` and build with `--config Debug`.

## Additional Debugging with DebugView

If you can't use debug build (e.g., testing Release build), use DebugView:

1. **Download**: https://download.sysinternals.com/files/DebugView.zip
2. **Run as Administrator**: Right-click Dbgview64.exe → Run as administrator
3. **Enable capture**: Capture → Capture Global Win32
4. **Run GoNhanh**: The debug logs will appear in DebugView

Note: The `[HOOK]` logs are only compiled in Debug builds (`#ifdef _DEBUG`), so Release builds won't show keystroke details in DebugView.

## Report Results

After testing, please provide:

1. **Console output** for typing "chaof" (copy the [HOOK] lines)
2. **What appeared in Notepad** (exact characters)
3. **System info**:
   - Windows version (Win+R → winver)
   - GoNhanh build (Debug or Release)
   - Any other keyboard tools running (AutoHotkey, gaming macros, etc.)

This will help identify whether the bug is in:
- Rust engine (wrong codepoints)
- Keycode mapping (wrong VK → keycode)
- Unicode output (SendUnicodeText)
- Backspace logic (SendBackspaces)
