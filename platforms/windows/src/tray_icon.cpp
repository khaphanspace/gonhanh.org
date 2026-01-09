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

    // Create initial icon (enabled, Telex by default)
    current_icon_ = create_icon(true, 0);

    // Setup notify icon data
    nid_.cbSize = sizeof(NOTIFYICONDATAW);
    nid_.hWnd = hwnd_;
    nid_.uID = 1;
    nid_.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid_.uCallbackMessage = WM_TRAY_ICON;
    nid_.hIcon = current_icon_;
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

    destroy_current_icon();
}

void TrayIcon::destroy_current_icon() {
    if (current_icon_) {
        DestroyIcon(current_icon_);
        current_icon_ = nullptr;
    }
}

HICON TrayIcon::create_icon(bool enabled, int method) {
    // Create a 16x16 icon with rounded rect background
    // Blue background with white "V" (Telex) or "N" (VNI) when enabled
    // Gray background with white "E" when disabled
    const int size = 16;
    const int radius = 3;

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

    HGDIOBJ old_bmp = SelectObject(mem_dc, color_bmp);

    // Clear with transparent (black with alpha=0 - but we're using mask for transparency)
    HBRUSH clear_brush = CreateSolidBrush(RGB(0, 0, 0));
    RECT full_rect = {0, 0, size, size};
    FillRect(mem_dc, &full_rect, clear_brush);
    DeleteObject(clear_brush);

    // Background color: blue when enabled, gray when disabled
    COLORREF bg_color = enabled ? RGB(37, 99, 235) : RGB(107, 114, 128);
    HBRUSH bg_brush = CreateSolidBrush(bg_color);
    HPEN bg_pen = CreatePen(PS_SOLID, 1, bg_color);
    HGDIOBJ old_brush = SelectObject(mem_dc, bg_brush);
    HGDIOBJ old_pen = SelectObject(mem_dc, bg_pen);

    // Draw rounded rectangle background
    RoundRect(mem_dc, 0, 0, size, size, radius * 2, radius * 2);

    SelectObject(mem_dc, old_brush);
    SelectObject(mem_dc, old_pen);
    DeleteObject(bg_brush);
    DeleteObject(bg_pen);

    // Select text character based on state
    const wchar_t* text;
    if (enabled) {
        text = (method == 0) ? L"V" : L"N";  // V for Telex, N for VNI
    } else {
        text = L"E";  // E for English/disabled
    }

    // Draw white text
    SetBkMode(mem_dc, TRANSPARENT);
    SetTextColor(mem_dc, RGB(255, 255, 255));

    HFONT font = CreateFontW(
        11, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI"
    );
    HFONT old_font = (HFONT)SelectObject(mem_dc, font);

    RECT text_rect = {0, 1, size, size};  // Slight offset for vertical centering
    DrawTextW(mem_dc, text, -1, &text_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

    SelectObject(mem_dc, old_font);
    DeleteObject(font);

    // Create mask bitmap (black = opaque, white = transparent)
    HDC mask_dc = CreateCompatibleDC(screen_dc);
    SelectObject(mask_dc, mask_bmp);

    // Fill with white (transparent)
    HBRUSH white_brush = CreateSolidBrush(RGB(255, 255, 255));
    FillRect(mask_dc, &full_rect, white_brush);
    DeleteObject(white_brush);

    // Draw black rounded rect (opaque area)
    HBRUSH black_brush = CreateSolidBrush(RGB(0, 0, 0));
    HPEN black_pen = CreatePen(PS_SOLID, 1, RGB(0, 0, 0));
    SelectObject(mask_dc, black_brush);
    SelectObject(mask_dc, black_pen);
    RoundRect(mask_dc, 0, 0, size, size, radius * 2, radius * 2);
    DeleteObject(black_brush);
    DeleteObject(black_pen);
    DeleteDC(mask_dc);

    SelectObject(mem_dc, old_bmp);

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
    AppendMenuW(menu_, MF_STRING, static_cast<UINT>(TrayMenuId::CheckForUpdates), L"Kiểm tra cập nhật...");
    AppendMenuW(menu_, MF_SEPARATOR, 0, nullptr);

    // Exit
    AppendMenuW(menu_, MF_STRING, static_cast<UINT>(TrayMenuId::Exit), L"Thoát");
}

void TrayIcon::update_icon(bool enabled, int method) {
    enabled_ = enabled;
    method_ = method;

    // Recreate icon with new state (shows V/N for Telex/VNI when enabled, E when disabled)
    destroy_current_icon();
    current_icon_ = create_icon(enabled, method);
    nid_.hIcon = current_icon_;

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
