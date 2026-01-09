#pragma once

#include <windows.h>
#include <functional>
#include <cstdint>
#include <string>

namespace gonhanh {

// Key press event data
struct KeyPressEvent {
    uint16_t keycode;
    bool shift;
    bool capslock;
    bool ctrl;
    bool alt;
    bool handled = false;  // Set to true to block key
};

// Low-level keyboard hook for system-wide key interception
class KeyboardHook {
public:
    using KeyPressCallback = std::function<void(KeyPressEvent&)>;

    static KeyboardHook& instance();

    // Start/stop hook
    bool start();
    void stop();
    bool is_running() const { return hook_ != nullptr; }

    // Set callback for key press events
    void set_callback(KeyPressCallback callback) { callback_ = std::move(callback); }

    // Check modifier states
    static bool is_shift_down();
    static bool is_ctrl_down();
    static bool is_alt_down();
    static bool is_capslock_on();

    // Key classification helpers
    static bool is_letter_key(uint16_t vk);
    static bool is_number_key(uint16_t vk);
    static bool is_punctuation_key(uint16_t vk);
    static bool is_buffer_clearing_key(uint16_t vk);

    // Caret position tracking
    struct CaretPosition {
        int x;
        int y;
        bool valid;
    };

    static CaretPosition get_caret_position();

    // Get foreground app executable name
    static std::wstring get_foreground_app_name();

private:
    KeyboardHook() = default;
    ~KeyboardHook();
    KeyboardHook(const KeyboardHook&) = delete;
    KeyboardHook& operator=(const KeyboardHook&) = delete;

    static LRESULT CALLBACK hook_callback(int code, WPARAM wparam, LPARAM lparam);
    LRESULT process_key(int code, WPARAM wparam, LPARAM lparam);

    HHOOK hook_ = nullptr;
    KeyPressCallback callback_;
    bool is_processing_ = false;

    // Marker for injected keys (to avoid recursive processing)
    static constexpr ULONG_PTR INJECTED_KEY_MARKER = 0x474E4820;  // "GNH "
};

// Virtual key codes (commonly used)
namespace VK {
    constexpr uint16_t BACK = 0x08;
    constexpr uint16_t TAB = 0x09;
    constexpr uint16_t RETURN = 0x0D;
    constexpr uint16_t SHIFT = 0x10;
    constexpr uint16_t CTRL = 0x11;
    constexpr uint16_t ALT = 0x12;
    constexpr uint16_t CAPSLOCK = 0x14;
    constexpr uint16_t ESCAPE = 0x1B;
    constexpr uint16_t SPACE = 0x20;
    constexpr uint16_t KEY_0 = 0x30;
    constexpr uint16_t KEY_9 = 0x39;
    constexpr uint16_t KEY_A = 0x41;
    constexpr uint16_t KEY_Z = 0x5A;
    constexpr uint16_t OEM_1 = 0xBA;      // ;:
    constexpr uint16_t OEM_PLUS = 0xBB;   // =+
    constexpr uint16_t OEM_COMMA = 0xBC;  // ,<
    constexpr uint16_t OEM_MINUS = 0xBD;  // -_
    constexpr uint16_t OEM_PERIOD = 0xBE; // .>
    constexpr uint16_t OEM_2 = 0xBF;      // /?
    constexpr uint16_t OEM_3 = 0xC0;      // `~
    constexpr uint16_t OEM_4 = 0xDB;      // [{
    constexpr uint16_t OEM_5 = 0xDC;      // \|
    constexpr uint16_t OEM_6 = 0xDD;      // ]}
    constexpr uint16_t OEM_7 = 0xDE;      // '"
}

} // namespace gonhanh
