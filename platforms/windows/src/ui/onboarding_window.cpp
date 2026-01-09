#include "onboarding_window.h"
#include "d2d_renderer.h"
#include "controls/button.h"
#include "../app.h"
#include "../settings.h"
#include <windowsx.h>

namespace gonhanh::ui {

static constexpr const wchar_t* ONBOARDING_WINDOW_CLASS = L"GoNhanhOnboardingClass";

// Layout constants
static constexpr float CONTENT_PADDING = 40.0f;
static constexpr float FOOTER_HEIGHT = 80.0f;
static constexpr float STEP_INDICATOR_SIZE = 8.0f;
static constexpr float BUTTON_HEIGHT = 40.0f;
static constexpr float BUTTON_WIDTH = 120.0f;
static constexpr float OPTION_HEIGHT = 70.0f;
static constexpr float OPTION_PADDING = 16.0f;

OnboardingWindow& OnboardingWindow::instance() {
    static OnboardingWindow instance;
    return instance;
}

OnboardingWindow::~OnboardingWindow() {
    if (hwnd_) {
        DestroyWindow(hwnd_);
        hwnd_ = nullptr;
    }
}

void OnboardingWindow::show() {
    if (!hwnd_) {
        if (!create_window()) return;
    }

    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
}

void OnboardingWindow::hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
    }
}

bool OnboardingWindow::is_visible() const {
    return hwnd_ && IsWindowVisible(hwnd_);
}

bool OnboardingWindow::create_window() {
    auto& app = gonhanh::App::instance();

    // Register window class
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(wc);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = wnd_proc;
    wc.hInstance = app.hinstance();
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.lpszClassName = ONBOARDING_WINDOW_CLASS;

    if (!GetClassInfoExW(app.hinstance(), ONBOARDING_WINDOW_CLASS, &wc)) {
        RegisterClassExW(&wc);
    }

    // Get DPI scale factor for proper Windows 11 style scaling
    float dpi_scale = get_dpi_scale();
    int scaled_width = scale_by_dpi(WIDTH, dpi_scale);
    int scaled_height = scale_by_dpi(HEIGHT, dpi_scale);

    // Adjust for window chrome
    RECT rc = {0, 0, scaled_width, scaled_height};
    AdjustWindowRectEx(&rc, WS_OVERLAPPEDWINDOW & ~WS_MAXIMIZEBOX & ~WS_THICKFRAME & ~WS_MINIMIZEBOX, FALSE, 0);

    // Calculate window position (center screen)
    int screen_width = GetSystemMetrics(SM_CXSCREEN);
    int screen_height = GetSystemMetrics(SM_CYSCREEN);
    int x = (screen_width - (rc.right - rc.left)) / 2;
    int y = (screen_height - (rc.bottom - rc.top)) / 2;

    hwnd_ = CreateWindowExW(
        0,
        ONBOARDING_WINDOW_CLASS,
        L"Chào mừng đến với GoNhanh",
        (WS_OVERLAPPEDWINDOW & ~WS_MAXIMIZEBOX & ~WS_THICKFRAME & ~WS_MINIMIZEBOX),
        x, y,
        rc.right - rc.left,
        rc.bottom - rc.top,
        nullptr,
        nullptr,
        app.hinstance(),
        this
    );

    if (!hwnd_) return false;

    // Ensure client area matches expected dimensions (fixes DPI scaling issues)
    ensure_client_area(hwnd_, WIDTH, HEIGHT);

    // Create render target
    auto& renderer = D2DRenderer::instance();
    if (!renderer.is_initialized()) {
        renderer.initialize();
    }
    render_target_.Attach(renderer.create_render_target(hwnd_));

    return true;
}

void OnboardingWindow::render() {
    if (!render_target_) return;

    render_target_->BeginDraw();
    render_target_->Clear(Colors::CardBg);

    if (current_step_ == OnboardingStep::Welcome) {
        render_welcome_step();
    } else {
        render_setup_step();
    }

    render_footer();

    render_target_->EndDraw();
}

void OnboardingWindow::render_welcome_step() {
    auto& renderer = D2DRenderer::instance();
    float content_y = 60.0f;

    // Logo placeholder (80x80) - centered
    float logo_x = (WIDTH - 80.0f) / 2.0f;
    auto logo_brush = create_brush(render_target_.Get(), Colors::Primary);
    D2D1_ROUNDED_RECT logo_rect = {
        {logo_x, content_y, logo_x + 80.0f, content_y + 80.0f},
        8.0f, 8.0f
    };
    render_target_->FillRoundedRectangle(logo_rect, logo_brush.Get());

    // "GoNhanh" text on logo
    auto white_brush = create_brush(render_target_.Get(), D2D1::ColorF(D2D1::ColorF::White));
    D2D1_RECT_F logo_text_rect = {logo_x, content_y + 25.0f, logo_x + 80.0f, content_y + 55.0f};

    ComPtr<IDWriteTextFormat> logo_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI",
        nullptr,
        DWRITE_FONT_WEIGHT_BOLD,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        16.0f,
        L"vi-VN",
        &logo_format
    );
    logo_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
    logo_format->SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

    render_target_->DrawText(
        L"GoNhanh",
        8,
        logo_format.Get(),
        logo_text_rect,
        white_brush.Get()
    );

    content_y += 100.0f;

    // Title
    auto title_brush = create_brush(render_target_.Get(), Colors::Text);
    D2D1_RECT_F title_rect = {CONTENT_PADDING, content_y, WIDTH - CONTENT_PADDING, content_y + 40.0f};

    ComPtr<IDWriteTextFormat> title_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI",
        nullptr,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        24.0f,
        L"vi-VN",
        &title_format
    );
    title_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);

    render_target_->DrawText(
        L"Chào mừng đến với GoNhanh",
        26,
        title_format.Get(),
        title_rect,
        title_brush.Get()
    );

    content_y += 50.0f;

    // Subtitle
    auto subtitle_brush = create_brush(render_target_.Get(), Colors::TextSecondary);
    D2D1_RECT_F subtitle_rect = {CONTENT_PADDING, content_y, WIDTH - CONTENT_PADDING, content_y + 30.0f};

    render_target_->DrawText(
        L"Bộ gõ tiếng Việt nhanh và nhẹ",
        30,
        renderer.text_format_body(),
        subtitle_rect,
        subtitle_brush.Get()
    );
}

void OnboardingWindow::render_setup_step() {
    auto& renderer = D2DRenderer::instance();
    float content_y = 60.0f;

    // Keyboard icon (60x60) - centered
    float icon_x = (WIDTH - 60.0f) / 2.0f;
    auto icon_brush = create_brush(render_target_.Get(), Colors::Primary);

    // Simple keyboard icon representation
    D2D1_ROUNDED_RECT key1 = {{icon_x, content_y, icon_x + 18.0f, content_y + 18.0f}, 3.0f, 3.0f};
    D2D1_ROUNDED_RECT key2 = {{icon_x + 21.0f, content_y, icon_x + 39.0f, content_y + 18.0f}, 3.0f, 3.0f};
    D2D1_ROUNDED_RECT key3 = {{icon_x + 42.0f, content_y, icon_x + 60.0f, content_y + 18.0f}, 3.0f, 3.0f};
    D2D1_ROUNDED_RECT key4 = {{icon_x + 5.0f, content_y + 21.0f, icon_x + 23.0f, content_y + 39.0f}, 3.0f, 3.0f};
    D2D1_ROUNDED_RECT key5 = {{icon_x + 26.0f, content_y + 21.0f, icon_x + 34.0f, content_y + 39.0f}, 3.0f, 3.0f};
    D2D1_ROUNDED_RECT key6 = {{icon_x + 37.0f, content_y + 21.0f, icon_x + 55.0f, content_y + 39.0f}, 3.0f, 3.0f};
    D2D1_ROUNDED_RECT key7 = {{icon_x + 10.0f, content_y + 42.0f, icon_x + 50.0f, content_y + 60.0f}, 3.0f, 3.0f};

    render_target_->FillRoundedRectangle(key1, icon_brush.Get());
    render_target_->FillRoundedRectangle(key2, icon_brush.Get());
    render_target_->FillRoundedRectangle(key3, icon_brush.Get());
    render_target_->FillRoundedRectangle(key4, icon_brush.Get());
    render_target_->FillRoundedRectangle(key5, icon_brush.Get());
    render_target_->FillRoundedRectangle(key6, icon_brush.Get());
    render_target_->FillRoundedRectangle(key7, icon_brush.Get());

    content_y += 80.0f;

    // Title
    auto title_brush = create_brush(render_target_.Get(), Colors::Text);
    D2D1_RECT_F title_rect = {CONTENT_PADDING, content_y, WIDTH - CONTENT_PADDING, content_y + 40.0f};

    ComPtr<IDWriteTextFormat> title_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI",
        nullptr,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        24.0f,
        L"vi-VN",
        &title_format
    );
    title_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);

    render_target_->DrawText(
        L"Chọn kiểu gõ",
        13,
        title_format.Get(),
        title_rect,
        title_brush.Get()
    );

    content_y += 40.0f;

    // Subtitle
    auto subtitle_brush = create_brush(render_target_.Get(), Colors::TextSecondary);
    D2D1_RECT_F subtitle_rect = {CONTENT_PADDING, content_y, WIDTH - CONTENT_PADDING, content_y + 30.0f};

    ComPtr<IDWriteTextFormat> subtitle_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI",
        nullptr,
        DWRITE_FONT_WEIGHT_NORMAL,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        14.0f,
        L"vi-VN",
        &subtitle_format
    );
    subtitle_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);

    render_target_->DrawText(
        L"Có thể thay đổi sau trong menu.",
        32,
        subtitle_format.Get(),
        subtitle_rect,
        subtitle_brush.Get()
    );

    content_y += 40.0f;

    // Options
    float option_x = CONTENT_PADDING;
    float option_width = WIDTH - 2.0f * CONTENT_PADDING;

    // Telex option
    bool telex_selected = (selected_method_ == 0);
    auto telex_bg_brush = create_brush(render_target_.Get(),
        telex_selected ? D2D1::ColorF(0.937f, 0.961f, 1.0f) : Colors::CardBg);  // Light blue tint
    auto telex_border_brush = create_brush(render_target_.Get(),
        telex_selected ? Colors::Primary : Colors::Border);

    D2D1_ROUNDED_RECT telex_rect = {
        {option_x, content_y, option_x + option_width, content_y + OPTION_HEIGHT},
        8.0f, 8.0f
    };

    render_target_->FillRoundedRectangle(telex_rect, telex_bg_brush.Get());
    render_target_->DrawRoundedRectangle(telex_rect, telex_border_brush.Get(),
        telex_selected ? 2.0f : 1.0f);

    // Telex text
    auto telex_title_brush = create_brush(render_target_.Get(), Colors::Text);
    D2D1_RECT_F telex_title_rect = {
        option_x + OPTION_PADDING,
        content_y + 12.0f,
        option_x + option_width - OPTION_PADDING,
        content_y + 35.0f
    };

    ComPtr<IDWriteTextFormat> option_title_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI",
        nullptr,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        16.0f,
        L"vi-VN",
        &option_title_format
    );

    render_target_->DrawText(
        L"Telex",
        5,
        option_title_format.Get(),
        telex_title_rect,
        telex_title_brush.Get()
    );

    // Telex description
    auto telex_desc_brush = create_brush(render_target_.Get(), Colors::TextSecondary);
    D2D1_RECT_F telex_desc_rect = {
        option_x + OPTION_PADDING,
        content_y + 35.0f,
        option_x + option_width - OPTION_PADDING,
        content_y + OPTION_HEIGHT - 12.0f
    };

    ComPtr<IDWriteTextFormat> option_desc_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI",
        nullptr,
        DWRITE_FONT_WEIGHT_NORMAL,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        13.0f,
        L"vi-VN",
        &option_desc_format
    );

    render_target_->DrawText(
        L"Gõ như nói (dd=đ, aa=â, s=sắc)",
        31,
        option_desc_format.Get(),
        telex_desc_rect,
        telex_desc_brush.Get()
    );

    content_y += OPTION_HEIGHT + 12.0f;

    // VNI option
    bool vni_selected = (selected_method_ == 1);
    auto vni_bg_brush = create_brush(render_target_.Get(),
        vni_selected ? D2D1::ColorF(0.937f, 0.961f, 1.0f) : Colors::CardBg);
    auto vni_border_brush = create_brush(render_target_.Get(),
        vni_selected ? Colors::Primary : Colors::Border);

    D2D1_ROUNDED_RECT vni_rect = {
        {option_x, content_y, option_x + option_width, content_y + OPTION_HEIGHT},
        8.0f, 8.0f
    };

    render_target_->FillRoundedRectangle(vni_rect, vni_bg_brush.Get());
    render_target_->DrawRoundedRectangle(vni_rect, vni_border_brush.Get(),
        vni_selected ? 2.0f : 1.0f);

    // VNI text
    auto vni_title_brush = create_brush(render_target_.Get(), Colors::Text);
    D2D1_RECT_F vni_title_rect = {
        option_x + OPTION_PADDING,
        content_y + 12.0f,
        option_x + option_width - OPTION_PADDING,
        content_y + 35.0f
    };

    render_target_->DrawText(
        L"VNI",
        3,
        option_title_format.Get(),
        vni_title_rect,
        vni_title_brush.Get()
    );

    // VNI description
    auto vni_desc_brush = create_brush(render_target_.Get(), Colors::TextSecondary);
    D2D1_RECT_F vni_desc_rect = {
        option_x + OPTION_PADDING,
        content_y + 35.0f,
        option_x + option_width - OPTION_PADDING,
        content_y + OPTION_HEIGHT - 12.0f
    };

    render_target_->DrawText(
        L"Gõ bằng số (d9=đ, a6=â, 1=sắc)",
        31,
        option_desc_format.Get(),
        vni_desc_rect,
        vni_desc_brush.Get()
    );
}

void OnboardingWindow::render_footer() {
    float footer_y = HEIGHT - FOOTER_HEIGHT;

    // Separator line
    auto border_brush = create_brush(render_target_.Get(), Colors::Border);
    render_target_->DrawLine(
        D2D1::Point2F(0, footer_y),
        D2D1::Point2F(static_cast<float>(WIDTH), footer_y),
        border_brush.Get(),
        1.0f
    );

    // Step indicators (centered)
    float indicator_spacing = 16.0f;
    float indicators_width = (2 * STEP_INDICATOR_SIZE) + indicator_spacing;
    float indicator_x = (WIDTH - indicators_width) / 2.0f;
    float indicator_y = footer_y + (FOOTER_HEIGHT - STEP_INDICATOR_SIZE) / 2.0f;

    auto active_indicator_brush = create_brush(render_target_.Get(), Colors::Primary);
    auto inactive_indicator_brush = create_brush(render_target_.Get(), Colors::Border);

    // Step 1 indicator
    D2D1_ELLIPSE step1_ellipse = {
        {indicator_x + STEP_INDICATOR_SIZE / 2.0f, indicator_y + STEP_INDICATOR_SIZE / 2.0f},
        STEP_INDICATOR_SIZE / 2.0f,
        STEP_INDICATOR_SIZE / 2.0f
    };
    render_target_->FillEllipse(
        step1_ellipse,
        (current_step_ == OnboardingStep::Welcome) ? active_indicator_brush.Get() : inactive_indicator_brush.Get()
    );

    // Step 2 indicator
    D2D1_ELLIPSE step2_ellipse = {
        {indicator_x + STEP_INDICATOR_SIZE + indicator_spacing + STEP_INDICATOR_SIZE / 2.0f,
         indicator_y + STEP_INDICATOR_SIZE / 2.0f},
        STEP_INDICATOR_SIZE / 2.0f,
        STEP_INDICATOR_SIZE / 2.0f
    };
    render_target_->FillEllipse(
        step2_ellipse,
        (current_step_ == OnboardingStep::Setup) ? active_indicator_brush.Get() : inactive_indicator_brush.Get()
    );

    // Back button (step 2 only)
    if (current_step_ == OnboardingStep::Setup) {
        float back_x = 24.0f;
        float back_y = footer_y + (FOOTER_HEIGHT - BUTTON_HEIGHT) / 2.0f;

        Button::draw(
            render_target_.Get(),
            back_x, back_y,
            100.0f, BUTTON_HEIGHT,
            L"Quay lại",
            ButtonStyle::Secondary,
            hover_back_button_,
            false
        );
    }

    // Primary action button (right aligned)
    float primary_x = WIDTH - BUTTON_WIDTH - 24.0f;
    float primary_y = footer_y + (FOOTER_HEIGHT - BUTTON_HEIGHT) / 2.0f;

    const wchar_t* button_text = (current_step_ == OnboardingStep::Welcome) ? L"Tiếp tục" : L"Hoàn tất";

    Button::draw(
        render_target_.Get(),
        primary_x, primary_y,
        BUTTON_WIDTH, BUTTON_HEIGHT,
        button_text,
        ButtonStyle::Primary,
        hover_primary_button_,
        false
    );
}

void OnboardingWindow::handle_mouse_move(int x, int y) {
    mouse_x_ = x;
    mouse_y_ = y;

    float footer_y = HEIGHT - FOOTER_HEIGHT;

    // Primary button
    float primary_x = WIDTH - BUTTON_WIDTH - 24.0f;
    float primary_y = footer_y + (FOOTER_HEIGHT - BUTTON_HEIGHT) / 2.0f;
    hover_primary_button_ = Button::hit_test(
        primary_x, primary_y, BUTTON_WIDTH, BUTTON_HEIGHT,
        static_cast<float>(x), static_cast<float>(y)
    );

    // Back button (step 2 only)
    if (current_step_ == OnboardingStep::Setup) {
        float back_x = 24.0f;
        float back_y = footer_y + (FOOTER_HEIGHT - BUTTON_HEIGHT) / 2.0f;
        hover_back_button_ = Button::hit_test(
            back_x, back_y, 100.0f, BUTTON_HEIGHT,
            static_cast<float>(x), static_cast<float>(y)
        );

        // Options hover
        float content_y = 220.0f;  // Matches setup step layout
        float option_x = CONTENT_PADDING;
        float option_width = WIDTH - 2.0f * CONTENT_PADDING;

        hover_telex_option_ = (x >= option_x && x <= option_x + option_width &&
                               y >= content_y && y <= content_y + OPTION_HEIGHT);

        content_y += OPTION_HEIGHT + 12.0f;
        hover_vni_option_ = (x >= option_x && x <= option_x + option_width &&
                             y >= content_y && y <= content_y + OPTION_HEIGHT);
    } else {
        hover_back_button_ = false;
        hover_telex_option_ = false;
        hover_vni_option_ = false;
    }

    InvalidateRect(hwnd_, nullptr, FALSE);
}

void OnboardingWindow::handle_mouse_down(int x, int y) {
    // Handle option selection in setup step
    if (current_step_ == OnboardingStep::Setup) {
        if (hover_telex_option_) {
            selected_method_ = 0;
            InvalidateRect(hwnd_, nullptr, FALSE);
        } else if (hover_vni_option_) {
            selected_method_ = 1;
            InvalidateRect(hwnd_, nullptr, FALSE);
        }
    }
}

void OnboardingWindow::handle_mouse_up(int x, int y) {
    if (hover_primary_button_) {
        if (current_step_ == OnboardingStep::Welcome) {
            next_step();
        } else {
            complete_onboarding();
        }
    } else if (hover_back_button_ && current_step_ == OnboardingStep::Setup) {
        prev_step();
    }
}

void OnboardingWindow::next_step() {
    if (current_step_ == OnboardingStep::Welcome) {
        current_step_ = OnboardingStep::Setup;
        InvalidateRect(hwnd_, nullptr, FALSE);
    }
}

void OnboardingWindow::prev_step() {
    if (current_step_ == OnboardingStep::Setup) {
        current_step_ = OnboardingStep::Welcome;
        InvalidateRect(hwnd_, nullptr, FALSE);
    }
}

void OnboardingWindow::complete_onboarding() {
    auto& settings = gonhanh::Settings::instance();

    // Save selected input method
    settings.set_input_method(selected_method_);

    // Mark onboarding as completed
    settings.set_onboarding_completed(true);

    // Hide onboarding window
    hide();
}

LRESULT CALLBACK OnboardingWindow::wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    OnboardingWindow* window = nullptr;

    if (msg == WM_CREATE) {
        auto cs = reinterpret_cast<CREATESTRUCT*>(lparam);
        window = static_cast<OnboardingWindow*>(cs->lpCreateParams);
        SetWindowLongPtr(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(window));
    } else {
        window = reinterpret_cast<OnboardingWindow*>(GetWindowLongPtr(hwnd, GWLP_USERDATA));
    }

    if (!window) {
        return DefWindowProc(hwnd, msg, wparam, lparam);
    }

    switch (msg) {
        case WM_PAINT: {
            PAINTSTRUCT ps;
            BeginPaint(hwnd, &ps);
            window->render();
            EndPaint(hwnd, &ps);
            return 0;
        }

        case WM_MOUSEMOVE: {
            int x = GET_X_LPARAM(lparam);
            int y = GET_Y_LPARAM(lparam);
            window->handle_mouse_move(x, y);
            return 0;
        }

        case WM_LBUTTONDOWN: {
            int x = GET_X_LPARAM(lparam);
            int y = GET_Y_LPARAM(lparam);
            window->handle_mouse_down(x, y);
            return 0;
        }

        case WM_LBUTTONUP: {
            int x = GET_X_LPARAM(lparam);
            int y = GET_Y_LPARAM(lparam);
            window->handle_mouse_up(x, y);
            return 0;
        }

        case WM_CLOSE:
            window->hide();
            return 0;

        case WM_DESTROY:
            window->hwnd_ = nullptr;
            return 0;
    }

    return DefWindowProc(hwnd, msg, wparam, lparam);
}

} // namespace gonhanh::ui
