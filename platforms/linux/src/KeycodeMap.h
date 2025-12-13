#ifndef GONHANH_KEYCODE_MAP_H
#define GONHANH_KEYCODE_MAP_H

#include <cstdint>
#include <xkbcommon/xkbcommon-keysyms.h>

// Map X11/Wayland keysym to macOS virtual keycode
// Reference: core/src/data/keys.rs for macOS keycodes

namespace KeycodeMap {

// macOS virtual keycodes (from core/src/data/keys.rs)
namespace MacKey {
    constexpr uint16_t A = 0;
    constexpr uint16_t S = 1;
    constexpr uint16_t D = 2;
    constexpr uint16_t F = 3;
    constexpr uint16_t H = 4;
    constexpr uint16_t G = 5;
    constexpr uint16_t Z = 6;
    constexpr uint16_t X = 7;
    constexpr uint16_t C = 8;
    constexpr uint16_t V = 9;
    constexpr uint16_t B = 11;
    constexpr uint16_t Q = 12;
    constexpr uint16_t W = 13;
    constexpr uint16_t E = 14;
    constexpr uint16_t R = 15;
    constexpr uint16_t Y = 16;
    constexpr uint16_t T = 17;
    constexpr uint16_t N1 = 18;
    constexpr uint16_t N2 = 19;
    constexpr uint16_t N3 = 20;
    constexpr uint16_t N4 = 21;
    constexpr uint16_t N6 = 22;
    constexpr uint16_t N5 = 23;
    constexpr uint16_t EQUAL = 24;
    constexpr uint16_t N9 = 25;
    constexpr uint16_t N7 = 26;
    constexpr uint16_t MINUS = 27;
    constexpr uint16_t N8 = 28;
    constexpr uint16_t N0 = 29;
    constexpr uint16_t RBRACKET = 30;
    constexpr uint16_t O = 31;
    constexpr uint16_t U = 32;
    constexpr uint16_t LBRACKET = 33;
    constexpr uint16_t I = 34;
    constexpr uint16_t P = 35;
    constexpr uint16_t RETURN = 36;
    constexpr uint16_t L = 37;
    constexpr uint16_t J = 38;
    constexpr uint16_t QUOTE = 39;
    constexpr uint16_t K = 40;
    constexpr uint16_t SEMICOLON = 41;
    constexpr uint16_t BACKSLASH = 42;
    constexpr uint16_t COMMA = 43;
    constexpr uint16_t SLASH = 44;
    constexpr uint16_t N = 45;
    constexpr uint16_t M = 46;
    constexpr uint16_t DOT = 47;
    constexpr uint16_t TAB = 48;
    constexpr uint16_t SPACE = 49;
    constexpr uint16_t BACKQUOTE = 50;
    constexpr uint16_t DELETE = 51;
    constexpr uint16_t ESC = 53;
    constexpr uint16_t ENTER = 76;
    constexpr uint16_t LEFT = 123;
    constexpr uint16_t RIGHT = 124;
    constexpr uint16_t DOWN = 125;
    constexpr uint16_t UP = 126;

    // Invalid/unknown key
    constexpr uint16_t UNKNOWN = 0xFF;
}

// Convert XKB keysym to macOS keycode
inline uint16_t keysymToMacKeycode(uint32_t keysym) {
    // Handle both lowercase and uppercase letters
    switch (keysym) {
        // Letters (lowercase and uppercase)
        case XKB_KEY_a: case XKB_KEY_A: return MacKey::A;
        case XKB_KEY_b: case XKB_KEY_B: return MacKey::B;
        case XKB_KEY_c: case XKB_KEY_C: return MacKey::C;
        case XKB_KEY_d: case XKB_KEY_D: return MacKey::D;
        case XKB_KEY_e: case XKB_KEY_E: return MacKey::E;
        case XKB_KEY_f: case XKB_KEY_F: return MacKey::F;
        case XKB_KEY_g: case XKB_KEY_G: return MacKey::G;
        case XKB_KEY_h: case XKB_KEY_H: return MacKey::H;
        case XKB_KEY_i: case XKB_KEY_I: return MacKey::I;
        case XKB_KEY_j: case XKB_KEY_J: return MacKey::J;
        case XKB_KEY_k: case XKB_KEY_K: return MacKey::K;
        case XKB_KEY_l: case XKB_KEY_L: return MacKey::L;
        case XKB_KEY_m: case XKB_KEY_M: return MacKey::M;
        case XKB_KEY_n: case XKB_KEY_N: return MacKey::N;
        case XKB_KEY_o: case XKB_KEY_O: return MacKey::O;
        case XKB_KEY_p: case XKB_KEY_P: return MacKey::P;
        case XKB_KEY_q: case XKB_KEY_Q: return MacKey::Q;
        case XKB_KEY_r: case XKB_KEY_R: return MacKey::R;
        case XKB_KEY_s: case XKB_KEY_S: return MacKey::S;
        case XKB_KEY_t: case XKB_KEY_T: return MacKey::T;
        case XKB_KEY_u: case XKB_KEY_U: return MacKey::U;
        case XKB_KEY_v: case XKB_KEY_V: return MacKey::V;
        case XKB_KEY_w: case XKB_KEY_W: return MacKey::W;
        case XKB_KEY_x: case XKB_KEY_X: return MacKey::X;
        case XKB_KEY_y: case XKB_KEY_Y: return MacKey::Y;
        case XKB_KEY_z: case XKB_KEY_Z: return MacKey::Z;

        // Numbers
        case XKB_KEY_0: case XKB_KEY_parenright: return MacKey::N0;
        case XKB_KEY_1: case XKB_KEY_exclam: return MacKey::N1;
        case XKB_KEY_2: case XKB_KEY_at: return MacKey::N2;
        case XKB_KEY_3: case XKB_KEY_numbersign: return MacKey::N3;
        case XKB_KEY_4: case XKB_KEY_dollar: return MacKey::N4;
        case XKB_KEY_5: case XKB_KEY_percent: return MacKey::N5;
        case XKB_KEY_6: case XKB_KEY_asciicircum: return MacKey::N6;
        case XKB_KEY_7: case XKB_KEY_ampersand: return MacKey::N7;
        case XKB_KEY_8: case XKB_KEY_asterisk: return MacKey::N8;
        case XKB_KEY_9: case XKB_KEY_parenleft: return MacKey::N9;

        // Punctuation
        case XKB_KEY_space: return MacKey::SPACE;
        case XKB_KEY_Return: return MacKey::RETURN;
        case XKB_KEY_Tab: return MacKey::TAB;
        case XKB_KEY_BackSpace: return MacKey::DELETE;
        case XKB_KEY_Escape: return MacKey::ESC;
        case XKB_KEY_comma: case XKB_KEY_less: return MacKey::COMMA;
        case XKB_KEY_period: case XKB_KEY_greater: return MacKey::DOT;
        case XKB_KEY_slash: case XKB_KEY_question: return MacKey::SLASH;
        case XKB_KEY_semicolon: case XKB_KEY_colon: return MacKey::SEMICOLON;
        case XKB_KEY_apostrophe: case XKB_KEY_quotedbl: return MacKey::QUOTE;
        case XKB_KEY_bracketleft: case XKB_KEY_braceleft: return MacKey::LBRACKET;
        case XKB_KEY_bracketright: case XKB_KEY_braceright: return MacKey::RBRACKET;
        case XKB_KEY_backslash: case XKB_KEY_bar: return MacKey::BACKSLASH;
        case XKB_KEY_minus: case XKB_KEY_underscore: return MacKey::MINUS;
        case XKB_KEY_equal: case XKB_KEY_plus: return MacKey::EQUAL;
        case XKB_KEY_grave: case XKB_KEY_asciitilde: return MacKey::BACKQUOTE;

        // Arrow keys
        case XKB_KEY_Left: return MacKey::LEFT;
        case XKB_KEY_Right: return MacKey::RIGHT;
        case XKB_KEY_Up: return MacKey::UP;
        case XKB_KEY_Down: return MacKey::DOWN;

        default: return MacKey::UNKNOWN;
    }
}

// Check if key is a word break (space, punctuation, arrows, etc.)
inline bool isBreakKey(uint32_t keysym) {
    switch (keysym) {
        case XKB_KEY_space:
        case XKB_KEY_Tab:
        case XKB_KEY_Return:
        case XKB_KEY_Escape:
        case XKB_KEY_Left:
        case XKB_KEY_Right:
        case XKB_KEY_Up:
        case XKB_KEY_Down:
        case XKB_KEY_comma: case XKB_KEY_less:
        case XKB_KEY_period: case XKB_KEY_greater:
        case XKB_KEY_slash: case XKB_KEY_question:
        case XKB_KEY_semicolon: case XKB_KEY_colon:
        case XKB_KEY_apostrophe: case XKB_KEY_quotedbl:
        case XKB_KEY_bracketleft: case XKB_KEY_braceleft:
        case XKB_KEY_bracketright: case XKB_KEY_braceright:
        case XKB_KEY_backslash: case XKB_KEY_bar:
        case XKB_KEY_minus: case XKB_KEY_underscore:
        case XKB_KEY_equal: case XKB_KEY_plus:
        case XKB_KEY_grave: case XKB_KEY_asciitilde:
            return true;
        default:
            return false;
    }
}

// Check if key is a letter (for IME processing)
inline bool isLetterKey(uint32_t keysym) {
    return (keysym >= XKB_KEY_a && keysym <= XKB_KEY_z) ||
           (keysym >= XKB_KEY_A && keysym <= XKB_KEY_Z);
}

// Check if key is a number (for VNI mode)
inline bool isNumberKey(uint32_t keysym) {
    return (keysym >= XKB_KEY_0 && keysym <= XKB_KEY_9);
}

} // namespace KeycodeMap

#endif // GONHANH_KEYCODE_MAP_H
