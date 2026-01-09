#pragma once

#include <d2d1.h>

namespace gonhanh::ui {

// Windows 11 style card container (similar to WinUI 3 Card/Expander)
// Uses 8px corner radius and subtle elevation
class Card {
public:
    // Windows 11 design tokens
    static constexpr float CORNER_RADIUS = 8.0f;    // WinUI 3 uses 8px for cards
    static constexpr float PADDING = 16.0f;
    static constexpr float BORDER_WIDTH = 1.0f;

    // Draw card background with subtle shadow
    static void draw(
        ID2D1RenderTarget* rt,
        float x, float y,
        float width, float height,
        bool with_border = true
    );

    // Draw row separator inside card
    static void draw_separator(
        ID2D1RenderTarget* rt,
        float x, float y,
        float width
    );
};

} // namespace gonhanh::ui
