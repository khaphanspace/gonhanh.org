#pragma once

#include <d2d1.h>
#include <string>
#include <vector>

namespace gonhanh::ui {

struct SidebarItem {
    std::wstring label;
    int id;
};

// Navigation sidebar with selection state
class Sidebar {
public:
    static constexpr float ITEM_HEIGHT = 36.0f;
    static constexpr float ITEM_PADDING = 12.0f;
    static constexpr float CORNER_RADIUS = 6.0f;

    // Draw sidebar item
    static void draw_item(
        ID2D1RenderTarget* rt,
        float x, float y,
        float width,
        const std::wstring& text,
        bool selected,
        bool hovered = false
    );

    // Hit test for item
    static bool hit_test_item(float x, float y, float width, float mouse_x, float mouse_y);
};

} // namespace gonhanh::ui
