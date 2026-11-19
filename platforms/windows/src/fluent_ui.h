#pragma once
// Shared Windows 11 (Fluent) drawing layer for custom-painted windows:
// palette that follows the system theme/accent, and a small CPU renderer that draws into a
// 32-bit premultiplied buffer (so Mica can show through): anti-aliased rounded rects, toggles,
// SVG paths, the app logo, and DirectWrite text (glyph coverage blended by hand).
// No Direct2D: its software path (WARP) alone costs ~3.5 MB while a window is open.
#include <windows.h>
#include <dwrite.h>
#include <cstdint>
#include <functional>

namespace gonhanh {
namespace fluent {

// ---------------------------------------------------------------------------
// Colors
// ---------------------------------------------------------------------------
using ARGB = uint32_t;  // Straight (non-premultiplied) alpha, 0xAARRGGBB

ARGB Lerp(ARGB a, ARGB b, float t);
ARGB Over(ARGB top, ARGB bottom);  // Composite a translucent color onto an opaque one
COLORREF ToColorRef(ARGB opaque);

struct Palette {
    bool dark;
    ARGB base;                                  // Solid window background (no Mica)
    ARGB navHover, navSelected, navPressed;
    ARGB card, cardHover, cardPressed, cardStroke, divider;
    ARGB text, textSecondary, textTertiary;
    ARGB toggleOffFill, toggleStroke, toggleKnobOff;
    ARGB controlFill, controlFillHover, controlSelected, controlStroke;
    ARGB accent, accentHover, onAccent;
    ARGB success, heart, danger;
};

bool IsDarkMode();  // System "app mode" (Settings > Personalization > Colors)
const Palette& Pal();
void InvalidatePalette();  // Call on WM_SETTINGCHANGE("ImmersiveColorSet") / accent change

// Posted to registered windows when the system accent color or light/dark mode changes
// (from Windows' UISettings.ColorValuesChanged). Handle with InvalidatePalette() + repaint.
constexpr UINT WM_FLUENT_THEME_CHANGED = WM_APP + 40;
void RegisterThemeWindow(HWND hwnd);
void UnregisterThemeWindow(HWND hwnd);  // From WM_DESTROY

// ---------------------------------------------------------------------------
// Window frame
// ---------------------------------------------------------------------------
// Dark/light title bar, no caption text or icon. With wantMica, enables the Mica
// backdrop (Windows 11 22H2+) and returns true if active; otherwise the title bar is
// tinted with the solid base color.
bool ConfigureWindowFrame(HWND hwnd, bool wantMica);

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------
bool InitRenderer();
// Paint `draw` into an off-screen 32-bit buffer cleared to `clear` (0 = transparent,
// for Mica) and blit it to hdc in one go.
void RenderBuffered(HWND hwnd, HDC hdc, ARGB clear, const std::function<void()>& draw);
void ClearDpiCaches();  // Fonts and scaled bitmaps, after a DPI change

// When no custom-painted window is visible: drop DirectWrite/WIC objects, cached fonts/bitmaps and
// return memory to the OS, so the background IME stays small. Everything is recreated lazily.
void ReleaseRenderResources();

enum class Face { Text, Display, Icon };
IDWriteTextFormat* Font(Face face, int px, DWRITE_FONT_WEIGHT weight = DWRITE_FONT_WEIGHT_NORMAL);
void DrawLogo(const RECT& dest);  // App logo (PNG resource), high-quality scaled to dest

enum Align { kLeft = 0, kCenter = 1, kRight = 2 };
enum VAlign { kTop = 0, kMiddle = 1, kBottom = 2 };

// Fractional rectangle, for animated positions
struct RectF {
    float left, top, right, bottom;
};
void DrawStr(const wchar_t* text, const RECT& rect, IDWriteTextFormat* font, ARGB color,
             Align align = kLeft, VAlign valign = kMiddle);
void DrawGlyph(wchar_t glyph, const RECT& rect, IDWriteTextFormat* font, ARGB color);
int MeasureStr(const wchar_t* text, IDWriteTextFormat* font);
void FillRectF(const RECT& r, ARGB color);
// Anti-aliased rounded rect: fill (alpha 0 = none) and optional 1px inner stroke
void FillRound(const RECT& r, float radius, ARGB fill, ARGB stroke = 0);
void FillRound(const RectF& r, float radius, ARGB fill, ARGB stroke = 0);
// Clip subsequent drawing to a rounded rect (nestable; intersected with the current clip)
void PushClipRound(const RECT& r, float radius);
void PopClip();
// Fill an SVG path ("d" attribute; M L H V C S A Z, absolute and relative) defined in a
// viewBox of size `viewBox`, scaled into `rect` (centered, aspect kept). Geometry is cached
// per `d` pointer, so pass a string literal / static.
void FillSvgPath(const char* d, float viewBox, const RECT& rect, ARGB color);

// Windows 11 toggle switch (40x20 logical). progress: 0 = off, 1 = on.
void DrawToggle(int x, int y, float scale, float progress, bool hover, bool pressed);

int ClientWidth(HWND hwnd);
int ClientHeight(HWND hwnd);

} // namespace fluent
} // namespace gonhanh
