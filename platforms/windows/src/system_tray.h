#pragma once
#include <windows.h>
#include <shellapi.h>

namespace gonhanh {

class SystemTray {
public:
    static SystemTray& Instance();
    bool Create(HWND hwnd);
    // The taskbar was (re)created: Explorer started after GoNhanh, or restarted and dropped the
    // icon. Adds the icon again. Also used to retry when Create couldn't add it.
    bool Readd();
    bool IsAdded() const { return created_; }
    void Destroy();
    void UpdateIcon();
    void ShowMenu();
    void HandleMessage(WPARAM wParam, LPARAM lParam);

private:
    SystemTray() = default;
    ~SystemTray();
    SystemTray(const SystemTray&) = delete;
    SystemTray& operator=(const SystemTray&) = delete;
    bool Add();

    HWND hwnd_ = nullptr;
    NOTIFYICONDATAW nid_ = {};
    bool created_ = false;
    HICON iconOn_ = nullptr;   // Vietnamese: full-color logo
    HICON iconOff_ = nullptr;  // English: grayscale, dimmed logo
};

} // namespace gonhanh
