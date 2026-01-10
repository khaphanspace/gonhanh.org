#pragma once

#include <windows.h>
#include <string>
#include <cstdint>

namespace gonhanh {

// High-performance text sender using SendInput API
// Pre-allocates buffers to minimize allocations
class TextSender {
public:
    static TextSender& instance();

    // Send text with optional backspaces before
    // backspace_count: number of backspaces to send before text
    void send_text(const std::wstring& text, uint8_t backspace_count = 0);

    // Send only backspaces
    void send_backspaces(uint8_t count);

    // Marker for injected keys (same as KeyboardHook)
    static constexpr ULONG_PTR INJECTED_KEY_MARKER = 0x474E4820;

private:
    TextSender();
    ~TextSender() = default;
    TextSender(const TextSender&) = delete;
    TextSender& operator=(const TextSender&) = delete;

    // Pre-allocated buffer for SendInput
    static constexpr size_t MAX_INPUTS = 512;  // Max 256 chars * 2 (down+up) + backspaces
    INPUT inputs_[MAX_INPUTS];
};

} // namespace gonhanh
