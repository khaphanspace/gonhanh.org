#include <windows.h>
#include "rust_bridge.h"
#include "keyboard_hook.h"
#include "system_tray.h"
#include "settings.h"
#include "settings_window.h"
#include "about_dialog.h"
#include "app_compat.h"
#include "per_app.h"
#include "utils.h"
#include "debug_console.h"
#include "resource.h"

static const wchar_t* WINDOW_CLASS = L"GoNhanhHidden";
static HWINEVENTHOOK g_eventHook = nullptr;

// Event-driven foreground app change detection (more efficient than timer polling)
void CALLBACK WinEventProc(
    HWINEVENTHOOK hWinEventHook,
    DWORD event,
    HWND hwnd,
    LONG idObject,
    LONG idChild,
    DWORD dwEventThread,
    DWORD dwmsEventTime
) {
    if (event == EVENT_SYSTEM_FOREGROUND) {
        auto& settings = gonhanh::Settings::Instance();
        if (settings.perApp) {
            auto& appCompat = gonhanh::AppCompat::Instance();
            std::wstring appName = appCompat.GetForegroundAppName();
            if (!appName.empty()) {
                gonhanh::PerAppMode::Instance().SwitchToApp(appName);
            }
        }
    }
}

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
                    if (settings.sound) {
                        gonhanh::PlayToggleSound();
                    }
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
#ifdef GONHANH_DEBUG_CONSOLE
    // Create debug console for troubleshooting
    auto& console = gonhanh::DebugConsole::Instance();
    console.Create();
    console.Log(L"[STARTUP] GoNhanh starting...");
#endif

    // Initialize COM for common controls
    INITCOMMONCONTROLSEX icc = {};
    icc.dwSize = sizeof(INITCOMMONCONTROLSEX);
    icc.dwICC = ICC_LISTVIEW_CLASSES | ICC_STANDARD_CLASSES;
    InitCommonControlsEx(&icc);

    // Initialize Rust engine
    ime_init();
    gonhanh::LogInfo(L"Rust engine initialized");

    // Load settings from Registry
    auto& settings = gonhanh::Settings::Instance();
    settings.Load();
    settings.ApplyToEngine();
    gonhanh::LogInfo(L"Settings loaded from Registry");

    // Load per-app mode states
    auto& perApp = gonhanh::PerAppMode::Instance();
    perApp.Load();
    gonhanh::LogInfo(L"Per-app mode states loaded");

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
        gonhanh::LogError(L"Failed to create message window");
        MessageBoxW(NULL, L"Failed to create message window", L"Error", MB_ICONERROR);
        return 1;
    }

    // Set up event hook for app switching detection (event-driven, more efficient than timer)
    g_eventHook = SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND,
        NULL,           // Hook procedure is in this process
        WinEventProc,
        0, 0,           // All processes and threads
        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS
    );
    if (!g_eventHook) {
        gonhanh::LogError(L"Failed to install WinEvent hook for app switching");
    }

    // Install keyboard hook
    auto& hook = gonhanh::KeyboardHook::Instance();
    if (!hook.Install()) {
        gonhanh::LogError(L"Failed to install keyboard hook");
        MessageBoxW(NULL, L"Failed to install keyboard hook", L"Error", MB_ICONERROR);
        return 1;
    }

    // Create system tray icon
    auto& tray = gonhanh::SystemTray::Instance();
    if (!tray.Create(hwnd)) {
        gonhanh::LogError(L"Failed to create system tray icon");
        MessageBoxW(NULL, L"Failed to create system tray icon", L"Error", MB_ICONERROR);
        return 1;
    }

    gonhanh::LogInfo(L"GoNhanh started successfully");

    // Message loop (REQUIRED for WH_KEYBOARD_LL)
    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    // Cleanup
    if (g_eventHook) {
        UnhookWinEvent(g_eventHook);
    }
    tray.Destroy();
    hook.Uninstall();
    ime_clear_all();

    gonhanh::LogInfo(L"GoNhanh shut down successfully");

    return 0;
}
