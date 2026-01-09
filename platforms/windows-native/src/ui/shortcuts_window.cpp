#include "shortcuts_window.h"
#include "d2d_renderer.h"
#include "../app.h"
#include "../settings.h"
#include "../rust_bridge.h"
#include <shlobj.h>
#include <fstream>
#include <sstream>
#include <algorithm>

namespace gonhanh::ui {

static constexpr const wchar_t* SHORTCUTS_WINDOW_CLASS = L"GoNhanhShortcutsClass";
static constexpr const wchar_t* REG_SHORTCUTS_PATH = L"SOFTWARE\\GoNhanh\\Shortcuts";

ShortcutsWindow& ShortcutsWindow::instance() {
    static ShortcutsWindow instance;
    return instance;
}

ShortcutsWindow::~ShortcutsWindow() {
    if (hwnd_) {
        DestroyWindow(hwnd_);
        hwnd_ = nullptr;
    }
}

void ShortcutsWindow::show() {
    if (!hwnd_) {
        if (!create_window()) return;
    }

    load_shortcuts();
    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
}

void ShortcutsWindow::hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
    }
}

bool ShortcutsWindow::is_visible() const {
    return hwnd_ && IsWindowVisible(hwnd_);
}

bool ShortcutsWindow::create_window() {
    auto& app = gonhanh::App::instance();

    // Register window class
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(wc);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = wnd_proc;
    wc.hInstance = app.hinstance();
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.lpszClassName = SHORTCUTS_WINDOW_CLASS;

    if (!GetClassInfoExW(app.hinstance(), SHORTCUTS_WINDOW_CLASS, &wc)) {
        RegisterClassExW(&wc);
    }

    // Calculate window position (center screen)
    int screen_width = GetSystemMetrics(SM_CXSCREEN);
    int screen_height = GetSystemMetrics(SM_CYSCREEN);
    int x = (screen_width - WIDTH) / 2;
    int y = (screen_height - HEIGHT) / 2;

    // Adjust for window chrome
    RECT rc = {0, 0, WIDTH, HEIGHT};
    AdjustWindowRectEx(&rc, WS_OVERLAPPEDWINDOW & ~WS_MAXIMIZEBOX & ~WS_THICKFRAME, FALSE, 0);

    hwnd_ = CreateWindowExW(
        0,
        SHORTCUTS_WINDOW_CLASS,
        L"Từ viết tắt",
        (WS_OVERLAPPEDWINDOW & ~WS_MAXIMIZEBOX & ~WS_THICKFRAME),
        x, y,
        rc.right - rc.left,
        rc.bottom - rc.top,
        nullptr,
        nullptr,
        app.hinstance(),
        this
    );

    if (!hwnd_) return false;

    // Create render target
    auto& renderer = D2DRenderer::instance();
    if (!renderer.is_initialized()) {
        renderer.initialize();
    }
    render_target_.Attach(renderer.create_render_target(hwnd_));

    return true;
}

void ShortcutsWindow::render() {
    if (!render_target_) return;

    render_target_->BeginDraw();
    render_target_->Clear(Colors::Background);

    auto& renderer = D2DRenderer::instance();
    auto text_brush = create_brush(render_target_.Get(), Colors::Text);
    auto text_secondary_brush = create_brush(render_target_.Get(), Colors::TextSecondary);
    auto text_tertiary_brush = create_brush(render_target_.Get(), Colors::TextTertiary);
    auto border_brush = create_brush(render_target_.Get(), Colors::Border);
    auto card_brush = create_brush(render_target_.Get(), Colors::CardBg);
    auto primary_brush = create_brush(render_target_.Get(), Colors::Primary);

    float y = PADDING;

    // Header: "Từ viết tắt (X)" with count
    std::wstring header_text = L"Từ viết tắt";
    if (!shortcuts_.empty()) {
        header_text += L" (" + std::to_wstring(shortcuts_.size()) + L")";
    }

    D2D1_RECT_F header_rect = {PADDING, y, WIDTH - PADDING, y + 30};
    render_target_->DrawText(
        header_text.c_str(),
        static_cast<UINT32>(header_text.length()),
        renderer.text_format_title(),
        header_rect,
        text_brush.Get()
    );
    y += HEADER_HEIGHT;

    // Form section: Input fields + Add/Update button
    D2D1_RECT_F form_bg = {PADDING, y, WIDTH - PADDING, y + FORM_HEIGHT};
    render_target_->FillRectangle(form_bg, card_brush.Get());
    render_target_->DrawRectangle(form_bg, border_brush.Get(), 1.0f);

    // Key input label
    D2D1_RECT_F key_label_rect = {PADDING + 12, y + 12, PADDING + 100, y + 30};
    render_target_->DrawText(
        L"Viết tắt:",
        9,
        renderer.text_format_small(),
        key_label_rect,
        text_secondary_brush.Get()
    );

    // Key input field
    D2D1_RECT_F key_input_rect = {PADDING + 12, y + 32, WIDTH / 2 - 12, y + 60};
    render_target_->FillRectangle(key_input_rect, Colors::CardBg);
    render_target_->DrawRectangle(key_input_rect, border_brush.Get(), 1.0f);
    if (!key_input_.empty()) {
        D2D1_RECT_F key_text_rect = {key_input_rect.left + 8, key_input_rect.top + 4,
                                      key_input_rect.right - 8, key_input_rect.bottom - 4};
        render_target_->DrawText(
            key_input_.c_str(),
            static_cast<UINT32>(key_input_.length()),
            renderer.text_format_body(),
            key_text_rect,
            text_brush.Get()
        );
    }

    // Value input label
    D2D1_RECT_F value_label_rect = {WIDTH / 2 + 12, y + 12, WIDTH - PADDING - 12, y + 30};
    render_target_->DrawText(
        L"Nội dung:",
        10,
        renderer.text_format_small(),
        value_label_rect,
        text_secondary_brush.Get()
    );

    // Value input field
    D2D1_RECT_F value_input_rect = {WIDTH / 2 + 12, y + 32, WIDTH - PADDING - 12 - 80, y + 60};
    render_target_->FillRectangle(value_input_rect, Colors::CardBg);
    render_target_->DrawRectangle(value_input_rect, border_brush.Get(), 1.0f);
    if (!value_input_.empty()) {
        D2D1_RECT_F value_text_rect = {value_input_rect.left + 8, value_input_rect.top + 4,
                                        value_input_rect.right - 8, value_input_rect.bottom - 4};
        render_target_->DrawText(
            value_input_.c_str(),
            static_cast<UINT32>(value_input_.length()),
            renderer.text_format_body(),
            value_text_rect,
            text_brush.Get()
        );
    }

    // Add/Update button
    D2D1_RECT_F button_rect = {WIDTH - PADDING - 80, y + 32, WIDTH - PADDING - 12, y + 60};
    const wchar_t* button_text = selected_index_ >= 0 ? L"Cập nhật" : L"Thêm";
    render_target_->FillRectangle(button_rect, primary_brush.Get());
    D2D1_RECT_F button_text_rect = {button_rect.left, button_rect.top + 6,
                                     button_rect.right, button_rect.bottom - 6};
    auto white_brush = create_brush(render_target_.Get(), D2D1::ColorF(1.0f, 1.0f, 1.0f));
    render_target_->DrawText(
        button_text,
        static_cast<UINT32>(wcslen(button_text)),
        renderer.text_format_small(),
        button_text_rect,
        white_brush.Get()
    );

    y += FORM_HEIGHT + 16;

    // Shortcuts list/table
    if (shortcuts_.empty()) {
        // Empty state
        D2D1_RECT_F empty_rect = {PADDING, y + 40, WIDTH - PADDING, y + 100};
        render_target_->DrawText(
            L"Chưa có từ viết tắt nào",
            23,
            renderer.text_format_body(),
            empty_rect,
            text_tertiary_brush.Get()
        );
    } else {
        // Table header
        D2D1_RECT_F table_bg = {PADDING, y, WIDTH - PADDING, HEIGHT - TOOLBAR_HEIGHT - 8};
        render_target_->FillRectangle(table_bg, card_brush.Get());
        render_target_->DrawRectangle(table_bg, border_brush.Get(), 1.0f);

        float table_y = y + 8;

        // Column headers
        D2D1_RECT_F col_header_rect = {PADDING + 12, table_y, WIDTH - PADDING - 12, table_y + 20};
        render_target_->DrawText(
            L"Viết tắt",
            8,
            renderer.text_format_small(),
            col_header_rect,
            text_secondary_brush.Get()
        );
        D2D1_RECT_F col_value_rect = {PADDING + 150, table_y, WIDTH - PADDING - 100, table_y + 20};
        render_target_->DrawText(
            L"Nội dung",
            8,
            renderer.text_format_small(),
            col_value_rect,
            text_secondary_brush.Get()
        );

        table_y += 28;

        // Draw separator
        render_target_->DrawLine(
            D2D1::Point2F(PADDING + 12, table_y),
            D2D1::Point2F(WIDTH - PADDING - 12, table_y),
            border_brush.Get(),
            1.0f
        );

        table_y += 4;

        // Rows
        float max_table_height = HEIGHT - TOOLBAR_HEIGHT - y - 40;
        int visible_rows = static_cast<int>(max_table_height / ROW_HEIGHT);
        int start_index = static_cast<int>(scroll_offset_ / ROW_HEIGHT);
        int end_index = std::min(start_index + visible_rows + 1, static_cast<int>(shortcuts_.size()));

        for (int i = start_index; i < end_index; ++i) {
            const auto& item = shortcuts_[i];
            float row_y = table_y + (i - start_index) * ROW_HEIGHT - (scroll_offset_ - start_index * ROW_HEIGHT);

            if (row_y + ROW_HEIGHT < table_y || row_y > table_y + max_table_height) {
                continue;
            }

            // Row background (hover state)
            if (hovered_row_ == i) {
                D2D1_RECT_F row_bg = {PADDING + 8, row_y, WIDTH - PADDING - 8, row_y + ROW_HEIGHT};
                auto hover_brush = create_brush(render_target_.Get(), D2D1::ColorF(0.96f, 0.96f, 0.97f));
                render_target_->FillRectangle(row_bg, hover_brush.Get());
            }

            // Checkbox (enabled state)
            D2D1_RECT_F checkbox_rect = {PADDING + 12, row_y + 10, PADDING + 28, row_y + 26};
            render_target_->DrawRectangle(checkbox_rect, border_brush.Get(), 1.0f);
            if (item.enabled) {
                D2D1_RECT_F check_fill = {checkbox_rect.left + 3, checkbox_rect.top + 3,
                                           checkbox_rect.right - 3, checkbox_rect.bottom - 3};
                render_target_->FillRectangle(check_fill, primary_brush.Get());
            }

            // Key text
            D2D1_RECT_F key_rect = {PADDING + 40, row_y + 8, PADDING + 150, row_y + 32};
            render_target_->DrawText(
                item.key.c_str(),
                static_cast<UINT32>(item.key.length()),
                renderer.text_format_body(),
                key_rect,
                text_brush.Get()
            );

            // Value text (truncated if too long)
            D2D1_RECT_F value_rect = {PADDING + 150, row_y + 8, WIDTH - PADDING - 80, row_y + 32};
            render_target_->DrawText(
                item.value.c_str(),
                static_cast<UINT32>(item.value.length()),
                renderer.text_format_body(),
                value_rect,
                text_secondary_brush.Get()
            );

            // Delete button
            D2D1_RECT_F delete_rect = {WIDTH - PADDING - 60, row_y + 8, WIDTH - PADDING - 20, row_y + 32};
            auto delete_text_brush = (hovered_row_ == i && hovering_delete_) ?
                create_brush(render_target_.Get(), D2D1::ColorF(0.85f, 0.27f, 0.27f)) : text_tertiary_brush;
            render_target_->DrawText(
                L"Xóa",
                3,
                renderer.text_format_small(),
                delete_rect,
                delete_text_brush.Get()
            );
        }
    }

    // Toolbar at bottom
    float toolbar_y = HEIGHT - TOOLBAR_HEIGHT;
    D2D1_RECT_F toolbar_bg = {0, toolbar_y, WIDTH, HEIGHT};
    auto toolbar_brush = create_brush(render_target_.Get(), D2D1::ColorF(0.98f, 0.98f, 0.98f));
    render_target_->FillRectangle(toolbar_bg, toolbar_brush.Get());
    render_target_->DrawLine(
        D2D1::Point2F(0, toolbar_y),
        D2D1::Point2F(WIDTH, toolbar_y),
        border_brush.Get(),
        1.0f
    );

    // Import button
    D2D1_RECT_F import_btn = {PADDING, toolbar_y + 12, PADDING + 80, toolbar_y + 38};
    render_target_->DrawRectangle(import_btn, border_brush.Get(), 1.0f);
    D2D1_RECT_F import_text_rect = {import_btn.left, import_btn.top + 6, import_btn.right, import_btn.bottom - 6};
    render_target_->DrawText(
        L"Nhập",
        4,
        renderer.text_format_small(),
        import_text_rect,
        text_brush.Get()
    );

    // Export button
    D2D1_RECT_F export_btn = {PADDING + 90, toolbar_y + 12, PADDING + 170, toolbar_y + 38};
    render_target_->DrawRectangle(export_btn, border_brush.Get(), 1.0f);
    D2D1_RECT_F export_text_rect = {export_btn.left, export_btn.top + 6, export_btn.right, export_btn.bottom - 6};
    render_target_->DrawText(
        L"Xuất",
        4,
        renderer.text_format_small(),
        export_text_rect,
        text_brush.Get()
    );

    // Done button (right aligned)
    D2D1_RECT_F done_btn = {WIDTH - PADDING - 80, toolbar_y + 12, WIDTH - PADDING, toolbar_y + 38};
    render_target_->FillRectangle(done_btn, primary_brush.Get());
    D2D1_RECT_F done_text_rect = {done_btn.left, done_btn.top + 6, done_btn.right, done_btn.bottom - 6};
    render_target_->DrawText(
        L"Xong",
        4,
        renderer.text_format_small(),
        done_text_rect,
        white_brush.Get()
    );

    render_target_->EndDraw();
}

void ShortcutsWindow::load_shortcuts() {
    shortcuts_.clear();

    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REG_SHORTCUTS_PATH, 0, KEY_READ, &key) == ERROR_SUCCESS) {
        DWORD index = 0;
        wchar_t name[256];
        DWORD name_size = 256;
        wchar_t value[1024];
        DWORD value_size = sizeof(value);
        DWORD type;

        while (RegEnumValueW(key, index, name, &name_size, nullptr, &type,
                             reinterpret_cast<BYTE*>(value), &value_size) == ERROR_SUCCESS) {
            if (type == REG_SZ) {
                // Format: "enabled|value" or just "value" (enabled by default)
                std::wstring val_str(value);
                bool enabled = true;
                std::wstring actual_value;

                size_t pipe_pos = val_str.find(L'|');
                if (pipe_pos != std::wstring::npos) {
                    enabled = (val_str.substr(0, pipe_pos) == L"1");
                    actual_value = val_str.substr(pipe_pos + 1);
                } else {
                    actual_value = val_str;
                }

                shortcuts_.emplace_back(name, actual_value, enabled);
            }

            name_size = 256;
            value_size = sizeof(value);
            ++index;
        }

        RegCloseKey(key);
    }

    sync_to_engine();
}

void ShortcutsWindow::save_shortcuts() {
    // Clear existing registry entries
    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REG_SHORTCUTS_PATH, 0, KEY_ALL_ACCESS, &key) == ERROR_SUCCESS) {
        RegDeleteTreeW(key, nullptr);
        RegCloseKey(key);
    }

    // Create key and save shortcuts
    if (RegCreateKeyExW(HKEY_CURRENT_USER, REG_SHORTCUTS_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &key, nullptr) == ERROR_SUCCESS) {
        for (const auto& item : shortcuts_) {
            // Format: "enabled|value"
            std::wstring val = (item.enabled ? L"1" : L"0") + std::wstring(L"|") + item.value;
            RegSetValueExW(key, item.key.c_str(), 0, REG_SZ,
                          reinterpret_cast<const BYTE*>(val.c_str()),
                          static_cast<DWORD>((val.length() + 1) * sizeof(wchar_t)));
        }
        RegCloseKey(key);
    }

    sync_to_engine();
}

void ShortcutsWindow::sync_to_engine() {
    auto& bridge = gonhanh::RustBridge::instance();
    if (!bridge.is_loaded()) return;

    // Convert to format expected by RustBridge
    std::vector<std::tuple<std::string, std::string, bool>> shortcuts_data;
    for (const auto& item : shortcuts_) {
        // Convert wstring to UTF-8 string
        int key_len = WideCharToMultiByte(CP_UTF8, 0, item.key.c_str(), -1, nullptr, 0, nullptr, nullptr);
        int val_len = WideCharToMultiByte(CP_UTF8, 0, item.value.c_str(), -1, nullptr, 0, nullptr, nullptr);

        if (key_len > 0 && val_len > 0) {
            std::string key_utf8(key_len - 1, '\0');
            std::string val_utf8(val_len - 1, '\0');
            WideCharToMultiByte(CP_UTF8, 0, item.key.c_str(), -1, &key_utf8[0], key_len, nullptr, nullptr);
            WideCharToMultiByte(CP_UTF8, 0, item.value.c_str(), -1, &val_utf8[0], val_len, nullptr, nullptr);
            shortcuts_data.emplace_back(key_utf8, val_utf8, item.enabled);
        }
    }

    bridge.sync_shortcuts(shortcuts_data);
}

void ShortcutsWindow::add_shortcut(const std::wstring& key, const std::wstring& value) {
    if (key.empty() || value.empty()) return;

    // Check for duplicate key
    auto it = std::find_if(shortcuts_.begin(), shortcuts_.end(),
                          [&key](const ShortcutItem& item) { return item.key == key; });

    if (it != shortcuts_.end()) {
        // Update existing
        it->value = value;
    } else {
        // Add new
        shortcuts_.emplace_back(key, value, true);
    }

    save_shortcuts();
    key_input_.clear();
    value_input_.clear();
    selected_index_ = -1;
    InvalidateRect(hwnd_, nullptr, FALSE);
}

void ShortcutsWindow::update_shortcut(size_t index, const std::wstring& key, const std::wstring& value) {
    if (index >= shortcuts_.size() || key.empty() || value.empty()) return;

    shortcuts_[index].key = key;
    shortcuts_[index].value = value;
    save_shortcuts();
    key_input_.clear();
    value_input_.clear();
    selected_index_ = -1;
    InvalidateRect(hwnd_, nullptr, FALSE);
}

void ShortcutsWindow::delete_shortcut(size_t index) {
    if (index >= shortcuts_.size()) return;

    shortcuts_.erase(shortcuts_.begin() + index);
    save_shortcuts();
    InvalidateRect(hwnd_, nullptr, FALSE);
}

void ShortcutsWindow::toggle_shortcut(size_t index) {
    if (index >= shortcuts_.size()) return;

    shortcuts_[index].enabled = !shortcuts_[index].enabled;
    save_shortcuts();
    InvalidateRect(hwnd_, nullptr, FALSE);
}

void ShortcutsWindow::import_shortcuts() {
    wchar_t file_path[MAX_PATH] = {};
    OPENFILENAMEW ofn = {};
    ofn.lStructSize = sizeof(ofn);
    ofn.hwndOwner = hwnd_;
    ofn.lpstrFile = file_path;
    ofn.nMaxFile = MAX_PATH;
    ofn.lpstrFilter = L"Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0";
    ofn.lpstrTitle = L"Nhập từ viết tắt";
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;

    if (GetOpenFileNameW(&ofn)) {
        std::wifstream file(file_path);
        if (file.is_open()) {
            shortcuts_.clear();
            std::wstring line;
            while (std::getline(file, line)) {
                size_t sep = line.find(L'\t');
                if (sep != std::wstring::npos) {
                    std::wstring key = line.substr(0, sep);
                    std::wstring value = line.substr(sep + 1);
                    if (!key.empty() && !value.empty()) {
                        shortcuts_.emplace_back(key, value, true);
                    }
                }
            }
            file.close();
            save_shortcuts();
            InvalidateRect(hwnd_, nullptr, FALSE);
        }
    }
}

void ShortcutsWindow::export_shortcuts() {
    if (shortcuts_.empty()) return;

    wchar_t file_path[MAX_PATH] = {};
    OPENFILENAMEW ofn = {};
    ofn.lStructSize = sizeof(ofn);
    ofn.hwndOwner = hwnd_;
    ofn.lpstrFile = file_path;
    ofn.nMaxFile = MAX_PATH;
    ofn.lpstrFilter = L"Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0";
    ofn.lpstrTitle = L"Xuất từ viết tắt";
    ofn.lpstrDefExt = L"txt";
    ofn.Flags = OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST;

    if (GetSaveFileNameW(&ofn)) {
        std::wofstream file(file_path);
        if (file.is_open()) {
            for (const auto& item : shortcuts_) {
                file << item.key << L'\t' << item.value << L'\n';
            }
            file.close();
        }
    }
}

LRESULT CALLBACK ShortcutsWindow::wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    ShortcutsWindow* self = nullptr;

    if (msg == WM_NCCREATE) {
        auto* cs = reinterpret_cast<CREATESTRUCTW*>(lparam);
        self = static_cast<ShortcutsWindow*>(cs->lpCreateParams);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
    } else {
        self = reinterpret_cast<ShortcutsWindow*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    }

    switch (msg) {
        case WM_PAINT: {
            PAINTSTRUCT ps;
            BeginPaint(hwnd, &ps);
            if (self) self->render();
            EndPaint(hwnd, &ps);
            return 0;
        }

        case WM_SIZE:
            if (self && self->render_target_) {
                D2D1_SIZE_U size = {LOWORD(lparam), HIWORD(lparam)};
                self->render_target_->Resize(size);
            }
            return 0;

        case WM_LBUTTONDOWN:
            if (self) {
                int x = GET_X_LPARAM(lparam);
                int y = GET_Y_LPARAM(lparam);

                // Check toolbar buttons
                float toolbar_y = HEIGHT - TOOLBAR_HEIGHT;
                if (y >= toolbar_y + 12 && y <= toolbar_y + 38) {
                    // Import button
                    if (x >= PADDING && x <= PADDING + 80) {
                        self->import_shortcuts();
                        return 0;
                    }
                    // Export button
                    if (x >= PADDING + 90 && x <= PADDING + 170) {
                        self->export_shortcuts();
                        return 0;
                    }
                    // Done button
                    if (x >= WIDTH - PADDING - 80 && x <= WIDTH - PADDING) {
                        self->hide();
                        return 0;
                    }
                }

                // Check Add/Update button
                float form_y = PADDING + HEADER_HEIGHT;
                if (y >= form_y + 32 && y <= form_y + 60 &&
                    x >= WIDTH - PADDING - 80 && x <= WIDTH - PADDING - 12) {
                    if (self->selected_index_ >= 0) {
                        self->update_shortcut(self->selected_index_, self->key_input_, self->value_input_);
                    } else {
                        self->add_shortcut(self->key_input_, self->value_input_);
                    }
                    return 0;
                }

                // Check shortcuts list clicks (checkbox, row, delete)
                float table_y = form_y + FORM_HEIGHT + 16 + 32;
                if (!self->shortcuts_.empty() && y >= table_y) {
                    int row_index = static_cast<int>((y - table_y + self->scroll_offset_) / ROW_HEIGHT);
                    if (row_index >= 0 && row_index < static_cast<int>(self->shortcuts_.size())) {
                        // Checkbox click
                        if (x >= PADDING + 12 && x <= PADDING + 28) {
                            self->toggle_shortcut(row_index);
                        }
                        // Delete button click
                        else if (x >= WIDTH - PADDING - 60 && x <= WIDTH - PADDING - 20) {
                            self->delete_shortcut(row_index);
                        }
                        // Row click (edit)
                        else {
                            self->selected_index_ = row_index;
                            self->key_input_ = self->shortcuts_[row_index].key;
                            self->value_input_ = self->shortcuts_[row_index].value;
                            InvalidateRect(hwnd, nullptr, FALSE);
                        }
                    }
                }
            }
            return 0;

        case WM_MOUSEWHEEL: {
            if (self && !self->shortcuts_.empty()) {
                int delta = GET_WHEEL_DELTA_WPARAM(wparam);
                self->scroll_offset_ -= (delta > 0 ? SCROLL_AMOUNT : -SCROLL_AMOUNT);

                float max_scroll = std::max(0.0f, self->shortcuts_.size() * ROW_HEIGHT - 200.0f);
                self->scroll_offset_ = std::max(0.0f, std::min(self->scroll_offset_, max_scroll));

                InvalidateRect(hwnd, nullptr, FALSE);
            }
            return 0;
        }

        case WM_MOUSEMOVE: {
            if (self && !self->shortcuts_.empty()) {
                int x = GET_X_LPARAM(lparam);
                int y = GET_Y_LPARAM(lparam);

                float form_y = PADDING + HEADER_HEIGHT;
                float table_y = form_y + FORM_HEIGHT + 16 + 32;

                int old_hovered = self->hovered_row_;
                self->hovered_row_ = -1;
                self->hovering_delete_ = false;

                if (y >= table_y) {
                    int row_index = static_cast<int>((y - table_y + self->scroll_offset_) / ROW_HEIGHT);
                    if (row_index >= 0 && row_index < static_cast<int>(self->shortcuts_.size())) {
                        self->hovered_row_ = row_index;
                        if (x >= WIDTH - PADDING - 60 && x <= WIDTH - PADDING - 20) {
                            self->hovering_delete_ = true;
                        }
                    }
                }

                if (old_hovered != self->hovered_row_) {
                    InvalidateRect(hwnd, nullptr, FALSE);
                }
            }
            return 0;
        }

        case WM_CHAR:
            if (self) {
                // Simple text input handling (for demonstration)
                wchar_t ch = static_cast<wchar_t>(wparam);
                if (ch >= 32 && ch < 127) {
                    // Add to current input field (simplified - would need focus management)
                    if (self->key_input_.length() < 20) {
                        self->key_input_ += ch;
                    } else if (self->value_input_.length() < 100) {
                        self->value_input_ += ch;
                    }
                    InvalidateRect(hwnd, nullptr, FALSE);
                } else if (ch == VK_BACK) {
                    if (!self->value_input_.empty()) {
                        self->value_input_.pop_back();
                    } else if (!self->key_input_.empty()) {
                        self->key_input_.pop_back();
                    }
                    InvalidateRect(hwnd, nullptr, FALSE);
                }
            }
            return 0;

        case WM_CLOSE:
            ShowWindow(hwnd, SW_HIDE);
            return 0;

        case WM_DESTROY:
            return 0;

        default:
            return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
}

} // namespace gonhanh::ui
