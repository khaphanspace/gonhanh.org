#include "modern_ui.h"
#include <uxtheme.h>

#pragma comment(lib, "uxtheme.lib")

namespace gonhanh {
namespace ui {

static ULONG_PTR gdiplusToken = 0;
static bool isDarkModeCache = false;
static bool themeCacheValid = false;

// Toggle switch state
struct ToggleData {
    bool isOn;
    bool isHovered;
    bool isPressed;
    float animProgress; // 0.0 = off, 1.0 = on
};

bool IsDarkMode() {
    if (!themeCacheValid) {
        HKEY hKey;
        DWORD value = 1;
        DWORD size = sizeof(value);

        if (RegOpenKeyExW(HKEY_CURRENT_USER,
            L"Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            0, KEY_READ, &hKey) == ERROR_SUCCESS) {
            RegQueryValueExW(hKey, L"AppsUseLightTheme", NULL, NULL, (LPBYTE)&value, &size);
            RegCloseKey(hKey);
        }

        isDarkModeCache = (value == 0);
        themeCacheValid = true;
    }
    return isDarkModeCache;
}

const Theme& GetTheme() {
    return IsDarkMode() ? DarkTheme : LightTheme;
}

void InitGdiPlus() {
    Gdiplus::GdiplusStartupInput gdiplusStartupInput;
    Gdiplus::GdiplusStartup(&gdiplusToken, &gdiplusStartupInput, NULL);
}

void ShutdownGdiPlus() {
    if (gdiplusToken) {
        Gdiplus::GdiplusShutdown(gdiplusToken);
        gdiplusToken = 0;
    }
}

void DrawRoundedRect(HDC hdc, const RECT& rect, int radius, COLORREF fill, COLORREF border) {
    Gdiplus::Graphics g(hdc);
    g.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);

    int width = rect.right - rect.left;
    int height = rect.bottom - rect.top;

    Gdiplus::GraphicsPath path;
    path.AddArc(rect.left, rect.top, radius * 2, radius * 2, 180, 90);
    path.AddArc(rect.right - radius * 2, rect.top, radius * 2, radius * 2, 270, 90);
    path.AddArc(rect.right - radius * 2, rect.bottom - radius * 2, radius * 2, radius * 2, 0, 90);
    path.AddArc(rect.left, rect.bottom - radius * 2, radius * 2, radius * 2, 90, 90);
    path.CloseFigure();

    // Fill
    Gdiplus::SolidBrush brush(Gdiplus::Color(
        GetRValue(fill), GetGValue(fill), GetBValue(fill)));
    g.FillPath(&brush, &path);

    // Border
    if (border != fill) {
        Gdiplus::Pen pen(Gdiplus::Color(
            GetRValue(border), GetGValue(border), GetBValue(border)), 1.0f);
        g.DrawPath(&pen, &path);
    }
}

void DrawToggleSwitch(HDC hdc, int x, int y, int width, int height, bool isOn, bool isHovered) {
    const Theme& theme = GetTheme();

    Gdiplus::Graphics g(hdc);
    g.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);

    // Track dimensions
    int trackWidth = width;
    int trackHeight = height;
    int trackRadius = trackHeight / 2;

    // Track color
    COLORREF trackColor = isOn ? theme.toggleOn : theme.toggleOff;
    if (isHovered && !isOn) {
        trackColor = RGB(
            min(255, GetRValue(trackColor) + 20),
            min(255, GetGValue(trackColor) + 20),
            min(255, GetBValue(trackColor) + 20)
        );
    }

    // Draw track
    RECT trackRect = { x, y, x + trackWidth, y + trackHeight };
    DrawRoundedRect(hdc, trackRect, trackRadius, trackColor, trackColor);

    // Knob dimensions
    int knobPadding = 2;
    int knobSize = trackHeight - knobPadding * 2;
    int knobX = isOn ? (x + trackWidth - knobSize - knobPadding) : (x + knobPadding);
    int knobY = y + knobPadding;

    // Draw knob shadow
    Gdiplus::SolidBrush shadowBrush(Gdiplus::Color(30, 0, 0, 0));
    g.FillEllipse(&shadowBrush, knobX, knobY + 1, knobSize, knobSize);

    // Draw knob
    Gdiplus::SolidBrush knobBrush(Gdiplus::Color(255, 255, 255));
    g.FillEllipse(&knobBrush, knobX, knobY, knobSize, knobSize);
}

void DrawText(HDC hdc, const wchar_t* text, const RECT& rect, COLORREF color, int fontSize, bool bold, UINT align) {
    HFONT hFont = CreateFontW(
        -MulDiv(fontSize, GetDeviceCaps(hdc, LOGPIXELSY), 72),
        0, 0, 0,
        bold ? FW_SEMIBOLD : FW_NORMAL,
        FALSE, FALSE, FALSE,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY,
        DEFAULT_PITCH | FF_DONTCARE,
        L"Segoe UI"
    );

    HFONT oldFont = (HFONT)SelectObject(hdc, hFont);
    SetTextColor(hdc, color);
    SetBkMode(hdc, TRANSPARENT);

    RECT drawRect = rect;
    DrawTextW(hdc, text, -1, &drawRect, align | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, oldFont);
    DeleteObject(hFont);
}

void DrawDivider(HDC hdc, int x, int y, int width) {
    const Theme& theme = GetTheme();
    HPEN pen = CreatePen(PS_SOLID, 1, theme.divider);
    HPEN oldPen = (HPEN)SelectObject(hdc, pen);
    MoveToEx(hdc, x, y, NULL);
    LineTo(hdc, x + width, y);
    SelectObject(hdc, oldPen);
    DeleteObject(pen);
}

// Toggle switch window procedure
static LRESULT CALLBACK ToggleSwitchProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    ToggleData* data = (ToggleData*)GetWindowLongPtr(hwnd, GWLP_USERDATA);

    switch (msg) {
        case WM_CREATE: {
            data = new ToggleData{ false, false, false, 0.0f };
            SetWindowLongPtr(hwnd, GWLP_USERDATA, (LONG_PTR)data);
            return 0;
        }

        case WM_DESTROY: {
            if (data) delete data;
            SetWindowLongPtr(hwnd, GWLP_USERDATA, 0);
            return 0;
        }

        case WM_PAINT: {
            PAINTSTRUCT ps;
            HDC hdc = BeginPaint(hwnd, &ps);

            RECT rect;
            GetClientRect(hwnd, &rect);

            // Double buffer
            HDC memDC = CreateCompatibleDC(hdc);
            HBITMAP memBitmap = CreateCompatibleBitmap(hdc, rect.right, rect.bottom);
            HBITMAP oldBitmap = (HBITMAP)SelectObject(memDC, memBitmap);

            // Fill background (transparent - use parent bg)
            const Theme& theme = GetTheme();
            HBRUSH bgBrush = CreateSolidBrush(theme.cardBg);
            FillRect(memDC, &rect, bgBrush);
            DeleteObject(bgBrush);

            // Draw toggle
            DrawToggleSwitch(memDC, 0, 0, rect.right, rect.bottom,
                data ? data->isOn : false,
                data ? data->isHovered : false);

            // Blit to screen
            BitBlt(hdc, 0, 0, rect.right, rect.bottom, memDC, 0, 0, SRCCOPY);

            SelectObject(memDC, oldBitmap);
            DeleteObject(memBitmap);
            DeleteDC(memDC);

            EndPaint(hwnd, &ps);
            return 0;
        }

        case WM_LBUTTONDOWN: {
            if (data) data->isPressed = true;
            SetCapture(hwnd);
            return 0;
        }

        case WM_LBUTTONUP: {
            if (data && data->isPressed) {
                data->isOn = !data->isOn;
                data->isPressed = false;
                InvalidateRect(hwnd, NULL, FALSE);

                // Notify parent
                HWND parent = GetParent(hwnd);
                if (parent) {
                    SendMessageW(parent, WM_TOGGLE_CHANGED,
                        (WPARAM)GetDlgCtrlID(hwnd), (LPARAM)data->isOn);
                }
            }
            ReleaseCapture();
            return 0;
        }

        case WM_MOUSEMOVE: {
            if (data && !data->isHovered) {
                data->isHovered = true;
                InvalidateRect(hwnd, NULL, FALSE);

                TRACKMOUSEEVENT tme = { sizeof(tme), TME_LEAVE, hwnd, 0 };
                TrackMouseEvent(&tme);
            }
            return 0;
        }

        case WM_MOUSELEAVE: {
            if (data) {
                data->isHovered = false;
                InvalidateRect(hwnd, NULL, FALSE);
            }
            return 0;
        }

        case WM_ERASEBKGND:
            return 1;
    }

    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

void RegisterCustomControls(HINSTANCE hInstance) {
    WNDCLASSEXW wc = { sizeof(wc) };
    wc.lpfnWndProc = ToggleSwitchProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = TOGGLE_SWITCH_CLASS;
    wc.hCursor = LoadCursor(NULL, IDC_HAND);
    RegisterClassExW(&wc);
}

HWND CreateToggleSwitch(HWND parent, int x, int y, int id, bool initialState) {
    HWND hwnd = CreateWindowExW(
        0, TOGGLE_SWITCH_CLASS, NULL,
        WS_CHILD | WS_VISIBLE,
        x, y, 51, 31,  // macOS toggle size
        parent, (HMENU)(INT_PTR)id,
        GetModuleHandle(NULL), NULL
    );

    if (hwnd && initialState) {
        SetToggleState(hwnd, true);
    }

    return hwnd;
}

bool GetToggleState(HWND hwnd) {
    ToggleData* data = (ToggleData*)GetWindowLongPtr(hwnd, GWLP_USERDATA);
    return data ? data->isOn : false;
}

void SetToggleState(HWND hwnd, bool state) {
    ToggleData* data = (ToggleData*)GetWindowLongPtr(hwnd, GWLP_USERDATA);
    if (data) {
        data->isOn = state;
        InvalidateRect(hwnd, NULL, FALSE);
    }
}

} // namespace ui
} // namespace gonhanh
