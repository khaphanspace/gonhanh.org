#pragma once

#include <windows.h>
#include <functional>

namespace gonhanh {

// Global hotkey registration
class HotKey {
public:
    using HotKeyCallback = std::function<void()>;

    static HotKey& instance();

    // Initialize with window handle (for WM_HOTKEY messages)
    void initialize(HWND hwnd);
    void shutdown();

    // Register/unregister toggle hotkey
    bool register_toggle(uint32_t modifiers, uint32_t vk);
    void unregister_toggle();

    // Set callback
    void set_callback(HotKeyCallback callback) { callback_ = std::move(callback); }

    // Process WM_HOTKEY (call from WndProc)
    bool process_message(UINT msg, WPARAM wparam, LPARAM lparam);

    // Hotkey ID
    static constexpr int HOTKEY_TOGGLE = 1;

private:
    HotKey() = default;
    ~HotKey() { shutdown(); }
    HotKey(const HotKey&) = delete;
    HotKey& operator=(const HotKey&) = delete;

    HWND hwnd_ = nullptr;
    bool registered_ = false;
    HotKeyCallback callback_;
};

// Modifier key flags (matches Windows MOD_* values)
namespace ModKey {
    constexpr uint32_t ALT = MOD_ALT;
    constexpr uint32_t CTRL = MOD_CONTROL;
    constexpr uint32_t SHIFT = MOD_SHIFT;
    constexpr uint32_t WIN = MOD_WIN;
}

} // namespace gonhanh
