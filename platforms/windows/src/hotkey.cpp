#include "hotkey.h"

namespace gonhanh {

HotKey& HotKey::instance() {
    static HotKey instance;
    return instance;
}

void HotKey::initialize(HWND hwnd) {
    hwnd_ = hwnd;
}

void HotKey::shutdown() {
    unregister_toggle();
    hwnd_ = nullptr;
}

bool HotKey::register_toggle(uint32_t modifiers, uint32_t vk) {
    if (!hwnd_) return false;

    // Unregister existing hotkey first
    unregister_toggle();

    if (vk == 0) return false;

    // Register new hotkey
    if (RegisterHotKey(hwnd_, HOTKEY_TOGGLE, modifiers, vk)) {
        registered_ = true;
        return true;
    }

    return false;
}

void HotKey::unregister_toggle() {
    if (registered_ && hwnd_) {
        UnregisterHotKey(hwnd_, HOTKEY_TOGGLE);
        registered_ = false;
    }
}

bool HotKey::process_message(UINT msg, WPARAM wparam, LPARAM lparam) {
    if (msg != WM_HOTKEY) return false;

    if (wparam == HOTKEY_TOGGLE && callback_) {
        callback_();
        return true;
    }

    return false;
}

} // namespace gonhanh
