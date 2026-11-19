#include "shortcuts_dialog.h"
#include "fluent_ui.h"
#include "resource.h"
#include "settings.h"
#include "settings_window.h"
#include <commctrl.h>
#include <commdlg.h>
#include <windowsx.h>
#include <algorithm>
#include <fstream>
#include <sstream>

#pragma comment(lib, "comctl32.lib")

namespace gonhanh {

using namespace fluent;

namespace {

// Layout (logical pixels at 96 DPI)
constexpr int kClientW = 600;
constexpr int kClientH = 560;
constexpr int kPad = 24;
constexpr int kFormTop = 84;
constexpr int kFormH = 64;
constexpr int kRowH = 48;
constexpr int kRadius = 6;

constexpr UINT_PTR kTimerStatus = 1;
constexpr UINT_PTR kEditSubclassId = 1;

constexpr wchar_t kIcoImport = 0xE8B5;
constexpr wchar_t kIcoExport = 0xEDE1;
constexpr wchar_t kIcoDelete = 0xE74D;

std::wstring Trim(const std::wstring& s) {
    size_t a = s.find_first_not_of(L" \t\r\n");
    if (a == std::wstring::npos) return L"";
    size_t b = s.find_last_not_of(L" \t\r\n");
    return s.substr(a, b - a + 1);
}

std::vector<Shortcut>& List() {
    return Settings::Instance().shortcuts;
}

// Solid colors for the native edit fields (they are GDI child windows, so no alpha)
ARGB FieldBackground() {
    const Palette& pal = Pal();
    return Over(pal.controlFill, Over(pal.card, pal.base));
}

}  // namespace

// ===========================================================================
// Lifecycle
// ===========================================================================
ShortcutsDialog& ShortcutsDialog::Instance() {
    static ShortcutsDialog instance;
    return instance;
}

ShortcutsDialog::~ShortcutsDialog() {
    if (hwnd_) DestroyWindow(hwnd_);
    if (editFont_) DeleteObject(editFont_);
    if (fieldBrush_) DeleteObject(fieldBrush_);
}

int ShortcutsDialog::S(int logical) const {
    return MulDiv(logical, static_cast<int>(dpi_), 96);
}

void ShortcutsDialog::Show(HWND owner) {
    if (!hwnd_) Create(owner);
    if (!hwnd_) return;

    // Center over the owner (Settings window) or the current monitor
    RECT wr;
    GetWindowRect(hwnd_, &wr);
    int w = wr.right - wr.left;
    int h = wr.bottom - wr.top;
    RECT anchor;
    if (owner && IsWindowVisible(owner)) {
        GetWindowRect(owner, &anchor);
    } else {
        POINT cursor;
        GetCursorPos(&cursor);
        MONITORINFO mi = { sizeof(mi) };
        GetMonitorInfoW(MonitorFromPoint(cursor, MONITOR_DEFAULTTOPRIMARY), &mi);
        anchor = mi.rcWork;
    }
    SetWindowPos(hwnd_, nullptr, (anchor.left + anchor.right - w) / 2, (anchor.top + anchor.bottom - h) / 2,
                 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);

    CancelEdit();
    status_.clear();
    scrollPos_ = 0;
    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
    SetFocus(triggerEdit_);
    visible_ = true;
    InvalidateRect(hwnd_, nullptr, FALSE);
}

void ShortcutsDialog::Hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
        visible_ = false;
        if (!SettingsWindow::Instance().IsVisible()) {
            // Settings is closed too: destroy it, which also destroys this owned window,
            // and release rendering resources -> back to the idle footprint
            SettingsWindow::Instance().DestroyIfHidden();
        }
    }
}

void ShortcutsDialog::Create(HWND owner) {
    if (!InitRenderer()) return;
    HINSTANCE hInst = GetModuleHandle(nullptr);

    WNDCLASSEXW wc = { sizeof(wc) };
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInst;
    wc.lpszClassName = L"GoNhanhShortcutsWindow";
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.hIcon = LoadIconW(hInst, MAKEINTRESOURCEW(IDI_APP_LOGO));
    wc.hIconSm = LoadIconW(hInst, MAKEINTRESOURCEW(IDI_APP_LOGO));
    RegisterClassExW(&wc);

    dpi_ = owner ? GetDpiForWindow(owner) : GetDpiForSystem();
    const DWORD style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN;
    RECT rc = { 0, 0, S(kClientW), S(kClientH) };
    AdjustWindowRectExForDpi(&rc, style, FALSE, 0, dpi_);

    hwnd_ = CreateWindowExW(0, L"GoNhanhShortcutsWindow", L"Bảng gõ tắt - Gõ Nhanh", style,
                            CW_USEDEFAULT, CW_USEDEFAULT, rc.right - rc.left, rc.bottom - rc.top,
                            owner, nullptr, hInst, this);
    if (!hwnd_) return;

    CreateEdits();
    ApplyTheme();
    Layout();
    RegisterThemeWindow(hwnd_);  // Follow accent / light-dark changes live
}

void ShortcutsDialog::CreateEdits() {
    HINSTANCE hInst = GetModuleHandle(nullptr);
    const DWORD style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_AUTOHSCROLL;
    triggerEdit_ = CreateWindowExW(0, L"EDIT", nullptr, style, 0, 0, 0, 0, hwnd_, nullptr, hInst, nullptr);
    replacementEdit_ = CreateWindowExW(0, L"EDIT", nullptr, style, 0, 0, 0, 0, hwnd_, nullptr, hInst, nullptr);
    SendMessageW(triggerEdit_, EM_SETCUEBANNER, TRUE, reinterpret_cast<LPARAM>(L"Từ viết tắt"));
    SendMessageW(replacementEdit_, EM_SETCUEBANNER, TRUE, reinterpret_cast<LPARAM>(L"Thay thế bằng"));
    SendMessageW(triggerEdit_, EM_SETLIMITTEXT, 64, 0);
    SendMessageW(replacementEdit_, EM_SETLIMITTEXT, 1024, 0);
    SetWindowSubclass(triggerEdit_, EditProc, kEditSubclassId, reinterpret_cast<DWORD_PTR>(this));
    SetWindowSubclass(replacementEdit_, EditProc, kEditSubclassId, reinterpret_cast<DWORD_PTR>(this));
}

void ShortcutsDialog::ApplyTheme() {
    ConfigureWindowFrame(hwnd_, false);  // Solid background: native edit fields can't draw on Mica

    if (fieldBrush_) DeleteObject(fieldBrush_);
    fieldBrush_ = CreateSolidBrush(ToColorRef(FieldBackground()));

    if (editFont_) DeleteObject(editFont_);
    editFont_ = CreateFontW(-S(14), 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, DEFAULT_CHARSET,
                            OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY,
                            DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI Variable Text");
    SendMessageW(triggerEdit_, WM_SETFONT, reinterpret_cast<WPARAM>(editFont_), TRUE);
    SendMessageW(replacementEdit_, WM_SETFONT, reinterpret_cast<WPARAM>(editFont_), TRUE);
}

// ===========================================================================
// Layout
// ===========================================================================
void ShortcutsDialog::Layout() {
    if (!hwnd_) return;
    int width = ClientWidth(hwnd_);
    int height = ClientHeight(hwnd_);
    int left = S(kPad);
    int right = width - S(kPad);

    // Header buttons (right-aligned): [Nhập] [Xuất]
    IDWriteTextFormat* btnFont = Font(Face::Text, S(13));
    int btnH = S(32);
    int btnY = S(22);
    int exportW = S(12) + S(16) + S(6) + MeasureStr(L"Xuất", btnFont) + S(14);
    int importW = S(12) + S(16) + S(6) + MeasureStr(L"Nhập", btnFont) + S(14);
    exportRect_ = { right - exportW, btnY, right, btnY + btnH };
    importRect_ = { exportRect_.left - S(8) - importW, btnY, exportRect_.left - S(8), btnY + btnH };

    // Add / edit form
    formCard_ = { left, S(kFormTop), right, S(kFormTop) + S(kFormH) };
    int fieldH = S(32);
    int fieldY = formCard_.top + (S(kFormH) - fieldH) / 2;
    int inner = S(16);
    submitRect_ = { formCard_.right - inner - S(84), fieldY, formCard_.right - inner, fieldY + fieldH };
    int replacementRight = submitRect_.left - S(8);
    cancelRect_ = {};
    if (editIndex_ >= 0) {
        cancelRect_ = { submitRect_.left - S(8) - S(64), fieldY, submitRect_.left - S(8), fieldY + fieldH };
        replacementRight = cancelRect_.left - S(8);
    }
    triggerField_ = { formCard_.left + inner, fieldY, formCard_.left + inner + S(150), fieldY + fieldH };
    replacementField_ = { triggerField_.right + S(8), fieldY, replacementRight, fieldY + fieldH };

    // Native edits sit inside the painted fields
    auto place = [&](HWND edit, const RECT& field) {
        int textH = S(20);
        MoveWindow(edit, field.left + S(10), field.top + (fieldH - textH) / 2,
                   field.right - field.left - S(20), textH, TRUE);
    };
    place(triggerEdit_, triggerField_);
    place(replacementEdit_, replacementField_);

    listCard_ = { left, formCard_.bottom + S(16), right, height - S(kPad) };
    ScrollTo(scrollPos_);
}

RECT ShortcutsDialog::ListViewport() const {
    return listCard_;
}

int ShortcutsDialog::ListContentHeight() const {
    return static_cast<int>(List().size()) * S(kRowH);
}

void ShortcutsDialog::ScrollTo(int pos) {
    RECT view = ListViewport();
    int maxScroll = (std::max)(0, ListContentHeight() - static_cast<int>(view.bottom - view.top));
    pos = std::clamp(pos, 0, maxScroll);
    if (pos != scrollPos_) {
        scrollPos_ = pos;
        InvalidateRect(hwnd_, nullptr, FALSE);
    }
}

bool ShortcutsDialog::ScrollbarGeometry(RECT& track, RECT& thumb) const {
    RECT view = ListViewport();
    int viewH = view.bottom - view.top;
    int contentH = ListContentHeight();
    if (contentH <= viewH) return false;
    track = { view.right - S(12), view.top + S(4), view.right, view.bottom - S(4) };
    int trackH = track.bottom - track.top;
    int thumbH = (std::max)(S(32), MulDiv(trackH, viewH, contentH));
    int thumbTop = track.top + MulDiv(trackH - thumbH, scrollPos_, contentH - viewH);
    thumb = { track.left, thumbTop, track.right, thumbTop + thumbH };
    return true;
}

// ===========================================================================
// Painting
// ===========================================================================
void ShortcutsDialog::Paint(HDC hdc) {
    RenderBuffered(hwnd_, hdc, Pal().base, [this] {
        const Palette& pal = Pal();
        float scale = dpi_ / 96.0f;
        int left = S(kPad);
        int right = ClientWidth(hwnd_) - S(kPad);

        // Header
        DrawStr(L"Bảng gõ tắt", { left, S(18), importRect_.left - S(8), S(48) },
                Font(Face::Display, S(20), DWRITE_FONT_WEIGHT_SEMI_BOLD), pal.text);
        std::wstring subtitle = status_;
        ARGB subtitleColor = statusError_ ? pal.danger : pal.textSecondary;
        if (subtitle.empty()) {
            auto [enabled, total] = Settings::Instance().GetShortcutsCount();
            subtitle = std::to_wstring(enabled) + L"/" + std::to_wstring(total) + L" đang bật";
        }
        DrawStr(subtitle.c_str(), { left, S(50), right, S(70) }, Font(Face::Text, S(12)), subtitleColor);

        // Header buttons
        auto drawButton = [&](const RECT& r, wchar_t icon, const wchar_t* label, Hit hit) {
            bool hovered = hover_.kind == hit;
            bool pressed = hovered && pressed_ == hover_;
            ARGB fill = pressed ? pal.controlFill : hovered ? pal.controlFillHover : pal.controlFill;
            FillRound(r, static_cast<float>(S(4)), fill, pal.controlStroke);
            RECT iconRect = { r.left + S(12), r.top, r.left + S(28), r.bottom };
            DrawGlyph(icon, iconRect, Font(Face::Icon, S(14)), pal.text);
            RECT textRect = { iconRect.right + S(6), r.top, r.right - S(8), r.bottom };
            DrawStr(label, textRect, Font(Face::Text, S(13)), pal.text);
        };
        drawButton(importRect_, kIcoImport, L"Nhập", Hit::Import);
        drawButton(exportRect_, kIcoExport, L"Xuất", Hit::Export);

        // Form card with the two fields + submit (and cancel while editing)
        FillRound(formCard_, static_cast<float>(S(kRadius)), pal.card, pal.cardStroke);
        HWND focus = GetFocus();
        auto drawField = [&](const RECT& field, HWND edit) {
            bool focused = focus == edit;
            FillRound(field, static_cast<float>(S(4)), FieldBackground(), pal.controlStroke);
            int lineH = focused ? S(2) : 1;
            RECT underline = { field.left + S(2), field.bottom - lineH, field.right - S(2), field.bottom };
            FillRectF(underline, focused ? pal.accent : pal.toggleStroke);
        };
        drawField(triggerField_, triggerEdit_);
        drawField(replacementField_, replacementEdit_);

        bool submitHover = hover_.kind == Hit::Submit;
        bool submitPressed = submitHover && pressed_ == hover_;
        ARGB submitFill = submitPressed ? Lerp(pal.accent, pal.base, 0.2f) : submitHover ? pal.accentHover : pal.accent;
        FillRound(submitRect_, static_cast<float>(S(4)), submitFill);
        DrawStr(editIndex_ >= 0 ? L"Lưu" : L"Thêm", submitRect_,
                Font(Face::Text, S(13), DWRITE_FONT_WEIGHT_SEMI_BOLD), pal.onAccent, kCenter);
        if (editIndex_ >= 0) {
            bool cancelHover = hover_.kind == Hit::Cancel;
            FillRound(cancelRect_, static_cast<float>(S(4)), cancelHover ? pal.controlFillHover : pal.controlFill,
                      pal.controlStroke);
            DrawStr(L"Hủy", cancelRect_, Font(Face::Text, S(13)), pal.text, kCenter);
        }

        // List card
        float radius = static_cast<float>(S(kRadius));
        FillRound(listCard_, radius, pal.card, pal.cardStroke);
        const auto& list = List();
        if (list.empty()) {
            int midY = (listCard_.top + listCard_.bottom) / 2;
            DrawStr(L"Chưa có từ viết tắt nào", { listCard_.left, midY - S(22), listCard_.right, midY },
                    Font(Face::Text, S(14), DWRITE_FONT_WEIGHT_SEMI_BOLD), pal.textSecondary, kCenter);
            DrawStr(L"Nhập từ viết tắt và nội dung thay thế ở ô phía trên",
                    { listCard_.left, midY + S(2), listCard_.right, midY + S(22) },
                    Font(Face::Text, S(12)), pal.textTertiary, kCenter);
            return;
        }

        PushClipRound(listCard_, radius);

        IDWriteTextFormat* triggerFont = Font(Face::Text, S(14), DWRITE_FONT_WEIGHT_SEMI_BOLD);
        IDWriteTextFormat* textFont = Font(Face::Text, S(14));
        int rowH = S(kRowH);
        int firstVisible = scrollPos_ / rowH;
        for (int i = firstVisible; i < static_cast<int>(list.size()); i++) {
            RECT row = { listCard_.left, listCard_.top + i * rowH - scrollPos_, listCard_.right,
                         listCard_.top + (i + 1) * rowH - scrollPos_ };
            if (row.top >= listCard_.bottom) break;
            const Shortcut& sc = list[i];
            bool rowHovered = hover_.index == i &&
                (hover_.kind == Hit::Row || hover_.kind == Hit::Toggle || hover_.kind == Hit::Delete);
            bool editing = i == editIndex_;
            if (editing || rowHovered) {
                FillRectF(row, editing ? pal.cardHover : Lerp(pal.card, pal.cardHover, 0.6f));
            }
            if (editing) {
                RECT pill = { row.left, row.top + S(14), row.left + S(3), row.bottom - S(14) };
                FillRound(pill, S(3) / 2.0f, pal.accent);
            }
            if (i > 0) {
                FillRectF({ row.left + S(16), row.top, row.right - S(16), row.top + 1 }, pal.divider);
            }
            int midY = (row.top + row.bottom) / 2;

            bool toggleHover = hover_.kind == Hit::Toggle && hover_.index == i;
            DrawToggle(row.left + S(16), midY - S(10), scale, sc.enabled ? 1.0f : 0.0f, toggleHover,
                       toggleHover && pressed_ == hover_);

            ARGB textColor = sc.enabled ? pal.text : pal.textTertiary;
            int tx = row.left + S(72);
            DrawStr(sc.trigger.c_str(), { tx, row.top, tx + S(140), row.bottom }, triggerFont, textColor);
            DrawStr(L"→", { tx + S(144), row.top, tx + S(168), row.bottom }, textFont, pal.textTertiary, kCenter);
            DrawStr(sc.replacement.c_str(), { tx + S(176), row.top, row.right - S(60), row.bottom }, textFont,
                    sc.enabled ? pal.textSecondary : pal.textTertiary);

            RECT del = { row.right - S(16) - S(32), midY - S(16), row.right - S(16), midY + S(16) };
            bool delHover = hover_.kind == Hit::Delete && hover_.index == i;
            if (delHover) FillRound(del, static_cast<float>(S(4)), pal.controlFillHover);
            if (rowHovered || editing) {
                DrawGlyph(kIcoDelete, del, Font(Face::Icon, S(14)), delHover ? pal.danger : pal.textSecondary);
            }
        }

        PopClip();

        // Overlay scrollbar
        RECT track, thumb;
        if (ScrollbarGeometry(track, thumb)) {
            bool active = draggingThumb_ || hover_.kind == Hit::Scrollbar;
            int w = active ? S(6) : S(3);
            RECT bar = { track.right - S(4) - w, thumb.top, track.right - S(4), thumb.bottom };
            FillRound(bar, w / 2.0f, active ? pal.textSecondary : pal.textTertiary);
        }
    });
}

// ===========================================================================
// Interaction
// ===========================================================================
ShortcutsDialog::HitResult ShortcutsDialog::HitTest(POINT pt) const {
    if (PtInRect(&importRect_, pt)) return { Hit::Import, 0 };
    if (PtInRect(&exportRect_, pt)) return { Hit::Export, 0 };
    if (PtInRect(&submitRect_, pt)) return { Hit::Submit, 0 };
    if (editIndex_ >= 0 && PtInRect(&cancelRect_, pt)) return { Hit::Cancel, 0 };

    if (!PtInRect(&listCard_, pt)) return {};
    RECT track, thumb;
    if (ScrollbarGeometry(track, thumb) && pt.x >= track.left) return { Hit::Scrollbar, 0 };

    int index = (pt.y - listCard_.top + scrollPos_) / S(kRowH);
    if (index < 0 || index >= static_cast<int>(List().size())) return {};
    int rowTop = listCard_.top + index * S(kRowH) - scrollPos_;
    int midY = rowTop + S(kRowH) / 2;
    if (pt.x < listCard_.left + S(64)) return { Hit::Toggle, index };
    RECT del = { listCard_.right - S(16) - S(32), midY - S(16), listCard_.right - S(16), midY + S(16) };
    if (PtInRect(&del, pt)) return { Hit::Delete, index };
    return { Hit::Row, index };
}

void ShortcutsDialog::Activate(const HitResult& hit) {
    switch (hit.kind) {
        case Hit::Import: ImportShortcuts(); break;
        case Hit::Export: ExportShortcuts(); break;
        case Hit::Submit: Submit(); break;
        case Hit::Cancel: CancelEdit(); break;
        case Hit::Toggle: ToggleShortcut(hit.index); break;
        case Hit::Delete: DeleteShortcut(hit.index); break;
        case Hit::Row: BeginEdit(hit.index); break;
        default: break;
    }
}

std::wstring ShortcutsDialog::EditText(HWND edit) const {
    int len = GetWindowTextLengthW(edit);
    std::wstring text(static_cast<size_t>(len) + 1, L'\0');
    GetWindowTextW(edit, text.data(), len + 1);
    text.resize(len);
    return text;
}

void ShortcutsDialog::SetStatus(const std::wstring& text, bool error) {
    status_ = text;
    statusError_ = error;
    SetTimer(hwnd_, kTimerStatus, 3000, nullptr);
    InvalidateRect(hwnd_, nullptr, FALSE);
}

void ShortcutsDialog::Submit() {
    std::wstring trigger = Trim(EditText(triggerEdit_));
    std::wstring replacement = Trim(EditText(replacementEdit_));
    if (trigger.empty()) {
        SetFocus(triggerEdit_);
        SetStatus(L"Hãy nhập từ viết tắt", true);
        return;
    }
    if (trigger.find_first_of(L" \t") != std::wstring::npos) {
        SetFocus(triggerEdit_);
        SetStatus(L"Từ viết tắt không được chứa dấu cách", true);
        return;
    }
    if (replacement.empty()) {
        SetFocus(replacementEdit_);
        SetStatus(L"Hãy nhập nội dung thay thế", true);
        return;
    }

    auto& list = List();
    int existing = -1;
    for (int i = 0; i < static_cast<int>(list.size()); i++) {
        if (list[i].trigger == trigger && i != editIndex_) { existing = i; break; }
    }

    if (existing >= 0) {
        // Same trigger already exists: update it (and drop the row being edited, if any)
        list[existing].replacement = replacement;
        list[existing].enabled = true;
        if (editIndex_ >= 0) list.erase(list.begin() + editIndex_);
        SetStatus(L"Đã cập nhật \"" + trigger + L"\"", false);
    } else if (editIndex_ >= 0) {
        list[editIndex_].trigger = trigger;
        list[editIndex_].replacement = replacement;
        SetStatus(L"Đã lưu \"" + trigger + L"\"", false);
    } else {
        list.push_back({ trigger, replacement, true });
        SetStatus(L"Đã thêm \"" + trigger + L"\"", false);
        ScrollTo(ListContentHeight());
    }

    Commit();
    CancelEdit();
}

void ShortcutsDialog::BeginEdit(int index) {
    const auto& list = List();
    if (index < 0 || index >= static_cast<int>(list.size())) return;
    editIndex_ = index;
    SetWindowTextW(triggerEdit_, list[index].trigger.c_str());
    SetWindowTextW(replacementEdit_, list[index].replacement.c_str());
    Layout();
    SetFocus(replacementEdit_);
    SendMessageW(replacementEdit_, EM_SETSEL, 0, -1);
    InvalidateRect(hwnd_, nullptr, FALSE);
}

void ShortcutsDialog::CancelEdit() {
    editIndex_ = -1;
    SetWindowTextW(triggerEdit_, L"");
    SetWindowTextW(replacementEdit_, L"");
    Layout();
    if (visible_) SetFocus(triggerEdit_);
    InvalidateRect(hwnd_, nullptr, FALSE);
}

void ShortcutsDialog::DeleteShortcut(int index) {
    auto& list = List();
    if (index < 0 || index >= static_cast<int>(list.size())) return;
    std::wstring trigger = list[index].trigger;
    list.erase(list.begin() + index);
    if (editIndex_ == index) {
        CancelEdit();
    } else if (editIndex_ > index) {
        editIndex_--;
    }
    Commit();
    ScrollTo(scrollPos_);
    SetStatus(L"Đã xóa \"" + trigger + L"\"", false);
}

void ShortcutsDialog::ToggleShortcut(int index) {
    auto& list = List();
    if (index < 0 || index >= static_cast<int>(list.size())) return;
    list[index].enabled = !list[index].enabled;
    Commit();
    InvalidateRect(hwnd_, nullptr, FALSE);
}

void ShortcutsDialog::Commit() {
    auto& settings = Settings::Instance();
    settings.Save();
    settings.ApplyToEngine();
    SettingsWindow::Instance().Refresh();  // "x/y đang bật" on the Settings page
}

void ShortcutsDialog::ImportShortcuts() {
    OPENFILENAMEW ofn = {};
    wchar_t filename[MAX_PATH] = {};
    ofn.lStructSize = sizeof(OPENFILENAMEW);
    ofn.hwndOwner = hwnd_;
    ofn.lpstrFilter = L"Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0";
    ofn.lpstrFile = filename;
    ofn.nMaxFile = MAX_PATH;
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;
    if (!GetOpenFileNameW(&ofn)) return;

    // Read file as UTF-8 bytes, then convert to UTF-16
    std::ifstream file(filename, std::ios::binary);
    if (!file.is_open()) {
        SetStatus(L"Không thể mở file", true);
        return;
    }
    std::string utf8((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());
    file.close();
    if (utf8.size() >= 3 && static_cast<unsigned char>(utf8[0]) == 0xEF &&
        static_cast<unsigned char>(utf8[1]) == 0xBB && static_cast<unsigned char>(utf8[2]) == 0xBF) {
        utf8.erase(0, 3);  // UTF-8 BOM
    }
    std::wstring content;
    if (!utf8.empty()) {
        int len = MultiByteToWideChar(CP_UTF8, 0, utf8.c_str(), static_cast<int>(utf8.size()), nullptr, 0);
        if (len > 0) {
            content.resize(len);
            MultiByteToWideChar(CP_UTF8, 0, utf8.c_str(), static_cast<int>(utf8.size()), content.data(), len);
        }
    }

    // One shortcut per line: "trigger<TAB>replacement" or "trigger=replacement".
    // Merged into the current list: existing triggers are updated, new ones appended.
    auto& list = List();
    int imported = 0;
    std::wistringstream stream(content);
    std::wstring line;
    while (std::getline(stream, line)) {
        size_t pos = line.find(L'\t');
        if (pos == std::wstring::npos) pos = line.find(L'=');
        if (pos == std::wstring::npos) continue;
        std::wstring trigger = Trim(line.substr(0, pos));
        std::wstring replacement = Trim(line.substr(pos + 1));
        if (trigger.empty() || replacement.empty() || trigger[0] == L';') continue;

        auto it = std::find_if(list.begin(), list.end(), [&](const Shortcut& s) { return s.trigger == trigger; });
        if (it != list.end()) {
            it->replacement = replacement;
            it->enabled = true;
        } else {
            list.push_back({ trigger, replacement, true });
        }
        imported++;
    }

    CancelEdit();
    Commit();
    SetStatus(imported ? L"Đã nhập " + std::to_wstring(imported) + L" từ viết tắt"
                       : L"Không tìm thấy từ viết tắt nào trong file",
              imported == 0);
}

void ShortcutsDialog::ExportShortcuts() {
    OPENFILENAMEW ofn = {};
    wchar_t filename[MAX_PATH] = L"gonhanh-shortcuts.txt";
    ofn.lStructSize = sizeof(OPENFILENAMEW);
    ofn.hwndOwner = hwnd_;
    ofn.lpstrFilter = L"Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0";
    ofn.lpstrFile = filename;
    ofn.nMaxFile = MAX_PATH;
    ofn.lpstrDefExt = L"txt";
    ofn.Flags = OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST;
    if (!GetSaveFileNameW(&ofn)) return;

    std::wstring content;
    for (const auto& s : List()) {
        content += s.trigger + L"\t" + s.replacement + L"\n";
    }
    std::string utf8;
    if (!content.empty()) {
        int len = WideCharToMultiByte(CP_UTF8, 0, content.c_str(), static_cast<int>(content.size()),
                                      nullptr, 0, nullptr, nullptr);
        if (len > 0) {
            utf8.resize(len);
            WideCharToMultiByte(CP_UTF8, 0, content.c_str(), static_cast<int>(content.size()),
                                utf8.data(), len, nullptr, nullptr);
        }
    }

    std::ofstream file(filename, std::ios::binary);
    if (!file.is_open()) {
        SetStatus(L"Không thể tạo file", true);
        return;
    }
    file.write(utf8.data(), static_cast<std::streamsize>(utf8.size()));
    file.close();
    SetStatus(L"Đã xuất " + std::to_wstring(List().size()) + L" từ viết tắt", false);
}

// ===========================================================================
// Window procedures
// ===========================================================================
LRESULT CALLBACK ShortcutsDialog::EditProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam,
                                           UINT_PTR, DWORD_PTR data) {
    auto* d = reinterpret_cast<ShortcutsDialog*>(data);
    switch (msg) {
        case WM_KEYDOWN:
            if (wParam == VK_RETURN) {
                d->Submit();
                return 0;
            }
            if (wParam == VK_ESCAPE) {
                if (d->editIndex_ >= 0) d->CancelEdit();
                else d->Hide();
                return 0;
            }
            if (wParam == VK_TAB) {
                SetFocus(hwnd == d->triggerEdit_ ? d->replacementEdit_ : d->triggerEdit_);
                return 0;
            }
            break;
        case WM_CHAR:
            if (wParam == L'\r' || wParam == L'\t' || wParam == 0x1B) return 0;  // No beep
            break;
        case WM_SETFOCUS:
        case WM_KILLFOCUS:
            InvalidateRect(d->hwnd_, nullptr, FALSE);  // Focus underline
            break;
    }
    return DefSubclassProc(hwnd, msg, wParam, lParam);
}

LRESULT CALLBACK ShortcutsDialog::WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    ShortcutsDialog* d = nullptr;
    if (msg == WM_NCCREATE) {
        auto* cs = reinterpret_cast<CREATESTRUCTW*>(lParam);
        d = static_cast<ShortcutsDialog*>(cs->lpCreateParams);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(d));
        d->hwnd_ = hwnd;
    } else {
        d = reinterpret_cast<ShortcutsDialog*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    }
    if (!d) return DefWindowProcW(hwnd, msg, wParam, lParam);

    switch (msg) {
        case WM_ERASEBKGND:
            return 1;

        case WM_PAINT: {
            PAINTSTRUCT ps;
            HDC hdc = BeginPaint(hwnd, &ps);
            d->Paint(hdc);
            EndPaint(hwnd, &ps);
            return 0;
        }

        case WM_CTLCOLOREDIT: {
            HDC hdc = reinterpret_cast<HDC>(wParam);
            ARGB bg = FieldBackground();
            SetTextColor(hdc, ToColorRef(Over(Pal().text, bg)));
            SetBkColor(hdc, ToColorRef(bg));
            return reinterpret_cast<LRESULT>(d->fieldBrush_);
        }

        case WM_MOUSEMOVE: {
            POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
            if (!d->trackingMouse_) {
                TRACKMOUSEEVENT tme = { sizeof(tme), TME_LEAVE, hwnd, 0 };
                TrackMouseEvent(&tme);
                d->trackingMouse_ = true;
            }
            if (d->draggingThumb_) {
                RECT track, thumb;
                if (d->ScrollbarGeometry(track, thumb)) {
                    RECT view = d->ListViewport();
                    int trackRange = (track.bottom - track.top) - (thumb.bottom - thumb.top);
                    int scrollRange = d->ListContentHeight() - (view.bottom - view.top);
                    if (trackRange > 0) {
                        d->ScrollTo(d->dragStartScroll_ + MulDiv(pt.y - d->dragStartY_, scrollRange, trackRange));
                    }
                }
                return 0;
            }
            HitResult hit = d->HitTest(pt);
            if (hit != d->hover_) {
                d->hover_ = hit;
                InvalidateRect(hwnd, nullptr, FALSE);
            }
            return 0;
        }

        case WM_MOUSELEAVE:
            d->trackingMouse_ = false;
            if (d->hover_.kind != Hit::None && !d->draggingThumb_) {
                d->hover_ = {};
                InvalidateRect(hwnd, nullptr, FALSE);
            }
            return 0;

        case WM_LBUTTONDOWN: {
            POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
            // Clicking the padding of a text field focuses its edit control
            if (PtInRect(&d->triggerField_, pt)) { SetFocus(d->triggerEdit_); return 0; }
            if (PtInRect(&d->replacementField_, pt)) { SetFocus(d->replacementEdit_); return 0; }

            HitResult hit = d->HitTest(pt);
            if (hit.kind == Hit::Scrollbar) {
                d->draggingThumb_ = true;
                d->dragStartY_ = pt.y;
                d->dragStartScroll_ = d->scrollPos_;
            }
            d->pressed_ = hit;
            SetCapture(hwnd);
            InvalidateRect(hwnd, nullptr, FALSE);
            return 0;
        }

        case WM_LBUTTONUP: {
            POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
            HitResult pressed = d->pressed_;
            bool wasDragging = d->draggingThumb_;
            d->pressed_ = {};
            d->draggingThumb_ = false;
            ReleaseCapture();
            HitResult hit = d->HitTest(pt);
            d->hover_ = hit;
            if (!wasDragging && pressed.kind != Hit::None && hit == pressed) {
                d->Activate(hit);
            }
            InvalidateRect(hwnd, nullptr, FALSE);
            return 0;
        }

        case WM_SETCURSOR:
            if (reinterpret_cast<HWND>(wParam) == hwnd && LOWORD(lParam) == HTCLIENT) {
                POINT pt;
                GetCursorPos(&pt);
                ScreenToClient(hwnd, &pt);
                if (PtInRect(&d->triggerField_, pt) || PtInRect(&d->replacementField_, pt)) {
                    SetCursor(LoadCursor(nullptr, IDC_IBEAM));
                    return TRUE;
                }
                Hit k = d->hover_.kind;
                bool hand = k != Hit::None && k != Hit::Scrollbar;
                SetCursor(LoadCursor(nullptr, hand ? IDC_HAND : IDC_ARROW));
                return TRUE;
            }
            break;

        case WM_MOUSEWHEEL: {
            int delta = GET_WHEEL_DELTA_WPARAM(wParam);
            d->ScrollTo(d->scrollPos_ - MulDiv(delta, d->S(kRowH) * 2, WHEEL_DELTA));
            POINT pt;
            GetCursorPos(&pt);
            ScreenToClient(hwnd, &pt);
            d->hover_ = d->HitTest(pt);
            return 0;
        }

        case WM_KEYDOWN:
            if (wParam == VK_ESCAPE) d->Hide();
            return 0;

        case WM_TIMER:
            if (wParam == kTimerStatus) {
                KillTimer(hwnd, kTimerStatus);
                d->status_.clear();
                InvalidateRect(hwnd, nullptr, FALSE);
            }
            return 0;

        case WM_SETTINGCHANGE:
            if (!lParam || wcscmp(reinterpret_cast<const wchar_t*>(lParam), L"ImmersiveColorSet") != 0) break;
            [[fallthrough]];
        case WM_DWMCOLORIZATIONCOLORCHANGED:
        case WM_FLUENT_THEME_CHANGED:  // Accent / light-dark change from UISettings
            InvalidatePalette();
            d->ApplyTheme();
            InvalidateRect(hwnd, nullptr, FALSE);
            InvalidateRect(d->triggerEdit_, nullptr, TRUE);
            InvalidateRect(d->replacementEdit_, nullptr, TRUE);
            if (msg == WM_SETTINGCHANGE) break;
            return 0;

        case WM_DPICHANGED: {
            d->dpi_ = HIWORD(wParam);
            ClearDpiCaches();
            const RECT* r = reinterpret_cast<const RECT*>(lParam);
            SetWindowPos(hwnd, nullptr, r->left, r->top, r->right - r->left, r->bottom - r->top,
                         SWP_NOZORDER | SWP_NOACTIVATE);
            d->ApplyTheme();  // Recreates the DPI-scaled edit font
            d->Layout();
            InvalidateRect(hwnd, nullptr, FALSE);
            return 0;
        }

        case WM_CLOSE:
            d->Hide();
            return 0;

        case WM_DESTROY:
            // Destroyed with its owner (Settings); Show() recreates it
            KillTimer(hwnd, kTimerStatus);
            UnregisterThemeWindow(hwnd);
            d->hwnd_ = nullptr;
            d->triggerEdit_ = nullptr;
            d->replacementEdit_ = nullptr;
            d->visible_ = false;
            d->editIndex_ = -1;
            d->hover_ = {};
            d->pressed_ = {};
            d->trackingMouse_ = false;
            d->draggingThumb_ = false;
            return 0;
    }
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

} // namespace gonhanh
