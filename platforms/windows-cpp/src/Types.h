#pragma once
#include <cstdint>
#include <Windows.h>

// Must match Rust ImeResult struct exactly (see docs/API_SPEC.md)
struct ImeResult {
    uint8_t action;      // 0=None, 1=Send, 2=Restore
    uint8_t backspace;   // Number of chars to delete
    uint32_t chars[16];  // UTF-32 codepoints to insert
    int64_t count;       // Valid char count (0-16)
};

// Key event passed from hook callback to worker thread
struct KeyEvent {
    uint16_t vkCode;     // Windows virtual keycode
    bool isKeyDown;      // true if key press, false if key release
    bool isCaps;         // CapsLock state
    bool isCtrl;         // Ctrl/Cmd modifier
    bool isShift;        // Shift modifier
    LARGE_INTEGER timestamp;  // QueryPerformanceCounter for latency measurement
};
