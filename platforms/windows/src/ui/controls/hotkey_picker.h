#pragma once

#include <windows.h>
#include <d2d1.h>
#include <dwrite.h>
#include <string>
#include <functional>

namespace gonhanh::ui {

// Hotkey recorder control - captures keyboard shortcut
// Displays current hotkey and records new ones on click
class HotkeyPicker {
public:
    // Draw the hotkey picker control
    // Returns true if recording mode is active
    static void draw(
        ID2D1RenderTarget* rt,
        float x, float y,
        float width,
        uint32_t modifiers,    // MOD_CTRL, MOD_ALT, MOD_SHIFT, MOD_WIN
        uint32_t vk,           // Virtual key code
        bool recording,        // Currently recording?
        bool hover
    );

    // Hit test for mouse interaction
    static bool hit_test(float x, float y, float width, float mx, float my);

    // Convert hotkey to display string
    static std::wstring hotkey_to_string(uint32_t modifiers, uint32_t vk);

    // Parse key name from virtual key code
    static std::wstring vk_to_string(uint32_t vk);

    // Dimensions
    static constexpr float HEIGHT = 28.0f;
    static constexpr float MIN_WIDTH = 120.0f;
    static constexpr float BORDER_RADIUS = 6.0f;
    static constexpr float PADDING = 10.0f;
};

// Hotkey recording handler - manages recording state
class HotkeyRecorder {
public:
    using Callback = std::function<void(uint32_t modifiers, uint32_t vk)>;

    static HotkeyRecorder& instance();

    // Start recording a new hotkey
    void start_recording(Callback on_complete);

    // Stop recording without saving
    void cancel();

    // Check if currently recording
    bool is_recording() const { return recording_; }

    // Process keyboard message during recording
    // Returns true if message was handled
    bool process_key(uint32_t vk, bool key_down);

    // Get current recording state
    uint32_t current_modifiers() const { return current_modifiers_; }
    uint32_t current_vk() const { return current_vk_; }

private:
    HotkeyRecorder() = default;

    bool recording_ = false;
    uint32_t current_modifiers_ = 0;
    uint32_t current_vk_ = 0;
    Callback callback_;
};

} // namespace gonhanh::ui
