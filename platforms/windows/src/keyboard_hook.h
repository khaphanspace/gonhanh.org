#pragma once
#include <windows.h>

namespace gonhanh {

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
};

} // namespace gonhanh
