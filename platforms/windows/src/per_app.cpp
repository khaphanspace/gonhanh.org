#include "per_app.h"
#include "settings.h"

namespace gonhanh {

static const wchar_t* APP_STATES_KEY = L"Software\\GoNhanh\\AppStates";

PerAppMode::PerAppMode() {
    InitializeCriticalSection(&lock_);
}

PerAppMode::~PerAppMode() {
    DeleteCriticalSection(&lock_);
}

PerAppMode& PerAppMode::Instance() {
    static PerAppMode instance;
    return instance;
}

void PerAppMode::Load() {
    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, APP_STATES_KEY, 0, KEY_READ, &key) != ERROR_SUCCESS) {
        return;  // No saved states yet
    }

    // Enumerate all values
    wchar_t valueName[16384];  // Maximum Registry value name length
    DWORD valueNameSize;
    DWORD index = 0;

    while (true) {
        valueNameSize = sizeof(valueName) / sizeof(wchar_t);
        DWORD type;
        DWORD enabled = 0;
        DWORD dataSize = sizeof(DWORD);

        LONG result = RegEnumValueW(key, index, valueName, &valueNameSize,
                                     nullptr, &type, (LPBYTE)&enabled, &dataSize);

        if (result == ERROR_NO_MORE_ITEMS) break;
        if (result != ERROR_SUCCESS) break;

        // Validate: must be REG_DWORD and name must end with .exe
        if (type == REG_DWORD) {
            std::wstring name(valueName);
            // Security: Validate app name ends with .exe and contains no control characters
            if (name.length() >= 5 && name.length() < 256 &&
                _wcsicmp(name.substr(name.length() - 4).c_str(), L".exe") == 0) {
                // Check for control characters
                bool valid = true;
                for (wchar_t c : name) {
                    if (c < 32 || c == 127) {  // Control characters
                        valid = false;
                        break;
                    }
                }
                if (valid) {
                    appStates_[name] = (enabled != 0);
                }
            }
        }

        index++;
    }

    RegCloseKey(key);
}

void PerAppMode::Save() {
    // Create AppStates subkey
    HKEY key;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, APP_STATES_KEY, 0, nullptr, 0,
                        KEY_WRITE, nullptr, &key, nullptr) != ERROR_SUCCESS) {
        return;
    }

    // Save each app state
    for (const auto& [appName, enabled] : appStates_) {
        DWORD value = enabled ? 1 : 0;
        RegSetValueExW(key, appName.c_str(), 0, REG_DWORD,
                      (const BYTE*)&value, sizeof(DWORD));
    }

    RegCloseKey(key);
}

bool PerAppMode::GetAppState(const std::wstring& appName) {
    auto it = appStates_.find(appName);
    if (it != appStates_.end()) {
        return it->second;
    }

    // Default: use global enabled state
    return Settings::Instance().enabled;
}

void PerAppMode::SetAppState(const std::wstring& appName, bool enabled) {
    appStates_[appName] = enabled;
    Save();
}

void PerAppMode::SwitchToApp(const std::wstring& appName) {
    if (appName.empty() || appName == currentApp_) return;

    // Thread-safe state update
    EnterCriticalSection(&lock_);

    currentApp_ = appName;

    // Get app-specific state
    bool appEnabled = GetAppState(appName);

    // Update global state (this affects keyboard hook behavior)
    auto& settings = Settings::Instance();
    settings.enabled = appEnabled;
    settings.ApplyToEngine();

    LeaveCriticalSection(&lock_);
}

} // namespace gonhanh
