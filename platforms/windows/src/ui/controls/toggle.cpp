#include "toggle.h"
#include "../d2d_renderer.h"

namespace gonhanh::ui {

// Windows 11 toggle switch colors
namespace ToggleColors {
    // Off state
    constexpr D2D1_COLOR_F OffBorder = {0.678f, 0.678f, 0.678f, 1.0f};      // #ADADAD
    constexpr D2D1_COLOR_F OffBorderHover = {0.533f, 0.533f, 0.533f, 1.0f}; // #888888
    constexpr D2D1_COLOR_F OffThumb = {0.369f, 0.369f, 0.369f, 1.0f};       // #5E5E5E
    constexpr D2D1_COLOR_F OffThumbHover = {0.2f, 0.2f, 0.2f, 1.0f};        // #333333
    // On state (uses accent color from Colors::Primary)
    constexpr D2D1_COLOR_F OnHover = {0.035f, 0.424f, 0.784f, 1.0f};        // #096CC8 (darker accent)
    constexpr D2D1_COLOR_F OnThumb = {1.0f, 1.0f, 1.0f, 1.0f};              // White
}

void Toggle::draw(ID2D1RenderTarget* rt, float x, float y, bool checked, bool hovered) {
    // Track (pill shape) - Windows 11 style
    D2D1_ROUNDED_RECT track_rect = {
        {x, y, x + WIDTH, y + HEIGHT},
        HEIGHT / 2.0f,
        HEIGHT / 2.0f
    };

    if (checked) {
        // On state: filled with accent color
        auto fill_color = hovered ? ToggleColors::OnHover : Colors::Primary;
        auto track_brush = create_brush(rt, fill_color);
        rt->FillRoundedRectangle(track_rect, track_brush.Get());
    } else {
        // Off state: transparent with border (Windows 11 outline style)
        auto border_color = hovered ? ToggleColors::OffBorderHover : ToggleColors::OffBorder;
        auto border_brush = create_brush(rt, border_color);
        rt->DrawRoundedRectangle(track_rect, border_brush.Get(), BORDER_WIDTH);
    }

    // Thumb (circle) - Windows 11 uses different sizes for on/off
    float thumb_size = checked ? THUMB_SIZE_ON : THUMB_SIZE_OFF;
    float thumb_x = checked
        ? x + WIDTH - THUMB_MARGIN - thumb_size / 2.0f
        : x + THUMB_MARGIN + thumb_size / 2.0f;
    float thumb_y = y + HEIGHT / 2.0f;

    D2D1_ELLIPSE thumb = {{thumb_x, thumb_y}, thumb_size / 2.0f, thumb_size / 2.0f};

    // Thumb color: white when on, dark gray when off
    D2D1_COLOR_F thumb_color;
    if (checked) {
        thumb_color = ToggleColors::OnThumb;
    } else {
        thumb_color = hovered ? ToggleColors::OffThumbHover : ToggleColors::OffThumb;
    }
    auto thumb_brush = create_brush(rt, thumb_color);
    rt->FillEllipse(thumb, thumb_brush.Get());
}

bool Toggle::hit_test(float x, float y, float mouse_x, float mouse_y) {
    return mouse_x >= x && mouse_x <= x + WIDTH &&
           mouse_y >= y && mouse_y <= y + HEIGHT;
}

} // namespace gonhanh::ui
