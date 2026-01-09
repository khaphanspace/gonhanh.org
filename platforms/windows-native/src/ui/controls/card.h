#pragma once

#include <d2d1.h>

namespace gonhanh::ui {

// Rounded card container with optional border
class Card {
public:
    static constexpr float CORNER_RADIUS = 10.0f;
    static constexpr float PADDING = 16.0f;

    // Draw card background
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
