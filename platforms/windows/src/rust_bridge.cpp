#include "rust_bridge.h"
#include <windows.h>

namespace gonhanh {

// UTF-32 to UTF-16 conversion
std::wstring Utf32ToUtf16(const uint32_t* chars, uint8_t count) {
    std::wstring result;
    result.reserve(count * 2);  // Worst case: all surrogate pairs

    for (uint8_t i = 0; i < count; ++i) {
        uint32_t cp = chars[i];
        if (cp <= 0xFFFF) {
            result.push_back(static_cast<wchar_t>(cp));
        } else {
            // Surrogate pair for codepoints > 0xFFFF
            cp -= 0x10000;
            result.push_back(static_cast<wchar_t>(0xD800 + (cp >> 10)));
            result.push_back(static_cast<wchar_t>(0xDC00 + (cp & 0x3FF)));
        }
    }
    return result;
}

// UTF-16 to UTF-8 for FFI string params
std::string Utf16ToUtf8(const std::wstring& wstr) {
    if (wstr.empty()) return {};
    int size = WideCharToMultiByte(CP_UTF8, 0, wstr.c_str(), -1, nullptr, 0, nullptr, nullptr);
    std::string result(size - 1, 0);
    WideCharToMultiByte(CP_UTF8, 0, wstr.c_str(), -1, result.data(), size, nullptr, nullptr);
    return result;
}

} // namespace gonhanh
