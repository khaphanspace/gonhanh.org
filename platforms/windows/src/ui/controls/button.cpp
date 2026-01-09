#include "button.h"
#include "../d2d_renderer.h"

namespace gonhanh::ui {

// Windows 11 button colors
namespace ButtonColors {
    // Primary button (AccentButton in WinUI 3)
    constexpr D2D1_COLOR_F PrimaryBg = {0.0f, 0.471f, 0.831f, 1.0f};         // #0078D4
    constexpr D2D1_COLOR_F PrimaryBgHover = {0.035f, 0.424f, 0.784f, 1.0f};  // #096CC8
    constexpr D2D1_COLOR_F PrimaryBgPressed = {0.067f, 0.380f, 0.733f, 1.0f}; // #1161BB
    // Secondary button (standard Button in WinUI 3)
    constexpr D2D1_COLOR_F SecondaryBg = {1.0f, 1.0f, 1.0f, 0.7f};           // Translucent white
    constexpr D2D1_COLOR_F SecondaryBgHover = {0.969f, 0.969f, 0.969f, 0.9f}; // #F7F7F7
    constexpr D2D1_COLOR_F SecondaryBgPressed = {0.949f, 0.949f, 0.949f, 1.0f}; // #F2F2F2
    constexpr D2D1_COLOR_F SecondaryBorder = {0.820f, 0.820f, 0.820f, 1.0f}; // #D1D1D1
}

void Button::draw(
    ID2D1RenderTarget* rt,
    float x, float y,
    float width, float height,
    const std::wstring& text,
    ButtonStyle style,
    bool hovered,
    bool pressed
) {
    // Windows 11 uses 4px corner radius for buttons
    constexpr float corner_radius = 4.0f;

    D2D1_ROUNDED_RECT rect = {
        {x, y, x + width, y + height},
        corner_radius,
        corner_radius
    };

    // Background and text colors based on style
    D2D1_COLOR_F bg_color;
    D2D1_COLOR_F text_color;
    D2D1_COLOR_F border_color = ButtonColors::SecondaryBorder;

    switch (style) {
        case ButtonStyle::Primary:
            // AccentButton style - solid accent color
            bg_color = pressed ? ButtonColors::PrimaryBgPressed :
                      hovered ? ButtonColors::PrimaryBgHover :
                      ButtonColors::PrimaryBg;
            text_color = {1.0f, 1.0f, 1.0f, 1.0f};  // White text
            border_color = bg_color;
            break;

        case ButtonStyle::Secondary:
            // Standard button - subtle fill with border
            bg_color = pressed ? ButtonColors::SecondaryBgPressed :
                      hovered ? ButtonColors::SecondaryBgHover :
                      ButtonColors::SecondaryBg;
            text_color = Colors::Text;
            break;

        case ButtonStyle::Text:
            // HyperlinkButton style - transparent background
            bg_color = hovered ? D2D1::ColorF(0.0f, 0.0f, 0.0f, 0.04f) :
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
