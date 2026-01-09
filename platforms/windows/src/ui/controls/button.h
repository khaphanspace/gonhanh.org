#pragma once

#include <d2d1.h>
#include <string>

namespace gonhanh::ui {

enum class ButtonStyle {
    Primary,    // Blue background, white text
    Secondary,  // White background, gray border
    Text        // No background, text only
};

// Styled button control
class Button {
public:
    // Draw button at position
    static void draw(
        ID2D1RenderTarget* rt,
        float x, float y,
        float width, float height,
        const std::wstring& text,
        ButtonStyle style = ButtonStyle::Secondary,
        bool hovered = false,
        bool pressed = false
    );

    // Hit test
    static bool hit_test(float x, float y, float width, float height, float mouse_x, float mouse_y);
};

} // namespace gonhanh::ui
