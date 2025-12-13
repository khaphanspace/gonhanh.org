#include "RustBridge.h"
#include <codecvt>
#include <locale>

bool RustBridge::initialized_ = false;

void RustBridge::initialize() {
    if (initialized_) return;
    ime_init();
    initialized_ = true;
}

std::pair<int, std::string> RustBridge::processKey(
    uint16_t keyCode,
    bool caps,
    bool ctrl,
    bool shift
) {
    if (!initialized_) {
        initialize();
    }

    ImeResult* result = ime_key_ext(keyCode, caps, ctrl, shift);
    if (!result) {
        return {0, ""};
    }

    std::pair<int, std::string> output = {0, ""};

    if (result->action == static_cast<uint8_t>(ImeAction::Send)) {
        output.first = result->backspace;

        // Convert UTF-32 chars to UTF-8 string
        for (uint8_t i = 0; i < result->count && i < 32; ++i) {
            if (result->chars[i] > 0) {
                output.second += codePointToUtf8(result->chars[i]);
            }
        }
    }

    ime_free(result);
    return output;
}

void RustBridge::setMethod(InputMethod method) {
    ime_method(static_cast<uint8_t>(method));
}

void RustBridge::setEnabled(bool enabled) {
    ime_enabled(enabled);
}

void RustBridge::clear() {
    ime_clear();
}

std::string RustBridge::codePointToUtf8(uint32_t cp) {
    std::string result;

    // Check for invalid codepoints (out of range or surrogates)
    if (cp >= 0x110000 || (cp >= 0xD800 && cp <= 0xDFFF)) {
        // Return replacement character U+FFFD for invalid codepoints
        return "\xEF\xBF\xBD";
    }

    if (cp < 0x80) {
        // 1-byte UTF-8
        result += static_cast<char>(cp);
    } else if (cp < 0x800) {
        // 2-byte UTF-8
        result += static_cast<char>(0xC0 | (cp >> 6));
        result += static_cast<char>(0x80 | (cp & 0x3F));
    } else if (cp < 0x10000) {
        // 3-byte UTF-8
        result += static_cast<char>(0xE0 | (cp >> 12));
        result += static_cast<char>(0x80 | ((cp >> 6) & 0x3F));
        result += static_cast<char>(0x80 | (cp & 0x3F));
    } else {
        // 4-byte UTF-8
        result += static_cast<char>(0xF0 | (cp >> 18));
        result += static_cast<char>(0x80 | ((cp >> 12) & 0x3F));
        result += static_cast<char>(0x80 | ((cp >> 6) & 0x3F));
        result += static_cast<char>(0x80 | (cp & 0x3F));
    }

    return result;
}
