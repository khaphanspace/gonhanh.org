using System.Runtime.InteropServices;

namespace GoNhanh.Core;

/// <summary>
/// Sends text to the active window using Windows SendInput API
/// Handles backspace deletion and Unicode character insertion
/// </summary>
public static class TextSender
{
    #region Win32 Constants

    private const uint INPUT_KEYBOARD = 1;
    private const uint KEYEVENTF_KEYUP = 0x0002;
    private const uint KEYEVENTF_UNICODE = 0x0004;

    #endregion

    #region Win32 Imports

    [DllImport("user32.dll", SetLastError = true)]
    private static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);

    #endregion

    #region Structures

    // INPUT struct for x64 Windows - must match native layout exactly
    // type (4 bytes) + padding (4 bytes on x64) + union (40 bytes) = 48 bytes
    [StructLayout(LayoutKind.Explicit, Size = 40)]
    private struct INPUT
    {
        [FieldOffset(0)] public uint type;
        [FieldOffset(8)] public KEYBDINPUT ki;  // Offset 8 on x64 due to alignment
    }

    // KEYBDINPUT: wVk(2) + wScan(2) + dwFlags(4) + time(4) + dwExtraInfo(8 on x64) = 20 bytes
    // But aligned to 8 bytes = 24 bytes with padding
    [StructLayout(LayoutKind.Sequential)]
    private struct KEYBDINPUT
    {
        public ushort wVk;
        public ushort wScan;
        public uint dwFlags;
        public uint time;
        public IntPtr dwExtraInfo;
    }

    #endregion

    /// <summary>
    /// Send text replacement: delete characters and insert new text
    /// </summary>
    /// <param name="text">Text to insert</param>
    /// <param name="backspaces">Number of backspaces to send first</param>
    public static void SendText(string text, int backspaces)
    {
        if (string.IsNullOrEmpty(text) && backspaces == 0)
            return;

        var inputs = new List<INPUT>();
        var marker = KeyboardHook.GetInjectedKeyMarker();

        // Add backspaces
        for (int i = 0; i < backspaces; i++)
        {
            // Key down
            inputs.Add(new INPUT
            {
                type = INPUT_KEYBOARD,
                ki = new KEYBDINPUT
                {
                    wVk = KeyCodes.VK_BACK,
                    wScan = 0,
                    dwFlags = 0,
                    time = 0,
                    dwExtraInfo = marker
                }
            });

            // Key up
            inputs.Add(new INPUT
            {
                type = INPUT_KEYBOARD,
                ki = new KEYBDINPUT
                {
                    wVk = KeyCodes.VK_BACK,
                    wScan = 0,
                    dwFlags = KEYEVENTF_KEYUP,
                    time = 0,
                    dwExtraInfo = marker
                }
            });
        }

        // Add text characters (Unicode)
        foreach (char c in text)
        {
            // For Unicode characters, use wScan with KEYEVENTF_UNICODE flag
            // Key down
            inputs.Add(new INPUT
            {
                type = INPUT_KEYBOARD,
                ki = new KEYBDINPUT
                {
                    wVk = 0,
                    wScan = c,
                    dwFlags = KEYEVENTF_UNICODE,
                    time = 0,
                    dwExtraInfo = marker
                }
            });

            // Key up
            inputs.Add(new INPUT
            {
                type = INPUT_KEYBOARD,
                ki = new KEYBDINPUT
                {
                    wVk = 0,
                    wScan = c,
                    dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    time = 0,
                    dwExtraInfo = marker
                }
            });
        }

        if (inputs.Count > 0)
        {
            var inputArray = inputs.ToArray();
            SendInput((uint)inputArray.Length, inputArray, Marshal.SizeOf<INPUT>());
        }
    }
}
