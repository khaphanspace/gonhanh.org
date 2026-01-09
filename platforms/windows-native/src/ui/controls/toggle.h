#pragma once

#include <d2d1.h>

namespace gonhanh::ui {

// Apple-style toggle switch control
// Dimensions: 44x24 with pill shape
class Toggle {
public:
    static constexpr float WIDTH = 44.0f;
    static constexpr float HEIGHT = 24.0f;
    static constexpr float THUMB_SIZE = 20.0f;
    static constexpr float THUMB_MARGIN = 2.0f;

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
