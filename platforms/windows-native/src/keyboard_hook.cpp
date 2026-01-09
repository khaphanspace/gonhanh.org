#include "keyboard_hook.h"

namespace gonhanh {

// Static instance pointer for hook callback
static KeyboardHook* g_hook_instance = nullptr;

KeyboardHook& KeyboardHook::instance() {
    static KeyboardHook instance;
    return instance;
}

KeyboardHook::~KeyboardHook() {
    stop();
}

bool KeyboardHook::start() {
    if (hook_) return true;

    g_hook_instance = this;
    hook_ = SetWindowsHookExW(
        WH_KEYBOARD_LL,
        hook_callback,
        GetModuleHandleW(nullptr),
        0  // Thread 0 = all threads
    );

    return hook_ != nullptr;
}

void KeyboardHook::stop() {
    if (hook_) {
        UnhookWindowsHookEx(hook_);
        hook_ = nullptr;
    }
    g_hook_instance = nullptr;
}

LRESULT CALLBACK KeyboardHook::hook_callback(int code, WPARAM wparam, LPARAM lparam) {
    if (g_hook_instance && code >= 0) {
        return g_hook_instance->process_key(code, wparam, lparam);
    }
    return CallNextHookEx(nullptr, code, wparam, lparam);
}

LRESULT KeyboardHook::process_key(int code, WPARAM wparam, LPARAM lparam) {
    // Only process keydown events
    if (wparam != WM_KEYDOWN && wparam != WM_SYSKEYDOWN) {
        return CallNextHookEx(hook_, code, wparam, lparam);
    }

    auto* kbd = reinterpret_cast<KBDLLHOOKSTRUCT*>(lparam);

    // Skip injected keys (our own SendInput or other apps)
    if (kbd->flags & LLKHF_INJECTED) {
        return CallNextHookEx(hook_, code, wparam, lparam);
    }

    // Skip keys with our marker (double protection)
    if (kbd->dwExtraInfo == INJECTED_KEY_MARKER) {
        return CallNextHookEx(hook_, code, wparam, lparam);
    }

    // Prevent recursive processing
    if (is_processing_) {
        return CallNextHookEx(hook_, code, wparam, lparam);
    }

    uint16_t vk = static_cast<uint16_t>(kbd->vkCode);

    // Skip modifier keys themselves
    if (vk == VK::SHIFT || vk == VK::CTRL || vk == VK::ALT ||
        vk == VK_LSHIFT || vk == VK_RSHIFT ||
        vk == VK_LCONTROL || vk == VK_RCONTROL ||
        vk == VK_LMENU || vk == VK_RMENU) {
        return CallNextHookEx(hook_, code, wparam, lparam);
    }

    // Check modifiers
    bool ctrl = is_ctrl_down();
    bool alt = is_alt_down();

    // Skip if Ctrl or Alt is pressed (system shortcuts)
    if (ctrl || alt) {
        return CallNextHookEx(hook_, code, wparam, lparam);
    }

    // Only process relevant keys
    if (!is_letter_key(vk) && !is_number_key(vk) &&
        !is_punctuation_key(vk) && !is_buffer_clearing_key(vk)) {
        return CallNextHookEx(hook_, code, wparam, lparam);
    }

    // Fire callback if set
    if (callback_) {
        is_processing_ = true;

        KeyPressEvent event{
            vk,
            is_shift_down(),
            is_capslock_on(),
            ctrl,
            alt,
            false
        };

        callback_(event);

        is_processing_ = false;

        if (event.handled) {
            return 1;  // Block the key
        }
    }

    return CallNextHookEx(hook_, code, wparam, lparam);
}

bool KeyboardHook::is_shift_down() {
    return (GetAsyncKeyState(VK_SHIFT) & 0x8000) != 0;
}

bool KeyboardHook::is_ctrl_down() {
    return (GetAsyncKeyState(VK_CONTROL) & 0x8000) != 0;
}

bool KeyboardHook::is_alt_down() {
    return (GetAsyncKeyState(VK_MENU) & 0x8000) != 0;
}

bool KeyboardHook::is_capslock_on() {
    return (GetKeyState(VK_CAPITAL) & 0x0001) != 0;
}

bool KeyboardHook::is_letter_key(uint16_t vk) {
    return vk >= VK::KEY_A && vk <= VK::KEY_Z;
}

bool KeyboardHook::is_number_key(uint16_t vk) {
    return vk >= VK::KEY_0 && vk <= VK::KEY_9;
}

bool KeyboardHook::is_punctuation_key(uint16_t vk) {
    switch (vk) {
        case VK::OEM_1:      // ;:
        case VK::OEM_PLUS:   // =+
        case VK::OEM_COMMA:  // ,<
        case VK::OEM_MINUS:  // -_
        case VK::OEM_PERIOD: // .>
        case VK::OEM_2:      // /?
        case VK::OEM_3:      // `~
        case VK::OEM_4:      // [{
        case VK::OEM_5:      // \|
        case VK::OEM_6:      // ]}
        case VK::OEM_7:      // '"
            return true;
        default:
            return false;
    }
}

bool KeyboardHook::is_buffer_clearing_key(uint16_t vk) {
    switch (vk) {
        case VK::SPACE:
        case VK::RETURN:
        case VK::TAB:
        case VK::ESCAPE:
        case VK::BACK:
            return true;
        default:
            return false;
    }
}

} // namespace gonhanh
