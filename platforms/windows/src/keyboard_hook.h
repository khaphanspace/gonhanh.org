#pragma once
#include <windows.h>
#include <string>
#include <cstdint>
#include "app_compat.h"

namespace gonhanh {

// Thread-safe text injector with app-specific timing
class TextInjector {
public:
    static TextInjector& Instance();

    // Main injection method - selects strategy based on app
    void Inject(int backspaceCount, const uint32_t* chars, uint8_t charCount);

    // Specific injection strategies
    void InjectViaBackspace(int backspaceCount, const uint32_t* chars, uint8_t charCount,
                            const InjectionTiming& timing);
    void InjectViaSelection(int backspaceCount, const uint32_t* chars, uint8_t charCount,
                            const InjectionTiming& timing);

private:
    TextInjector();
    ~TextInjector();
    TextInjector(const TextInjector&) = delete;
    TextInjector& operator=(const TextInjector&) = delete;

    void SendBackspaces(int count, uint32_t delayUs);
    void SendUnicodeText(const uint32_t* chars, uint8_t count, uint32_t delayUs);
    void MicrosecondSleep(uint32_t us);

    CRITICAL_SECTION cs_;
};

class KeyboardHook {
public:
    static KeyboardHook& Instance();
    bool Install();
    void Uninstall();
    void Toggle();
    bool IsEnabled() const { return enabled_; }
    void SetEnabled(bool enabled);

private:
    KeyboardHook() = default;
    ~KeyboardHook();
    KeyboardHook(const KeyboardHook&) = delete;
    KeyboardHook& operator=(const KeyboardHook&) = delete;

    static LRESULT CALLBACK LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam);

    HHOOK hook_ = nullptr;
    bool enabled_ = true;
    bool processing_ = false;

    // Word tracking for restore functionality
    std::wstring lastWord_;
    std::wstring currentWord_;
    bool afterSpace_ = false;
};

} // namespace gonhanh
