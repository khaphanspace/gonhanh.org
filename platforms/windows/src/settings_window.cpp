#include "settings_window.h"
#include "resource.h"
#include "settings.h"
#include "shortcuts_dialog.h"
#include "modern_ui.h"
#include "update_checker.h"
#include <windowsx.h>
#include <commctrl.h>
#include <sstream>

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
static HWND toggleAutoRestore = nullptr;
static HWND toggleAutoStart = nullptr;
static HWND togglePerApp = nullptr;
static HWND toggleSound = nullptr;
static HWND toggleModern = nullptr;
static HWND toggleCapitalize = nullptr;
static HWND toggleForeignConsonants = nullptr;

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
    } else {
        // Reset scroll position when re-showing
        if (scrollPos_ != 0) {
            ScrollContent(0);
        }
    }
    LoadSettings();
    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
    visible_ = true;

    // Trigger update check in background
    UpdateChecker::Instance().CheckAsync();

    // Set timer to refresh version badge when update check completes
    SetTimer(hwnd_, 1, 500, nullptr);  // Check every 500ms
}

void SettingsWindow::Hide() {
    if (hwnd_) {
        KillTimer(hwnd_, 1);
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

    // Create window with vertical scrollbar
    hwnd_ = CreateWindowExW(
        WS_EX_APPWINDOW,
        L"GoNhanhSettingsWindow",
        L"G\x00F5 Nhanh - C\x00E0i \x0111\x1EB7t",
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VSCROLL,
        x, y, windowWidth, windowHeight,
        NULL, NULL, GetModuleHandle(NULL), this
    );

    if (!hwnd_) return;

    CreateControls();
    UpdateScrollInfo();
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
    int rowHeight = Scale(36, dpi);       // Compact rows for toggle items
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

    // === Section 1: Input Method (5 rows - match macOS) ===
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

    toggleWShortcut = createToggleRow(L"G\x00F5 W th\x00E0nh \x01AF", IDC_CHK_W_SHORTCUT, contentX, contentWidth);
    toggleBracket = createToggleRow(L"G\x00F5 ] th\x00E0nh \x01AF, [ th\x00E0nh \x01A0", IDC_CHK_BRACKET, contentX, contentWidth);
    toggleAutoRestore = createToggleRow(L"T\x1EF1 kh\x00F4i ph\x1EE5" L"c ti\x1EBFng Anh", IDC_CHK_AUTORESTORE, contentX, contentWidth);

    // Divider after section 1
    dividerY1_ = y;
    y += Scale(2, dpi);  // Thin divider space

    // === Section 2: Shortcuts (2 rows with subtitle - taller than toggle rows) ===
    int subtitleRowHeight = Scale(44, dpi);
    section2Y_ = y;
    y += subtitleRowHeight * 2;

    // Divider after section 2 (painted by PaintContent)
    dividerY2_ = y;
    y += Scale(2, dpi);

    // === Section 3: Typing Rules (2 rows) ===
    toggleModern = createToggleRow(L"\x0110\x1EB7t d\x1EA5u ki\x1EC3u m\x1EDBi", IDC_CHK_MODERN, contentX, contentWidth);
    toggleForeignConsonants = createToggleRow(L"Cho ph\x00E9p ph\x1EE5 \x00E2m ngo\x1EA1i", IDC_CHK_FOREIGN, contentX, contentWidth);

    // Divider after section 3
    dividerY3_ = y;
    y += Scale(2, dpi);

    // === Section 4: Advanced (4 rows) ===
    toggleCapitalize = createToggleRow(L"T\x1EF1 vi\x1EBFt hoa \x0111\x1EA7u c\x00E2u", IDC_CHK_CAPITALIZE, contentX, contentWidth);
    togglePerApp = createToggleRow(L"Nh\x1EDB tr\x1EA1ng th\x00E1i theo app", IDC_CHK_PERAPP, contentX, contentWidth);
    toggleSound = createToggleRow(L"\x00C2m thanh khi b\x1EADt/t\x1EAFt", IDC_CHK_SOUND, contentX, contentWidth);
    toggleAutoStart = createToggleRow(L"Kh\x1EDFi \x0111\x1ED9ng c\x00F9ng h\x1EC7 th\x1ED1ng", IDC_CHK_AUTOSTART, contentX, contentWidth);

    // Store total content height for scrolling
    contentHeight_ = y + Scale(20, dpi);

    // Collect all child controls for scrolling
    contentControls_.clear();
    EnumChildWindows(hwnd_, [](HWND child, LPARAM lParam) -> BOOL {
        auto* controls = reinterpret_cast<std::vector<HWND>*>(lParam);
        controls->push_back(child);
        return TRUE;
    }, reinterpret_cast<LPARAM>(&contentControls_));
}

void SettingsWindow::UpdateScrollInfo() {
    RECT clientRect;
    GetClientRect(hwnd_, &clientRect);
    int viewHeight = clientRect.bottom;

    SCROLLINFO si = { sizeof(si) };
    si.fMask = SIF_RANGE | SIF_PAGE | SIF_POS;
    si.nMin = 0;
    si.nMax = contentHeight_;
    si.nPage = viewHeight;
    si.nPos = scrollPos_;
    SetScrollInfo(hwnd_, SB_VERT, &si, TRUE);

    // Show/hide scrollbar based on content
    ShowScrollBar(hwnd_, SB_VERT, contentHeight_ > viewHeight);
}

void SettingsWindow::ScrollContent(int newPos) {
    RECT clientRect;
    GetClientRect(hwnd_, &clientRect);
    int viewHeight = clientRect.bottom;
    int maxScroll = max(0, contentHeight_ - viewHeight);

    // Clamp scroll position
    newPos = max(0, min(newPos, maxScroll));
    if (newPos == scrollPos_) return;

    int delta = scrollPos_ - newPos;
    scrollPos_ = newPos;

    // Move all child controls
    for (HWND child : contentControls_) {
        RECT rc;
        GetWindowRect(child, &rc);
        MapWindowPoints(HWND_DESKTOP, hwnd_, (LPPOINT)&rc, 2);
        SetWindowPos(child, NULL, rc.left, rc.top + delta, 0, 0,
                     SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
    }

    // Update custom painted positions
    section2Y_ += delta;
    dividerY1_ += delta;
    dividerY2_ += delta;
    dividerY3_ += delta;

    // Update scrollbar
    SCROLLINFO si = { sizeof(si) };
    si.fMask = SIF_POS;
    si.nPos = scrollPos_;
    SetScrollInfo(hwnd_, SB_VERT, &si, TRUE);

    // Repaint
    InvalidateRect(hwnd_, NULL, TRUE);
}

void SettingsWindow::LoadSettings() {
    auto& settings = Settings::Instance();

    SetToggleState(toggleEnabled, settings.enabled);
    ComboBox_SetCurSel(cmbMethod_, settings.method);
    SetToggleState(toggleWShortcut, !settings.skipWShortcut);
    SetToggleState(toggleBracket, settings.bracketShortcut);
    SetToggleState(toggleAutoRestore, settings.autoRestore);
    SetToggleState(toggleModern, settings.modernTone);
    SetToggleState(toggleForeignConsonants, settings.allowForeignConsonants);
    SetToggleState(toggleCapitalize, settings.autoCapitalize);
    SetToggleState(togglePerApp, settings.perApp);
    SetToggleState(toggleSound, settings.sound);
    SetToggleState(toggleAutoStart, settings.autoStart);
}

void SettingsWindow::SaveSettings() {
    auto& settings = Settings::Instance();

    settings.enabled = GetToggleState(toggleEnabled);
    settings.method = static_cast<uint8_t>(ComboBox_GetCurSel(cmbMethod_));
    settings.skipWShortcut = !GetToggleState(toggleWShortcut);
    settings.bracketShortcut = GetToggleState(toggleBracket);
    settings.autoRestore = GetToggleState(toggleAutoRestore);
    settings.modernTone = GetToggleState(toggleModern);
    settings.allowForeignConsonants = GetToggleState(toggleForeignConsonants);
    settings.autoCapitalize = GetToggleState(toggleCapitalize);
    settings.perApp = GetToggleState(togglePerApp);
    settings.sound = GetToggleState(toggleSound);
    settings.autoStart = GetToggleState(toggleAutoStart);

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

    // Version with update badge
    int versionY = logoY + logoSize + Scale(36, dpi);
    auto& checker = UpdateChecker::Instance();
    UpdateStatus status = checker.GetStatus();

    // Build version string: "v1.0.112"
    std::wstring versionStr = L"v";
    versionStr += GetAppVersion();

    if (status == UpdateStatus::UpToDate) {
        // Show version + green checkmark + "Mới nhất"
        int checkmarkSize = Scale(12, dpi);
        int totalWidth = Scale(100, dpi);  // Approximate width

        // Measure version text width
        HFONT hFont = CreateFontW(-MulDiv(10, GetDeviceCaps(hdc, LOGPIXELSY), 72),
            0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY, DEFAULT_PITCH, L"Segoe UI");
        HFONT oldFont = (HFONT)SelectObject(hdc, hFont);
        SIZE verSize;
        GetTextExtentPoint32W(hdc, versionStr.c_str(), (int)versionStr.length(), &verSize);
        SelectObject(hdc, oldFont);
        DeleteObject(hFont);

        int startX = (sidebarWidth - verSize.cx - checkmarkSize - Scale(50, dpi)) / 2;

        // Draw version text
        RECT verRect = { startX, versionY, startX + verSize.cx + Scale(4, dpi), versionY + Scale(16, dpi) };
        DrawText(hdc, versionStr.c_str(), verRect, theme.textTertiary, 10, false, DT_LEFT | DT_VCENTER);

        // Draw green checkmark circle
        int checkX = startX + verSize.cx + Scale(6, dpi);
        int checkY = versionY + Scale(2, dpi);
        DrawCheckmarkCircle(hdc, checkX, checkY, checkmarkSize, RGB(34, 197, 94));

        // Draw "Mới nhất" text
        RECT badgeRect = { checkX + checkmarkSize + Scale(4, dpi), versionY,
                          sidebarWidth, versionY + Scale(16, dpi) };
        DrawText(hdc, L"M\x1EDBi nh\x1EA5t", badgeRect, RGB(34, 197, 94), 10, false, DT_LEFT | DT_VCENTER);
    } else {
        // Just show version centered
        RECT versionRect = { 0, versionY, sidebarWidth, versionY + Scale(16, dpi) };
        DrawText(hdc, versionStr.c_str(), versionRect, theme.textTertiary, 10, false, DT_CENTER | DT_VCENTER);
    }

    // Navigation tabs at BOTTOM (like macOS)
    int tabHeight = Scale(36, dpi);
    int tabPadding = Scale(12, dpi);
    int tabY = clientRect.bottom - Scale(100, dpi);
    int iconSize = Scale(16, dpi);
    int iconPadding = Scale(8, dpi);

    // "Cài đặt" tab (Settings) with gear icon
    RECT tabSettingsRect = { tabPadding, tabY, sidebarWidth - tabPadding, tabY + tabHeight };
    COLORREF tabColor = (currentTab == TAB_SETTINGS) ? theme.textPrimary : theme.textSecondary;

    if (currentTab == TAB_SETTINGS) {
        // Highlight bar on left
        RECT barRect = { tabPadding, tabY + Scale(8, dpi), tabPadding + Scale(3, dpi), tabY + tabHeight - Scale(8, dpi) };
        DrawRoundedRect(hdc, barRect, Scale(2, dpi), RGB(0, 120, 212), RGB(0, 120, 212));
    }

    // Draw gear icon
    int iconX = tabPadding + Scale(16, dpi);
    int iconY = tabY + (tabHeight - iconSize) / 2;
    DrawGearIcon(hdc, iconX, iconY, iconSize, tabColor);

    // Draw tab text with offset for icon
    RECT textSettingsRect = { iconX + iconSize + iconPadding, tabY, sidebarWidth - tabPadding, tabY + tabHeight };
    DrawText(hdc, L"C\x00E0i \x0111\x1EB7t", textSettingsRect, tabColor, 12,
             currentTab == TAB_SETTINGS, DT_LEFT | DT_VCENTER);

    // "Giới thiệu" tab (About) with bolt icon
    tabY += tabHeight + Scale(4, dpi);
    RECT tabAboutRect = { tabPadding, tabY, sidebarWidth - tabPadding, tabY + tabHeight };
    tabColor = (currentTab == TAB_ABOUT) ? theme.textPrimary : theme.textSecondary;

    if (currentTab == TAB_ABOUT) {
        RECT barRect = { tabPadding, tabY + Scale(8, dpi), tabPadding + Scale(3, dpi), tabY + tabHeight - Scale(8, dpi) };
        DrawRoundedRect(hdc, barRect, Scale(2, dpi), RGB(0, 120, 212), RGB(0, 120, 212));
    }

    // Draw bolt icon
    iconY = tabY + (tabHeight - iconSize) / 2;
    DrawBoltIcon(hdc, iconX, iconY, iconSize, tabColor);

    // Draw tab text with offset for icon
    RECT textAboutRect = { iconX + iconSize + iconPadding, tabY, sidebarWidth - tabPadding, tabY + tabHeight };
    DrawText(hdc, L"Gi\x1EDBi thi\x1EC7u", textAboutRect, tabColor, 12,
             currentTab == TAB_ABOUT, DT_LEFT | DT_VCENTER);
}

void SettingsWindow::PaintContent(HDC hdc) {
    const Theme& theme = GetTheme();
    float dpi = GetDpiScale(hwnd_);

    int sidebarWidth = Scale(BASE_SIDEBAR_WIDTH, dpi);
    int contentPadding = Scale(20, dpi);
    int rowHeight = Scale(44, dpi);  // Subtitle rows are taller
    int sectionPadding = Scale(16, dpi);

    int contentX = sidebarWidth + contentPadding;
    int contentRight = Scale(BASE_WINDOW_WIDTH, dpi) - contentPadding - Scale(16, dpi);
    int contentWidth = contentRight - contentX;
    int labelX = contentX + sectionPadding;
    int dividerWidth = contentWidth - sectionPadding * 2;

    // Draw section dividers (thin lines at section boundaries)
    DrawDivider(hdc, labelX, dividerY1_, dividerWidth, theme.textSecondary);
    DrawDivider(hdc, labelX, dividerY3_, dividerWidth, theme.textSecondary);

    // Use stored section2 Y position
    int y = section2Y_;

    // === Row 1: Phím tắt bật/tắt ===
    int rowY = y;

    // Title
    RECT titleRect = { labelX, rowY + Scale(6, dpi), labelX + Scale(200, dpi), rowY + Scale(24, dpi) };
    DrawText(hdc, L"Ph\x00EDm t\x1EAFt b\x1EADt/t\x1EAFt", titleRect, theme.textPrimary, 12, false, DT_LEFT | DT_VCENTER);

    // Subtitle
    RECT subtitleRect = { labelX, rowY + Scale(24, dpi), labelX + Scale(200, dpi), rowY + Scale(40, dpi) };
    DrawText(hdc, L"Nh\x1EA5n \x0111\x1EC3 thay \x0111\x1ED5i", subtitleRect, theme.textTertiary, 10, false, DT_LEFT | DT_VCENTER);

    // Keycaps on right: "^" + "Space"
    int keycapY = rowY + Scale(12, dpi);
    int keycapX = contentX + contentWidth - sectionPadding - Scale(90, dpi);
    DrawKeycap(hdc, keycapX, keycapY, L"^", 10, dpi);
    DrawKeycap(hdc, keycapX + Scale(32, dpi), keycapY, L"Space", 10, dpi);

    // Move to next row
    y += rowHeight;

    // Divider
    DrawDivider(hdc, labelX, y - Scale(2, dpi), contentWidth - sectionPadding * 2, theme.textSecondary);

    // === Row 2: Bảng gõ tắt ===
    rowY = y;

    // Title
    titleRect = { labelX, rowY + Scale(6, dpi), labelX + Scale(200, dpi), rowY + Scale(24, dpi) };
    DrawText(hdc, L"B\x1EA3ng g\x00F5 t\x1EAFt", titleRect, theme.textPrimary, 12, false, DT_LEFT | DT_VCENTER);

    // Subtitle: "X/Y đang bật"
    auto& settings = Settings::Instance();
    auto [enabled, total] = settings.GetShortcutsCount();
    std::wstringstream ss;
    ss << enabled << L"/" << total << L" \x0111" L"ang b\x1EADt";
    std::wstring countStr = ss.str();

    subtitleRect = { labelX, rowY + Scale(24, dpi), labelX + Scale(200, dpi), rowY + Scale(40, dpi) };
    DrawText(hdc, countStr.c_str(), subtitleRect, theme.textTertiary, 10, false, DT_LEFT | DT_VCENTER);

    // Chevron on right
    int chevronSize = Scale(14, dpi);
    int chevronX = contentX + contentWidth - sectionPadding - chevronSize;
    int chevronY = rowY + Scale(14, dpi);
    DrawChevronRight(hdc, chevronX, chevronY, chevronSize, theme.textTertiary);

    // Store rect for click detection
    shortcutsRowRect_ = {
        labelX, rowY,
        contentX + contentWidth - sectionPadding,
        rowY + rowHeight
    };

    // Divider after row
    y += rowHeight;
    DrawDivider(hdc, labelX, y - Scale(2, dpi), contentWidth - sectionPadding * 2, theme.textSecondary);
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
            if (window) {
                window->PaintSidebar(hdc);
                window->PaintContent(hdc);
            }
            EndPaint(hwnd, &ps);
            return 0;
        }

        case WM_CTLCOLORSTATIC: {
            HDC hdcStatic = (HDC)wParam;
            const Theme& theme = GetTheme();
            SetTextColor(hdcStatic, theme.textPrimary);
            SetBkColor(hdcStatic, theme.windowBg);
            // Recreate brush when theme changes (dark/light toggle)
            static HBRUSH bgBrush = nullptr;
            static COLORREF cachedBgColor = 0;
            if (!bgBrush || cachedBgColor != theme.windowBg) {
                if (bgBrush) DeleteObject(bgBrush);
                bgBrush = CreateSolidBrush(theme.windowBg);
                cachedBgColor = theme.windowBg;
            }
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

        case WM_LBUTTONDOWN: {
            if (window) {
                POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
                // Check if clicked on shortcuts row
                if (PtInRect(&window->shortcutsRowRect_, pt)) {
                    ShortcutsDialog::Instance().Show();
                    // Refresh count display after shortcuts dialog closes
                    InvalidateRect(hwnd, NULL, FALSE);
                }
            }
            return 0;
        }

        case WM_SETCURSOR: {
            if (window && LOWORD(lParam) == HTCLIENT) {
                POINT pt;
                GetCursorPos(&pt);
                ScreenToClient(hwnd, &pt);
                if (PtInRect(&window->shortcutsRowRect_, pt)) {
                    SetCursor(LoadCursor(NULL, IDC_HAND));
                    return TRUE;
                }
            }
            break;  // Let DefWindowProc handle other cases
        }

        case WM_COMMAND: {
            WORD id = LOWORD(wParam);
            WORD code = HIWORD(wParam);

            // Combobox change
            if (code == CBN_SELCHANGE) {
                if (window) window->SaveSettings();
            }
            return 0;
        }

        case WM_VSCROLL: {
            if (window) {
                SCROLLINFO si = { sizeof(si) };
                si.fMask = SIF_ALL;
                GetScrollInfo(hwnd, SB_VERT, &si);

                int newPos = si.nPos;
                switch (LOWORD(wParam)) {
                    case SB_LINEUP:        newPos -= 30; break;
                    case SB_LINEDOWN:      newPos += 30; break;
                    case SB_PAGEUP:        newPos -= si.nPage; break;
                    case SB_PAGEDOWN:      newPos += si.nPage; break;
                    case SB_THUMBTRACK:    newPos = si.nTrackPos; break;
                    case SB_THUMBPOSITION: newPos = si.nTrackPos; break;
                }
                window->ScrollContent(newPos);
            }
            return 0;
        }

        case WM_MOUSEWHEEL: {
            if (window) {
                int delta = GET_WHEEL_DELTA_WPARAM(wParam);
                int scrollAmount = -delta / 4;  // Smooth scrolling
                window->ScrollContent(window->scrollPos_ + scrollAmount);
            }
            return 0;
        }

        case WM_TIMER: {
            if (wParam == 1) {
                // Check if update status changed from Checking to something else
                UpdateStatus status = UpdateChecker::Instance().GetStatus();
                if (status != UpdateStatus::Checking && status != UpdateStatus::Idle) {
                    // Kill timer and repaint sidebar to show update badge
                    KillTimer(hwnd, 1);
                    InvalidateRect(hwnd, NULL, FALSE);
                }
            }
            return 0;
        }

        case WM_CLOSE:
            if (window) window->Hide();
            return 0;

        case WM_DESTROY:
            KillTimer(hwnd, 1);
            ShutdownGdiPlus();
            return 0;
    }

    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

} // namespace gonhanh
