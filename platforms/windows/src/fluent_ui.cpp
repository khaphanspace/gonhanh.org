#include "fluent_ui.h"
#include "utils.h"
#include "resource.h"
#include <dwmapi.h>
#include <dwrite_2.h>
#include <uxtheme.h>
#include <wincodec.h>
#include <wrl/client.h>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.UI.ViewManagement.h>
#include <algorithm>
#include <cmath>
#include <mutex>
#include <string>
#include <vector>

#pragma comment(lib, "dwmapi.lib")
#pragma comment(lib, "uxtheme.lib")
#pragma comment(lib, "dwrite.lib")
#pragma comment(lib, "windowscodecs.lib")
#pragma comment(lib, "ole32.lib")
#pragma comment(lib, "windowsapp.lib")

namespace gonhanh {
namespace fluent {

using Microsoft::WRL::ComPtr;

// ===========================================================================
// Colors
// ===========================================================================
ARGB Lerp(ARGB a, ARGB b, float t) {
    t = std::clamp(t, 0.0f, 1.0f);
    ARGB out = 0;
    for (int shift = 0; shift < 32; shift += 8) {
        int ca = (a >> shift) & 0xFF;
        int cb = (b >> shift) & 0xFF;
        out |= static_cast<ARGB>(ca + (cb - ca) * t + 0.5f) << shift;
    }
    return out;
}

ARGB Over(ARGB top, ARGB bottom) {
    float a = ((top >> 24) & 0xFF) / 255.0f;
    return Lerp(bottom | 0xFF000000u, top | 0xFF000000u, a);
}

COLORREF ToColorRef(ARGB c) {
    return RGB((c >> 16) & 0xFF, (c >> 8) & 0xFF, c & 0xFF);
}

namespace {

ARGB FromWinRT(const winrt::Windows::UI::Color& c) {
    return (static_cast<ARGB>(c.A) << 24) | (static_cast<ARGB>(c.R) << 16) | (static_cast<ARGB>(c.G) << 8) | c.B;
}

// The accent shades Windows itself uses for controls: AccentLight2 on dark, AccentDark1 on light.
// UISettings is the same source as Windows Settings; the DWM registry value can disagree with it.
bool ReadSystemAccent(bool dark, ARGB& out) {
    using namespace winrt::Windows::UI::ViewManagement;
    try {
        UISettings settings;
        out = FromWinRT(settings.GetColorValue(dark ? UIColorType::AccentLight2 : UIColorType::AccentDark1));
        return true;
    } catch (...) {
    }

    // Fallback (UISettings unavailable): DWM accent, stored as 0xAABBGGRR
    DWORD value = 0;
    DWORD size = sizeof(value);
    if (RegGetValueW(HKEY_CURRENT_USER, L"Software\\Microsoft\\Windows\\DWM", L"AccentColor",
                     RRF_RT_REG_DWORD, nullptr, &value, &size) != ERROR_SUCCESS) {
        return false;
    }
    ARGB base = 0xFF000000u | ((value & 0xFF) << 16) | (value & 0xFF00) | ((value >> 16) & 0xFF);
    out = dark ? Lerp(base, 0xFFFFFFFF, 0.45f) : Lerp(base, 0xFF000000, 0.18f);
    return true;
}

// Theme change notifications: UISettings fires ColorValuesChanged on a worker thread when the
// accent (or light/dark mode) changes; forward it to every registered window's UI thread.
std::mutex g_themeMutex;
std::vector<HWND> g_themeWindows;
// Intentionally leaked: must outlive every window and must not be released after COM shuts down
struct UISettingsHolder {
    winrt::Windows::UI::ViewManagement::UISettings settings;
};
UISettingsHolder* g_uiSettings = nullptr;

void EnsureThemeListener() {
    if (g_uiSettings) return;
    try {
        g_uiSettings = new UISettingsHolder{};
        g_uiSettings->settings.ColorValuesChanged([](auto&&, auto&&) {
            std::lock_guard<std::mutex> lock(g_themeMutex);
            for (HWND hwnd : g_themeWindows) {
                PostMessageW(hwnd, WM_FLUENT_THEME_CHANGED, 0, 0);
            }
        });
    } catch (...) {
        delete g_uiSettings;
        g_uiSettings = nullptr;
    }
}

// Windows 11 Fluent tokens (straight-alpha ARGB). Translucent fills pick up the
// Mica wallpaper tint exactly like Windows Settings.
Palette MakePalette() {
    Palette p{};
    p.dark = IsDarkMode();
    ARGB accent = 0;
    bool hasAccent = ReadSystemAccent(p.dark, accent);
    p.heart = 0xFFEC4899;

    if (p.dark) {
        p.base = 0xFF202020;
        p.navHover = 0x0FFFFFFF;
        p.navSelected = 0x0FFFFFFF;
        p.navPressed = 0x0AFFFFFF;
        p.card = 0x0DFFFFFF;
        p.cardHover = 0x15FFFFFF;
        p.cardPressed = 0x08FFFFFF;
        p.cardStroke = 0x19000000;
        p.divider = 0x33000000;
        p.text = 0xFFFFFFFF;
        p.textSecondary = 0xC5FFFFFF;
        p.textTertiary = 0x87FFFFFF;
        p.toggleOffFill = 0x19000000;
        p.toggleStroke = 0x8BFFFFFF;
        p.toggleKnobOff = 0xC5FFFFFF;
        p.controlFill = 0x0FFFFFFF;
        p.controlFillHover = 0x15FFFFFF;
        p.controlSelected = 0x24FFFFFF;
        p.controlStroke = 0x12FFFFFF;
        p.accent = hasAccent ? accent : 0xFF4CC2FF;
        p.accentHover = Lerp(p.accent, 0xFF000000, 0.1f);
        p.onAccent = 0xFF000000;
        p.success = 0xFF6CCB5F;
        p.danger = 0xFFFF99A4;
    } else {
        p.base = 0xFFF3F3F3;
        p.navHover = 0x0A000000;
        p.navSelected = 0x0A000000;
        p.navPressed = 0x06000000;
        p.card = 0xB3FFFFFF;
        p.cardHover = 0x80F9F9F9;
        p.cardPressed = 0x4DF9F9F9;
        p.cardStroke = 0x0F000000;
        p.divider = 0x0F000000;
        p.text = 0xE4000000;
        p.textSecondary = 0x9E000000;
        p.textTertiary = 0x72000000;
        p.toggleOffFill = 0x06000000;
        p.toggleStroke = 0x72000000;
        p.toggleKnobOff = 0x9E000000;
        p.controlFill = 0xB3FFFFFF;
        p.controlFillHover = 0x80F9F9F9;
        p.controlSelected = 0xFFFFFFFF;
        p.controlStroke = 0x0F000000;
        p.accent = hasAccent ? accent : 0xFF005FB8;
        p.accentHover = Lerp(p.accent, 0xFFFFFFFF, 0.1f);
        p.onAccent = 0xFFFFFFFF;
        p.success = 0xFF0F7B0F;
        p.danger = 0xFFC42B1C;
    }
    return p;
}

Palette g_palette;
bool g_paletteValid = false;

}  // namespace

bool IsDarkMode() {
    DWORD value = 1;  // Light if the setting is missing
    DWORD size = sizeof(value);
    RegGetValueW(HKEY_CURRENT_USER, L"Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                 L"AppsUseLightTheme", RRF_RT_REG_DWORD, nullptr, &value, &size);
    return value == 0;
}

const Palette& Pal() {
    if (!g_paletteValid) {
        g_palette = MakePalette();
        g_paletteValid = true;
    }
    return g_palette;
}

void InvalidatePalette() {
    g_paletteValid = false;
}

void RegisterThemeWindow(HWND hwnd) {
    EnsureThemeListener();
    std::lock_guard<std::mutex> lock(g_themeMutex);
    if (std::find(g_themeWindows.begin(), g_themeWindows.end(), hwnd) == g_themeWindows.end()) {
        g_themeWindows.push_back(hwnd);
    }
}

void UnregisterThemeWindow(HWND hwnd) {
    std::lock_guard<std::mutex> lock(g_themeMutex);
    g_themeWindows.erase(std::remove(g_themeWindows.begin(), g_themeWindows.end(), hwnd), g_themeWindows.end());
}

// ===========================================================================
// Window frame
// ===========================================================================
bool ConfigureWindowFrame(HWND hwnd, bool wantMica) {
    constexpr DWORD kDwmUseImmersiveDarkMode = 20;
    constexpr DWORD kDwmCaptionColor = 35;
    constexpr DWORD kDwmSystemBackdropType = 38;
    constexpr int kBackdropMainWindow = 2;  // DWMSBT_MAINWINDOW = Mica

    const Palette& pal = Pal();
    BOOL dark = pal.dark;
    DwmSetWindowAttribute(hwnd, kDwmUseImmersiveDarkMode, &dark, sizeof(dark));

    bool mica = false;
    if (wantMica) {
        int backdrop = kBackdropMainWindow;
        mica = SUCCEEDED(DwmSetWindowAttribute(hwnd, kDwmSystemBackdropType, &backdrop, sizeof(backdrop)));
        if (mica) {
            MARGINS margins = { -1, -1, -1, -1 };
            DwmExtendFrameIntoClientArea(hwnd, &margins);
        }
    }
    if (!mica) {
        // Blend the title bar into the solid background (Windows 11; ignored elsewhere)
        COLORREF caption = ToColorRef(pal.base);
        DwmSetWindowAttribute(hwnd, kDwmCaptionColor, &caption, sizeof(caption));
    }

    // Clean title bar: no icon or caption text (the title is kept for taskbar / Alt+Tab)
    WTA_OPTIONS options = { WTNCA_NODRAWCAPTION | WTNCA_NODRAWICON, WTNCA_NODRAWCAPTION | WTNCA_NODRAWICON };
    SetWindowThemeAttribute(hwnd, WTA_NONCLIENT, &options, sizeof(options));
    return mica;
}


// ===========================================================================
// Renderer state: DirectWrite + WIC objects, caches, and the current canvas
// ===========================================================================
namespace {

struct FormatEntry {
    Face face;
    int px;
    DWRITE_FONT_WEIGHT weight;
    ComPtr<IDWriteTextFormat> format;
    ComPtr<IDWriteInlineObject> ellipsis;
};

struct LogoEntry {
    int px;
    std::vector<uint32_t> pixels;  // Premultiplied BGRA, px * px
};

struct Point {
    float x, y;
};
using Polygon = std::vector<Point>;

struct PathEntry {
    const char* d;
    std::vector<Polygon> polygons;  // Flattened, in viewBox units
};

struct Renderer {
    ComPtr<IDWriteFactory> dwrite;
    ComPtr<IDWriteFactory2> dwrite2;  // Grayscale glyph analysis (Windows 8.1+)
    ComPtr<IWICImagingFactory> wic;
    ComPtr<IWICBitmapFrameDecode> logoFrame;
    std::vector<FormatEntry> formats;
    std::vector<LogoEntry> logos;
    std::vector<PathEntry> paths;
    std::wstring faceText, faceDisplay, faceIcon;
};

Renderer g_r;

// Pixels: 32-bit premultiplied BGRA, top-down (same bit layout as ARGB: 0xAARRGGBB)
struct Clip {
    RectF r;
    float radius;
};

struct Canvas {
    uint32_t* px = nullptr;
    int w = 0;
    int h = 0;
    std::vector<Clip> clips;
};

Canvas g_c;

bool FamilyExists(const wchar_t* name) {
    ComPtr<IDWriteFontCollection> fonts;
    if (FAILED(g_r.dwrite->GetSystemFontCollection(&fonts))) return false;
    UINT32 index = 0;
    BOOL exists = FALSE;
    fonts->FindFamilyName(name, &index, &exists);
    return exists != FALSE;
}

// ---------------------------------------------------------------------------
// Pixel blending
// ---------------------------------------------------------------------------
inline uint32_t Div255(uint32_t x) {
    return (x + 128 + ((x + 128) >> 8)) >> 8;
}

// src-over with a straight-alpha color scaled by coverage (0..1)
inline void BlendColor(uint32_t& dst, ARGB color, float coverage) {
    uint32_t a = static_cast<uint32_t>(((color >> 24) & 0xFF) * coverage + 0.5f);
    if (a == 0) return;
    uint32_t r = Div255(((color >> 16) & 0xFF) * a);
    uint32_t g = Div255(((color >> 8) & 0xFF) * a);
    uint32_t b = Div255((color & 0xFF) * a);
    uint32_t inv = 255 - a;
    uint32_t d = dst;
    uint32_t da = a + Div255(((d >> 24) & 0xFF) * inv);
    uint32_t dr = r + Div255(((d >> 16) & 0xFF) * inv);
    uint32_t dg = g + Div255(((d >> 8) & 0xFF) * inv);
    uint32_t db = b + Div255((d & 0xFF) * inv);
    dst = (da << 24) | (dr << 16) | (dg << 8) | db;
}

// src-over with an already premultiplied pixel scaled by coverage
inline void BlendPremultiplied(uint32_t& dst, uint32_t src, float coverage) {
    uint32_t c = static_cast<uint32_t>(coverage * 255.0f + 0.5f);
    uint32_t sa = Div255(((src >> 24) & 0xFF) * c);
    if (sa == 0) return;
    uint32_t sr = Div255(((src >> 16) & 0xFF) * c);
    uint32_t sg = Div255(((src >> 8) & 0xFF) * c);
    uint32_t sb = Div255((src & 0xFF) * c);
    uint32_t inv = 255 - sa;
    uint32_t d = dst;
    dst = ((sa + Div255(((d >> 24) & 0xFF) * inv)) << 24) | ((sr + Div255(((d >> 16) & 0xFF) * inv)) << 16) |
          ((sg + Div255(((d >> 8) & 0xFF) * inv)) << 8) | (sb + Div255((d & 0xFF) * inv));
}

// ---------------------------------------------------------------------------
// Geometry: signed distance to a rounded rect -> 1px anti-aliased coverage
// ---------------------------------------------------------------------------
inline float RoundRectDistance(float x, float y, const RectF& r, float radius) {
    float hw = (r.right - r.left) * 0.5f;
    float hh = (r.bottom - r.top) * 0.5f;
    radius = (std::min)(radius, (std::min)(hw, hh));
    float qx = std::fabs(x - (r.left + hw)) - (hw - radius);
    float qy = std::fabs(y - (r.top + hh)) - (hh - radius);
    if (qx <= 0.0f || qy <= 0.0f) return (std::max)(qx, qy) - radius;  // Straight edges: no sqrt
    return std::sqrt(qx * qx + qy * qy) - radius;
}

inline float CoverageFromDistance(float d) {
    return std::clamp(0.5f - d, 0.0f, 1.0f);
}

// Coverage of the active clip stack at a pixel center
inline float ClipCoverage(float x, float y) {
    float c = 1.0f;
    for (const Clip& clip : g_c.clips) {
        c *= CoverageFromDistance(RoundRectDistance(x, y, clip.r, clip.radius));
        if (c <= 0.0f) break;
    }
    return c;
}

// Pixel bounds of a float rect, intersected with the canvas and the clip stack
bool PixelBounds(float l, float t, float r, float b, int& x0, int& y0, int& x1, int& y1) {
    for (const Clip& clip : g_c.clips) {
        l = (std::max)(l, clip.r.left);
        t = (std::max)(t, clip.r.top);
        r = (std::min)(r, clip.r.right);
        b = (std::min)(b, clip.r.bottom);
    }
    x0 = (std::max)(0, static_cast<int>(std::floor(l)));
    y0 = (std::max)(0, static_cast<int>(std::floor(t)));
    x1 = (std::min)(g_c.w, static_cast<int>(std::ceil(r)));
    y1 = (std::min)(g_c.h, static_cast<int>(std::ceil(b)));
    return g_c.px && x0 < x1 && y0 < y1;
}

// ---------------------------------------------------------------------------
// SVG path -> polygons (enough of the spec for icon paths: M L H V C S A Z)
// ---------------------------------------------------------------------------
class SvgPathReader {
public:
    explicit SvgPathReader(const char* d) : p_(d) {}

    bool AtEnd() {
        SkipSeparators();
        return *p_ == 0;
    }
    bool NextIsCommand() {
        SkipSeparators();
        return (*p_ >= 'A' && *p_ <= 'Z') || (*p_ >= 'a' && *p_ <= 'z');
    }
    char Command() { return *p_++; }

    bool Number(float& out) {
        SkipSeparators();
        const char* start = p_;
        if (*p_ == '+' || *p_ == '-') p_++;
        bool digits = false, dot = false;
        while ((*p_ >= '0' && *p_ <= '9') || (*p_ == '.' && !dot)) {
            if (*p_ == '.') dot = true; else digits = true;
            p_++;
        }
        if (!digits) { p_ = start; return false; }
        if (*p_ == 'e' || *p_ == 'E') {
            const char* e = p_ + 1;
            if (*e == '+' || *e == '-') e++;
            if (*e >= '0' && *e <= '9') {
                p_ = e;
                while (*p_ >= '0' && *p_ <= '9') p_++;
            }
        }
        out = strtof(std::string(start, p_).c_str(), nullptr);
        return true;
    }
    // Arc flags may be written without separators ("a1 1 0 011 1")
    bool Flag(bool& out) {
        SkipSeparators();
        if (*p_ != '0' && *p_ != '1') return false;
        out = *p_++ == '1';
        return true;
    }

private:
    void SkipSeparators() {
        while (*p_ == ' ' || *p_ == ',' || *p_ == '\t' || *p_ == '\n' || *p_ == '\r') p_++;
    }
    const char* p_;
};

void AddCubic(Polygon& poly, Point p0, Point c1, Point c2, Point p3) {
    const int steps = 12;
    for (int i = 1; i <= steps; i++) {
        float t = static_cast<float>(i) / steps, u = 1.0f - t;
        float a = u * u * u, b = 3 * u * u * t, c = 3 * u * t * t, d = t * t * t;
        poly.push_back({ a * p0.x + b * c1.x + c * c2.x + d * p3.x, a * p0.y + b * c1.y + c * c2.y + d * p3.y });
    }
}

// SVG elliptical arc (endpoint parameterization, SVG 1.1 F.6.5) flattened to points
void AddArc(Polygon& poly, Point p1, float rx, float ry, float rotationDeg, bool large, bool sweep, Point p2) {
    if (rx == 0.0f || ry == 0.0f || (p1.x == p2.x && p1.y == p2.y)) {
        poly.push_back(p2);
        return;
    }
    const float pi = 3.14159265358979f;
    float phi = rotationDeg * pi / 180.0f, cs = std::cos(phi), sn = std::sin(phi);
    float dx = (p1.x - p2.x) / 2, dy = (p1.y - p2.y) / 2;
    float x1 = cs * dx + sn * dy, y1 = -sn * dx + cs * dy;
    rx = std::fabs(rx);
    ry = std::fabs(ry);
    float lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if (lambda > 1.0f) {
        rx *= std::sqrt(lambda);
        ry *= std::sqrt(lambda);
    }
    float num = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
    float den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    float coef = (den > 0 ? std::sqrt((std::max)(0.0f, num / den)) : 0.0f) * (large == sweep ? -1.0f : 1.0f);
    float cxp = coef * rx * y1 / ry, cyp = -coef * ry * x1 / rx;
    float cx = cs * cxp - sn * cyp + (p1.x + p2.x) / 2, cy = sn * cxp + cs * cyp + (p1.y + p2.y) / 2;
    auto angle = [](float ux, float uy, float vx, float vy) { return std::atan2(ux * vy - uy * vx, ux * vx + uy * vy); };
    float ux = (x1 - cxp) / rx, uy = (y1 - cyp) / ry, vx = (-x1 - cxp) / rx, vy = (-y1 - cyp) / ry;
    float theta = angle(1, 0, ux, uy);
    float delta = angle(ux, uy, vx, vy);
    if (!sweep && delta > 0) delta -= 2 * pi;
    if (sweep && delta < 0) delta += 2 * pi;
    int steps = (std::max)(2, static_cast<int>(std::ceil(std::fabs(delta) / (pi / 24))));
    for (int i = 1; i <= steps; i++) {
        float t = theta + delta * i / steps;
        float ex = rx * std::cos(t), ey = ry * std::sin(t);
        poly.push_back({ cs * ex - sn * ey + cx, sn * ex + cs * ey + cy });
    }
}

std::vector<Polygon> ParseSvgPath(const char* d) {
    std::vector<Polygon> polys;
    SvgPathReader r(d);
    Point cur = { 0, 0 }, start = { 0, 0 }, lastCtrl = { 0, 0 };
    char cmd = 0, prev = 0;
    auto poly = [&]() -> Polygon& {
        if (polys.empty()) polys.push_back({ cur });
        return polys.back();
    };

    while (!r.AtEnd()) {
        if (r.NextIsCommand()) cmd = r.Command();
        else if (cmd == 'M') cmd = 'L';  // Extra pairs after a move are line-tos
        else if (cmd == 'm') cmd = 'l';
        bool rel = cmd >= 'a';
        Point base = rel ? cur : Point{ 0, 0 };
        float a = 0, b = 0, c = 0, e = 0, f = 0, g = 0;

        switch (cmd) {
            case 'M': case 'm':
                if (!r.Number(a) || !r.Number(b)) return {};
                cur = { base.x + a, base.y + b };
                start = cur;
                polys.push_back({ cur });
                break;
            case 'L': case 'l':
                if (!r.Number(a) || !r.Number(b)) return {};
                cur = { base.x + a, base.y + b };
                poly().push_back(cur);
                break;
            case 'H': case 'h':
                if (!r.Number(a)) return {};
                cur.x = (rel ? cur.x : 0) + a;
                poly().push_back(cur);
                break;
            case 'V': case 'v':
                if (!r.Number(a)) return {};
                cur.y = (rel ? cur.y : 0) + a;
                poly().push_back(cur);
                break;
            case 'C': case 'c': {
                if (!r.Number(a) || !r.Number(b) || !r.Number(c) || !r.Number(e) || !r.Number(f) || !r.Number(g))
                    return {};
                Point c1 = { base.x + a, base.y + b }, c2 = { base.x + c, base.y + e }, end = { base.x + f, base.y + g };
                AddCubic(poly(), cur, c1, c2, end);
                cur = end;
                lastCtrl = c2;
                break;
            }
            case 'S': case 's': {
                if (!r.Number(c) || !r.Number(e) || !r.Number(f) || !r.Number(g)) return {};
                bool smooth = prev == 'C' || prev == 'c' || prev == 'S' || prev == 's';
                Point c1 = smooth ? Point{ 2 * cur.x - lastCtrl.x, 2 * cur.y - lastCtrl.y } : cur;
                Point c2 = { base.x + c, base.y + e }, end = { base.x + f, base.y + g };
                AddCubic(poly(), cur, c1, c2, end);
                cur = end;
                lastCtrl = c2;
                break;
            }
            case 'A': case 'a': {
                bool large = false, sweep = false;
                if (!r.Number(a) || !r.Number(b) || !r.Number(c) || !r.Flag(large) || !r.Flag(sweep) ||
                    !r.Number(f) || !r.Number(g))
                    return {};
                Point end = { base.x + f, base.y + g };
                AddArc(poly(), cur, a, b, c, large, sweep, end);
                cur = end;
                break;
            }
            case 'Z': case 'z':
                cur = start;  // Polygons are implicitly closed when filled
                break;
            default:
                return {};  // Unsupported command
        }
        prev = cmd;
    }
    return polys;
}

// ---------------------------------------------------------------------------
// DirectWrite text -> grayscale glyph coverage blended into the canvas
// ---------------------------------------------------------------------------
class GlyphRenderer final : public IDWriteTextRenderer {
public:
    // IUnknown (static instance, not reference counted)
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** out) override {
        if (iid == __uuidof(IUnknown) || iid == __uuidof(IDWritePixelSnapping) || iid == __uuidof(IDWriteTextRenderer)) {
            *out = this;
            return S_OK;
        }
        *out = nullptr;
        return E_NOINTERFACE;
    }
    ULONG STDMETHODCALLTYPE AddRef() override { return 1; }
    ULONG STDMETHODCALLTYPE Release() override { return 1; }

    // IDWritePixelSnapping: layout is in device pixels
    HRESULT STDMETHODCALLTYPE IsPixelSnappingDisabled(void*, BOOL* disabled) override {
        *disabled = FALSE;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE GetCurrentTransform(void*, DWRITE_MATRIX* m) override {
        *m = { 1, 0, 0, 1, 0, 0 };
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE GetPixelsPerDip(void*, FLOAT* ppd) override {
        *ppd = 1.0f;
        return S_OK;
    }

    HRESULT STDMETHODCALLTYPE DrawGlyphRun(void* context, FLOAT baselineX, FLOAT baselineY,
                                           DWRITE_MEASURING_MODE measuring, const DWRITE_GLYPH_RUN* run,
                                           const DWRITE_GLYPH_RUN_DESCRIPTION*, IUnknown*) override {
        ARGB color = *static_cast<const ARGB*>(context);
        ComPtr<IDWriteGlyphRunAnalysis> analysis;
        DWRITE_TEXTURE_TYPE texture;
        if (g_r.dwrite2) {
            if (FAILED(g_r.dwrite2->CreateGlyphRunAnalysis(run, nullptr, DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC,
                                                           measuring, DWRITE_GRID_FIT_MODE_DEFAULT,
                                                           DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE, baselineX,
                                                           baselineY, &analysis)))
                return S_OK;
            texture = DWRITE_TEXTURE_ALIASED_1x1;  // 1 byte coverage per pixel in grayscale mode
        } else {
            if (FAILED(g_r.dwrite->CreateGlyphRunAnalysis(run, 1.0f, nullptr, DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC,
                                                          measuring, baselineX, baselineY, &analysis)))
                return S_OK;
            texture = DWRITE_TEXTURE_CLEARTYPE_3x1;  // Averaged to grayscale below
        }
        RECT bounds = {};
        if (FAILED(analysis->GetAlphaTextureBounds(texture, &bounds)) || IsRectEmpty(&bounds)) return S_OK;
        const int w = bounds.right - bounds.left, h = bounds.bottom - bounds.top;
        const int bpp = texture == DWRITE_TEXTURE_CLEARTYPE_3x1 ? 3 : 1;
        std::vector<BYTE> alpha(static_cast<size_t>(w) * h * bpp);
        if (FAILED(analysis->CreateAlphaTexture(texture, &bounds, alpha.data(), static_cast<UINT32>(alpha.size()))))
            return S_OK;

        int x0, y0, x1, y1;
        if (!PixelBounds(static_cast<float>(bounds.left), static_cast<float>(bounds.top),
                         static_cast<float>(bounds.right), static_cast<float>(bounds.bottom), x0, y0, x1, y1))
            return S_OK;
        const bool clipped = !g_c.clips.empty();
        for (int y = y0; y < y1; y++) {
            const BYTE* src = alpha.data() + (static_cast<size_t>(y - bounds.top) * w + (x0 - bounds.left)) * bpp;
            uint32_t* dst = g_c.px + static_cast<size_t>(y) * g_c.w;
            for (int x = x0; x < x1; x++, src += bpp) {
                int a = bpp == 3 ? (src[0] + src[1] + src[2]) / 3 : src[0];
                if (a == 0) continue;
                float cov = a / 255.0f;
                cov = cov + 0.35f * cov * (1.0f - cov);  // Mild contrast boost, like D2D's text rendering
                if (clipped) cov *= ClipCoverage(x + 0.5f, y + 0.5f);
                BlendColor(dst[x], color, cov);
            }
        }
        return S_OK;
    }

    HRESULT STDMETHODCALLTYPE DrawUnderline(void*, FLOAT, FLOAT, const DWRITE_UNDERLINE*, IUnknown*) override {
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE DrawStrikethrough(void*, FLOAT, FLOAT, const DWRITE_STRIKETHROUGH*, IUnknown*) override {
        return S_OK;
    }
    // Trimming ellipsis ("…") is an inline object; let it draw through this renderer
    HRESULT STDMETHODCALLTYPE DrawInlineObject(void* context, FLOAT x, FLOAT y, IDWriteInlineObject* object,
                                               BOOL sideways, BOOL rtl, IUnknown* effect) override {
        return object ? object->Draw(context, this, x, y, sideways, rtl, effect) : S_OK;
    }
};

GlyphRenderer g_glyphRenderer;

}  // namespace

// ===========================================================================
// Lifecycle
// ===========================================================================
bool InitRenderer() {
    if (g_r.dwrite) return true;
    CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);  // Needed for WIC; harmless if already done

    if (FAILED(DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, __uuidof(IDWriteFactory),
                                   reinterpret_cast<IUnknown**>(g_r.dwrite.GetAddressOf())))) {
        return false;
    }
    g_r.dwrite.As(&g_r.dwrite2);
    CoCreateInstance(CLSID_WICImagingFactory, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(g_r.wic.GetAddressOf()));

    g_r.faceText = FamilyExists(L"Segoe UI Variable Text") ? L"Segoe UI Variable Text" : L"Segoe UI";
    g_r.faceDisplay = FamilyExists(L"Segoe UI Variable Display") ? L"Segoe UI Variable Display" : L"Segoe UI";
    g_r.faceIcon = FamilyExists(L"Segoe Fluent Icons") ? L"Segoe Fluent Icons" : L"Segoe MDL2 Assets";

    // Decode the PNG logo once; scaled copies are cached per size
    if (g_r.wic) {
        HINSTANCE hInst = GetModuleHandle(nullptr);
        HRSRC res = FindResourceW(hInst, MAKEINTRESOURCEW(IDR_LOGO_PNG), L"PNG");
        HGLOBAL data = res ? LoadResource(hInst, res) : nullptr;
        void* bytes = data ? LockResource(data) : nullptr;
        DWORD size = res ? SizeofResource(hInst, res) : 0;
        ComPtr<IWICStream> stream;
        ComPtr<IWICBitmapDecoder> decoder;
        if (bytes && size && SUCCEEDED(g_r.wic->CreateStream(&stream)) &&
            SUCCEEDED(stream->InitializeFromMemory(static_cast<BYTE*>(bytes), size)) &&
            SUCCEEDED(g_r.wic->CreateDecoderFromStream(stream.Get(), nullptr, WICDecodeMetadataCacheOnLoad,
                                                       &decoder))) {
            decoder->GetFrame(0, &g_r.logoFrame);
        }
    }
    return true;
}

void RenderBuffered(HWND hwnd, HDC hdcWindow, ARGB clear, const std::function<void()>& draw) {
    RECT client;
    GetClientRect(hwnd, &client);
    if (client.right <= 0 || client.bottom <= 0) return;
    InitRenderer();  // May have been released while the windows were hidden

    // 32-bit premultiplied buffer: transparent pixels let Mica show through
    BITMAPINFO bi = {};
    bi.bmiHeader.biSize = sizeof(bi.bmiHeader);
    bi.bmiHeader.biWidth = client.right;
    bi.bmiHeader.biHeight = -client.bottom;  // Top-down
    bi.bmiHeader.biPlanes = 1;
    bi.bmiHeader.biBitCount = 32;
    bi.bmiHeader.biCompression = BI_RGB;
    void* bits = nullptr;
    HBITMAP dib = CreateDIBSection(hdcWindow, &bi, DIB_RGB_COLORS, &bits, nullptr, 0);
    if (!dib || !bits) {
        if (dib) DeleteObject(dib);
        return;
    }

    g_c.px = static_cast<uint32_t*>(bits);
    g_c.w = client.right;
    g_c.h = client.bottom;
    g_c.clips.clear();
    uint32_t a = (clear >> 24) & 0xFF;
    uint32_t clearPx = (a << 24) | (Div255(((clear >> 16) & 0xFF) * a) << 16) |
                       (Div255(((clear >> 8) & 0xFF) * a) << 8) | Div255((clear & 0xFF) * a);
    std::fill(g_c.px, g_c.px + static_cast<size_t>(g_c.w) * g_c.h, clearPx);

    draw();

    HDC hdc = CreateCompatibleDC(hdcWindow);
    HGDIOBJ oldBitmap = SelectObject(hdc, dib);
    BitBlt(hdcWindow, 0, 0, client.right, client.bottom, hdc, 0, 0, SRCCOPY);
    SelectObject(hdc, oldBitmap);
    DeleteDC(hdc);
    DeleteObject(dib);
    g_c.px = nullptr;
    g_c.w = g_c.h = 0;
    g_c.clips.clear();
}

void ClearDpiCaches() {
    g_r.formats.clear();
    g_r.logos.clear();
}

void ReleaseRenderResources() {
    // Everything: text formats and the factories hold font/glyph caches. InitRenderer() rebuilds lazily.
    g_r.formats.clear();
    g_r.logos.clear();
    g_r.paths.clear();
    g_r.logoFrame.Reset();
    g_r.wic.Reset();
    g_r.dwrite2.Reset();
    g_r.dwrite.Reset();
    TrimWorkingSet();
}

// ===========================================================================
// Text
// ===========================================================================
IDWriteTextFormat* Font(Face face, int px, DWRITE_FONT_WEIGHT weight) {
    if (!InitRenderer()) return nullptr;
    for (auto& e : g_r.formats) {
        if (e.face == face && e.px == px && e.weight == weight) return e.format.Get();
    }
    const std::wstring& family = face == Face::Display ? g_r.faceDisplay
                               : face == Face::Icon    ? g_r.faceIcon
                                                       : g_r.faceText;
    FormatEntry e{ face, px, weight };
    g_r.dwrite->CreateTextFormat(family.c_str(), nullptr, weight, DWRITE_FONT_STYLE_NORMAL,
                                 DWRITE_FONT_STRETCH_NORMAL, static_cast<float>(px), L"vi-VN", &e.format);
    if (!e.format) return nullptr;
    e.format->SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP);
    if (SUCCEEDED(g_r.dwrite->CreateEllipsisTrimmingSign(e.format.Get(), &e.ellipsis))) {
        DWRITE_TRIMMING trimming = { DWRITE_TRIMMING_GRANULARITY_CHARACTER, 0, 0 };
        e.format->SetTrimming(&trimming, e.ellipsis.Get());
    }
    g_r.formats.push_back(e);
    return g_r.formats.back().format.Get();
}

void DrawStr(const wchar_t* text, const RECT& rect, IDWriteTextFormat* font, ARGB color, Align align,
             VAlign valign) {
    if (!font || !g_c.px || !g_r.dwrite) return;
    font->SetTextAlignment(align == kCenter ? DWRITE_TEXT_ALIGNMENT_CENTER
                         : align == kRight  ? DWRITE_TEXT_ALIGNMENT_TRAILING
                                            : DWRITE_TEXT_ALIGNMENT_LEADING);
    font->SetParagraphAlignment(valign == kMiddle ? DWRITE_PARAGRAPH_ALIGNMENT_CENTER
                              : valign == kBottom ? DWRITE_PARAGRAPH_ALIGNMENT_FAR
                                                  : DWRITE_PARAGRAPH_ALIGNMENT_NEAR);
    ComPtr<IDWriteTextLayout> layout;
    float w = static_cast<float>((std::max)(0L, rect.right - rect.left));
    float h = static_cast<float>((std::max)(0L, rect.bottom - rect.top));
    if (FAILED(g_r.dwrite->CreateTextLayout(text, static_cast<UINT32>(wcslen(text)), font, w, h, &layout))) return;
    layout->Draw(&color, &g_glyphRenderer, static_cast<float>(rect.left), static_cast<float>(rect.top));
}

void DrawGlyph(wchar_t glyph, const RECT& rect, IDWriteTextFormat* font, ARGB color) {
    wchar_t text[2] = { glyph, 0 };
    DrawStr(text, rect, font, color, kCenter, kMiddle);
}

int MeasureStr(const wchar_t* text, IDWriteTextFormat* font) {
    if (!font || !g_r.dwrite) return 0;
    font->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
    ComPtr<IDWriteTextLayout> layout;
    if (FAILED(g_r.dwrite->CreateTextLayout(text, static_cast<UINT32>(wcslen(text)), font, 10000.0f, 1000.0f,
                                            &layout))) {
        return 0;
    }
    DWRITE_TEXT_METRICS metrics = {};
    layout->GetMetrics(&metrics);
    return static_cast<int>(metrics.widthIncludingTrailingWhitespace + 0.99f);
}

// ===========================================================================
// Shapes
// ===========================================================================
void FillRound(const RectF& r, float radius, ARGB fill, ARGB stroke) {
    int x0, y0, x1, y1;
    if (!PixelBounds(r.left, r.top, r.right, r.bottom, x0, y0, x1, y1)) return;
    const bool hasFill = (fill >> 24) != 0;
    const bool hasStroke = (stroke >> 24) != 0;
    const bool clipped = !g_c.clips.empty();
    // 1px stroke inside the edge: coverage(outer) - coverage(inset by 1px)
    const RectF inner = { r.left + 1, r.top + 1, r.right - 1, r.bottom - 1 };
    const bool hasInner = inner.right > inner.left && inner.bottom > inner.top;
    const float innerRadius = (std::max)(0.0f, radius - 1.0f);
    for (int y = y0; y < y1; y++) {
        uint32_t* row = g_c.px + static_cast<size_t>(y) * g_c.w;
        float cy = y + 0.5f;
        for (int x = x0; x < x1; x++) {
            float cx = x + 0.5f;
            float outer = CoverageFromDistance(RoundRectDistance(cx, cy, r, radius));
            if (outer <= 0.0f) continue;
            float clip = clipped ? ClipCoverage(cx, cy) : 1.0f;
            if (clip <= 0.0f) continue;
            if (hasFill) BlendColor(row[x], fill, outer * clip);
            if (hasStroke) {
                float in = hasInner ? CoverageFromDistance(RoundRectDistance(cx, cy, inner, innerRadius)) : 0.0f;
                float band = outer - in;
                if (band > 0.0f) BlendColor(row[x], stroke, band * clip);
            }
        }
    }
}

void FillRound(const RECT& r, float radius, ARGB fill, ARGB stroke) {
    FillRound(RectF{ static_cast<float>(r.left), static_cast<float>(r.top), static_cast<float>(r.right),
                     static_cast<float>(r.bottom) },
              radius, fill, stroke);
}

void FillRectF(const RECT& r, ARGB color) {
    FillRound(r, 0.0f, color);
}

void PushClipRound(const RECT& r, float radius) {
    g_c.clips.push_back({ { static_cast<float>(r.left), static_cast<float>(r.top), static_cast<float>(r.right),
                            static_cast<float>(r.bottom) },
                          radius });
}

void PopClip() {
    if (!g_c.clips.empty()) g_c.clips.pop_back();
}

void FillSvgPath(const char* d, float viewBox, const RECT& rect, ARGB color) {
    const std::vector<Polygon>* polys = nullptr;
    for (const auto& entry : g_r.paths) {
        if (entry.d == d) polys = &entry.polygons;
    }
    if (!polys) {
        g_r.paths.push_back({ d, ParseSvgPath(d) });
        polys = &g_r.paths.back().polygons;
    }
    if (polys->empty()) return;

    // viewBox -> device: centered, aspect kept
    float w = static_cast<float>(rect.right - rect.left), h = static_cast<float>(rect.bottom - rect.top);
    float scale = (std::min)(w, h) / viewBox;
    float ox = rect.left + (w - viewBox * scale) / 2, oy = rect.top + (h - viewBox * scale) / 2;
    struct Edge {
        float x0, y0, x1, y1;
    };
    std::vector<Edge> edges;
    float minX = 1e9f, minY = 1e9f, maxX = -1e9f, maxY = -1e9f;
    for (const Polygon& poly : *polys) {
        for (size_t i = 0; i < poly.size(); i++) {
            Point a = poly[i], b = poly[(i + 1) % poly.size()];  // Closed
            Edge e = { ox + a.x * scale, oy + a.y * scale, ox + b.x * scale, oy + b.y * scale };
            edges.push_back(e);
            minX = (std::min)(minX, e.x0);
            maxX = (std::max)(maxX, e.x0);
            minY = (std::min)(minY, e.y0);
            maxY = (std::max)(maxY, e.y0);
        }
    }
    int x0, y0, x1, y1;
    if (!PixelBounds(minX, minY, maxX, maxY, x0, y0, x1, y1)) return;

    // 4x4 supersampling, nonzero winding rule
    const bool clipped = !g_c.clips.empty();
    for (int y = y0; y < y1; y++) {
        uint32_t* row = g_c.px + static_cast<size_t>(y) * g_c.w;
        for (int x = x0; x < x1; x++) {
            int inside = 0;
            for (int sy = 0; sy < 4; sy++) {
                float py = y + (sy + 0.5f) / 4;
                for (int sx = 0; sx < 4; sx++) {
                    float px = x + (sx + 0.5f) / 4;
                    int winding = 0;
                    for (const Edge& e : edges) {
                        if ((e.y0 <= py) != (e.y1 <= py)) {
                            float xi = e.x0 + (py - e.y0) * (e.x1 - e.x0) / (e.y1 - e.y0);
                            if (xi > px) winding += e.y1 > e.y0 ? 1 : -1;
                        }
                    }
                    if (winding != 0) inside++;
                }
            }
            if (!inside) continue;
            float cov = inside / 16.0f;
            if (clipped) cov *= ClipCoverage(x + 0.5f, y + 0.5f);
            BlendColor(row[x], color, cov);
        }
    }
}

void DrawLogo(const RECT& dest) {
    int px = (std::min)(dest.right - dest.left, dest.bottom - dest.top);
    if (px <= 0 || !g_c.px || !InitRenderer()) return;
    const LogoEntry* logo = nullptr;
    for (const auto& e : g_r.logos) {
        if (e.px == px) logo = &e;
    }
    if (!logo) {
        // High-quality downscale (300px source) with WIC, converted to premultiplied BGRA
        if (!g_r.logoFrame || !g_r.wic) return;
        ComPtr<IWICBitmapScaler> scaler;
        ComPtr<IWICFormatConverter> converter;
        LogoEntry e{ px, std::vector<uint32_t>(static_cast<size_t>(px) * px) };
        if (FAILED(g_r.wic->CreateBitmapScaler(&scaler)) ||
            FAILED(scaler->Initialize(g_r.logoFrame.Get(), px, px, WICBitmapInterpolationModeHighQualityCubic)) ||
            FAILED(g_r.wic->CreateFormatConverter(&converter)) ||
            FAILED(converter->Initialize(scaler.Get(), GUID_WICPixelFormat32bppPBGRA, WICBitmapDitherTypeNone,
                                         nullptr, 0.0, WICBitmapPaletteTypeMedianCut)) ||
            FAILED(converter->CopyPixels(nullptr, px * 4, px * px * 4, reinterpret_cast<BYTE*>(e.pixels.data())))) {
            return;
        }
        g_r.logos.push_back(std::move(e));
        logo = &g_r.logos.back();
    }
    int x0, y0, x1, y1;
    if (!PixelBounds(static_cast<float>(dest.left), static_cast<float>(dest.top), static_cast<float>(dest.left + px),
                     static_cast<float>(dest.top + px), x0, y0, x1, y1))
        return;
    const bool clipped = !g_c.clips.empty();
    for (int y = y0; y < y1; y++) {
        uint32_t* row = g_c.px + static_cast<size_t>(y) * g_c.w;
        const uint32_t* src = logo->pixels.data() + static_cast<size_t>(y - dest.top) * px;
        for (int x = x0; x < x1; x++) {
            BlendPremultiplied(row[x], src[x - dest.left], clipped ? ClipCoverage(x + 0.5f, y + 0.5f) : 1.0f);
        }
    }
}

// Windows 11 toggle switch. progress: 0 = off, 1 = on.
void DrawToggle(int x, int y, float scale, float progress, bool hover, bool pressed) {
    const Palette& pal = Pal();
    float w = 40.0f * scale;
    float h = 20.0f * scale;
    float fx = static_cast<float>(x);
    float fy = static_cast<float>(y);

    ARGB accent = hover ? Lerp(pal.accent, pal.dark ? 0xFFFFFFFF : 0xFF000000, 0.08f) : pal.accent;
    ARGB offFill = hover ? Lerp(pal.toggleOffFill, pal.dark ? 0x30FFFFFF : 0x10000000, 0.5f) : pal.toggleOffFill;
    FillRound(RectF{ fx, fy, fx + w, fy + h }, h / 2.0f, Lerp(offFill, accent, progress),
              Lerp(pal.toggleStroke, accent, progress));

    float knobH = (hover || pressed ? 14.0f : 12.0f) * scale;
    float knobW = pressed ? knobH + 3.0f * scale : knobH;
    float cxOff = fx + h / 2.0f;
    float cxOn = fx + w - h / 2.0f;
    float cx = cxOff + (cxOn - cxOff) * progress;
    float ky = fy + (h - knobH) / 2.0f;
    FillRound(RectF{ cx - knobW / 2.0f, ky, cx + knobW / 2.0f, ky + knobH }, knobH / 2.0f,
              Lerp(pal.toggleKnobOff, pal.onAccent, progress));
}

int ClientWidth(HWND hwnd) {
    RECT rc;
    GetClientRect(hwnd, &rc);
    return rc.right;
}

int ClientHeight(HWND hwnd) {
    RECT rc;
    GetClientRect(hwnd, &rc);
    return rc.bottom;
}

} // namespace fluent
} // namespace gonhanh
