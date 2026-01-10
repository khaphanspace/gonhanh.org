#pragma once

#include <Windows.h>
#include <d2d1.h>
#include <dwrite.h>
#include <string>

namespace gonhanh {

class CompositionWindow {
public:
    CompositionWindow();
    ~CompositionWindow();

    bool create(HINSTANCE instance);
    void destroy();

    // Display control
    void show(const std::wstring& buffer, int caret_x, int caret_y);
    void hide();
    void update_text(const std::wstring& buffer);
    void update_position(int caret_x, int caret_y);

    bool is_visible() const { return visible_; }

private:
    static LRESULT CALLBACK window_proc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp);
    LRESULT handle_message(UINT msg, WPARAM wp, LPARAM lp);

    void create_render_target();
    void release_render_resources();
    void render();
    D2D1_SIZE_F measure_text(const std::wstring& text);
    void position_window(int caret_x, int caret_y);

    HWND hwnd_ = nullptr;
    HINSTANCE instance_ = nullptr;

    // Direct2D
    ID2D1Factory* d2d_factory_ = nullptr;
    ID2D1HwndRenderTarget* render_target_ = nullptr;
    ID2D1SolidColorBrush* bg_brush_ = nullptr;
    ID2D1SolidColorBrush* text_brush_ = nullptr;
    ID2D1SolidColorBrush* border_brush_ = nullptr;

    // DirectWrite
    IDWriteFactory* dwrite_factory_ = nullptr;
    IDWriteTextFormat* text_format_ = nullptr;

    std::wstring buffer_;
    bool visible_ = false;

    // Constants
    static constexpr float PADDING_H = 10.0f;
    static constexpr float PADDING_V = 6.0f;
    static constexpr int MIN_WIDTH = 80;
    static constexpr int MAX_WIDTH = 400;
    static constexpr int HEIGHT = 28;
    static constexpr int OFFSET_Y = 20;  // Below caret
};

} // namespace gonhanh
