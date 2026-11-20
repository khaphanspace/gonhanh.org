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
        return;  // Use defaults
    }

    // A missing value keeps the field's default (declared in settings.h)
    auto readBool = [&](const wchar_t* name, bool& field) {
        DWORD value = 0;
        DWORD size = sizeof(value);
        if (RegQueryValueExW(key, name, nullptr, nullptr, reinterpret_cast<LPBYTE>(&value), &size) == ERROR_SUCCESS) {
            field = (value != 0);
        }
    };

    readBool(L"Enabled", enabled);
    DWORD methodValue = method;
    DWORD size = sizeof(methodValue);
    if (RegQueryValueExW(key, L"Method", nullptr, nullptr, reinterpret_cast<LPBYTE>(&methodValue), &size) == ERROR_SUCCESS) {
        method = methodValue == 1 ? 1 : 0;
    } else {
        // The former WPF app stored it as "InputMethod" (same 0 = Telex, 1 = VNI); the next
        // Save() writes "Method", so a VNI user keeps VNI after upgrading
        size = sizeof(methodValue);
        if (RegQueryValueExW(key, L"InputMethod", nullptr, nullptr, reinterpret_cast<LPBYTE>(&methodValue), &size) == ERROR_SUCCESS) {
            method = methodValue == 1 ? 1 : 0;
        }
    }
    readBool(L"SkipWShortcut", skipWShortcut);
    readBool(L"BracketShortcut", bracketShortcut);
    readBool(L"EscRestore", escRestore);
    readBool(L"AutoStart", autoStart);
    readBool(L"PerApp", perApp);
    readBool(L"AutoRestore", autoRestore);
    readBool(L"Sound", sound);
    readBool(L"ModernTone", modernTone);
    readBool(L"AutoCapitalize", autoCapitalize);
    readBool(L"FreeTone", freeTone);
    readBool(L"AllowForeignConsonants", allowForeignConsonants);
    readBool(L"CheckUpdates", checkUpdates);

    // Shortcuts: REG_MULTI_SZ of "trigger\0replacement\0" pairs. Enabled ones live in
    // "Shortcuts" (original format), disabled ones in "ShortcutsDisabled".
    auto readPairs = [&](const wchar_t* name, bool enabledFlag) {
        DWORD bufferSize = 0, type = 0;
        if (RegQueryValueExW(key, name, nullptr, &type, nullptr, &bufferSize) != ERROR_SUCCESS ||
            type != REG_MULTI_SZ || bufferSize <= sizeof(wchar_t)) {
            return;
        }
        std::vector<wchar_t> buffer(bufferSize / sizeof(wchar_t) + 1, L'\0');
        if (RegQueryValueExW(key, name, nullptr, nullptr, reinterpret_cast<LPBYTE>(buffer.data()), &bufferSize) != ERROR_SUCCESS) {
            return;
        }
        const wchar_t* ptr = buffer.data();
        const wchar_t* end = buffer.data() + buffer.size();
        while (ptr < end && *ptr) {
            std::wstring trigger = ptr;
            ptr += trigger.length() + 1;
            if (ptr >= end || !*ptr) break;
            std::wstring replacement = ptr;
            ptr += replacement.length() + 1;
            shortcuts.push_back({ trigger, replacement, enabledFlag });
        }
    };
    shortcuts.clear();
    readPairs(L"Shortcuts", true);
    readPairs(L"ShortcutsDisabled", false);

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
    writeDword(L"FreeTone", freeTone ? 1 : 0);
    writeDword(L"AllowForeignConsonants", allowForeignConsonants ? 1 : 0);
    writeDword(L"CheckUpdates", checkUpdates ? 1 : 0);

    // Save shortcuts as REG_MULTI_SZ: enabled and disabled in separate values, so turning a
    // shortcut off no longer deletes it (and older builds still read "Shortcuts" unchanged)
    auto writePairs = [&](const wchar_t* name, bool enabledFlag) {
        std::wstring multiSz;
        for (const auto& shortcut : shortcuts) {
            if (shortcut.enabled == enabledFlag) {
                multiSz += shortcut.trigger + L'\0';
                multiSz += shortcut.replacement + L'\0';
            }
        }
        multiSz += L'\0';
        RegSetValueExW(key, name, 0, REG_MULTI_SZ, reinterpret_cast<const BYTE*>(multiSz.c_str()),
                       static_cast<DWORD>((multiSz.length() + 1) * sizeof(wchar_t)));
    };
    writePairs(L"Shortcuts", true);
    writePairs(L"ShortcutsDisabled", false);
    RegCloseKey(key);

    // Handle auto-start
    HKEY runKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REG_RUN_KEY, 0, KEY_WRITE, &runKey) == ERROR_SUCCESS) {
        if (autoStart) {
            wchar_t exePath[MAX_PATH];
            if (GetModuleFileNameW(nullptr, exePath, MAX_PATH) > 0) {
                std::wstring quotedPath = L"\"" + std::wstring(exePath) + L"\"";
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
    ime_free_tone(freeTone);
    ime_allow_foreign_consonants(allowForeignConsonants);

    // Clear and re-add all shortcuts
    ime_clear_shortcuts();
    for (const auto& shortcut : shortcuts) {
        // A trigger longer than the engine's 8-bit backspace count would wrap and leave the
        // trigger on screen (only possible with entries saved by older builds)
        if (shortcut.enabled && CodePointCount(shortcut.trigger) <= kMaxEngineTrigger) {
            std::string trigger = Utf16ToUtf8(shortcut.trigger);
            std::string replacement = Utf16ToUtf8(shortcut.replacement);
            // Validate UTF-8 conversion succeeded
            if (!trigger.empty() && !replacement.empty()) {
                ime_add_shortcut(trigger.c_str(), replacement.c_str());
            }
        }
    }
}

std::pair<int, int> Settings::GetShortcutsCount() const {
    int enabled = 0;
    int total = static_cast<int>(shortcuts.size());
    for (const auto& shortcut : shortcuts) {
        if (shortcut.enabled) {
            enabled++;
        }
    }
    return {enabled, total};
}

} // namespace gonhanh
