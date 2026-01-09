#include "settings_window.h"
#include "d2d_renderer.h"
#include "controls/card.h"
#include "controls/toggle.h"
#include "controls/sidebar.h"
#include "controls/button.h"
#include "../app.h"
#include "../settings.h"
#include <shellapi.h>
#include <windowsx.h>

namespace gonhanh::ui {

static constexpr const wchar_t* SETTINGS_WINDOW_CLASS = L"GoNhanhSettingsClass";
static constexpr const wchar_t* APP_VERSION = L"1.0.0";

// Layout constants
static constexpr float CONTENT_PADDING = 24.0f;
static constexpr float CARD_SPACING = 16.0f;
static constexpr float ROW_HEIGHT = 44.0f;

SettingsWindow& SettingsWindow::instance() {
    static SettingsWindow instance;
    return instance;
}

SettingsWindow::~SettingsWindow() {
    if (hwnd_) {
        DestroyWindow(hwnd_);
        hwnd_ = nullptr;
    }
}

void SettingsWindow::show() {
    if (!hwnd_) {
        if (!create_window()) return;
    }

    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
}

void SettingsWindow::hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
    }
}

bool SettingsWindow::is_visible() const {
    return hwnd_ && IsWindowVisible(hwnd_);
}

bool SettingsWindow::create_window() {
    auto& app = gonhanh::App::instance();

    // Register window class
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(wc);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = wnd_proc;
    wc.hInstance = app.hinstance();
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.lpszClassName = SETTINGS_WINDOW_CLASS;

    if (!GetClassInfoExW(app.hinstance(), SETTINGS_WINDOW_CLASS, &wc)) {
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
        SETTINGS_WINDOW_CLASS,
        L"GoNhanh Cài đặt",
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

void SettingsWindow::render() {
    if (!render_target_) return;

    render_target_->BeginDraw();
    render_target_->Clear(Colors::Background);

    render_sidebar();
    render_content();

    render_target_->EndDraw();
}

void SettingsWindow::render_sidebar() {
    // Sidebar background
    auto sidebar_brush = create_brush(render_target_.Get(), D2D1::ColorF(0.95f, 0.95f, 0.96f));
    D2D1_RECT_F sidebar_rect = {0, 0, static_cast<float>(SIDEBAR_WIDTH), static_cast<float>(HEIGHT)};
    render_target_->FillRectangle(sidebar_rect, sidebar_brush.Get());

    // Border line
    auto border_brush = create_brush(render_target_.Get(), Colors::Border);
    render_target_->DrawLine(
        D2D1::Point2F(static_cast<float>(SIDEBAR_WIDTH), 0),
        D2D1::Point2F(static_cast<float>(SIDEBAR_WIDTH), static_cast<float>(HEIGHT)),
        border_brush.Get(),
        1.0f
    );

    auto& renderer = D2DRenderer::instance();

    // Logo area at top
    D2D1_RECT_F logo_rect = {20, 40, static_cast<float>(SIDEBAR_WIDTH - 20), 80};
    auto logo_brush = create_brush(render_target_.Get(), Colors::Primary);
    render_target_->DrawText(
        L"GoNhanh",
        8,
        renderer.text_format_title(),
        logo_rect,
        logo_brush.Get()
    );

    // Navigation items
    float nav_y = static_cast<float>(HEIGHT) - 100;
    float item_x = 12;
    float item_width = static_cast<float>(SIDEBAR_WIDTH) - 24;

    // Settings button
    Sidebar::draw_item(
        render_target_.Get(),
        item_x, nav_y,
        item_width,
        L"Cài đặt",
        current_page_ == Page::Settings,
        hover_sidebar_item_ == 0
    );

    // About button
    nav_y += Sidebar::ITEM_HEIGHT + 4;
    Sidebar::draw_item(
        render_target_.Get(),
        item_x, nav_y,
        item_width,
        L"Giới thiệu",
        current_page_ == Page::About,
        hover_sidebar_item_ == 1
    );

    // Version badge at bottom
    nav_y = static_cast<float>(HEIGHT) - 30;
    D2D1_RECT_F version_rect = {20, nav_y, static_cast<float>(SIDEBAR_WIDTH - 20), nav_y + 20};
    auto version_brush = create_brush(render_target_.Get(), Colors::TextTertiary);

    std::wstring version_text = L"v";
    version_text += APP_VERSION;

    render_target_->DrawText(
        version_text.c_str(),
        static_cast<UINT32>(version_text.length()),
        renderer.text_format_small(),
        version_rect,
        version_brush.Get()
    );
}

void SettingsWindow::render_content() {
    switch (current_page_) {
        case Page::Settings:
            render_settings_page();
            break;
        case Page::About:
            render_about_page();
            break;
    }
}

void SettingsWindow::render_settings_page() {
    auto& settings = gonhanh::Settings::instance();
    auto& renderer = D2DRenderer::instance();

    float content_x = SIDEBAR_WIDTH + CONTENT_PADDING;
    float content_y = CONTENT_PADDING;
    float card_width = CONTENT_WIDTH - (CONTENT_PADDING * 2);
    float row_x = content_x + Card::PADDING;
    float toggle_x = content_x + card_width - Card::PADDING - Toggle::WIDTH;

    auto text_brush = create_brush(render_target_.Get(), Colors::Text);
    auto secondary_brush = create_brush(render_target_.Get(), Colors::TextSecondary);

    // Card 1: Input Method Settings
    {
        int num_rows = settings.input_method() == 0 ? 4 : 2; // Telex has 4 rows, VNI has 2
        float card_height = (ROW_HEIGHT * num_rows) + (Card::PADDING * 2) - 1;

        Card::draw(render_target_.Get(), content_x, content_y, card_width, card_height);

        float row_y = content_y + Card::PADDING;

        // Row 1: Bộ gõ tiếng Việt toggle
        D2D1_RECT_F text_rect = {row_x, row_y + 12, toggle_x - 8, row_y + 32};
        render_target_->DrawText(L"Bộ gõ tiếng Việt", 16, renderer.text_format_body(), text_rect, text_brush.Get());
        Toggle::draw(render_target_.Get(), toggle_x, row_y + 10, settings.is_enabled(), hover_toggle_index_ == 0);
        row_y += ROW_HEIGHT;

        // Separator
        Card::draw_separator(render_target_.Get(), content_x + Card::PADDING, row_y - 1, card_width - (Card::PADDING * 2));

        // Row 2: Kiểu gõ dropdown
        text_rect = {row_x, row_y + 12, toggle_x - 8, row_y + 32};
        render_target_->DrawText(L"Kiểu gõ", 7, renderer.text_format_body(), text_rect, text_brush.Get());

        // Draw dropdown button (simplified - shows current selection)
        float dropdown_x = toggle_x - 40;
        float dropdown_width = 100;
        const wchar_t* method_text = settings.input_method() == 0 ? L"Telex" : L"VNI";
        D2D1_RECT_F dropdown_rect = {dropdown_x, row_y + 10, dropdown_x + dropdown_width, row_y + 34};
        auto dropdown_bg = create_brush(render_target_.Get(), Colors::CardBg);
        auto dropdown_border = create_brush(render_target_.Get(), Colors::Border);

        D2D1_ROUNDED_RECT rounded_dropdown = {dropdown_rect, 6.0f, 6.0f};
        render_target_->FillRoundedRectangle(rounded_dropdown, dropdown_bg.Get());
        render_target_->DrawRoundedRectangle(rounded_dropdown, dropdown_border.Get(), 1.0f);

        D2D1_RECT_F method_text_rect = {dropdown_x + 8, row_y + 10, dropdown_x + dropdown_width - 8, row_y + 34};
        render_target_->DrawText(method_text, wcslen(method_text), renderer.text_format_body(), method_text_rect, text_brush.Get());

        row_y += ROW_HEIGHT;

        // Only show these rows for Telex
        if (settings.input_method() == 0) {
            // Separator
            Card::draw_separator(render_target_.Get(), content_x + Card::PADDING, row_y - 1, card_width - (Card::PADDING * 2));

            // Row 3: Gõ W thành Ư
            text_rect = {row_x, row_y + 12, toggle_x - 8, row_y + 32};
            render_target_->DrawText(L"Gõ W thành Ư ở đầu từ", 21, renderer.text_format_body(), text_rect, text_brush.Get());
            Toggle::draw(render_target_.Get(), toggle_x, row_y + 10, settings.w_shortcut(), hover_toggle_index_ == 1);
            row_y += ROW_HEIGHT;

            // Separator
            Card::draw_separator(render_target_.Get(), content_x + Card::PADDING, row_y - 1, card_width - (Card::PADDING * 2));

            // Row 4: Gõ ] thành Ư, [ thành Ơ
            text_rect = {row_x, row_y + 12, toggle_x - 8, row_y + 32};
            render_target_->DrawText(L"Gõ ] thành Ư, [ thành Ơ", 23, renderer.text_format_body(), text_rect, text_brush.Get());
            Toggle::draw(render_target_.Get(), toggle_x, row_y + 10, settings.bracket_shortcut(), hover_toggle_index_ == 2);
            row_y += ROW_HEIGHT;
        }

        content_y += card_height + CARD_SPACING;
    }

    // Card 2: Shortcuts
    {
        float card_height = (ROW_HEIGHT * 2) + (Card::PADDING * 2) - 1;
        Card::draw(render_target_.Get(), content_x, content_y, card_width, card_height);

        float row_y = content_y + Card::PADDING;

        // Row 1: Phím tắt bật/tắt
        D2D1_RECT_F text_rect = {row_x, row_y + 12, toggle_x - 120, row_y + 32};
        render_target_->DrawText(L"Phím tắt bật/tắt", 16, renderer.text_format_body(), text_rect, text_brush.Get());

        // Display hotkey (Ctrl+Shift+V as example)
        float hotkey_x = toggle_x - 110;
        D2D1_RECT_F hotkey_rect = {hotkey_x, row_y + 10, hotkey_x + 100, row_y + 34};
        auto hotkey_bg = create_brush(render_target_.Get(), D2D1::ColorF(0.95f, 0.95f, 0.96f));
        D2D1_ROUNDED_RECT rounded_hotkey = {hotkey_rect, 6.0f, 6.0f};
        render_target_->FillRoundedRectangle(rounded_hotkey, hotkey_bg.Get());

        D2D1_RECT_F hotkey_text_rect = {hotkey_x + 8, row_y + 10, hotkey_x + 92, row_y + 34};
        auto hotkey_text_brush = create_brush(render_target_.Get(), Colors::TextSecondary);
        render_target_->DrawText(L"Ctrl+Shift+V", 12, renderer.text_format_small(), hotkey_text_rect, hotkey_text_brush.Get());

        row_y += ROW_HEIGHT;

        // Separator
        Card::draw_separator(render_target_.Get(), content_x + Card::PADDING, row_y - 1, card_width - (Card::PADDING * 2));

        // Row 2: Bảng gõ tắt (clickable)
        text_rect = {row_x, row_y + 12, toggle_x - 40, row_y + 32};
        render_target_->DrawText(L"Bảng gõ tắt", 11, renderer.text_format_body(), text_rect, text_brush.Get());

        // Chevron
        float chevron_x = content_x + card_width - Card::PADDING - 16;
        auto chevron_brush = create_brush(render_target_.Get(), Colors::TextTertiary);
        D2D1_POINT_2F p1 = {chevron_x, row_y + 18};
        D2D1_POINT_2F p2 = {chevron_x + 6, row_y + 22};
        D2D1_POINT_2F p3 = {chevron_x, row_y + 26};
        render_target_->DrawLine(p1, p2, chevron_brush.Get(), 2.0f);
        render_target_->DrawLine(p2, p3, chevron_brush.Get(), 2.0f);

        content_y += card_height + CARD_SPACING;
    }

    // Card 3: System
    {
        float card_height = (ROW_HEIGHT * 3) + (Card::PADDING * 2) - 1;
        Card::draw(render_target_.Get(), content_x, content_y, card_width, card_height);

        float row_y = content_y + Card::PADDING;

        // Row 1: Khởi động cùng hệ thống
        D2D1_RECT_F text_rect = {row_x, row_y + 12, toggle_x - 8, row_y + 32};
        render_target_->DrawText(L"Khởi động cùng hệ thống", 23, renderer.text_format_body(), text_rect, text_brush.Get());
        Toggle::draw(render_target_.Get(), toggle_x, row_y + 10, settings.auto_start(), hover_toggle_index_ == 3);
        row_y += ROW_HEIGHT;

        // Separator
        Card::draw_separator(render_target_.Get(), content_x + Card::PADDING, row_y - 1, card_width - (Card::PADDING * 2));

        // Row 2: Tự chuyển chế độ theo ứng dụng
        text_rect = {row_x, row_y + 12, toggle_x - 8, row_y + 32};
        render_target_->DrawText(L"Tự chuyển chế độ theo ứng dụng", 30, renderer.text_format_body(), text_rect, text_brush.Get());
        Toggle::draw(render_target_.Get(), toggle_x, row_y + 10, settings.per_app_mode(), hover_toggle_index_ == 4);
        row_y += ROW_HEIGHT;

        // Separator
        Card::draw_separator(render_target_.Get(), content_x + Card::PADDING, row_y - 1, card_width - (Card::PADDING * 2));

        // Row 3: Tự khôi phục từ tiếng Anh with Beta badge
        text_rect = {row_x, row_y + 12, toggle_x - 60, row_y + 32};
        render_target_->DrawText(L"Tự khôi phục từ tiếng Anh", 25, renderer.text_format_body(), text_rect, text_brush.Get());

        // Beta badge
        float badge_x = toggle_x - 50;
        D2D1_RECT_F badge_rect = {badge_x, row_y + 14, badge_x + 40, row_y + 28};
        auto badge_brush = create_brush(render_target_.Get(), Colors::Orange);
        D2D1_ROUNDED_RECT rounded_badge = {badge_rect, 8.0f, 8.0f};
        render_target_->FillRoundedRectangle(rounded_badge, badge_brush.Get());

        D2D1_RECT_F badge_text_rect = {badge_x + 4, row_y + 14, badge_x + 36, row_y + 28};
        auto white_brush = create_brush(render_target_.Get(), D2D1::ColorF(D2D1::ColorF::White));
        render_target_->DrawText(L"Beta", 4, renderer.text_format_small(), badge_text_rect, white_brush.Get());

        Toggle::draw(render_target_.Get(), toggle_x, row_y + 10, settings.english_auto_restore(), hover_toggle_index_ == 5);

        content_y += card_height + CARD_SPACING;
    }

    // Card 4: Other Options
    {
        float card_height = (ROW_HEIGHT * 4) + (Card::PADDING * 2) - 1;
        Card::draw(render_target_.Get(), content_x, content_y, card_width, card_height);

        float row_y = content_y + Card::PADDING;

        // Row 1: Âm thanh chuyển ngôn ngữ
        D2D1_RECT_F text_rect = {row_x, row_y + 12, toggle_x - 8, row_y + 32};
        render_target_->DrawText(L"Âm thanh chuyển ngôn ngữ", 24, renderer.text_format_body(), text_rect, text_brush.Get());
        Toggle::draw(render_target_.Get(), toggle_x, row_y + 10, settings.sound_enabled(), hover_toggle_index_ == 6);
        row_y += ROW_HEIGHT;

        // Separator
        Card::draw_separator(render_target_.Get(), content_x + Card::PADDING, row_y - 1, card_width - (Card::PADDING * 2));

        // Row 2: Đặt dấu kiểu mới
        text_rect = {row_x, row_y + 12, toggle_x - 8, row_y + 32};
        render_target_->DrawText(L"Đặt dấu kiểu mới (oà, uý)", 25, renderer.text_format_body(), text_rect, text_brush.Get());
        Toggle::draw(render_target_.Get(), toggle_x, row_y + 10, settings.modern_tone(), hover_toggle_index_ == 7);
        row_y += ROW_HEIGHT;

        // Separator
        Card::draw_separator(render_target_.Get(), content_x + Card::PADDING, row_y - 1, card_width - (Card::PADDING * 2));

        // Row 3: Tự viết hoa đầu câu
        text_rect = {row_x, row_y + 12, toggle_x - 8, row_y + 32};
        render_target_->DrawText(L"Tự viết hoa đầu câu", 19, renderer.text_format_body(), text_rect, text_brush.Get());
        Toggle::draw(render_target_.Get(), toggle_x, row_y + 10, settings.auto_capitalize(), hover_toggle_index_ == 8);
        row_y += ROW_HEIGHT;

        // Separator
        Card::draw_separator(render_target_.Get(), content_x + Card::PADDING, row_y - 1, card_width - (Card::PADDING * 2));

        // Row 4: Gõ ESC hoàn tác dấu
        text_rect = {row_x, row_y + 12, toggle_x - 8, row_y + 32};
        render_target_->DrawText(L"Gõ ESC hoàn tác dấu", 19, renderer.text_format_body(), text_rect, text_brush.Get());
        Toggle::draw(render_target_.Get(), toggle_x, row_y + 10, settings.esc_restore(), hover_toggle_index_ == 9);
    }
}

void SettingsWindow::render_about_page() {
    auto& renderer = D2DRenderer::instance();

    float content_x = SIDEBAR_WIDTH + CONTENT_PADDING;
    float content_width = CONTENT_WIDTH - (CONTENT_PADDING * 2);
    float center_x = SIDEBAR_WIDTH + (CONTENT_WIDTH / 2.0f);

    auto text_brush = create_brush(render_target_.Get(), Colors::Text);
    auto secondary_brush = create_brush(render_target_.Get(), Colors::TextSecondary);
    auto primary_brush = create_brush(render_target_.Get(), Colors::Primary);

    float y = 80;

    // Logo placeholder (would be actual logo image)
    D2D1_ELLIPSE logo_circle = {{center_x, y + 40}, 40, 40};
    render_target_->FillEllipse(logo_circle, primary_brush.Get());

    y += 100;

    // App name
    D2D1_RECT_F name_rect = {content_x, y, content_x + content_width, y + 30};
    ComPtr<IDWriteTextFormat> title_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI", nullptr,
        DWRITE_FONT_WEIGHT_BOLD,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        24.0f, L"en-US",
        title_format.GetAddressOf()
    );
    title_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
    render_target_->DrawText(L"GoNhanh", 7, title_format.Get(), name_rect, text_brush.Get());

    y += 40;

    // Description
    D2D1_RECT_F desc_rect = {content_x, y, content_x + content_width, y + 20};
    ComPtr<IDWriteTextFormat> desc_format;
    renderer.dwrite_factory()->CreateTextFormat(
        L"Segoe UI", nullptr,
        DWRITE_FONT_WEIGHT_REGULAR,
        DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        13.0f, L"en-US",
        desc_format.GetAddressOf()
    );
    desc_format->SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
    render_target_->DrawText(L"Bộ gõ tiếng Việt nhanh và nhẹ", 29, desc_format.Get(), desc_rect, secondary_brush.Get());

    y += 30;

    // Version
    std::wstring version_text = L"Phiên bản ";
    version_text += APP_VERSION;
    D2D1_RECT_F version_rect = {content_x, y, content_x + content_width, y + 20};
    render_target_->DrawText(version_text.c_str(), static_cast<UINT32>(version_text.length()),
                            desc_format.Get(), version_rect, secondary_brush.Get());

    y += 40;

    // Links section
    float link_y = y;
    const wchar_t* links[] = {L"GitHub", L"Báo lỗi", L"Ủng hộ"};
    for (int i = 0; i < 3; i++) {
        D2D1_RECT_F link_rect = {content_x, link_y, content_x + content_width, link_y + 20};
        render_target_->DrawText(links[i], wcslen(links[i]), desc_format.Get(), link_rect, primary_brush.Get());
        link_y += 28;
    }

    y = link_y + 20;

    // Author info
    D2D1_RECT_F author_rect = {content_x, y, content_x + content_width, y + 20};
    render_target_->DrawText(L"Phát triển bởi Kha Phan", 23, desc_format.Get(), author_rect, secondary_brush.Get());
}

void SettingsWindow::handle_mouse_move(int x, int y) {
    mouse_x_ = x;
    mouse_y_ = y;

    int old_sidebar_item = hover_sidebar_item_;
    int old_toggle_index = hover_toggle_index_;
    bool old_shortcuts_hover = hover_shortcuts_row_;

    hover_sidebar_item_ = -1;
    hover_toggle_index_ = -1;
    hover_shortcuts_row_ = false;

    // Check sidebar items
    if (x < SIDEBAR_WIDTH) {
        float nav_y = static_cast<float>(HEIGHT) - 100;
        if (y >= nav_y && y < nav_y + Sidebar::ITEM_HEIGHT) {
            hover_sidebar_item_ = 0; // Settings
        } else if (y >= nav_y + Sidebar::ITEM_HEIGHT + 4 && y < nav_y + (Sidebar::ITEM_HEIGHT * 2) + 4) {
            hover_sidebar_item_ = 1; // About
        }
    } else if (current_page_ == Page::Settings) {
        // Check toggles in settings page
        auto& settings = gonhanh::Settings::instance();
        float content_x = SIDEBAR_WIDTH + CONTENT_PADDING;
        float card_width = CONTENT_WIDTH - (CONTENT_PADDING * 2);
        float toggle_x = content_x + card_width - Card::PADDING - Toggle::WIDTH;

        float content_y = CONTENT_PADDING;

        // Card 1: Input method (toggles at index 0, 1, 2)
        int num_rows = settings.input_method() == 0 ? 4 : 2;
        float card_height = (ROW_HEIGHT * num_rows) + (Card::PADDING * 2) - 1;
        float row_y = content_y + Card::PADDING;

        // Toggle 0: Bộ gõ tiếng Việt
        if (Toggle::hit_test(toggle_x, row_y + 10, static_cast<float>(x), static_cast<float>(y))) {
            hover_toggle_index_ = 0;
        }

        if (settings.input_method() == 0) {
            // Toggle 1: W shortcut (row 3)
            row_y += ROW_HEIGHT * 2;
            if (Toggle::hit_test(toggle_x, row_y + 10, static_cast<float>(x), static_cast<float>(y))) {
                hover_toggle_index_ = 1;
            }
            // Toggle 2: Bracket shortcut (row 4)
            row_y += ROW_HEIGHT;
            if (Toggle::hit_test(toggle_x, row_y + 10, static_cast<float>(x), static_cast<float>(y))) {
                hover_toggle_index_ = 2;
            }
        }

        content_y += card_height + CARD_SPACING;

        // Skip Card 2 (shortcuts)
        card_height = (ROW_HEIGHT * 2) + (Card::PADDING * 2) - 1;
        content_y += card_height + CARD_SPACING;

        // Card 3: System (toggles at index 3, 4, 5)
        card_height = (ROW_HEIGHT * 3) + (Card::PADDING * 2) - 1;
        row_y = content_y + Card::PADDING;

        for (int i = 3; i <= 5; i++) {
            if (Toggle::hit_test(toggle_x, row_y + 10, static_cast<float>(x), static_cast<float>(y))) {
                hover_toggle_index_ = i;
                break;
            }
            row_y += ROW_HEIGHT;
        }

        content_y += card_height + CARD_SPACING;

        // Card 4: Other options (toggles at index 6, 7, 8, 9)
        row_y = content_y + Card::PADDING;
        for (int i = 6; i <= 9; i++) {
            if (Toggle::hit_test(toggle_x, row_y + 10, static_cast<float>(x), static_cast<float>(y))) {
                hover_toggle_index_ = i;
                break;
            }
            row_y += ROW_HEIGHT;
        }
    }

    // Redraw if hover state changed
    if (old_sidebar_item != hover_sidebar_item_ ||
        old_toggle_index != hover_toggle_index_ ||
        old_shortcuts_hover != hover_shortcuts_row_) {
        InvalidateRect(hwnd_, nullptr, FALSE);
    }
}

void SettingsWindow::handle_mouse_down(int x, int y) {
    // Handle sidebar clicks
    if (x < SIDEBAR_WIDTH) {
        if (hover_sidebar_item_ == 0) {
            current_page_ = Page::Settings;
            InvalidateRect(hwnd_, nullptr, FALSE);
        } else if (hover_sidebar_item_ == 1) {
            current_page_ = Page::About;
            InvalidateRect(hwnd_, nullptr, FALSE);
        }
        return;
    }

    // Handle toggle clicks in settings page
    if (current_page_ == Page::Settings && hover_toggle_index_ >= 0) {
        auto& settings = gonhanh::Settings::instance();

        switch (hover_toggle_index_) {
            case 0: settings.set_enabled(!settings.is_enabled()); break;
            case 1: settings.set_w_shortcut(!settings.w_shortcut()); break;
            case 2: settings.set_bracket_shortcut(!settings.bracket_shortcut()); break;
            case 3: settings.set_auto_start(!settings.auto_start()); break;
            case 4: settings.set_per_app_mode(!settings.per_app_mode()); break;
            case 5: settings.set_english_auto_restore(!settings.english_auto_restore()); break;
            case 6: settings.set_sound_enabled(!settings.sound_enabled()); break;
            case 7: settings.set_modern_tone(!settings.modern_tone()); break;
            case 8: settings.set_auto_capitalize(!settings.auto_capitalize()); break;
            case 9: settings.set_esc_restore(!settings.esc_restore()); break;
        }

        InvalidateRect(hwnd_, nullptr, FALSE);
    }
}

void SettingsWindow::handle_mouse_up(int x, int y) {
    // Currently unused, but available for future interactions
}

LRESULT CALLBACK SettingsWindow::wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    SettingsWindow* self = nullptr;

    if (msg == WM_NCCREATE) {
        auto* cs = reinterpret_cast<CREATESTRUCTW*>(lparam);
        self = static_cast<SettingsWindow*>(cs->lpCreateParams);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
    } else {
        self = reinterpret_cast<SettingsWindow*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
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

        case WM_MOUSEMOVE:
            if (self) {
                int x = GET_X_LPARAM(lparam);
                int y = GET_Y_LPARAM(lparam);
                self->handle_mouse_move(x, y);
            }
            return 0;

        case WM_LBUTTONDOWN:
            if (self) {
                int x = GET_X_LPARAM(lparam);
                int y = GET_Y_LPARAM(lparam);
                self->handle_mouse_down(x, y);
            }
            return 0;

        case WM_LBUTTONUP:
            if (self) {
                int x = GET_X_LPARAM(lparam);
                int y = GET_Y_LPARAM(lparam);
                self->handle_mouse_up(x, y);
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
