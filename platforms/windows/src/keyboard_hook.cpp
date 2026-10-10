#include "keyboard_hook.h"
#include "rust_bridge.h"
#include "app_compat.h"

namespace gonhanh {

static KeyboardHook* g_instance = nullptr;

// RAII guard for reentrancy protection
class ProcessingGuard {
    bool& flag_;
public:
    explicit ProcessingGuard(bool& flag) : flag_(flag) { flag_ = true; }
    ~ProcessingGuard() { flag_ = false; }
};

// RAII guard for CRITICAL_SECTION
class CriticalSectionGuard {
    CRITICAL_SECTION& cs_;
public:
    explicit CriticalSectionGuard(CRITICAL_SECTION& cs) : cs_(cs) { EnterCriticalSection(&cs_); }
    ~CriticalSectionGuard() { LeaveCriticalSection(&cs_); }
};

// ============================================================================
// TextInjector Implementation
// ============================================================================

TextInjector& TextInjector::Instance() {
    static TextInjector instance;
    return instance;
}

TextInjector::TextInjector() {
    InitializeCriticalSection(&cs_);
}

TextInjector::~TextInjector() {
    DeleteCriticalSection(&cs_);
}

void TextInjector::MicrosecondSleep(uint32_t us) {
    if (us == 0) return;

    // For delays under 1ms, use busy-wait with QueryPerformanceCounter
    // Sleep(1) has ~15ms minimum granularity on Windows
    if (us < 1000) {
        LARGE_INTEGER freq, start, now;
        QueryPerformanceFrequency(&freq);
        QueryPerformanceCounter(&start);
        double targetTicks = (double)us * freq.QuadPart / 1000000.0;
        do {
            QueryPerformanceCounter(&now);
        } while ((now.QuadPart - start.QuadPart) < targetTicks);
    } else {
        // For larger delays, use Sleep (rounded up to ms)
        Sleep((us + 999) / 1000);
    }
}

void TextInjector::SendBackspaces(int count, uint32_t delayUs) {
    if (count <= 0 || count > 32) return;

    for (int i = 0; i < count; ++i) {
        INPUT inputs[2] = {};
        inputs[0].type = INPUT_KEYBOARD;
        inputs[0].ki.wVk = VK_BACK;
        inputs[1].type = INPUT_KEYBOARD;
        inputs[1].ki.wVk = VK_BACK;
        inputs[1].ki.dwFlags = KEYEVENTF_KEYUP;
        SendInput(2, inputs, sizeof(INPUT));

        if (delayUs > 0 && i < count - 1) {
            MicrosecondSleep(delayUs);
        }
    }
}

void TextInjector::SendUnicodeText(const uint32_t* chars, uint8_t count, uint32_t delayUs) {
    for (uint8_t i = 0; i < count; ++i) {
        uint32_t cp = chars[i];

        if (cp <= 0xFFFF) {
            // BMP codepoint - single UTF-16 unit
            INPUT input = {};
            input.type = INPUT_KEYBOARD;
            input.ki.wVk = 0;
            input.ki.wScan = static_cast<WORD>(cp);

            input.ki.dwFlags = KEYEVENTF_UNICODE;
            SendInput(1, &input, sizeof(INPUT));

            input.ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
            SendInput(1, &input, sizeof(INPUT));
        } else {
            // Supplementary plane - surrogate pair
            cp -= 0x10000;
            WORD high = 0xD800 + static_cast<WORD>(cp >> 10);
            WORD low = 0xDC00 + static_cast<WORD>(cp & 0x3FF);

            INPUT inputs[4] = {};
            inputs[0].type = INPUT_KEYBOARD;
            inputs[0].ki.wScan = high;
            inputs[0].ki.dwFlags = KEYEVENTF_UNICODE;

            inputs[1].type = INPUT_KEYBOARD;
            inputs[1].ki.wScan = high;
            inputs[1].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;

            inputs[2].type = INPUT_KEYBOARD;
            inputs[2].ki.wScan = low;
            inputs[2].ki.dwFlags = KEYEVENTF_UNICODE;

            inputs[3].type = INPUT_KEYBOARD;
            inputs[3].ki.wScan = low;
            inputs[3].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;

            SendInput(4, inputs, sizeof(INPUT));
        }

        if (delayUs > 0 && i < count - 1) {
            MicrosecondSleep(delayUs);
        }
    }
}

void TextInjector::Inject(int backspaceCount, const uint32_t* chars, uint8_t charCount) {
    CriticalSectionGuard lock(cs_);

    // Get injection method from app detection
    auto detection = AppCompat::Instance().GetInjectionMethod();

    switch (detection.method) {
        case InjectionMethod::Selection:
            InjectViaSelection(backspaceCount, chars, charCount, detection.timing);
            break;
        case InjectionMethod::Slow:
        case InjectionMethod::Fast:
        default:
            InjectViaBackspace(backspaceCount, chars, charCount, detection.timing);
            break;
    }
}

void TextInjector::InjectViaBackspace(int backspaceCount, const uint32_t* chars, uint8_t charCount,
                                       const InjectionTiming& timing) {
    // Send backspaces with per-backspace delay
    if (backspaceCount > 0) {
        SendBackspaces(backspaceCount, timing.backspaceDelayUs);
        MicrosecondSleep(timing.waitDelayUs);
    }

    // Send text with per-character delay
    if (charCount > 0) {
        SendUnicodeText(chars, charCount, timing.textDelayUs);
    }
}

void TextInjector::InjectViaSelection(int backspaceCount, const uint32_t* chars, uint8_t charCount,
                                       const InjectionTiming& timing) {
    // Selection method: use Shift+Left to select characters, then type to replace
    // This works better in address bars where backspace may navigate back

    if (backspaceCount > 0) {
        // Send Shift+Left 'backspaceCount' times to select text
        for (int i = 0; i < backspaceCount; ++i) {
            INPUT inputs[4] = {};

            // Shift down
            inputs[0].type = INPUT_KEYBOARD;
            inputs[0].ki.wVk = VK_SHIFT;

            // Left down
            inputs[1].type = INPUT_KEYBOARD;
            inputs[1].ki.wVk = VK_LEFT;

            // Left up
            inputs[2].type = INPUT_KEYBOARD;
            inputs[2].ki.wVk = VK_LEFT;
            inputs[2].ki.dwFlags = KEYEVENTF_KEYUP;

            // Shift up
            inputs[3].type = INPUT_KEYBOARD;
            inputs[3].ki.wVk = VK_SHIFT;
            inputs[3].ki.dwFlags = KEYEVENTF_KEYUP;

            SendInput(4, inputs, sizeof(INPUT));

            if (i < backspaceCount - 1) {
                MicrosecondSleep(timing.backspaceDelayUs);
            }
        }

        MicrosecondSleep(timing.waitDelayUs);
    }

    // Type replacement text (will replace selection)
    if (charCount > 0) {
        SendUnicodeText(chars, charCount, timing.textDelayUs);
    }
}

// ============================================================================
// KeyboardHook Implementation
// ============================================================================

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
    if (hook_) return true;

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

    // Word tracking for restore functionality
    bool isWordBoundary = (kb->vkCode == VK_SPACE ||
                           kb->vkCode == VK_RETURN ||
                           kb->vkCode == VK_TAB ||
                           (kb->vkCode >= VK_OEM_1 && kb->vkCode <= VK_OEM_3) ||  // ;=,-./`
                           (kb->vkCode >= VK_OEM_4 && kb->vkCode <= VK_OEM_7));   // [\]'

    if (isWordBoundary && !g_instance->currentWord_.empty()) {
        // Save completed word and mark as after space
        g_instance->lastWord_ = g_instance->currentWord_;
        g_instance->currentWord_.clear();
        g_instance->afterSpace_ = true;
    } else if (kb->vkCode == VK_BACK) {
        // Handle backspace - check if we should restore word
        if (g_instance->afterSpace_ && g_instance->currentWord_.empty() && !g_instance->lastWord_.empty()) {
            // Backspace right after completing a word - restore it
            std::string utf8Word = Utf16ToUtf8(g_instance->lastWord_);
            if (!utf8Word.empty()) {
                ime_restore_word(utf8Word.c_str());
            }
            g_instance->afterSpace_ = false;
        } else if (!g_instance->currentWord_.empty()) {
            // Remove last character from current word
            g_instance->currentWord_.pop_back();
        }
        g_instance->afterSpace_ = false;
    } else if (kb->vkCode >= 'A' && kb->vkCode <= 'Z') {
        // Track letter for current word
        g_instance->afterSpace_ = false;
        // Note: actual character will be tracked after engine processes it
    } else {
        g_instance->afterSpace_ = false;
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

    // Call Rust engine with RAII reentrancy guard
    ProcessingGuard guard(g_instance->processing_);
    ImeResultGuard result(ime_key_ext(keycode, caps, ctrl, shift));

    if (!result) {
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    // Check if engine wants to transform
    if (result->action == 0) {  // Pass through
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    // Log transform for debugging
#ifdef _DEBUG
    wchar_t logBuf[256];
    swprintf_s(logBuf, L"[HOOK] VK=0x%02X keycode=%d backspace=%d count=%d chars=",
               kb->vkCode, keycode, result->backspace, result->count);
    std::wstring logMsg(logBuf);
    for (uint8_t i = 0; i < result->count; ++i) {
        wchar_t charBuf[16];
        swprintf_s(charBuf, L"U+%04X ", result->chars[i]);
        logMsg += charBuf;
    }
    OutputDebugStringW(logMsg.c_str());
#endif

    // Use TextInjector for app-aware text injection
    TextInjector::Instance().Inject(result->backspace, result->chars, result->count);

    // Update word tracking with output characters
    if (result->count > 0) {
        std::wstring output = Utf32ToUtf16(result->chars, result->count);
        // If there were backspaces, adjust currentWord_ accordingly
        if (result->backspace > 0 && !g_instance->currentWord_.empty()) {
            size_t removeCount = (std::min)(static_cast<size_t>(result->backspace),
                                            g_instance->currentWord_.size());
            g_instance->currentWord_.erase(g_instance->currentWord_.size() - removeCount);
        }
        g_instance->currentWord_ += output;
    }

    return 1;  // Suppress original keystroke
}

} // namespace gonhanh
