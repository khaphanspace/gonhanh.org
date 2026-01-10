using System.Runtime.InteropServices;

namespace GoNhanh.Core;

/// <summary>
/// Inject Unicode text using SendInput with KEYEVENTF_UNICODE.
/// Also handles backspace simulation for character replacement.
/// Implemented in Phase 2.
/// </summary>
public static class TextInjector
{
    private const uint INPUT_KEYBOARD = 1;
    private const uint KEYEVENTF_UNICODE = 0x0004;
    private const uint KEYEVENTF_KEYUP = 0x0002;
    private const ushort VK_BACK = 0x08;

    /// <summary>
    /// Send backspaces then inject Unicode text.
    /// </summary>
    public static void ReplaceText(int backspaceCount, string text)
    {
        int totalInputs = (backspaceCount * 2) + (text.Length * 2);
        if (totalInputs == 0) return;

        var inputs = new INPUT[totalInputs];
        int idx = 0;

        // Send backspaces
        for (int i = 0; i < backspaceCount; i++)
        {
            inputs[idx++] = CreateKeyboardInput(VK_BACK, 0, 0);
            inputs[idx++] = CreateKeyboardInput(VK_BACK, 0, KEYEVENTF_KEYUP);
        }

        // Send Unicode characters
        foreach (char c in text)
        {
            inputs[idx++] = CreateUnicodeInput(c, 0);
            inputs[idx++] = CreateUnicodeInput(c, KEYEVENTF_KEYUP);
        }

        SendInput((uint)inputs.Length, inputs, Marshal.SizeOf<INPUT>());
    }

    /// <summary>
    /// Send single key press.
    /// </summary>
    public static void SendKey(ushort vkCode)
    {
        var inputs = new INPUT[2];
        inputs[0] = CreateKeyboardInput(vkCode, 0, 0);
        inputs[1] = CreateKeyboardInput(vkCode, 0, KEYEVENTF_KEYUP);
        SendInput(2, inputs, Marshal.SizeOf<INPUT>());
    }

    /// <summary>
    /// Inject Unicode string only (no backspaces).
    /// </summary>
    public static void SendUnicode(string text)
    {
        if (string.IsNullOrEmpty(text)) return;

        var inputs = new INPUT[text.Length * 2];
        int idx = 0;

        foreach (char c in text)
        {
            inputs[idx++] = CreateUnicodeInput(c, 0);
            inputs[idx++] = CreateUnicodeInput(c, KEYEVENTF_KEYUP);
        }

        SendInput((uint)inputs.Length, inputs, Marshal.SizeOf<INPUT>());
    }

    private static INPUT CreateKeyboardInput(ushort vkCode, ushort scanCode, uint flags)
    {
        return new INPUT
        {
            type = INPUT_KEYBOARD,
            u = new InputUnion
            {
                ki = new KEYBDINPUT
                {
                    wVk = vkCode,
                    wScan = scanCode,
                    dwFlags = flags,
                    time = 0,
                    dwExtraInfo = IntPtr.Zero
                }
            }
        };
    }

    private static INPUT CreateUnicodeInput(char c, uint flags)
    {
        return new INPUT
        {
            type = INPUT_KEYBOARD,
            u = new InputUnion
            {
                ki = new KEYBDINPUT
                {
                    wVk = 0,
                    wScan = c,
                    dwFlags = KEYEVENTF_UNICODE | flags,
                    time = 0,
                    dwExtraInfo = IntPtr.Zero
                }
            }
        };
    }

    // P/Invoke structures
    [StructLayout(LayoutKind.Sequential)]
    private struct INPUT
    {
        public uint type;
        public InputUnion u;
    }

    [StructLayout(LayoutKind.Explicit)]
    private struct InputUnion
    {
        [FieldOffset(0)] public KEYBDINPUT ki;
        [FieldOffset(0)] public MOUSEINPUT mi;
        [FieldOffset(0)] public HARDWAREINPUT hi;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct KEYBDINPUT
    {
        public ushort wVk;
        public ushort wScan;
        public uint dwFlags;
        public uint time;
        public IntPtr dwExtraInfo;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct MOUSEINPUT
    {
        public int dx, dy;
        public uint mouseData, dwFlags, time;
        public IntPtr dwExtraInfo;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct HARDWAREINPUT
    {
        public uint uMsg;
        public ushort wParamL, wParamH;
    }

    [DllImport("user32.dll", SetLastError = true)]
    private static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);
}
