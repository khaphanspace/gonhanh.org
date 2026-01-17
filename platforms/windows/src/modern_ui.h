#pragma once

#include <windows.h>
#include <gdiplus.h>

#pragma comment(lib, "gdiplus.lib")

namespace gonhanh {
namespace ui {

// Theme colors
struct Theme {
    COLORREF windowBg;
    COLORREF sidebarBg;
    COLORREF textPrimary;
    COLORREF textSecondary;
    COLORREF textTertiary;
    COLORREF toggleOn;      // Windows 11 blue
    COLORREF toggleOff;     // Gray
    COLORREF toggleKnob;    // White
};

// Dark theme
inline const Theme DarkTheme = {
    RGB(30, 30, 30),      // windowBg
    RGB(40, 40, 40),      // sidebarBg
    RGB(255, 255, 255),   // textPrimary
    RGB(170, 170, 170),   // textSecondary
    RGB(120, 120, 120),   // textTertiary
    RGB(0, 120, 212),     // toggleOn - Windows 11 blue
    RGB(100, 100, 100),   // toggleOff
    RGB(255, 255, 255),   // toggleKnob
};

// Light theme
inline const Theme LightTheme = {
    RGB(246, 246, 246),   // windowBg
    RGB(236, 236, 236),   // sidebarBg
    RGB(0, 0, 0),         // textPrimary
    RGB(100, 100, 100),   // textSecondary
    RGB(150, 150, 150),   // textTertiary
    RGB(0, 120, 212),     // toggleOn - Windows 11 blue
    RGB(200, 200, 200),   // toggleOff
    RGB(255, 255, 255),   // toggleKnob
};

// Get current system theme
bool IsDarkMode();
const Theme& GetTheme();

// DPI scaling
float GetDpiScale(HWND hwnd);
int Scale(int value, HWND hwnd);
int Scale(int value, float dpiScale);

// Initialize GDI+
void InitGdiPlus();
void ShutdownGdiPlus();

// Drawing helpers
void DrawRoundedRect(HDC hdc, const RECT& rect, int radius, COLORREF fill, COLORREF border);
void DrawText(HDC hdc, const wchar_t* text, const RECT& rect, COLORREF color, int fontSize, bool bold = false, UINT align = DT_LEFT | DT_VCENTER);
void DrawToggleSwitch(HDC hdc, int x, int y, int width, int height, bool isOn, bool isHovered);
void DrawPngFromResource(HDC hdc, int resourceId, int x, int y, int width, int height);

// Toggle switch control
#define TOGGLE_SWITCH_CLASS L"GoNhanhToggleSwitch"
#define WM_TOGGLE_CHANGED (WM_USER + 100)

void RegisterToggleSwitchClass(HINSTANCE hInstance);
HWND CreateToggleSwitch(HWND parent, int x, int y, int id, bool initialState = false);
bool GetToggleState(HWND hwnd);
void SetToggleState(HWND hwnd, bool state);

} // namespace ui
} // namespace gonhanh
