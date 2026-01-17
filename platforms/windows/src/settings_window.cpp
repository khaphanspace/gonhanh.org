#include "settings_window.h"
#include "resource.h"
#include "settings.h"
#include "shortcuts_dialog.h"
#include "modern_ui.h"
#include <windowsx.h>
#include <commctrl.h>

#pragma comment(lib, "comctl32.lib")

namespace gonhanh {

using namespace ui;

// Base window dimensions (at 100% / 96 DPI)
static const int BASE_WINDOW_WIDTH = 680;
static const int BASE_WINDOW_HEIGHT = 520;
static const int BASE_SIDEBAR_WIDTH = 180;
static const int BASE_CONTENT_PADDING = 24;
static const int BASE_ROW_HEIGHT = 36;
static const int BASE_TOGGLE_WIDTH = 44;
static const int BASE_TOGGLE_HEIGHT = 20;

// Sidebar tab IDs
static const int TAB_SETTINGS = 0;
static const int TAB_ABOUT = 1;
static int currentTab = TAB_SETTINGS;

// Toggle switch handles
static HWND toggleEnabled = nullptr;
static HWND toggleWShortcut = nullptr;
static HWND toggleBracket = nullptr;
static HWND toggleAutoStart = nullptr;
static HWND togglePerApp = nullptr;
static HWND toggleAutoRestore = nullptr;
static HWND toggleSound = nullptr;
static HWND toggleModern = nullptr;
static HWND toggleCapitalize = nullptr;

SettingsWindow& SettingsWindow::Instance() {
    static SettingsWindow instance;
    return instance;
}

SettingsWindow::~SettingsWindow() {
    if (hwnd_) {
        DestroyWindow(hwnd_);
    }
}

void SettingsWindow::Show() {
    if (!hwnd_) {
        Create();
    }
    LoadSettings();
    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
    visible_ = true;
}

void SettingsWindow::Hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
        visible_ = false;
    }
}

void SettingsWindow::Create() {
    // Initialize common controls
    INITCOMMONCONTROLSEX icex = { sizeof(icex), ICC_STANDARD_CLASSES };
    InitCommonControlsEx(&icex);

    HINSTANCE hInst = GetModuleHandle(NULL);
    InitGdiPlus();
    RegisterToggleSwitchClass(hInst);

    // Register window class
    WNDCLASSEXW wc = { sizeof(wc) };
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInst;
    wc.lpszClassName = L"GoNhanhSettingsWindow";
    wc.hCursor = LoadCursor(NULL, IDC_ARROW);
    wc.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
    wc.hIcon = LoadIconW(hInst, MAKEINTRESOURCEW(IDI_APP_LOGO));
    wc.hIconSm = LoadIconW(hInst, MAKEINTRESOURCEW(IDI_APP_LOGO));
    RegisterClassExW(&wc);

    // Get DPI scale from primary monitor
    HDC hdc = GetDC(NULL);
    float dpiScale = GetDeviceCaps(hdc, LOGPIXELSX) / 96.0f;
    ReleaseDC(NULL, hdc);

    int windowWidth = Scale(BASE_WINDOW_WIDTH, dpiScale);
    int windowHeight = Scale(BASE_WINDOW_HEIGHT, dpiScale);

    // Calculate center position
    int screenWidth = GetSystemMetrics(SM_CXSCREEN);
    int screenHeight = GetSystemMetrics(SM_CYSCREEN);
    int x = (screenWidth - windowWidth) / 2;
    int y = (screenHeight - windowHeight) / 2;

    // Create window
    hwnd_ = CreateWindowExW(
        WS_EX_APPWINDOW,
        L"GoNhanhSettingsWindow",
        L"G\x00F5 Nhanh - C\x00E0i \x0111\x1EB7t",
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
        x, y, windowWidth, windowHeight,
        NULL, NULL, GetModuleHandle(NULL), this
    );

    if (!hwnd_) return;

    CreateControls();
}

void SettingsWindow::CreateControls() {
    HINSTANCE hInst = GetModuleHandle(NULL);
    float dpi = GetDpiScale(hwnd_);

    // Create DPI-scaled font
    int fontSize = Scale(13, dpi);
    HFONT hFont = CreateFontW(-fontSize, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH, L"Segoe UI");

    int sidebarWidth = Scale(BASE_SIDEBAR_WIDTH, dpi);
    int contentPadding = Scale(20, dpi);
    int rowHeight = Scale(40, dpi);
    int sectionPadding = Scale(16, dpi);

    // Content area dimensions
    int contentX = sidebarWidth + contentPadding;
    int contentRight = Scale(BASE_WINDOW_WIDTH, dpi) - contentPadding - Scale(16, dpi);
    int contentWidth = contentRight - contentX;
    int toggleWidth = Scale(44, dpi);

    int y = Scale(16, dpi);

    // Helper lambda - creates label + toggle, toggle aligned right
    auto createToggleRow = [&](const wchar_t* text, int id, int sectionX, int sectionWidth) -> HWND {
        int labelX = sectionX + sectionPadding;
        int toggleX = sectionX + sectionWidth - sectionPadding - toggleWidth;
        int labelWidth = toggleX - labelX - Scale(10, dpi);

        HWND lbl = CreateWindowExW(0, L"STATIC", text,
            WS_CHILD | WS_VISIBLE | SS_LEFT | SS_CENTERIMAGE,
            labelX, y, labelWidth, rowHeight,
            hwnd_, NULL, hInst, NULL);
        SendMessage(lbl, WM_SETFONT, (WPARAM)hFont, TRUE);

        HWND toggle = CreateToggleSwitch(hwnd_, toggleX, y + (rowHeight - Scale(20, dpi)) / 2, id, false);
        y += rowHeight;
        return toggle;
    };

    // === Section 1: Input Method (4 rows) ===
    int section1Y = y;
    toggleEnabled = createToggleRow(L"B\x1ED9 g\x00F5 ti\x1EBFng Vi\x1EC7t", IDC_CHK_ENABLED, contentX, contentWidth);

    // "Kiểu gõ" row with combobox on right
    {
        int labelX = contentX + sectionPadding;
        int comboWidth = Scale(100, dpi);
        int comboX = contentX + contentWidth - sectionPadding - comboWidth;

        HWND lbl = CreateWindowExW(0, L"STATIC", L"Ki\x1EC3u g\x00F5",
            WS_CHILD | WS_VISIBLE | SS_LEFT | SS_CENTERIMAGE,
            labelX, y, Scale(150, dpi), rowHeight,
            hwnd_, NULL, hInst, NULL);
        SendMessage(lbl, WM_SETFONT, (WPARAM)hFont, TRUE);

        cmbMethod_ = CreateWindowExW(0, L"COMBOBOX", NULL,
            WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST | WS_VSCROLL,
            comboX, y + (rowHeight - Scale(24, dpi)) / 2, comboWidth, Scale(200, dpi),
            hwnd_, (HMENU)IDC_CMB_METHOD, hInst, NULL);
        SendMessage(cmbMethod_, WM_SETFONT, (WPARAM)hFont, TRUE);
        ComboBox_AddString(cmbMethod_, L"Telex");
        ComboBox_AddString(cmbMethod_, L"VNI");
        y += rowHeight;
    }

    toggleWShortcut = createToggleRow(L"G\x00F5 W th\x00E0nh \x01AF \x1EDF \x0111\x1EA7u t\x1EEB", IDC_CHK_W_SHORTCUT, contentX, contentWidth);
    toggleBracket = createToggleRow(L"G\x00F5 ] th\x00E0nh \x01AF, [ th\x00E0nh \x01A0", IDC_CHK_BRACKET, contentX, contentWidth);
    int section1Height = y - section1Y;
    y += Scale(12, dpi);

    // === Section 2: Shortcuts (2 rows) ===
    int section2Y = y;
    {
        int labelX = contentX + sectionPadding;
        int valueX = contentX + contentWidth - sectionPadding - Scale(100, dpi);

        HWND lbl = CreateWindowExW(0, L"STATIC", L"Ph\x00EDm t\x1EAFt b\x1EADt/t\x1EAFt",
            WS_CHILD | WS_VISIBLE | SS_LEFT | SS_CENTERIMAGE,
            labelX, y, Scale(200, dpi), rowHeight,
            hwnd_, NULL, hInst, NULL);
        SendMessage(lbl, WM_SETFONT, (WPARAM)hFont, TRUE);

        HWND lblHotkey = CreateWindowExW(0, L"STATIC", L"Ctrl + Space",
            WS_CHILD | WS_VISIBLE | SS_RIGHT | SS_CENTERIMAGE,
            valueX, y, Scale(100, dpi), rowHeight,
            hwnd_, (HMENU)IDC_HOTKEY, hInst, NULL);
        SendMessage(lblHotkey, WM_SETFONT, (WPARAM)hFont, TRUE);
        y += rowHeight;
    }
    {
        int labelX = contentX + sectionPadding;
        int btnWidth = Scale(60, dpi);
        int btnX = contentX + contentWidth - sectionPadding - btnWidth;

        HWND lbl = CreateWindowExW(0, L"STATIC", L"B\x1EA3ng g\x00F5 t\x1EAFt",
            WS_CHILD | WS_VISIBLE | SS_LEFT | SS_CENTERIMAGE,
            labelX, y, Scale(150, dpi), rowHeight,
            hwnd_, NULL, hInst, NULL);
        SendMessage(lbl, WM_SETFONT, (WPARAM)hFont, TRUE);

        btnShortcuts_ = CreateWindowExW(0, L"BUTTON", L"M\x1EDF...",
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            btnX, y + (rowHeight - Scale(26, dpi)) / 2, btnWidth, Scale(26, dpi),
            hwnd_, (HMENU)IDC_BTN_SHORTCUTS, hInst, NULL);
        SendMessage(btnShortcuts_, WM_SETFONT, (WPARAM)hFont, TRUE);
        y += rowHeight;
    }
    int section2Height = y - section2Y;
    y += Scale(12, dpi);

    // === Section 3: Behavior (3 rows) ===
    int section3Y = y;
    toggleAutoStart = createToggleRow(L"Kh\x1EDFi \x0111\x1ED9ng c\x00F9ng h\x1EC7 th\x1ED1ng", IDC_CHK_AUTOSTART, contentX, contentWidth);
    togglePerApp = createToggleRow(L"T\x1EF1 chuy\x1EC3n ch\x1EBF \x0111\x1ED9 theo \x1EE9ng d\x1EE5ng", IDC_CHK_PERAPP, contentX, contentWidth);
    toggleAutoRestore = createToggleRow(L"T\x1EF1 kh\x00F4i ph\x1EE5" L"c t\x1EEB ti\x1EBFng Anh (Beta)", IDC_CHK_AUTORESTORE, contentX, contentWidth);
    int section3Height = y - section3Y;
    y += Scale(12, dpi);

    // === Section 4: Advanced (3 rows) ===
    int section4Y = y;
    toggleSound = createToggleRow(L"\x00C2m thanh chuy\x1EC3n ng\x00F4n ng\x1EEF", IDC_CHK_SOUND, contentX, contentWidth);
    toggleModern = createToggleRow(L"\x0110\x1EB7t d\x1EA5u ki\x1EC3u m\x1EDBi (o\x00E0, u\x00FD)", IDC_CHK_MODERN, contentX, contentWidth);
    toggleCapitalize = createToggleRow(L"T\x1EF1 vi\x1EBFt hoa \x0111\x1EA7u c\x00E2u", IDC_CHK_CAPITALIZE, contentX, contentWidth);
}

void SettingsWindow::LoadSettings() {
    auto& settings = Settings::Instance();

    SetToggleState(toggleEnabled, settings.enabled);
    ComboBox_SetCurSel(cmbMethod_, settings.method);
    SetToggleState(toggleWShortcut, !settings.skipWShortcut);
    SetToggleState(toggleBracket, settings.bracketShortcut);
    SetToggleState(toggleAutoStart, settings.autoStart);
    SetToggleState(togglePerApp, settings.perApp);
    SetToggleState(toggleAutoRestore, settings.autoRestore);
    SetToggleState(toggleSound, settings.sound);
    SetToggleState(toggleModern, settings.modernTone);
    SetToggleState(toggleCapitalize, settings.autoCapitalize);
}

void SettingsWindow::SaveSettings() {
    auto& settings = Settings::Instance();

    settings.enabled = GetToggleState(toggleEnabled);
    settings.method = static_cast<uint8_t>(ComboBox_GetCurSel(cmbMethod_));
    settings.skipWShortcut = !GetToggleState(toggleWShortcut);
    settings.bracketShortcut = GetToggleState(toggleBracket);
    settings.autoStart = GetToggleState(toggleAutoStart);
    settings.perApp = GetToggleState(togglePerApp);
    settings.autoRestore = GetToggleState(toggleAutoRestore);
    settings.sound = GetToggleState(toggleSound);
    settings.modernTone = GetToggleState(toggleModern);
    settings.autoCapitalize = GetToggleState(toggleCapitalize);

    settings.Save();
    settings.ApplyToEngine();
}

void SettingsWindow::PaintSidebar(HDC hdc) {
    const Theme& theme = GetTheme();
    float dpi = GetDpiScale(hwnd_);

    RECT clientRect;
    GetClientRect(hwnd_, &clientRect);

    int sidebarWidth = Scale(BASE_SIDEBAR_WIDTH, dpi);

    // Fill entire window with content background first
    HBRUSH contentBrush = CreateSolidBrush(theme.windowBg);
    FillRect(hdc, &clientRect, contentBrush);
    DeleteObject(contentBrush);

    // Sidebar background
    RECT sidebarRect = { 0, 0, sidebarWidth, clientRect.bottom };
    HBRUSH sidebarBrush = CreateSolidBrush(theme.sidebarBg);
    FillRect(hdc, &sidebarRect, sidebarBrush);
    DeleteObject(sidebarBrush);

    // Logo from PNG resource
    int logoSize = Scale(72, dpi);
    int logoX = (sidebarWidth - logoSize) / 2;
    int logoY = Scale(30, dpi);
    DrawPngFromResource(hdc, IDR_LOGO_PNG, logoX, logoY, logoSize, logoSize);

    // App name (font size in points, not scaled - DrawText handles DPI)
    RECT nameRect = { 0, logoY + logoSize + Scale(12, dpi), sidebarWidth, logoY + logoSize + Scale(34, dpi) };
    DrawText(hdc, L"G\x00F5 Nhanh", nameRect, theme.textPrimary, 15, true, DT_CENTER | DT_VCENTER);

    // Version
    RECT versionRect = { 0, logoY + logoSize + Scale(36, dpi), sidebarWidth, logoY + logoSize + Scale(52, dpi) };
    DrawText(hdc, L"v1.0.107", versionRect, theme.textTertiary, 10, false, DT_CENTER | DT_VCENTER);

    // Navigation tabs at BOTTOM (like macOS)
    int tabHeight = Scale(36, dpi);
    int tabPadding = Scale(12, dpi);
    int tabY = clientRect.bottom - Scale(100, dpi);

    // "Cài đặt" tab (Settings) - font size in points (not scaled)
    RECT tabSettingsRect = { tabPadding, tabY, sidebarWidth - tabPadding, tabY + tabHeight };
    if (currentTab == TAB_SETTINGS) {
        DrawRoundedRect(hdc, tabSettingsRect, Scale(6, dpi), theme.sidebarBg, theme.sidebarBg);
        // Highlight bar on left
        RECT barRect = { tabPadding, tabY + Scale(8, dpi), tabPadding + Scale(3, dpi), tabY + tabHeight - Scale(8, dpi) };
        DrawRoundedRect(hdc, barRect, Scale(2, dpi), RGB(0, 120, 212), RGB(0, 120, 212));
        DrawText(hdc, L"C\x00E0i \x0111\x1EB7t", tabSettingsRect, theme.textPrimary, 12, true, DT_CENTER | DT_VCENTER);
    } else {
        DrawText(hdc, L"C\x00E0i \x0111\x1EB7t", tabSettingsRect, theme.textSecondary, 12, false, DT_CENTER | DT_VCENTER);
    }

    // "Giới thiệu" tab (About)
    tabY += tabHeight + Scale(4, dpi);
    RECT tabAboutRect = { tabPadding, tabY, sidebarWidth - tabPadding, tabY + tabHeight };
    if (currentTab == TAB_ABOUT) {
        DrawRoundedRect(hdc, tabAboutRect, Scale(6, dpi), theme.sidebarBg, theme.sidebarBg);
        RECT barRect = { tabPadding, tabY + Scale(8, dpi), tabPadding + Scale(3, dpi), tabY + tabHeight - Scale(8, dpi) };
        DrawRoundedRect(hdc, barRect, Scale(2, dpi), RGB(0, 120, 212), RGB(0, 120, 212));
        DrawText(hdc, L"Gi\x1EDBi thi\x1EC7u", tabAboutRect, theme.textPrimary, 12, true, DT_CENTER | DT_VCENTER);
    } else {
        DrawText(hdc, L"Gi\x1EDBi thi\x1EC7u", tabAboutRect, theme.textSecondary, 12, false, DT_CENTER | DT_VCENTER);
    }
}

LRESULT CALLBACK SettingsWindow::WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    SettingsWindow* window = nullptr;

    if (msg == WM_CREATE) {
        CREATESTRUCT* cs = (CREATESTRUCT*)lParam;
        window = (SettingsWindow*)cs->lpCreateParams;
        SetWindowLongPtr(hwnd, GWLP_USERDATA, (LONG_PTR)window);
    } else {
        window = (SettingsWindow*)GetWindowLongPtr(hwnd, GWLP_USERDATA);
    }

    switch (msg) {
        case WM_PAINT: {
            PAINTSTRUCT ps;
            HDC hdc = BeginPaint(hwnd, &ps);
            if (window) window->PaintSidebar(hdc);
            EndPaint(hwnd, &ps);
            return 0;
        }

        case WM_CTLCOLORSTATIC: {
            HDC hdcStatic = (HDC)wParam;
            const Theme& theme = GetTheme();
            SetTextColor(hdcStatic, theme.textPrimary);
            SetBkColor(hdcStatic, theme.windowBg);
            static HBRUSH bgBrush = nullptr;
            if (!bgBrush) bgBrush = CreateSolidBrush(theme.windowBg);
            return (LRESULT)bgBrush;
        }

        case WM_CTLCOLORBTN: {
            return (LRESULT)GetStockObject(NULL_BRUSH);
        }

        case WM_TOGGLE_CHANGED: {
            // Toggle switch was clicked, save settings
            if (window) window->SaveSettings();
            return 0;
        }

        case WM_COMMAND: {
            WORD id = LOWORD(wParam);
            WORD code = HIWORD(wParam);

            // Combobox change
            if (code == CBN_SELCHANGE) {
                if (window) window->SaveSettings();
            }

            // Button click
            if (id == IDC_BTN_SHORTCUTS && code == BN_CLICKED) {
                ShortcutsDialog::Instance().Show();
            }
            return 0;
        }

        case WM_CLOSE:
            if (window) window->Hide();
            return 0;

        case WM_DESTROY:
            ShutdownGdiPlus();
            return 0;
    }

    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

INT_PTR CALLBACK SettingsWindow::DialogProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    return FALSE;
}

} // namespace gonhanh
