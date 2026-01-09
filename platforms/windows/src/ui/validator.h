#pragma once

#include <string>
#include <vector>
#include <cstdint>

namespace gonhanh {

struct ValidationResult {
    bool valid;
    std::wstring error_message;

    operator bool() const { return valid; }
};

class Validator {
public:
    // Shortcut validation
    static ValidationResult validate_shortcut_key(const std::wstring& key);
    static ValidationResult validate_shortcut_value(const std::wstring& value);
    static ValidationResult check_duplicate_key(
        const std::wstring& key,
        const std::vector<std::pair<std::wstring, std::wstring>>& existing,
        int exclude_index = -1);

    // Hotkey validation
    static ValidationResult validate_hotkey(uint32_t modifiers, uint32_t vk);
    static bool is_system_reserved_hotkey(uint32_t modifiers, uint32_t vk);

    // Text input validation
    static ValidationResult validate_text_length(
        const std::wstring& text,
        int max_length);

    // Constants
    static const int MAX_SHORTCUT_KEY_LENGTH = 32;
    static const int MAX_SHORTCUT_VALUE_LENGTH = 500;
    static const int MAX_TEXT_INPUT_LENGTH = 1000;
};

} // namespace gonhanh
