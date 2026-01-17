#include "app_compat.h"
#include <psapi.h>

namespace gonhanh {

AppCompat& AppCompat::Instance() {
    static AppCompat instance;
    return instance;
}

std::wstring AppCompat::GetForegroundAppName() {
    HWND hwnd = GetForegroundWindow();
    if (!hwnd) {
        // Clear cache on error
        cachedPid_ = 0;
        cachedAppName_.clear();
        return L"";
    }

    DWORD pid = 0;
    GetWindowThreadProcessId(hwnd, &pid);

    // Use cached result if same process
    if (pid == cachedPid_ && !cachedAppName_.empty()) {
        return cachedAppName_;
    }

    HANDLE process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
    if (!process) {
        // Clear cache on error
        cachedPid_ = 0;
        cachedAppName_.clear();
        return L"";
    }

    wchar_t path[MAX_PATH] = {};
    DWORD size = MAX_PATH;
    if (QueryFullProcessImageNameW(process, 0, path, &size) == 0) {
        CloseHandle(process);
        // Clear cache on error
        cachedPid_ = 0;
        cachedAppName_.clear();
        return L"";
    }

    CloseHandle(process);

    // Extract filename from full path
    std::wstring fullPath(path);
    size_t pos = fullPath.find_last_of(L"\\");
    std::wstring appName = (pos != std::wstring::npos) ? fullPath.substr(pos + 1) : fullPath;

    // Cache result
    cachedPid_ = pid;
    cachedAppName_ = appName;

    return appName;
}

bool AppCompat::IsProblematicApp(const std::wstring& appName) {
    // Known apps with compatibility issues
    static const wchar_t* problematicApps[] = {
        L"chrome.exe",
        L"msedge.exe",
        L"firefox.exe",
        L"brave.exe",
        L"opera.exe",
        L"devenv.exe",      // Visual Studio
        L"idea64.exe",      // IntelliJ IDEA
        L"pycharm64.exe",   // PyCharm
        L"code.exe",        // VS Code
        L"discord.exe",     // Discord (Electron)
        L"slack.exe",       // Slack (Electron)
        nullptr
    };

    for (int i = 0; problematicApps[i] != nullptr; ++i) {
        if (_wcsicmp(appName.c_str(), problematicApps[i]) == 0) {
            return true;
        }
    }

    return false;
}

bool AppCompat::NeedsSelectionMethod() {
    // Note: Selection method detection for address bars is complex on Windows
    // Unlike macOS with AXUIElement, Windows doesn't have easy accessibility APIs
    // For now, we always use backspace method (simpler, works in most cases)
    // Future: Implement UI Automation API to detect address bar focus
    return false;
}

} // namespace gonhanh
