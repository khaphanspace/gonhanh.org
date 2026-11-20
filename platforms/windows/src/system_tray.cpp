#include "system_tray.h"
#include "resource.h"
#include "settings.h"
#include "ime_state.h"
#include <cstdint>
#include <string>
#include <vector>

namespace gonhanh {

// Win32 popup menus are always light unless the process opts in to dark mode.
// uxtheme exports these by ordinal only (Windows 10 1903+); they are what Explorer,
// Notepad and other inbox apps use. Missing on older systems -> menus stay light.
static void PrepareMenuTheme(HWND owner) {
    using AllowDarkModeForWindowFn = BOOL(WINAPI*)(HWND, BOOL);
    using SetPreferredAppModeFn = int(WINAPI*)(int);
    using FlushMenuThemesFn = void(WINAPI*)();
    static AllowDarkModeForWindowFn allowDarkModeForWindow = nullptr;
    static SetPreferredAppModeFn setPreferredAppMode = nullptr;
    static FlushMenuThemesFn flushMenuThemes = nullptr;
    static bool initialized = false;

    if (!initialized) {
        initialized = true;
        if (HMODULE uxtheme = LoadLibraryExW(L"uxtheme.dll", nullptr, LOAD_LIBRARY_SEARCH_SYSTEM32)) {
            allowDarkModeForWindow = reinterpret_cast<AllowDarkModeForWindowFn>(
                GetProcAddress(uxtheme, MAKEINTRESOURCEA(133)));
            setPreferredAppMode = reinterpret_cast<SetPreferredAppModeFn>(
                GetProcAddress(uxtheme, MAKEINTRESOURCEA(135)));
            flushMenuThemes = reinterpret_cast<FlushMenuThemesFn>(
                GetProcAddress(uxtheme, MAKEINTRESOURCEA(136)));
        }
        if (setPreferredAppMode) {
            setPreferredAppMode(1);  // AllowDark: follow the system "app mode" (light/dark)
        }
        if (allowDarkModeForWindow && owner) {
            allowDarkModeForWindow(owner, TRUE);
        }
    }
    // Re-read the current system theme so a light/dark switch applies to the next menu
    if (flushMenuThemes) {
        flushMenuThemes();
    }
}

// Build a grayscale, semi-transparent copy of an icon for the "English" (off) state
static HICON MakeDimmedIcon(HICON source) {
    ICONINFO info = {};
    if (!GetIconInfo(source, &info)) return nullptr;

    HICON result = nullptr;
    BITMAP bm = {};
    if (info.hbmColor && GetObjectW(info.hbmColor, sizeof(bm), &bm)) {
        int w = bm.bmWidth;
        int h = bm.bmHeight;
        BITMAPINFO bi = {};
        bi.bmiHeader.biSize = sizeof(bi.bmiHeader);
        bi.bmiHeader.biWidth = w;
        bi.bmiHeader.biHeight = -h;
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = BI_RGB;

        std::vector<uint32_t> pixels(static_cast<size_t>(w) * h);
        HDC screen = GetDC(nullptr);
        if (GetDIBits(screen, info.hbmColor, 0, h, pixels.data(), &bi, DIB_RGB_COLORS)) {
            bool hasAlpha = false;
            for (uint32_t p : pixels) {
                if (p >> 24) { hasAlpha = true; break; }
            }
            for (uint32_t& p : pixels) {
                uint32_t a = hasAlpha ? (p >> 24) : 255;
                uint32_t r = (p >> 16) & 0xFF, g = (p >> 8) & 0xFF, b = p & 0xFF;
                uint32_t gray = (r * 30 + g * 59 + b * 11) / 100;
                gray = 96 + gray * 128 / 255;  // Flatten contrast toward mid-gray
                a = a * 70 / 100;              // Slightly faded
                p = (a << 24) | (gray << 16) | (gray << 8) | gray;
            }
            void* bits = nullptr;
            HBITMAP color = CreateDIBSection(screen, &bi, DIB_RGB_COLORS, &bits, nullptr, 0);
            if (color && bits) {
                memcpy(bits, pixels.data(), pixels.size() * sizeof(uint32_t));
                ICONINFO dimmed = info;
                dimmed.hbmColor = color;
                result = CreateIconIndirect(&dimmed);
            }
            if (color) DeleteObject(color);
        }
        ReleaseDC(nullptr, screen);
    }
    if (info.hbmColor) DeleteObject(info.hbmColor);
    if (info.hbmMask) DeleteObject(info.hbmMask);
    return result;
}

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

    // Initialize NOTIFYICONDATAW with stable GUID
    // Using GUID prevents duplicate tray entries when exe path changes between builds
    nid_.cbSize = sizeof(NOTIFYICONDATAW);
    nid_.hWnd = hwnd;
    nid_.uID = 1;
    nid_.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_GUID;
    nid_.uCallbackMessage = WM_TRAYICON;
    // {7A1B3C4D-5E6F-4A8B-9C0D-1E2F3A4B5C6D} - stable GUID for GoNhanh tray icon
    nid_.guidItem = { 0x7A1B3C4D, 0x5E6F, 0x4A8B, { 0x9C, 0x0D, 0x1E, 0x2F, 0x3A, 0x4B, 0x5C, 0x6D } };

    // Load small icon for system tray
    int size = GetSystemMetrics(SM_CXSMICON);
    iconOn_ = (HICON)LoadImageW(GetModuleHandle(NULL), MAKEINTRESOURCEW(IDI_TRAY_ICON),
                                IMAGE_ICON, size, size, LR_DEFAULTCOLOR);
    iconOff_ = iconOn_ ? MakeDimmedIcon(iconOn_) : nullptr;
    nid_.hIcon = Settings::Instance().enabled || !iconOff_ ? iconOn_ : iconOff_;

    // Set tooltip
    wcscpy_s(nid_.szTip, L"G\x00F5 Nhanh - B\x1ED9 g\x00F5 ti\x1EBFng Vi\x1EC7t");

    // Add to system tray (fallback without GUID if it fails)
    if (Shell_NotifyIconW(NIM_ADD, &nid_)) {
        created_ = true;
        UpdateIcon();
        return true;
    }
    // Fallback: some systems reject NIF_GUID, retry without it
    nid_.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid_.guidItem = {};
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
    if (iconOff_) { DestroyIcon(iconOff_); iconOff_ = nullptr; }
    if (iconOn_) { DestroyIcon(iconOn_); iconOn_ = nullptr; }
}

void SystemTray::UpdateIcon() {
    if (!created_) return;

    // Icon + tooltip reflect Vietnamese (color) vs English (dimmed)
    auto& settings = Settings::Instance();
    std::wstring tooltip = L"Gõ Nhanh - ";
    if (settings.enabled) {
        tooltip += L"Tiếng Việt (";
        tooltip += (settings.method == 0) ? L"Telex" : L"VNI";
        tooltip += L")";
    } else {
        tooltip += L"Tiếng Anh";
    }
    tooltip += L"\nNhấn để chuyển · Ctrl+Space";

    nid_.hIcon = settings.enabled || !iconOff_ ? iconOn_ : iconOff_;
    nid_.uFlags |= NIF_ICON | NIF_TIP | NIF_SHOWTIP;
    wcscpy_s(nid_.szTip, tooltip.c_str());
    Shell_NotifyIconW(NIM_MODIFY, &nid_);
}

void SystemTray::ShowMenu() {
    if (!hwnd_) return;

    auto& settings = Settings::Instance();
    PrepareMenuTheme(hwnd_);

    // Create popup menu
    HMENU menu = CreatePopupMenu();
    if (!menu) return;

    // Enable/Disable toggle (Unicode escapes for Vietnamese)
    AppendMenuW(menu, MF_STRING | (settings.enabled ? MF_CHECKED : 0),
                IDM_ENABLE, L"B\x1EADt ti\x1EBFng Vi\x1EC7t");

    AppendMenuW(menu, MF_SEPARATOR, 0, nullptr);

    // Method submenu
    HMENU methodMenu = CreatePopupMenu();
    AppendMenuW(methodMenu, MF_STRING | (settings.method == 0 ? MF_CHECKED : 0),
                IDM_TELEX, L"Telex");
    AppendMenuW(methodMenu, MF_STRING | (settings.method == 1 ? MF_CHECKED : 0),
                IDM_VNI, L"VNI");
    AppendMenuW(menu, MF_STRING | MF_POPUP, (UINT_PTR)methodMenu, L"Ki\x1EC3u g\x00F5");

    AppendMenuW(menu, MF_SEPARATOR, 0, nullptr);

    // Settings (app info and links live in its sidebar), Exit
    AppendMenuW(menu, MF_STRING, IDM_SETTINGS, L"C\x00E0i \x0111\x1EB7t...");

    AppendMenuW(menu, MF_SEPARATOR, 0, nullptr);

    AppendMenuW(menu, MF_STRING, IDM_EXIT, L"Tho\x00E1t");

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

        case WM_LBUTTONUP:
            // Single click switches Vietnamese <-> English
            ToggleVietnamese();
            break;
    }
}

} // namespace gonhanh
