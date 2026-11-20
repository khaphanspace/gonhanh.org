#pragma once
#include <string>
#include <vector>
#include <cstdint>
#include <utility>

namespace gonhanh {

struct Shortcut {
    std::wstring trigger;
    std::wstring replacement;
    bool enabled = true;
};

// Shortcut length limits, in code points. The engine erases the trigger with an 8-bit
// backspace count and returns at most 255 characters (core MAX_REPLACEMENT_LEN), so longer
// text would leave the trigger on screen or be cut short. The editor allows 64 for triggers.
constexpr size_t kMaxShortcutTrigger = 64;
constexpr size_t kMaxEngineTrigger = 255;
constexpr size_t kMaxShortcutReplacement = 255;

// Number of code points (a surrogate pair counts once)
inline size_t CodePointCount(const std::wstring& s) {
    size_t n = 0;
    for (wchar_t c : s) {
        if (c < 0xDC00 || c > 0xDFFF) n++;
    }
    return n;
}

class Settings {
public:
    // Feature flags (match macOS defaults)
    bool enabled = true;
    uint8_t method = 0;  // 0=Telex, 1=VNI
    bool skipWShortcut = false;
    bool bracketShortcut = false;
    bool escRestore = false;
    bool autoStart = false;
    bool perApp = true;
    bool autoRestore = true;
    bool sound = false;
    bool modernTone = true;
    bool autoCapitalize = false;
    bool freeTone = false;
    bool allowForeignConsonants = false;
    bool checkUpdates = false;  // Off by default: the app makes no network requests unless enabled

    // Shortcuts list
    std::vector<Shortcut> shortcuts;

    // Methods
    void Load();                    // Load from Registry
    void Save();                    // Save to Registry
    void ApplyToEngine();           // Apply to Rust core via FFI
    static Settings& Instance();    // Singleton

    // Helper: Get count of (enabled, total) shortcuts
    std::pair<int, int> GetShortcutsCount() const;

private:
    Settings() = default;
    static const wchar_t* REG_KEY;
    static const wchar_t* REG_RUN_KEY;
};

} // namespace gonhanh
