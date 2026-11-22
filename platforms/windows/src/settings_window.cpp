#include "settings_window.h"
#include "resource.h"
#include "settings.h"
#include "shortcuts_dialog.h"
#include "system_tray.h"
#include "update_checker.h"
#include "ime_state.h"
#include "cli_path.h"
#include <shellapi.h>
#include <windowsx.h>
#include "fluent_ui.h"
#include <algorithm>
#include <cstdint>
#include <iterator>
#include <string>
#include <vector>


namespace gonhanh {

namespace {

using namespace fluent;

// ---------------------------------------------------------------------------
// Layout constants (logical pixels at 96 DPI)
// ---------------------------------------------------------------------------
constexpr int kClientW = 780;
constexpr int kMinClientH = 480;  // Otherwise the height fits the content (ContentHeightLogical)
constexpr int kSidebarW = 264;
constexpr int kPagePadX = 24;
constexpr int kPagePadTop = 4;      // The (invisible) title bar already spaces the top
constexpr int kPagePadBottom = 20;
constexpr int kGroupGap = 12;
constexpr int kRowH = 42;
constexpr int kRowSubH = 52;
constexpr int kCardRadius = 6;
constexpr int kRowPadX = 18;
constexpr int kScrollHitW = 14;

constexpr UINT_PTR kTimerUpdate = 1;
constexpr UINT_PTR kTimerAnim = 2;
constexpr float kAnimStep = 16.0f / 140.0f;  // ~140 ms toggle slide at 60 fps


// Segoe Fluent Icons (Windows 11) / Segoe MDL2 Assets (Windows 10) glyphs
constexpr wchar_t kIcoChevron = 0xE76C;
constexpr wchar_t kIcoHeartFill = 0xEB52;
constexpr wchar_t kIcoBug = 0xEBE8;

// GitHub mark (Octicons "mark-github", 16x16 viewBox, MIT License, github.com/primer/octicons)
const char* kGitHubMark =
    "M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 "
    "0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 "
    "0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 "
    "1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55"
    "-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67"
    ".01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z";

const wchar_t* kUrlSponsor = L"https://github.com/sponsors/khaphanspace";
const wchar_t* kUrlIssues = L"https://github.com/khaphanspace/gonhanh.org/issues";
const wchar_t* kUrlRepo = L"https://github.com/khaphanspace/gonhanh.org";
const wchar_t* kUrlReleases = L"https://github.com/khaphanspace/gonhanh.org/releases";
const wchar_t* kUrlAuthor = L"https://www.linkedin.com/in/khaphanspace";
const wchar_t* kUrlContributors = L"https://github.com/khaphanspace/gonhanh.org/blob/main/CONTRIBUTORS.md";

// ---------------------------------------------------------------------------
// Settings page model
// ---------------------------------------------------------------------------
// CliPath: a toggle backed by the user's PATH instead of a Settings field
enum class RowKind { Toggle, Method, Hotkey, Shortcuts, CliPath };

struct RowDef {
    RowKind kind;
    const wchar_t* title;
    const wchar_t* subtitle;     // Static subtitle, may be nullptr
    bool Settings::* field;      // Toggle rows only
    bool inverted;               // Toggle shows !field
};

struct GroupDef {
    const wchar_t* title;
    int first;
    int count;
};

const RowDef kRows[] = {
    // Bộ gõ
    { RowKind::Toggle,    L"Bộ gõ tiếng Việt",        nullptr, &Settings::enabled, false },
    { RowKind::Method,    L"Kiểu gõ",                 nullptr, nullptr, false },
    { RowKind::Toggle,    L"Gõ W thành Ư",            nullptr, &Settings::skipWShortcut, true },
    { RowKind::Toggle,    L"Gõ [ ] thành Ơ Ư",        nullptr, &Settings::bracketShortcut, false },
    { RowKind::Toggle,    L"Tự khôi phục tiếng Anh",  nullptr, &Settings::autoRestore, false },
    // Phím tắt
    { RowKind::Hotkey,    L"Bật/tắt tiếng Việt",      nullptr, nullptr, false },
    { RowKind::Shortcuts, L"Bảng gõ tắt",             nullptr, nullptr, false },
    // Quy tắc gõ
    { RowKind::Toggle,    L"Đặt dấu kiểu mới",        L"oà thay vì òa, uý thay vì úy", &Settings::modernTone, false },
    { RowKind::Toggle,    L"Cho phép phụ âm ngoại",   L"z, w, j, f làm phụ âm đầu", &Settings::allowForeignConsonants, false },
    { RowKind::Toggle,    L"Tự viết hoa đầu câu",     nullptr, &Settings::autoCapitalize, false },
    // Hệ thống
    { RowKind::Toggle,    L"Nhớ trạng thái theo app", L"Tự bật/tắt khi chuyển ứng dụng", &Settings::perApp, false },
    { RowKind::Toggle,    L"Âm thanh khi bật/tắt",    nullptr, &Settings::sound, false },
    { RowKind::Toggle,    L"Khởi động cùng Windows",  nullptr, &Settings::autoStart, false },
    { RowKind::CliPath,   L"Dòng lệnh gn",            nullptr, nullptr, false },
    { RowKind::Toggle,    L"Tự kiểm tra bản mới",     L"Kết nối GitHub khi mở Cài đặt", &Settings::checkUpdates, false },
};
constexpr int kRowCount = static_cast<int>(sizeof(kRows) / sizeof(kRows[0]));

const GroupDef kGroups[] = {
    { L"Bộ gõ", 0, 5 },
    { L"Phím tắt", 5, 2 },
    { L"Quy tắc gõ", 7, 3 },
    { L"Hệ thống", 10, 5 },
};

// `gn` on PATH: read from the registry when the window is shown or activated, not every frame
struct CliRow {
    CliPathState state = CliPathState::Missing;
    bool available = false;  // bin\ with the CLI exists next to GoNhanh.exe
    bool on = false;         // a Gõ Nhanh CLI folder is in the user's PATH
};
CliRow g_cliPath;

void ReadCliPathState() {
    g_cliPath.state = GetCliPathState();
    g_cliPath.available = g_cliPath.state != CliPathState::Missing;
    g_cliPath.on = g_cliPath.state == CliPathState::On || g_cliPath.state == CliPathState::OtherCopy;
}

bool RowIsToggle(const RowDef& row) {
    return row.kind == RowKind::Toggle || row.kind == RowKind::CliPath;
}

bool RowHasSubtitle(const RowDef& row) {
    return row.subtitle != nullptr || row.kind == RowKind::Shortcuts || row.kind == RowKind::CliPath;
}

bool RowClickable(const RowDef& row) {
    if (row.kind == RowKind::CliPath) return g_cliPath.available;
    return row.kind == RowKind::Toggle || row.kind == RowKind::Shortcuts;
}

// Height of the settings page in logical px (all groups, no scrolling needed)
int ContentHeightLogical() {
    int h = kPagePadTop + kPagePadBottom - kGroupGap;
    for (const auto& group : kGroups) {
        for (int i = group.first; i < group.first + group.count; i++) {
            h += RowHasSubtitle(kRows[i]) ? kRowSubH : kRowH;
        }
        h += kGroupGap;
    }
    return h;
}

bool ToggleValue(int row) {
    const RowDef& def = kRows[row];
    if (def.kind == RowKind::CliPath) return g_cliPath.on;
    bool value = Settings::Instance().*(def.field);
    return def.inverted ? !value : value;
}

}  // namespace

// ===========================================================================
// Window lifecycle
// ===========================================================================
SettingsWindow& SettingsWindow::Instance() {
    static SettingsWindow instance;
    return instance;
}

SettingsWindow::~SettingsWindow() {
    if (hwnd_) DestroyWindow(hwnd_);
}

int SettingsWindow::S(int logical) const {
    return MulDiv(logical, static_cast<int>(dpi_), 96);
}

void SettingsWindow::Show() {
    if (!hwnd_) Create();
    if (!hwnd_) return;

    scrollPos_ = 0;
    hover_ = {};
    ReadCliPathState();
    SyncToggleAnim(true);
    Layout();
    InvalidateRect(hwnd_, nullptr, FALSE);

    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
    visible_ = true;

    // Update check is opt-in: without it the app never touches the network
    if (Settings::Instance().checkUpdates) {
        StartUpdateCheck();
    }
}

void SettingsWindow::StartUpdateCheck() {
    // Runs in the background; poll so the sidebar badge refreshes when it completes
    UpdateChecker::Instance().CheckAsync();
    SetTimer(hwnd_, kTimerUpdate, 500, nullptr);
}

void SettingsWindow::Hide() {
    if (hwnd_) {
        KillTimer(hwnd_, kTimerUpdate);
        KillTimer(hwnd_, kTimerAnim);
        ShowWindow(hwnd_, SW_HIDE);
        visible_ = false;
        if (!ShortcutsDialog::Instance().IsVisible()) {
            DestroyIfHidden();
        }
    }
}

void SettingsWindow::DestroyIfHidden() {
    // Destroying (not just hiding) also frees the per-window DWM, theme and text-input state:
    // after closing, the app is back to its idle footprint. Show() recreates the window.
    // The Shortcuts window is owned by this one and goes with it.
    if (hwnd_ && !visible_) {
        DestroyWindow(hwnd_);
    }
    ReleaseRenderResources();
}

void SettingsWindow::Create() {
    if (!InitRenderer()) return;
    HINSTANCE hInst = GetModuleHandle(nullptr);

    WNDCLASSEXW wc = { sizeof(wc) };
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInst;
    wc.lpszClassName = L"GoNhanhSettingsWindow";
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.hbrBackground = nullptr;  // Fully custom painted
    wc.hIcon = LoadIconW(hInst, MAKEINTRESOURCEW(IDI_APP_LOGO));
    wc.hIconSm = LoadIconW(hInst, MAKEINTRESOURCEW(IDI_APP_LOGO));
    RegisterClassExW(&wc);

    // Size for the monitor under the cursor
    POINT cursor;
    GetCursorPos(&cursor);
    HMONITOR monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTOPRIMARY);
    MONITORINFO mi = { sizeof(mi) };
    GetMonitorInfoW(monitor, &mi);
    dpi_ = GetDpiForSystem();

    // Close button only: without WS_MINIMIZEBOX Windows also drops the (disabled) maximize button
    const DWORD style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU;
    const DWORD exStyle = WS_EX_APPWINDOW;
    // Tall enough to show every setting without scrolling, but never taller than the screen
    const RECT& work = mi.rcWork;
    auto clientHeight = [&] {
        RECT frame = { 0, 0, 0, 0 };
        AdjustWindowRectExForDpi(&frame, style, FALSE, exStyle, dpi_);
        int available = (work.bottom - work.top) - (frame.bottom - frame.top) - S(24);
        return (std::max)(S(kMinClientH), (std::min)(S(ContentHeightLogical()), available));
    };
    RECT rc = { 0, 0, S(kClientW), clientHeight() };
    AdjustWindowRectExForDpi(&rc, style, FALSE, exStyle, dpi_);
    int width = rc.right - rc.left;
    int height = rc.bottom - rc.top;
    int x = work.left + (work.right - work.left - width) / 2;
    int y = work.top + (work.bottom - work.top - height) / 2;

    hwnd_ = CreateWindowExW(exStyle, L"GoNhanhSettingsWindow", L"Gõ Nhanh - Cài đặt", style,
                            x, y, width, height, nullptr, nullptr, hInst, this);
    if (!hwnd_) return;

    // The window may land on a monitor with a different DPI than the system DPI
    UINT actualDpi = GetDpiForWindow(hwnd_);
    if (actualDpi != dpi_) {
        dpi_ = actualDpi;
        rc = { 0, 0, S(kClientW), clientHeight() };
        AdjustWindowRectExForDpi(&rc, style, FALSE, exStyle, dpi_);
        SetWindowPos(hwnd_, nullptr, 0, 0, rc.right - rc.left, rc.bottom - rc.top,
                     SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);
    }

    toggleAnim_.assign(kRowCount, 0.0f);
    RegisterThemeWindow(hwnd_);  // Follow accent / light-dark changes live
    ApplyWindowTheme();
    Layout();
}

void SettingsWindow::ApplyWindowTheme() {
    // Mica behind title bar + client (Windows 11 22H2+), clean title bar
    mica_ = ConfigureWindowFrame(hwnd_, true);
}

// ===========================================================================
// Layout
// ===========================================================================
void SettingsWindow::Layout() {
    if (!hwnd_) return;
    int left = S(kSidebarW) + S(kPagePadX);
    int right = ClientWidth(hwnd_) - S(kPagePadX);

    rowRects_.assign(kRowCount, RECT{});

    // No page title or group headings: groups are just cards separated by a gap (like macOS)
    int y = S(kPagePadTop);
    for (const auto& group : kGroups) {
        for (int i = group.first; i < group.first + group.count; i++) {
            int h = RowHasSubtitle(kRows[i]) ? S(kRowSubH) : S(kRowH);
            rowRects_[i] = { left, y, right, y + h };
            y += h;
        }
        y += S(kGroupGap);
    }
    contentHeight_ = y - S(kGroupGap) + S(kPagePadBottom);

    // Telex | VNI segmented control, right-aligned in the "Kiểu gõ" row
    for (int i = 0; i < kRowCount; i++) {
        if (kRows[i].kind != RowKind::Method) continue;
        const RECT& row = rowRects_[i];
        int segW = S(72);
        int segH = S(28);
        int segRight = row.right - S(kRowPadX) - S(3);
        int segTop = row.top + (row.bottom - row.top - segH) / 2;
        segmentRects_[0] = { segRight - segW * 2, segTop, segRight - segW, segTop + segH };
        segmentRects_[1] = { segRight - segW, segTop, segRight, segTop + segH };
    }

    ScrollTo(scrollPos_);  // Re-clamp
}

void SettingsWindow::ScrollTo(int pos) {
    if (!hwnd_) return;
    int maxScroll = (std::max)(0, contentHeight_ - ClientHeight(hwnd_));
    pos = std::clamp(pos, 0, maxScroll);
    if (pos != scrollPos_) {
        scrollPos_ = pos;
        InvalidateRect(hwnd_, nullptr, FALSE);
    }
}

bool SettingsWindow::ScrollbarGeometry(RECT& track, RECT& thumb) const {
    int viewH = ClientHeight(hwnd_);
    if (contentHeight_ <= viewH) return false;

    int clientW = ClientWidth(hwnd_);
    track = { clientW - S(kScrollHitW), S(4), clientW, viewH - S(4) };
    int trackH = track.bottom - track.top;
    int thumbH = (std::max)(S(40), MulDiv(trackH, viewH, contentHeight_));
    int maxScroll = contentHeight_ - viewH;
    int thumbTop = track.top + MulDiv(trackH - thumbH, scrollPos_, maxScroll);
    thumb = { track.left, thumbTop, track.right, thumbTop + thumbH };
    return true;
}

bool SettingsWindow::SyncToggleAnim(bool snap) {
    bool moving = false;
    for (int i = 0; i < kRowCount; i++) {
        if (!RowIsToggle(kRows[i])) continue;
        float target = ToggleValue(i) ? 1.0f : 0.0f;
        float& p = toggleAnim_[i];
        if (snap) {
            p = target;
        } else if (p < target) {
            p = (std::min)(target, p + kAnimStep);
        } else if (p > target) {
            p = (std::max)(target, p - kAnimStep);
        }
        if (p != target) moving = true;
    }

    // Telex/VNI selector pill
    float methodTarget = Settings::Instance().method == 1 ? 1.0f : 0.0f;
    if (snap) {
        methodAnim_ = methodTarget;
    } else if (methodAnim_ < methodTarget) {
        methodAnim_ = (std::min)(methodTarget, methodAnim_ + kAnimStep);
    } else if (methodAnim_ > methodTarget) {
        methodAnim_ = (std::max)(methodTarget, methodAnim_ - kAnimStep);
    }
    if (methodAnim_ != methodTarget) moving = true;
    return moving;
}

// ===========================================================================
// Painting
// ===========================================================================
void SettingsWindow::Paint(HDC hdc) {
    // Transparent clear lets Mica show through; solid base when Mica is unavailable
    RenderBuffered(hwnd_, hdc, mica_ ? 0 : Pal().base, [this] {
        PaintSidebar();
        PaintSettings();
    });
}

void SettingsWindow::PaintSidebar() {
    const Palette& pal = Pal();
    int sidebarW = S(kSidebarW);

    // App header (logo, name, version, tagline) + links from the top; credits at the bottom
    int centerX = sidebarW / 2;
    int logo = S(72);
    int y = S(kPagePadTop + 8);
    DrawLogo({ centerX - logo / 2, y, centerX - logo / 2 + logo, y + logo });
    y += logo + S(14);

    DrawStr(L"Gõ Nhanh", { 0, y, sidebarW, y + S(28) },
            Font(Face::Display, S(20), DWRITE_FONT_WEIGHT_SEMI_BOLD), pal.text, kCenter);
    y += S(28) + S(10);

    std::wstring version = L"v";
    version += GetAppVersion();
    IDWriteTextFormat* smallFont = Font(Face::Text, S(12));
    int lineH = S(18);

    UpdateStatus status = UpdateChecker::Instance().GetStatus();
    bool upToDate = status == UpdateStatus::UpToDate;
    bool available = status == UpdateStatus::Available;
    const wchar_t* badge = available ? L"Có bản mới" : upToDate ? L"Mới nhất" : nullptr;

    // Measure the whole line so it can be centered
    int versionW = MeasureStr(version.c_str(), smallFont);
    int checkW = upToDate ? S(12) : 0;
    int badgeW = badge ? MeasureStr(badge, smallFont) : 0;
    int gap = S(6);
    int totalW = versionW + (badge ? gap + (checkW ? checkW + S(4) : 0) + badgeW : 0);
    int x = centerX - totalW / 2;

    DrawStr(version.c_str(), { x, y, x + versionW, y + lineH }, smallFont, pal.textTertiary);
    x += versionW + gap;

    updateBadgeRect_ = {};
    if (upToDate) {
        // Green filled check circle
        DrawGlyph(0xEC61, { x, y, x + checkW, y + lineH }, Font(Face::Icon, S(12)), pal.success);
        x += checkW + S(4);
    }
    if (badge) {
        RECT badgeRect = { x, y, x + badgeW, y + lineH };
        DrawStr(badge, badgeRect, smallFont, available ? pal.accent : pal.textTertiary);
        if (available) {
            updateBadgeRect_ = badgeRect;
            if (hover_.kind == Hit::UpdateBadge) {
                FillRectF({ badgeRect.left, badgeRect.bottom - S(2), badgeRect.right, badgeRect.bottom - S(1) },
                          pal.accent);
            }
        }
    }

    y += lineH + S(6);

    // Tagline
    DrawStr(L"Bộ gõ tiếng Việt nhanh và nhẹ", { S(12), y, sidebarW - S(12), y + S(18) }, smallFont,
            pal.textSecondary, kCenter);
    y += S(18) + S(20);


    // Credits pinned to the bottom: "Phát triển bởi [Kha Phan] và [Cộng đồng]" / "© 2026 · GPL-3.0"
    struct Seg { const wchar_t* text; Hit link; };
    const Seg credits[] = {
        { L"Phát triển bởi ", Hit::None }, { L"Kha Phan", Hit::AuthorLink },
        { L" và ", Hit::None }, { L"Cộng đồng", Hit::CommunityLink },
    };
    int creditH = S(18);
    int bottom = ClientHeight(hwnd_) - S(20);
    int creditsW = 0;
    for (const auto& seg : credits) creditsW += MeasureStr(seg.text, smallFont);
    // One line when it fits (it does at the default sidebar width); otherwise wrap after the author
    bool oneLine = creditsW <= sidebarW - S(16);
    int creditY = bottom - creditH * (oneLine ? 2 : 3);
    // Links (formerly the About page): three tiles - Ủng hộ, Báo lỗi, GitHub - grouped with the
    // credits at the bottom; app identity stays at the top
    struct LinkTile { wchar_t glyph; const char* svg; const wchar_t* label; ARGB iconColor; };
    const LinkTile tiles[] = {
        { kIcoHeartFill, nullptr, L"Ủng hộ", pal.heart },
        { kIcoBug, nullptr, L"Báo lỗi", pal.text },
        { 0, kGitHubMark, L"GitHub", pal.text },
    };
    const int tileGap = S(8);
    const int tileW = (sidebarW - S(16) * 2 - tileGap * 2) / 3;
    const int tileH = S(66);
    const int tilesY = creditY - S(16) - tileH;
    int tileX = (sidebarW - (tileW * 3 + tileGap * 2)) / 2;
    for (int i = 0; i < 3; i++) {
        RECT tile = { tileX, tilesY, tileX + tileW, tilesY + tileH };
        linkRects_[i] = tile;
        bool hovered = hover_.kind == Hit::Link && hover_.index == i;
        bool pressed = hovered && pressed_ == hover_;
        FillRound(tile, static_cast<float>(S(kCardRadius)),
                  pressed ? pal.cardPressed : hovered ? pal.cardHover : pal.card, pal.cardStroke);
        RECT icon = { tile.left, tile.top + S(12), tile.right, tile.top + S(34) };
        if (tiles[i].svg) {
            RECT mark = { (tile.left + tile.right) / 2 - S(10), icon.top + S(1),
                          (tile.left + tile.right) / 2 + S(10), icon.top + S(21) };
            FillSvgPath(tiles[i].svg, 16.0f, mark, tiles[i].iconColor);
        } else {
            DrawGlyph(tiles[i].glyph, icon, Font(Face::Icon, S(20)), tiles[i].iconColor);
        }
        DrawStr(tiles[i].label, { tile.left, tile.top + S(38), tile.right, tile.bottom - S(8) },
                Font(Face::Text, S(12)), hovered ? pal.text : pal.textSecondary, kCenter);
        tileX += tileW + tileGap;
    }

    auto drawLine = [&](const Seg* segs, int count, int lineY) {
        int total = 0;
        for (int i = 0; i < count; i++) total += MeasureStr(segs[i].text, smallFont);
        int x = centerX - total / 2;
        for (int i = 0; i < count; i++) {
            int w = MeasureStr(segs[i].text, smallFont);
            RECT r = { x, lineY, x + w, lineY + creditH };
            bool isLink = segs[i].link != Hit::None;
            DrawStr(segs[i].text, r, smallFont, isLink ? pal.accent : pal.textTertiary);
            if (isLink) {
                (segs[i].link == Hit::AuthorLink ? authorLinkRect_ : communityLinkRect_) = r;
                if (hover_.kind == segs[i].link) {
                    FillRectF({ r.left, r.bottom - S(2), r.right, r.bottom - S(1) }, pal.accent);
                }
            }
            x += w;
        }
    };
    if (oneLine) {
        drawLine(credits, 4, creditY);
    } else {
        const Seg line2[] = { { L"và ", Hit::None }, credits[3] };
        drawLine(credits, 2, creditY);
        drawLine(line2, 2, creditY + creditH);
    }
    DrawStr(L"© 2026 · GPL-3.0", { 0, bottom - creditH, sidebarW, bottom }, Font(Face::Text, S(11)),
            pal.textTertiary, kCenter);
}

void SettingsWindow::PaintSettings() {
    const Palette& pal = Pal();
    float scale = dpi_ / 96.0f;
    int left = S(kSidebarW) + S(kPagePadX);
    int right = ClientWidth(hwnd_) - S(kPagePadX);
    int viewH = ClientHeight(hwnd_);
    int off = -scrollPos_;

    IDWriteTextFormat* titleFont = Font(Face::Text, S(14));
    IDWriteTextFormat* subFont = Font(Face::Text, S(12));
    IDWriteTextFormat* smallIconFont = Font(Face::Icon, S(12));

    for (size_t gi = 0; gi < std::size(kGroups); gi++) {
        const GroupDef& group = kGroups[gi];
        int last = group.first + group.count - 1;
        RECT card = { left, rowRects_[group.first].top + off, right, rowRects_[last].bottom + off };
        if (card.bottom < 0 || card.top > viewH) continue;

        // Each row is filled separately (hovered rows use the hover fill) inside the card's
        // rounded shape, then the card stroke is drawn on top.
        float radius = static_cast<float>(S(kCardRadius));
        PushClipRound(card, radius);

        for (int i = group.first; i <= last; i++) {
            const RowDef& def = kRows[i];
            RECT row = rowRects_[i];
            OffsetRect(&row, 0, off);

            bool rowHovered = hover_.kind == Hit::Row && hover_.index == i;
            bool rowPressed = rowHovered && pressed_ == hover_;
            ARGB fill = pal.card;
            if (rowHovered && RowClickable(def)) fill = rowPressed ? pal.cardPressed : pal.cardHover;
            FillRectF(row, fill);

            // Divider between rows, inset to the text column
            if (i != group.first) {
                FillRectF({ row.left + S(kRowPadX), row.top, row.right - S(kRowPadX), row.top + 1 }, pal.divider);
            }

            // Title + optional subtitle
            int textLeft = row.left + S(kRowPadX);
            int textRight = row.right - S(200);
            std::wstring subtitle;
            if (def.kind == RowKind::Shortcuts) {
                auto [enabled, total] = Settings::Instance().GetShortcutsCount();
                subtitle = std::to_wstring(enabled) + L"/" + std::to_wstring(total) + L" đang bật";
            } else if (def.kind == RowKind::CliPath) {
                switch (g_cliPath.state) {
                    case CliPathState::Missing: subtitle = L"Không thấy bin\\gn.exe cạnh app"; break;
                    case CliPathState::Off: subtitle = L"Dùng gn vi, gn en trong terminal"; break;
                    case CliPathState::On: subtitle = L"Đã thêm vào PATH, mở terminal mới"; break;
                    case CliPathState::OtherCopy: subtitle = L"PATH đang trỏ tới bản Gõ Nhanh khác"; break;
                }
            } else if (def.subtitle) {
                subtitle = def.subtitle;
            }
            int midY = (row.top + row.bottom) / 2;
            if (subtitle.empty()) {
                DrawStr(def.title, { textLeft, row.top, textRight, row.bottom }, titleFont, pal.text);
            } else {
                DrawStr(def.title, { textLeft, midY - S(21), textRight, midY + S(1) }, titleFont, pal.text,
                        kLeft, kBottom);
                DrawStr(subtitle.c_str(), { textLeft, midY + S(1), textRight, midY + S(20) }, subFont,
                        pal.textSecondary, kLeft, kTop);
            }

            // Right-hand control
            int controlRight = row.right - S(kRowPadX);
            switch (def.kind) {
                case RowKind::Toggle:
                case RowKind::CliPath:
                    if (RowClickable(def)) {
                        DrawToggle(controlRight - S(40), midY - S(10), scale, toggleAnim_[i], rowHovered, rowPressed);
                    }
                    break;
                case RowKind::Method: {
                    RECT segs[2] = { segmentRects_[0], segmentRects_[1] };
                    OffsetRect(&segs[0], 0, off);
                    OffsetRect(&segs[1], 0, off);
                    RECT container = { segs[0].left - S(3), segs[0].top - S(3), segs[1].right + S(3), segs[1].bottom + S(3) };
                    FillRound(container, static_cast<float>(S(6)), pal.toggleOffFill, pal.controlStroke);

                    // Selection pill: inset inside its segment and slides between them
                    float t = methodAnim_ * methodAnim_ * (3.0f - 2.0f * methodAnim_);  // Smoothstep
                    float insetX = static_cast<float>(S(4));
                    float insetY = static_cast<float>(S(2));
                    float pillL = segs[0].left + (segs[1].left - segs[0].left) * t + insetX;
                    float pillW = static_cast<float>(segs[0].right - segs[0].left) - insetX * 2.0f;
                    if (hover_.kind == Hit::Segment && hover_.index != Settings::Instance().method) {
                        RECT h = segs[hover_.index];
                        InflateRect(&h, -S(4), -S(2));
                        FillRound(h, static_cast<float>(S(4)), pal.controlFillHover);
                    }
                    RectF pill = { pillL, segs[0].top + insetY, pillL + pillW, segs[0].bottom - insetY };
                    FillRound(pill, static_cast<float>(S(4)), pal.controlSelected, pal.controlStroke);

                    const wchar_t* labels[] = { L"Telex", L"VNI" };
                    int method = Settings::Instance().method;
                    for (int s = 0; s < 2; s++) {
                        bool selected = method == s;
                        float closeness = s == 0 ? 1.0f - t : t;  // 1 when the pill sits on this segment
                        DrawStr(labels[s], segs[s],
                                Font(Face::Text, S(13), selected ? DWRITE_FONT_WEIGHT_SEMI_BOLD : DWRITE_FONT_WEIGHT_NORMAL),
                                Lerp(pal.textSecondary, pal.text, closeness), kCenter);
                    }
                    break;
                }
                case RowKind::Hotkey: {
                    const wchar_t* keys[] = { L"Space", L"Ctrl" };  // Laid out right-to-left
                    IDWriteTextFormat* keyFont = Font(Face::Text, S(12), DWRITE_FONT_WEIGHT_SEMI_BOLD);
                    int kx = controlRight;
                    for (const wchar_t* key : keys) {
                        int kw = MeasureStr(key, keyFont) + S(20);
                        RECT cap = { kx - kw, midY - S(13), kx, midY + S(13) };
                        FillRound(cap, static_cast<float>(S(4)), pal.controlFill, pal.controlStroke);
                        DrawStr(key, cap, keyFont, pal.textSecondary, kCenter);
                        kx -= kw + S(6);
                    }
                    break;
                }
                case RowKind::Shortcuts:
                    DrawGlyph(kIcoChevron, { controlRight - S(16), row.top, controlRight, row.bottom },
                              smallIconFont, pal.textSecondary);
                    break;
            }
        }

        PopClip();
        // Card outline on top of the row fills
        FillRound(card, radius, 0, pal.cardStroke);
    }

    // Overlay scrollbar (thin, widens on hover / drag)
    RECT track, thumb;
    if (ScrollbarGeometry(track, thumb)) {
        bool active = draggingThumb_ || hover_.kind == Hit::Scrollbar;
        int w = active ? S(6) : S(3);
        RECT bar = { track.right - S(4) - w, thumb.top, track.right - S(4), thumb.bottom };
        FillRound(bar, w / 2.0f, active ? pal.textSecondary : pal.textTertiary);
    }
}

// ===========================================================================
// Interaction
// ===========================================================================
SettingsWindow::HitResult SettingsWindow::HitTest(POINT pt) const {
    HitResult r;

    // Sidebar: update badge, links, credits
    if (pt.x < S(kSidebarW)) {
        if (PtInRect(&updateBadgeRect_, pt)) return { Hit::UpdateBadge, 0 };
        for (int i = 0; i < 3; i++) {
            if (PtInRect(&linkRects_[i], pt)) return { Hit::Link, i };
        }
        if (PtInRect(&authorLinkRect_, pt)) return { Hit::AuthorLink, 0 };
        if (PtInRect(&communityLinkRect_, pt)) return { Hit::CommunityLink, 0 };
        return r;
    }

    RECT track, thumb;
    if (ScrollbarGeometry(track, thumb) && pt.x >= track.left) return { Hit::Scrollbar, 0 };

    POINT content = { pt.x, pt.y + scrollPos_ };
    for (int s = 0; s < 2; s++) {
        RECT seg = segmentRects_[s];
        InflateRect(&seg, S(2), S(3));
        if (PtInRect(&seg, content)) return { Hit::Segment, s };
    }
    for (int i = 0; i < kRowCount; i++) {
        if (PtInRect(&rowRects_[i], content)) {
            if (RowClickable(kRows[i])) return { Hit::Row, i };
            return r;
        }
    }
    return r;
}

void SettingsWindow::Activate(const HitResult& hit) {
    switch (hit.kind) {
        case Hit::Segment: SetMethod(hit.index); break;
        case Hit::Row:
            if (kRows[hit.index].kind == RowKind::Toggle) {
                ToggleRow(hit.index);
            } else if (kRows[hit.index].kind == RowKind::CliPath) {
                // Pointing at another copy counts as on: the click removes it; the next one adds this copy
                SetCliOnPath(!g_cliPath.on);
                ReadCliPathState();  // Shows the real result, also if the registry write failed
                Refresh();
            } else if (kRows[hit.index].kind == RowKind::Shortcuts) {
                ShortcutsDialog::Instance().Show(hwnd_);
            }
            break;
        case Hit::Link: {
            const wchar_t* urls[] = { kUrlSponsor, kUrlIssues, kUrlRepo };
            ShellExecuteW(nullptr, L"open", urls[hit.index], nullptr, nullptr, SW_SHOW);
            break;
        }
        case Hit::AuthorLink: ShellExecuteW(nullptr, L"open", kUrlAuthor, nullptr, nullptr, SW_SHOW); break;
        case Hit::CommunityLink: ShellExecuteW(nullptr, L"open", kUrlContributors, nullptr, nullptr, SW_SHOW); break;
        case Hit::UpdateBadge: ShellExecuteW(nullptr, L"open", kUrlReleases, nullptr, nullptr, SW_SHOW); break;
        default: break;
    }
}

void SettingsWindow::ToggleRow(int row) {
    const RowDef& def = kRows[row];
    auto& settings = Settings::Instance();
    if (def.field == &Settings::enabled) {
        ToggleVietnamese();  // Keeps hook, tray icon and per-app memory in sync
        return;              // (calls Refresh())
    }
    settings.*(def.field) = !(settings.*(def.field));
    settings.Save();
    settings.ApplyToEngine();
    if (def.field == &Settings::checkUpdates && settings.checkUpdates) {
        StartUpdateCheck();
    }
    Refresh();
}

void SettingsWindow::Refresh() {
    if (!hwnd_ || !visible_) return;
    SetTimer(hwnd_, kTimerAnim, 16, nullptr);
    InvalidateRect(hwnd_, nullptr, FALSE);
}

void SettingsWindow::SetMethod(int method) {
    auto& settings = Settings::Instance();
    if (settings.method == method) return;
    settings.method = static_cast<uint8_t>(method);
    settings.Save();
    settings.ApplyToEngine();
    SystemTray::Instance().UpdateIcon();
    Refresh();
}

LRESULT CALLBACK SettingsWindow::WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    SettingsWindow* w = nullptr;
    if (msg == WM_NCCREATE) {
        auto* cs = reinterpret_cast<CREATESTRUCTW*>(lParam);
        w = static_cast<SettingsWindow*>(cs->lpCreateParams);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(w));
    } else {
        w = reinterpret_cast<SettingsWindow*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    }
    if (!w) return DefWindowProcW(hwnd, msg, wParam, lParam);

    switch (msg) {
        case WM_ERASEBKGND:
            return 1;

        case WM_PAINT: {
            PAINTSTRUCT ps;
            HDC hdc = BeginPaint(hwnd, &ps);
            w->Paint(hdc);
            EndPaint(hwnd, &ps);
            return 0;
        }

        case WM_MOUSEMOVE: {
            POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
            if (!w->trackingMouse_) {
                TRACKMOUSEEVENT tme = { sizeof(tme), TME_LEAVE, hwnd, 0 };
                TrackMouseEvent(&tme);
                w->trackingMouse_ = true;
            }
            if (w->draggingThumb_) {
                RECT track, thumb;
                if (w->ScrollbarGeometry(track, thumb)) {
                    int trackRange = (track.bottom - track.top) - (thumb.bottom - thumb.top);
                    int scrollRange = w->contentHeight_ - ClientHeight(hwnd);
                    if (trackRange > 0) {
                        w->ScrollTo(w->dragStartScroll_ + MulDiv(pt.y - w->dragStartY_, scrollRange, trackRange));
                    }
                }
                return 0;
            }
            HitResult hit = w->HitTest(pt);
            if (hit != w->hover_) {
                w->hover_ = hit;
                InvalidateRect(hwnd, nullptr, FALSE);
            }
            return 0;
        }

        case WM_MOUSELEAVE:
            w->trackingMouse_ = false;
            if (w->hover_.kind != Hit::None && !w->draggingThumb_) {
                w->hover_ = {};
                InvalidateRect(hwnd, nullptr, FALSE);
            }
            return 0;

        case WM_LBUTTONDOWN: {
            POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
            HitResult hit = w->HitTest(pt);
            if (hit.kind == Hit::Scrollbar) {
                RECT track, thumb;
                w->ScrollbarGeometry(track, thumb);
                if (pt.y < thumb.top || pt.y >= thumb.bottom) {
                    // Jump so the thumb centers on the click, then drag from there
                    int trackRange = (track.bottom - track.top) - (thumb.bottom - thumb.top);
                    int scrollRange = w->contentHeight_ - ClientHeight(hwnd);
                    int thumbTop = pt.y - (thumb.bottom - thumb.top) / 2 - track.top;
                    if (trackRange > 0) w->ScrollTo(MulDiv(thumbTop, scrollRange, trackRange));
                }
                w->draggingThumb_ = true;
                w->dragStartY_ = pt.y;
                w->dragStartScroll_ = w->scrollPos_;
            }
            w->pressed_ = hit;
            SetCapture(hwnd);
            InvalidateRect(hwnd, nullptr, FALSE);
            return 0;
        }

        case WM_LBUTTONUP: {
            POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
            HitResult pressed = w->pressed_;
            bool wasDragging = w->draggingThumb_;
            w->pressed_ = {};
            w->draggingThumb_ = false;
            ReleaseCapture();
            HitResult hit = w->HitTest(pt);
            w->hover_ = hit;
            if (!wasDragging && pressed.kind != Hit::None && hit == pressed) {
                w->Activate(hit);
            }
            InvalidateRect(hwnd, nullptr, FALSE);
            return 0;
        }

        case WM_SETCURSOR:
            if (LOWORD(lParam) == HTCLIENT) {
                Hit k = w->hover_.kind;
                bool hand = k != Hit::None && k != Hit::Scrollbar;
                SetCursor(LoadCursor(nullptr, hand ? IDC_HAND : IDC_ARROW));
                return TRUE;
            }
            break;

        case WM_MOUSEWHEEL: {
            int delta = GET_WHEEL_DELTA_WPARAM(wParam);
            w->ScrollTo(w->scrollPos_ - MulDiv(delta, w->S(72), WHEEL_DELTA));
            // Refresh hover under the (now moved) content
            POINT pt;
            GetCursorPos(&pt);
            ScreenToClient(hwnd, &pt);
            w->hover_ = w->HitTest(pt);
            return 0;
        }

        case WM_KEYDOWN: {
            int page = ClientHeight(hwnd) - w->S(80);
            switch (wParam) {
                case VK_UP: w->ScrollTo(w->scrollPos_ - w->S(48)); break;
                case VK_DOWN: w->ScrollTo(w->scrollPos_ + w->S(48)); break;
                case VK_PRIOR: w->ScrollTo(w->scrollPos_ - page); break;
                case VK_NEXT: w->ScrollTo(w->scrollPos_ + page); break;
                case VK_HOME: w->ScrollTo(0); break;
                case VK_END: w->ScrollTo(w->contentHeight_); break;
            }
            if (wParam == VK_ESCAPE) w->Hide();
            return 0;
        }

        case WM_ACTIVATE:
            if (LOWORD(wParam) != WA_INACTIVE) {
                // Settings may have changed elsewhere (Ctrl+Space, tray menu, shortcuts dialog, PATH)
                ReadCliPathState();
                w->SyncToggleAnim(true);
                InvalidateRect(hwnd, nullptr, FALSE);
            }
            break;

        case WM_TIMER:
            if (wParam == kTimerAnim) {
                if (!w->SyncToggleAnim(false)) KillTimer(hwnd, kTimerAnim);
                InvalidateRect(hwnd, nullptr, FALSE);
            } else if (wParam == kTimerUpdate) {
                UpdateStatus status = UpdateChecker::Instance().GetStatus();
                if (status != UpdateStatus::Checking && status != UpdateStatus::Idle) {
                    KillTimer(hwnd, kTimerUpdate);
                    InvalidateRect(hwnd, nullptr, FALSE);
                }
            }
            return 0;

        case WM_SETTINGCHANGE:
            if (lParam && wcscmp(reinterpret_cast<const wchar_t*>(lParam), L"ImmersiveColorSet") == 0) {
                InvalidatePalette();
                w->ApplyWindowTheme();
                InvalidateRect(hwnd, nullptr, FALSE);
            }
            break;

        case WM_DWMCOLORIZATIONCOLORCHANGED:
        case WM_FLUENT_THEME_CHANGED:  // Accent / light-dark change from UISettings
            InvalidatePalette();
            w->ApplyWindowTheme();
            InvalidateRect(hwnd, nullptr, FALSE);
            break;

        case WM_DPICHANGED: {
            w->dpi_ = HIWORD(wParam);
            ClearDpiCaches();
            const RECT* suggested = reinterpret_cast<const RECT*>(lParam);
            SetWindowPos(hwnd, nullptr, suggested->left, suggested->top,
                         suggested->right - suggested->left, suggested->bottom - suggested->top,
                         SWP_NOZORDER | SWP_NOACTIVATE);
            w->Layout();
            InvalidateRect(hwnd, nullptr, FALSE);
            return 0;
        }

        case WM_CLOSE:
            w->Hide();
            return 0;

        case WM_DESTROY:
            KillTimer(hwnd, kTimerUpdate);
            KillTimer(hwnd, kTimerAnim);
            UnregisterThemeWindow(hwnd);
            w->hwnd_ = nullptr;
            w->visible_ = false;
            w->hover_ = {};
            w->pressed_ = {};
            w->trackingMouse_ = false;
            w->draggingThumb_ = false;
            return 0;
    }

    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

} // namespace gonhanh
