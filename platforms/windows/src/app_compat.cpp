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

    // Terminals and editors with a terminal-style input path: they read keystrokes
    // per frame, so a single burst of backspaces + text drops or reorders characters
    // (diacritics lost, issue #362). Pace them (8000/25000/8000 µs).
    static const wchar_t* const kPacedApps[] = {
        // Console hosts and shells
        L"windowsterminal.exe", L"openconsole.exe", L"conhost.exe",
        L"cmd.exe", L"powershell.exe", L"pwsh.exe",
        // GPU-rendered terminals
        L"warp.exe", L"wezterm-gui.exe", L"alacritty.exe", L"rio.exe",
        // Other terminal emulators (mintty = Git Bash / MSYS2 / Cygwin)
        L"mintty.exe", L"conemu.exe", L"conemu64.exe",
        L"tabby.exe", L"hyper.exe", L"termius.exe",
        // VS Code-based editors (xterm.js / Monaco)
        L"code.exe", L"code - insiders.exe", L"cursor.exe", L"windsurf.exe",
    };
    for (const wchar_t* app : kPacedApps) {
        if (lower == app) {
            return DetectionResult(InjectionMethod::Slow, 8000, 25000, 8000);
        }
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
        // Browsers handle a single burst of input in order
        // Address bar detection would require UI Automation
        return DetectionResult(InjectionMethod::Fast, 0, 0, 0);
    }

    // Default: one unpaced burst, sent straight from the hook (sub-millisecond)
    return DetectionResult(InjectionMethod::Fast, 0, 0, 0);
}

} // namespace gonhanh
