#pragma once
#include <string>
#include <vector>
#include <cstdint>

namespace gonhanh {

struct Shortcut {
    std::wstring trigger;
    std::wstring replacement;
    bool enabled = true;
};

class Settings {
public:
    // Feature flags (match macOS)
    bool enabled = true;
    uint8_t method = 0;  // 0=Telex, 1=VNI
    bool skipWShortcut = false;
    bool bracketShortcut = true;
    bool escRestore = true;
    bool autoStart = false;
    bool perApp = false;
    bool autoRestore = false;
    bool sound = true;
    bool modernTone = false;
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

private:
    Settings() = default;
    static const wchar_t* REG_KEY;
    static const wchar_t* REG_RUN_KEY;
};

} // namespace gonhanh
