#include "settings_window.h"
#include "resource.h"
#include "settings.h"
#include "shortcuts_dialog.h"
#include <windowsx.h>

namespace gonhanh {

SettingsWindow& SettingsWindow::Instance() {
    static SettingsWindow instance;
    return instance;
}

SettingsWindow::~SettingsWindow() {
    if (hwnd_) {
        DestroyWindow(hwnd_);
    }
}

void SettingsWindow::Show() {
    if (!hwnd_) {
        Create();
    }
    LoadSettings();
    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
    visible_ = true;
}

void SettingsWindow::Hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
        visible_ = false;
    }
}

void SettingsWindow::Create() {
    hwnd_ = CreateDialogParamW(
        GetModuleHandle(NULL),
        MAKEINTRESOURCEW(IDD_SETTINGS),
        NULL,
        DialogProc,
        (LPARAM)this
    );

    if (!hwnd_) {
        return;
    }

    CreateControls();
}

void SettingsWindow::CreateControls() {
    // Card 1: Input Method (10, 10, 680, 180)
    CreateWindowExW(0, L"BUTTON", L"Phương thức nhập liệu",
        WS_CHILD | WS_VISIBLE | BS_GROUPBOX,
        10, 10, 680, 180, hwnd_, NULL, GetModuleHandle(NULL), NULL);

    chkEnabled_ = CreateWindowExW(0, L"BUTTON", L"Bật tiếng Việt",
        WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
        25, 35, 300, 24, hwnd_, (HMENU)IDC_CHK_ENABLED, GetModuleHandle(NULL), NULL);

    CreateWindowExW(0, L"STATIC", L"Kiểu gõ:",
        WS_CHILD | WS_VISIBLE | SS_LEFT,
        25, 65, 100, 20, hwnd_, NULL, GetModuleHandle(NULL), NULL);

    cmbMethod_ = CreateWindowExW(0, L"COMBOBOX", NULL,
        WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST | WS_VSCROLL,
        130, 63, 200, 200, hwnd_, (HMENU)IDC_CMB_METHOD, GetModuleHandle(NULL), NULL);
    ComboBox_AddString(cmbMethod_, L"Telex");
    ComboBox_AddString(cmbMethod_, L"VNI");

    chkWShortcut_ = CreateWindowExW(0, L"BUTTON", L"Bỏ qua phím tắt W (ví dụ: Ctrl+W)",
        WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
        25, 95, 640, 24, hwnd_, (HMENU)IDC_CHK_W_SHORTCUT, GetModuleHandle(NULL), NULL);

    chkBracket_ = CreateWindowExW(0, L"BUTTON", L"Dấu ngoặc kép tự động \"\"",
        WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
        25, 125, 300, 24, hwnd_, (HMENU)IDC_CHK_BRACKET, GetModuleHandle(NULL), NULL);

    chkEscRestore_ = CreateWindowExW(0, L"BUTTON", L"Phím Esc khôi phục từ gốc",
        WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
        25, 155, 300, 24, hwnd_, (HMENU)IDC_CHK_ESC_RESTORE, GetModuleHandle(NULL), NULL);

    // Card 2: Hotkey & Shortcuts (10, 200, 680, 100)
    CreateWindowExW(0, L"BUTTON", L"Phím tắt",
        WS_CHILD | WS_VISIBLE | BS_GROUPBOX,
        10, 200, 680, 100, hwnd_, NULL, GetModuleHandle(NULL), NULL);

    CreateWindowExW(0, L"STATIC", L"Phím tắt toàn cục:",
        WS_CHILD | WS_VISIBLE | SS_LEFT,
        25, 225, 150, 20, hwnd_, NULL, GetModuleHandle(NULL), NULL);

    txtHotkey_ = CreateWindowExW(WS_EX_CLIENTEDGE, L"EDIT", L"Ctrl+Shift+V",
        WS_CHILD | WS_VISIBLE | ES_LEFT | ES_READONLY,
        180, 223, 200, 24, hwnd_, (HMENU)IDC_HOTKEY, GetModuleHandle(NULL), NULL);

    btnShortcuts_ = CreateWindowExW(0, L"BUTTON", L"Quản lý từ viết tắt...",
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
        25, 260, 200, 28, hwnd_, (HMENU)IDC_BTN_SHORTCUTS, GetModuleHandle(NULL), NULL);

    // Card 3: Behavior (10, 310, 680, 130)
    CreateWindowExW(0, L"BUTTON", L"Hành vi",
        WS_CHILD | WS_VISIBLE | BS_GROUPBOX,
        10, 310, 680, 130, hwnd_, NULL, GetModuleHandle(NULL), NULL);

    chkAutoStart_ = CreateWindowExW(0, L"BUTTON", L"Khởi động cùng Windows",
        WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
        25, 335, 300, 24, hwnd_, (HMENU)IDC_CHK_AUTOSTART, GetModuleHandle(NULL), NULL);

    chkPerApp_ = CreateWindowExW(0, L"BUTTON", L"Ghi nhớ trạng thái theo ứng dụng",
        WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
        25, 365, 300, 24, hwnd_, (HMENU)IDC_CHK_PERAPP, GetModuleHandle(NULL), NULL);

    chkAutoRestore_ = CreateWindowExW(0, L"BUTTON", L"Tự động khôi phục từ tiếng Anh",
        WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
        25, 395, 300, 24, hwnd_, (HMENU)IDC_CHK_AUTORESTORE, GetModuleHandle(NULL), NULL);

    // Card 4: Advanced (10, 450, 680, 130)
    CreateWindowExW(0, L"BUTTON", L"Nâng cao",
        WS_CHILD | WS_VISIBLE | BS_GROUPBOX,
        10, 450, 680, 130, hwnd_, NULL, GetModuleHandle(NULL), NULL);

    chkSound_ = CreateWindowExW(0, L"BUTTON", L"Âm thanh thông báo",
        WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
        25, 475, 300, 24, hwnd_, (HMENU)IDC_CHK_SOUND, GetModuleHandle(NULL), NULL);

    chkModern_ = CreateWindowExW(0, L"BUTTON", L"Dấu thanh hiện đại (hòa/hoà)",
        WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
        25, 505, 300, 24, hwnd_, (HMENU)IDC_CHK_MODERN, GetModuleHandle(NULL), NULL);

    chkCapitalize_ = CreateWindowExW(0, L"BUTTON", L"Tự động viết hoa đầu câu",
        WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
        25, 535, 300, 24, hwnd_, (HMENU)IDC_CHK_CAPITALIZE, GetModuleHandle(NULL), NULL);

    // Buttons (590, 590, 200, 80)
    CreateWindowExW(0, L"BUTTON", L"OK",
        WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
        490, 600, 90, 32, hwnd_, (HMENU)IDOK, GetModuleHandle(NULL), NULL);

    CreateWindowExW(0, L"BUTTON", L"Áp dụng",
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
        590, 600, 90, 32, hwnd_, (HMENU)IDC_BTN_SHORTCUTS + 100, GetModuleHandle(NULL), NULL);

    // Set default font
    HFONT hFont = (HFONT)GetStockObject(DEFAULT_GUI_FONT);
    EnumChildWindows(hwnd_, [](HWND child, LPARAM lParam) -> BOOL {
        SendMessage(child, WM_SETFONT, (WPARAM)lParam, TRUE);
        return TRUE;
    }, (LPARAM)hFont);
}

void SettingsWindow::LoadSettings() {
    auto& settings = Settings::Instance();

    Button_SetCheck(chkEnabled_, settings.enabled ? BST_CHECKED : BST_UNCHECKED);
    ComboBox_SetCurSel(cmbMethod_, settings.method);
    Button_SetCheck(chkWShortcut_, settings.skipWShortcut ? BST_CHECKED : BST_UNCHECKED);
    Button_SetCheck(chkBracket_, settings.bracketShortcut ? BST_CHECKED : BST_UNCHECKED);
    Button_SetCheck(chkEscRestore_, settings.escRestore ? BST_CHECKED : BST_UNCHECKED);
    Button_SetCheck(chkAutoStart_, settings.autoStart ? BST_CHECKED : BST_UNCHECKED);
    Button_SetCheck(chkPerApp_, settings.perApp ? BST_CHECKED : BST_UNCHECKED);
    Button_SetCheck(chkAutoRestore_, settings.autoRestore ? BST_CHECKED : BST_UNCHECKED);
    Button_SetCheck(chkSound_, settings.sound ? BST_CHECKED : BST_UNCHECKED);
    Button_SetCheck(chkModern_, settings.modernTone ? BST_CHECKED : BST_UNCHECKED);
    Button_SetCheck(chkCapitalize_, settings.autoCapitalize ? BST_CHECKED : BST_UNCHECKED);
}

void SettingsWindow::SaveSettings() {
    auto& settings = Settings::Instance();

    settings.enabled = Button_GetCheck(chkEnabled_) == BST_CHECKED;
    settings.method = static_cast<uint8_t>(ComboBox_GetCurSel(cmbMethod_));
    settings.skipWShortcut = Button_GetCheck(chkWShortcut_) == BST_CHECKED;
    settings.bracketShortcut = Button_GetCheck(chkBracket_) == BST_CHECKED;
    settings.escRestore = Button_GetCheck(chkEscRestore_) == BST_CHECKED;
    settings.autoStart = Button_GetCheck(chkAutoStart_) == BST_CHECKED;
    settings.perApp = Button_GetCheck(chkPerApp_) == BST_CHECKED;
    settings.autoRestore = Button_GetCheck(chkAutoRestore_) == BST_CHECKED;
    settings.sound = Button_GetCheck(chkSound_) == BST_CHECKED;
    settings.modernTone = Button_GetCheck(chkModern_) == BST_CHECKED;
    settings.autoCapitalize = Button_GetCheck(chkCapitalize_) == BST_CHECKED;

    settings.Save();
}

void SettingsWindow::ApplySettings() {
    SaveSettings();
    Settings::Instance().ApplyToEngine();
}

INT_PTR CALLBACK SettingsWindow::DialogProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    SettingsWindow* window = nullptr;

    if (msg == WM_INITDIALOG) {
        window = reinterpret_cast<SettingsWindow*>(lParam);
        SetWindowLongPtr(hwnd, GWLP_USERDATA, lParam);
    } else {
        window = reinterpret_cast<SettingsWindow*>(GetWindowLongPtr(hwnd, GWLP_USERDATA));
    }

    switch (msg) {
        case WM_INITDIALOG:
            return TRUE;

        case WM_COMMAND:
            if (LOWORD(wParam) == IDOK) {
                window->SaveSettings();
                window->ApplySettings();
                window->Hide();
                return TRUE;
            }
            if (LOWORD(wParam) == IDC_BTN_SHORTCUTS + 100) {  // Apply button
                window->ApplySettings();
                return TRUE;
            }
            if (LOWORD(wParam) == IDC_BTN_SHORTCUTS) {
                ShortcutsDialog::Instance().Show();
                return TRUE;
            }
            break;

        case WM_CLOSE:
            window->Hide();
            return TRUE;

        case WM_DESTROY:
            window->visible_ = false;
            return TRUE;
    }

    return FALSE;
}

} // namespace gonhanh
