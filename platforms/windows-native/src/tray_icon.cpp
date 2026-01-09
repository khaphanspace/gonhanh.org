#include "tray_icon.h"

namespace gonhanh {

TrayIcon& TrayIcon::instance() {
    static TrayIcon instance;
    return instance;
}

TrayIcon::~TrayIcon() {
    shutdown();
}

bool TrayIcon::initialize(HWND parent_hwnd) {
    hwnd_ = parent_hwnd;

    // Create icons
    icon_enabled_ = create_icon(true);
    icon_disabled_ = create_icon(false);

    // Setup notify icon data
    nid_.cbSize = sizeof(NOTIFYICONDATAW);
    nid_.hWnd = hwnd_;
    nid_.uID = 1;
    nid_.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid_.uCallbackMessage = WM_TRAY_ICON;
    nid_.hIcon = icon_enabled_;
    wcscpy_s(nid_.szTip, L"GoNhanh - Telex [ON]");

    // Add icon to tray
    if (!Shell_NotifyIconW(NIM_ADD, &nid_)) {
        return false;
    }

    // Create context menu
    create_context_menu();

    return true;
}

void TrayIcon::shutdown() {
    if (nid_.hWnd) {
        Shell_NotifyIconW(NIM_DELETE, &nid_);
        nid_.hWnd = nullptr;
    }

    if (menu_) {
        DestroyMenu(menu_);
        menu_ = nullptr;
    }

    if (icon_enabled_) {
        DestroyIcon(icon_enabled_);
        icon_enabled_ = nullptr;
    }

    if (icon_disabled_) {
        DestroyIcon(icon_disabled_);
        icon_disabled_ = nullptr;
    }
}

HICON TrayIcon::create_icon(bool enabled) {
    // Create a simple 16x16 icon
    // Blue "V" when enabled, gray "E" when disabled
    const int size = 16;

    HDC screen_dc = GetDC(nullptr);
    HDC mem_dc = CreateCompatibleDC(screen_dc);

    BITMAPINFO bmi = {};
    bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    bmi.bmiHeader.biWidth = size;
    bmi.bmiHeader.biHeight = -size;  // Top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;

    void* bits = nullptr;
    HBITMAP color_bmp = CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &bits, nullptr, 0);
    HBITMAP mask_bmp = CreateBitmap(size, size, 1, 1, nullptr);

    SelectObject(mem_dc, color_bmp);

    // Fill with rounded rect background
    HBRUSH bg_brush = CreateSolidBrush(RGB(255, 255, 255));
    RECT rect = {0, 0, size, size};
    FillRect(mem_dc, &rect, bg_brush);
    DeleteObject(bg_brush);

    // Draw text
    COLORREF text_color = enabled ? RGB(37, 99, 235) : RGB(156, 163, 175);  // Blue or gray
    const wchar_t* text = enabled ? L"V" : L"E";

    SetBkMode(mem_dc, TRANSPARENT);
    SetTextColor(mem_dc, text_color);

    HFONT font = CreateFontW(
        12, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI"
    );
    HFONT old_font = (HFONT)SelectObject(mem_dc, font);

    DrawTextW(mem_dc, text, -1, &rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

    SelectObject(mem_dc, old_font);
    DeleteObject(font);

    // Create icon
    ICONINFO icon_info = {};
    icon_info.fIcon = TRUE;
    icon_info.hbmMask = mask_bmp;
    icon_info.hbmColor = color_bmp;

    HICON icon = CreateIconIndirect(&icon_info);

    DeleteObject(color_bmp);
    DeleteObject(mask_bmp);
    DeleteDC(mem_dc);
    ReleaseDC(nullptr, screen_dc);

    return icon;
}

void TrayIcon::create_context_menu() {
    menu_ = CreatePopupMenu();

    // Method selection
    AppendMenuW(menu_, MF_STRING, static_cast<UINT>(TrayMenuId::Telex), L"Telex");
    AppendMenuW(menu_, MF_STRING, static_cast<UINT>(TrayMenuId::VNI), L"VNI");
    AppendMenuW(menu_, MF_SEPARATOR, 0, nullptr);

    // Toggle
    AppendMenuW(menu_, MF_STRING, static_cast<UINT>(TrayMenuId::Toggle), L"Bật/Tắt");
    AppendMenuW(menu_, MF_SEPARATOR, 0, nullptr);

    // Settings & About
    AppendMenuW(menu_, MF_STRING, static_cast<UINT>(TrayMenuId::Settings), L"Cài đặt...");
    AppendMenuW(menu_, MF_STRING, static_cast<UINT>(TrayMenuId::About), L"Giới thiệu");
    AppendMenuW(menu_, MF_SEPARATOR, 0, nullptr);

    // Exit
    AppendMenuW(menu_, MF_STRING, static_cast<UINT>(TrayMenuId::Exit), L"Thoát");
}

void TrayIcon::update_icon(bool enabled, int method) {
    enabled_ = enabled;
    method_ = method;

    nid_.hIcon = enabled ? icon_enabled_ : icon_disabled_;

    // Update tooltip
    const wchar_t* method_name = (method == 0) ? L"Telex" : L"VNI";
    const wchar_t* state = enabled ? L"ON" : L"OFF";
    swprintf_s(nid_.szTip, L"GoNhanh - %s [%s]", method_name, state);

    Shell_NotifyIconW(NIM_MODIFY, &nid_);

    // Update menu checkmarks
    if (menu_) {
        CheckMenuItem(menu_, static_cast<UINT>(TrayMenuId::Telex),
                     MF_BYCOMMAND | (method == 0 ? MF_CHECKED : MF_UNCHECKED));
        CheckMenuItem(menu_, static_cast<UINT>(TrayMenuId::VNI),
                     MF_BYCOMMAND | (method == 1 ? MF_CHECKED : MF_UNCHECKED));
    }
}

void TrayIcon::show_context_menu() {
    if (!menu_) return;

    POINT pt;
    GetCursorPos(&pt);

    // Required for the menu to work properly
    SetForegroundWindow(hwnd_);

    UINT cmd = TrackPopupMenu(
        menu_,
        TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY,
        pt.x, pt.y,
        0, hwnd_, nullptr
    );

    PostMessage(hwnd_, WM_NULL, 0, 0);

    if (cmd && menu_callback_) {
        menu_callback_(static_cast<TrayMenuId>(cmd));
    }
}

bool TrayIcon::process_message(UINT msg, WPARAM wparam, LPARAM lparam) {
    if (msg != WM_TRAY_ICON) return false;

    switch (LOWORD(lparam)) {
        case WM_RBUTTONUP:
        case WM_CONTEXTMENU:
            show_context_menu();
            return true;

        case WM_LBUTTONDBLCLK:
            // Double-click toggles
            if (menu_callback_) {
                menu_callback_(TrayMenuId::Toggle);
            }
            return true;

        default:
            break;
    }

    return false;
}

} // namespace gonhanh
