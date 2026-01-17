#include "settings_window.h"
#include "resource.h"
#include "settings.h"
#include "shortcuts_dialog.h"
#include "modern_ui.h"
#include <windowsx.h>
#include <shellapi.h>

namespace gonhanh {

using namespace ui;

// Window dimensions (adjusted for Vietnamese text + 4 cards)
static const int WINDOW_WIDTH = 880;
static const int WINDOW_HEIGHT = 640;
static const int SIDEBAR_WIDTH = 220;
static const int CONTENT_PADDING = 20;
static const int CARD_RADIUS = 10;
static const int ROW_HEIGHT = 44;
static const int TOGGLE_WIDTH = 51;
static const int TOGGLE_HEIGHT = 31;

// Navigation
enum class NavPage { Settings, About };
static NavPage currentPage = NavPage::Settings;

// Control IDs for toggles
enum ToggleID {
    IDT_ENABLED = 1001,
    IDT_W_SHORTCUT,
    IDT_BRACKET,
    IDT_AUTOSTART,
    IDT_PERAPP,
    IDT_AUTORESTORE,
    IDT_SOUND,
    IDT_MODERN_TONE,
    IDT_CAPITALIZE,
};

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
    // Register custom controls
    RegisterCustomControls(GetModuleHandle(NULL));
    InitGdiPlus();

    // Register window class
    WNDCLASSEXW wc = { sizeof(wc) };
    wc.lpfnWndProc = WndProc;
    wc.hInstance = GetModuleHandle(NULL);
    wc.lpszClassName = L"GoNhanhSettingsWindow";
    wc.hCursor = LoadCursor(NULL, IDC_ARROW);
    wc.hbrBackground = NULL;  // We handle painting
    RegisterClassExW(&wc);

    // Calculate center position
    int screenWidth = GetSystemMetrics(SM_CXSCREEN);
    int screenHeight = GetSystemMetrics(SM_CYSCREEN);
    int x = (screenWidth - WINDOW_WIDTH) / 2;
    int y = (screenHeight - WINDOW_HEIGHT) / 2;

    // Create window with no title bar
    hwnd_ = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_APPWINDOW,
        L"GoNhanhSettingsWindow",
        L"Gõ Nhanh - Cài đặt",
        WS_POPUP | WS_MINIMIZEBOX | WS_SYSMENU,
        x, y, WINDOW_WIDTH, WINDOW_HEIGHT,
        NULL, NULL, GetModuleHandle(NULL), this
    );

    if (!hwnd_) return;

    // Enable rounded corners on Windows 11
    DWMNCRENDERINGPOLICY policy = DWMNCRP_ENABLED;
    DwmSetWindowAttribute(hwnd_, DWMWA_NCRENDERING_POLICY, &policy, sizeof(policy));

    // Set corner preference to rounded
    DWM_WINDOW_CORNER_PREFERENCE corner = DWMWCP_ROUND;
    DwmSetWindowAttribute(hwnd_, DWMWA_WINDOW_CORNER_PREFERENCE, &corner, sizeof(corner));

    // Make window opaque
    SetLayeredWindowAttributes(hwnd_, 0, 255, LWA_ALPHA);

    CreateControls();
}

void SettingsWindow::CreateControls() {
    const int contentX = SIDEBAR_WIDTH + CONTENT_PADDING;
    const int contentWidth = WINDOW_WIDTH - SIDEBAR_WIDTH - CONTENT_PADDING * 2;
    int y = CONTENT_PADDING;

    // Card 1: Input method (4 rows)
    int card1Y = y;
    int card1H = ROW_HEIGHT * 4 + 1;  // +1 for dividers

    toggleEnabled_ = CreateToggleSwitch(hwnd_, contentX + contentWidth - TOGGLE_WIDTH - 12, card1Y + 6, IDT_ENABLED);
    toggleWShortcut_ = CreateToggleSwitch(hwnd_, contentX + contentWidth - TOGGLE_WIDTH - 12, card1Y + ROW_HEIGHT * 2 + 6, IDT_W_SHORTCUT);
    toggleBracket_ = CreateToggleSwitch(hwnd_, contentX + contentWidth - TOGGLE_WIDTH - 12, card1Y + ROW_HEIGHT * 3 + 6, IDT_BRACKET);

    // Combo box for method
    cmbMethod_ = CreateWindowExW(0, L"COMBOBOX", NULL,
        WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST | WS_VSCROLL,
        contentX + contentWidth - 110, card1Y + ROW_HEIGHT + 10, 100, 200,
        hwnd_, (HMENU)IDC_CMB_METHOD, GetModuleHandle(NULL), NULL);
    ComboBox_AddString(cmbMethod_, L"Telex");
    ComboBox_AddString(cmbMethod_, L"VNI");

    y += card1H + 20;

    // Card 2: Shortcuts (2 rows)
    int card2Y = y;
    int card2H = ROW_HEIGHT * 2 + 1;

    btnShortcuts_ = CreateWindowExW(0, L"BUTTON", L">",
        WS_CHILD | WS_VISIBLE | BS_FLAT,
        contentX + contentWidth - 30, card2Y + ROW_HEIGHT + 10, 20, 24,
        hwnd_, (HMENU)IDC_BTN_SHORTCUTS, GetModuleHandle(NULL), NULL);

    y += card2H + 20;

    // Card 3: Behavior (3 rows)
    int card3Y = y;
    int card3H = ROW_HEIGHT * 3 + 1;

    toggleAutoStart_ = CreateToggleSwitch(hwnd_, contentX + contentWidth - TOGGLE_WIDTH - 12, card3Y + 6, IDT_AUTOSTART);
    togglePerApp_ = CreateToggleSwitch(hwnd_, contentX + contentWidth - TOGGLE_WIDTH - 12, card3Y + ROW_HEIGHT + 6, IDT_PERAPP);
    toggleAutoRestore_ = CreateToggleSwitch(hwnd_, contentX + contentWidth - TOGGLE_WIDTH - 12, card3Y + ROW_HEIGHT * 2 + 6, IDT_AUTORESTORE);

    y += card3H + 20;

    // Card 4: Advanced (3 rows)
    int card4Y = y;
    int card4H = ROW_HEIGHT * 3 + 1;

    toggleSound_ = CreateToggleSwitch(hwnd_, contentX + contentWidth - TOGGLE_WIDTH - 12, card4Y + 6, IDT_SOUND);
    toggleModernTone_ = CreateToggleSwitch(hwnd_, contentX + contentWidth - TOGGLE_WIDTH - 12, card4Y + ROW_HEIGHT + 6, IDT_MODERN_TONE);
    toggleCapitalize_ = CreateToggleSwitch(hwnd_, contentX + contentWidth - TOGGLE_WIDTH - 12, card4Y + ROW_HEIGHT * 2 + 6, IDT_CAPITALIZE);

    // Store card positions for painting
    cards_[0] = { contentX, card1Y, contentX + contentWidth, card1Y + card1H };
    cards_[1] = { contentX, card2Y, contentX + contentWidth, card2Y + card2H };
    cards_[2] = { contentX, card3Y, contentX + contentWidth, card3Y + card3H };
    cards_[3] = { contentX, card4Y, contentX + contentWidth, card4Y + card4H };

    // Set font for combo
    HFONT hFont = CreateFontW(-13, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH, L"Segoe UI");
    SendMessage(cmbMethod_, WM_SETFONT, (WPARAM)hFont, TRUE);
}

void SettingsWindow::LoadSettings() {
    auto& settings = Settings::Instance();

    SetToggleState(toggleEnabled_, settings.enabled);
    ComboBox_SetCurSel(cmbMethod_, settings.method);
    SetToggleState(toggleWShortcut_, !settings.skipWShortcut);  // Inverted
    SetToggleState(toggleBracket_, settings.bracketShortcut);
    SetToggleState(toggleAutoStart_, settings.autoStart);
    SetToggleState(togglePerApp_, settings.perApp);
    SetToggleState(toggleAutoRestore_, settings.autoRestore);
    SetToggleState(toggleSound_, settings.sound);
    SetToggleState(toggleModernTone_, settings.modernTone);
    SetToggleState(toggleCapitalize_, settings.autoCapitalize);
}

void SettingsWindow::SaveSettings() {
    auto& settings = Settings::Instance();

    settings.enabled = GetToggleState(toggleEnabled_);
    settings.method = static_cast<uint8_t>(ComboBox_GetCurSel(cmbMethod_));
    settings.skipWShortcut = !GetToggleState(toggleWShortcut_);  // Inverted
    settings.bracketShortcut = GetToggleState(toggleBracket_);
    settings.autoStart = GetToggleState(toggleAutoStart_);
    settings.perApp = GetToggleState(togglePerApp_);
    settings.autoRestore = GetToggleState(toggleAutoRestore_);
    settings.sound = GetToggleState(toggleSound_);
    settings.modernTone = GetToggleState(toggleModernTone_);
    settings.autoCapitalize = GetToggleState(toggleCapitalize_);

    settings.Save();
    settings.ApplyToEngine();
}

void SettingsWindow::PaintWindow(HDC hdc) {
    const Theme& theme = GetTheme();

    RECT clientRect;
    GetClientRect(hwnd_, &clientRect);

    // Create memory DC for double buffering
    HDC memDC = CreateCompatibleDC(hdc);
    HBITMAP memBitmap = CreateCompatibleBitmap(hdc, clientRect.right, clientRect.bottom);
    HBITMAP oldBitmap = (HBITMAP)SelectObject(memDC, memBitmap);

    // Draw sidebar background
    RECT sidebarRect = { 0, 0, SIDEBAR_WIDTH, clientRect.bottom };
    HBRUSH sidebarBrush = CreateSolidBrush(theme.sidebarBg);
    FillRect(memDC, &sidebarRect, sidebarBrush);
    DeleteObject(sidebarBrush);

    // Draw content background
    RECT contentRect = { SIDEBAR_WIDTH, 0, clientRect.right, clientRect.bottom };
    HBRUSH contentBrush = CreateSolidBrush(theme.windowBg);
    FillRect(memDC, &contentRect, contentBrush);
    DeleteObject(contentBrush);

    // Draw sidebar content
    PaintSidebar(memDC);

    // Draw cards
    PaintCards(memDC);

    // Draw card content (labels)
    PaintCardContent(memDC);

    // Blit to screen
    BitBlt(hdc, 0, 0, clientRect.right, clientRect.bottom, memDC, 0, 0, SRCCOPY);

    SelectObject(memDC, oldBitmap);
    DeleteObject(memBitmap);
    DeleteDC(memDC);
}

void SettingsWindow::PaintSidebar(HDC hdc) {
    const Theme& theme = GetTheme();

    // Draw logo placeholder (96x96 at center top)
    int logoSize = 96;
    int logoX = (SIDEBAR_WIDTH - logoSize) / 2;
    int logoY = 40;

    // Draw orange rounded square for logo
    RECT logoRect = { logoX, logoY, logoX + logoSize, logoY + logoSize };
    DrawRoundedRect(hdc, logoRect, 20, RGB(255, 100, 50), RGB(255, 100, 50));

    // Draw lightning bolt (using Segoe UI Emoji or Wingdings)
    RECT boltRect = { logoX, logoY + 20, logoX + logoSize, logoY + logoSize - 20 };
    DrawText(hdc, L"\x26A1", boltRect, RGB(255, 220, 50), 48, true, DT_CENTER | DT_VCENTER);

    // Draw app name (use ASCII-safe name + Unicode escape)
    RECT nameRect = { 0, logoY + logoSize + 12, SIDEBAR_WIDTH, logoY + logoSize + 40 };
    DrawText(hdc, L"G\x00F5 Nhanh", nameRect, theme.textPrimary, 18, true, DT_CENTER | DT_VCENTER);

    // Draw version
    RECT versionRect = { 0, logoY + logoSize + 45, SIDEBAR_WIDTH, logoY + logoSize + 65 };
    DrawText(hdc, L"v1.0.107", versionRect, theme.textTertiary, 11, false, DT_CENTER | DT_VCENTER);

    // Draw navigation buttons at bottom
    int navY = WINDOW_HEIGHT - 90;

    // Settings button (selected)
    RECT settingsBtn = { 12, navY, SIDEBAR_WIDTH - 12, navY + 36 };
    DrawRoundedRect(hdc, settingsBtn, 8, theme.selectedBg, theme.selectedBg);
    RECT settingsText = { 20, navY, SIDEBAR_WIDTH - 8, navY + 36 };
    DrawText(hdc, L"C\x00E0i \x0111\x1EB7t", settingsText, theme.textPrimary, 13, false, DT_LEFT | DT_VCENTER);

    // About button
    RECT aboutBtn = { 12, navY + 40, SIDEBAR_WIDTH - 12, navY + 76 };
    RECT aboutText = { 20, navY + 40, SIDEBAR_WIDTH - 8, navY + 76 };
    DrawText(hdc, L"Gi\x1EDBi thi\x1EC7u", aboutText, theme.textSecondary, 13, false, DT_LEFT | DT_VCENTER);
}

void SettingsWindow::PaintCards(HDC hdc) {
    const Theme& theme = GetTheme();

    for (int i = 0; i < 4; i++) {
        DrawRoundedRect(hdc, cards_[i], CARD_RADIUS, theme.cardBg, theme.cardBorder);
    }
}

void SettingsWindow::PaintCardContent(HDC hdc) {
    const Theme& theme = GetTheme();
    const int textX = cards_[0].left + 12;
    const int textWidth = cards_[0].right - cards_[0].left - TOGGLE_WIDTH - 30;

    // Vietnamese strings using Unicode escapes
    // Card 1: Input method
    int y = cards_[0].top;
    DrawSettingsRow(hdc, textX, y, textWidth, L"B\x1ED9 g\x00F5 ti\x1EBFng Vi\x1EC7t", nullptr);
    DrawDivider(hdc, textX, y + ROW_HEIGHT, textWidth + TOGGLE_WIDTH + 10);

    y += ROW_HEIGHT;
    DrawSettingsRow(hdc, textX, y, textWidth, L"Ki\x1EC3u g\x00F5", nullptr);
    DrawDivider(hdc, textX, y + ROW_HEIGHT, textWidth + TOGGLE_WIDTH + 10);

    y += ROW_HEIGHT;
    DrawSettingsRow(hdc, textX, y, textWidth, L"G\x00F5 W th\x00E0nh \x01AF \x1EDF \x0111\x1EA7u t\x1EEB", nullptr);
    DrawDivider(hdc, textX, y + ROW_HEIGHT, textWidth + TOGGLE_WIDTH + 10);

    y += ROW_HEIGHT;
    DrawSettingsRow(hdc, textX, y, textWidth, L"G\x00F5 ] th\x00E0nh \x01AF, [ th\x00E0nh \x01A0", nullptr);

    // Card 2: Shortcuts
    y = cards_[1].top;
    DrawSettingsRow(hdc, textX, y, textWidth, L"Ph\x00EDm t\x1EAFt b\x1EADt/t\x1EAFt", L"Nh\x1EA5n \x0111\x1EC3 thay \x0111\x1ED5i");
    DrawDivider(hdc, textX, y + ROW_HEIGHT, textWidth + TOGGLE_WIDTH + 10);

    // Draw hotkey display
    RECT hotkeyRect = { cards_[1].right - 100, (int)(y + 8), cards_[1].right - 20, (int)(y + 36) };
    DrawRoundedRect(hdc, hotkeyRect, 4, theme.sidebarBg, theme.cardBorder);
    DrawText(hdc, L"^ Space", hotkeyRect, theme.textSecondary, 11, false, DT_CENTER | DT_VCENTER);

    y += ROW_HEIGHT;
    DrawSettingsRow(hdc, textX, y, textWidth, L"B\x1EA3ng g\x00F5 t\x1EAFt", L"1/5 \x0111ang b\x1EADt");

    // Card 3: Behavior
    y = cards_[2].top;
    DrawSettingsRow(hdc, textX, y, textWidth, L"Kh\x1EDFi \x0111\x1ED9ng c\x00F9ng h\x1EC7 th\x1ED1ng", nullptr);
    DrawDivider(hdc, textX, y + ROW_HEIGHT, textWidth + TOGGLE_WIDTH + 10);

    y += ROW_HEIGHT;
    DrawSettingsRow(hdc, textX, y, textWidth, L"T\x1EF1 chuy\x1EC3n ch\x1EBF \x0111\x1ED9 theo \x1EE9ng " L"d\x1EE5ng", nullptr);
    DrawDivider(hdc, textX, y + ROW_HEIGHT, textWidth + TOGGLE_WIDTH + 10);

    y += ROW_HEIGHT;
    DrawSettingsRow(hdc, textX, y, textWidth, L"T\x1EF1 kh\x00F4i ph\x1EE5" L"c t\x1EEB ti\x1EBFng Anh", nullptr);
    // Draw beta badge (positioned after text)
    RECT badgeRect = { textX + 280, (int)(y + 12), textX + 330, (int)(y + 28) };
    DrawRoundedRect(hdc, badgeRect, 8, RGB(255, 165, 0), RGB(255, 165, 0));
    DrawText(hdc, L"Beta", badgeRect, RGB(255, 255, 255), 9, true, DT_CENTER | DT_VCENTER);

    // Card 4: Advanced
    y = cards_[3].top;
    DrawSettingsRow(hdc, textX, y, textWidth, L"\x00C2m thanh chuy\x1EC3n ng\x00F4n ng\x1EEF", nullptr);
    DrawDivider(hdc, textX, y + ROW_HEIGHT, textWidth + TOGGLE_WIDTH + 10);

    y += ROW_HEIGHT;
    DrawSettingsRow(hdc, textX, y, textWidth, L"\x0110\x1EB7t d\x1EA5u ki\x1EC3u m\x1EDBi (o\x00E0, u\x00FD)", nullptr);
    DrawDivider(hdc, textX, y + ROW_HEIGHT, textWidth + TOGGLE_WIDTH + 10);

    y += ROW_HEIGHT;
    DrawSettingsRow(hdc, textX, y, textWidth, L"T\x1EF1 vi\x1EBFt hoa \x0111\x1EA7u c\x00E2u", nullptr);
}

void SettingsWindow::DrawSettingsRow(HDC hdc, int x, int y, int width, const wchar_t* title, const wchar_t* subtitle) {
    const Theme& theme = GetTheme();

    RECT titleRect = { x, y + (subtitle ? 8 : 0), x + width, y + ROW_HEIGHT - (subtitle ? 14 : 0) };
    DrawText(hdc, title, titleRect, theme.textPrimary, 13, false, DT_LEFT | DT_VCENTER);

    if (subtitle) {
        RECT subtitleRect = { x, y + 24, x + width, y + ROW_HEIGHT - 4 };
        DrawText(hdc, subtitle, subtitleRect, theme.textSecondary, 11, false, DT_LEFT | DT_VCENTER);
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
            if (window) window->PaintWindow(hdc);
            EndPaint(hwnd, &ps);
            return 0;
        }

        case WM_ERASEBKGND:
            return 1;

        case WM_TOGGLE_CHANGED: {
            if (window) window->SaveSettings();
            InvalidateRect(hwnd, NULL, FALSE);
            return 0;
        }

        case WM_COMMAND: {
            if (LOWORD(wParam) == IDC_CMB_METHOD && HIWORD(wParam) == CBN_SELCHANGE) {
                if (window) window->SaveSettings();
            }
            if (LOWORD(wParam) == IDC_BTN_SHORTCUTS) {
                ShortcutsDialog::Instance().Show();
            }
            return 0;
        }

        case WM_NCHITTEST: {
            // Allow dragging from sidebar area
            POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
            ScreenToClient(hwnd, &pt);
            if (pt.x < SIDEBAR_WIDTH && pt.y < 200) {
                return HTCAPTION;
            }
            return HTCLIENT;
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

// Legacy dialog proc - not used anymore
INT_PTR CALLBACK SettingsWindow::DialogProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    return FALSE;
}

} // namespace gonhanh
