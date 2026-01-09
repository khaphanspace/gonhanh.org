#pragma once

#include <windows.h>
#include <d2d1.h>
#include <wrl/client.h>

using Microsoft::WRL::ComPtr;

namespace gonhanh::ui {

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

private:
    SettingsWindow() = default;
    ~SettingsWindow();
    SettingsWindow(const SettingsWindow&) = delete;
    SettingsWindow& operator=(const SettingsWindow&) = delete;

    bool create_window();
    void render();

    static LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam);

    HWND hwnd_ = nullptr;
    ComPtr<ID2D1HwndRenderTarget> render_target_;
    int selected_tab_ = 0;  // 0=Settings, 1=Shortcuts, 2=About
};

} // namespace gonhanh::ui
