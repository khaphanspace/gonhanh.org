#pragma once
#include <windows.h>

namespace gonhanh {

class AboutDialog {
public:
    static AboutDialog& Instance();
    void Show();
    void Hide();
    bool IsVisible() const { return visible_; }

private:
    AboutDialog() = default;
    ~AboutDialog();
    AboutDialog(const AboutDialog&) = delete;
    AboutDialog& operator=(const AboutDialog&) = delete;

    void Create();
    static INT_PTR CALLBACK DialogProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);

    HWND hwnd_ = nullptr;
    bool visible_ = false;
};

} // namespace gonhanh
