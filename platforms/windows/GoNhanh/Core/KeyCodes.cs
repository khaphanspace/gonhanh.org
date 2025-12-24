namespace GoNhanh.Core;

/// <summary>
/// Windows Virtual Key Codes and macOS keycode mapping
/// Reference: https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
/// </summary>
public static class KeyCodes
{
    #region macOS Keycodes (for Rust core FFI)

    // The Rust core uses macOS virtual keycodes, not Windows VK codes
    // This mapping converts Windows VK → macOS keycode for FFI calls

    private const ushort MAC_A = 0;
    private const ushort MAC_S = 1;
    private const ushort MAC_D = 2;
    private const ushort MAC_F = 3;
    private const ushort MAC_H = 4;
    private const ushort MAC_G = 5;
    private const ushort MAC_Z = 6;
    private const ushort MAC_X = 7;
    private const ushort MAC_C = 8;
    private const ushort MAC_V = 9;
    private const ushort MAC_B = 11;
    private const ushort MAC_Q = 12;
    private const ushort MAC_W = 13;
    private const ushort MAC_E = 14;
    private const ushort MAC_R = 15;
    private const ushort MAC_Y = 16;
    private const ushort MAC_T = 17;
    private const ushort MAC_N1 = 18;
    private const ushort MAC_N2 = 19;
    private const ushort MAC_N3 = 20;
    private const ushort MAC_N4 = 21;
    private const ushort MAC_N6 = 22;
    private const ushort MAC_N5 = 23;
    private const ushort MAC_EQUAL = 24;
    private const ushort MAC_N9 = 25;
    private const ushort MAC_N7 = 26;
    private const ushort MAC_MINUS = 27;
    private const ushort MAC_N8 = 28;
    private const ushort MAC_N0 = 29;
    private const ushort MAC_RBRACKET = 30;
    private const ushort MAC_O = 31;
    private const ushort MAC_U = 32;
    private const ushort MAC_LBRACKET = 33;
    private const ushort MAC_I = 34;
    private const ushort MAC_P = 35;
    private const ushort MAC_RETURN = 36;
    private const ushort MAC_L = 37;
    private const ushort MAC_J = 38;
    private const ushort MAC_QUOTE = 39;
    private const ushort MAC_K = 40;
    private const ushort MAC_SEMICOLON = 41;
    private const ushort MAC_BACKSLASH = 42;
    private const ushort MAC_COMMA = 43;
    private const ushort MAC_SLASH = 44;
    private const ushort MAC_N = 45;
    private const ushort MAC_M = 46;
    private const ushort MAC_DOT = 47;
    private const ushort MAC_TAB = 48;
    private const ushort MAC_SPACE = 49;
    private const ushort MAC_BACKQUOTE = 50;
    private const ushort MAC_DELETE = 51;
    private const ushort MAC_ESC = 53;

    #endregion
    // Letters A-Z (0x41 - 0x5A)
    public const ushort VK_A = 0x41;
    public const ushort VK_B = 0x42;
    public const ushort VK_C = 0x43;
    public const ushort VK_D = 0x44;
    public const ushort VK_E = 0x45;
    public const ushort VK_F = 0x46;
    public const ushort VK_G = 0x47;
    public const ushort VK_H = 0x48;
    public const ushort VK_I = 0x49;
    public const ushort VK_J = 0x4A;
    public const ushort VK_K = 0x4B;
    public const ushort VK_L = 0x4C;
    public const ushort VK_M = 0x4D;
    public const ushort VK_N = 0x4E;
    public const ushort VK_O = 0x4F;
    public const ushort VK_P = 0x50;
    public const ushort VK_Q = 0x51;
    public const ushort VK_R = 0x52;
    public const ushort VK_S = 0x53;
    public const ushort VK_T = 0x54;
    public const ushort VK_U = 0x55;
    public const ushort VK_V = 0x56;
    public const ushort VK_W = 0x57;
    public const ushort VK_X = 0x58;
    public const ushort VK_Y = 0x59;
    public const ushort VK_Z = 0x5A;

    // Numbers 0-9 (0x30 - 0x39)
    public const ushort VK_0 = 0x30;
    public const ushort VK_1 = 0x31;
    public const ushort VK_2 = 0x32;
    public const ushort VK_3 = 0x33;
    public const ushort VK_4 = 0x34;
    public const ushort VK_5 = 0x35;
    public const ushort VK_6 = 0x36;
    public const ushort VK_7 = 0x37;
    public const ushort VK_8 = 0x38;
    public const ushort VK_9 = 0x39;

    // Special keys
    public const ushort VK_BACK = 0x08;      // Backspace
    public const ushort VK_TAB = 0x09;
    public const ushort VK_RETURN = 0x0D;    // Enter
    public const ushort VK_SHIFT = 0x10;
    public const ushort VK_CONTROL = 0x11;
    public const ushort VK_MENU = 0x12;      // Alt
    public const ushort VK_CAPITAL = 0x14;   // Caps Lock
    public const ushort VK_ESCAPE = 0x1B;
    public const ushort VK_SPACE = 0x20;

    // Punctuation (US keyboard layout)
    public const ushort VK_OEM_1 = 0xBA;     // ;:
    public const ushort VK_OEM_PLUS = 0xBB;  // =+
    public const ushort VK_OEM_COMMA = 0xBC; // ,<
    public const ushort VK_OEM_MINUS = 0xBD; // -_
    public const ushort VK_OEM_PERIOD = 0xBE;// .>
    public const ushort VK_OEM_2 = 0xBF;     // /?
    public const ushort VK_OEM_3 = 0xC0;     // `~
    public const ushort VK_OEM_4 = 0xDB;     // [{
    public const ushort VK_OEM_5 = 0xDC;     // \|
    public const ushort VK_OEM_6 = 0xDD;     // ]}
    public const ushort VK_OEM_7 = 0xDE;     // '"

    // Numpad
    public const ushort VK_NUMPAD0 = 0x60;
    public const ushort VK_NUMPAD1 = 0x61;
    public const ushort VK_NUMPAD2 = 0x62;
    public const ushort VK_NUMPAD3 = 0x63;
    public const ushort VK_NUMPAD4 = 0x64;
    public const ushort VK_NUMPAD5 = 0x65;
    public const ushort VK_NUMPAD6 = 0x66;
    public const ushort VK_NUMPAD7 = 0x67;
    public const ushort VK_NUMPAD8 = 0x68;
    public const ushort VK_NUMPAD9 = 0x69;

    /// <summary>
    /// Check if a key code is a letter (A-Z)
    /// </summary>
    public static bool IsLetter(ushort keyCode) => keyCode >= VK_A && keyCode <= VK_Z;

    /// <summary>
    /// Check if a key code is a number (0-9)
    /// </summary>
    public static bool IsNumber(ushort keyCode) => keyCode >= VK_0 && keyCode <= VK_9;

    /// <summary>
    /// Check if a key code is relevant for Vietnamese input
    /// Letters, numbers (for VNI), and word-breaking keys
    /// </summary>
    public static bool IsRelevantKey(ushort keyCode)
    {
        return IsLetter(keyCode) ||
               IsNumber(keyCode) ||
               keyCode == VK_SPACE ||
               keyCode == VK_RETURN ||
               keyCode == VK_BACK ||
               keyCode == VK_OEM_4 ||  // [
               keyCode == VK_OEM_6;    // ]
    }

    /// <summary>
    /// Check if a key should clear the IME buffer (word boundaries)
    /// </summary>
    public static bool IsBufferClearKey(ushort keyCode)
    {
        return keyCode == VK_SPACE ||
               keyCode == VK_RETURN ||
               keyCode == VK_TAB ||
               keyCode == VK_ESCAPE;
    }

    /// <summary>
    /// Convert Windows VK code to macOS keycode for Rust core FFI
    /// Returns 0xFFFF if no mapping exists
    /// </summary>
    public static ushort ToMacKeyCode(ushort vkCode)
    {
        return vkCode switch
        {
            // Letters
            VK_A => MAC_A,
            VK_B => MAC_B,
            VK_C => MAC_C,
            VK_D => MAC_D,
            VK_E => MAC_E,
            VK_F => MAC_F,
            VK_G => MAC_G,
            VK_H => MAC_H,
            VK_I => MAC_I,
            VK_J => MAC_J,
            VK_K => MAC_K,
            VK_L => MAC_L,
            VK_M => MAC_M,
            VK_N => MAC_N,
            VK_O => MAC_O,
            VK_P => MAC_P,
            VK_Q => MAC_Q,
            VK_R => MAC_R,
            VK_S => MAC_S,
            VK_T => MAC_T,
            VK_U => MAC_U,
            VK_V => MAC_V,
            VK_W => MAC_W,
            VK_X => MAC_X,
            VK_Y => MAC_Y,
            VK_Z => MAC_Z,

            // Numbers
            VK_0 => MAC_N0,
            VK_1 => MAC_N1,
            VK_2 => MAC_N2,
            VK_3 => MAC_N3,
            VK_4 => MAC_N4,
            VK_5 => MAC_N5,
            VK_6 => MAC_N6,
            VK_7 => MAC_N7,
            VK_8 => MAC_N8,
            VK_9 => MAC_N9,

            // Special keys
            VK_SPACE => MAC_SPACE,
            VK_RETURN => MAC_RETURN,
            VK_BACK => MAC_DELETE,
            VK_TAB => MAC_TAB,
            VK_ESCAPE => MAC_ESC,

            // Punctuation
            VK_OEM_1 => MAC_SEMICOLON,      // ;:
            VK_OEM_PLUS => MAC_EQUAL,       // =+
            VK_OEM_COMMA => MAC_COMMA,      // ,<
            VK_OEM_MINUS => MAC_MINUS,      // -_
            VK_OEM_PERIOD => MAC_DOT,       // .>
            VK_OEM_2 => MAC_SLASH,          // /?
            VK_OEM_3 => MAC_BACKQUOTE,      // `~
            VK_OEM_4 => MAC_LBRACKET,       // [{
            VK_OEM_5 => MAC_BACKSLASH,      // \|
            VK_OEM_6 => MAC_RBRACKET,       // ]}
            VK_OEM_7 => MAC_QUOTE,          // '"

            _ => 0xFFFF  // No mapping
        };
    }
}
