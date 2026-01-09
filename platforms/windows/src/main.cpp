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
    // Initialize COM (needed for some Windows APIs)
    CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);

    // Initialize common controls
    INITCOMMONCONTROLSEX icex = {};
    icex.dwSize = sizeof(icex);
    icex.dwICC = ICC_STANDARD_CLASSES;
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
