#include "rust_bridge.h"
#include "logger.h"
#include <codecvt>
#include <locale>

namespace gonhanh {

const char* RustBridge::error_to_string(FfiError error) {
    switch (error) {
        case FfiError::SUCCESS: return "Success";
        case FfiError::DLL_NOT_LOADED: return "DLL not loaded";
        case FfiError::FUNCTION_NOT_FOUND: return "Function not found";
        case FfiError::INVALID_PARAMETER: return "Invalid parameter";
        case FfiError::ENGINE_ERROR: return "Engine error";
        case FfiError::UNKNOWN_ERROR: return "Unknown error";
        default: return "Unknown error code";
    }
}

void RustBridge::set_error(FfiError error, const char* message) {
    last_error_.error = error;
    last_error_.error_message = message;
    Logger::error("FFI Error: %s (code=%d)", message, static_cast<int>(error));
}

void RustBridge::clear_error() {
    last_error_.error = FfiError::SUCCESS;
    last_error_.error_message.clear();
}

RustBridge& RustBridge::instance() {
    static RustBridge instance;
    return instance;
}

RustBridge::~RustBridge() {
    shutdown();
}

bool RustBridge::initialize() {
    if (dll_) {
        clear_error();
        return true;
    }
    if (!load_dll()) {
        set_error(FfiError::DLL_NOT_LOADED, "Failed to load gonhanh_core.dll");
        return false;
    }
    load_functions();
    Logger::info("Rust bridge initialized successfully");
    clear_error();
    return true;
}

void RustBridge::shutdown() {
    if (dll_) {
        Logger::info("Rust bridge shutting down");
        FreeLibrary(dll_);
        dll_ = nullptr;
    }
    // Reset all function pointers
    fn_ime_init_ = nullptr;
    fn_ime_clear_ = nullptr;
    fn_ime_free_ = nullptr;
    fn_ime_method_ = nullptr;
    fn_ime_enabled_ = nullptr;
    fn_ime_modern_ = nullptr;
    fn_ime_skip_w_shortcut_ = nullptr;
    fn_ime_esc_restore_ = nullptr;
    fn_ime_english_auto_restore_ = nullptr;
    fn_ime_auto_capitalize_ = nullptr;
    fn_ime_bracket_shortcut_ = nullptr;
    fn_ime_key_ = nullptr;
    fn_ime_add_shortcut_ = nullptr;
    fn_ime_remove_shortcut_ = nullptr;
    fn_ime_clear_shortcuts_ = nullptr;
    fn_version_compare_ = nullptr;
    fn_version_has_update_ = nullptr;
    clear_error();
}

bool RustBridge::load_dll() {
    // Try loading from same directory as executable
    dll_ = LoadLibraryW(L"gonhanh_core.dll");
    if (dll_) return true;

    // Try loading from Native subdirectory
    dll_ = LoadLibraryW(L"Native\\gonhanh_core.dll");
    return dll_ != nullptr;
}

void RustBridge::load_functions() {
    if (!dll_) return;

    #define LOAD_FN(name) fn_##name##_ = reinterpret_cast<decltype(fn_##name##_)>(GetProcAddress(dll_, #name))

    LOAD_FN(ime_init);
    LOAD_FN(ime_clear);
    LOAD_FN(ime_free);
    LOAD_FN(ime_method);
    LOAD_FN(ime_enabled);
    LOAD_FN(ime_modern);
    LOAD_FN(ime_skip_w_shortcut);
    LOAD_FN(ime_esc_restore);
    LOAD_FN(ime_english_auto_restore);
    LOAD_FN(ime_auto_capitalize);
    LOAD_FN(ime_bracket_shortcut);
    LOAD_FN(ime_key);
    LOAD_FN(ime_add_shortcut);
    LOAD_FN(ime_remove_shortcut);
    LOAD_FN(ime_clear_shortcuts);
    LOAD_FN(version_compare);
    LOAD_FN(version_has_update);

    #undef LOAD_FN
}

void RustBridge::init() {
    if (!fn_ime_init_) {
        set_error(FfiError::FUNCTION_NOT_FOUND, "ime_init function not found");
        return;
    }
    fn_ime_init_();
    Logger::log_ffi_call("ime_init", true);
    clear_error();
}

void RustBridge::clear() {
    if (fn_ime_clear_) {
        fn_ime_clear_();
        clear_error();
    }
}

void RustBridge::set_method(InputMethod method) {
    if (fn_ime_method_) fn_ime_method_(static_cast<uint8_t>(method));
}

void RustBridge::set_enabled(bool enabled) {
    if (fn_ime_enabled_) fn_ime_enabled_(enabled);
}

void RustBridge::set_modern_tone(bool modern) {
    if (fn_ime_modern_) fn_ime_modern_(modern);
}

void RustBridge::set_skip_w_shortcut(bool skip) {
    if (fn_ime_skip_w_shortcut_) fn_ime_skip_w_shortcut_(skip);
}

void RustBridge::set_esc_restore(bool enabled) {
    if (fn_ime_esc_restore_) fn_ime_esc_restore_(enabled);
}

void RustBridge::set_english_auto_restore(bool enabled) {
    if (fn_ime_english_auto_restore_) fn_ime_english_auto_restore_(enabled);
}

void RustBridge::set_auto_capitalize(bool enabled) {
    if (fn_ime_auto_capitalize_) fn_ime_auto_capitalize_(enabled);
}

void RustBridge::set_bracket_shortcut(bool enabled) {
    if (fn_ime_bracket_shortcut_) fn_ime_bracket_shortcut_(enabled);
}

ImeResult RustBridge::process_key(uint16_t keycode, bool shift, bool capslock) {
    ImeResult result{ImeAction::None, 0, 0, L""};

    if (!dll_) {
        set_error(FfiError::DLL_NOT_LOADED, "gonhanh_core.dll not loaded");
        return result;
    }

    if (!fn_ime_key_) {
        set_error(FfiError::FUNCTION_NOT_FOUND, "ime_key function not found");
        return result;
    }

    if (!fn_ime_free_) {
        set_error(FfiError::FUNCTION_NOT_FOUND, "ime_free function not found");
        return result;
    }

    NativeResult* native = fn_ime_key_(keycode, shift, capslock);
    if (!native) {
        set_error(FfiError::ENGINE_ERROR, "ime_key returned null");
        return result;
    }

    result.action = static_cast<ImeAction>(native->action);
    result.backspace = native->backspace;
    result.count = native->count;

    // Convert UTF-32 codepoints to wstring
    if (native->count > 0) {
        result.text.reserve(native->count * 2);  // UTF-16 may need surrogates
        for (uint8_t i = 0; i < native->count && i < 256; ++i) {
            uint32_t cp = native->chars[i];
            if (cp == 0) break;

            if (cp <= 0xFFFF) {
                result.text += static_cast<wchar_t>(cp);
            } else {
                // Surrogate pair for codepoints > 0xFFFF
                cp -= 0x10000;
                result.text += static_cast<wchar_t>(0xD800 + (cp >> 10));
                result.text += static_cast<wchar_t>(0xDC00 + (cp & 0x3FF));
            }
        }
    }

    fn_ime_free_(native);
    clear_error();
    return result;
}

void RustBridge::add_shortcut(const std::string& trigger, const std::string& replacement) {
    if (fn_ime_add_shortcut_ && !trigger.empty() && !replacement.empty()) {
        fn_ime_add_shortcut_(trigger.c_str(), replacement.c_str());
    }
}

void RustBridge::remove_shortcut(const std::string& trigger) {
    if (fn_ime_remove_shortcut_ && !trigger.empty()) {
        fn_ime_remove_shortcut_(trigger.c_str());
    }
}

void RustBridge::clear_shortcuts() {
    if (fn_ime_clear_shortcuts_) fn_ime_clear_shortcuts_();
}

void RustBridge::sync_shortcuts(const std::vector<std::tuple<std::string, std::string, bool>>& shortcuts) {
    clear_shortcuts();
    for (const auto& [trigger, replacement, enabled] : shortcuts) {
        if (enabled) {
            add_shortcut(trigger, replacement);
        }
    }
}

int RustBridge::compare_versions(const std::string& v1, const std::string& v2) {
    if (fn_version_compare_ && !v1.empty() && !v2.empty()) {
        return fn_version_compare_(v1.c_str(), v2.c_str());
    }
    return -99;
}

bool RustBridge::has_update(const std::string& current, const std::string& latest) {
    if (fn_version_has_update_ && !current.empty() && !latest.empty()) {
        return fn_version_has_update_(current.c_str(), latest.c_str()) == 1;
    }
    return false;
}

} // namespace gonhanh
