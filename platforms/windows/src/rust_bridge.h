#pragma once

#include <windows.h>
#include <string>
#include <vector>
#include <cstdint>

namespace gonhanh {

// FFI error codes
enum class FfiError {
    SUCCESS = 0,
    DLL_NOT_LOADED,
    FUNCTION_NOT_FOUND,
    INVALID_PARAMETER,
    ENGINE_ERROR,
    UNKNOWN_ERROR
};

// FFI result with error information
struct FfiResult {
    FfiError error;
    std::string error_message;

    bool ok() const { return error == FfiError::SUCCESS; }
    operator bool() const { return ok(); }
};

// Input method types
enum class InputMethod : uint8_t {
    Telex = 0,
    VNI = 1
};

// IME action types (matches Rust core)
enum class ImeAction : uint8_t {
    None = 0,
    Send = 1,
    Restore = 2
};

// IME result from Rust core
struct ImeResult {
    ImeAction action;
    uint8_t backspace;
    uint8_t count;
    std::wstring text;

    bool has_action() const { return action != ImeAction::None; }
};

// FFI bridge to Rust core (gonhanh_core.dll)
class RustBridge {
public:
    static RustBridge& instance();

    // Initialize - call once at startup
    bool initialize();
    void shutdown();
    bool is_loaded() const { return dll_ != nullptr; }

    // Error handling
    FfiResult get_last_error() const { return last_error_; }
    static const char* error_to_string(FfiError error);

    // Core IME functions
    void init();
    void clear();
    void set_method(InputMethod method);
    void set_enabled(bool enabled);
    void set_modern_tone(bool modern);
    void set_skip_w_shortcut(bool skip);
    void set_esc_restore(bool enabled);
    void set_english_auto_restore(bool enabled);
    void set_auto_capitalize(bool enabled);
    void set_bracket_shortcut(bool enabled);

    // Process keystroke
    ImeResult process_key(uint16_t keycode, bool shift, bool capslock);

    // Text abbreviations
    void add_shortcut(const std::string& trigger, const std::string& replacement);
    void remove_shortcut(const std::string& trigger);
    void clear_shortcuts();
    void sync_shortcuts(const std::vector<std::tuple<std::string, std::string, bool>>& shortcuts);

    // Version utilities
    int compare_versions(const std::string& v1, const std::string& v2);
    bool has_update(const std::string& current, const std::string& latest);

private:
    RustBridge() = default;
    ~RustBridge();
    RustBridge(const RustBridge&) = delete;
    RustBridge& operator=(const RustBridge&) = delete;

    bool load_dll();
    void load_functions();
    void set_error(FfiError error, const char* message);
    void clear_error();

    FfiResult last_error_ = {FfiError::SUCCESS, ""};

    HMODULE dll_ = nullptr;

    // Native result structure (must match Rust core/src/lib.rs)
    #pragma pack(push, 1)
    struct NativeResult {
        uint32_t chars[256];
        uint8_t action;
        uint8_t backspace;
        uint8_t count;
        uint8_t _pad;
    };
    #pragma pack(pop)

    // Function pointers
    void (*fn_ime_init_)() = nullptr;
    void (*fn_ime_clear_)() = nullptr;
    void (*fn_ime_free_)(void*) = nullptr;
    void (*fn_ime_method_)(uint8_t) = nullptr;
    void (*fn_ime_enabled_)(bool) = nullptr;
    void (*fn_ime_modern_)(bool) = nullptr;
    void (*fn_ime_skip_w_shortcut_)(bool) = nullptr;
    void (*fn_ime_esc_restore_)(bool) = nullptr;
    void (*fn_ime_english_auto_restore_)(bool) = nullptr;
    void (*fn_ime_auto_capitalize_)(bool) = nullptr;
    void (*fn_ime_bracket_shortcut_)(bool) = nullptr;
    NativeResult* (*fn_ime_key_)(uint16_t, bool, bool) = nullptr;
    void (*fn_ime_add_shortcut_)(const char*, const char*) = nullptr;
    void (*fn_ime_remove_shortcut_)(const char*) = nullptr;
    void (*fn_ime_clear_shortcuts_)() = nullptr;
    int (*fn_version_compare_)(const char*, const char*) = nullptr;
    int (*fn_version_has_update_)(const char*, const char*) = nullptr;
};

} // namespace gonhanh
