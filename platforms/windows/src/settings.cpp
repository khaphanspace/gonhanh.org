#include "settings.h"
#include "rust_bridge.h"
#include <windows.h>

namespace gonhanh {

const wchar_t* Settings::REG_KEY = L"Software\\GoNhanh";
const wchar_t* Settings::REG_RUN_KEY = L"Software\\Microsoft\\Windows\\CurrentVersion\\Run";

Settings& Settings::Instance() {
    static Settings instance;
    return instance;
}

void Settings::Load() {
    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REG_KEY, 0, KEY_READ, &key) != ERROR_SUCCESS) {
        return;  // Use defaults if key doesn't exist
    }

    auto readDword = [&](const wchar_t* name, DWORD& value) {
        DWORD size = sizeof(DWORD);
        RegQueryValueExW(key, name, nullptr, nullptr, (LPBYTE)&value, &size);
    };

    DWORD temp;
    readDword(L"Enabled", temp);
    enabled = (temp != 0);

    readDword(L"Method", temp);
    method = static_cast<uint8_t>(temp);

    readDword(L"SkipWShortcut", temp);
    skipWShortcut = (temp != 0);

    readDword(L"BracketShortcut", temp);
    bracketShortcut = (temp != 0);

    readDword(L"EscRestore", temp);
    escRestore = (temp != 0);

    readDword(L"AutoStart", temp);
    autoStart = (temp != 0);

    readDword(L"PerApp", temp);
    perApp = (temp != 0);

    readDword(L"AutoRestore", temp);
    autoRestore = (temp != 0);

    readDword(L"Sound", temp);
    sound = (temp != 0);

    readDword(L"ModernTone", temp);
    modernTone = (temp != 0);

    readDword(L"AutoCapitalize", temp);
    autoCapitalize = (temp != 0);

    // Load shortcuts from REG_MULTI_SZ (dynamic allocation)
    DWORD bufferSize = 0;
    DWORD type;
    // Query size first
    if (RegQueryValueExW(key, L"Shortcuts", nullptr, &type, nullptr, &bufferSize) == ERROR_SUCCESS
        && type == REG_MULTI_SZ && bufferSize > 0) {
        // Allocate buffer dynamically
        std::vector<wchar_t> buffer(bufferSize / sizeof(wchar_t));
        if (RegQueryValueExW(key, L"Shortcuts", nullptr, &type, (LPBYTE)buffer.data(), &bufferSize) == ERROR_SUCCESS) {
            shortcuts.clear();
            const wchar_t* ptr = buffer.data();
            while (*ptr) {
                std::wstring trigger = ptr;
                ptr += trigger.length() + 1;
                if (*ptr) {
                    std::wstring replacement = ptr;
                    ptr += replacement.length() + 1;
                    shortcuts.push_back({trigger, replacement, true});
                } else {
                    break;
                }
            }
        }
    }

    RegCloseKey(key);
}

void Settings::Save() {
    HKEY key;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, REG_KEY, 0, nullptr, 0,
                        KEY_WRITE, nullptr, &key, nullptr) != ERROR_SUCCESS) {
        return;
    }

    auto writeDword = [&](const wchar_t* name, DWORD value) {
        RegSetValueExW(key, name, 0, REG_DWORD, (const BYTE*)&value, sizeof(DWORD));
    };

    writeDword(L"Enabled", enabled ? 1 : 0);
    writeDword(L"Method", method);
    writeDword(L"SkipWShortcut", skipWShortcut ? 1 : 0);
    writeDword(L"BracketShortcut", bracketShortcut ? 1 : 0);
    writeDword(L"EscRestore", escRestore ? 1 : 0);
    writeDword(L"AutoStart", autoStart ? 1 : 0);
    writeDword(L"PerApp", perApp ? 1 : 0);
    writeDword(L"AutoRestore", autoRestore ? 1 : 0);
    writeDword(L"Sound", sound ? 1 : 0);
    writeDword(L"ModernTone", modernTone ? 1 : 0);
    writeDword(L"AutoCapitalize", autoCapitalize ? 1 : 0);

    // Save shortcuts as REG_MULTI_SZ
    std::wstring multiSz;
    for (const auto& shortcut : shortcuts) {
        if (shortcut.enabled) {
            multiSz += shortcut.trigger + L'\0';
            multiSz += shortcut.replacement + L'\0';
        }
    }
    multiSz += L'\0';  // Double null terminator

    RegSetValueExW(key, L"Shortcuts", 0, REG_MULTI_SZ,
                   (const BYTE*)multiSz.c_str(),
                   static_cast<DWORD>((multiSz.length() + 1) * sizeof(wchar_t)));

    RegCloseKey(key);

    // Handle auto-start Registry key
    HKEY runKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REG_RUN_KEY, 0, KEY_WRITE, &runKey) == ERROR_SUCCESS) {
        if (autoStart) {
            wchar_t exePath[MAX_PATH];
            DWORD len = GetModuleFileNameW(nullptr, exePath, MAX_PATH);
            if (len > 0 && len < MAX_PATH) {
                // Quote path to prevent injection
                std::wstring quotedPath = L"\"";
                quotedPath += exePath;
                quotedPath += L"\"";
                RegSetValueExW(runKey, L"GoNhanh", 0, REG_SZ,
                              (const BYTE*)quotedPath.c_str(),
                              static_cast<DWORD>((quotedPath.length() + 1) * sizeof(wchar_t)));
            }
        } else {
            RegDeleteValueW(runKey, L"GoNhanh");
        }
        RegCloseKey(runKey);
    }

    // Apply to engine
    ApplyToEngine();
}

void Settings::ApplyToEngine() {
    ime_enabled(enabled);
    ime_method(method);
    ime_skip_w_shortcut(skipWShortcut);
    ime_bracket_shortcut(bracketShortcut);
    ime_esc_restore(escRestore);
    ime_modern(modernTone);
    ime_english_auto_restore(autoRestore);
    ime_auto_capitalize(autoCapitalize);

    // Clear and re-add all shortcuts
    ime_clear_shortcuts();
    for (const auto& shortcut : shortcuts) {
        if (shortcut.enabled) {
            std::string trigger = Utf16ToUtf8(shortcut.trigger);
            std::string replacement = Utf16ToUtf8(shortcut.replacement);
            // Validate UTF-8 conversion succeeded
            if (!trigger.empty() && !replacement.empty()) {
                ime_add_shortcut(trigger.c_str(), replacement.c_str());
            }
        }
    }
}

} // namespace gonhanh
