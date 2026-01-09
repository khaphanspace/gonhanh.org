#pragma once

#include <windows.h>
#include <string>
#include <vector>
#include <tuple>

namespace gonhanh {

// Settings storage using Windows Registry
// Path: HKEY_CURRENT_USER\SOFTWARE\GoNhanh
class Settings {
public:
    static Settings& instance();

    // Load/save all settings
    void load();
    void save();

    // Individual settings (getters/setters)
    bool is_enabled() const { return enabled_; }
    void set_enabled(bool v) { enabled_ = v; save_value(L"Enabled", v); }

    int input_method() const { return input_method_; }
    void set_input_method(int v) { input_method_ = v; save_value(L"InputMethod", v); }

    bool modern_tone() const { return modern_tone_; }
    void set_modern_tone(bool v) { modern_tone_ = v; save_value(L"ModernTone", v); }

    bool auto_start() const { return auto_start_; }
    void set_auto_start(bool v);

    bool w_shortcut() const { return w_shortcut_; }
    void set_w_shortcut(bool v) { w_shortcut_ = v; save_value(L"AutoWShortcut", v); }

    bool esc_restore() const { return esc_restore_; }
    void set_esc_restore(bool v) { esc_restore_ = v; save_value(L"EscRestore", v); }

    bool english_auto_restore() const { return english_auto_restore_; }
    void set_english_auto_restore(bool v) { english_auto_restore_ = v; save_value(L"EnglishAutoRestore", v); }

    bool auto_capitalize() const { return auto_capitalize_; }
    void set_auto_capitalize(bool v) { auto_capitalize_ = v; save_value(L"AutoCapitalize", v); }

    bool sound_enabled() const { return sound_enabled_; }
    void set_sound_enabled(bool v) { sound_enabled_ = v; save_value(L"SoundEnabled", v); }

    bool per_app_mode() const { return per_app_mode_; }
    void set_per_app_mode(bool v) { per_app_mode_ = v; save_value(L"PerAppModeEnabled", v); }

    bool bracket_shortcut() const { return bracket_shortcut_; }
    void set_bracket_shortcut(bool v) { bracket_shortcut_ = v; save_value(L"BracketShortcut", v); }

    bool first_run() const { return first_run_; }
    void set_first_run(bool v) { first_run_ = v; save_value(L"FirstRun", v); }

    bool onboarding_completed() const { return onboarding_completed_; }
    void set_onboarding_completed(bool v) { onboarding_completed_ = v; save_value(L"OnboardingCompleted", v); }

    // Toggle shortcut (modifier + key)
    uint32_t toggle_shortcut() const { return toggle_shortcut_; }
    void set_toggle_shortcut(uint32_t v) { toggle_shortcut_ = v; save_value(L"ToggleShortcut", static_cast<int>(v)); }

    // Per-app mode states
    bool is_app_disabled(const std::wstring& app_name) const;
    void set_app_disabled(const std::wstring& app_name, bool disabled);

    // Registry paths
    static constexpr const wchar_t* REG_PATH = L"SOFTWARE\\GoNhanh";
    static constexpr const wchar_t* REG_PER_APP_PATH = L"SOFTWARE\\GoNhanh\\PerAppModes";

private:
    Settings() { load(); }
    ~Settings() = default;
    Settings(const Settings&) = delete;
    Settings& operator=(const Settings&) = delete;

    void save_value(const wchar_t* name, bool value);
    void save_value(const wchar_t* name, int value);
    bool load_bool(const wchar_t* name, bool default_val);
    int load_int(const wchar_t* name, int default_val);

    void update_auto_start_registry();

    // Settings values
    bool enabled_ = true;
    int input_method_ = 0;  // 0=Telex, 1=VNI
    bool modern_tone_ = true;
    bool auto_start_ = false;
    bool w_shortcut_ = true;
    bool esc_restore_ = false;
    bool english_auto_restore_ = false;
    bool auto_capitalize_ = false;
    bool sound_enabled_ = true;
    bool per_app_mode_ = false;
    bool bracket_shortcut_ = false;
    bool first_run_ = true;
    bool onboarding_completed_ = false;
    uint32_t toggle_shortcut_ = 0;  // Encoded as (modifiers << 16) | vk
};

} // namespace gonhanh
