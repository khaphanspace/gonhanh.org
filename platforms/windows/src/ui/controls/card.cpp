#include "card.h"
#include "../d2d_renderer.h"

namespace gonhanh::ui {

// Windows 11 card colors
namespace CardColors {
    constexpr D2D1_COLOR_F Background = {1.0f, 1.0f, 1.0f, 1.0f};           // #FFFFFF
    constexpr D2D1_COLOR_F Border = {0.918f, 0.918f, 0.918f, 1.0f};         // #EAEAEA
    constexpr D2D1_COLOR_F Separator = {0.918f, 0.918f, 0.918f, 1.0f};      // #EAEAEA
    constexpr D2D1_COLOR_F Shadow = {0.0f, 0.0f, 0.0f, 0.04f};              // Subtle shadow
}

void Card::draw(ID2D1RenderTarget* rt, float x, float y, float width, float height, bool with_border) {
    // Draw subtle shadow first (offset by 1px down)
    D2D1_ROUNDED_RECT shadow_rect = {
        {x, y + 1, x + width, y + height + 1},
        CORNER_RADIUS,
        CORNER_RADIUS
    };
    auto shadow_brush = create_brush(rt, CardColors::Shadow);
    rt->FillRoundedRectangle(shadow_rect, shadow_brush.Get());

    // Main card rect
    D2D1_ROUNDED_RECT rect = {
        {x, y, x + width, y + height},
        CORNER_RADIUS,
        CORNER_RADIUS
    };

    // Fill with solid white
    auto bg_brush = create_brush(rt, CardColors::Background);
    rt->FillRoundedRectangle(rect, bg_brush.Get());

    // Border (Windows 11 uses subtle borders)
    if (with_border) {
        auto border_brush = create_brush(rt, CardColors::Border);
        rt->DrawRoundedRectangle(rect, border_brush.Get(), BORDER_WIDTH);
    }
}

void Card::draw_separator(ID2D1RenderTarget* rt, float x, float y, float width) {
    // Windows 11 style separator - full width minus padding
    auto brush = create_brush(rt, CardColors::Separator);
    rt->DrawLine(
        D2D1::Point2F(x, y),
        D2D1::Point2F(x + width, y),
        brush.Get(),
        1.0f
    );
}

} // namespace gonhanh::ui
