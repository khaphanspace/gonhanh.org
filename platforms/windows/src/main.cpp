#include <windows.h>
#include "rust_bridge.h"

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrev, LPSTR cmdLine, int nShow) {
    // Initialize Rust engine
    ime_init();
    ime_method(0);  // Default: Telex
    ime_enabled(true);

    // Test FFI call - keycode 0 corresponds to 'a' key in macOS mapping
    ImeResultGuard result(ime_key(0, false, false));
    if (result) {
        // Success - FFI working
        OutputDebugStringA("FFI OK: ime_key returned valid result\n");

        // Test UTF conversion if we have chars
        if (result->count > 0) {
            std::wstring text = gonhanh::Utf32ToUtf16(result->chars, result->count);
            OutputDebugStringW(L"Converted text: ");
            OutputDebugStringW(text.c_str());
            OutputDebugStringW(L"\n");
        }
    }

    ime_clear();

    // Minimal message box for testing
    MessageBoxW(NULL, L"Gõ Nhanh - FFI Test OK", L"Test", MB_OK);
    return 0;
}
