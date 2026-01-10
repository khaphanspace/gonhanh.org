#pragma once

#include <windows.h>
#include <d2d1.h>
#include <wrl/client.h>
#include <string>

using Microsoft::WRL::ComPtr;

namespace gonhanh::ui {

// About window displaying app information, links, and credits (400x520)
class AboutWindow {
public:
    static AboutWindow& instance();

    // Show/hide window
    void show();
    void hide();
    bool is_visible() const;

    // Window dimensions
    static constexpr int WIDTH = 400;
    static constexpr int HEIGHT = 520;

private:
    AboutWindow() = default;
    ~AboutWindow();
    AboutWindow(const AboutWindow&) = delete;
    AboutWindow& operator=(const AboutWindow&) = delete;

    bool create_window();
    void render();
    void handle_click(int x, int y);

    static LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam);

    // Link button structure for hit testing
    struct LinkButton {
        D2D1_RECT_F bounds;
        std::wstring url;
        std::wstring label;
        bool hovered = false;
    };

    HWND hwnd_ = nullptr;
    ComPtr<ID2D1HwndRenderTarget> render_target_;
    ComPtr<ID2D1Bitmap> logo_bitmap_;

    // Link buttons for interaction
    LinkButton github_btn_;
    LinkButton issues_btn_;
    LinkButton sponsor_btn_;
    LinkButton linkedin_btn_;

    POINT last_mouse_pos_ = {-1, -1};
};

} // namespace gonhanh::ui
