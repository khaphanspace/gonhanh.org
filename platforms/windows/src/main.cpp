#include <windows.h>
#include <oleacc.h>  // For OBJID_WINDOW, OBJID_CLIENT constants
#include <commctrl.h>
#include "rust_bridge.h"
#include "keyboard_hook.h"
#include "system_tray.h"
#include "settings.h"
#include "settings_window.h"
#include "app_compat.h"
#include "per_app.h"
#include "utils.h"
#include "debug_console.h"
#include "resource.h"
#include "ime_state.h"
#include "control_protocol.h"
#include "autostart.h"

static const wchar_t* WINDOW_CLASS = gonhanh::control::kWindowClass;
static HWND g_msgWindow = nullptr;
// Broadcast by Explorer when the taskbar is created: at sign-in (GoNhanh may start first) and
// after Explorer restarts, which drops every tray icon
static UINT g_taskbarCreatedMsg = 0;
static const UINT_PTR kTrayRetryTimer = 1;
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

// Request from the `gonhanh` / `gn` CLI (control_protocol.h). Vim plugins call `vi` / `en` on
// every mode change, so a command that wouldn't change anything does nothing: no registry write,
// no toggle sound.
static LRESULT HandleControlCommand(WPARAM command, LPARAM version) {
    using gonhanh::control::Command;
    if (version != gonhanh::control::kProtocolVersion) return 0;

    auto& settings = gonhanh::Settings::Instance();
    auto setEnabled = [&](bool on) {
        if (settings.enabled != on) gonhanh::SetVietnameseEnabled(on);
    };
    auto setMethod = [&](int method) {
        if (settings.method != method) gonhanh::SetInputMethod(method);
        setEnabled(true);
    };

    switch (static_cast<Command>(command)) {
        case Command::Get: break;
        case Command::Vi: setEnabled(true); break;
        case Command::En: setEnabled(false); break;
        case Command::Toggle: gonhanh::ToggleVietnamese(); break;
        case Command::Telex: setMethod(0); break;
        case Command::Vni: setMethod(1); break;
        case Command::Quit: PostQuitMessage(0); break;  // Same as tray → Thoát, after this reply
        default: return 0;
    }

    LRESULT result = gonhanh::control::kOk;
    if (settings.enabled) result |= gonhanh::control::kEnabled;
    if (settings.method == 1) result |= gonhanh::control::kMethodVni;
    return result;
}

LRESULT CALLBACK WindowProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    if (g_taskbarCreatedMsg && msg == g_taskbarCreatedMsg) {
        if (gonhanh::SystemTray::Instance().Readd()) KillTimer(g_msgWindow, kTrayRetryTimer);
        return 0;
    }
    switch (msg) {
        case WM_TIMER:
            // The tray icon couldn't be added at startup: retry until the taskbar takes it
            if (wParam == kTrayRetryTimer && gonhanh::SystemTray::Instance().Readd()) {
                KillTimer(hwnd, kTrayRetryTimer);
            }
            return 0;

        case gonhanh::control::kMessage:
            return HandleControlCommand(wParam, lParam);

        case WM_TRAYICON:
            gonhanh::SystemTray::Instance().HandleMessage(wParam, lParam);
            return 0;

        case WM_TOGGLE_IME:
            gonhanh::ToggleVietnamese();
            return 0;

        case WM_COMMAND: {
            switch (LOWORD(wParam)) {
                case IDM_ENABLE:
                    gonhanh::ToggleVietnamese();
                    break;

                case IDM_TELEX:
                case IDM_VNI:
                    gonhanh::SetInputMethod(LOWORD(wParam) == IDM_TELEX ? 0 : 1);
                    break;

                case IDM_SETTINGS:
                    gonhanh::SettingsWindow::Instance().Show();
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
    const DWORD mutexError = GetLastError();
    // ERROR_ACCESS_DENIED: the mutex exists but was created by an instance running as
    // administrator, which this (normal) process may not open. It is still a running instance.
    const bool alreadyRunning = instanceMutex ? mutexError == ERROR_ALREADY_EXISTS
                                              : mutexError == ERROR_ACCESS_DENIED;
    if (alreadyRunning) {
        // The first instance may still be starting up (engine, settings) and not have created
        // its window yet: wait a little for it instead of silently doing nothing
        HWND running = nullptr;
        for (int i = 0; i < 50 && !running; ++i) {
            running = FindWindowExW(HWND_MESSAGE, nullptr, WINDOW_CLASS, nullptr);
            if (!running) Sleep(100);
        }
        if (running) {
            AllowSetForegroundWindow(ASFW_ANY);  // Let it bring Settings to the front
            PostMessageW(running, WM_COMMAND, IDM_SETTINGS, 0);  // dropped by UIPI if it's elevated
        }
        if (instanceMutex) CloseHandle(instanceMutex);
        return 0;
    }

    // Every keystroke waits for the keyboard hook. When the machine is busy (builds, games),
    // a normal-priority hook can miss Windows' hook timeout, so keys lag or skip the engine.
    // GoNhanh is idle between keys (a few µs per key), so high priority costs other apps nothing.
    // Not REALTIME: that would outrank the input and disk threads of the system itself.
    SetPriorityClass(GetCurrentProcess(), HIGH_PRIORITY_CLASS);

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

    g_msgWindow = hwnd;

    // Message-only windows get no broadcasts: a hidden top-level window listens for TaskbarCreated
    g_taskbarCreatedMsg = RegisterWindowMessageW(L"TaskbarCreated");
    WNDCLASSEX watchClass = {};
    watchClass.cbSize = sizeof(WNDCLASSEX);
    watchClass.lpfnWndProc = WindowProc;
    watchClass.hInstance = hInstance;
    watchClass.lpszClassName = L"GoNhanhTaskbarWatch";
    RegisterClassEx(&watchClass);
    HWND watch = CreateWindowEx(WS_EX_TOOLWINDOW, watchClass.lpszClassName, L"", WS_POPUP, 0, 0, 0, 0,
                                nullptr, nullptr, hInstance, nullptr);
    if (watch && g_taskbarCreatedMsg) {
        ChangeWindowMessageFilterEx(watch, g_taskbarCreatedMsg, MSGFLT_ALLOW, nullptr);  // when elevated
    }

    // When GoNhanh runs as administrator (to type into elevated apps), UIPI drops messages from
    // the non-elevated `gonhanh` CLI. Let the control message through; it only switches modes.
    ChangeWindowMessageFilterEx(hwnd, gonhanh::control::kMessage, MSGFLT_ALLOW, nullptr);

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
        // Usually no taskbar yet (started at sign-in before Explorer). Keep typing working and
        // add the icon on TaskbarCreated, or on this timer if that broadcast was missed.
        gonhanh::LogError(L"System tray icon not added yet; retrying");
        SetTimer(hwnd, kTrayRetryTimer, 2000, nullptr);
    }

    // Start with the hook, engine and tray icon all reflecting the saved state
    gonhanh::SyncEnabledState();

    // The app already in the foreground gets no foreground event until the user switches away:
    // with per-app mode on, apply its saved state now; either way remember it, so a toggle made
    // before the next switch is saved for it
    if (!IsTaskbarWindow(GetForegroundWindow())) {
        std::wstring appName = gonhanh::AppCompat::Instance().GetForegroundAppName();
        if (settings.perApp && !appName.empty()) {
            bool before = settings.enabled;
            gonhanh::PerAppMode::Instance().SwitchToApp(appName);
            if (settings.enabled != before) gonhanh::SyncEnabledState();
        } else {
            gonhanh::PerAppMode::Instance().TrackApp(appName);
        }
    }

    // Auto-start: make sure the logon task exists and runs this exe (GoNhanh may have moved, or an
    // earlier version left a Run entry). Runs on a background thread.
    if (settings.autoStart) gonhanh::SetAutoStart(true);

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
