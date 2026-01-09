#pragma once

#include <d2d1.h>

namespace gonhanh::ui {

// Windows 11 style toggle switch control
// Based on WinUI 3 ToggleSwitch component
// Dimensions: 40x20 with pill shape
class Toggle {
public:
    // Windows 11 ToggleSwitch dimensions
    static constexpr float WIDTH = 40.0f;
    static constexpr float HEIGHT = 20.0f;
    static constexpr float THUMB_SIZE_OFF = 12.0f;   // Thumb when off
    static constexpr float THUMB_SIZE_ON = 14.0f;    // Thumb when on (slightly larger)
    static constexpr float THUMB_MARGIN = 4.0f;
    static constexpr float BORDER_WIDTH = 1.0f;

    // Draw toggle at position
    static void draw(
        ID2D1RenderTarget* rt,
        float x, float y,
        bool checked,
        bool hovered = false
    );

    // Hit test
    static bool hit_test(float x, float y, float mouse_x, float mouse_y);
};

} // namespace gonhanh::ui
