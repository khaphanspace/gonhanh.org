#include "sidebar.h"
#include "../d2d_renderer.h"

namespace gonhanh::ui {

// Windows 11 NavigationView colors
namespace NavColors {
    constexpr D2D1_COLOR_F ItemSelected = {0.918f, 0.918f, 0.918f, 1.0f};     // #EAEAEA (subtle fill)
    constexpr D2D1_COLOR_F ItemHover = {0.949f, 0.949f, 0.949f, 1.0f};        // #F2F2F2
    constexpr D2D1_COLOR_F Indicator = {0.0f, 0.471f, 0.831f, 1.0f};          // #0078D4 (accent)
}

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

    // Background (Windows 11 subtle fill style)
    if (selected || hovered) {
        auto bg_color = selected ? NavColors::ItemSelected : NavColors::ItemHover;
        auto bg_brush = create_brush(rt, bg_color);
        rt->FillRoundedRectangle(rect, bg_brush.Get());
    }

    // Selection indicator (Windows 11 pill bar on left)
    if (selected) {
        float indicator_y = y + (ITEM_HEIGHT - INDICATOR_HEIGHT) / 2.0f;
        D2D1_ROUNDED_RECT indicator = {
            {x + 4, indicator_y, x + 4 + INDICATOR_WIDTH, indicator_y + INDICATOR_HEIGHT},
            INDICATOR_WIDTH / 2.0f,  // Pill shape
            INDICATOR_WIDTH / 2.0f
        };
        auto indicator_brush = create_brush(rt, NavColors::Indicator);
        rt->FillRoundedRectangle(indicator, indicator_brush.Get());
    }

    // Text
    auto text_color = selected ? Colors::Text : Colors::TextSecondary;
    auto text_brush = create_brush(rt, text_color);
    auto& renderer = D2DRenderer::instance();

    D2D1_RECT_F text_rect = {
        x + ITEM_PADDING + (selected ? 4 : 0),  // Indent text when selected (after indicator)
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
