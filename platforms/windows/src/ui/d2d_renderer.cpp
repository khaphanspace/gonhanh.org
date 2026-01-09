#include "d2d_renderer.h"

#pragma comment(lib, "d2d1.lib")
#pragma comment(lib, "dwrite.lib")

namespace gonhanh::ui {

D2DRenderer& D2DRenderer::instance() {
    static D2DRenderer instance;
    return instance;
}

bool D2DRenderer::initialize() {
    if (d2d_factory_) return true;

    // Create D2D factory
    HRESULT hr = D2D1CreateFactory(
        D2D1_FACTORY_TYPE_SINGLE_THREADED,
        d2d_factory_.GetAddressOf()
    );
    if (FAILED(hr)) return false;

    // Create DirectWrite factory
    hr = DWriteCreateFactory(
        DWRITE_FACTORY_TYPE_SHARED,
        __uuidof(IDWriteFactory),
        reinterpret_cast<IUnknown**>(dwrite_factory_.GetAddressOf())
    );
    if (FAILED(hr)) return false;

    // Create text formats
    if (!create_text_formats()) return false;

    return true;
}

void D2DRenderer::shutdown() {
    text_format_small_.Reset();
    text_format_body_.Reset();
    text_format_title_.Reset();
    dwrite_factory_.Reset();
    d2d_factory_.Reset();
}

ID2D1HwndRenderTarget* D2DRenderer::create_render_target(HWND hwnd) {
    if (!d2d_factory_) return nullptr;

    RECT rc;
    GetClientRect(hwnd, &rc);

    D2D1_SIZE_U size = D2D1::SizeU(rc.right - rc.left, rc.bottom - rc.top);

    D2D1_RENDER_TARGET_PROPERTIES rt_props = D2D1::RenderTargetProperties();
    D2D1_HWND_RENDER_TARGET_PROPERTIES hwnd_props = D2D1::HwndRenderTargetProperties(hwnd, size);

    ID2D1HwndRenderTarget* render_target = nullptr;
    HRESULT hr = d2d_factory_->CreateHwndRenderTarget(rt_props, hwnd_props, &render_target);

    return SUCCEEDED(hr) ? render_target : nullptr;
}

bool D2DRenderer::create_text_formats() {
    if (!dwrite_factory_) return false;

    const wchar_t* font_family = L"Segoe UI";

    // Title: 20px bold
    HRESULT hr = dwrite_factory_->CreateTextFormat(
        font_family,
        nullptr,
        DWRITE_FONT_WEIGHT_BOLD,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        20.0f,
        L"en-US",
        text_format_title_.GetAddressOf()
    );
    if (FAILED(hr)) return false;

    // Body: 13px regular
    hr = dwrite_factory_->CreateTextFormat(
        font_family,
        nullptr,
        DWRITE_FONT_WEIGHT_REGULAR,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        13.0f,
        L"en-US",
        text_format_body_.GetAddressOf()
    );
    if (FAILED(hr)) return false;

    // Small: 11px regular
    hr = dwrite_factory_->CreateTextFormat(
        font_family,
        nullptr,
        DWRITE_FONT_WEIGHT_REGULAR,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        11.0f,
        L"en-US",
        text_format_small_.GetAddressOf()
    );
    if (FAILED(hr)) return false;

    return true;
}

} // namespace gonhanh::ui
