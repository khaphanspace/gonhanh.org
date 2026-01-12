#include "keyboard_hook.h"
#include "rust_bridge.h"

namespace gonhanh {

static KeyboardHook* g_instance = nullptr;

// VK→macOS keycode mapping verified against core/src/data/keys.rs
static uint16_t VkToMacKeycode(DWORD vk) {
    // Letters A-Z
    static const uint16_t letters[] = {
        0x00, 0x0B, 0x08, 0x02, 0x0E, 0x03, 0x05, 0x04, // A-H
        0x22, 0x26, 0x28, 0x25, 0x2E, 0x2D, 0x1F, 0x23, // I-P
        0x0C, 0x0F, 0x01, 0x11, 0x20, 0x09, 0x0D, 0x07, // Q-X
        0x10, 0x06                                       // Y-Z
    };
    if (vk >= 'A' && vk <= 'Z') {
        return letters[vk - 'A'];
    }

    // Numbers 0-9
    static const uint16_t numbers[] = {
        0x1D, 0x12, 0x13, 0x14, 0x15, 0x17, 0x16, 0x1A, 0x1C, 0x19
    };
    if (vk >= '0' && vk <= '9') {
        return numbers[vk - '0'];
    }

    // Special keys
    switch (vk) {
        case VK_SPACE: return 0x31;
        case VK_RETURN: return 0x24;
        case VK_BACK: return 0x33;
        case VK_ESCAPE: return 0x35;
        case VK_OEM_4: return 0x21;  // [ key
        case VK_OEM_6: return 0x1E;  // ] key
        default: return 0xFF;  // Unknown
    }
}

// Send backspaces via SendInput
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

// Send Unicode text via SendInput
static void SendUnicodeText(const uint32_t* chars, uint8_t count) {
    for (uint8_t i = 0; i < count; ++i) {
        uint32_t cp = chars[i];

        if (cp <= 0xFFFF) {
            // BMP codepoint - single UTF-16 unit
            INPUT input = {};
            input.type = INPUT_KEYBOARD;
            input.ki.wVk = 0;  // Use Unicode, not VK
            input.ki.wScan = static_cast<WORD>(cp);

            // Key down
            input.ki.dwFlags = KEYEVENTF_UNICODE;
            SendInput(1, &input, sizeof(INPUT));

            // Key up
            input.ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
            SendInput(1, &input, sizeof(INPUT));
        } else {
            // Supplementary plane - surrogate pair
            cp -= 0x10000;
            WORD high = 0xD800 + static_cast<WORD>(cp >> 10);
            WORD low = 0xDC00 + static_cast<WORD>(cp & 0x3FF);

            INPUT inputs[4] = {};
            // High surrogate down
            inputs[0].type = INPUT_KEYBOARD;
            inputs[0].ki.wScan = high;
            inputs[0].ki.dwFlags = KEYEVENTF_UNICODE;

            // High surrogate up
            inputs[1].type = INPUT_KEYBOARD;
            inputs[1].ki.wScan = high;
            inputs[1].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;

            // Low surrogate down
            inputs[2].type = INPUT_KEYBOARD;
            inputs[2].ki.wScan = low;
            inputs[2].ki.dwFlags = KEYEVENTF_UNICODE;

            // Low surrogate up
            inputs[3].type = INPUT_KEYBOARD;
            inputs[3].ki.wScan = low;
            inputs[3].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;

            SendInput(4, inputs, sizeof(INPUT));
        }
    }
}

// Singleton getter
KeyboardHook& KeyboardHook::Instance() {
    static KeyboardHook instance;
    g_instance = &instance;
    return instance;
}

KeyboardHook::~KeyboardHook() {
    Uninstall();
}

bool KeyboardHook::Install() {
    if (hook_) return true;  // Already installed

    hook_ = SetWindowsHookEx(
        WH_KEYBOARD_LL,
        LowLevelKeyboardProc,
        GetModuleHandle(NULL),
        0
    );

    return hook_ != nullptr;
}

void KeyboardHook::Uninstall() {
    if (hook_) {
        UnhookWindowsHookEx(hook_);
        hook_ = nullptr;
    }
}

void KeyboardHook::Toggle() {
    enabled_ = !enabled_;
}

void KeyboardHook::SetEnabled(bool enabled) {
    enabled_ = enabled;
}

// Hook callback
LRESULT CALLBACK KeyboardHook::LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam) {
    if (nCode != HC_ACTION) {
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    auto* kb = reinterpret_cast<KBDLLHOOKSTRUCT*>(lParam);

    // Ignore injected events to prevent infinite loops
    if (kb->flags & LLKHF_INJECTED) {
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    // Only process key down events
    if (wParam != WM_KEYDOWN && wParam != WM_SYSKEYDOWN) {
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    // Handle Ctrl+Space toggle (check BEFORE enabled check)
    if (kb->vkCode == VK_SPACE && (GetKeyState(VK_CONTROL) & 0x8000)) {
        if (g_instance) {
            g_instance->Toggle();
        }
        return 1;  // Suppress key
    }

    // Pass through if disabled
    if (!g_instance || !g_instance->enabled_) {
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    // Reentrancy guard
    if (g_instance->processing_) {
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    // Convert VK to macOS keycode
    uint16_t keycode = VkToMacKeycode(kb->vkCode);
    if (keycode == 0xFF) {
        return CallNextHookEx(NULL, nCode, wParam, lParam);  // Unknown key
    }

    // Get modifier states
    bool caps = (GetKeyState(VK_CAPITAL) & 0x0001) != 0;
    bool ctrl = (GetKeyState(VK_CONTROL) & 0x8000) != 0;
    bool shift = (GetKeyState(VK_SHIFT) & 0x8000) != 0;

    // Call Rust engine
    g_instance->processing_ = true;
    ImeResultGuard result(ime_key_ext(keycode, caps, ctrl, shift));
    g_instance->processing_ = false;

    if (!result) {
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    // Check if engine wants to transform
    if (result->action == 0) {  // Pass through
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    // Suppress original key
    if (result->backspace > 0) {
        SendBackspaces(result->backspace);
    }

    if (result->count > 0) {
        SendUnicodeText(result->chars, result->count);
    }

    return 1;  // Suppress original keystroke
}

} // namespace gonhanh
