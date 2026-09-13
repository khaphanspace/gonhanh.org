#pragma once
#include <windows.h>
#include <commctrl.h>
#include <string>

namespace gonhanh {

class ShortcutsDialog {
public:
    static ShortcutsDialog& Instance();
    void Show();
    void Hide();
    bool IsVisible() const { return visible_; }

private:
    ShortcutsDialog() = default;
    ~ShortcutsDialog();
    ShortcutsDialog(const ShortcutsDialog&) = delete;
    ShortcutsDialog& operator=(const ShortcutsDialog&) = delete;

    void Create();
    void CreateControls();
    void LoadShortcuts();
    void SaveShortcuts();
    void AddShortcut();
    void DeleteShortcut(int index);
    void ImportShortcuts();
    void ExportShortcuts();
    void UpdateListItem(int index);

    static INT_PTR CALLBACK DialogProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    static LRESULT CALLBACK ListViewProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam, UINT_PTR id, DWORD_PTR data);

    HWND hwnd_ = nullptr;
    HWND listView_ = nullptr;
    bool visible_ = false;
    bool editing_ = false;
    int editIndex_ = -1;
};

} // namespace gonhanh
