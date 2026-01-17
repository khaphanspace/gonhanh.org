#pragma once
#include <cstdint>
#include <string>

extern "C" {
    struct ImeResult {
        uint32_t chars[256];  // Matches Rust MAX buffer size
        uint8_t action;
        uint8_t backspace;
        uint8_t count;
        uint8_t flags;  // Renamed from _pad to match Rust struct
    };

    void ime_init();
    ImeResult* ime_key(uint16_t keycode, bool caps, bool ctrl);
    ImeResult* ime_key_ext(uint16_t keycode, bool caps, bool ctrl, bool shift);
    void ime_method(uint8_t method);
    void ime_enabled(bool enabled);
    void ime_clear();
    void ime_clear_all();
    void ime_free(ImeResult* result);

    // Feature flags
    void ime_skip_w_shortcut(bool skip);
    void ime_bracket_shortcut(bool enabled);
    void ime_esc_restore(bool enabled);
    void ime_modern(bool modern);
    void ime_english_auto_restore(bool enabled);
    void ime_auto_capitalize(bool enabled);

    // Shortcuts
    void ime_add_shortcut(const char* trigger, const char* replacement);
    void ime_remove_shortcut(const char* trigger);
    void ime_clear_shortcuts();
}

// RAII wrapper
class ImeResultGuard {
    ImeResult* ptr_;
public:
    explicit ImeResultGuard(ImeResult* p) : ptr_(p) {}
    ~ImeResultGuard() { if (ptr_) ime_free(ptr_); }
    ImeResultGuard(const ImeResultGuard&) = delete;
    ImeResultGuard& operator=(const ImeResultGuard&) = delete;
    ImeResultGuard(ImeResultGuard&&) = delete;  // Prevent move
    ImeResultGuard& operator=(ImeResultGuard&&) = delete;
    ImeResult* get() const { return ptr_; }
    ImeResult* operator->() const { return ptr_; }
    explicit operator bool() const { return ptr_ != nullptr; }
};

namespace gonhanh {
    // UTF-32 to UTF-16 conversion
    std::wstring Utf32ToUtf16(const uint32_t* chars, uint8_t count);

    // UTF-16 to UTF-8 for FFI string params
    std::string Utf16ToUtf8(const std::wstring& wstr);
}
