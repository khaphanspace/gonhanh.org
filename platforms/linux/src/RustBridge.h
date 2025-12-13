#ifndef GONHANH_RUST_BRIDGE_H
#define GONHANH_RUST_BRIDGE_H

#include <cstdint>
#include <string>
#include <vector>

// FFI Result structure - must match core/src/engine/mod.rs
// #[repr(C)]
// pub struct Result {
//     pub chars: [u32; 32],
//     pub action: u8,
//     pub backspace: u8,
//     pub count: u8,
//     pub _pad: u8,
// }
//
// Note: Rust #[repr(C)] uses C ABI layout, which matches C++ struct layout
// for this specific arrangement. The array (128 bytes) is followed by
// 4 bytes of u8 fields = 132 bytes total with no implicit padding needed.
struct ImeResult {
    uint32_t chars[32];  // 128 bytes
    uint8_t action;      // 1 byte
    uint8_t backspace;   // 1 byte
    uint8_t count;       // 1 byte
    uint8_t _pad;        // 1 byte (explicit padding to 4-byte boundary)
};

// Verify struct size matches Rust at compile time
static_assert(sizeof(ImeResult) == 132, "ImeResult size mismatch with Rust core");

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
