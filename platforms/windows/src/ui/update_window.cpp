#include "update_window.h"
#include "d2d_renderer.h"
#include "../app.h"
#include "../update_manager.h"
#include <cmath>
#include <windowsx.h>

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

namespace gonhanh::ui {

static constexpr const wchar_t* UPDATE_WINDOW_CLASS = L"GoNhanhUpdateClass";

UpdateWindow& UpdateWindow::instance() {
    static UpdateWindow instance;
    return instance;
}

UpdateWindow::UpdateWindow() {
    // Set up callbacks
    auto& manager = gonhanh::UpdateManager::instance();
    manager.set_state_callback([this](gonhanh::UpdateState state) {
        on_state_changed(state);
    });
    manager.set_progress_callback([this](double progress) {
        on_progress_changed(progress);
    });
}

UpdateWindow::~UpdateWindow() {
    if (timer_id_) {
        KillTimer(hwnd_, timer_id_);
    }
    if (hwnd_) {
        DestroyWindow(hwnd_);
    }
}

void UpdateWindow::show() {
    if (!hwnd_) {
        if (!create_window()) return;
    }

    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
    InvalidateRect(hwnd_, nullptr, FALSE);
}

void UpdateWindow::hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
    }
    if (timer_id_) {
        KillTimer(hwnd_, timer_id_);
        timer_id_ = 0;
    }
}

bool UpdateWindow::is_visible() const {
    return hwnd_ && IsWindowVisible(hwnd_);
}

bool UpdateWindow::create_window() {
    auto& app = gonhanh::App::instance();

    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(wc);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = wnd_proc;
    wc.hInstance = app.hinstance();
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.lpszClassName = UPDATE_WINDOW_CLASS;

    if (!GetClassInfoExW(app.hinstance(), UPDATE_WINDOW_CLASS, &wc)) {
        RegisterClassExW(&wc);
    }

    int screen_width = GetSystemMetrics(SM_CXSCREEN);
    int screen_height = GetSystemMetrics(SM_CYSCREEN);
    int x = (screen_width - WIDTH) / 2;
    int y = (screen_height - HEIGHT) / 2;

    RECT rc = {0, 0, WIDTH, HEIGHT};
    AdjustWindowRectEx(&rc, WS_OVERLAPPEDWINDOW & ~WS_MAXIMIZEBOX & ~WS_THICKFRAME, FALSE, 0);

    hwnd_ = CreateWindowExW(
        0,
        UPDATE_WINDOW_CLASS,
        L"Cập nhật GoNhanh",
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

    auto& renderer = D2DRenderer::instance();
    if (!renderer.is_initialized()) {
        renderer.initialize();
    }
    render_target_.Attach(renderer.create_render_target(hwnd_));

    return true;
}

void UpdateWindow::render() {
    if (!render_target_) return;

    render_target_->BeginDraw();
    render_target_->Clear(Colors::Background);

    auto& manager = gonhanh::UpdateManager::instance();
    switch (manager.state()) {
        case gonhanh::UpdateState::Idle:
            render_idle();
            break;
        case gonhanh::UpdateState::Checking:
            render_checking();
            break;
        case gonhanh::UpdateState::Available:
            render_available();
            break;
        case gonhanh::UpdateState::UpToDate:
            render_up_to_date();
            break;
        case gonhanh::UpdateState::Downloading:
            render_downloading();
            break;
        case gonhanh::UpdateState::Installing:
            render_installing();
            break;
        case gonhanh::UpdateState::Error:
            render_error();
            break;
    }

    render_target_->EndDraw();
}

void UpdateWindow::render_idle() {
    auto& renderer = D2DRenderer::instance();
    auto text_brush = create_brush(render_target_.Get(), Colors::Text);
    auto text_secondary = create_brush(render_target_.Get(), Colors::TextSecondary);
    auto primary_brush = create_brush(render_target_.Get(), Colors::Primary);
    auto white_brush = create_brush(render_target_.Get(), D2D1::ColorF(1.0f, 1.0f, 1.0f));

    float cx = WIDTH / 2.0f;
    float y = 60.0f;

    // Logo placeholder (circle)
    D2D1_ELLIPSE logo = {{cx, y + 30}, 30, 30};
    render_target_->FillEllipse(logo, primary_brush.Get());

    // App name
    y += 80;
    D2D1_RECT_F title_rect = {0, y, WIDTH, y + 30};
    render_target_->DrawText(
        L"Gõ Nhanh",
        8,
        renderer.text_format_title(),
        title_rect,
        text_brush.Get()
    );

    // Current version
    y += 35;
    std::wstring version_text = L"Phiên bản hiện tại: 1.0.0"; // TODO: Get from app
    D2D1_RECT_F version_rect = {0, y, WIDTH, y + 24};
    render_target_->DrawText(
        version_text.c_str(),
        static_cast<UINT32>(version_text.length()),
        renderer.text_format_body(),
        version_rect,
        text_secondary.Get()
    );

    // Check button
    y = HEIGHT - PADDING - 40;
    btn_check_ = {cx - 80, y, cx + 80, y + 36};
    D2D1_RECT_F btn_rect = {btn_check_.left, btn_check_.top, btn_check_.right, btn_check_.bottom};
    render_target_->FillRectangle(btn_rect, primary_brush.Get());

    D2D1_RECT_F btn_text_rect = {btn_rect.left, btn_rect.top + 8, btn_rect.right, btn_rect.bottom - 8};
    render_target_->DrawText(
        L"Kiểm tra cập nhật",
        17,
        renderer.text_format_body(),
        btn_text_rect,
        white_brush.Get()
    );
}

void UpdateWindow::render_checking() {
    auto& renderer = D2DRenderer::instance();
    auto text_brush = create_brush(render_target_.Get(), Colors::Text);

    float cx = WIDTH / 2.0f;
    float cy = HEIGHT / 2.0f - 20;

    // Spinner
    draw_spinner(cx, cy, 24);

    // Text
    D2D1_RECT_F text_rect = {0, cy + 50, WIDTH, cy + 80};
    render_target_->DrawText(
        L"Đang kiểm tra cập nhật...",
        24,
        renderer.text_format_body(),
        text_rect,
        text_brush.Get()
    );
}

void UpdateWindow::render_available() {
    auto& renderer = D2DRenderer::instance();
    auto& manager = gonhanh::UpdateManager::instance();
    auto text_brush = create_brush(render_target_.Get(), Colors::Text);
    auto text_secondary = create_brush(render_target_.Get(), Colors::TextSecondary);
    auto primary_brush = create_brush(render_target_.Get(), Colors::Primary);
    auto border_brush = create_brush(render_target_.Get(), Colors::Border);
    auto white_brush = create_brush(render_target_.Get(), D2D1::ColorF(1.0f, 1.0f, 1.0f));

    float y = PADDING;

    // Title
    D2D1_RECT_F title_rect = {PADDING, y, WIDTH - PADDING, y + 30};
    render_target_->DrawText(
        L"Có bản cập nhật mới!",
        20,
        renderer.text_format_title(),
        title_rect,
        text_brush.Get()
    );

    // Version info
    y += 40;
    const auto& info = manager.update_info();
    std::wstring version_text = L"Phiên bản " + info.version;
    D2D1_RECT_F version_rect = {PADDING, y, WIDTH - PADDING, y + 24};
    render_target_->DrawText(
        version_text.c_str(),
        static_cast<UINT32>(version_text.length()),
        renderer.text_format_body(),
        version_rect,
        text_brush.Get()
    );

    // Release notes (scrollable area)
    y += 40;
    D2D1_RECT_F notes_bg = {PADDING, y, WIDTH - PADDING, HEIGHT - 100};
    render_target_->DrawRectangle(notes_bg, border_brush.Get(), 1.0f);

    D2D1_RECT_F notes_rect = {PADDING + 12, y + 8, WIDTH - PADDING - 12, HEIGHT - 108};
    std::wstring notes = info.release_notes.empty() ? L"Không có ghi chú" : info.release_notes;
    render_target_->DrawText(
        notes.c_str(),
        static_cast<UINT32>(notes.length()),
        renderer.text_format_small(),
        notes_rect,
        text_secondary.Get()
    );

    // Buttons
    y = HEIGHT - PADDING - 40;
    float btn_width = 100;

    // Skip button
    btn_skip_ = {PADDING, y, PADDING + btn_width, y + 36};
    D2D1_RECT_F skip_rect = {btn_skip_.left, btn_skip_.top, btn_skip_.right, btn_skip_.bottom};
    render_target_->DrawRectangle(skip_rect, border_brush.Get(), 1.0f);
    D2D1_RECT_F skip_text = {skip_rect.left, skip_rect.top + 8, skip_rect.right, skip_rect.bottom - 8};
    render_target_->DrawText(L"Bỏ qua", 6, renderer.text_format_body(), skip_text, text_brush.Get());

    // Download button
    btn_download_ = {WIDTH - PADDING - btn_width, y, WIDTH - PADDING, y + 36};
    D2D1_RECT_F download_rect = {btn_download_.left, btn_download_.top, btn_download_.right, btn_download_.bottom};
    render_target_->FillRectangle(download_rect, primary_brush.Get());
    D2D1_RECT_F download_text = {download_rect.left, download_rect.top + 8, download_rect.right, download_rect.bottom - 8};
    render_target_->DrawText(L"Tải về", 6, renderer.text_format_body(), download_text, white_brush.Get());
}

void UpdateWindow::render_up_to_date() {
    auto& renderer = D2DRenderer::instance();
    auto text_brush = create_brush(render_target_.Get(), Colors::Text);
    auto success_brush = create_brush(render_target_.Get(), D2D1::ColorF(0x22C55E));
    auto primary_brush = create_brush(render_target_.Get(), Colors::Primary);
    auto white_brush = create_brush(render_target_.Get(), D2D1::ColorF(1.0f, 1.0f, 1.0f));

    float cx = WIDTH / 2.0f;
    float cy = HEIGHT / 2.0f - 40;

    // Checkmark circle
    D2D1_ELLIPSE circle = {{cx, cy}, 30, 30};
    render_target_->FillEllipse(circle, success_brush.Get());

    // Checkmark (simple line drawing)
    auto stroke_brush = create_brush(render_target_.Get(), D2D1::ColorF(1.0f, 1.0f, 1.0f));
    render_target_->DrawLine(
        D2D1::Point2F(cx - 12, cy),
        D2D1::Point2F(cx - 2, cy + 10),
        stroke_brush.Get(), 3.0f
    );
    render_target_->DrawLine(
        D2D1::Point2F(cx - 2, cy + 10),
        D2D1::Point2F(cx + 14, cy - 8),
        stroke_brush.Get(), 3.0f
    );

    // Text
    D2D1_RECT_F text_rect = {0, cy + 50, WIDTH, cy + 80};
    render_target_->DrawText(
        L"Bạn đang dùng phiên bản mới nhất",
        32,
        renderer.text_format_body(),
        text_rect,
        text_brush.Get()
    );

    // Close button
    float y = HEIGHT - PADDING - 40;
    btn_close_ = {cx - 60, y, cx + 60, y + 36};
    D2D1_RECT_F btn_rect = {btn_close_.left, btn_close_.top, btn_close_.right, btn_close_.bottom};
    render_target_->FillRectangle(btn_rect, primary_brush.Get());
    D2D1_RECT_F btn_text = {btn_rect.left, btn_rect.top + 8, btn_rect.right, btn_rect.bottom - 8};
    render_target_->DrawText(L"Đóng", 4, renderer.text_format_body(), btn_text, white_brush.Get());
}

void UpdateWindow::render_downloading() {
    auto& renderer = D2DRenderer::instance();
    auto& manager = gonhanh::UpdateManager::instance();
    auto text_brush = create_brush(render_target_.Get(), Colors::Text);
    auto text_secondary = create_brush(render_target_.Get(), Colors::TextSecondary);

    float cx = WIDTH / 2.0f;
    float cy = HEIGHT / 2.0f - 20;

    // Progress circle
    draw_progress_circle(cx, cy, 40, manager.download_progress());

    // Percentage text
    int percent = static_cast<int>(manager.download_progress() * 100);
    std::wstring percent_text = std::to_wstring(percent) + L"%";
    D2D1_RECT_F percent_rect = {cx - 30, cy - 12, cx + 30, cy + 12};
    render_target_->DrawText(
        percent_text.c_str(),
        static_cast<UINT32>(percent_text.length()),
        renderer.text_format_body(),
        percent_rect,
        text_brush.Get()
    );

    // Status text
    D2D1_RECT_F text_rect = {0, cy + 60, WIDTH, cy + 90};
    render_target_->DrawText(
        L"Đang tải về...",
        13,
        renderer.text_format_body(),
        text_rect,
        text_secondary.Get()
    );
}

void UpdateWindow::render_installing() {
    auto& renderer = D2DRenderer::instance();
    auto text_brush = create_brush(render_target_.Get(), Colors::Text);

    float cx = WIDTH / 2.0f;
    float cy = HEIGHT / 2.0f - 20;

    // Spinner
    draw_spinner(cx, cy, 24);

    // Text
    D2D1_RECT_F text_rect = {0, cy + 50, WIDTH, cy + 80};
    render_target_->DrawText(
        L"Đang cài đặt...",
        15,
        renderer.text_format_body(),
        text_rect,
        text_brush.Get()
    );
}

void UpdateWindow::render_error() {
    auto& renderer = D2DRenderer::instance();
    auto& manager = gonhanh::UpdateManager::instance();
    auto text_brush = create_brush(render_target_.Get(), Colors::Text);
    auto error_brush = create_brush(render_target_.Get(), D2D1::ColorF(0xEF4444));
    auto primary_brush = create_brush(render_target_.Get(), Colors::Primary);
    auto white_brush = create_brush(render_target_.Get(), D2D1::ColorF(1.0f, 1.0f, 1.0f));

    float cx = WIDTH / 2.0f;
    float cy = HEIGHT / 2.0f - 60;

    // Error icon (X in circle)
    D2D1_ELLIPSE circle = {{cx, cy}, 30, 30};
    render_target_->FillEllipse(circle, error_brush.Get());

    auto x_brush = create_brush(render_target_.Get(), D2D1::ColorF(1.0f, 1.0f, 1.0f));
    render_target_->DrawLine(D2D1::Point2F(cx - 10, cy - 10), D2D1::Point2F(cx + 10, cy + 10), x_brush.Get(), 3.0f);
    render_target_->DrawLine(D2D1::Point2F(cx + 10, cy - 10), D2D1::Point2F(cx - 10, cy + 10), x_brush.Get(), 3.0f);

    // Error title
    D2D1_RECT_F title_rect = {0, cy + 50, WIDTH, cy + 80};
    render_target_->DrawText(
        L"Đã xảy ra lỗi",
        13,
        renderer.text_format_body(),
        title_rect,
        text_brush.Get()
    );

    // Error message
    const auto& error = manager.error_message();
    D2D1_RECT_F error_rect = {PADDING, cy + 85, WIDTH - PADDING, cy + 130};
    render_target_->DrawText(
        error.c_str(),
        static_cast<UINT32>(error.length()),
        renderer.text_format_small(),
        error_rect,
        error_brush.Get()
    );

    // Retry button
    float y = HEIGHT - PADDING - 40;
    btn_retry_ = {cx - 60, y, cx + 60, y + 36};
    D2D1_RECT_F btn_rect = {btn_retry_.left, btn_retry_.top, btn_retry_.right, btn_retry_.bottom};
    render_target_->FillRectangle(btn_rect, primary_brush.Get());
    D2D1_RECT_F btn_text = {btn_rect.left, btn_rect.top + 8, btn_rect.right, btn_rect.bottom - 8};
    render_target_->DrawText(L"Thử lại", 7, renderer.text_format_body(), btn_text, white_brush.Get());
}

void UpdateWindow::draw_progress_circle(float cx, float cy, float radius, double progress) {
    auto bg_brush = create_brush(render_target_.Get(), Colors::Border);
    auto progress_brush = create_brush(render_target_.Get(), Colors::Primary);

    // Background circle
    D2D1_ELLIPSE bg_circle = {{cx, cy}, radius, radius};
    render_target_->DrawEllipse(bg_circle, bg_brush.Get(), 4.0f);

    // Progress arc
    if (progress > 0) {
        ComPtr<ID2D1Factory> factory;
        render_target_->GetFactory(&factory);

        ComPtr<ID2D1PathGeometry> path;
        factory->CreatePathGeometry(&path);

        ComPtr<ID2D1GeometrySink> sink;
        path->Open(&sink);

        float start_angle = -static_cast<float>(M_PI) / 2.0f;
        float sweep = static_cast<float>(progress * 2.0 * M_PI);

        D2D1_POINT_2F start = {
            cx + radius * cosf(start_angle),
            cy + radius * sinf(start_angle)
        };

        D2D1_POINT_2F end = {
            cx + radius * cosf(start_angle + sweep),
            cy + radius * sinf(start_angle + sweep)
        };

        sink->BeginFigure(start, D2D1_FIGURE_BEGIN_HOLLOW);
        sink->AddArc(D2D1::ArcSegment(
            end,
            D2D1::SizeF(radius, radius),
            0,
            D2D1_SWEEP_DIRECTION_CLOCKWISE,
            progress > 0.5 ? D2D1_ARC_SIZE_LARGE : D2D1_ARC_SIZE_SMALL
        ));
        sink->EndFigure(D2D1_FIGURE_END_OPEN);
        sink->Close();

        render_target_->DrawGeometry(path.Get(), progress_brush.Get(), 4.0f);
    }
}

void UpdateWindow::draw_spinner(float cx, float cy, float radius) {
    auto brush = create_brush(render_target_.Get(), Colors::Primary);

    // Draw partial arc (spinner)
    ComPtr<ID2D1Factory> factory;
    render_target_->GetFactory(&factory);

    ComPtr<ID2D1PathGeometry> path;
    factory->CreatePathGeometry(&path);

    ComPtr<ID2D1GeometrySink> sink;
    path->Open(&sink);

    float start_angle = spinner_angle_;
    float sweep = static_cast<float>(M_PI) * 1.5f;  // 3/4 circle

    D2D1_POINT_2F start = {
        cx + radius * cosf(start_angle),
        cy + radius * sinf(start_angle)
    };

    D2D1_POINT_2F end = {
        cx + radius * cosf(start_angle + sweep),
        cy + radius * sinf(start_angle + sweep)
    };

    sink->BeginFigure(start, D2D1_FIGURE_BEGIN_HOLLOW);
    sink->AddArc(D2D1::ArcSegment(
        end,
        D2D1::SizeF(radius, radius),
        0,
        D2D1_SWEEP_DIRECTION_CLOCKWISE,
        D2D1_ARC_SIZE_LARGE
    ));
    sink->EndFigure(D2D1_FIGURE_END_OPEN);
    sink->Close();

    render_target_->DrawGeometry(path.Get(), brush.Get(), 3.0f);
}

void UpdateWindow::handle_mouse_down(int x, int y) {
    auto& manager = gonhanh::UpdateManager::instance();

    switch (manager.state()) {
        case gonhanh::UpdateState::Idle:
            if (btn_check_.contains(x, y)) {
                manager.check_for_updates_manual();
            }
            break;

        case gonhanh::UpdateState::Available:
            if (btn_download_.contains(x, y)) {
                manager.download_update();
            } else if (btn_skip_.contains(x, y)) {
                manager.skip_version(manager.update_info().version);
            }
            break;

        case gonhanh::UpdateState::UpToDate:
            if (btn_close_.contains(x, y)) {
                hide();
            }
            break;

        case gonhanh::UpdateState::Error:
            if (btn_retry_.contains(x, y)) {
                manager.reset();
                manager.check_for_updates_manual();
            }
            break;

        default:
            break;
    }
}

void UpdateWindow::on_state_changed(gonhanh::UpdateState state) {
    if (!hwnd_) return;

    // Start/stop animation timer for spinner states
    bool needs_animation = (state == gonhanh::UpdateState::Checking ||
                           state == gonhanh::UpdateState::Installing);

    if (needs_animation && !timer_id_) {
        timer_id_ = SetTimer(hwnd_, 1, 16, nullptr);  // ~60fps
    } else if (!needs_animation && timer_id_) {
        KillTimer(hwnd_, timer_id_);
        timer_id_ = 0;
    }

    InvalidateRect(hwnd_, nullptr, FALSE);
}

void UpdateWindow::on_progress_changed(double /*progress*/) {
    if (!hwnd_) return;
    InvalidateRect(hwnd_, nullptr, FALSE);
}

LRESULT CALLBACK UpdateWindow::wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    UpdateWindow* self = nullptr;

    if (msg == WM_NCCREATE) {
        auto* cs = reinterpret_cast<CREATESTRUCTW*>(lparam);
        self = static_cast<UpdateWindow*>(cs->lpCreateParams);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
    } else {
        self = reinterpret_cast<UpdateWindow*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
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

        case WM_LBUTTONDOWN:
            if (self) {
                self->handle_mouse_down(GET_X_LPARAM(lparam), GET_Y_LPARAM(lparam));
            }
            return 0;

        case WM_TIMER:
            if (self && wparam == 1) {
                self->spinner_angle_ += 0.15f;
                if (self->spinner_angle_ > 2.0f * static_cast<float>(M_PI)) {
                    self->spinner_angle_ -= 2.0f * static_cast<float>(M_PI);
                }
                InvalidateRect(hwnd, nullptr, FALSE);
            }
            return 0;

        case WM_CLOSE:
            ShowWindow(hwnd, SW_HIDE);
            return 0;

        default:
            return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
}

} // namespace gonhanh::ui
