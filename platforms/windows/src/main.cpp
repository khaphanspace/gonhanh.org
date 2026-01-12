#include <windows.h>
#include "rust_bridge.h"
#include "keyboard_hook.h"
#include "system_tray.h"
#include "settings.h"
#include "settings_window.h"
#include "about_dialog.h"
#include "resource.h"

static const wchar_t* WINDOW_CLASS = L"GoNhanhHidden";

LRESULT CALLBACK WindowProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
        case WM_TRAYICON:
            gonhanh::SystemTray::Instance().HandleMessage(wParam, lParam);
            return 0;

        case WM_COMMAND: {
            auto& settings = gonhanh::Settings::Instance();
            auto& tray = gonhanh::SystemTray::Instance();

            switch (LOWORD(wParam)) {
                case IDM_ENABLE:
                    settings.enabled = !settings.enabled;
                    settings.Save();
                    tray.UpdateIcon();
                    break;

                case IDM_TELEX:
                    settings.method = 0;
                    settings.Save();
                    tray.UpdateIcon();
                    break;

                case IDM_VNI:
                    settings.method = 1;
                    settings.Save();
                    tray.UpdateIcon();
                    break;

                case IDM_SETTINGS:
                    gonhanh::SettingsWindow::Instance().Show();
                    break;

                case IDM_ABOUT:
                    gonhanh::AboutDialog::Instance().Show();
                    break;

                case IDM_EXIT:
                    PostQuitMessage(0);
                    break;
            }
            return 0;
        }

        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;

        default:
            return DefWindowProc(hwnd, msg, wParam, lParam);
    }
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrev, LPSTR cmdLine, int nShow) {
    // Initialize COM for common controls
    INITCOMMONCONTROLSEX icc = {};
    icc.dwSize = sizeof(INITCOMMONCONTROLSEX);
    icc.dwICC = ICC_LISTVIEW_CLASSES | ICC_STANDARD_CLASSES;
    InitCommonControlsEx(&icc);

    // Initialize Rust engine
    ime_init();

    // Load settings from Registry
    auto& settings = gonhanh::Settings::Instance();
    settings.Load();
    settings.ApplyToEngine();

    // Register window class
    WNDCLASSEX wc = {};
    wc.cbSize = sizeof(WNDCLASSEX);
    wc.lpfnWndProc = WindowProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = WINDOW_CLASS;
    RegisterClassEx(&wc);

    // Create hidden message-only window
    HWND hwnd = CreateWindowEx(
        0, WINDOW_CLASS, L"GoNhanhMsg",
        0, 0, 0, 0, 0,
        HWND_MESSAGE,  // Critical: message-only window
        NULL, hInstance, NULL
    );

    if (!hwnd) {
        MessageBoxW(NULL, L"Failed to create message window", L"Error", MB_ICONERROR);
        return 1;
    }

    // Install keyboard hook
    auto& hook = gonhanh::KeyboardHook::Instance();
    if (!hook.Install()) {
        MessageBoxW(NULL, L"Failed to install keyboard hook", L"Error", MB_ICONERROR);
        return 1;
    }

    // Create system tray icon
    auto& tray = gonhanh::SystemTray::Instance();
    if (!tray.Create(hwnd)) {
        MessageBoxW(NULL, L"Failed to create system tray icon", L"Error", MB_ICONERROR);
        return 1;
    }

    // Message loop (REQUIRED for WH_KEYBOARD_LL)
    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    // Cleanup
    tray.Destroy();
    hook.Uninstall();
    ime_clear_all();

    return 0;
}
