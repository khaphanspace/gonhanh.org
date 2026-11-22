#include "cli_path.h"

#include <windows.h>

#include <thread>
#include <vector>

namespace gonhanh {

namespace {

constexpr wchar_t kEnvKey[] = L"Environment";
constexpr wchar_t kPathValue[] = L"Path";

// Normalized form of a PATH entry for comparison: env vars expanded, no trailing slash, lowercase
std::wstring Normalize(const std::wstring& entry) {
    std::wstring s = entry;
    if (s.find(L'%') != std::wstring::npos) {
        wchar_t buf[MAX_PATH * 2];
        DWORD n = ExpandEnvironmentStringsW(s.c_str(), buf, ARRAYSIZE(buf));
        if (n > 0 && n <= ARRAYSIZE(buf)) s = buf;
    }
    while (!s.empty() && (s.back() == L'\\' || s.back() == L'/')) s.pop_back();
    CharLowerBuffW(s.data(), static_cast<DWORD>(s.size()));
    return s;
}

// Every ';'-separated part, empty ones included, so joining them back gives the same string
std::vector<std::wstring> Split(const std::wstring& path) {
    std::vector<std::wstring> parts;
    size_t start = 0;
    for (;;) {
        size_t end = path.find(L';', start);
        if (end == std::wstring::npos) {
            parts.push_back(path.substr(start));
            return parts;
        }
        parts.push_back(path.substr(start, end - start));
        start = end + 1;
    }
}

bool FileExists(const std::wstring& path) {
    DWORD attr = GetFileAttributesW(path.c_str());
    return attr != INVALID_FILE_ATTRIBUTES && !(attr & FILE_ATTRIBUTE_DIRECTORY);
}

// A PATH entry holding a Gõ Nhanh CLI, ours or another copy's (an older install, a moved folder,
// a dev build): both gn.exe and gonhanh.exe, which Google's `gn` build tool doesn't have.
bool IsGoNhanhCliDir(const std::wstring& entry) {
    std::wstring dir = Normalize(entry);
    return !dir.empty() && FileExists(dir + L"\\gn.exe") && FileExists(dir + L"\\gonhanh.exe");
}

// PATH without `bin` (any spelling) or any other entry `isGoNhanh` accepts, then with `bin`
// appended when `on`. Every other entry is kept exactly as it was, empty ones included.
template <typename IsGoNhanh>
std::wstring RebuildPath(const std::wstring& path, const std::wstring& bin, bool on, IsGoNhanh isGoNhanh) {
    std::wstring target = Normalize(bin);
    std::wstring result;
    bool first = true;
    for (const auto& part : Split(path)) {
        if (!part.empty() && (Normalize(part) == target || isGoNhanh(part))) continue;
        if (!first) result += L';';
        result += part;
        first = false;
    }
    if (on) {
        if (!result.empty() && result.back() != L';') result += L';';
        result += bin;
    }
    return result;
}

// Reads HKCU\Environment\Path as stored (unexpanded). *type keeps REG_EXPAND_SZ vs REG_SZ.
bool ReadUserPath(std::wstring* value, DWORD* type) {
    value->clear();
    *type = REG_EXPAND_SZ;
    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, kEnvKey, 0, KEY_QUERY_VALUE, &key) != ERROR_SUCCESS) return true;
    DWORD bytes = 0;
    LSTATUS st = RegQueryValueExW(key, kPathValue, nullptr, type, nullptr, &bytes);
    if (st == ERROR_SUCCESS && bytes > 0) {
        std::wstring buf(bytes / sizeof(wchar_t) + 1, L'\0');
        st = RegQueryValueExW(key, kPathValue, nullptr, type, reinterpret_cast<BYTE*>(buf.data()), &bytes);
        if (st == ERROR_SUCCESS) {
            buf.resize(wcsnlen(buf.c_str(), buf.size()));
            *value = buf;
        }
    }
    RegCloseKey(key);
    if (*type != REG_SZ && *type != REG_EXPAND_SZ) *type = REG_EXPAND_SZ;
    return st == ERROR_SUCCESS || st == ERROR_FILE_NOT_FOUND;
}

}  // namespace

std::wstring CliBinDir() {
    wchar_t exe[MAX_PATH];
    DWORD n = GetModuleFileNameW(nullptr, exe, MAX_PATH);
    if (n == 0 || n >= MAX_PATH) return {};
    std::wstring dir(exe, n);
    size_t slash = dir.find_last_of(L'\\');
    if (slash == std::wstring::npos) return {};
    dir.resize(slash);
    dir += L"\\bin";
    return IsGoNhanhCliDir(dir) ? dir : std::wstring();
}

CliPathState GetCliPathState() {
    std::wstring bin = CliBinDir();
    if (bin.empty()) return CliPathState::Missing;
    std::wstring path;
    DWORD type;
    ReadUserPath(&path, &type);
    std::wstring target = Normalize(bin);
    bool other = false;
    for (const auto& part : Split(path)) {
        if (part.empty()) continue;
        if (Normalize(part) == target) return CliPathState::On;
        if (IsGoNhanhCliDir(part)) other = true;
    }
    return other ? CliPathState::OtherCopy : CliPathState::Off;
}

bool SetCliOnPath(bool on) {
    std::wstring bin = CliBinDir();
    if (bin.empty()) return false;

    std::wstring path;
    DWORD type;
    if (!ReadUserPath(&path, &type)) return false;

    // Off removes every Gõ Nhanh CLI folder; on also drops other copies so `gn` is this version
    std::wstring result = RebuildPath(path, bin, on, IsGoNhanhCliDir);
    if (result == path) return true;

    HKEY key;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, kEnvKey, 0, nullptr, 0, KEY_SET_VALUE, nullptr, &key, nullptr) !=
        ERROR_SUCCESS) {
        return false;
    }
    LSTATUS st = RegSetValueExW(key, kPathValue, 0, type, reinterpret_cast<const BYTE*>(result.c_str()),
                                static_cast<DWORD>((result.size() + 1) * sizeof(wchar_t)));
    RegCloseKey(key);
    if (st != ERROR_SUCCESS) return false;

    // Explorer (and so new terminals) reloads the environment on this broadcast. A hung top-level
    // window can delay it, so it runs off the UI thread.
    std::thread([] {
        SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, reinterpret_cast<LPARAM>(L"Environment"),
                            SMTO_ABORTIFHUNG, 2000, nullptr);
    }).detach();
    return true;
}

} // namespace gonhanh
