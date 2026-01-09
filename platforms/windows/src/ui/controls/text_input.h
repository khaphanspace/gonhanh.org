#pragma once

#include <d2d1.h>
#include <string>
#include <vector>

namespace gonhanh::ui {

// Text input control for settings and shortcuts
class TextInput {
public:
    // Constants
    static constexpr float DEFAULT_HEIGHT = 40.0f;
    static constexpr float PADDING_X = 12.0f;
    static constexpr float PADDING_Y = 10.0f;
    static constexpr float BORDER_RADIUS = 6.0f;

    // Regular text input
    static void draw(
        ID2D1RenderTarget* rt,
        float x, float y,
        float width, float height,
        const std::wstring& text,
        const std::wstring& placeholder = L"",
        bool focused = false
    );

    // Hotkey recording input (shows key caps like "Ctrl", "Shift", "V")
    static void draw_hotkey_input(
        ID2D1RenderTarget* rt,
        float x, float y,
        float width, float height,
        const std::vector<std::wstring>& keys,
        bool recording = false,
        bool focused = false
    );

    // Hit test
    static bool hit_test(float x, float y, float width, float height, float mouse_x, float mouse_y);
};

} // namespace gonhanh::ui
