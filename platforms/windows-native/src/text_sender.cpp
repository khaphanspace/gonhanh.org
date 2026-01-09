#include "text_sender.h"
#include <cstring>

namespace gonhanh {

TextSender& TextSender::instance() {
    static TextSender instance;
    return instance;
}

TextSender::TextSender() {
    // Zero-initialize the buffer
    std::memset(inputs_, 0, sizeof(inputs_));
}

void TextSender::send_text(const std::wstring& text, uint8_t backspace_count) {
    if (text.empty() && backspace_count == 0) return;

    size_t input_count = 0;

    // Add backspaces first
    for (uint8_t i = 0; i < backspace_count && input_count < MAX_INPUTS - 1; ++i) {
        // Key down
        inputs_[input_count].type = INPUT_KEYBOARD;
        inputs_[input_count].ki.wVk = VK_BACK;
        inputs_[input_count].ki.wScan = 0;
        inputs_[input_count].ki.dwFlags = 0;
        inputs_[input_count].ki.time = 0;
        inputs_[input_count].ki.dwExtraInfo = INJECTED_KEY_MARKER;
        input_count++;

        // Key up
        inputs_[input_count].type = INPUT_KEYBOARD;
        inputs_[input_count].ki.wVk = VK_BACK;
        inputs_[input_count].ki.wScan = 0;
        inputs_[input_count].ki.dwFlags = KEYEVENTF_KEYUP;
        inputs_[input_count].ki.time = 0;
        inputs_[input_count].ki.dwExtraInfo = INJECTED_KEY_MARKER;
        input_count++;
    }

    // Add Unicode text
    for (wchar_t ch : text) {
        if (input_count >= MAX_INPUTS - 1) break;

        // Key down (Unicode)
        inputs_[input_count].type = INPUT_KEYBOARD;
        inputs_[input_count].ki.wVk = 0;
        inputs_[input_count].ki.wScan = ch;
        inputs_[input_count].ki.dwFlags = KEYEVENTF_UNICODE;
        inputs_[input_count].ki.time = 0;
        inputs_[input_count].ki.dwExtraInfo = INJECTED_KEY_MARKER;
        input_count++;

        // Key up (Unicode)
        inputs_[input_count].type = INPUT_KEYBOARD;
        inputs_[input_count].ki.wVk = 0;
        inputs_[input_count].ki.wScan = ch;
        inputs_[input_count].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
        inputs_[input_count].ki.time = 0;
        inputs_[input_count].ki.dwExtraInfo = INJECTED_KEY_MARKER;
        input_count++;
    }

    if (input_count > 0) {
        SendInput(static_cast<UINT>(input_count), inputs_, sizeof(INPUT));
    }
}

void TextSender::send_backspaces(uint8_t count) {
    send_text(L"", count);
}

} // namespace gonhanh
