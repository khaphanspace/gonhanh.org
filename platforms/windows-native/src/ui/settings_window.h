#pragma once

#include <windows.h>
#include <d2d1.h>
#include <wrl/client.h>
#include <string>

using Microsoft::WRL::ComPtr;

namespace gonhanh::ui {

enum class Page {
    Settings = 0,
    About = 1
};

// Settings window with sidebar layout (700x480)
class SettingsWindow {
public:
    static SettingsWindow& instance();

    // Show/hide window
    void show();
    void hide();
    bool is_visible() const;

    // Window dimensions
    static constexpr int WIDTH = 700;
    static constexpr int HEIGHT = 480;
    static constexpr int SIDEBAR_WIDTH = 200;
    static constexpr int CONTENT_WIDTH = WIDTH - SIDEBAR_WIDTH;

private:
    SettingsWindow() = default;
    ~SettingsWindow();
    SettingsWindow(const SettingsWindow&) = delete;
    SettingsWindow& operator=(const SettingsWindow&) = delete;

    bool create_window();
    void render();
    void render_sidebar();
    void render_content();
    void render_settings_page();
    void render_about_page();

    void handle_mouse_move(int x, int y);
    void handle_mouse_down(int x, int y);
    void handle_mouse_up(int x, int y);

    static LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam);

    // UI state
    HWND hwnd_ = nullptr;
    ComPtr<ID2D1HwndRenderTarget> render_target_;
    Page current_page_ = Page::Settings;

    // Hover tracking
    int hover_sidebar_item_ = -1;  // -1=none, 0=settings, 1=about
    int hover_toggle_index_ = -1;   // Which toggle is hovered
    bool hover_shortcuts_row_ = false;

    // Track mouse position
    int mouse_x_ = 0;
    int mouse_y_ = 0;
};

} // namespace gonhanh::ui
