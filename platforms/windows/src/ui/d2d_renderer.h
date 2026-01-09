#pragma once

#include <windows.h>
#include <d2d1.h>
#include <dwrite.h>
#include <wrl/client.h>

using Microsoft::WRL::ComPtr;

namespace gonhanh::ui {

// Color palette matching macOS design
namespace Colors {
    constexpr D2D1_COLOR_F Primary = {0.145f, 0.388f, 0.922f, 1.0f};      // #2563EB
    constexpr D2D1_COLOR_F Text = {0.122f, 0.161f, 0.216f, 1.0f};         // #1F2937
    constexpr D2D1_COLOR_F TextSecondary = {0.420f, 0.451f, 0.502f, 1.0f}; // #6B7280
    constexpr D2D1_COLOR_F TextTertiary = {0.612f, 0.639f, 0.686f, 1.0f};  // #9CA3AF
    constexpr D2D1_COLOR_F Border = {0.898f, 0.906f, 0.922f, 1.0f};        // #E5E7EB
    constexpr D2D1_COLOR_F Background = {0.980f, 0.980f, 0.980f, 1.0f};    // #FAFAFA
    constexpr D2D1_COLOR_F CardBg = {1.0f, 1.0f, 1.0f, 1.0f};             // White
    constexpr D2D1_COLOR_F ToggleOn = Primary;
    constexpr D2D1_COLOR_F ToggleOff = {0.898f, 0.906f, 0.922f, 1.0f};    // #E5E7EB
    constexpr D2D1_COLOR_F Orange = {0.976f, 0.451f, 0.086f, 1.0f};       // #F97316 (Beta badge)
    constexpr D2D1_COLOR_F Green = {0.133f, 0.773f, 0.369f, 1.0f};        // #22C55E (Success)
}

// Direct2D renderer singleton
class D2DRenderer {
public:
    static D2DRenderer& instance();

    // Initialize/shutdown
    bool initialize();
    void shutdown();
    bool is_initialized() const { return d2d_factory_ != nullptr; }

    // Create render target for a window
    ID2D1HwndRenderTarget* create_render_target(HWND hwnd);

    // Factory access
    ID2D1Factory* d2d_factory() const { return d2d_factory_.Get(); }
    IDWriteFactory* dwrite_factory() const { return dwrite_factory_.Get(); }

    // Text formats (pre-created for common sizes)
    IDWriteTextFormat* text_format_title() const { return text_format_title_.Get(); }
    IDWriteTextFormat* text_format_body() const { return text_format_body_.Get(); }
    IDWriteTextFormat* text_format_small() const { return text_format_small_.Get(); }

private:
    D2DRenderer() = default;
    ~D2DRenderer() { shutdown(); }
    D2DRenderer(const D2DRenderer&) = delete;
    D2DRenderer& operator=(const D2DRenderer&) = delete;

    bool create_text_formats();

    ComPtr<ID2D1Factory> d2d_factory_;
    ComPtr<IDWriteFactory> dwrite_factory_;
    ComPtr<IDWriteTextFormat> text_format_title_;
    ComPtr<IDWriteTextFormat> text_format_body_;
    ComPtr<IDWriteTextFormat> text_format_small_;
};

// Helper to create solid color brush
inline ComPtr<ID2D1SolidColorBrush> create_brush(ID2D1RenderTarget* rt, D2D1_COLOR_F color) {
    ComPtr<ID2D1SolidColorBrush> brush;
    rt->CreateSolidColorBrush(color, &brush);
    return brush;
}

// Get DPI scale factor for the system (Windows 11 style scaling)
// Returns scale factor (1.0 = 96 DPI, 1.25 = 120 DPI, 1.5 = 144 DPI, 2.0 = 192 DPI)
inline float get_dpi_scale() {
    HDC hdc = GetDC(nullptr);
    int dpi = GetDeviceCaps(hdc, LOGPIXELSX);
    ReleaseDC(nullptr, hdc);
    return static_cast<float>(dpi) / 96.0f;
}

// Get DPI scale factor for a specific window (per-monitor aware)
inline float get_dpi_scale(HWND hwnd) {
    // Use GetDpiForWindow for per-monitor DPI (Windows 10 1607+)
    typedef UINT (WINAPI *GetDpiForWindowFn)(HWND);
    static auto fn = reinterpret_cast<GetDpiForWindowFn>(
        GetProcAddress(GetModuleHandleW(L"user32.dll"), "GetDpiForWindow"));
    if (fn && hwnd) {
        UINT dpi = fn(hwnd);
        if (dpi > 0) return static_cast<float>(dpi) / 96.0f;
    }
    return get_dpi_scale();
}

// Scale a value by DPI factor
inline int scale_by_dpi(int value, float dpi_scale) {
    return static_cast<int>(value * dpi_scale + 0.5f);
}

// Helper to ensure window has exact client area size (scaled for DPI)
// target_width/height are logical sizes, will be scaled by DPI internally
inline void ensure_client_area(HWND hwnd, int target_width, int target_height) {
    float scale = get_dpi_scale(hwnd);
    int scaled_width = scale_by_dpi(target_width, scale);
    int scaled_height = scale_by_dpi(target_height, scale);

    RECT client_rc;
    GetClientRect(hwnd, &client_rc);
    if (client_rc.right != scaled_width || client_rc.bottom != scaled_height) {
        RECT window_rc;
        GetWindowRect(hwnd, &window_rc);
        int extra_width = scaled_width - client_rc.right;
        int extra_height = scaled_height - client_rc.bottom;
        SetWindowPos(hwnd, nullptr, 0, 0,
                     (window_rc.right - window_rc.left) + extra_width,
                     (window_rc.bottom - window_rc.top) + extra_height,
                     SWP_NOMOVE | SWP_NOZORDER);
    }
}

} // namespace gonhanh::ui
