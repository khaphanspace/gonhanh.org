#pragma once
#include <windows.h>
#include <string>
#include <unordered_map>

namespace gonhanh {

class PerAppMode {
public:
    static PerAppMode& Instance();

    // Load per-app states from Registry
    void Load();

    // Save per-app states to Registry
    void Save();

    // Get IME enabled state for current app
    bool GetAppState(const std::wstring& appName);

    // Set IME enabled state for current app
    void SetAppState(const std::wstring& appName, bool enabled);

    // Switch to app's saved state
    void SwitchToApp(const std::wstring& appName);

private:
    PerAppMode();
    ~PerAppMode();
    PerAppMode(const PerAppMode&) = delete;
    PerAppMode& operator=(const PerAppMode&) = delete;

    std::unordered_map<std::wstring, bool> appStates_;
    std::wstring currentApp_;
    CRITICAL_SECTION lock_;  // Thread-safety for state changes
};

} // namespace gonhanh
