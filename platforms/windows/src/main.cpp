#include <windows.h>
#include <objbase.h>
#include <commctrl.h>
#include "app.h"

#pragma comment(lib, "comctl32.lib")

int WINAPI wWinMain(
    _In_ HINSTANCE hInstance,
    _In_opt_ HINSTANCE /*hPrevInstance*/,
    _In_ LPWSTR /*lpCmdLine*/,
    _In_ int /*nShowCmd*/
) {
    // Initialize COM (required for Direct2D, DirectWrite, WIC)
    // Use COINIT_APARTMENTTHREADED for UI thread
    HRESULT hr = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
    if (FAILED(hr)) {
        MessageBoxW(nullptr, L"Failed to initialize COM", L"GoNhanh", MB_ICONERROR);
        return 1;
    }

    // Initialize common controls (for modern visual styles)
    INITCOMMONCONTROLSEX icex = {};
    icex.dwSize = sizeof(icex);
    icex.dwICC = ICC_STANDARD_CLASSES | ICC_WIN95_CLASSES;
    InitCommonControlsEx(&icex);

    auto& app = gonhanh::App::instance();

    if (!app.initialize(hInstance)) {
        CoUninitialize();
        return 1;
    }

    int result = app.run();

    app.shutdown();
    CoUninitialize();

    return result;
}
