#include "about_window.h"
#include "d2d_renderer.h"
#include "../app.h"
#include <shellapi.h>
#include <wincodec.h>

#pragma comment(lib, "windowscodecs.lib")

namespace gonhanh::ui {

static constexpr const wchar_t* ABOUT_WINDOW_CLASS = L"GoNhanhAboutClass";

// App metadata
static constexpr const wchar_t* APP_NAME = L"Gõ Nhanh";
static constexpr const wchar_t* APP_TAGLINE = L"Bộ gõ tiếng Việt nhanh và nhẹ";
static constexpr const wchar_t* APP_VERSION = L"0.1.0";
static constexpr const wchar_t* APP_AUTHOR = L"Kha Phan";
static constexpr const wchar_t* APP_FOOTER = L"Từ Việt Nam với ❤️";

// URLs
static constexpr const wchar_t* URL_GITHUB = L"https://github.com/nicepkg/gonhanh";
static constexpr const wchar_t* URL_ISSUES = L"https://github.com/nicepkg/gonhanh/issues";
static constexpr const wchar_t* URL_SPONSOR = L"https://github.com/sponsors/nicepkg";
static constexpr const wchar_t* URL_LINKEDIN = L"https://linkedin.com/in/khaphan";

AboutWindow& AboutWindow::instance() {
    static AboutWindow instance;
    return instance;
}

AboutWindow::~AboutWindow() {
    if (hwnd_) {
        DestroyWindow(hwnd_);
        hwnd_ = nullptr;
    }
}

void AboutWindow::show() {
    if (!hwnd_) {
        if (!create_window()) return;
    }

    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
}

void AboutWindow::hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
    }
}

bool AboutWindow::is_visible() const {
    return hwnd_ && IsWindowVisible(hwnd_);
}

bool AboutWindow::create_window() {
    auto& app = gonhanh::App::instance();

    // Register window class
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(wc);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = wnd_proc;
    wc.hInstance = app.hinstance();
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.lpszClassName = ABOUT_WINDOW_CLASS;

    if (!GetClassInfoExW(app.hinstance(), ABOUT_WINDOW_CLASS, &wc)) {
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
        ABOUT_WINDOW_CLASS,
        L"Về Gõ Nhanh",
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

    // Load logo from ICO file
    ComPtr<IWICImagingFactory> wic_factory;
    HRESULT hr = CoCreateInstance(
        CLSID_WICImagingFactory,
        nullptr,
        CLSCTX_INPROC_SERVER,
        IID_PPV_ARGS(&wic_factory)
    );

    if (SUCCEEDED(hr)) {
        // Try to load app.ico from resources directory
        wchar_t exe_path[MAX_PATH];
        GetModuleFileNameW(nullptr, exe_path, MAX_PATH);
        std::wstring icon_path = exe_path;
        size_t last_slash = icon_path.find_last_of(L"\\/");
        if (last_slash != std::wstring::npos) {
            icon_path = icon_path.substr(0, last_slash + 1) + L"..\\resources\\app.ico";
        }

        ComPtr<IWICBitmapDecoder> decoder;
        hr = wic_factory->CreateDecoderFromFilename(
            icon_path.c_str(),
            nullptr,
            GENERIC_READ,
            WICDecodeMetadataCacheOnDemand,
            &decoder
        );

        if (SUCCEEDED(hr)) {
            ComPtr<IWICBitmapFrameDecode> frame;
            hr = decoder->GetFrame(0, &frame);

            if (SUCCEEDED(hr)) {
                ComPtr<IWICFormatConverter> converter;
                hr = wic_factory->CreateFormatConverter(&converter);

                if (SUCCEEDED(hr)) {
                    hr = converter->Initialize(
                        frame.Get(),
                        GUID_WICPixelFormat32bppPBGRA,
                        WICBitmapDitherTypeNone,
                        nullptr,
                        0.0f,
                        WICBitmapPaletteTypeCustom
                    );

                    if (SUCCEEDED(hr)) {
                        render_target_->CreateBitmapFromWicBitmap(
                            converter.Get(),
                            nullptr,
                            &logo_bitmap_
                        );
                    }
                }
            }
        }
    }

    return true;
}

void AboutWindow::render() {
    if (!render_target_) return;

    render_target_->BeginDraw();
    render_target_->Clear(Colors::Background);

    auto& renderer = D2DRenderer::instance();
    const float center_x = WIDTH / 2.0f;
    float y_offset = 32.0f;

    // --- Header Section ---
    // Logo (80x80 centered)
    if (logo_bitmap_) {
        D2D1_SIZE_F logo_size = logo_bitmap_->GetSize();
        float logo_scale = 80.0f / logo_size.width;
        D2D1_RECT_F logo_rect = {
            center_x - 40.0f,
            y_offset,
            center_x + 40.0f,
            y_offset + 80.0f
        };
        render_target_->DrawBitmap(
            logo_bitmap_.Get(),
            logo_rect,
            1.0f,
            D2D1_BITMAP_INTERPOLATION_MODE_LINEAR
        );
    }
    y_offset += 92.0f;

    // App name (20px bold)
    auto title_brush = create_brush(render_target_.Get(), Colors::Text);
    ComPtr<IDWriteTextFormat> title_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI",
        nullptr,
        DWRITE_FONT_WEIGHT_BOLD,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        20.0f,
        L"vi-VN",
        &title_format
    );
    title_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);

    D2D1_RECT_F title_rect = {0, y_offset, static_cast<float>(WIDTH), y_offset + 30};
    render_target_->DrawText(APP_NAME, wcslen(APP_NAME), title_format.Get(), title_rect, title_brush.Get());
    y_offset += 36.0f;

    // Tagline (13px secondary)
    auto secondary_brush = create_brush(render_target_.Get(), Colors::TextSecondary);
    ComPtr<IDWriteTextFormat> body_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI",
        nullptr,
        DWRITE_FONT_WEIGHT_REGULAR,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        13.0f,
        L"vi-VN",
        &body_format
    );
    body_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);

    D2D1_RECT_F tagline_rect = {0, y_offset, static_cast<float>(WIDTH), y_offset + 20};
    render_target_->DrawText(APP_TAGLINE, wcslen(APP_TAGLINE), body_format.Get(), tagline_rect, secondary_brush.Get());
    y_offset += 28.0f;

    // Version (12px tertiary)
    auto tertiary_brush = create_brush(render_target_.Get(), Colors::TextTertiary);
    ComPtr<IDWriteTextFormat> small_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI",
        nullptr,
        DWRITE_FONT_WEIGHT_REGULAR,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        12.0f,
        L"vi-VN",
        &small_format
    );
    small_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);

    std::wstring version_text = L"Phiên bản " + std::wstring(APP_VERSION);
    D2D1_RECT_F version_rect = {0, y_offset, static_cast<float>(WIDTH), y_offset + 18};
    render_target_->DrawText(version_text.c_str(), version_text.length(), small_format.Get(), version_rect, tertiary_brush.Get());
    y_offset += 32.0f;

    // --- Divider ---
    auto border_brush = create_brush(render_target_.Get(), Colors::Border);
    render_target_->DrawLine(
        D2D1::Point2F(40.0f, y_offset),
        D2D1::Point2F(WIDTH - 40.0f, y_offset),
        border_brush.Get(),
        1.0f
    );
    y_offset += 24.0f;

    // --- Links Section (3 buttons) ---
    const float btn_width = 100.0f;
    const float btn_height = 36.0f;
    const float btn_spacing = 16.0f;
    const float total_width = (btn_width * 3) + (btn_spacing * 2);
    float btn_x = (WIDTH - total_width) / 2.0f;

    auto draw_link_button = [&](LinkButton& btn, const wchar_t* label, float x) {
        btn.bounds = D2D1::RectF(x, y_offset, x + btn_width, y_offset + btn_height);
        btn.label = label;

        // Background with hover effect
        auto bg_color = btn.hovered ?
            D2D1::ColorF(Colors::Primary.r, Colors::Primary.g, Colors::Primary.b, 0.1f) :
            D2D1::ColorF(0.0f, 0.0f, 0.0f, 0.03f);
        auto bg_brush = create_brush(render_target_.Get(), bg_color);

        D2D1_ROUNDED_RECT rounded_rect = {btn.bounds, 6.0f, 6.0f};
        render_target_->FillRoundedRectangle(rounded_rect, bg_brush.Get());

        // Border
        auto border_color = btn.hovered ? Colors::Primary : Colors::Border;
        auto btn_border_brush = create_brush(render_target_.Get(), border_color);
        render_target_->DrawRoundedRectangle(rounded_rect, btn_border_brush.Get(), 1.0f);

        // Label
        auto label_brush = create_brush(render_target_.Get(), btn.hovered ? Colors::Primary : Colors::Text);
        render_target_->DrawText(label, wcslen(label), body_format.Get(), btn.bounds, label_brush.Get());
    };

    github_btn_.url = URL_GITHUB;
    draw_link_button(github_btn_, L"GitHub", btn_x);

    issues_btn_.url = URL_ISSUES;
    draw_link_button(issues_btn_, L"Báo lỗi", btn_x + btn_width + btn_spacing);

    sponsor_btn_.url = URL_SPONSOR;
    draw_link_button(sponsor_btn_, L"Ủng hộ", btn_x + (btn_width + btn_spacing) * 2);

    y_offset += btn_height + 24.0f;

    // --- Divider ---
    render_target_->DrawLine(
        D2D1::Point2F(40.0f, y_offset),
        D2D1::Point2F(WIDTH - 40.0f, y_offset),
        border_brush.Get(),
        1.0f
    );
    y_offset += 24.0f;

    // --- Author Section ---
    std::wstring author_text = L"Phát triển bởi ";
    D2D1_RECT_F author_label_rect = {0, y_offset, center_x - 30.0f, y_offset + 20};
    body_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING);
    render_target_->DrawText(author_text.c_str(), author_text.length(), body_format.Get(), author_label_rect, secondary_brush.Get());

    // Author name (clickable LinkedIn link)
    linkedin_btn_.bounds = D2D1::RectF(center_x - 25.0f, y_offset, center_x + 45.0f, y_offset + 20);
    linkedin_btn_.url = URL_LINKEDIN;
    linkedin_btn_.label = APP_AUTHOR;

    auto author_brush = create_brush(render_target_.Get(), linkedin_btn_.hovered ? Colors::Primary : Colors::Text);
    body_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);

    ComPtr<IDWriteTextFormat> author_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI",
        nullptr,
        DWRITE_FONT_WEIGHT_SEMIBOLD,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        13.0f,
        L"vi-VN",
        &author_format
    );
    author_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);

    render_target_->DrawText(APP_AUTHOR, wcslen(APP_AUTHOR), author_format.Get(), linkedin_btn_.bounds, author_brush.Get());

    // Underline on hover
    if (linkedin_btn_.hovered) {
        auto underline_brush = create_brush(render_target_.Get(), Colors::Primary);
        render_target_->DrawLine(
            D2D1::Point2F(linkedin_btn_.bounds.left, linkedin_btn_.bounds.bottom - 2),
            D2D1::Point2F(linkedin_btn_.bounds.right, linkedin_btn_.bounds.bottom - 2),
            underline_brush.Get(),
            1.0f
        );
    }

    y_offset += 32.0f;

    // --- Divider ---
    render_target_->DrawLine(
        D2D1::Point2F(40.0f, y_offset),
        D2D1::Point2F(WIDTH - 40.0f, y_offset),
        border_brush.Get(),
        1.0f
    );
    y_offset += 20.0f;

    // --- Footer ---
    small_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
    D2D1_RECT_F footer_rect = {0, y_offset, static_cast<float>(WIDTH), y_offset + 18};
    render_target_->DrawText(APP_FOOTER, wcslen(APP_FOOTER), small_format.Get(), footer_rect, tertiary_brush.Get());

    render_target_->EndDraw();
}

void AboutWindow::handle_click(int x, int y) {
    D2D1_POINT_2F point = D2D1::Point2F(static_cast<float>(x), static_cast<float>(y));

    auto open_url = [](const std::wstring& url) {
        ShellExecuteW(nullptr, L"open", url.c_str(), nullptr, nullptr, SW_SHOWNORMAL);
    };

    if (point.x >= github_btn_.bounds.left && point.x <= github_btn_.bounds.right &&
        point.y >= github_btn_.bounds.top && point.y <= github_btn_.bounds.bottom) {
        open_url(github_btn_.url);
    }
    else if (point.x >= issues_btn_.bounds.left && point.x <= issues_btn_.bounds.right &&
             point.y >= issues_btn_.bounds.top && point.y <= issues_btn_.bounds.bottom) {
        open_url(issues_btn_.url);
    }
    else if (point.x >= sponsor_btn_.bounds.left && point.x <= sponsor_btn_.bounds.right &&
             point.y >= sponsor_btn_.bounds.top && point.y <= sponsor_btn_.bounds.bottom) {
        open_url(sponsor_btn_.url);
    }
    else if (point.x >= linkedin_btn_.bounds.left && point.x <= linkedin_btn_.bounds.right &&
             point.y >= linkedin_btn_.bounds.top && point.y <= linkedin_btn_.bounds.bottom) {
        open_url(linkedin_btn_.url);
    }
}

LRESULT CALLBACK AboutWindow::wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    AboutWindow* self = nullptr;

    if (msg == WM_NCCREATE) {
        auto* cs = reinterpret_cast<CREATESTRUCTW*>(lparam);
        self = static_cast<AboutWindow*>(cs->lpCreateParams);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
    } else {
        self = reinterpret_cast<AboutWindow*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
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

        case WM_LBUTTONDOWN: {
            if (self) {
                int x = LOWORD(lparam);
                int y = HIWORD(lparam);
                self->handle_click(x, y);
            }
            return 0;
        }

        case WM_MOUSEMOVE: {
            if (self) {
                POINT pt = {LOWORD(lparam), HIWORD(lparam)};

                // Track mouse leave
                TRACKMOUSEEVENT tme = {};
                tme.cbSize = sizeof(tme);
                tme.dwFlags = TME_LEAVE;
                tme.hwndTrack = hwnd;
                TrackMouseEvent(&tme);

                // Update hover states
                bool needs_redraw = false;
                D2D1_POINT_2F point = D2D1::Point2F(static_cast<float>(pt.x), static_cast<float>(pt.y));

                auto update_hover = [&](LinkButton& btn) {
                    bool was_hovered = btn.hovered;
                    btn.hovered = (point.x >= btn.bounds.left && point.x <= btn.bounds.right &&
                                   point.y >= btn.bounds.top && point.y <= btn.bounds.bottom);
                    if (was_hovered != btn.hovered) needs_redraw = true;
                };

                update_hover(self->github_btn_);
                update_hover(self->issues_btn_);
                update_hover(self->sponsor_btn_);
                update_hover(self->linkedin_btn_);

                // Update cursor
                bool any_hovered = self->github_btn_.hovered || self->issues_btn_.hovered ||
                                   self->sponsor_btn_.hovered || self->linkedin_btn_.hovered;
                SetCursor(LoadCursor(nullptr, any_hovered ? IDC_HAND : IDC_ARROW));

                if (needs_redraw) {
                    InvalidateRect(hwnd, nullptr, FALSE);
                }

                self->last_mouse_pos_ = pt;
            }
            return 0;
        }

        case WM_MOUSELEAVE: {
            if (self) {
                // Clear all hover states
                bool needs_redraw = false;
                if (self->github_btn_.hovered) { self->github_btn_.hovered = false; needs_redraw = true; }
                if (self->issues_btn_.hovered) { self->issues_btn_.hovered = false; needs_redraw = true; }
                if (self->sponsor_btn_.hovered) { self->sponsor_btn_.hovered = false; needs_redraw = true; }
                if (self->linkedin_btn_.hovered) { self->linkedin_btn_.hovered = false; needs_redraw = true; }

                if (needs_redraw) {
                    InvalidateRect(hwnd, nullptr, FALSE);
                }

                self->last_mouse_pos_ = {-1, -1};
            }
            return 0;
        }

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
