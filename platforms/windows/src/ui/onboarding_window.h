#pragma once

#include <windows.h>
#include <d2d1.h>
#include <wrl/client.h>
#include <string>

using Microsoft::WRL::ComPtr;

namespace gonhanh::ui {

enum class OnboardingStep {
    Welcome = 0,
    Setup = 1
};

// Onboarding window - 2-step wizard (440x380)
class OnboardingWindow {
public:
    static OnboardingWindow& instance();

    // Show/hide window
    void show();
    void hide();
    bool is_visible() const;

    // Window dimensions
    static constexpr int WIDTH = 440;
    static constexpr int HEIGHT = 380;

private:
    OnboardingWindow() = default;
    ~OnboardingWindow();
    OnboardingWindow(const OnboardingWindow&) = delete;
    OnboardingWindow& operator=(const OnboardingWindow&) = delete;

    bool create_window();
    void render();
    void render_welcome_step();
    void render_setup_step();
    void render_footer();

    void handle_mouse_move(int x, int y);
    void handle_mouse_down(int x, int y);
    void handle_mouse_up(int x, int y);

    void next_step();
    void prev_step();
    void complete_onboarding();

    static LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam);

    // UI state
    HWND hwnd_ = nullptr;
    ComPtr<ID2D1HwndRenderTarget> render_target_;
    OnboardingStep current_step_ = OnboardingStep::Welcome;

    // Selection state
    int selected_method_ = 0;  // 0=Telex, 1=VNI

    // Hover tracking
    bool hover_primary_button_ = false;
    bool hover_back_button_ = false;
    bool hover_telex_option_ = false;
    bool hover_vni_option_ = false;

    // Mouse tracking
    int mouse_x_ = 0;
    int mouse_y_ = 0;
};

} // namespace gonhanh::ui
