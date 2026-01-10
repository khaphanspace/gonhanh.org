namespace GoNhanh.Core;

/// <summary>
/// Map Windows Virtual Key codes to macOS keycodes (Rust engine format).
/// Keycodes must match core/src/data/keys.rs exactly.
/// </summary>
public static class KeycodeMap
{
    // macOS keycodes from core/src/data/keys.rs
    public const ushort A = 0;
    public const ushort S = 1;
    public const ushort D = 2;
    public const ushort F = 3;
    public const ushort H = 4;
    public const ushort G = 5;
    public const ushort Z = 6;
    public const ushort X = 7;
    public const ushort C = 8;
    public const ushort V = 9;
    public const ushort B = 11;
    public const ushort Q = 12;
    public const ushort W = 13;
    public const ushort E = 14;
    public const ushort R = 15;
    public const ushort Y = 16;
    public const ushort T = 17;
    public const ushort O = 31;
    public const ushort U = 32;
    public const ushort I = 34;
    public const ushort P = 35;
    public const ushort L = 37;
    public const ushort J = 38;
    public const ushort K = 40;
    public const ushort N = 45;
    public const ushort M = 46;

    // Numbers
    public const ushort N1 = 18;
    public const ushort N2 = 19;
    public const ushort N3 = 20;
    public const ushort N4 = 21;
    public const ushort N5 = 23;
    public const ushort N6 = 22;
    public const ushort N7 = 26;
    public const ushort N8 = 28;
    public const ushort N9 = 25;
    public const ushort N0 = 29;

    // Special keys
    public const ushort SPACE = 49;
    public const ushort DELETE = 51;
    public const ushort TAB = 48;
    public const ushort RETURN = 36;
    public const ushort ENTER = 76;
    public const ushort ESC = 53;
    public const ushort LEFT = 123;
    public const ushort RIGHT = 124;
    public const ushort DOWN = 125;
    public const ushort UP = 126;

    // Punctuation
    public const ushort DOT = 47;
    public const ushort COMMA = 43;
    public const ushort SLASH = 44;
    public const ushort SEMICOLON = 41;
    public const ushort QUOTE = 39;
    public const ushort LBRACKET = 33;
    public const ushort RBRACKET = 30;
    public const ushort BACKSLASH = 42;
    public const ushort MINUS = 27;
    public const ushort EQUAL = 24;
    public const ushort BACKQUOTE = 50;

    // Windows Virtual Key codes
    private const int VK_BACK = 0x08;
    private const int VK_TAB = 0x09;
    private const int VK_RETURN = 0x0D;
    private const int VK_ESCAPE = 0x1B;
    private const int VK_SPACE = 0x20;
    private const int VK_LEFT = 0x25;
    private const int VK_UP = 0x26;
    private const int VK_RIGHT = 0x27;
    private const int VK_DOWN = 0x28;

    /// <summary>
    /// Convert Windows VK code to macOS keycode.
    /// Returns null if key not mapped (non-IME key).
    /// </summary>
    public static ushort? FromVirtualKey(int vkCode)
    {
        return vkCode switch
        {
            // Letters A-Z (VK codes 0x41-0x5A)
            0x41 => A, 0x42 => B, 0x43 => C, 0x44 => D, 0x45 => E,
            0x46 => F, 0x47 => G, 0x48 => H, 0x49 => I, 0x4A => J,
            0x4B => K, 0x4C => L, 0x4D => M, 0x4E => N, 0x4F => O,
            0x50 => P, 0x51 => Q, 0x52 => R, 0x53 => S, 0x54 => T,
            0x55 => U, 0x56 => V, 0x57 => W, 0x58 => X, 0x59 => Y,
            0x5A => Z,

            // Numbers 0-9 (VK codes 0x30-0x39)
            0x30 => N0, 0x31 => N1, 0x32 => N2, 0x33 => N3, 0x34 => N4,
            0x35 => N5, 0x36 => N6, 0x37 => N7, 0x38 => N8, 0x39 => N9,

            // Special keys
            VK_SPACE => SPACE,
            VK_RETURN => RETURN,
            VK_BACK => DELETE,
            VK_TAB => TAB,
            VK_ESCAPE => ESC,
            VK_LEFT => LEFT,
            VK_RIGHT => RIGHT,
            VK_UP => UP,
            VK_DOWN => DOWN,

            // OEM keys (US keyboard layout)
            0xBA => SEMICOLON,  // VK_OEM_1 (;:)
            0xBB => EQUAL,      // VK_OEM_PLUS (=+)
            0xBC => COMMA,      // VK_OEM_COMMA (,<)
            0xBD => MINUS,      // VK_OEM_MINUS (-_)
            0xBE => DOT,        // VK_OEM_PERIOD (.>)
            0xBF => SLASH,      // VK_OEM_2 (/?)
            0xC0 => BACKQUOTE,  // VK_OEM_3 (`~)
            0xDB => LBRACKET,   // VK_OEM_4 ([{)
            0xDC => BACKSLASH,  // VK_OEM_5 (\|)
            0xDD => RBRACKET,   // VK_OEM_6 (]})
            0xDE => QUOTE,      // VK_OEM_7 ('")

            _ => null
        };
    }

    /// <summary>
    /// Check if the key is a break key (space, punctuation, arrows).
    /// Should clear IME buffer when pressed.
    /// </summary>
    public static bool IsBreakKey(int vkCode)
    {
        return vkCode switch
        {
            VK_SPACE or VK_TAB or VK_RETURN or VK_ESCAPE => true,
            VK_LEFT or VK_RIGHT or VK_UP or VK_DOWN => true,
            0xBA or 0xBC or 0xBD or 0xBE or 0xBF or 0xC0 => true, // ;,-.`/
            0xDB or 0xDC or 0xDD or 0xDE => true,  // []\'
            _ => false
        };
    }

    /// <summary>
    /// Check if VK code is a letter key (A-Z).
    /// </summary>
    public static bool IsLetter(int vkCode) => vkCode >= 0x41 && vkCode <= 0x5A;

    /// <summary>
    /// Check if VK code is a number key (0-9).
    /// </summary>
    public static bool IsNumber(int vkCode) => vkCode >= 0x30 && vkCode <= 0x39;
}
