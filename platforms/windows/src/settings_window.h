#pragma once
#include <windows.h>
#include <commctrl.h>

namespace gonhanh {

class SettingsWindow {
public:
    static SettingsWindow& Instance();
    void Show();
    void Hide();
    bool IsVisible() const { return visible_; }

private:
    SettingsWindow() = default;
    ~SettingsWindow();
    SettingsWindow(const SettingsWindow&) = delete;
    SettingsWindow& operator=(const SettingsWindow&) = delete;

    void Create();
    void CreateControls();
    void LoadSettings();
    void SaveSettings();
    void ApplySettings();

    static INT_PTR CALLBACK DialogProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    static LRESULT CALLBACK HotkeyProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam, UINT_PTR id, DWORD_PTR data);

    HWND hwnd_ = nullptr;
    bool visible_ = false;

    // Control handles
    HWND chkEnabled_;
    HWND cmbMethod_;
    HWND chkWShortcut_;
    HWND chkBracket_;
    HWND chkEscRestore_;
    HWND chkAutoStart_;
    HWND chkPerApp_;
    HWND chkAutoRestore_;
    HWND chkSound_;
    HWND chkModern_;
    HWND chkCapitalize_;
    HWND btnShortcuts_;
    HWND txtHotkey_;
};

} // namespace gonhanh
