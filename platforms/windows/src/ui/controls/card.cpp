#include "card.h"
#include "../d2d_renderer.h"

namespace gonhanh::ui {

void Card::draw(ID2D1RenderTarget* rt, float x, float y, float width, float height, bool with_border) {
    D2D1_ROUNDED_RECT rect = {
        {x, y, x + width, y + height},
        CORNER_RADIUS,
        CORNER_RADIUS
    };

    // Fill
    auto bg_brush = create_brush(rt, Colors::CardBg);
    rt->FillRoundedRectangle(rect, bg_brush.Get());

    // Border
    if (with_border) {
        auto border_brush = create_brush(rt, Colors::Border);
        rt->DrawRoundedRectangle(rect, border_brush.Get(), 1.0f);
    }
}

void Card::draw_separator(ID2D1RenderTarget* rt, float x, float y, float width) {
    auto brush = create_brush(rt, D2D1::ColorF(0.953f, 0.957f, 0.965f));  // #F3F4F6
    rt->DrawLine(
        D2D1::Point2F(x + PADDING, y),
        D2D1::Point2F(x + width - PADDING, y),
        brush.Get(),
        1.0f
    );
}

} // namespace gonhanh::ui
