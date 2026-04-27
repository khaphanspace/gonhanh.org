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

DetectionResult AppCompat::GetInjectionMethod() {
    // Check if cache is valid (TTL not expired)
    if (hasDetectionCache_) {
        DWORD now = GetTickCount();
        DWORD elapsed = now - cachedDetection_.timestamp;
        if (elapsed < DETECTION_TTL_MS) {
            return cachedDetection_;
        }
    }

    // Cache miss or expired - detect and cache
    std::wstring appName = GetForegroundAppName();
    cachedDetection_ = DetectInjectionMethod(appName);
    hasDetectionCache_ = true;

    return cachedDetection_;
}

void AppCompat::ClearDetectionCache() {
    hasDetectionCache_ = false;
    cachedDetection_ = DetectionResult();
}

DetectionResult AppCompat::DetectInjectionMethod(const std::wstring& appName) {
    // Convert to lowercase for case-insensitive comparison
    std::wstring lower = appName;
    for (auto& c : lower) {
        c = towlower(c);
    }

    // Terminals: need slow injection (8000/25000/8000 µs)
    if (lower == L"windowsterminal.exe" ||
        lower == L"cmd.exe" ||
        lower == L"powershell.exe" ||
        lower == L"pwsh.exe" ||
        lower == L"conhost.exe") {
        return DetectionResult(InjectionMethod::Slow, 8000, 25000, 8000);
    }

    // VSCode-based editors: need slow injection
    if (lower == L"code.exe" ||
        lower == L"cursor.exe" ||
        lower == L"code - insiders.exe" ||
        lower == L"windsurf.exe") {
        return DetectionResult(InjectionMethod::Slow, 8000, 25000, 8000);
    }

    // Electron chat apps: medium slow
    if (lower == L"teams.exe" ||
        lower == L"slack.exe" ||
        lower == L"discord.exe" ||
        lower == L"telegram.exe") {
        return DetectionResult(InjectionMethod::Slow, 3000, 8000, 3000);
    }

    // Browsers: check if in address bar (future: UI Automation)
    // For now, use selection method for browser address bars
    // Note: This is a simplification - proper detection requires UI Automation
    if (lower == L"chrome.exe" ||
        lower == L"msedge.exe" ||
        lower == L"firefox.exe" ||
        lower == L"brave.exe" ||
        lower == L"opera.exe") {
        // For browsers, use slightly slower timing
        // Address bar detection would require UI Automation
        return DetectionResult(InjectionMethod::Fast, 500, 1500, 800);
    }

    // Default: fast injection (200/800/500 µs)
    return DetectionResult(InjectionMethod::Fast, 200, 800, 500);
}

} // namespace gonhanh
