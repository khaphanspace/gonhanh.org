using System.Runtime.InteropServices;

namespace GoNhanh.Core;

/// <summary>
/// High-performance text sender using Windows SendInput API
/// Optimized for minimal latency and GC pressure
/// Uses pre-allocated buffers and efficient Win32 calls
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

    [StructLayout(LayoutKind.Sequential)]
    private struct INPUT
    {
        public uint type;
        public INPUTUNION u;
    }

    [StructLayout(LayoutKind.Explicit)]
    private struct INPUTUNION
    {
        [FieldOffset(0)] public KEYBDINPUT ki;
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

    #endregion

    #region Pre-allocated Buffers

    // Maximum: 32 chars * 2 (down+up) + 10 backspaces * 2 = 84 inputs
    // Use 128 for safety margin
    private const int MaxInputs = 128;

    // Thread-local buffer to avoid allocation per call
    [ThreadStatic]
    private static INPUT[]? _inputBuffer;

    private static readonly int InputSize = Marshal.SizeOf<INPUT>();

    #endregion

    /// <summary>
    /// Send text replacement with optimized performance
    /// Uses pre-allocated buffer, minimizes allocations
    /// </summary>
    /// <param name="text">Text to insert (max 32 chars from Rust core)</param>
    /// <param name="backspaces">Number of backspaces (max 10)</param>
    public static void SendText(string text, int backspaces)
    {
        if (string.IsNullOrEmpty(text) && backspaces == 0)
            return;

        // Get or create thread-local buffer
        _inputBuffer ??= new INPUT[MaxInputs];

        var marker = KeyboardHook.GetInjectedKeyMarker();
        int index = 0;

        // Add backspaces (key down + key up pairs)
        for (int i = 0; i < backspaces && index < MaxInputs - 1; i++)
        {
            // Backspace key down
            _inputBuffer[index++] = new INPUT
            {
                type = INPUT_KEYBOARD,
                u = new INPUTUNION
                {
                    ki = new KEYBDINPUT
                    {
                        wVk = KeyCodes.VK_BACK,
                        wScan = 0,
                        dwFlags = 0,
                        time = 0,
                        dwExtraInfo = marker
                    }
                }
            };

            // Backspace key up
            _inputBuffer[index++] = new INPUT
            {
                type = INPUT_KEYBOARD,
                u = new INPUTUNION
                {
                    ki = new KEYBDINPUT
                    {
                        wVk = KeyCodes.VK_BACK,
                        wScan = 0,
                        dwFlags = KEYEVENTF_KEYUP,
                        time = 0,
                        dwExtraInfo = marker
                    }
                }
            };
        }

        // Add text characters using Unicode (KEYEVENTF_UNICODE)
        // This works for all Vietnamese characters without keyboard layout issues
        foreach (char c in text)
        {
            if (index >= MaxInputs - 1) break;

            // Unicode key down
            _inputBuffer[index++] = new INPUT
            {
                type = INPUT_KEYBOARD,
                u = new INPUTUNION
                {
                    ki = new KEYBDINPUT
                    {
                        wVk = 0,
                        wScan = c,
                        dwFlags = KEYEVENTF_UNICODE,
                        time = 0,
                        dwExtraInfo = marker
                    }
                }
            };

            // Unicode key up
            _inputBuffer[index++] = new INPUT
            {
                type = INPUT_KEYBOARD,
                u = new INPUTUNION
                {
                    ki = new KEYBDINPUT
                    {
                        wVk = 0,
                        wScan = c,
                        dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time = 0,
                        dwExtraInfo = marker
                    }
                }
            };
        }

        // Send all inputs in one call for best performance
        if (index > 0)
        {
            SendInput((uint)index, _inputBuffer, InputSize);
        }
    }

    /// <summary>
    /// Send a single backspace (used for buffer corrections)
    /// </summary>
    public static void SendBackspace()
    {
        _inputBuffer ??= new INPUT[MaxInputs];
        var marker = KeyboardHook.GetInjectedKeyMarker();

        _inputBuffer[0] = new INPUT
        {
            type = INPUT_KEYBOARD,
            u = new INPUTUNION
            {
                ki = new KEYBDINPUT
                {
                    wVk = KeyCodes.VK_BACK,
                    dwFlags = 0,
                    dwExtraInfo = marker
                }
            }
        };

        _inputBuffer[1] = new INPUT
        {
            type = INPUT_KEYBOARD,
            u = new INPUTUNION
            {
                ki = new KEYBDINPUT
                {
                    wVk = KeyCodes.VK_BACK,
                    dwFlags = KEYEVENTF_KEYUP,
                    dwExtraInfo = marker
                }
            }
        };

        SendInput(2, _inputBuffer, InputSize);
    }
}
