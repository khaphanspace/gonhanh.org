#include "toggle.h"
#include "../d2d_renderer.h"

namespace gonhanh::ui {

void Toggle::draw(ID2D1RenderTarget* rt, float x, float y, bool checked, bool hovered) {
    // Track (pill shape)
    D2D1_ROUNDED_RECT track_rect = {
        {x, y, x + WIDTH, y + HEIGHT},
        HEIGHT / 2.0f,
        HEIGHT / 2.0f
    };

    auto track_brush = create_brush(rt, checked ? Colors::ToggleOn : Colors::ToggleOff);
    rt->FillRoundedRectangle(track_rect, track_brush.Get());

    // Thumb (circle)
    float thumb_x = checked
        ? x + WIDTH - THUMB_MARGIN - THUMB_SIZE / 2.0f
        : x + THUMB_MARGIN + THUMB_SIZE / 2.0f;
    float thumb_y = y + HEIGHT / 2.0f;

    D2D1_ELLIPSE thumb = {{thumb_x, thumb_y}, THUMB_SIZE / 2.0f, THUMB_SIZE / 2.0f};

    auto thumb_brush = create_brush(rt, Colors::CardBg);
    rt->FillEllipse(thumb, thumb_brush.Get());

    // Subtle shadow on thumb
    if (hovered) {
        auto shadow_brush = create_brush(rt, D2D1::ColorF(0, 0, 0, 0.1f));
        D2D1_ELLIPSE shadow = {{thumb_x, thumb_y + 1}, THUMB_SIZE / 2.0f + 1, THUMB_SIZE / 2.0f + 1};
        rt->DrawEllipse(shadow, shadow_brush.Get(), 1.0f);
    }
}

bool Toggle::hit_test(float x, float y, float mouse_x, float mouse_y) {
    return mouse_x >= x && mouse_x <= x + WIDTH &&
           mouse_y >= y && mouse_y <= y + HEIGHT;
}

} // namespace gonhanh::ui
