#include "utils.h"
#include <sstream>
#include <iomanip>
#include <ctime>
#include <malloc.h>

namespace gonhanh {

void PlayToggleSound() {
    // Play system beep sound (simple, no external dependencies)
    // MB_OK = 0x00000000 (default beep sound)
    MessageBeep(MB_OK);
}

void TrimWorkingSet() {
    // Release the CRT/Rust heaps' free pages too, then empty the working set
    _heapmin();
    SetProcessWorkingSetSizeEx(GetCurrentProcess(), static_cast<SIZE_T>(-1), static_cast<SIZE_T>(-1), 0);
}

static std::wstring GetTimestamp() {
    auto now = std::time(nullptr);
    struct tm tm;
    localtime_s(&tm, &now);  // Thread-safe version

    std::wostringstream wss;
    wss << std::put_time(&tm, L"%Y-%m-%d %H:%M:%S");
    return wss.str();
}

void LogError(const std::wstring& message) {
    std::wstring log = L"[ERROR] [" + GetTimestamp() + L"] " + message + L"\n";
    OutputDebugStringW(log.c_str());
}

void LogInfo(const std::wstring& message) {
    std::wstring log = L"[INFO] [" + GetTimestamp() + L"] " + message + L"\n";
    OutputDebugStringW(log.c_str());
}

} // namespace gonhanh
