#include "modern_ui.h"
#include <shlwapi.h>
#include <cmath>

#pragma comment(lib, "shlwapi.lib")

namespace gonhanh {
namespace ui {

static ULONG_PTR gdiplusToken = 0;
static bool isDarkModeCache = false;
static bool themeCacheValid = false;

// Animation constants
static const UINT_PTR TOGGLE_ANIM_TIMER_ID = 42;
static const int ANIM_INTERVAL_MS = 16;       // ~60fps
static const float ANIM_DURATION_MS = 150.0f;  // 150ms total slide
static const float ANIM_STEP = ANIM_INTERVAL_MS / ANIM_DURATION_MS;

// Toggle switch state with animation
struct ToggleData {
    bool isOn;
    bool isHovered;
    bool isPressed;
    float progress;    // 0.0 = off position, 1.0 = on position
    bool animating;
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

// Ease-out cubic: decelerates toward end (matches Windows 11 feel)
static float EaseOutCubic(float t) {
    float f = 1.0f - t;
    return 1.0f - f * f * f;
}

// Lerp between two COLORREFs by t (0.0→1.0)
static COLORREF LerpColor(COLORREF a, COLORREF b, float t) {
    return RGB(
        (int)(GetRValue(a) + (GetRValue(b) - GetRValue(a)) * t),
        (int)(GetGValue(a) + (GetGValue(b) - GetGValue(a)) * t),
        (int)(GetBValue(a) + (GetBValue(b) - GetBValue(a)) * t)
    );
}

// Windows 11 style toggle switch with animation support
// progress: 0.0 = fully off, 1.0 = fully on (animated between states)
void DrawToggleSwitch(HDC hdc, int x, int y, int width, int height, float progress, bool isHovered, bool isPressed) {
    const Theme& theme = GetTheme();

    Gdiplus::Graphics g(hdc);
    g.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);

    // Track color blends between off→on based on progress
    COLORREF offColor = theme.toggleOff;
    if (isHovered && progress < 0.5f) {
        offColor = RGB(
            min(255, GetRValue(offColor) + 30),
            min(255, GetGValue(offColor) + 30),
            min(255, GetBValue(offColor) + 30)
        );
    }
    COLORREF trackColor = LerpColor(offColor, theme.toggleOn, progress);

    // Draw track (pill shape)
    Gdiplus::GraphicsPath trackPath;
    trackPath.AddArc(x, y, height, height, 90, 180);
    trackPath.AddArc(x + width - height, y, height, height, 270, 180);
    trackPath.CloseFigure();

    Gdiplus::SolidBrush trackBrush(Gdiplus::Color(
        GetRValue(trackColor), GetGValue(trackColor), GetBValue(trackColor)));
    g.FillPath(&trackBrush, &trackPath);

    // Knob sizing: slightly smaller when pressed (Windows 11 squish effect)
    int knobPadding = 3;
    int baseKnobSize = height - knobPadding * 2;
    int knobSize = isPressed ? (int)(baseKnobSize * 0.85f) : baseKnobSize;
    int knobYOffset = isPressed ? (baseKnobSize - knobSize) / 2 : 0;

    // Knob position: interpolate between off (left) and on (right)
    int offX = x + knobPadding;
    int onX = x + width - baseKnobSize - knobPadding;
    int knobX = offX + (int)((onX - offX) * progress);
    int knobY = y + knobPadding + knobYOffset;

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

// Draw horizontal divider line
void DrawDivider(HDC hdc, int x, int y, int width, COLORREF color) {
    Gdiplus::Graphics g(hdc);
    g.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);

    Gdiplus::Pen pen(Gdiplus::Color(50, GetRValue(color), GetGValue(color), GetBValue(color)), 1.0f);
    g.DrawLine(&pen, x, y, x + width, y);
}

// Draw gear icon (settings)
void DrawGearIcon(HDC hdc, int x, int y, int size, COLORREF color) {
    Gdiplus::Graphics g(hdc);
    g.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);

    Gdiplus::Pen pen(Gdiplus::Color(GetRValue(color), GetGValue(color), GetBValue(color)), 1.5f);
    Gdiplus::SolidBrush brush(Gdiplus::Color(GetRValue(color), GetGValue(color), GetBValue(color)));

    float cx = x + size / 2.0f;
    float cy = y + size / 2.0f;
    float outerR = size * 0.45f;
    float innerR = size * 0.25f;
    float toothH = size * 0.12f;

    // Draw center circle
    g.DrawEllipse(&pen, cx - innerR, cy - innerR, innerR * 2, innerR * 2);

    // Draw gear teeth (8 teeth)
    int teeth = 8;
    for (int i = 0; i < teeth; i++) {
        float angle = (float)(i * 360 / teeth) * 3.14159f / 180.0f;
        float angleNext = (float)((i + 0.4f) * 360 / teeth) * 3.14159f / 180.0f;

        float x1 = cx + (outerR - toothH) * cosf(angle);
        float y1 = cy + (outerR - toothH) * sinf(angle);
        float x2 = cx + outerR * cosf(angle);
        float y2 = cy + outerR * sinf(angle);

        g.DrawLine(&pen, x1, y1, x2, y2);
    }

    // Draw outer circle
    g.DrawEllipse(&pen, cx - outerR + toothH, cy - outerR + toothH,
                  (outerR - toothH) * 2, (outerR - toothH) * 2);
}

// Draw bolt/lightning icon (about/info)
void DrawBoltIcon(HDC hdc, int x, int y, int size, COLORREF color) {
    Gdiplus::Graphics g(hdc);
    g.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);

    Gdiplus::SolidBrush brush(Gdiplus::Color(GetRValue(color), GetGValue(color), GetBValue(color)));

    // Lightning bolt shape
    Gdiplus::PointF points[7];
    float w = size * 0.5f;
    float h = size * 0.9f;
    float ox = x + size * 0.25f;
    float oy = y + size * 0.05f;

    points[0] = Gdiplus::PointF(ox + w * 0.6f, oy);           // top
    points[1] = Gdiplus::PointF(ox, oy + h * 0.5f);           // left middle
    points[2] = Gdiplus::PointF(ox + w * 0.35f, oy + h * 0.5f);
    points[3] = Gdiplus::PointF(ox + w * 0.2f, oy + h);       // bottom
    points[4] = Gdiplus::PointF(ox + w, oy + h * 0.45f);      // right middle
    points[5] = Gdiplus::PointF(ox + w * 0.65f, oy + h * 0.45f);
    points[6] = Gdiplus::PointF(ox + w * 0.6f, oy);           // back to top

    g.FillPolygon(&brush, points, 7);
}

// Draw chevron right arrow
void DrawChevronRight(HDC hdc, int x, int y, int size, COLORREF color) {
    Gdiplus::Graphics g(hdc);
    g.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);

    Gdiplus::Pen pen(Gdiplus::Color(GetRValue(color), GetGValue(color), GetBValue(color)), 1.5f);
    pen.SetLineCap(Gdiplus::LineCapRound, Gdiplus::LineCapRound, Gdiplus::DashCapRound);

    float pad = size * 0.3f;
    float cx = x + size / 2.0f;
    float cy = y + size / 2.0f;

    // Draw > shape
    g.DrawLine(&pen, cx - pad * 0.5f, cy - pad, cx + pad * 0.5f, cy);
    g.DrawLine(&pen, cx + pad * 0.5f, cy, cx - pad * 0.5f, cy + pad);
}

// Draw checkmark inside circle (for "up to date" badge)
void DrawCheckmarkCircle(HDC hdc, int x, int y, int size, COLORREF color) {
    Gdiplus::Graphics g(hdc);
    g.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);

    // Green filled circle
    Gdiplus::SolidBrush circleBrush(Gdiplus::Color(34, 197, 94));  // Green-500
    g.FillEllipse(&circleBrush, (float)x, (float)y, (float)size, (float)size);

    // White checkmark
    Gdiplus::Pen pen(Gdiplus::Color(255, 255, 255), 1.5f);
    pen.SetLineCap(Gdiplus::LineCapRound, Gdiplus::LineCapRound, Gdiplus::DashCapRound);

    float cx = x + size / 2.0f;
    float cy = y + size / 2.0f;
    float s = size * 0.25f;

    // Checkmark path
    g.DrawLine(&pen, cx - s, cy, cx - s * 0.3f, cy + s * 0.7f);
    g.DrawLine(&pen, cx - s * 0.3f, cy + s * 0.7f, cx + s, cy - s * 0.5f);
}

// Draw keycap style button (like macOS keyboard shortcuts)
void DrawKeycap(HDC hdc, int x, int y, const wchar_t* text, int fontSize, float dpi) {
    const Theme& theme = GetTheme();

    // Measure text width
    HFONT hFont = CreateFontW(
        -MulDiv(fontSize, GetDeviceCaps(hdc, LOGPIXELSY), 72),
        0, 0, 0, FW_MEDIUM, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI"
    );

    HFONT oldFont = (HFONT)SelectObject(hdc, hFont);
    SIZE textSize;
    GetTextExtentPoint32W(hdc, text, (int)wcslen(text), &textSize);
    SelectObject(hdc, oldFont);
    DeleteObject(hFont);

    int padding = Scale(8, dpi);
    int height = Scale(22, dpi);
    int width = textSize.cx + padding * 2;
    int radius = Scale(4, dpi);

    // Keycap background (subtle gray)
    COLORREF bgColor = IsDarkMode() ? RGB(60, 60, 60) : RGB(230, 230, 230);
    COLORREF borderColor = IsDarkMode() ? RGB(80, 80, 80) : RGB(200, 200, 200);

    RECT keycapRect = { x, y, x + width, y + height };
    DrawRoundedRect(hdc, keycapRect, radius, bgColor, borderColor);

    // Draw text centered
    RECT textRect = { x, y, x + width, y + height };
    DrawText(hdc, text, textRect, theme.textPrimary, fontSize, true, DT_CENTER | DT_VCENTER);
}

// Toggle switch window procedure with animation
static LRESULT CALLBACK ToggleSwitchProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    ToggleData* data = (ToggleData*)GetWindowLongPtr(hwnd, GWLP_USERDATA);

    switch (msg) {
        case WM_CREATE: {
            data = new ToggleData{ false, false, false, 0.0f, false };
            SetWindowLongPtr(hwnd, GWLP_USERDATA, (LONG_PTR)data);
            return 0;
        }

        case WM_DESTROY: {
            KillTimer(hwnd, TOGGLE_ANIM_TIMER_ID);
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

            // Draw toggle with animated progress
            float displayProgress = data ? EaseOutCubic(data->progress) : 0.0f;
            DrawToggleSwitch(memDC, 0, 0, rect.right, rect.bottom,
                displayProgress,
                data ? data->isHovered : false,
                data ? data->isPressed : false);

            BitBlt(hdc, 0, 0, rect.right, rect.bottom, memDC, 0, 0, SRCCOPY);

            SelectObject(memDC, oldBitmap);
            DeleteObject(memBitmap);
            DeleteDC(memDC);

            EndPaint(hwnd, &ps);
            return 0;
        }

        case WM_TIMER: {
            if (wParam == TOGGLE_ANIM_TIMER_ID && data && data->animating) {
                float target = data->isOn ? 1.0f : 0.0f;
                float diff = target - data->progress;

                if (fabsf(diff) < 0.01f) {
                    // Animation complete — snap to target
                    data->progress = target;
                    data->animating = false;
                    KillTimer(hwnd, TOGGLE_ANIM_TIMER_ID);
                } else {
                    // Step toward target
                    data->progress += (diff > 0 ? ANIM_STEP : -ANIM_STEP);
                    // Clamp
                    if (data->progress < 0.0f) data->progress = 0.0f;
                    if (data->progress > 1.0f) data->progress = 1.0f;
                }

                InvalidateRect(hwnd, NULL, FALSE);
            }
            return 0;
        }

        case WM_LBUTTONDOWN: {
            if (data) {
                data->isPressed = true;
                SetCapture(hwnd);
                InvalidateRect(hwnd, NULL, FALSE);
            }
            return 0;
        }

        case WM_LBUTTONUP: {
            if (data) {
                data->isPressed = false;
                ReleaseCapture();

                // Toggle state and start animation
                data->isOn = !data->isOn;
                data->animating = true;
                SetTimer(hwnd, TOGGLE_ANIM_TIMER_ID, ANIM_INTERVAL_MS, NULL);

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
        data->progress = state ? 1.0f : 0.0f;  // Snap to target (no animation on load)
        data->animating = false;
        KillTimer(hwnd, TOGGLE_ANIM_TIMER_ID);
        InvalidateRect(hwnd, NULL, FALSE);
    }
}

} // namespace ui
} // namespace gonhanh
