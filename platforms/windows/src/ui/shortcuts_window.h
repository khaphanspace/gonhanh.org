#pragma once

#include <windows.h>
#include <d2d1.h>
#include <wrl/client.h>
#include <string>
#include <vector>

using Microsoft::WRL::ComPtr;

namespace gonhanh::ui {

// Shortcut item structure
struct ShortcutItem {
    std::wstring key;
    std::wstring value;
    bool enabled;

    ShortcutItem() : enabled(true) {}
    ShortcutItem(const std::wstring& k, const std::wstring& v, bool e = true)
        : key(k), value(v), enabled(e) {}
};

// Shortcuts management window (480x420)
class ShortcutsWindow {
public:
    static ShortcutsWindow& instance();

    // Show/hide window
    void show();
    void hide();
    bool is_visible() const;

    // Shortcuts management
    const std::vector<ShortcutItem>& shortcuts() const { return shortcuts_; }
    void add_shortcut(const std::wstring& key, const std::wstring& value);
    void update_shortcut(size_t index, const std::wstring& key, const std::wstring& value);
    void delete_shortcut(size_t index);
    void toggle_shortcut(size_t index);
    void import_shortcuts();
    void export_shortcuts();

    // Window dimensions
    static constexpr int WIDTH = 480;
    static constexpr int HEIGHT = 420;

private:
    ShortcutsWindow() = default;
    ~ShortcutsWindow();
    ShortcutsWindow(const ShortcutsWindow&) = delete;
    ShortcutsWindow& operator=(const ShortcutsWindow&) = delete;

    bool create_window();
    void render();
    void load_shortcuts();
    void save_shortcuts();
    void sync_to_engine();

    static LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam);

    HWND hwnd_ = nullptr;
    ComPtr<ID2D1HwndRenderTarget> render_target_;

    // UI state
    std::vector<ShortcutItem> shortcuts_;
    std::wstring key_input_;
    std::wstring value_input_;
    int selected_index_ = -1;  // -1 = Add mode, >= 0 = Edit mode
    int hovered_row_ = -1;
    bool hovering_delete_ = false;
    float scroll_offset_ = 0.0f;

    // UI layout constants
    static constexpr int PADDING = 24;
    static constexpr int HEADER_HEIGHT = 60;
    static constexpr int FORM_HEIGHT = 90;
    static constexpr int TOOLBAR_HEIGHT = 50;
    static constexpr int ROW_HEIGHT = 40;
    static constexpr int SCROLL_AMOUNT = 40;
};

} // namespace gonhanh::ui
