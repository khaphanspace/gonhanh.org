#pragma once

#include <d2d1.h>
#include <string>
#include <vector>

namespace gonhanh::ui {

// Styled dropdown control matching macOS design
class Dropdown {
public:
    // Draw dropdown at position
    static void draw(
        ID2D1RenderTarget* rt,
        float x, float y,
        float width, float height,
        const std::wstring& label,
        const std::wstring& selected_value,
        bool expanded = false,
        bool hovered = false
    );

    // Draw dropdown menu (expanded state)
    static void draw_menu(
        ID2D1RenderTarget* rt,
        float x, float y,
        float width,
        const std::vector<std::wstring>& items,
        int hovered_index = -1,
        int selected_index = -1
    );

    // Hit test for dropdown control
    static bool hit_test(float x, float y, float width, float height, float mouse_x, float mouse_y);

    // Hit test for menu item
    static int menu_item_hit_test(
        float menu_x, float menu_y, float menu_width,
        const std::vector<std::wstring>& items,
        float mouse_x, float mouse_y
    );

    // Constants
    static constexpr float ITEM_HEIGHT = 36.0f;
    static constexpr float MENU_PADDING = 4.0f;
};

} // namespace gonhanh::ui
