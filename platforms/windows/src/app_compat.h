#pragma once
#include <windows.h>
#include <string>

namespace gonhanh {

class AppCompat {
public:
    static AppCompat& Instance();

    // Get current foreground app name
    std::wstring GetForegroundAppName();

    // Check if app has known compatibility issues
    bool IsProblematicApp(const std::wstring& appName);

    // Check if current context needs selection method (address bars)
    bool NeedsSelectionMethod();

private:
    AppCompat() = default;
    ~AppCompat() = default;
    AppCompat(const AppCompat&) = delete;
    AppCompat& operator=(const AppCompat&) = delete;

    std::wstring cachedAppName_;
    DWORD cachedPid_ = 0;
};

} // namespace gonhanh
