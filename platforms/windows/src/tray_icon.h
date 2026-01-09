#pragma once

#include <windows.h>
#include <shellapi.h>
#include <functional>
#include <string>

namespace gonhanh {

// Tray icon menu item IDs
enum class TrayMenuId : UINT {
    Toggle = 1001,
    Telex = 1002,
    VNI = 1003,
    Settings = 1004,
    About = 1005,
    Exit = 1006,
};

// System tray icon management
class TrayIcon {
public:
    using MenuCallback = std::function<void(TrayMenuId)>;

    static TrayIcon& instance();

    // Initialize with parent window handle
    bool initialize(HWND parent_hwnd);
    void shutdown();

    // Update icon based on state
    void update_icon(bool enabled, int method);  // method: 0=Telex, 1=VNI

    // Show context menu
    void show_context_menu();

    // Set callback for menu selection
    void set_menu_callback(MenuCallback callback) { menu_callback_ = std::move(callback); }

    // Process tray messages (call from WndProc)
    bool process_message(UINT msg, WPARAM wparam, LPARAM lparam);

    // Tray message ID
    static constexpr UINT WM_TRAY_ICON = WM_USER + 100;

private:
    TrayIcon() = default;
    ~TrayIcon();
    TrayIcon(const TrayIcon&) = delete;
    TrayIcon& operator=(const TrayIcon&) = delete;

    HICON create_icon(bool enabled);
    void create_context_menu();

    HWND hwnd_ = nullptr;
    NOTIFYICONDATAW nid_ = {};
    HMENU menu_ = nullptr;
    HICON icon_enabled_ = nullptr;
    HICON icon_disabled_ = nullptr;
    MenuCallback menu_callback_;
    bool enabled_ = true;
    int method_ = 0;
};

} // namespace gonhanh
