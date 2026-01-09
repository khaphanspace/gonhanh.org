#include "dropdown.h"
#include "../d2d_renderer.h"
#include <cmath>

namespace gonhanh::ui {

void Dropdown::draw(
    ID2D1RenderTarget* rt,
    float x, float y,
    float width, float height,
    const std::wstring& label,
    const std::wstring& selected_value,
    bool expanded,
    bool hovered
) {
    constexpr float corner_radius = 6.0f;
    constexpr float padding_h = 12.0f;
    constexpr float chevron_size = 8.0f;
    constexpr float label_width_ratio = 0.4f;

    auto& renderer = D2DRenderer::instance();

    // Calculate layout
    float label_width = width * label_width_ratio;
    float value_x = x + label_width + 8.0f;
    float value_width = width - label_width - 8.0f;

    // Draw label (left side)
    auto label_color = create_brush(rt, Colors::Text);
    auto label_format = renderer.text_format_body();
    if (label_format) {
        label_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
        label_format->SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
    }

    D2D1_RECT_F label_rect = {x, y, x + label_width, y + height};
    rt->DrawText(
        label.c_str(),
        static_cast<UINT32>(label.length()),
        label_format,
        label_rect,
        label_color.Get()
    );

    // Draw value container (right side)
    D2D1_ROUNDED_RECT value_container = {
        {value_x, y, value_x + value_width, y + height},
        corner_radius,
        corner_radius
    };

    // Background color
    D2D1_COLOR_F bg_color = expanded ? D2D1::ColorF(0.97f, 0.97f, 0.98f) :
                           hovered ? D2D1::ColorF(0.98f, 0.98f, 0.99f) :
                           Colors::CardBg;

    auto bg_brush = create_brush(rt, bg_color);
    rt->FillRoundedRectangle(value_container, bg_brush.Get());

    // Border
    auto border_brush = create_brush(rt, Colors::Border);
    rt->DrawRoundedRectangle(value_container, border_brush.Get(), 1.0f);

    // Draw selected value text
    auto text_brush = create_brush(rt, Colors::Text);
    auto text_format = renderer.text_format_body();
    if (text_format) {
        text_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
        text_format->SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
    }

    D2D1_RECT_F text_rect = {
        value_x + padding_h,
        y,
        value_x + value_width - padding_h - chevron_size - 8.0f,
        y + height
    };
    rt->DrawText(
        selected_value.c_str(),
        static_cast<UINT32>(selected_value.length()),
        text_format,
        text_rect,
        text_brush.Get()
    );

    // Draw chevron (down arrow)
    float chevron_x = value_x + value_width - padding_h - chevron_size;
    float chevron_y = y + height / 2.0f;

    // Chevron points (down arrow)
    D2D1_POINT_2F chevron_points[3] = {
        {chevron_x, chevron_y - chevron_size / 4.0f},                           // Top left
        {chevron_x + chevron_size, chevron_y - chevron_size / 4.0f},            // Top right
        {chevron_x + chevron_size / 2.0f, chevron_y + chevron_size / 4.0f}     // Bottom center
    };

    // Create path for chevron
    ComPtr<ID2D1PathGeometry> path_geometry;
    renderer.d2d_factory()->CreatePathGeometry(&path_geometry);

    ComPtr<ID2D1GeometrySink> sink;
    path_geometry->Open(&sink);
    sink->BeginFigure(chevron_points[0], D2D1_FIGURE_BEGIN_FILLED);
    sink->AddLine(chevron_points[1]);
    sink->AddLine(chevron_points[2]);
    sink->EndFigure(D2D1_FIGURE_END_CLOSED);
    sink->Close();

    auto chevron_brush = create_brush(rt, Colors::TextSecondary);
    rt->FillGeometry(path_geometry.Get(), chevron_brush.Get());
}

void Dropdown::draw_menu(
    ID2D1RenderTarget* rt,
    float x, float y,
    float width,
    const std::vector<std::wstring>& items,
    int hovered_index,
    int selected_index
) {
    constexpr float corner_radius = 6.0f;
    constexpr float padding_h = 12.0f;

    float menu_height = MENU_PADDING * 2 + items.size() * ITEM_HEIGHT;

    // Menu background with shadow effect
    D2D1_ROUNDED_RECT menu_rect = {
        {x, y, x + width, y + menu_height},
        corner_radius,
        corner_radius
    };

    auto bg_brush = create_brush(rt, Colors::CardBg);
    rt->FillRoundedRectangle(menu_rect, bg_brush.Get());

    // Border
    auto border_brush = create_brush(rt, Colors::Border);
    rt->DrawRoundedRectangle(menu_rect, border_brush.Get(), 1.0f);

    // Draw items
    auto& renderer = D2DRenderer::instance();
    auto text_format = renderer.text_format_body();
    if (text_format) {
        text_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
        text_format->SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
    }

    for (size_t i = 0; i < items.size(); ++i) {
        float item_y = y + MENU_PADDING + i * ITEM_HEIGHT;

        // Highlight hovered/selected item
        if (static_cast<int>(i) == hovered_index) {
            D2D1_RECT_F hover_rect = {
                x + MENU_PADDING,
                item_y,
                x + width - MENU_PADDING,
                item_y + ITEM_HEIGHT
            };
            auto hover_brush = create_brush(rt, D2D1::ColorF(0.96f, 0.97f, 0.98f));
            rt->FillRectangle(hover_rect, hover_brush.Get());
        }

        // Text color (selected item uses primary color)
        D2D1_COLOR_F text_color = (static_cast<int>(i) == selected_index) ?
            Colors::Primary : Colors::Text;
        auto text_brush = create_brush(rt, text_color);

        D2D1_RECT_F text_rect = {
            x + padding_h,
            item_y,
            x + width - padding_h,
            item_y + ITEM_HEIGHT
        };

        rt->DrawText(
            items[i].c_str(),
            static_cast<UINT32>(items[i].length()),
            text_format,
            text_rect,
            text_brush.Get()
        );

        // Checkmark for selected item
        if (static_cast<int>(i) == selected_index) {
            float check_x = x + width - padding_h - 12.0f;
            float check_y = item_y + ITEM_HEIGHT / 2.0f;

            // Simple checkmark (✓)
            D2D1_POINT_2F check_points[3] = {
                {check_x, check_y},
                {check_x + 4.0f, check_y + 4.0f},
                {check_x + 10.0f, check_y - 4.0f}
            };

            auto check_brush = create_brush(rt, Colors::Primary);
            rt->DrawLine(check_points[0], check_points[1], check_brush.Get(), 2.0f);
            rt->DrawLine(check_points[1], check_points[2], check_brush.Get(), 2.0f);
        }
    }
}

bool Dropdown::hit_test(float x, float y, float width, float height, float mouse_x, float mouse_y) {
    return mouse_x >= x && mouse_x <= x + width &&
           mouse_y >= y && mouse_y <= y + height;
}

int Dropdown::menu_item_hit_test(
    float menu_x, float menu_y, float menu_width,
    const std::vector<std::wstring>& items,
    float mouse_x, float mouse_y
) {
    float menu_height = MENU_PADDING * 2 + items.size() * ITEM_HEIGHT;

    // Check if mouse is within menu bounds
    if (mouse_x < menu_x || mouse_x > menu_x + menu_width ||
        mouse_y < menu_y || mouse_y > menu_y + menu_height) {
        return -1;
    }

    // Calculate item index
    float relative_y = mouse_y - menu_y - MENU_PADDING;
    if (relative_y < 0) return -1;

    int index = static_cast<int>(relative_y / ITEM_HEIGHT);
    if (index >= 0 && index < static_cast<int>(items.size())) {
        return index;
    }

    return -1;
}

} // namespace gonhanh::ui
