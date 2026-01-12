#pragma once
#include "Types.h"
#include <cstdint>

// Text injection via SendInput with Unicode support
// Handles backspace (delete previous chars) + insert (UTF-32 Vietnamese chars)
class TextSender {
public:
    // Delete constructors to prevent instantiation
    TextSender() = delete;
    TextSender(const TextSender&) = delete;
    TextSender& operator=(const TextSender&) = delete;

    // Send text based on ImeResult
    // backspaceCount: Number of characters to delete
    // chars: UTF-32 codepoints to insert
    // charCount: Number of valid codepoints in chars array
    static void Send(uint8_t backspaceCount, const uint32_t* chars, int64_t charCount);

private:
    // Send N backspaces
    static void SendBackspaces(int count);

    // Send single Unicode character (handles surrogate pairs for chars > 0xFFFF)
    static void SendUnicode(uint32_t codepoint);
};
