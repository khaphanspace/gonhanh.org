#include "text_input.h"
#include "../d2d_renderer.h"

namespace gonhanh::ui {

void TextInput::draw(
    ID2D1RenderTarget* rt,
    float x, float y,
    float width, float height,
    const std::wstring& text,
    const std::wstring& placeholder,
    bool focused
) {
    D2D1_ROUNDED_RECT rect = {
        {x, y, x + width, y + height},
        BORDER_RADIUS,
        BORDER_RADIUS
    };

    // Background
    auto bg_brush = create_brush(rt, Colors::CardBg);
    rt->FillRoundedRectangle(rect, bg_brush.Get());

    // Border (blue when focused, gray otherwise)
    D2D1_COLOR_F border_color = focused ? Colors::Primary : Colors::Border;
    auto border_brush = create_brush(rt, border_color);
    rt->DrawRoundedRectangle(rect, border_brush.Get(), focused ? 2.0f : 1.0f);

    // Text content
    auto& renderer = D2DRenderer::instance();
    auto text_format = renderer.text_format_body();
    if (text_format) {
        text_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
        text_format->SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
    }

    D2D1_RECT_F text_rect = {
        x + PADDING_X,
        y,
        x + width - PADDING_X,
        y + height
    };

    if (!text.empty()) {
        // Draw actual text
        auto text_brush = create_brush(rt, Colors::Text);
        rt->DrawText(
            text.c_str(),
            static_cast<UINT32>(text.length()),
            text_format,
            text_rect,
            text_brush.Get()
        );
    } else if (!placeholder.empty()) {
        // Draw placeholder text
        auto placeholder_brush = create_brush(rt, Colors::TextSecondary);
        rt->DrawText(
            placeholder.c_str(),
            static_cast<UINT32>(placeholder.length()),
            text_format,
            text_rect,
            placeholder_brush.Get()
        );
    }
}

void TextInput::draw_hotkey_input(
    ID2D1RenderTarget* rt,
    float x, float y,
    float width, float height,
    const std::vector<std::wstring>& keys,
    bool recording,
    bool focused
) {
    D2D1_ROUNDED_RECT rect = {
        {x, y, x + width, y + height},
        BORDER_RADIUS,
        BORDER_RADIUS
    };

    // Background
    auto bg_brush = create_brush(rt, Colors::CardBg);
    rt->FillRoundedRectangle(rect, bg_brush.Get());

    // Border (blue when focused, gray otherwise)
    D2D1_COLOR_F border_color = focused ? Colors::Primary : Colors::Border;
    auto border_brush = create_brush(rt, border_color);
    rt->DrawRoundedRectangle(rect, border_brush.Get(), focused ? 2.0f : 1.0f);

    auto& renderer = D2DRenderer::instance();
    auto text_format = renderer.text_format_body();
    if (text_format) {
        text_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
        text_format->SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
    }

    if (recording) {
        // Show "Nhấn phím..." prompt when recording
        D2D1_RECT_F text_rect = {
            x + PADDING_X,
            y,
            x + width - PADDING_X,
            y + height
        };

        auto prompt_brush = create_brush(rt, Colors::TextSecondary);
        std::wstring prompt = L"Nhấn phím...";
        rt->DrawText(
            prompt.c_str(),
            static_cast<UINT32>(prompt.length()),
            text_format,
            text_rect,
            prompt_brush.Get()
        );
    } else if (!keys.empty()) {
        // Draw key caps (e.g., "Ctrl", "Shift", "V")
        float current_x = x + PADDING_X;
        constexpr float KEY_CAP_PADDING_X = 8.0f;
        constexpr float KEY_CAP_PADDING_Y = 4.0f;
        constexpr float KEY_CAP_SPACING = 6.0f;
        constexpr float KEY_CAP_RADIUS = 4.0f;

        auto key_bg_brush = create_brush(rt, Colors::Background);
        auto key_text_brush = create_brush(rt, Colors::Text);
        auto key_border_brush = create_brush(rt, Colors::Border);

        for (const auto& key : keys) {
            // Measure key text width
            ComPtr<IDWriteTextLayout> text_layout;
            renderer.dwrite_factory()->CreateTextLayout(
                key.c_str(),
                static_cast<UINT32>(key.length()),
                text_format,
                1000.0f,
                height,
                &text_layout
            );

            DWRITE_TEXT_METRICS metrics;
            text_layout->GetMetrics(&metrics);
            float key_width = metrics.width + KEY_CAP_PADDING_X * 2;
            float key_height = height - PADDING_Y * 2;

            // Draw key cap background
            D2D1_ROUNDED_RECT key_rect = {
                {current_x, y + PADDING_Y, current_x + key_width, y + PADDING_Y + key_height},
                KEY_CAP_RADIUS,
                KEY_CAP_RADIUS
            };
            rt->FillRoundedRectangle(key_rect, key_bg_brush.Get());
            rt->DrawRoundedRectangle(key_rect, key_border_brush.Get(), 1.0f);

            // Draw key text
            D2D1_RECT_F key_text_rect = {
                current_x + KEY_CAP_PADDING_X,
                y + PADDING_Y,
                current_x + key_width - KEY_CAP_PADDING_X,
                y + PADDING_Y + key_height
            };
            rt->DrawText(
                key.c_str(),
                static_cast<UINT32>(key.length()),
                text_format,
                key_text_rect,
                key_text_brush.Get()
            );

            current_x += key_width + KEY_CAP_SPACING;
        }
    } else {
        // Show placeholder when empty
        D2D1_RECT_F text_rect = {
            x + PADDING_X,
            y,
            x + width - PADDING_X,
            y + height
        };

        auto placeholder_brush = create_brush(rt, Colors::TextSecondary);
        std::wstring placeholder = L"Chưa đặt phím tắt";
        rt->DrawText(
            placeholder.c_str(),
            static_cast<UINT32>(placeholder.length()),
            text_format,
            text_rect,
            placeholder_brush.Get()
        );
    }
}

bool TextInput::hit_test(float x, float y, float width, float height, float mouse_x, float mouse_y) {
    return mouse_x >= x && mouse_x <= x + width &&
           mouse_y >= y && mouse_y <= y + height;
}

} // namespace gonhanh::ui
