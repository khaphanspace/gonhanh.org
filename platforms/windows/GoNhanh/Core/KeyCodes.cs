namespace GoNhanh.Core;

/// <summary>
/// Virtual key codes and mapping to macOS keycodes.
/// Rust engine uses macOS keycode values internally.
/// </summary>
public static class KeyCodes
{
    // Windows VK codes
    public const int VK_BACK = 0x08;
    public const int VK_TAB = 0x09;
    public const int VK_RETURN = 0x0D;
    public const int VK_SHIFT = 0x10;
    public const int VK_CONTROL = 0x11;
    public const int VK_MENU = 0x12;     // Alt
    public const int VK_CAPITAL = 0x14;  // CapsLock
    public const int VK_ESCAPE = 0x1B;
    public const int VK_SPACE = 0x20;
    public const int VK_LEFT = 0x25;
    public const int VK_UP = 0x26;
    public const int VK_RIGHT = 0x27;
    public const int VK_DOWN = 0x28;

    // Numbers 0-9: 0x30-0x39
    // Letters A-Z: 0x41-0x5A
    public const int VK_A = 0x41;
    public const int VK_Z = 0x5A;

    // OEM keys
    public const int VK_OEM_1 = 0xBA;      // ;:
    public const int VK_OEM_PLUS = 0xBB;   // =+
    public const int VK_OEM_COMMA = 0xBC;  // ,<
    public const int VK_OEM_MINUS = 0xBD;  // -_
    public const int VK_OEM_PERIOD = 0xBE; // .>
    public const int VK_OEM_2 = 0xBF;      // /?
    public const int VK_OEM_3 = 0xC0;      // `~
    public const int VK_OEM_4 = 0xDB;      // [{
    public const int VK_OEM_5 = 0xDC;      // \|
    public const int VK_OEM_6 = 0xDD;      // ]}
    public const int VK_OEM_7 = 0xDE;      // '"

    // macOS keycode values (used by Rust engine)
    private static readonly ushort[] MacKeycodes = new ushort[256];

    static KeyCodes()
    {
        // Initialize all to 0xFF (invalid)
        for (int i = 0; i < 256; i++)
            MacKeycodes[i] = 0xFF;

        // Letters A-Z (macOS uses 0x00-0x19 for A-Z, but not in order!)
        // Must match core/src/data/keys.rs
        MacKeycodes[VK_A] = 0;   // kVK_ANSI_A
        MacKeycodes[0x42] = 11;  // B -> kVK_ANSI_B
        MacKeycodes[0x43] = 8;   // C -> kVK_ANSI_C
        MacKeycodes[0x44] = 2;   // D -> kVK_ANSI_D
        MacKeycodes[0x45] = 14;  // E -> kVK_ANSI_E
        MacKeycodes[0x46] = 3;   // F -> kVK_ANSI_F
        MacKeycodes[0x47] = 5;   // G -> kVK_ANSI_G
        MacKeycodes[0x48] = 4;   // H -> kVK_ANSI_H
        MacKeycodes[0x49] = 34;  // I -> kVK_ANSI_I
        MacKeycodes[0x4A] = 38;  // J -> kVK_ANSI_J
        MacKeycodes[0x4B] = 40;  // K -> kVK_ANSI_K
        MacKeycodes[0x4C] = 37;  // L -> kVK_ANSI_L
        MacKeycodes[0x4D] = 46;  // M -> kVK_ANSI_M
        MacKeycodes[0x4E] = 45;  // N -> kVK_ANSI_N
        MacKeycodes[0x4F] = 31;  // O -> kVK_ANSI_O
        MacKeycodes[0x50] = 35;  // P -> kVK_ANSI_P
        MacKeycodes[0x51] = 12;  // Q -> kVK_ANSI_Q
        MacKeycodes[0x52] = 15;  // R -> kVK_ANSI_R
        MacKeycodes[0x53] = 1;   // S -> kVK_ANSI_S
        MacKeycodes[0x54] = 17;  // T -> kVK_ANSI_T
        MacKeycodes[0x55] = 32;  // U -> kVK_ANSI_U
        MacKeycodes[0x56] = 9;   // V -> kVK_ANSI_V
        MacKeycodes[0x57] = 13;  // W -> kVK_ANSI_W
        MacKeycodes[0x58] = 7;   // X -> kVK_ANSI_X
        MacKeycodes[0x59] = 16;  // Y -> kVK_ANSI_Y
        MacKeycodes[0x5A] = 6;   // Z -> kVK_ANSI_Z

        // Numbers 0-9
        MacKeycodes[0x30] = 29;  // 0 -> kVK_ANSI_0
        MacKeycodes[0x31] = 18;  // 1 -> kVK_ANSI_1
        MacKeycodes[0x32] = 19;  // 2
        MacKeycodes[0x33] = 20;  // 3
        MacKeycodes[0x34] = 21;  // 4
        MacKeycodes[0x35] = 22;  // 5
        MacKeycodes[0x36] = 23;  // 6
        MacKeycodes[0x37] = 26;  // 7
        MacKeycodes[0x38] = 28;  // 8
        MacKeycodes[0x39] = 25;  // 9

        // Special keys
        MacKeycodes[VK_SPACE] = 49;
        MacKeycodes[VK_RETURN] = 36;
        MacKeycodes[VK_BACK] = 51;
        MacKeycodes[VK_ESCAPE] = 53;
        MacKeycodes[VK_TAB] = 48;

        // Punctuation
        MacKeycodes[VK_OEM_COMMA] = 43;   // ,
        MacKeycodes[VK_OEM_PERIOD] = 47;  // .
        MacKeycodes[VK_OEM_1] = 41;       // ;
        MacKeycodes[VK_OEM_4] = 33;       // [
        MacKeycodes[VK_OEM_6] = 30;       // ]
        MacKeycodes[VK_OEM_MINUS] = 27;   // -
        MacKeycodes[VK_OEM_PLUS] = 24;    // =
    }

    /// <summary>Convert Windows VK to macOS keycode for Rust engine</summary>
    public static ushort ToMacKeycode(int vk) =>
        vk >= 0 && vk < 256 ? MacKeycodes[vk] : (ushort)0xFF;

    /// <summary>Check if key is a letter (A-Z)</summary>
    public static bool IsLetter(int vk) => vk >= VK_A && vk <= VK_Z;

    /// <summary>Check if key is a number (0-9)</summary>
    public static bool IsNumber(int vk) => vk >= 0x30 && vk <= 0x39;

    /// <summary>Check if key is word boundary (space, punctuation, etc.)</summary>
    public static bool IsWordBoundary(int vk) =>
        vk == VK_SPACE || vk == VK_RETURN || vk == VK_TAB ||
        vk == VK_OEM_COMMA || vk == VK_OEM_PERIOD || vk == VK_OEM_1;

    /// <summary>Check if key should clear buffer (cursor movement)</summary>
    public static bool IsCursorKey(int vk) =>
        vk == VK_LEFT || vk == VK_RIGHT || vk == VK_UP || vk == VK_DOWN;
}
