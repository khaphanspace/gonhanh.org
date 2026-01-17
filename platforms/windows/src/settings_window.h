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

    // Painting
    void PaintWindow(HDC hdc);
    void PaintSidebar(HDC hdc);
    void PaintCards(HDC hdc);
    void PaintCardContent(HDC hdc);
    void DrawSettingsRow(HDC hdc, int x, int y, int width, const wchar_t* title, const wchar_t* subtitle);

    static INT_PTR CALLBACK DialogProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);

    HWND hwnd_ = nullptr;
    bool visible_ = false;

    // Toggle switch handles
    HWND toggleEnabled_ = nullptr;
    HWND toggleWShortcut_ = nullptr;
    HWND toggleBracket_ = nullptr;
    HWND toggleAutoStart_ = nullptr;
    HWND togglePerApp_ = nullptr;
    HWND toggleAutoRestore_ = nullptr;
    HWND toggleSound_ = nullptr;
    HWND toggleModernTone_ = nullptr;
    HWND toggleCapitalize_ = nullptr;

    // Other controls
    HWND cmbMethod_ = nullptr;
    HWND btnShortcuts_ = nullptr;

    // Card rectangles for painting
    RECT cards_[4] = {};
};

} // namespace gonhanh
