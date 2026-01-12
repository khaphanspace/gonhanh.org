#pragma once
#include "Types.h"
#include <cstdint>

// C++ wrapper for Rust FFI functions
// All methods are static - this is a utility class, not instantiable
class RustBridge {
public:
    // Delete constructors to prevent instantiation
    RustBridge() = delete;
    RustBridge(const RustBridge&) = delete;
    RustBridge& operator=(const RustBridge&) = delete;

    // Lifecycle
    static void Init();

    // Keystroke processing
    static ImeResult* ProcessKey(uint16_t key, bool caps, bool ctrl);
    static ImeResult* ProcessKeyExt(uint16_t key, bool caps, bool ctrl, bool shift);
    static void Free(ImeResult* result);

    // Buffer management
    static void Clear();
    static void ClearAll();
    static int64_t GetBuffer(uint32_t* out, int64_t maxLen);

    // Configuration - Input method
    static void SetMethod(uint8_t method);  // 0=Telex, 1=VNI
    static void SetEnabled(bool enabled);

    // Configuration - Features
    static void SetSkipWShortcut(bool skip);
    static void SetBracketShortcut(bool enabled);
    static void SetEscRestore(bool enabled);
    static void SetModern(bool modern);
    static void SetEnglishAutoRestore(bool enabled);
    static void SetAutoCapitalize(bool enabled);

    // Shortcuts management
    static void AddShortcut(const char* trigger, const char* replacement);
    static void RemoveShortcut(const char* trigger);
    static void ClearShortcuts();

    // Word restore (for backspace into previous word)
    static void RestoreWord(const char* word);
};
