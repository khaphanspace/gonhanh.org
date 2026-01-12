# Phase 01 Testing Guide

**IMPORTANT:** Testing must be performed on Windows 10/11 machine after building the application.

## Prerequisites

1. Built `gonhanh.exe` from Step 2 (see [README.md](README.md))
2. `gonhanh_core.dll` in same directory as .exe
3. Notepad, Chrome, VS Code installed for testing

## Test Scenarios

### Test 3.1: Hook Installation

**Objective:** Verify SetWindowsHookEx succeeds

**Steps:**
1. Run `gonhanh.exe`
2. Verify no error dialog appears
3. Verify success message shows "started successfully"

**Expected Result:**
- ✅ No error about "Failed to install keyboard hook"
- ✅ Message box: "Gõ Nhanh started successfully!"

**Failure Modes:**
- ❌ Error dialog → Check antivirus, close other IMEs (UniKey, OpenKey, EVKey)
- ❌ 0xc000007b error → Rebuild with correct architecture (x64)

---

### Test 3.2: Key Event Capture

**Objective:** Verify worker thread receives key events from queue

**Steps:**
1. Run `gonhanh.exe`
2. Click OK on success dialog (don't close window)
3. Open Notepad
4. Type single letter: `a`

**Expected Result:**
- ✅ Letter 'a' appears in Notepad
- ✅ No crashes
- ✅ No system lag

**Failure Modes:**
- ❌ No response → Worker thread not started, check debug build
- ❌ Multiple 'a' characters → Injection loop, check LLKHF_INJECTED flag

**Debug Tips:**
```cpp
// Add to KeyboardHook::WorkerThreadFunc (after queue Pop)
OutputDebugStringA("Worker: Processing key event\n");
```

---

### Test 3.3: FFI Functionality

**Objective:** Verify ime_key returns valid ImeResult

**Steps:**
1. Run `gonhanh.exe`
2. Open Notepad
3. Type: `a` then `s`

**Expected Result:**
- ✅ First keystroke: 'a' appears
- ✅ Second keystroke: 'a' changes to 'á' (Telex: a+s = á)
- ✅ No crashes

**Failure Modes:**
- ❌ 'as' appears instead of 'á' → FFI not working, check DLL path
- ❌ Crash on second keystroke → ImeResult memory corruption, check struct alignment

**Debug Tips:**
```cpp
// Add to KeyboardHook::WorkerThreadFunc (before TextSender::Send)
if (result && result->action == 1) {
    char msg[256];
    sprintf_s(msg, "FFI: backspace=%d, count=%lld\n", result->backspace, result->count);
    OutputDebugStringA(msg);
}
```

---

### Test 3.4: Text Injection in Multiple Apps

**Objective:** Verify Vietnamese characters appear correctly

**Test Apps:**
1. Notepad (baseline)
2. Chrome (web browser)
3. VS Code (editor)
4. Terminal (command line)
5. Microsoft Word (if available)

**Test Sequence:**
For each app, type these sequences:

| Input | Expected Output | Description |
|-------|-----------------|-------------|
| `a` `s` | `á` | Tone mark (sắc) |
| `v` `i` `e` `e` `t` | `việt` | Horn mark (ơ→iê) + tone |
| `h` `a` `n` `o` `i` | `hà nội` | Multiple words |
| `d` `d` `d` | `đ` | Stroke (đ) + removal |
| `v` `i` `e` `w` `Space` | `view ` | English auto-restore |

**Expected Result:**
- ✅ All apps show correct Vietnamese characters
- ✅ No lag or stuttering
- ✅ Backspace works correctly

**Known Limitations:**
- ⚠️ UWP apps (Mail, Calculator, Settings) may NOT work - this is expected
- ⚠️ Some games with anti-cheat may block SendInput

---

### Test 3.5: Latency < 1ms

**Objective:** Measure keystroke-to-output latency

**Method 1: Visual Test (Qualitative)**
1. Run `gonhanh.exe`
2. Open Notepad
3. Type rapidly: `asdfasdfasdf`
4. Observe if there's noticeable delay

**Expected Result:**
- ✅ No perceptible lag
- ✅ Characters appear instantly

**Method 2: Performance Counter (Quantitative)**

Add this code to `KeyboardHook.cpp:WorkerThreadFunc`:

```cpp
// After RustBridge::ProcessKeyExt call
LARGE_INTEGER endTime;
QueryPerformanceCounter(&endTime);
double latencyMs = (endTime.QuadPart - event.timestamp.QuadPart) * 1000.0 /
                   s_performanceFrequency.QuadPart;

// Log latency
char msg[256];
sprintf_s(msg, "Latency: %.3f ms\n", latencyMs);
OutputDebugStringA(msg);
```

**How to View Debug Output:**
1. Open Visual Studio
2. Debug > Start Debugging (F5)
3. View > Output window
4. Type in Notepad
5. Check Output window for latency measurements

**Expected Result:**
- ✅ Average latency: 0.5-1.0 ms
- ✅ Max latency: < 2ms (99th percentile)

**Failure Modes:**
- ❌ Latency > 5ms → Check CPU usage, close background apps
- ❌ Latency > 10ms → Worker thread priority issue, verify THREAD_PRIORITY_TIME_CRITICAL

---

### Test 3.6: Stability (10-minute Typing)

**Objective:** Verify no crashes or memory leaks over extended use

**Steps:**
1. Run `gonhanh.exe`
2. Open Notepad
3. Type continuously for 10 minutes (or use auto-typing script below)
4. Monitor Task Manager for memory usage

**Auto-Typing Script (PowerShell):**
```powershell
# Run this in separate PowerShell window
Add-Type -AssemblyName System.Windows.Forms

Start-Sleep -Seconds 5  # Give time to focus Notepad

$text = "viet nam ha noi thanh pho ho chi minh "
for ($i = 0; $i < 100; $i++) {
    [System.Windows.Forms.SendKeys]::SendWait($text)
    Start-Sleep -Milliseconds 100
}
```

**Expected Result:**
- ✅ No crashes after 10 minutes
- ✅ Memory usage < 10MB (check Task Manager)
- ✅ No error dialogs
- ✅ Hook still responsive (type manually to verify)

**Failure Modes:**
- ❌ Crash → Memory corruption, check ImeResult Free calls
- ❌ Memory leak → ImeResult not freed, add RAII wrapper
- ❌ Hook stops working → Windows disabled hook (callback timeout > 200ms)

---

## Success Criteria Summary

| Test | Criteria | Pass/Fail |
|------|----------|-----------|
| 3.1 | Hook installs without error | ⏳ Pending (Windows) |
| 3.2 | Key events captured | ⏳ Pending (Windows) |
| 3.3 | FFI returns valid results | ⏳ Pending (Windows) |
| 3.4 | Vietnamese chars in all apps | ⏳ Pending (Windows) |
| 3.5 | Latency < 1ms average | ⏳ Pending (Windows) |
| 3.6 | No crashes/leaks (10min) | ⏳ Pending (Windows) |

**Overall Phase 1 Status:** ⏳ **READY FOR TESTING** (requires Windows machine)

---

## Troubleshooting

### Issue: Hook not installing

**Error:** "Failed to install keyboard hook"

**Solutions:**
1. Close other Vietnamese IMEs (UniKey, OpenKey, EVKey)
2. Add `gonhanh.exe` to antivirus exclusions
3. Run as administrator (right-click > Run as administrator)
4. Check Windows Event Viewer for system errors

### Issue: No Vietnamese characters appear

**Symptom:** Typing `a` `s` shows "as" instead of "á"

**Solutions:**
1. Verify `gonhanh_core.dll` is in same directory as .exe
2. Check DLL architecture (must be x64 if .exe is x64)
3. Rebuild Rust core with correct target: `cargo build --release --target x86_64-pc-windows-msvc`

### Issue: Crash on keystroke

**Symptom:** Application crashes when typing

**Solutions:**
1. Check Visual Studio Output window for exception details
2. Verify ImeResult struct matches Rust definition exactly
3. Add null check before TextSender::Send
4. Rebuild in Debug mode for better error messages

### Issue: High CPU usage

**Symptom:** CPU usage > 5% when idle

**Solutions:**
1. Verify worker thread has Sleep(1) when queue is empty
2. Check for infinite loops in WorkerThreadFunc
3. Ensure queue Push/Pop are correct (no busy-wait)

### Issue: Lag after some time

**Symptom:** Typing becomes slow after 5-10 minutes

**Solutions:**
1. Check Task Manager for memory leaks (> 20MB)
2. Verify ImeResult is freed after every ProcessKey call
3. Check if queue is filling up (add queue size logging)

---

## Debug Build Configuration

For troubleshooting, build in Debug mode:

```powershell
cd platforms\windows-cpp
cmake -B build -G "Visual Studio 17 2022" -A x64
cmake --build build --config Debug
```

Debug build includes:
- Latency logging (OutputDebugString)
- Assertions enabled
- No optimizations (easier to debug)
- Debug CRT (better error messages)

---

## Next Steps After Testing

1. If all tests pass → Proceed to Step 4: Code Review
2. If tests fail → Fix issues, re-test, document failures
3. Update this file with actual test results (change ⏳ to ✅ or ❌)

---

## Test Results (To be filled on Windows)

**Tester:** _________________
**Date:** _________________
**Windows Version:** _________________
**Build Config:** Release / Debug

| Test | Result | Notes |
|------|--------|-------|
| 3.1 | ⏳ | |
| 3.2 | ⏳ | |
| 3.3 | ⏳ | |
| 3.4 | ⏳ | |
| 3.5 | ⏳ | Latency: ____ ms |
| 3.6 | ⏳ | Memory: ____ MB |

**Overall:** ⏳ PENDING
