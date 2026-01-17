#include "about_dialog.h"
#include "resource.h"
#include <shellapi.h>

namespace gonhanh {

AboutDialog& AboutDialog::Instance() {
    static AboutDialog instance;
    return instance;
}

AboutDialog::~AboutDialog() {
    if (hwnd_) {
        DestroyWindow(hwnd_);
    }
}

void AboutDialog::Show() {
    if (!hwnd_) {
        Create();
    }
    ShowWindow(hwnd_, SW_SHOW);
    SetForegroundWindow(hwnd_);
    visible_ = true;
}

void AboutDialog::Hide() {
    if (hwnd_) {
        ShowWindow(hwnd_, SW_HIDE);
        visible_ = false;
    }
}

void AboutDialog::Create() {
    hwnd_ = CreateDialogParamW(
        GetModuleHandle(NULL),
        MAKEINTRESOURCEW(IDD_ABOUT),
        NULL,
        DialogProc,
        (LPARAM)this
    );

    if (!hwnd_) {
        return;
    }

    // Logo (130, 20, 140, 140)
    HWND hLogo = CreateWindowExW(
        0, L"STATIC", NULL,
        WS_CHILD | WS_VISIBLE | SS_ICON,
        130, 20, 140, 140,
        hwnd_, NULL, GetModuleHandle(NULL), NULL
    );
    HICON hIcon = LoadIconW(GetModuleHandle(NULL), MAKEINTRESOURCEW(IDI_APP_LOGO));
    SendMessage(hLogo, STM_SETICON, (WPARAM)hIcon, 0);

    // App name (20, 180, 360, 40)
    HWND hAppName = CreateWindowExW(
        0, L"STATIC", L"Gõ Nhanh",
        WS_CHILD | WS_VISIBLE | SS_CENTER,
        20, 180, 360, 40,
        hwnd_, NULL, GetModuleHandle(NULL), NULL
    );
    HFONT hTitleFont = CreateFontW(
        28, 0, 0, 0, FW_BOLD,
        FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE,
        L"Segoe UI"
    );
    SendMessage(hAppName, WM_SETFONT, (WPARAM)hTitleFont, TRUE);

    // Version (20, 230, 360, 24)
    CreateWindowExW(
        0, L"STATIC", L"Phiên bản 1.0.0",
        WS_CHILD | WS_VISIBLE | SS_CENTER,
        20, 230, 360, 24,
        hwnd_, NULL, GetModuleHandle(NULL), NULL
    );

    // Description (20, 260, 360, 40)
    CreateWindowExW(
        0, L"STATIC", L"Bộ gõ tiếng Việt cho Windows\nNhanh, nhẹ, không theo dõi người dùng",
        WS_CHILD | WS_VISIBLE | SS_CENTER,
        20, 260, 360, 40,
        hwnd_, NULL, GetModuleHandle(NULL), NULL
    );

    // Copyright (20, 310, 360, 24)
    CreateWindowExW(
        0, L"STATIC", L"© 2026 Gõ Nhanh. MIT License.",
        WS_CHILD | WS_VISIBLE | SS_CENTER,
        20, 310, 360, 24,
        hwnd_, NULL, GetModuleHandle(NULL), NULL
    );

    // GitHub link (20, 345, 360, 24)
    HWND hLink = CreateWindowExW(
        0, L"STATIC", L"https://github.com/khaphanspace/gonhanh.org",
        WS_CHILD | WS_VISIBLE | SS_CENTER | SS_NOTIFY,
        20, 345, 360, 24,
        hwnd_, (HMENU)1001, GetModuleHandle(NULL), NULL
    );
    HFONT hLinkFont = CreateFontW(
        16, 0, 0, 0, FW_NORMAL,
        FALSE, TRUE, FALSE,  // Underline
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE,
        L"Segoe UI"
    );
    SendMessage(hLink, WM_SETFONT, (WPARAM)hLinkFont, TRUE);

    // Close button (150, 385, 100, 32)
    CreateWindowExW(
        0, L"BUTTON", L"Đóng",
        WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
        150, 385, 100, 32,
        hwnd_, (HMENU)IDOK, GetModuleHandle(NULL), NULL
    );

    // Set default font for non-custom controls
    HFONT hFont = (HFONT)GetStockObject(DEFAULT_GUI_FONT);
    EnumChildWindows(hwnd_, [](HWND child, LPARAM lParam) -> BOOL {
        if (GetWindowLongPtr(child, GWL_STYLE) & BS_DEFPUSHBUTTON) {
            SendMessage(child, WM_SETFONT, lParam, TRUE);
        }
        return TRUE;
    }, (LPARAM)hFont);
}

INT_PTR CALLBACK AboutDialog::DialogProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    AboutDialog* dialog = nullptr;

    if (msg == WM_INITDIALOG) {
        dialog = reinterpret_cast<AboutDialog*>(lParam);
        SetWindowLongPtr(hwnd, GWLP_USERDATA, lParam);
    } else {
        dialog = reinterpret_cast<AboutDialog*>(GetWindowLongPtr(hwnd, GWLP_USERDATA));
    }

    switch (msg) {
        case WM_INITDIALOG:
            return TRUE;

        case WM_COMMAND:
            if (LOWORD(wParam) == IDOK) {
                dialog->Hide();
                return TRUE;
            }
            if (LOWORD(wParam) == 1001) {  // GitHub link
                ShellExecuteW(NULL, L"open",
                    L"https://github.com/khaphanspace/gonhanh.org",
                    NULL, NULL, SW_SHOW);
                return TRUE;
            }
            break;

        case WM_CTLCOLORSTATIC: {
            // Make GitHub link blue
            HDC hdc = (HDC)wParam;
            HWND hCtrl = (HWND)lParam;
            if (GetDlgCtrlID(hCtrl) == 1001) {
                SetTextColor(hdc, RGB(0, 102, 204));
                SetBkMode(hdc, TRANSPARENT);
                return (INT_PTR)GetStockObject(NULL_BRUSH);
            }
            break;
        }

        case WM_SETCURSOR: {
            // Change cursor to hand over GitHub link
            HWND hCtrl = (HWND)wParam;
            if (GetDlgCtrlID(hCtrl) == 1001) {
                SetCursor(LoadCursor(NULL, IDC_HAND));
                return TRUE;
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
