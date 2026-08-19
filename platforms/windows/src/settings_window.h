#pragma once
#include <windows.h>
#include <commctrl.h>
#include <vector>

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

    // Painting
    void PaintSidebar(HDC hdc);
    void PaintContent(HDC hdc);
    void PaintAbout(HDC hdc);
    void SwitchTab(int tab);

    static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);

    HWND hwnd_ = nullptr;
    HWND cmbMethod_ = nullptr;
    bool visible_ = false;

    // Custom painted section positions
    int section2Y_ = 0;
    RECT shortcutsRowRect_ = {};
    // Section divider Y positions (painted as thin lines)
    int dividerY1_ = 0;  // After section 1 (before shortcuts)
    int dividerY2_ = 0;  // After section 2 (after shortcuts)
    int dividerY3_ = 0;  // After section 3 (before advanced)

    // Sidebar tab hit rects
    RECT tabSettingsRect_ = {};
    RECT tabAboutRect_ = {};
    RECT aboutBtnRects_[3] = {};   // Ủng hộ, Báo lỗi, GitHub
    RECT authorLinkRect_ = {};     // "Kha Phan" link
    RECT communityLinkRect_ = {};  // "Cộng đồng" link

    // Scrolling (content area only)
    int scrollPos_ = 0;
    int contentHeight_ = 0;
    std::vector<HWND> contentControls_;  // Controls to scroll
    void UpdateScrollInfo();
    void ScrollContent(int newPos);

};

} // namespace gonhanh
