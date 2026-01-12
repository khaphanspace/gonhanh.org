#include "shortcuts_dialog.h"
#include "resource.h"
#include "settings.h"
#include <commdlg.h>
#include <fstream>
#include <sstream>

namespace gonhanh {

ShortcutsDialog& ShortcutsDialog::Instance() {
    static ShortcutsDialog instance;
    return instance;
}

ShortcutsDialog::~ShortcutsDialog() {
    if (hwnd_) {
        DestroyWindow(hwnd_);
    }
}

void ShortcutsDialog::Show() {
    if (!hwnd_) {
        Create();
    }
    LoadShortcuts();
    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
    visible_ = true;
}

void ShortcutsDialog::Hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
        visible_ = false;
    }
}

void ShortcutsDialog::Create() {
    hwnd_ = CreateDialogParamW(
        GetModuleHandle(NULL),
        MAKEINTRESOURCEW(IDD_SHORTCUTS),
        NULL,
        DialogProc,
        (LPARAM)this
    );

    if (!hwnd_) {
        return;
    }

    CreateControls();
}

void ShortcutsDialog::CreateControls() {
    // ListView (10, 10, 650, 350)
    listView_ = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        WC_LISTVIEW,
        NULL,
        WS_CHILD | WS_VISIBLE | LVS_REPORT | LVS_EDITLABELS | LVS_SINGLESEL,
        10, 10, 650, 350,
        hwnd_,
        (HMENU)IDC_SHORTCUTS_LIST,
        GetModuleHandle(NULL),
        NULL
    );

    // Set extended styles
    ListView_SetExtendedListViewStyle(listView_,
        LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES | LVS_EX_CHECKBOXES);

    // Add columns
    LVCOLUMNW col = {};
    col.mask = LVCF_TEXT | LVCF_WIDTH | LVCF_FMT;
    col.fmt = LVCFMT_LEFT;

    col.pszText = const_cast<LPWSTR>(L"✓");
    col.cx = 40;
    ListView_InsertColumn(listView_, 0, &col);

    col.pszText = const_cast<LPWSTR>(L"Từ viết tắt");
    col.cx = 200;
    ListView_InsertColumn(listView_, 1, &col);

    col.pszText = const_cast<LPWSTR>(L"Thay thế");
    col.cx = 350;
    ListView_InsertColumn(listView_, 2, &col);

    col.pszText = const_cast<LPWSTR>(L"");
    col.cx = 60;
    ListView_InsertColumn(listView_, 3, &col);

    // Buttons (10, 370, 650, 40)
    CreateWindowExW(0, L"BUTTON", L"Thêm mới",
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
        10, 370, 100, 32, hwnd_, (HMENU)IDC_BTN_ADD, GetModuleHandle(NULL), NULL);

    CreateWindowExW(0, L"BUTTON", L"Nhập từ file...",
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
        120, 370, 120, 32, hwnd_, (HMENU)IDC_BTN_IMPORT, GetModuleHandle(NULL), NULL);

    CreateWindowExW(0, L"BUTTON", L"Xuất ra file...",
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
        250, 370, 120, 32, hwnd_, (HMENU)IDC_BTN_EXPORT, GetModuleHandle(NULL), NULL);

    CreateWindowExW(0, L"BUTTON", L"OK",
        WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
        560, 370, 100, 32, hwnd_, (HMENU)IDOK, GetModuleHandle(NULL), NULL);

    // Set default font
    HFONT hFont = (HFONT)GetStockObject(DEFAULT_GUI_FONT);
    SendMessage(listView_, WM_SETFONT, (WPARAM)hFont, TRUE);
    EnumChildWindows(hwnd_, [](HWND child, LPARAM lParam) -> BOOL {
        SendMessage(child, WM_SETFONT, (WPARAM)lParam, TRUE);
        return TRUE;
    }, (LPARAM)hFont);
}

void ShortcutsDialog::LoadShortcuts() {
    ListView_DeleteAllItems(listView_);

    auto& settings = Settings::Instance();
    for (size_t i = 0; i < settings.shortcuts.size(); ++i) {
        const auto& shortcut = settings.shortcuts[i];

        LVITEMW item = {};
        item.mask = LVIF_TEXT;
        item.iItem = static_cast<int>(i);
        item.iSubItem = 0;
        item.pszText = const_cast<LPWSTR>(L"");
        int index = ListView_InsertItem(listView_, &item);

        // Set checkbox state
        ListView_SetCheckState(listView_, index, shortcut.enabled);

        // Set trigger
        ListView_SetItemText(listView_, index, 1, const_cast<LPWSTR>(shortcut.trigger.c_str()));

        // Set replacement
        ListView_SetItemText(listView_, index, 2, const_cast<LPWSTR>(shortcut.replacement.c_str()));

        // Set delete button text
        ListView_SetItemText(listView_, index, 3, const_cast<LPWSTR>(L"Xóa"));
    }
}

void ShortcutsDialog::SaveShortcuts() {
    auto& settings = Settings::Instance();
    settings.shortcuts.clear();

    int count = ListView_GetItemCount(listView_);
    for (int i = 0; i < count; ++i) {
        Shortcut shortcut;
        shortcut.enabled = ListView_GetCheckState(listView_, i) != 0;

        wchar_t buffer[256];
        ListView_GetItemText(listView_, i, 1, buffer, 256);
        shortcut.trigger = buffer;

        ListView_GetItemText(listView_, i, 2, buffer, 256);
        shortcut.replacement = buffer;

        if (!shortcut.trigger.empty() && !shortcut.replacement.empty()) {
            settings.shortcuts.push_back(shortcut);
        }
    }

    settings.Save();
    settings.ApplyToEngine();
}

void ShortcutsDialog::AddShortcut() {
    LVITEMW item = {};
    item.mask = LVIF_TEXT;
    item.iItem = ListView_GetItemCount(listView_);
    item.iSubItem = 0;
    item.pszText = const_cast<LPWSTR>(L"");
    int index = ListView_InsertItem(listView_, &item);

    ListView_SetCheckState(listView_, index, TRUE);
    ListView_SetItemText(listView_, index, 1, const_cast<LPWSTR>(L"btw"));
    ListView_SetItemText(listView_, index, 2, const_cast<LPWSTR>(L"by the way"));
    ListView_SetItemText(listView_, index, 3, const_cast<LPWSTR>(L"Xóa"));

    // Start editing
    ListView_EditLabel(listView_, index);
}

void ShortcutsDialog::DeleteShortcut(int index) {
    if (index >= 0 && index < ListView_GetItemCount(listView_)) {
        ListView_DeleteItem(listView_, index);
        SaveShortcuts();
    }
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

    if (!GetOpenFileNameW(&ofn)) {
        return;
    }

    std::wifstream file(filename);
    if (!file.is_open()) {
        MessageBoxW(hwnd_, L"Không thể mở file", L"Lỗi", MB_ICONERROR);
        return;
    }

    ListView_DeleteAllItems(listView_);
    auto& settings = Settings::Instance();
    settings.shortcuts.clear();

    std::wstring line;
    while (std::getline(file, line)) {
        size_t pos = line.find(L'\t');
        if (pos == std::wstring::npos) {
            pos = line.find(L'=');
        }
        if (pos == std::wstring::npos) {
            continue;
        }

        Shortcut shortcut;
        shortcut.trigger = line.substr(0, pos);
        shortcut.replacement = line.substr(pos + 1);
        shortcut.enabled = true;

        // Trim whitespace
        shortcut.trigger.erase(0, shortcut.trigger.find_first_not_of(L" \t\r\n"));
        shortcut.trigger.erase(shortcut.trigger.find_last_not_of(L" \t\r\n") + 1);
        shortcut.replacement.erase(0, shortcut.replacement.find_first_not_of(L" \t\r\n"));
        shortcut.replacement.erase(shortcut.replacement.find_last_not_of(L" \t\r\n") + 1);

        if (!shortcut.trigger.empty() && !shortcut.replacement.empty()) {
            settings.shortcuts.push_back(shortcut);
        }
    }

    file.close();
    LoadShortcuts();
    SaveShortcuts();

    MessageBoxW(hwnd_, L"Nhập thành công!", L"Thông báo", MB_ICONINFORMATION);
}

void ShortcutsDialog::ExportShortcuts() {
    OPENFILENAMEW ofn = {};
    wchar_t filename[MAX_PATH] = L"shortcuts.txt";

    ofn.lStructSize = sizeof(OPENFILENAMEW);
    ofn.hwndOwner = hwnd_;
    ofn.lpstrFilter = L"Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0";
    ofn.lpstrFile = filename;
    ofn.nMaxFile = MAX_PATH;
    ofn.lpstrDefExt = L"txt";
    ofn.Flags = OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST;

    if (!GetSaveFileNameW(&ofn)) {
        return;
    }

    std::wofstream file(filename);
    if (!file.is_open()) {
        MessageBoxW(hwnd_, L"Không thể tạo file", L"Lỗi", MB_ICONERROR);
        return;
    }

    auto& settings = Settings::Instance();
    for (const auto& shortcut : settings.shortcuts) {
        file << shortcut.trigger << L"\t" << shortcut.replacement << L"\n";
    }

    file.close();
    MessageBoxW(hwnd_, L"Xuất thành công!", L"Thông báo", MB_ICONINFORMATION);
}

INT_PTR CALLBACK ShortcutsDialog::DialogProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    ShortcutsDialog* dialog = nullptr;

    if (msg == WM_INITDIALOG) {
        dialog = reinterpret_cast<ShortcutsDialog*>(lParam);
        SetWindowLongPtr(hwnd, GWLP_USERDATA, lParam);
    } else {
        dialog = reinterpret_cast<ShortcutsDialog*>(GetWindowLongPtr(hwnd, GWLP_USERDATA));
    }

    switch (msg) {
        case WM_INITDIALOG:
            return TRUE;

        case WM_COMMAND:
            if (LOWORD(wParam) == IDOK) {
                dialog->SaveShortcuts();
                dialog->Hide();
                return TRUE;
            }
            if (LOWORD(wParam) == IDC_BTN_ADD) {
                dialog->AddShortcut();
                return TRUE;
            }
            if (LOWORD(wParam) == IDC_BTN_IMPORT) {
                dialog->ImportShortcuts();
                return TRUE;
            }
            if (LOWORD(wParam) == IDC_BTN_EXPORT) {
                dialog->ExportShortcuts();
                return TRUE;
            }
            break;

        case WM_NOTIFY: {
            NMHDR* nmhdr = reinterpret_cast<NMHDR*>(lParam);
            if (nmhdr->idFrom == IDC_SHORTCUTS_LIST) {
                if (nmhdr->code == NM_CLICK) {
                    // Check if delete button clicked
                    NMITEMACTIVATE* nmitem = reinterpret_cast<NMITEMACTIVATE*>(lParam);
                    if (nmitem->iSubItem == 3) {  // Delete column
                        dialog->DeleteShortcut(nmitem->iItem);
                    }
                }
                else if (nmhdr->code == LVN_ITEMCHANGED) {
                    // Checkbox state changed
                    NMLISTVIEW* nmlv = reinterpret_cast<NMLISTVIEW*>(lParam);
                    if ((nmlv->uChanged & LVIF_STATE) &&
                        ((nmlv->uNewState ^ nmlv->uOldState) & LVIS_STATEIMAGEMASK)) {
                        dialog->SaveShortcuts();
                    }
                }
                else if (nmhdr->code == LVN_ENDLABELEDIT) {
                    NMLVDISPINFO* nmlvdi = reinterpret_cast<NMLVDISPINFO*>(lParam);
                    if (nmlvdi->item.pszText) {
                        ListView_SetItemText(dialog->listView_, nmlvdi->item.iItem,
                            nmlvdi->item.iSubItem, nmlvdi->item.pszText);
                        dialog->SaveShortcuts();
                    }
                    return TRUE;
                }
            }
            break;
        }

        case WM_CLOSE:
            dialog->Hide();
            return TRUE;

        case WM_DESTROY:
            dialog->visible_ = false;
            return TRUE;
    }

    return FALSE;
}

} // namespace gonhanh
