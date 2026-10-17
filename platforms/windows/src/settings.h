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
