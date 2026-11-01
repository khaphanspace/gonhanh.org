#ifndef GONHANH_RUST_BRIDGE_H
#define GONHANH_RUST_BRIDGE_H

#include <cstddef>
#include <cstdint>
#include <string>
#include <utility>

// Keep this value synchronized with core/src/engine/buffer.rs::MAX. The
// layout assertions below and the real FFI tests fail if the C++ view drifts.
constexpr std::size_t kImeResultCapacity = 256;

// FFI Result structure - must match core/src/engine/mod.rs
// #[repr(C)]
// pub struct Result {
//     pub chars: [u32; 256],
//     pub action: u8,
//     pub backspace: u8,
//     pub count: u8,
//     pub flags: u8,
// }
struct ImeResult {
    uint32_t chars[kImeResultCapacity];
    uint8_t action;      // 1 byte
    uint8_t backspace;   // 1 byte
    uint8_t count;       // 1 byte
    uint8_t flags;       // bit 0: trigger key was consumed
};

constexpr std::size_t kImeResultMetadataOffset =
    kImeResultCapacity * sizeof(uint32_t);
static_assert(offsetof(ImeResult, action) == kImeResultMetadataOffset,
              "ImeResult action offset must match Rust #[repr(C)] Result");
static_assert(offsetof(ImeResult, backspace) == kImeResultMetadataOffset + 1,
              "ImeResult backspace offset must match Rust Result");
static_assert(offsetof(ImeResult, count) == kImeResultMetadataOffset + 2,
              "ImeResult count offset must match Rust Result");
static_assert(offsetof(ImeResult, flags) == kImeResultMetadataOffset + 3,
              "ImeResult flags offset must match Rust Result");
static_assert(sizeof(ImeResult) == kImeResultMetadataOffset + 4,
              "ImeResult size must match Rust Result");

// Action types
enum class ImeAction : uint8_t {
    None = 0,    // Pass through
    Send = 1,    // Replace text
    Restore = 2  // Restore original
};

// Input method types
enum class InputMethod : uint8_t {
    Telex = 0,
    VNI = 1
};

// FFI function declarations (from core/src/lib.rs)
extern "C" {
    void ime_init();
    ImeResult* ime_key_ext(uint16_t key, bool caps, bool ctrl, bool shift);
    void ime_method(uint8_t method);
    void ime_enabled(bool enabled);
    void ime_clear();
    void ime_free(ImeResult* result);
    void ime_add_shortcut(const char* trigger, const char* replacement);
    void ime_clear_shortcuts();
}

// C++ wrapper class for Rust bridge
class RustBridge {
public:
    // Initialize the IME engine (call once at startup)
    static void initialize();

    // Process a keystroke and return result
    // Returns: (backspace_count, output_text) or empty if no action needed
    static std::pair<int, std::string> processKey(
        uint16_t keyCode,
        bool caps,
        bool ctrl,
        bool shift
    );

    // Set input method (Telex=0, VNI=1)
    static void setMethod(InputMethod method);

    // Enable or disable IME processing
    static void setEnabled(bool enabled);

    // Clear the input buffer (call on word boundaries)
    static void clear();

    // Convert UTF-32 codepoint to UTF-8 string (public for testing)
    static std::string codePointToUtf8(uint32_t cp);

private:
    static bool initialized_;
};

#endif // GONHANH_RUST_BRIDGE_H
