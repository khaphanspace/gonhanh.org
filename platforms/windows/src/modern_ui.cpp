#include "modern_ui.h"
#include <shlwapi.h>

#pragma comment(lib, "shlwapi.lib")

namespace gonhanh {
namespace ui {

static ULONG_PTR gdiplusToken = 0;
static bool isDarkModeCache = false;
static bool themeCacheValid = false;

// Toggle switch state
struct ToggleData {
    bool isOn;
    bool isHovered;
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

float GetDpiScale(HWND hwnd) {
    HDC hdc = GetDC(hwnd);
    int dpi = GetDeviceCaps(hdc, LOGPIXELSX);
    ReleaseDC(hwnd, hdc);
    return dpi / 96.0f;
}

int Scale(int value, HWND hwnd) {
    return static_cast<int>(value * GetDpiScale(hwnd));
}

int Scale(int value, float dpiScale) {
    return static_cast<int>(value * dpiScale);
}

void InitGdiPlus() {
    if (!gdiplusToken) {
        Gdiplus::GdiplusStartupInput gdiplusStartupInput;
        Gdiplus::GdiplusStartup(&gdiplusToken, &gdiplusStartupInput, NULL);
    }
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

    Gdiplus::GraphicsPath path;
    path.AddArc(rect.left, rect.top, radius * 2, radius * 2, 180, 90);
    path.AddArc(rect.right - radius * 2, rect.top, radius * 2, radius * 2, 270, 90);
    path.AddArc(rect.right - radius * 2, rect.bottom - radius * 2, radius * 2, radius * 2, 0, 90);
    path.AddArc(rect.left, rect.bottom - radius * 2, radius * 2, radius * 2, 90, 90);
    path.CloseFigure();

    Gdiplus::SolidBrush brush(Gdiplus::Color(GetRValue(fill), GetGValue(fill), GetBValue(fill)));
    g.FillPath(&brush, &path);

    if (border != fill) {
        Gdiplus::Pen pen(Gdiplus::Color(GetRValue(border), GetGValue(border), GetBValue(border)), 1.0f);
        g.DrawPath(&pen, &path);
    }
}

void DrawText(HDC hdc, const wchar_t* text, const RECT& rect, COLORREF color, int fontSize, bool bold, UINT align) {
    HFONT hFont = CreateFontW(
        -MulDiv(fontSize, GetDeviceCaps(hdc, LOGPIXELSY), 72),
        0, 0, 0,
        bold ? FW_SEMIBOLD : FW_NORMAL,
        FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI"
    );

    HFONT oldFont = (HFONT)SelectObject(hdc, hFont);
    SetTextColor(hdc, color);
    SetBkMode(hdc, TRANSPARENT);

    RECT drawRect = rect;
    DrawTextW(hdc, text, -1, &drawRect, align | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, oldFont);
    DeleteObject(hFont);
}

// Windows 11 style toggle switch
void DrawToggleSwitch(HDC hdc, int x, int y, int width, int height, bool isOn, bool isHovered) {
    const Theme& theme = GetTheme();

    Gdiplus::Graphics g(hdc);
    g.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);

    int trackRadius = height / 2;

    // Track color - Windows 11 blue when on, gray when off
    COLORREF trackColor = isOn ? theme.toggleOn : theme.toggleOff;

    // Hover effect
    if (isHovered && !isOn) {
        trackColor = RGB(
            min(255, GetRValue(trackColor) + 30),
            min(255, GetGValue(trackColor) + 30),
            min(255, GetBValue(trackColor) + 30)
        );
    }

    // Draw track (pill shape)
    Gdiplus::GraphicsPath trackPath;
    trackPath.AddArc(x, y, height, height, 90, 180);
    trackPath.AddArc(x + width - height, y, height, height, 270, 180);
    trackPath.CloseFigure();

    Gdiplus::SolidBrush trackBrush(Gdiplus::Color(
        GetRValue(trackColor), GetGValue(trackColor), GetBValue(trackColor)));
    g.FillPath(&trackBrush, &trackPath);

    // Knob position
    int knobPadding = 3;
    int knobSize = height - knobPadding * 2;
    int knobX = isOn ? (x + width - knobSize - knobPadding) : (x + knobPadding);
    int knobY = y + knobPadding;

    // Draw knob shadow
    Gdiplus::SolidBrush shadowBrush(Gdiplus::Color(40, 0, 0, 0));
    g.FillEllipse(&shadowBrush, knobX, knobY + 1, knobSize, knobSize);

    // Draw knob (white circle)
    Gdiplus::SolidBrush knobBrush(Gdiplus::Color(
        GetRValue(theme.toggleKnob), GetGValue(theme.toggleKnob), GetBValue(theme.toggleKnob)));
    g.FillEllipse(&knobBrush, knobX, knobY, knobSize, knobSize);
}

void DrawPngFromResource(HDC hdc, int resourceId, int x, int y, int width, int height) {
    HINSTANCE hInst = GetModuleHandle(NULL);

    // Find and load resource
    HRSRC hRes = FindResourceW(hInst, MAKEINTRESOURCEW(resourceId), L"PNG");
    if (!hRes) return;

    HGLOBAL hData = LoadResource(hInst, hRes);
    if (!hData) return;

    void* pData = LockResource(hData);
    DWORD dataSize = SizeofResource(hInst, hRes);
    if (!pData || !dataSize) return;

    // Create IStream from memory
    IStream* pStream = SHCreateMemStream((const BYTE*)pData, dataSize);
    if (!pStream) return;

    // Load image using GDI+
    Gdiplus::Image* image = Gdiplus::Image::FromStream(pStream);
    pStream->Release();

    if (image && image->GetLastStatus() == Gdiplus::Ok) {
        Gdiplus::Graphics g(hdc);
        g.SetInterpolationMode(Gdiplus::InterpolationModeHighQualityBicubic);
        g.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);
        g.DrawImage(image, x, y, width, height);
        delete image;
    }
}

// Toggle switch window procedure
static LRESULT CALLBACK ToggleSwitchProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    ToggleData* data = (ToggleData*)GetWindowLongPtr(hwnd, GWLP_USERDATA);

    switch (msg) {
        case WM_CREATE: {
            data = new ToggleData{ false, false };
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

            // Fill with parent background
            HWND parent = GetParent(hwnd);
            HBRUSH bgBrush = (HBRUSH)SendMessage(parent, WM_CTLCOLORSTATIC, (WPARAM)memDC, (LPARAM)hwnd);
            if (!bgBrush) bgBrush = (HBRUSH)GetStockObject(WHITE_BRUSH);
            FillRect(memDC, &rect, bgBrush);

            // Draw toggle
            DrawToggleSwitch(memDC, 0, 0, rect.right, rect.bottom,
                data ? data->isOn : false,
                data ? data->isHovered : false);

            BitBlt(hdc, 0, 0, rect.right, rect.bottom, memDC, 0, 0, SRCCOPY);

            SelectObject(memDC, oldBitmap);
            DeleteObject(memBitmap);
            DeleteDC(memDC);

            EndPaint(hwnd, &ps);
            return 0;
        }

        case WM_LBUTTONUP: {
            if (data) {
                data->isOn = !data->isOn;
                InvalidateRect(hwnd, NULL, FALSE);

                // Notify parent
                HWND parent = GetParent(hwnd);
                if (parent) {
                    SendMessageW(parent, WM_TOGGLE_CHANGED,
                        (WPARAM)GetDlgCtrlID(hwnd), (LPARAM)data->isOn);
                }
            }
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

void RegisterToggleSwitchClass(HINSTANCE hInstance) {
    WNDCLASSEXW wc = { sizeof(wc) };
    wc.lpfnWndProc = ToggleSwitchProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = TOGGLE_SWITCH_CLASS;
    wc.hCursor = LoadCursor(NULL, IDC_HAND);
    RegisterClassExW(&wc);
}

HWND CreateToggleSwitch(HWND parent, int x, int y, int id, bool initialState) {
    // DPI-scaled toggle size
    float dpi = GetDpiScale(parent);
    int width = Scale(44, dpi);
    int height = Scale(20, dpi);

    HWND hwnd = CreateWindowExW(
        0, TOGGLE_SWITCH_CLASS, NULL,
        WS_CHILD | WS_VISIBLE,
        x, y, width, height,
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
