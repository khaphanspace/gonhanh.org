#pragma once
#include <windows.h>
#include <string>
#include <vector>

namespace gonhanh {

// "Bảng gõ tắt" window: Fluent-styled list of text shortcuts with an add/edit form,
// per-shortcut enable toggle, delete, and import/export. Changes apply immediately.
class ShortcutsDialog {
public:
    static ShortcutsDialog& Instance();
    void Show(HWND owner = nullptr);
    void Hide();
    bool IsVisible() const { return visible_; }

private:
    ShortcutsDialog() = default;
    ~ShortcutsDialog();
    ShortcutsDialog(const ShortcutsDialog&) = delete;
    ShortcutsDialog& operator=(const ShortcutsDialog&) = delete;

    enum class Hit { None, Import, Export, Submit, Cancel, Toggle, Row, Delete, Scrollbar };
    struct HitResult {
        Hit kind = Hit::None;
        int index = -1;
        bool operator==(const HitResult& o) const { return kind == o.kind && index == o.index; }
        bool operator!=(const HitResult& o) const { return !(*this == o); }
    };

    void Create(HWND owner);
    void CreateEdits();
    void ApplyTheme();
    void Layout();
    int S(int logical) const;

    void Paint(HDC hdc);
    HitResult HitTest(POINT pt) const;
    void Activate(const HitResult& hit);

    // Editing
    void Submit();                 // Add new, or save the shortcut being edited
    void BeginEdit(int index);
    void CancelEdit();
    void DeleteShortcut(int index);
    void ToggleShortcut(int index);
    void Commit();                 // Save to registry, apply to engine, refresh Settings
    void ImportShortcuts();
    void ExportShortcuts();
    void SetStatus(const std::wstring& text, bool error);

    // List scrolling
    RECT ListViewport() const;     // Visible list area (client coordinates)
    int ListContentHeight() const;
    void ScrollTo(int pos);
    bool ScrollbarGeometry(RECT& track, RECT& thumb) const;

    std::wstring EditText(HWND edit) const;
    static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    static LRESULT CALLBACK EditProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam,
                                     UINT_PTR id, DWORD_PTR data);

    HWND hwnd_ = nullptr;
    HWND triggerEdit_ = nullptr;
    HWND replacementEdit_ = nullptr;
    HFONT editFont_ = nullptr;
    HBRUSH fieldBrush_ = nullptr;
    bool visible_ = false;
    UINT dpi_ = 96;

    int editIndex_ = -1;           // Shortcut being edited, -1 when adding
    int scrollPos_ = 0;
    std::wstring status_;
    bool statusError_ = false;

    // Layout (client coordinates)
    RECT importRect_ = {}, exportRect_ = {};
    RECT formCard_ = {}, triggerField_ = {}, replacementField_ = {}, submitRect_ = {}, cancelRect_ = {};
    RECT listCard_ = {};

    // Mouse state
    HitResult hover_, pressed_;
    bool trackingMouse_ = false;
    bool draggingThumb_ = false;
    int dragStartY_ = 0;
    int dragStartScroll_ = 0;
};

} // namespace gonhanh
