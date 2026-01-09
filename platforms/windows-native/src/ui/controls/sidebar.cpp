#include "sidebar.h"
#include "../d2d_renderer.h"

namespace gonhanh::ui {

void Sidebar::draw_item(
    ID2D1RenderTarget* rt,
    float x, float y,
    float width,
    const std::wstring& text,
    bool selected,
    bool hovered
) {
    D2D1_ROUNDED_RECT rect = {
        {x, y, x + width, y + ITEM_HEIGHT},
        CORNER_RADIUS,
        CORNER_RADIUS
    };

    // Background
    if (selected) {
        auto bg_brush = create_brush(rt, D2D1::ColorF(0.9f, 0.9f, 0.92f));
        rt->FillRoundedRectangle(rect, bg_brush.Get());
    } else if (hovered) {
        auto bg_brush = create_brush(rt, D2D1::ColorF(0.93f, 0.93f, 0.95f, 0.5f));
        rt->FillRoundedRectangle(rect, bg_brush.Get());
    }

    // Text
    auto text_brush = create_brush(rt, selected ? Colors::Text : Colors::TextSecondary);
    auto& renderer = D2DRenderer::instance();

    D2D1_RECT_F text_rect = {
        x + ITEM_PADDING,
        y,
        x + width - ITEM_PADDING,
        y + ITEM_HEIGHT
    };

    auto text_format = renderer.text_format_body();
    if (text_format) {
        text_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
        text_format->SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
    }

    rt->DrawText(
        text.c_str(),
        static_cast<UINT32>(text.length()),
        text_format,
        text_rect,
        text_brush.Get()
    );
}

bool Sidebar::hit_test_item(float x, float y, float width, float mouse_x, float mouse_y) {
    return mouse_x >= x && mouse_x <= x + width &&
           mouse_y >= y && mouse_y <= y + ITEM_HEIGHT;
}

} // namespace gonhanh::ui
