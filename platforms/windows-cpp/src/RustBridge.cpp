#include "RustBridge.h"
#include <Windows.h>

// FFI declarations - extern "C" functions from libgonhanh_core.dll
extern "C" {
    // Lifecycle
    void ime_init();

    // Keystroke processing
    ImeResult* ime_key(uint16_t key, bool caps, bool ctrl);
    ImeResult* ime_key_ext(uint16_t key, bool caps, bool ctrl, bool shift);
    void ime_free(ImeResult* result);

    // Buffer management
    void ime_clear();
    void ime_clear_all();
    int64_t ime_get_buffer(uint32_t* out, int64_t maxLen);

    // Configuration
    void ime_method(uint8_t method);
    void ime_enabled(bool enabled);
    void ime_skip_w_shortcut(bool skip);
    void ime_bracket_shortcut(bool enabled);
    void ime_esc_restore(bool enabled);
    void ime_free_tone(bool enabled);
    void ime_modern(bool modern);
    void ime_english_auto_restore(bool enabled);
    void ime_auto_capitalize(bool enabled);

    // Shortcuts
    void ime_add_shortcut(const char* trigger, const char* replacement);
    void ime_remove_shortcut(const char* trigger);
    void ime_clear_shortcuts();

    // Word restore
    void ime_restore_word(const char* word);
}

// Lifecycle

void RustBridge::Init() {
    ime_init();
}

// Keystroke processing

ImeResult* RustBridge::ProcessKey(uint16_t key, bool caps, bool ctrl) {
    return ime_key(key, caps, ctrl);
}

ImeResult* RustBridge::ProcessKeyExt(uint16_t key, bool caps, bool ctrl, bool shift) {
    return ime_key_ext(key, caps, ctrl, shift);
}

void RustBridge::Free(ImeResult* result) {
    if (result) {
        ime_free(result);
    }
}

// Buffer management

void RustBridge::Clear() {
    ime_clear();
}

void RustBridge::ClearAll() {
    ime_clear_all();
}

int64_t RustBridge::GetBuffer(uint32_t* out, int64_t maxLen) {
    return ime_get_buffer(out, maxLen);
}

// Configuration - Input method

void RustBridge::SetMethod(uint8_t method) {
    ime_method(method);
}

void RustBridge::SetEnabled(bool enabled) {
    ime_enabled(enabled);
}

// Configuration - Features

void RustBridge::SetSkipWShortcut(bool skip) {
    ime_skip_w_shortcut(skip);
}

void RustBridge::SetBracketShortcut(bool enabled) {
    ime_bracket_shortcut(enabled);
}

void RustBridge::SetEscRestore(bool enabled) {
    ime_esc_restore(enabled);
}

void RustBridge::SetModern(bool modern) {
    ime_modern(modern);
}

void RustBridge::SetEnglishAutoRestore(bool enabled) {
    ime_english_auto_restore(enabled);
}

void RustBridge::SetAutoCapitalize(bool enabled) {
    ime_auto_capitalize(enabled);
}

// Shortcuts management

void RustBridge::AddShortcut(const char* trigger, const char* replacement) {
    ime_add_shortcut(trigger, replacement);
}

void RustBridge::RemoveShortcut(const char* trigger) {
    ime_remove_shortcut(trigger);
}

void RustBridge::ClearShortcuts() {
    ime_clear_shortcuts();
}

// Word restore

void RustBridge::RestoreWord(const char* word) {
    ime_restore_word(word);
}
