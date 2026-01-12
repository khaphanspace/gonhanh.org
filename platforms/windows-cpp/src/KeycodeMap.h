#pragma once
#include <cstdint>

// Map Windows VK codes to macOS keycodes
// Reference: core/src/data/keys.rs
// Windows implementation uses this mapping because Rust engine expects macOS keycodes

inline uint16_t VKToMacKeycode(uint16_t vk) {
    switch (vk) {
        // Letters (QWERTY layout)
        case 'A': return 0x00;
        case 'S': return 0x01;
        case 'D': return 0x02;
        case 'F': return 0x03;
        case 'H': return 0x04;
        case 'G': return 0x05;
        case 'Z': return 0x06;
        case 'X': return 0x07;
        case 'C': return 0x08;
        case 'V': return 0x09;
        case 'B': return 0x0B;
        case 'Q': return 0x0C;
        case 'W': return 0x0D;
        case 'E': return 0x0E;
        case 'R': return 0x0F;
        case 'Y': return 0x10;
        case 'T': return 0x11;
        case 'O': return 0x1F;
        case 'U': return 0x20;
        case 'I': return 0x22;
        case 'P': return 0x23;
        case 'L': return 0x25;
        case 'J': return 0x26;
        case 'K': return 0x28;
        case 'N': return 0x2D;
        case 'M': return 0x2E;

        // Numbers
        case '1': return 0x12;
        case '2': return 0x13;
        case '3': return 0x14;
        case '4': return 0x15;
        case '5': return 0x17;
        case '6': return 0x16;
        case '7': return 0x1A;
        case '8': return 0x1C;
        case '9': return 0x19;
        case '0': return 0x1D;

        // Special keys
        case VK_SPACE:  return 0x31;  // 49
        case VK_BACK:   return 0x33;  // 51 (Delete in macOS)
        case VK_TAB:    return 0x30;  // 48
        case VK_RETURN: return 0x24;  // 36
        case VK_ESCAPE: return 0x35;  // 53

        // Arrow keys
        case VK_LEFT:  return 0x7B;   // 123
        case VK_RIGHT: return 0x7C;   // 124
        case VK_DOWN:  return 0x7D;   // 125
        case VK_UP:    return 0x7E;   // 126

        // Punctuation
        case VK_OEM_PERIOD:    return 0x2F;  // 47 (.)
        case VK_OEM_COMMA:     return 0x2B;  // 43 (,)
        case VK_OEM_2:         return 0x2C;  // 44 (/)
        case VK_OEM_1:         return 0x29;  // 41 (;)
        case VK_OEM_7:         return 0x27;  // 39 (')
        case VK_OEM_4:         return 0x21;  // 33 ([)
        case VK_OEM_6:         return 0x1E;  // 30 (])
        case VK_OEM_5:         return 0x2A;  // 42 (\)
        case VK_OEM_MINUS:     return 0x1B;  // 27 (-)
        case VK_OEM_PLUS:      return 0x18;  // 24 (=)
        case VK_OEM_3:         return 0x32;  // 50 (`)

        default: return 0xFF; // Invalid key
    }
}

// Check if keycode is valid
inline bool IsValidKey(uint16_t keycode) {
    return keycode != 0xFF;
}

// Check if VK code should be ignored (modifiers, function keys, etc.)
inline bool ShouldIgnoreVK(uint16_t vk) {
    return vk == VK_SHIFT || vk == VK_CONTROL || vk == VK_MENU ||  // Modifiers
           vk == VK_LSHIFT || vk == VK_RSHIFT ||
           vk == VK_LCONTROL || vk == VK_RCONTROL ||
           vk == VK_LMENU || vk == VK_RMENU ||
           vk == VK_LWIN || vk == VK_RWIN ||                       // Win key
           (vk >= VK_F1 && vk <= VK_F24) ||                       // Function keys
           vk == VK_CAPITAL || vk == VK_NUMLOCK || vk == VK_SCROLL || // Lock keys
           vk == VK_PAUSE || vk == VK_SNAPSHOT;                    // System keys
}
