#pragma once

#include <windows.h>
#include <gdiplus.h>
#include <dwmapi.h>

#pragma comment(lib, "gdiplus.lib")
#pragma comment(lib, "dwmapi.lib")

namespace gonhanh {
namespace ui {

// Theme colors
struct Theme {
    // Background colors
    COLORREF windowBg;
    COLORREF sidebarBg;
    COLORREF cardBg;
    COLORREF cardBorder;
    COLORREF divider;

    // Text colors
    COLORREF textPrimary;
    COLORREF textSecondary;
    COLORREF textTertiary;

    // Accent colors
    COLORREF accent;
    COLORREF accentHover;

    // Toggle switch colors
    COLORREF toggleOn;
    COLORREF toggleOff;
    COLORREF toggleKnob;

    // Hover states
    COLORREF hoverBg;
    COLORREF selectedBg;
};

// Dark theme
inline const Theme DarkTheme = {
    RGB(30, 30, 30),      // windowBg
    RGB(40, 40, 40),      // sidebarBg
    RGB(50, 50, 50),      // cardBg
    RGB(70, 70, 70),      // cardBorder
    RGB(60, 60, 60),      // divider
    RGB(255, 255, 255),   // textPrimary
    RGB(170, 170, 170),   // textSecondary
    RGB(120, 120, 120),   // textTertiary
    RGB(0, 122, 255),     // accent
    RGB(30, 144, 255),    // accentHover
    RGB(52, 199, 89),     // toggleOn (green)
    RGB(80, 80, 80),      // toggleOff
    RGB(255, 255, 255),   // toggleKnob
    RGB(60, 60, 60),      // hoverBg
    RGB(70, 70, 70),      // selectedBg
};

// Light theme
inline const Theme LightTheme = {
    RGB(246, 246, 246),   // windowBg
    RGB(236, 236, 236),   // sidebarBg
    RGB(255, 255, 255),   // cardBg
    RGB(220, 220, 220),   // cardBorder
    RGB(230, 230, 230),   // divider
    RGB(0, 0, 0),         // textPrimary
    RGB(100, 100, 100),   // textSecondary
    RGB(150, 150, 150),   // textTertiary
    RGB(0, 122, 255),     // accent
    RGB(30, 144, 255),    // accentHover
    RGB(52, 199, 89),     // toggleOn (green)
    RGB(200, 200, 200),   // toggleOff
    RGB(255, 255, 255),   // toggleKnob
    RGB(230, 230, 230),   // hoverBg
    RGB(220, 220, 220),   // selectedBg
};

// Get current system theme (true = dark)
bool IsDarkMode();

// Get current theme
const Theme& GetTheme();

// Initialize GDI+
void InitGdiPlus();
void ShutdownGdiPlus();

// Drawing helpers
void DrawRoundedRect(HDC hdc, const RECT& rect, int radius, COLORREF fill, COLORREF border);
void DrawToggleSwitch(HDC hdc, int x, int y, int width, int height, bool isOn, bool isHovered);
void DrawText(HDC hdc, const wchar_t* text, const RECT& rect, COLORREF color, int fontSize, bool bold = false, UINT align = DT_LEFT | DT_VCENTER);
void DrawDivider(HDC hdc, int x, int y, int width);

// Custom control class names
#define TOGGLE_SWITCH_CLASS L"GoNhanhToggleSwitch"
#define CARD_CLASS L"GoNhanhCard"

// Register custom controls
void RegisterCustomControls(HINSTANCE hInstance);

// Toggle switch control
HWND CreateToggleSwitch(HWND parent, int x, int y, int id, bool initialState = false);
bool GetToggleState(HWND hwnd);
void SetToggleState(HWND hwnd, bool state);

// Messages
#define WM_TOGGLE_CHANGED (WM_USER + 100)

} // namespace ui
} // namespace gonhanh
