#include "system_tray.h"
#include "resource.h"
#include "settings.h"
#include <string>

namespace gonhanh {

SystemTray& SystemTray::Instance() {
    static SystemTray instance;
    return instance;
}

SystemTray::~SystemTray() {
    Destroy();
}

bool SystemTray::Create(HWND hwnd) {
    if (created_) return true;

    hwnd_ = hwnd;

    // Initialize NOTIFYICONDATAW
    nid_.cbSize = sizeof(NOTIFYICONDATAW);
    nid_.hWnd = hwnd;
    nid_.uID = 1;
    nid_.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid_.uCallbackMessage = WM_TRAYICON;

    // Load icon
    nid_.hIcon = LoadIconW(GetModuleHandle(NULL), MAKEINTRESOURCEW(IDI_TRAY_ICON));

    // Set tooltip
    wcscpy_s(nid_.szTip, L"Gõ Nhanh - Bộ gõ tiếng Việt");

    // Add to system tray
    if (Shell_NotifyIconW(NIM_ADD, &nid_)) {
        created_ = true;
        UpdateIcon();
        return true;
    }

    return false;
}

void SystemTray::Destroy() {
    if (created_) {
        Shell_NotifyIconW(NIM_DELETE, &nid_);
        created_ = false;
    }
}

void SystemTray::UpdateIcon() {
    if (!created_) return;

    // Update tooltip based on current state
    auto& settings = Settings::Instance();
    std::wstring tooltip = L"Gõ Nhanh - ";

    if (settings.enabled) {
        tooltip += (settings.method == 0) ? L"Telex" : L"VNI";
    } else {
        tooltip += L"Tắt";
    }

    wcscpy_s(nid_.szTip, tooltip.c_str());
    Shell_NotifyIconW(NIM_MODIFY, &nid_);
}

void SystemTray::ShowMenu() {
    if (!hwnd_) return;

    auto& settings = Settings::Instance();

    // Create popup menu
    HMENU menu = CreatePopupMenu();
    if (!menu) return;

    // Enable/Disable toggle
    AppendMenuW(menu, MF_STRING | (settings.enabled ? MF_CHECKED : 0),
                IDM_ENABLE, L"Bật tiếng Việt");

    AppendMenuW(menu, MF_SEPARATOR, 0, nullptr);

    // Method submenu
    HMENU methodMenu = CreatePopupMenu();
    AppendMenuW(methodMenu, MF_STRING | (settings.method == 0 ? MF_CHECKED : 0),
                IDM_TELEX, L"Telex");
    AppendMenuW(methodMenu, MF_STRING | (settings.method == 1 ? MF_CHECKED : 0),
                IDM_VNI, L"VNI");
    AppendMenuW(menu, MF_STRING | MF_POPUP, (UINT_PTR)methodMenu, L"Kiểu gõ");

    AppendMenuW(menu, MF_SEPARATOR, 0, nullptr);

    // Settings, About, Exit
    AppendMenuW(menu, MF_STRING, IDM_SETTINGS, L"Cài đặt...");
    AppendMenuW(menu, MF_STRING, IDM_ABOUT, L"Giới thiệu");

    AppendMenuW(menu, MF_SEPARATOR, 0, nullptr);

    AppendMenuW(menu, MF_STRING, IDM_EXIT, L"Thoát");

    // Get cursor position
    POINT pt;
    GetCursorPos(&pt);

    // Required for proper menu behavior
    SetForegroundWindow(hwnd_);

    // Show menu
    TrackPopupMenu(menu, TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                   pt.x, pt.y, 0, hwnd_, NULL);

    // Clean up
    DestroyMenu(menu);
}

void SystemTray::HandleMessage(WPARAM wParam, LPARAM lParam) {
    if (wParam != 1) return;  // Check tray icon ID

    switch (lParam) {
        case WM_RBUTTONUP:
            ShowMenu();
            break;

        case WM_LBUTTONDBLCLK: {
            // Double-click toggles enabled state
            auto& settings = Settings::Instance();
            settings.enabled = !settings.enabled;
            settings.Save();
            UpdateIcon();
            break;
        }
    }
}

} // namespace gonhanh
