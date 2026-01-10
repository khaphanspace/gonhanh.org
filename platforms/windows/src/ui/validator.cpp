#include "validator.h"
#include "shortcuts_window.h"
#include <Windows.h>
#include <algorithm>
#include <cwctype>

using gonhanh::ui::ShortcutItem;

namespace gonhanh {

ValidationResult Validator::validate_shortcut_key(const std::wstring& key) {
    if (key.empty()) {
        return {false, L"Từ viết tắt không được để trống"};
    }

    if (key.length() > MAX_SHORTCUT_KEY_LENGTH) {
        return {false, L"Từ viết tắt quá dài (tối đa 32 ký tự)"};
    }

    // Check for whitespace
    if (std::any_of(key.begin(), key.end(), [](wchar_t c) { return std::iswspace(c); })) {
        return {false, L"Từ viết tắt không được chứa khoảng trắng"};
    }

    // Check for special characters that would conflict with file formats
    const std::wstring forbidden = L":;\t\n\r";
    if (key.find_first_of(forbidden) != std::wstring::npos) {
        return {false, L"Từ viết tắt chứa ký tự không hợp lệ"};
    }

    return {true, L""};
}

ValidationResult Validator::validate_shortcut_value(const std::wstring& value) {
    if (value.empty()) {
        return {false, L"Nội dung không được để trống"};
    }

    if (value.length() > MAX_SHORTCUT_VALUE_LENGTH) {
        return {false, L"Nội dung quá dài (tối đa 500 ký tự)"};
    }

    return {true, L""};
}

ValidationResult Validator::check_duplicate_key(
    const std::wstring& key,
    const std::vector<ShortcutItem>& existing,
    int exclude_index) {

    for (size_t i = 0; i < existing.size(); i++) {
        if (static_cast<int>(i) == exclude_index) continue;
        if (_wcsicmp(existing[i].key.c_str(), key.c_str()) == 0) {
            return {false, L"Từ viết tắt đã tồn tại"};
        }
    }
    return {true, L""};
}

ValidationResult Validator::validate_hotkey(uint32_t modifiers, uint32_t vk) {
    // Must have at least one modifier
    if (modifiers == 0) {
        return {false, L"Phím tắt phải có ít nhất một phím bổ trợ (Ctrl, Alt, Shift)"};
    }

    // Must have a valid virtual key
    if (vk == 0) {
        return {false, L"Phím tắt không hợp lệ"};
    }

    // Check if system reserved
    if (is_system_reserved_hotkey(modifiers, vk)) {
        return {false, L"Phím tắt này đã được hệ thống sử dụng"};
    }

    return {true, L""};
}

bool Validator::is_system_reserved_hotkey(uint32_t modifiers, uint32_t vk) {
    // Windows system shortcuts
    const bool win = (modifiers & MOD_WIN) != 0;
    const bool ctrl = (modifiers & MOD_CONTROL) != 0;
    const bool alt = (modifiers & MOD_ALT) != 0;
    const bool shift = (modifiers & MOD_SHIFT) != 0;

    // Win+* (Windows system) - all Win key combinations are reserved
    if (win) return true;

    // Ctrl+Alt+Del (secure attention sequence)
    if (ctrl && alt && vk == VK_DELETE) return true;

    // Alt+Tab, Alt+F4 (window management)
    if (alt && !ctrl && !shift) {
        if (vk == VK_TAB || vk == VK_F4) return true;
    }

    // Ctrl+Shift+Esc (Task Manager)
    if (ctrl && shift && vk == VK_ESCAPE) return true;

    // Print Screen
    if (vk == VK_SNAPSHOT) return true;

    // Ctrl+Escape (Start menu)
    if (ctrl && !alt && !shift && vk == VK_ESCAPE) return true;

    return false;
}

ValidationResult Validator::validate_text_length(
    const std::wstring& text,
    int max_length) {

    if (static_cast<int>(text.length()) > max_length) {
        wchar_t msg[100];
        swprintf_s(msg, 100, L"Văn bản quá dài (tối đa %d ký tự)", max_length);
        return {false, msg};
    }
    return {true, L""};
}

} // namespace gonhanh
