#include "per_app_window.h"
#include "d2d_renderer.h"
#include "../app.h"
#include "../settings.h"
#include <psapi.h>
#include <tlhelp32.h>
#include <commdlg.h>
#include <shellapi.h>
#include <algorithm>

#pragma comment(lib, "psapi.lib")

namespace gonhanh::ui {

static constexpr const wchar_t* PER_APP_WINDOW_CLASS = L"GoNhanhPerAppClass";

PerAppWindow& PerAppWindow::instance() {
    static PerAppWindow instance;
    return instance;
}

PerAppWindow::~PerAppWindow() {
    // Clean up icons
    for (auto& app : running_apps_) {
        if (app.icon) {
            DestroyIcon(app.icon);
        }
    }

    if (hwnd_) {
        DestroyWindow(hwnd_);
        hwnd_ = nullptr;
    }
}

void PerAppWindow::show() {
    if (!hwnd_) {
        if (!create_window()) return;
    }

    load_disabled_apps();
    refresh_running_apps();

    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
}

void PerAppWindow::hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
    }
    show_running_apps_ = false;
}

bool PerAppWindow::is_visible() const {
    return hwnd_ && IsWindowVisible(hwnd_);
}

bool PerAppWindow::create_window() {
    auto& app = gonhanh::App::instance();

    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(wc);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = wnd_proc;
    wc.hInstance = app.hinstance();
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.lpszClassName = PER_APP_WINDOW_CLASS;

    if (!GetClassInfoExW(app.hinstance(), PER_APP_WINDOW_CLASS, &wc)) {
        RegisterClassExW(&wc);
    }

    int screen_width = GetSystemMetrics(SM_CXSCREEN);
    int screen_height = GetSystemMetrics(SM_CYSCREEN);
    int x = (screen_width - WIDTH) / 2;
    int y = (screen_height - HEIGHT) / 2;

    RECT rc = {0, 0, WIDTH, HEIGHT};
    AdjustWindowRectEx(&rc, WS_OVERLAPPEDWINDOW & ~WS_MAXIMIZEBOX & ~WS_THICKFRAME, FALSE, 0);

    hwnd_ = CreateWindowExW(
        0,
        PER_APP_WINDOW_CLASS,
        L"Quản lý ứng dụng",
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

    auto& renderer = D2DRenderer::instance();
    if (!renderer.is_initialized()) {
        renderer.initialize();
    }
    render_target_.Attach(renderer.create_render_target(hwnd_));

    return true;
}

void PerAppWindow::load_disabled_apps() {
    disabled_apps_ = Settings::instance().get_disabled_apps();
}

void PerAppWindow::refresh_running_apps() {
    // Clean up old icons
    for (auto& app : running_apps_) {
        if (app.icon) {
            DestroyIcon(app.icon);
        }
    }
    running_apps_.clear();

    // Get list of running processes
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snapshot == INVALID_HANDLE_VALUE) return;

    PROCESSENTRY32W pe = {};
    pe.dwSize = sizeof(pe);

    std::vector<std::wstring> seen;

    if (Process32FirstW(snapshot, &pe)) {
        do {
            std::wstring name = pe.szExeFile;

            // Skip system processes
            if (name == L"System" || name == L"svchost.exe" ||
                name == L"csrss.exe" || name == L"smss.exe" ||
                name == L"wininit.exe" || name == L"services.exe" ||
                name == L"lsass.exe" || name == L"winlogon.exe" ||
                name == L"conhost.exe" || name == L"dwm.exe" ||
                name == L"GoNhanh.exe") {
                continue;
            }

            // Skip if already seen
            if (std::find(seen.begin(), seen.end(), name) != seen.end()) {
                continue;
            }
            seen.push_back(name);

            // Skip if already disabled
            if (std::find(disabled_apps_.begin(), disabled_apps_.end(), name) != disabled_apps_.end()) {
                continue;
            }

            // Get full path and icon
            HANDLE process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pe.th32ProcessID);
            if (process) {
                wchar_t path[MAX_PATH] = {};
                DWORD path_size = MAX_PATH;
                if (QueryFullProcessImageNameW(process, 0, path, &path_size)) {
                    RunningApp app_info;
                    app_info.name = name;
                    app_info.path = path;

                    // Extract icon
                    SHFILEINFOW sfi = {};
                    if (SHGetFileInfoW(path, 0, &sfi, sizeof(sfi), SHGFI_ICON | SHGFI_SMALLICON)) {
                        app_info.icon = sfi.hIcon;
                    }

                    running_apps_.push_back(std::move(app_info));
                }
                CloseHandle(process);
            }

            // Limit to 20 apps
            if (running_apps_.size() >= 20) break;

        } while (Process32NextW(snapshot, &pe));
    }

    CloseHandle(snapshot);

    // Sort by name
    std::sort(running_apps_.begin(), running_apps_.end(),
              [](const RunningApp& a, const RunningApp& b) {
                  return _wcsicmp(a.name.c_str(), b.name.c_str()) < 0;
              });
}

void PerAppWindow::render() {
    if (!render_target_) return;

    render_target_->BeginDraw();
    render_target_->Clear(Colors::Background);

    auto& renderer = D2DRenderer::instance();
    const float padding = 24.0f;
    float y = padding;

    // Title
    auto title_brush = create_brush(render_target_.Get(), Colors::Text);
    ComPtr<IDWriteTextFormat> title_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI", nullptr,
        DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL, 18.0f, L"vi-VN", &title_format
    );

    D2D1_RECT_F title_rect = {padding, y, WIDTH - padding, y + 28};
    render_target_->DrawText(L"Ứng dụng bị tắt", 15, title_format.Get(), title_rect, title_brush.Get());
    y += 36.0f;

    // Description
    auto secondary_brush = create_brush(render_target_.Get(), Colors::TextSecondary);
    ComPtr<IDWriteTextFormat> body_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI", nullptr,
        DWRITE_FONT_WEIGHT_REGULAR, DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL, 13.0f, L"vi-VN", &body_format
    );

    const wchar_t* desc = L"Gõ Nhanh sẽ tự động tắt khi bạn sử dụng các ứng dụng này.";
    D2D1_RECT_F desc_rect = {padding, y, WIDTH - padding, y + 20};
    render_target_->DrawText(desc, wcslen(desc), body_format.Get(), desc_rect, secondary_brush.Get());
    y += 32.0f;

    // Disabled apps list area
    const float list_height = 200.0f;
    const float item_height = 40.0f;
    auto border_brush = create_brush(render_target_.Get(), Colors::Border);

    // List background
    D2D1_ROUNDED_RECT list_bg = {
        D2D1::RectF(padding, y, WIDTH - padding, y + list_height),
        8.0f, 8.0f
    };
    auto card_brush = create_brush(render_target_.Get(), Colors::CardBg);
    render_target_->FillRoundedRectangle(list_bg, card_brush.Get());
    render_target_->DrawRoundedRectangle(list_bg, border_brush.Get(), 1.0f);

    // Render disabled apps
    if (disabled_apps_.empty()) {
        // Empty state
        auto tertiary_brush = create_brush(render_target_.Get(), Colors::TextTertiary);
        body_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
        D2D1_RECT_F empty_rect = {padding, y + list_height / 2 - 10, WIDTH - padding, y + list_height / 2 + 10};
        render_target_->DrawText(L"Chưa có ứng dụng nào bị tắt", 26, body_format.Get(), empty_rect, tertiary_brush.Get());
        body_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
    } else {
        // Clip to list area
        render_target_->PushAxisAlignedClip(
            D2D1::RectF(padding + 1, y + 1, WIDTH - padding - 1, y + list_height - 1),
            D2D1_ANTIALIAS_MODE_PER_PRIMITIVE
        );

        float item_y = y + 4 - scroll_offset_;
        for (size_t i = 0; i < disabled_apps_.size(); i++) {
            if (item_y + item_height < y || item_y > y + list_height) {
                item_y += item_height;
                continue;
            }

            D2D1_RECT_F item_rect = {padding + 8, item_y, WIDTH - padding - 8, item_y + item_height};

            // Hover highlight
            if (static_cast<int>(i) == hovered_item_) {
                auto hover_brush = create_brush(render_target_.Get(), D2D1::ColorF(0.0f, 0.0f, 0.0f, 0.03f));
                D2D1_ROUNDED_RECT hover_rect = {item_rect, 4.0f, 4.0f};
                render_target_->FillRoundedRectangle(hover_rect, hover_brush.Get());
            }

            // App name
            D2D1_RECT_F name_rect = {padding + 16, item_y + 10, WIDTH - padding - 50, item_y + 30};
            render_target_->DrawText(disabled_apps_[i].c_str(), disabled_apps_[i].length(),
                                     body_format.Get(), name_rect, title_brush.Get());

            // Delete button (X)
            D2D1_RECT_F del_rect = {WIDTH - padding - 40, item_y + 8, WIDTH - padding - 16, item_y + 32};
            auto del_color = (static_cast<int>(i) == hovered_delete_) ?
                D2D1::ColorF(0.9f, 0.3f, 0.3f) : Colors::TextSecondary;
            auto del_brush = create_brush(render_target_.Get(), del_color);

            // Draw X
            render_target_->DrawLine(
                D2D1::Point2F(del_rect.left + 4, del_rect.top + 4),
                D2D1::Point2F(del_rect.right - 4, del_rect.bottom - 4),
                del_brush.Get(), 2.0f
            );
            render_target_->DrawLine(
                D2D1::Point2F(del_rect.right - 4, del_rect.top + 4),
                D2D1::Point2F(del_rect.left + 4, del_rect.bottom - 4),
                del_brush.Get(), 2.0f
            );

            item_y += item_height;
        }

        render_target_->PopAxisAlignedClip();
    }

    y += list_height + 16.0f;

    // Action buttons row
    const float btn_height = 36.0f;
    const float btn_spacing = 12.0f;

    // "Add from running apps" button
    D2D1_RECT_F add_btn_rect = {padding, y, padding + 180, y + btn_height};
    auto add_bg_color = hovered_add_btn_ ?
        D2D1::ColorF(Colors::Primary.r, Colors::Primary.g, Colors::Primary.b, 0.1f) :
        D2D1::ColorF(0.0f, 0.0f, 0.0f, 0.03f);
    auto add_bg_brush = create_brush(render_target_.Get(), add_bg_color);
    D2D1_ROUNDED_RECT add_btn_rounded = {add_btn_rect, 6.0f, 6.0f};
    render_target_->FillRoundedRectangle(add_btn_rounded, add_bg_brush.Get());

    auto add_border_color = hovered_add_btn_ ? Colors::Primary : Colors::Border;
    auto add_border_brush = create_brush(render_target_.Get(), add_border_color);
    render_target_->DrawRoundedRectangle(add_btn_rounded, add_border_brush.Get(), 1.0f);

    auto add_text_brush = create_brush(render_target_.Get(), hovered_add_btn_ ? Colors::Primary : Colors::Text);
    body_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
    render_target_->DrawText(L"+ Thêm từ ứng dụng", 18, body_format.Get(), add_btn_rect, add_text_brush.Get());

    // "Refresh" button
    D2D1_RECT_F refresh_btn_rect = {padding + 180 + btn_spacing, y, padding + 180 + btn_spacing + 100, y + btn_height};
    auto refresh_bg_color = hovered_refresh_btn_ ?
        D2D1::ColorF(Colors::Primary.r, Colors::Primary.g, Colors::Primary.b, 0.1f) :
        D2D1::ColorF(0.0f, 0.0f, 0.0f, 0.03f);
    auto refresh_bg_brush = create_brush(render_target_.Get(), refresh_bg_color);
    D2D1_ROUNDED_RECT refresh_btn_rounded = {refresh_btn_rect, 6.0f, 6.0f};
    render_target_->FillRoundedRectangle(refresh_btn_rounded, refresh_bg_brush.Get());

    auto refresh_border_color = hovered_refresh_btn_ ? Colors::Primary : Colors::Border;
    auto refresh_border_brush = create_brush(render_target_.Get(), refresh_border_color);
    render_target_->DrawRoundedRectangle(refresh_btn_rounded, refresh_border_brush.Get(), 1.0f);

    auto refresh_text_brush = create_brush(render_target_.Get(), hovered_refresh_btn_ ? Colors::Primary : Colors::Text);
    render_target_->DrawText(L"Làm mới", 7, body_format.Get(), refresh_btn_rect, refresh_text_brush.Get());

    y += btn_height + 16.0f;

    // Running apps dropdown (if showing)
    if (show_running_apps_ && !running_apps_.empty()) {
        const float dropdown_height = min(200.0f, running_apps_.size() * item_height + 8);

        D2D1_ROUNDED_RECT dropdown_bg = {
            D2D1::RectF(padding, y, WIDTH - padding, y + dropdown_height),
            8.0f, 8.0f
        };
        render_target_->FillRoundedRectangle(dropdown_bg, card_brush.Get());

        // Shadow effect (simple border)
        auto shadow_brush = create_brush(render_target_.Get(), D2D1::ColorF(0.0f, 0.0f, 0.0f, 0.15f));
        render_target_->DrawRoundedRectangle(dropdown_bg, shadow_brush.Get(), 1.0f);

        // Clip dropdown
        render_target_->PushAxisAlignedClip(
            D2D1::RectF(padding + 1, y + 1, WIDTH - padding - 1, y + dropdown_height - 1),
            D2D1_ANTIALIAS_MODE_PER_PRIMITIVE
        );

        float item_y = y + 4;
        for (size_t i = 0; i < running_apps_.size(); i++) {
            D2D1_RECT_F item_rect = {padding + 8, item_y, WIDTH - padding - 8, item_y + item_height};

            // Hover highlight
            if (static_cast<int>(i) == hovered_running_) {
                auto hover_brush = create_brush(render_target_.Get(), D2D1::ColorF(Colors::Primary.r, Colors::Primary.g, Colors::Primary.b, 0.1f));
                D2D1_ROUNDED_RECT hover_rect = {item_rect, 4.0f, 4.0f};
                render_target_->FillRoundedRectangle(hover_rect, hover_brush.Get());
            }

            // Icon (if available) - would need GDI interop for HICON
            // For now, just show name
            D2D1_RECT_F name_rect = {padding + 16, item_y + 10, WIDTH - padding - 16, item_y + 30};
            auto text_brush = create_brush(render_target_.Get(),
                static_cast<int>(i) == hovered_running_ ? Colors::Primary : Colors::Text);
            render_target_->DrawText(running_apps_[i].name.c_str(), running_apps_[i].name.length(),
                                     body_format.Get(), name_rect, text_brush.Get());

            item_y += item_height;
        }

        render_target_->PopAxisAlignedClip();
    }

    body_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
    render_target_->EndDraw();
}

void PerAppWindow::handle_mouse_move(int x, int y) {
    const float padding = 24.0f;
    const float list_y = 24.0f + 36.0f + 32.0f;  // title + desc
    const float list_height = 200.0f;
    const float item_height = 40.0f;

    int old_hovered_item = hovered_item_;
    int old_hovered_delete = hovered_delete_;
    int old_hovered_running = hovered_running_;
    bool old_hovered_add = hovered_add_btn_;
    bool old_hovered_refresh = hovered_refresh_btn_;

    hovered_item_ = -1;
    hovered_delete_ = -1;
    hovered_running_ = -1;
    hovered_add_btn_ = false;
    hovered_refresh_btn_ = false;

    float fy = static_cast<float>(y);
    float fx = static_cast<float>(x);

    // Check disabled apps list
    if (fx >= padding && fx <= WIDTH - padding &&
        fy >= list_y && fy <= list_y + list_height) {

        float item_y = list_y + 4 - scroll_offset_;
        for (size_t i = 0; i < disabled_apps_.size(); i++) {
            if (fy >= item_y && fy < item_y + item_height) {
                hovered_item_ = static_cast<int>(i);

                // Check delete button
                if (fx >= WIDTH - padding - 40 && fx <= WIDTH - padding - 16) {
                    hovered_delete_ = static_cast<int>(i);
                }
                break;
            }
            item_y += item_height;
        }
    }

    // Check buttons
    float btn_y = list_y + list_height + 16.0f;
    if (fy >= btn_y && fy <= btn_y + 36.0f) {
        if (fx >= padding && fx <= padding + 180) {
            hovered_add_btn_ = true;
        } else if (fx >= padding + 192 && fx <= padding + 292) {
            hovered_refresh_btn_ = true;
        }
    }

    // Check running apps dropdown
    if (show_running_apps_) {
        float dropdown_y = btn_y + 36.0f + 16.0f;
        float dropdown_height = min(200.0f, running_apps_.size() * item_height + 8);

        if (fx >= padding && fx <= WIDTH - padding &&
            fy >= dropdown_y && fy <= dropdown_y + dropdown_height) {

            float item_y = dropdown_y + 4;
            for (size_t i = 0; i < running_apps_.size(); i++) {
                if (fy >= item_y && fy < item_y + item_height) {
                    hovered_running_ = static_cast<int>(i);
                    break;
                }
                item_y += item_height;
            }
        }
    }

    // Update cursor
    bool any_hover = hovered_item_ >= 0 || hovered_delete_ >= 0 ||
                     hovered_running_ >= 0 || hovered_add_btn_ || hovered_refresh_btn_;
    SetCursor(LoadCursor(nullptr, any_hover ? IDC_HAND : IDC_ARROW));

    // Redraw if changed
    if (hovered_item_ != old_hovered_item ||
        hovered_delete_ != old_hovered_delete ||
        hovered_running_ != old_hovered_running ||
        hovered_add_btn_ != old_hovered_add ||
        hovered_refresh_btn_ != old_hovered_refresh) {
        InvalidateRect(hwnd_, nullptr, FALSE);
    }
}

void PerAppWindow::handle_click(int x, int y) {
    (void)x;
    (void)y;

    if (hovered_delete_ >= 0 && hovered_delete_ < static_cast<int>(disabled_apps_.size())) {
        remove_app(hovered_delete_);
        return;
    }

    if (hovered_add_btn_) {
        show_running_apps_ = !show_running_apps_;
        if (show_running_apps_) {
            refresh_running_apps();
        }
        InvalidateRect(hwnd_, nullptr, FALSE);
        return;
    }

    if (hovered_refresh_btn_) {
        refresh_running_apps();
        InvalidateRect(hwnd_, nullptr, FALSE);
        return;
    }

    if (hovered_running_ >= 0 && hovered_running_ < static_cast<int>(running_apps_.size())) {
        add_running_app(hovered_running_);
        return;
    }

    // Click outside dropdown closes it
    if (show_running_apps_ && hovered_running_ < 0) {
        show_running_apps_ = false;
        InvalidateRect(hwnd_, nullptr, FALSE);
    }
}

void PerAppWindow::handle_scroll(int delta) {
    if (disabled_apps_.empty()) return;

    const float item_height = 40.0f;
    const float list_height = 200.0f;
    float max_scroll = max(0.0f, disabled_apps_.size() * item_height - list_height + 8);

    scroll_offset_ -= delta / 3;
    scroll_offset_ = max(0, min(scroll_offset_, static_cast<int>(max_scroll)));

    InvalidateRect(hwnd_, nullptr, FALSE);
}

void PerAppWindow::add_running_app(int index) {
    if (index < 0 || index >= static_cast<int>(running_apps_.size())) return;

    const auto& app = running_apps_[index];
    Settings::instance().set_app_disabled(app.name, true);
    disabled_apps_.push_back(app.name);

    // Remove from running list
    running_apps_.erase(running_apps_.begin() + index);
    hovered_running_ = -1;

    if (running_apps_.empty()) {
        show_running_apps_ = false;
    }

    InvalidateRect(hwnd_, nullptr, FALSE);
}

void PerAppWindow::remove_app(int index) {
    if (index < 0 || index >= static_cast<int>(disabled_apps_.size())) return;

    const auto& app_name = disabled_apps_[index];
    Settings::instance().remove_disabled_app(app_name);
    disabled_apps_.erase(disabled_apps_.begin() + index);

    hovered_item_ = -1;
    hovered_delete_ = -1;

    InvalidateRect(hwnd_, nullptr, FALSE);
}

void PerAppWindow::add_app_from_picker() {
    OPENFILENAMEW ofn = {};
    wchar_t file[MAX_PATH] = {};

    ofn.lStructSize = sizeof(ofn);
    ofn.hwndOwner = hwnd_;
    ofn.lpstrFilter = L"Executables (*.exe)\0*.exe\0All Files (*.*)\0*.*\0";
    ofn.lpstrFile = file;
    ofn.nMaxFile = MAX_PATH;
    ofn.lpstrTitle = L"Chọn ứng dụng";
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;

    if (GetOpenFileNameW(&ofn)) {
        // Extract filename from path
        std::wstring path = file;
        size_t pos = path.find_last_of(L"\\/");
        std::wstring name = (pos != std::wstring::npos) ? path.substr(pos + 1) : path;

        if (!name.empty()) {
            Settings::instance().set_app_disabled(name, true);
            load_disabled_apps();
            InvalidateRect(hwnd_, nullptr, FALSE);
        }
    }
}

LRESULT CALLBACK PerAppWindow::wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    PerAppWindow* self = nullptr;

    if (msg == WM_NCCREATE) {
        auto* cs = reinterpret_cast<CREATESTRUCTW*>(lparam);
        self = static_cast<PerAppWindow*>(cs->lpCreateParams);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
    } else {
        self = reinterpret_cast<PerAppWindow*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
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

        case WM_MOUSEMOVE: {
            if (self) {
                TRACKMOUSEEVENT tme = {};
                tme.cbSize = sizeof(tme);
                tme.dwFlags = TME_LEAVE;
                tme.hwndTrack = hwnd;
                TrackMouseEvent(&tme);

                self->handle_mouse_move(LOWORD(lparam), HIWORD(lparam));
            }
            return 0;
        }

        case WM_MOUSELEAVE:
            if (self) {
                self->hovered_item_ = -1;
                self->hovered_delete_ = -1;
                self->hovered_running_ = -1;
                self->hovered_add_btn_ = false;
                self->hovered_refresh_btn_ = false;
                InvalidateRect(hwnd, nullptr, FALSE);
            }
            return 0;

        case WM_LBUTTONDOWN:
            if (self) {
                self->handle_click(LOWORD(lparam), HIWORD(lparam));
            }
            return 0;

        case WM_MOUSEWHEEL:
            if (self) {
                int delta = GET_WHEEL_DELTA_WPARAM(wparam);
                self->handle_scroll(delta);
            }
            return 0;

        case WM_CLOSE:
            ShowWindow(hwnd, SW_HIDE);
            if (self) {
                self->show_running_apps_ = false;
            }
            return 0;

        case WM_DESTROY:
            return 0;

        default:
            return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
}

} // namespace gonhanh::ui
