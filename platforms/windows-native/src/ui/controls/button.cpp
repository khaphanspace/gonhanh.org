#include "button.h"
#include "../d2d_renderer.h"

namespace gonhanh::ui {

void Button::draw(
    ID2D1RenderTarget* rt,
    float x, float y,
    float width, float height,
    const std::wstring& text,
    ButtonStyle style,
    bool hovered,
    bool pressed
) {
    constexpr float corner_radius = 6.0f;

    D2D1_ROUNDED_RECT rect = {
        {x, y, x + width, y + height},
        corner_radius,
        corner_radius
    };

    // Background
    D2D1_COLOR_F bg_color;
    D2D1_COLOR_F text_color;
    D2D1_COLOR_F border_color = Colors::Border;

    switch (style) {
        case ButtonStyle::Primary:
            bg_color = pressed ? D2D1::ColorF(0.1f, 0.3f, 0.8f) :
                      hovered ? D2D1::ColorF(0.12f, 0.35f, 0.85f) :
                      Colors::Primary;
            text_color = Colors::CardBg;
            border_color = bg_color;
            break;

        case ButtonStyle::Secondary:
            bg_color = pressed ? D2D1::ColorF(0.93f, 0.93f, 0.93f) :
                      hovered ? D2D1::ColorF(0.97f, 0.97f, 0.98f) :
                      Colors::CardBg;
            text_color = Colors::Text;
            break;

        case ButtonStyle::Text:
            bg_color = hovered ? D2D1::ColorF(0.95f, 0.95f, 0.96f, 0.5f) :
                       D2D1::ColorF(0, 0, 0, 0);
            text_color = Colors::Primary;
            border_color = D2D1::ColorF(0, 0, 0, 0);
            break;
    }

    // Fill background
    auto bg_brush = create_brush(rt, bg_color);
    rt->FillRoundedRectangle(rect, bg_brush.Get());

    // Border (for secondary style)
    if (style == ButtonStyle::Secondary) {
        auto border_brush = create_brush(rt, border_color);
        rt->DrawRoundedRectangle(rect, border_brush.Get(), 1.0f);
    }

    // Text
    auto text_brush = create_brush(rt, text_color);
    auto& renderer = D2DRenderer::instance();

    D2D1_RECT_F text_rect = {x, y, x + width, y + height};
    auto text_format = renderer.text_format_body();
    if (text_format) {
        text_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
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

bool Button::hit_test(float x, float y, float width, float height, float mouse_x, float mouse_y) {
    return mouse_x >= x && mouse_x <= x + width &&
           mouse_y >= y && mouse_y <= y + height;
}

} // namespace gonhanh::ui
