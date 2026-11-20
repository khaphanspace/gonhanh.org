#pragma once
#include <windows.h>
#include <shellapi.h>

namespace gonhanh {

class SystemTray {
public:
    static SystemTray& Instance();
    bool Create(HWND hwnd);
    void Destroy();
    void UpdateIcon();
    void ShowMenu();
    void HandleMessage(WPARAM wParam, LPARAM lParam);

private:
    SystemTray() = default;
    ~SystemTray();
    SystemTray(const SystemTray&) = delete;
    SystemTray& operator=(const SystemTray&) = delete;

    HWND hwnd_ = nullptr;
    NOTIFYICONDATAW nid_ = {};
    bool created_ = false;
    HICON iconOn_ = nullptr;   // Vietnamese: full-color logo
    HICON iconOff_ = nullptr;  // English: grayscale, dimmed logo
};

} // namespace gonhanh
