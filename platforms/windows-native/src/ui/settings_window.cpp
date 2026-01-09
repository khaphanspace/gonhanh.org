#include "settings_window.h"
#include "d2d_renderer.h"
#include "../app.h"

namespace gonhanh::ui {

static constexpr const wchar_t* SETTINGS_WINDOW_CLASS = L"GoNhanhSettingsClass";

SettingsWindow& SettingsWindow::instance() {
    static SettingsWindow instance;
    return instance;
}

SettingsWindow::~SettingsWindow() {
    if (hwnd_) {
        DestroyWindow(hwnd_);
        hwnd_ = nullptr;
    }
}

void SettingsWindow::show() {
    if (!hwnd_) {
        if (!create_window()) return;
    }

    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
}

void SettingsWindow::hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
    }
}

bool SettingsWindow::is_visible() const {
    return hwnd_ && IsWindowVisible(hwnd_);
}

bool SettingsWindow::create_window() {
    auto& app = gonhanh::App::instance();

    // Register window class
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(wc);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = wnd_proc;
    wc.hInstance = app.hinstance();
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.lpszClassName = SETTINGS_WINDOW_CLASS;

    if (!GetClassInfoExW(app.hinstance(), SETTINGS_WINDOW_CLASS, &wc)) {
        RegisterClassExW(&wc);
    }

    // Calculate window position (center screen)
    int screen_width = GetSystemMetrics(SM_CXSCREEN);
    int screen_height = GetSystemMetrics(SM_CYSCREEN);
    int x = (screen_width - WIDTH) / 2;
    int y = (screen_height - HEIGHT) / 2;

    // Adjust for window chrome
    RECT rc = {0, 0, WIDTH, HEIGHT};
    AdjustWindowRectEx(&rc, WS_OVERLAPPEDWINDOW & ~WS_MAXIMIZEBOX & ~WS_THICKFRAME, FALSE, 0);

    hwnd_ = CreateWindowExW(
        0,
        SETTINGS_WINDOW_CLASS,
        L"GoNhanh Cài đặt",
        (WS_OVERLAPPEDWINDOW & ~WS_MAXIMIZEBOX & ~WS_THICKFRAME),
        x, y,
        rc.right - rc.left,
        rc.bottom - rc.top,
        nullptr,
        nullptr,
        app.hinstance(),
        this
    );

    if (!hwnd_) return false;

    // Create render target
    auto& renderer = D2DRenderer::instance();
    if (!renderer.is_initialized()) {
        renderer.initialize();
    }
    render_target_.Attach(renderer.create_render_target(hwnd_));

    return true;
}

void SettingsWindow::render() {
    if (!render_target_) return;

    render_target_->BeginDraw();
    render_target_->Clear(Colors::Background);

    // Draw sidebar background
    auto sidebar_brush = create_brush(render_target_.Get(), D2D1::ColorF(0.95f, 0.95f, 0.96f));
    D2D1_RECT_F sidebar_rect = {0, 0, static_cast<float>(SIDEBAR_WIDTH), static_cast<float>(HEIGHT)};
    render_target_->FillRectangle(sidebar_rect, sidebar_brush.Get());

    // Draw sidebar border
    auto border_brush = create_brush(render_target_.Get(), Colors::Border);
    render_target_->DrawLine(
        D2D1::Point2F(static_cast<float>(SIDEBAR_WIDTH), 0),
        D2D1::Point2F(static_cast<float>(SIDEBAR_WIDTH), static_cast<float>(HEIGHT)),
        border_brush.Get(),
        1.0f
    );

    // Draw title in sidebar
    auto text_brush = create_brush(render_target_.Get(), Colors::Primary);
    auto& renderer = D2DRenderer::instance();

    D2D1_RECT_F title_rect = {20, 20, static_cast<float>(SIDEBAR_WIDTH - 20), 60};
    render_target_->DrawText(
        L"GoNhanh",
        7,
        renderer.text_format_title(),
        title_rect,
        text_brush.Get()
    );

    // Draw content area placeholder
    auto content_text_brush = create_brush(render_target_.Get(), Colors::TextSecondary);
    D2D1_RECT_F content_rect = {
        static_cast<float>(SIDEBAR_WIDTH + 24),
        24,
        static_cast<float>(WIDTH - 24),
        static_cast<float>(HEIGHT - 24)
    };
    render_target_->DrawText(
        L"Settings UI - Coming Soon",
        25,
        renderer.text_format_body(),
        content_rect,
        content_text_brush.Get()
    );

    render_target_->EndDraw();
}

LRESULT CALLBACK SettingsWindow::wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    SettingsWindow* self = nullptr;

    if (msg == WM_NCCREATE) {
        auto* cs = reinterpret_cast<CREATESTRUCTW*>(lparam);
        self = static_cast<SettingsWindow*>(cs->lpCreateParams);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
    } else {
        self = reinterpret_cast<SettingsWindow*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    }

    switch (msg) {
        case WM_PAINT: {
            PAINTSTRUCT ps;
            BeginPaint(hwnd, &ps);
            if (self) self->render();
            EndPaint(hwnd, &ps);
            return 0;
        }

        case WM_SIZE:
            if (self && self->render_target_) {
                D2D1_SIZE_U size = {LOWORD(lparam), HIWORD(lparam)};
                self->render_target_->Resize(size);
            }
            return 0;

        case WM_CLOSE:
            ShowWindow(hwnd, SW_HIDE);
            return 0;

        case WM_DESTROY:
            return 0;

        default:
            return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
}

} // namespace gonhanh::ui
