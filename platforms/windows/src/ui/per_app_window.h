#pragma once

#include <windows.h>
#include <d2d1.h>
#include <dwrite.h>
#include <wrl/client.h>
#include <string>
#include <vector>

using Microsoft::WRL::ComPtr;

namespace gonhanh::ui {

// Per-app exception management window
// Allows users to disable GoNhanh for specific applications
class PerAppWindow {
public:
    static PerAppWindow& instance();

    void show();
    void hide();
    bool is_visible() const;

    // Window dimensions
    static constexpr int WIDTH = 500;
    static constexpr int HEIGHT = 450;

private:
    PerAppWindow() = default;
    ~PerAppWindow();
    PerAppWindow(const PerAppWindow&) = delete;
    PerAppWindow& operator=(const PerAppWindow&) = delete;

    bool create_window();
    void render();
    void load_disabled_apps();

    // UI interaction
    void handle_mouse_move(int x, int y);
    void handle_click(int x, int y);
    void handle_scroll(int delta);

    // App management
    void add_app_from_picker();
    void add_running_app(int index);
    void remove_app(int index);

    // Running apps detection
    void refresh_running_apps();

    static LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam);

    HWND hwnd_ = nullptr;
    ComPtr<ID2D1HwndRenderTarget> render_target_;

    // Disabled apps list
    std::vector<std::wstring> disabled_apps_;

    // Running apps list (for quick add)
    struct RunningApp {
        std::wstring name;
        std::wstring path;
        HICON icon = nullptr;
    };
    std::vector<RunningApp> running_apps_;
    bool show_running_apps_ = false;

    // UI state
    int scroll_offset_ = 0;
    int hovered_item_ = -1;          // Which disabled app is hovered
    int hovered_delete_ = -1;        // Which delete button is hovered
    int hovered_running_ = -1;       // Which running app is hovered
    bool hovered_add_btn_ = false;   // Add button hovered
    bool hovered_refresh_btn_ = false;
};

} // namespace gonhanh::ui
