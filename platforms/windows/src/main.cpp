#include <windows.h>
#include <oleacc.h>  // For OBJID_WINDOW, OBJID_CLIENT constants
#include <commctrl.h>
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
#include "ime_state.h"

static const wchar_t* WINDOW_CLASS = L"GoNhanhHidden";
static HWINEVENTHOOK g_foregroundHook = nullptr;
static HWINEVENTHOOK g_focusHook = nullptr;

// Taskbar, notification area and its overflow flyout
static bool IsTaskbarWindow(HWND hwnd) {
    HWND root = hwnd ? GetAncestor(hwnd, GA_ROOT) : nullptr;
    if (!root) return false;
    wchar_t cls[64] = {};
    GetClassNameW(root, cls, 64);
    return wcscmp(cls, L"Shell_TrayWnd") == 0 ||
           wcscmp(cls, L"Shell_SecondaryTrayWnd") == 0 ||
           wcscmp(cls, L"NotifyIconOverflowWindow") == 0 ||
           wcscmp(cls, L"TopLevelWindowForOverflowXamlIsland") == 0 ||
           wcscmp(cls, L"XamlExplorerHostIslandWindow") == 0;
}

// Event-driven foreground app and focus change detection
void CALLBACK WinEventProc(
    HWINEVENTHOOK hWinEventHook,
    DWORD event,
    HWND hwnd,
    LONG idObject,
    LONG idChild,
    DWORD dwEventThread,
    DWORD dwmsEventTime
) {
    auto& appCompat = gonhanh::AppCompat::Instance();

    if (event == EVENT_SYSTEM_FOREGROUND) {
        // App switch - clear buffer and detection cache, drop paced text still meant for the
        // previous window (the rest of it would be typed into the new one)
        gonhanh::TextInjector::Instance().CancelPendingText();
        ime_clear_all();
        appCompat.ClearDetectionCache();

        // Clicking the tray icon or taskbar makes Explorer's taskbar the foreground window.
        // Treating that as an app switch would apply (and record) the toggle to explorer.exe
        // instead of the app the user is typing in, so ignore taskbar/tray surfaces.
        auto& settings = gonhanh::Settings::Instance();
        if (!IsTaskbarWindow(hwnd)) {
            std::wstring appName = appCompat.GetForegroundAppName();
            if (!settings.perApp) {
                // Still track the app, so turning per-app mode on later saves toggles for it
                gonhanh::PerAppMode::Instance().TrackApp(appName);
            } else if (!appName.empty()) {
                bool before = settings.enabled;
                gonhanh::PerAppMode::Instance().SwitchToApp(appName);
                if (settings.enabled != before) {
                    gonhanh::SyncEnabledState();
                }
            }
        }
    } else if (event == EVENT_OBJECT_FOCUS) {
        // Focus change within app - clear buffer to start fresh in new field
        // Only clear if focus is on a window (idObject == OBJID_WINDOW or client)
        if (idObject == OBJID_WINDOW || idObject == OBJID_CLIENT) {
            // Focus moved to another control (e.g. another field in the same app): drop paced
            // text still meant for the previous one. Only when the focused window changes:
            // Chromium/Electron apps and terminals also fire focus events for the same window
            // (in-page elements, popups), which must not cut text short.
            static HWND lastFocus = nullptr;
            if (hwnd != lastFocus) {
                if (lastFocus) gonhanh::TextInjector::Instance().CancelPendingText();
                lastFocus = hwnd;
            }
            ime_clear_all();
            appCompat.ClearDetectionCache();
        }
    }
}

LRESULT CALLBACK WindowProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
        case WM_TRAYICON:
            gonhanh::SystemTray::Instance().HandleMessage(wParam, lParam);
            return 0;

        case WM_TOGGLE_IME:
            gonhanh::ToggleVietnamese();
            return 0;

        case WM_COMMAND: {
            auto& settings = gonhanh::Settings::Instance();
            auto& tray = gonhanh::SystemTray::Instance();

            switch (LOWORD(wParam)) {
                case IDM_ENABLE:
                    gonhanh::ToggleVietnamese();
                    break;

                case IDM_TELEX:
                case IDM_VNI:
                    settings.method = (LOWORD(wParam) == IDM_TELEX) ? 0 : 1;
                    settings.Save();
                    settings.ApplyToEngine();
                    tray.UpdateIcon();
                    gonhanh::SettingsWindow::Instance().Refresh();
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

    // Single instance: a second keyboard hook would process every keystroke twice.
    // Launching again (Start menu, autostart + manual) just opens the running instance's Settings.
    HANDLE instanceMutex = CreateMutexW(nullptr, FALSE, L"Local\\GoNhanh.SingleInstance");
    if (instanceMutex && GetLastError() == ERROR_ALREADY_EXISTS) {
        // The first instance may still be starting up (engine, settings) and not have created
        // its window yet: wait a little for it instead of silently doing nothing
        HWND running = nullptr;
        for (int i = 0; i < 50 && !running; ++i) {
            running = FindWindowExW(HWND_MESSAGE, nullptr, WINDOW_CLASS, nullptr);
            if (!running) Sleep(100);
        }
        if (running) {
            AllowSetForegroundWindow(ASFW_ANY);  // Let it bring Settings to the front
            PostMessageW(running, WM_COMMAND, IDM_SETTINGS, 0);
        }
        CloseHandle(instanceMutex);
        return 0;
    }

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

    // Set up event hooks for app switching and focus change detection
    // Use separate hooks since EVENT_SYSTEM_FOREGROUND and EVENT_OBJECT_FOCUS are not contiguous
    g_foregroundHook = SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND,
        NULL,           // Hook procedure is in this process
        WinEventProc,
        0, 0,           // All processes and threads
        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS
    );
    if (!g_foregroundHook) {
        gonhanh::LogError(L"Failed to install foreground event hook");
    }

    g_focusHook = SetWinEventHook(
        EVENT_OBJECT_FOCUS, EVENT_OBJECT_FOCUS,
        NULL,
        WinEventProc,
        0, 0,
        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS
    );
    if (!g_focusHook) {
        gonhanh::LogError(L"Failed to install focus event hook");
    }

    // Install keyboard hook
    auto& hook = gonhanh::KeyboardHook::Instance();
    if (!hook.Install()) {
        gonhanh::LogError(L"Failed to install keyboard hook");
        MessageBoxW(NULL, L"Failed to install keyboard hook", L"Error", MB_ICONERROR);
        return 1;
    }
    hook.SetNotifyWindow(hwnd);  // Ctrl+Space -> WM_TOGGLE_IME

    // Create system tray icon
    auto& tray = gonhanh::SystemTray::Instance();
    if (!tray.Create(hwnd)) {
        gonhanh::LogError(L"Failed to create system tray icon");
        MessageBoxW(NULL, L"Failed to create system tray icon", L"Error", MB_ICONERROR);
        return 1;
    }

    // Start with the hook, engine and tray icon all reflecting the saved state
    gonhanh::SyncEnabledState();

    // The app already in the foreground gets no foreground event until the user switches away:
    // remember it now so a toggle made before that is saved for it (per-app mode)
    if (!IsTaskbarWindow(GetForegroundWindow())) {
        gonhanh::PerAppMode::Instance().TrackApp(gonhanh::AppCompat::Instance().GetForegroundAppName());
    }

    // Startup work (registry, icons, engine init) is done; give those pages back while idle
    gonhanh::TrimWorkingSet();

    gonhanh::LogInfo(L"GoNhanh started successfully");

    // Message loop (REQUIRED for WH_KEYBOARD_LL)
    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    // Cleanup
    if (g_foregroundHook) {
        UnhookWinEvent(g_foregroundHook);
    }
    if (g_focusHook) {
        UnhookWinEvent(g_focusHook);
    }
    tray.Destroy();
    hook.Uninstall();
    ime_clear_all();

    gonhanh::LogInfo(L"GoNhanh shut down successfully");

    return 0;
}
