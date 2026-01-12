#include <Windows.h>
#include "RustBridge.h"
#include "KeyboardHook.h"

// Hidden window class name
const wchar_t* WINDOW_CLASS = L"GoNhanhHiddenWindow";

// Window procedure (minimal - just handles WM_DESTROY)
LRESULT CALLBACK WindowProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
    switch (uMsg) {
        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;
    }
    return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

// Entry point
int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    // Unused parameters
    (void)hPrevInstance;
    (void)lpCmdLine;
    (void)nCmdShow;

    // Register window class for message-only window
    WNDCLASSEX wc = {};
    wc.cbSize = sizeof(WNDCLASSEX);
    wc.lpfnWndProc = WindowProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = WINDOW_CLASS;

    if (!RegisterClassEx(&wc)) {
        MessageBox(nullptr, L"Failed to register window class", L"Error", MB_OK | MB_ICONERROR);
        return 1;
    }

    // Create hidden message-only window (no visible UI)
    // HWND_MESSAGE = message-only window (no desktop window)
    HWND hwnd = CreateWindowEx(
        0,                      // dwExStyle
        WINDOW_CLASS,           // lpClassName
        L"GoNhanh",             // lpWindowName
        0,                      // dwStyle
        0, 0, 0, 0,             // x, y, width, height (ignored for message-only)
        HWND_MESSAGE,           // hWndParent (message-only window)
        nullptr,                // hMenu
        hInstance,              // hInstance
        nullptr                 // lpParam
    );

    if (!hwnd) {
        MessageBox(nullptr, L"Failed to create window", L"Error", MB_OK | MB_ICONERROR);
        return 1;
    }

    // Initialize Rust engine
    RustBridge::Init();

    // Configure engine (default settings)
    RustBridge::SetEnabled(true);
    RustBridge::SetMethod(0);  // Telex (0) or VNI (1)
    RustBridge::SetModern(true);  // Modern tone placement
    RustBridge::SetEnglishAutoRestore(true);  // Auto-restore English words
    RustBridge::SetAutoCapitalize(false);  // Disable for now

    // Add default shortcuts
    RustBridge::AddShortcut("vn", "Việt Nam");
    RustBridge::AddShortcut("hn", "Hà Nội");
    RustBridge::AddShortcut("hcm", "Hồ Chí Minh");
    RustBridge::AddShortcut("tphcm", "Thành phố Hồ Chí Minh");

    // Install keyboard hook
    if (!KeyboardHook::Install()) {
        MessageBox(
            nullptr,
            L"Failed to install keyboard hook.\n\n"
            L"Possible causes:\n"
            L"- Antivirus blocking the application\n"
            L"- Another IME is running (UniKey, OpenKey, EVKey)\n\n"
            L"Please close other Vietnamese IMEs and try again.",
            L"Gõ Nhanh - Error",
            MB_OK | MB_ICONERROR
        );
        return 1;
    }

    // Start worker thread
    KeyboardHook::StartWorker();

    // Show success message (temporary - will be replaced by system tray in Phase 2)
    MessageBox(
        nullptr,
        L"Gõ Nhanh started successfully!\n\n"
        L"Vietnamese input is now enabled (Telex mode).\n"
        L"Test in Notepad: a + s = á\n\n"
        L"Note: This is Phase 1 (Core Infrastructure).\n"
        L"System tray and settings UI will be added in Phase 2/3.\n\n"
        L"Press OK to continue. Close this window to quit.",
        L"Gõ Nhanh - Phase 1",
        MB_OK | MB_ICONINFORMATION
    );

    // Message loop
    MSG msg;
    while (GetMessage(&msg, nullptr, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    // Cleanup
    KeyboardHook::StopWorker();
    KeyboardHook::Uninstall();

    return (int)msg.wParam;
}
