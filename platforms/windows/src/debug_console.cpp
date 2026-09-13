#include "debug_console.h"
#include <io.h>
#include <fcntl.h>
#include <iostream>

namespace gonhanh {

DebugConsole& DebugConsole::Instance() {
    static DebugConsole instance;
    return instance;
}

DebugConsole::~DebugConsole() {
    if (consoleAvailable_) {
        FreeConsole();
    }
}

void DebugConsole::Create() {
    if (consoleAvailable_) return;

    // Allocate console
    if (!AllocConsole()) {
        return;
    }

    // Redirect stdout/stderr to console
    FILE* fp;
    freopen_s(&fp, "CONOUT$", "w", stdout);
    freopen_s(&fp, "CONOUT$", "w", stderr);

    // Set console title
    SetConsoleTitleW(L"GoNhanh Debug Console");

    // Enable UTF-8 output
    SetConsoleOutputCP(CP_UTF8);

    consoleAvailable_ = true;

    // Print startup banner
    wprintf(L"═══════════════════════════════════════════════════════════\n");
    wprintf(L"  GoNhanh Debug Console\n");
    wprintf(L"  Vietnamese Input Method - Windows C++ Implementation\n");
    wprintf(L"═══════════════════════════════════════════════════════════\n\n");
}

void DebugConsole::Log(const std::wstring& message) {
    // Always log to OutputDebugString
    OutputDebugStringW(message.c_str());
    OutputDebugStringW(L"\n");

    // If console available, also print there
    if (consoleAvailable_) {
        wprintf(L"%s\n", message.c_str());
        fflush(stdout);
    }
}

} // namespace gonhanh
