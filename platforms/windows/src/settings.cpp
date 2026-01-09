#include "settings.h"

namespace gonhanh {

Settings& Settings::instance() {
    static Settings instance;
    return instance;
}

void Settings::load() {
    enabled_ = load_bool(L"Enabled", true);
    input_method_ = load_int(L"InputMethod", 0);
    modern_tone_ = load_bool(L"ModernTone", true);
    auto_start_ = load_bool(L"AutoStart", false);
    w_shortcut_ = load_bool(L"AutoWShortcut", true);
    esc_restore_ = load_bool(L"EscRestore", false);
    english_auto_restore_ = load_bool(L"EnglishAutoRestore", false);
    auto_capitalize_ = load_bool(L"AutoCapitalize", false);
    sound_enabled_ = load_bool(L"SoundEnabled", true);
    per_app_mode_ = load_bool(L"PerAppModeEnabled", false);
    bracket_shortcut_ = load_bool(L"BracketShortcut", false);
    first_run_ = load_bool(L"FirstRun", true);
    onboarding_completed_ = load_bool(L"OnboardingCompleted", false);
    toggle_shortcut_ = static_cast<uint32_t>(load_int(L"ToggleShortcut", 0));
}

void Settings::save() {
    save_value(L"Enabled", enabled_);
    save_value(L"InputMethod", input_method_);
    save_value(L"ModernTone", modern_tone_);
    save_value(L"AutoStart", auto_start_);
    save_value(L"AutoWShortcut", w_shortcut_);
    save_value(L"EscRestore", esc_restore_);
    save_value(L"EnglishAutoRestore", english_auto_restore_);
    save_value(L"AutoCapitalize", auto_capitalize_);
    save_value(L"SoundEnabled", sound_enabled_);
    save_value(L"PerAppModeEnabled", per_app_mode_);
    save_value(L"BracketShortcut", bracket_shortcut_);
    save_value(L"FirstRun", first_run_);
    save_value(L"OnboardingCompleted", onboarding_completed_);
    save_value(L"ToggleShortcut", static_cast<int>(toggle_shortcut_));
}

void Settings::save_value(const wchar_t* name, bool value) {
    HKEY key;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, REG_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &key, nullptr) == ERROR_SUCCESS) {
        DWORD dw = value ? 1 : 0;
        RegSetValueExW(key, name, 0, REG_DWORD, reinterpret_cast<const BYTE*>(&dw), sizeof(dw));
        RegCloseKey(key);
    }
}

void Settings::save_value(const wchar_t* name, int value) {
    HKEY key;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, REG_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &key, nullptr) == ERROR_SUCCESS) {
        DWORD dw = static_cast<DWORD>(value);
        RegSetValueExW(key, name, 0, REG_DWORD, reinterpret_cast<const BYTE*>(&dw), sizeof(dw));
        RegCloseKey(key);
    }
}

bool Settings::load_bool(const wchar_t* name, bool default_val) {
    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REG_PATH, 0, KEY_READ, &key) == ERROR_SUCCESS) {
        DWORD value = 0;
        DWORD size = sizeof(value);
        DWORD type = 0;
        if (RegQueryValueExW(key, name, nullptr, &type, reinterpret_cast<BYTE*>(&value), &size) == ERROR_SUCCESS) {
            RegCloseKey(key);
            return value != 0;
        }
        RegCloseKey(key);
    }
    return default_val;
}

int Settings::load_int(const wchar_t* name, int default_val) {
    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REG_PATH, 0, KEY_READ, &key) == ERROR_SUCCESS) {
        DWORD value = 0;
        DWORD size = sizeof(value);
        DWORD type = 0;
        if (RegQueryValueExW(key, name, nullptr, &type, reinterpret_cast<BYTE*>(&value), &size) == ERROR_SUCCESS) {
            RegCloseKey(key);
            return static_cast<int>(value);
        }
        RegCloseKey(key);
    }
    return default_val;
}

void Settings::set_auto_start(bool v) {
    auto_start_ = v;
    save_value(L"AutoStart", v);
    update_auto_start_registry();
}

void Settings::update_auto_start_registry() {
    HKEY key;
    const wchar_t* startup_path = L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";

    if (auto_start_) {
        // Add to startup
        if (RegOpenKeyExW(HKEY_CURRENT_USER, startup_path, 0, KEY_WRITE, &key) == ERROR_SUCCESS) {
            wchar_t exe_path[MAX_PATH];
            GetModuleFileNameW(nullptr, exe_path, MAX_PATH);
            RegSetValueExW(key, L"GoNhanh", 0, REG_SZ,
                          reinterpret_cast<const BYTE*>(exe_path),
                          static_cast<DWORD>((wcslen(exe_path) + 1) * sizeof(wchar_t)));
            RegCloseKey(key);
        }
    } else {
        // Remove from startup
        if (RegOpenKeyExW(HKEY_CURRENT_USER, startup_path, 0, KEY_WRITE, &key) == ERROR_SUCCESS) {
            RegDeleteValueW(key, L"GoNhanh");
            RegCloseKey(key);
        }
    }
}

bool Settings::is_app_disabled(const std::wstring& app_name) const {
    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REG_PER_APP_PATH, 0, KEY_READ, &key) == ERROR_SUCCESS) {
        DWORD value = 0;
        DWORD size = sizeof(value);
        DWORD type = 0;
        LSTATUS status = RegQueryValueExW(key, app_name.c_str(), nullptr, &type,
                                          reinterpret_cast<BYTE*>(&value), &size);
        RegCloseKey(key);
        if (status == ERROR_SUCCESS) {
            return value == 0;  // 0 = disabled
        }
    }
    return false;  // Default: enabled
}

void Settings::set_app_disabled(const std::wstring& app_name, bool disabled) {
    HKEY key;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, REG_PER_APP_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &key, nullptr) == ERROR_SUCCESS) {
        if (disabled) {
            DWORD value = 0;
            RegSetValueExW(key, app_name.c_str(), 0, REG_DWORD,
                          reinterpret_cast<const BYTE*>(&value), sizeof(value));
        } else {
            RegDeleteValueW(key, app_name.c_str());
        }
        RegCloseKey(key);
    }
}

std::vector<std::wstring> Settings::get_disabled_apps() const {
    std::vector<std::wstring> apps;
    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REG_PER_APP_PATH, 0, KEY_READ, &key) == ERROR_SUCCESS) {
        DWORD index = 0;
        wchar_t name[256];
        DWORD name_len = 256;

        while (RegEnumValueW(key, index++, name, &name_len, nullptr, nullptr, nullptr, nullptr) == ERROR_SUCCESS) {
            apps.emplace_back(name);
            name_len = 256;  // Reset for next iteration
        }
        RegCloseKey(key);
    }
    return apps;
}

void Settings::remove_disabled_app(const std::wstring& app_name) {
    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REG_PER_APP_PATH, 0, KEY_WRITE, &key) == ERROR_SUCCESS) {
        RegDeleteValueW(key, app_name.c_str());
        RegCloseKey(key);
    }
}

} // namespace gonhanh
