#pragma once

#include <Windows.h>
#include <d2d1.h>
#include <wrl/client.h>
#include "../update_manager.h"

using Microsoft::WRL::ComPtr;

namespace gonhanh::ui {

// Update dialog window (400x350)
class UpdateWindow {
public:
    static UpdateWindow& instance();

    void show();
    void hide();
    bool is_visible() const;

    // Dimensions
    static constexpr int WIDTH = 400;
    static constexpr int HEIGHT = 350;
    static constexpr float PADDING = 24.0f;

private:
    UpdateWindow();
    ~UpdateWindow();
    UpdateWindow(const UpdateWindow&) = delete;
    UpdateWindow& operator=(const UpdateWindow&) = delete;

    bool create_window();
    void render();
    void render_idle();
    void render_checking();
    void render_available();
    void render_up_to_date();
    void render_downloading();
    void render_installing();
    void render_error();

    void draw_progress_circle(float cx, float cy, float radius, double progress);
    void draw_spinner(float cx, float cy, float radius);

    void handle_mouse_down(int x, int y);
    void on_state_changed(gonhanh::UpdateState state);
    void on_progress_changed(double progress);

    static LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam);

    HWND hwnd_ = nullptr;
    ComPtr<ID2D1HwndRenderTarget> render_target_;

    // Animation
    float spinner_angle_ = 0.0f;
    UINT_PTR timer_id_ = 0;

    // Button hit testing
    struct ButtonRect {
        float left, top, right, bottom;
        bool contains(int x, int y) const {
            return x >= left && x <= right && y >= top && y <= bottom;
        }
    };

    ButtonRect btn_check_;
    ButtonRect btn_download_;
    ButtonRect btn_skip_;
    ButtonRect btn_close_;
    ButtonRect btn_retry_;
};

} // namespace gonhanh::ui
