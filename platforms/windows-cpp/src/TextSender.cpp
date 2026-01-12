#include "TextSender.h"
#include <Windows.h>
#include <vector>

void TextSender::Send(uint8_t backspaceCount, const uint32_t* chars, int64_t charCount) {
    // Pre-calculate total INPUT count
    // Each key event requires keydown + keyup
    size_t backspaceInputs = backspaceCount * 2;
    size_t charInputs = 0;

    // Calculate char inputs (some chars > 0xFFFF need surrogate pairs = 4 inputs)
    for (int64_t i = 0; i < charCount; i++) {
        if (chars[i] == 0) break;
        charInputs += (chars[i] > 0xFFFF) ? 4 : 2;  // Surrogate pair or BMP char
    }

    size_t totalInputs = backspaceInputs + charInputs;
    if (totalInputs == 0) return;

    std::vector<INPUT> inputs(totalInputs);
    size_t idx = 0;

    // Add backspaces
    for (int i = 0; i < backspaceCount; i++) {
        // Keydown
        inputs[idx].type = INPUT_KEYBOARD;
        inputs[idx].ki.wVk = VK_BACK;
        inputs[idx].ki.wScan = 0;
        inputs[idx].ki.dwFlags = 0;
        inputs[idx].ki.time = 0;
        inputs[idx].ki.dwExtraInfo = 0;
        idx++;

        // Keyup
        inputs[idx].type = INPUT_KEYBOARD;
        inputs[idx].ki.wVk = VK_BACK;
        inputs[idx].ki.wScan = 0;
        inputs[idx].ki.dwFlags = KEYEVENTF_KEYUP;
        inputs[idx].ki.time = 0;
        inputs[idx].ki.dwExtraInfo = 0;
        idx++;
    }

    // Add Unicode characters
    for (int64_t i = 0; i < charCount; i++) {
        uint32_t cp = chars[i];
        if (cp == 0) break;

        // Handle surrogate pairs for codepoints > 0xFFFF
        if (cp > 0xFFFF) {
            // Encode as UTF-16 surrogate pair
            cp -= 0x10000;
            uint16_t high = 0xD800 + ((cp >> 10) & 0x3FF);
            uint16_t low = 0xDC00 + (cp & 0x3FF);

            // High surrogate keydown
            inputs[idx].type = INPUT_KEYBOARD;
            inputs[idx].ki.wVk = 0;
            inputs[idx].ki.wScan = high;
            inputs[idx].ki.dwFlags = KEYEVENTF_UNICODE;
            inputs[idx].ki.time = 0;
            inputs[idx].ki.dwExtraInfo = 0;
            idx++;

            // High surrogate keyup
            inputs[idx].type = INPUT_KEYBOARD;
            inputs[idx].ki.wVk = 0;
            inputs[idx].ki.wScan = high;
            inputs[idx].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
            inputs[idx].ki.time = 0;
            inputs[idx].ki.dwExtraInfo = 0;
            idx++;

            // Low surrogate keydown
            inputs[idx].type = INPUT_KEYBOARD;
            inputs[idx].ki.wVk = 0;
            inputs[idx].ki.wScan = low;
            inputs[idx].ki.dwFlags = KEYEVENTF_UNICODE;
            inputs[idx].ki.time = 0;
            inputs[idx].ki.dwExtraInfo = 0;
            idx++;

            // Low surrogate keyup
            inputs[idx].type = INPUT_KEYBOARD;
            inputs[idx].ki.wVk = 0;
            inputs[idx].ki.wScan = low;
            inputs[idx].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
            inputs[idx].ki.time = 0;
            inputs[idx].ki.dwExtraInfo = 0;
            idx++;
        } else {
            // BMP character (Basic Multilingual Plane, 0x0000-0xFFFF)
            // Keydown
            inputs[idx].type = INPUT_KEYBOARD;
            inputs[idx].ki.wVk = 0;
            inputs[idx].ki.wScan = (WORD)cp;
            inputs[idx].ki.dwFlags = KEYEVENTF_UNICODE;
            inputs[idx].ki.time = 0;
            inputs[idx].ki.dwExtraInfo = 0;
            idx++;

            // Keyup
            inputs[idx].type = INPUT_KEYBOARD;
            inputs[idx].ki.wVk = 0;
            inputs[idx].ki.wScan = (WORD)cp;
            inputs[idx].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
            inputs[idx].ki.time = 0;
            inputs[idx].ki.dwExtraInfo = 0;
            idx++;
        }
    }

    // Send all inputs atomically (single SendInput call)
    if (idx > 0) {
        UINT sent = SendInput((UINT)idx, inputs.data(), sizeof(INPUT));

        // Check for UIPI failures (UAC-elevated apps block SendInput)
        if (sent != idx) {
            // GetLastError() returns 0 for UIPI block (not an error per se)
            // Known limitation: Cannot inject into Task Manager, RegEdit, etc.
            // Log warning in debug builds
#ifdef _DEBUG
            char msg[128];
            sprintf_s(msg, "SendInput: sent %u/%u inputs (UIPI block?)\n", sent, (UINT)idx);
            OutputDebugStringA(msg);
#endif
        }
    }
}

void TextSender::SendBackspaces(int count) {
    if (count <= 0) return;

    std::vector<INPUT> inputs(count * 2);
    for (int i = 0; i < count; i++) {
        // Keydown
        inputs[i * 2].type = INPUT_KEYBOARD;
        inputs[i * 2].ki.wVk = VK_BACK;
        inputs[i * 2].ki.wScan = 0;
        inputs[i * 2].ki.dwFlags = 0;
        inputs[i * 2].ki.time = 0;
        inputs[i * 2].ki.dwExtraInfo = 0;

        // Keyup
        inputs[i * 2 + 1].type = INPUT_KEYBOARD;
        inputs[i * 2 + 1].ki.wVk = VK_BACK;
        inputs[i * 2 + 1].ki.wScan = 0;
        inputs[i * 2 + 1].ki.dwFlags = KEYEVENTF_KEYUP;
        inputs[i * 2 + 1].ki.time = 0;
        inputs[i * 2 + 1].ki.dwExtraInfo = 0;
    }

    UINT sent = SendInput((UINT)inputs.size(), inputs.data(), sizeof(INPUT));

#ifdef _DEBUG
    if (sent != (UINT)inputs.size()) {
        char msg[128];
        sprintf_s(msg, "SendBackspaces: sent %u/%zu inputs (UIPI block?)\n", sent, inputs.size());
        OutputDebugStringA(msg);
    }
#endif
}

void TextSender::SendUnicode(uint32_t codepoint) {
    Send(0, &codepoint, 1);
}
