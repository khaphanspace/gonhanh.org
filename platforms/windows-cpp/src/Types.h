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

// Compile-time verification that struct matches Rust FFI layout
// Rust: #[repr(C)] pub struct ImeResult { action: u8, backspace: u8, chars: [u32; 16], count: i64 }
// Expected: 1 + 1 + (4*16) + 8 = 74 bytes → aligned to 80 bytes (8-byte boundary)
static_assert(sizeof(ImeResult) == 80, "ImeResult size mismatch with Rust FFI");
static_assert(offsetof(ImeResult, action) == 0, "ImeResult::action offset mismatch");
static_assert(offsetof(ImeResult, backspace) == 1, "ImeResult::backspace offset mismatch");
static_assert(offsetof(ImeResult, chars) == 4, "ImeResult::chars offset mismatch");
static_assert(offsetof(ImeResult, count) == 72, "ImeResult::count offset mismatch");

// Key event passed from hook callback to worker thread
struct KeyEvent {
    uint16_t vkCode;     // Windows virtual keycode
    bool isKeyDown;      // true if key press, false if key release
    bool isCaps;         // CapsLock state
    bool isCtrl;         // Ctrl/Cmd modifier
    bool isShift;        // Shift modifier
    LARGE_INTEGER timestamp;  // QueryPerformanceCounter for latency measurement
};
