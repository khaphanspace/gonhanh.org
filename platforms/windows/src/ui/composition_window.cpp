#include "composition_window.h"
#include "../logger.h"
#include <dwmapi.h>
#include <algorithm>

#pragma comment(lib, "dwmapi.lib")

namespace gonhanh {

CompositionWindow::CompositionWindow() {}

CompositionWindow::~CompositionWindow() {
    destroy();
}

bool CompositionWindow::create(HINSTANCE instance) {
    instance_ = instance;

    // Register window class
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(wc);
    wc.lpfnWndProc = window_proc;
    wc.hInstance = instance;
    wc.lpszClassName = L"GoNhanhComposition";
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.style = CS_HREDRAW | CS_VREDRAW;

    if (!RegisterClassExW(&wc)) {
        DWORD err = GetLastError();
        if (err != ERROR_CLASS_ALREADY_EXISTS) {
            Logger::error("Failed to register composition window class: %lu", err);
            return false;
        }
    }

    // Create window (WS_EX_TOOLWINDOW: no taskbar, WS_EX_TOPMOST: always on top)
    hwnd_ = CreateWindowExW(
        WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_LAYERED,
        L"GoNhanhComposition",
        L"",
        WS_POPUP,
        0, 0, MIN_WIDTH, HEIGHT,
        nullptr, nullptr, instance, this
    );

    if (!hwnd_) {
        Logger::error("Failed to create composition window: %lu", GetLastError());
        return false;
    }

    // Make window slightly transparent for polish
    SetLayeredWindowAttributes(hwnd_, 0, 250, LWA_ALPHA);

    // Enable shadow (Windows 10+ style)
    BOOL enable = TRUE;
    DwmSetWindowAttribute(hwnd_, DWMWA_USE_IMMERSIVE_DARK_MODE, &enable, sizeof(enable));

    // Create Direct2D factory
    HRESULT hr = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, &d2d_factory_);
    if (FAILED(hr)) {
        Logger::error("Failed to create D2D factory: 0x%08X", hr);
        return false;
    }

    // Create DirectWrite factory
    hr = DWriteCreateFactory(
        DWRITE_FACTORY_TYPE_SHARED,
        __uuidof(IDWriteFactory),
        reinterpret_cast<IUnknown**>(&dwrite_factory_)
    );
    if (FAILED(hr)) {
        Logger::error("Failed to create DWrite factory: 0x%08X", hr);
        return false;
    }

    // Create text format
    hr = dwrite_factory_->CreateTextFormat(
        L"Segoe UI",
        nullptr,
        DWRITE_FONT_WEIGHT_NORMAL,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        14.0f,
        L"vi-VN",
        &text_format_
    );
    if (FAILED(hr)) {
        Logger::error("Failed to create text format: 0x%08X", hr);
        return false;
    }

    text_format_->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
    text_format_->SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

    create_render_target();

    Logger::info("Composition window created");
    return true;
}

void CompositionWindow::destroy() {
    release_render_resources();

    if (text_format_) { text_format_->Release(); text_format_ = nullptr; }
    if (d2d_factory_) { d2d_factory_->Release(); d2d_factory_ = nullptr; }
    if (dwrite_factory_) { dwrite_factory_->Release(); dwrite_factory_ = nullptr; }

    if (hwnd_) {
        DestroyWindow(hwnd_);
        hwnd_ = nullptr;
    }
}

void CompositionWindow::release_render_resources() {
    if (render_target_) { render_target_->Release(); render_target_ = nullptr; }
    if (bg_brush_) { bg_brush_->Release(); bg_brush_ = nullptr; }
    if (text_brush_) { text_brush_->Release(); text_brush_ = nullptr; }
    if (border_brush_) { border_brush_->Release(); border_brush_ = nullptr; }
}

void CompositionWindow::create_render_target() {
    if (!d2d_factory_ || !hwnd_) return;

    release_render_resources();

    RECT rc;
    GetClientRect(hwnd_, &rc);

    D2D1_RENDER_TARGET_PROPERTIES rtProps = D2D1::RenderTargetProperties();
    D2D1_HWND_RENDER_TARGET_PROPERTIES hwndProps = D2D1::HwndRenderTargetProperties(
        hwnd_, D2D1::SizeU(rc.right - rc.left, rc.bottom - rc.top)
    );

    HRESULT hr = d2d_factory_->CreateHwndRenderTarget(rtProps, hwndProps, &render_target_);
    if (FAILED(hr)) {
        Logger::error("Failed to create render target: 0x%08X", hr);
        return;
    }

    // Create brushes
    render_target_->CreateSolidColorBrush(D2D1::ColorF(0xFFFFFF), &bg_brush_);
    render_target_->CreateSolidColorBrush(D2D1::ColorF(0x1F2937), &text_brush_);
    render_target_->CreateSolidColorBrush(D2D1::ColorF(0xE5E7EB), &border_brush_);
}

void CompositionWindow::show(const std::wstring& buffer, int caret_x, int caret_y) {
    if (buffer.empty()) {
        hide();
        return;
    }

    buffer_ = buffer;

    // Calculate width based on text
    auto size = measure_text(buffer);
    int width = static_cast<int>(size.width) + static_cast<int>(PADDING_H * 2);
    width = (std::max)(MIN_WIDTH, (std::min)(width, MAX_WIDTH));

    SetWindowPos(hwnd_, nullptr, 0, 0, width, HEIGHT,
        SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);

    create_render_target();
    position_window(caret_x, caret_y);

    ShowWindow(hwnd_, SW_SHOWNOACTIVATE);
    visible_ = true;

    render();
}

void CompositionWindow::hide() {
    if (!visible_) return;

    ShowWindow(hwnd_, SW_HIDE);
    visible_ = false;
    buffer_.clear();
}

void CompositionWindow::update_text(const std::wstring& buffer) {
    if (buffer.empty()) {
        hide();
        return;
    }

    buffer_ = buffer;

    // Resize window based on text
    auto size = measure_text(buffer);
    int width = static_cast<int>(size.width) + static_cast<int>(PADDING_H * 2);
    width = (std::max)(MIN_WIDTH, (std::min)(width, MAX_WIDTH));

    RECT rc;
    GetWindowRect(hwnd_, &rc);
    int current_width = rc.right - rc.left;

    // Only resize if width changed significantly
    if (abs(width - current_width) > 10) {
        SetWindowPos(hwnd_, nullptr, 0, 0, width, HEIGHT,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);
        create_render_target();
    }

    render();
}

void CompositionWindow::update_position(int caret_x, int caret_y) {
    position_window(caret_x, caret_y);
}

D2D1_SIZE_F CompositionWindow::measure_text(const std::wstring& text) {
    if (!dwrite_factory_ || !text_format_ || text.empty()) {
        return {0, 0};
    }

    IDWriteTextLayout* layout = nullptr;
    HRESULT hr = dwrite_factory_->CreateTextLayout(
        text.c_str(),
        static_cast<UINT32>(text.length()),
        text_format_,
        static_cast<float>(MAX_WIDTH),
        static_cast<float>(HEIGHT),
        &layout
    );

    if (FAILED(hr) || !layout) {
        return {0, 0};
    }

    DWRITE_TEXT_METRICS metrics;
    layout->GetMetrics(&metrics);
    layout->Release();

    return {metrics.width, metrics.height};
}

void CompositionWindow::position_window(int caret_x, int caret_y) {
    if (!hwnd_) return;

    RECT rc;
    GetWindowRect(hwnd_, &rc);
    int width = rc.right - rc.left;

    // Position below caret
    int x = caret_x;
    int y = caret_y + OFFSET_Y;

    // Keep on screen
    HMONITOR monitor = MonitorFromPoint({caret_x, caret_y}, MONITOR_DEFAULTTONEAREST);
    MONITORINFO mi = {sizeof(mi)};
    GetMonitorInfo(monitor, &mi);

    if (x + width > mi.rcWork.right) {
        x = mi.rcWork.right - width;
    }
    if (x < mi.rcWork.left) {
        x = mi.rcWork.left;
    }
    if (y + HEIGHT > mi.rcWork.bottom) {
        // Show above caret instead
        y = caret_y - HEIGHT - 5;
    }
    if (y < mi.rcWork.top) {
        y = mi.rcWork.top;
    }

    SetWindowPos(hwnd_, nullptr, x, y, 0, 0,
        SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
}

void CompositionWindow::render() {
    if (!render_target_) return;

    render_target_->BeginDraw();

    auto size = render_target_->GetSize();
    D2D1_RECT_F rect = {0, 0, size.width, size.height};

    // Background with rounded corners
    D2D1_ROUNDED_RECT rrect = {rect, 4.0f, 4.0f};
    render_target_->FillRoundedRectangle(rrect, bg_brush_);
    render_target_->DrawRoundedRectangle(rrect, border_brush_, 1.0f);

    // Text
    D2D1_RECT_F textRect = {
        PADDING_H, PADDING_V,
        size.width - PADDING_H, size.height - PADDING_V
    };
    render_target_->DrawText(
        buffer_.c_str(),
        static_cast<UINT32>(buffer_.length()),
        text_format_,
        textRect,
        text_brush_
    );

    render_target_->EndDraw();
}

LRESULT CALLBACK CompositionWindow::window_proc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    CompositionWindow* self = nullptr;

    if (msg == WM_NCCREATE) {
        auto cs = reinterpret_cast<CREATESTRUCT*>(lp);
        self = static_cast<CompositionWindow*>(cs->lpCreateParams);
        SetWindowLongPtr(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
    } else {
        self = reinterpret_cast<CompositionWindow*>(GetWindowLongPtr(hwnd, GWLP_USERDATA));
    }

    if (self) {
        return self->handle_message(msg, wp, lp);
    }

    return DefWindowProc(hwnd, msg, wp, lp);
}

LRESULT CompositionWindow::handle_message(UINT msg, WPARAM wp, LPARAM lp) {
    switch (msg) {
        case WM_PAINT: {
            PAINTSTRUCT ps;
            BeginPaint(hwnd_, &ps);
            render();
            EndPaint(hwnd_, &ps);
            return 0;
        }

        case WM_SIZE:
            if (render_target_) {
                render_target_->Resize(D2D1::SizeU(LOWORD(lp), HIWORD(lp)));
            }
            return 0;

        case WM_ERASEBKGND:
            // Prevent flickering
            return 1;
    }

    return DefWindowProc(hwnd_, msg, wp, lp);
}

} // namespace gonhanh
