using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

namespace GoNhanh.Core;

/// <summary>
/// Sends text via Windows SendInput API
/// </summary>
public sealed class TextSender
{
    // Win32 constants
    private const uint INPUT_KEYBOARD = 1;
    private const uint KEYEVENTF_KEYUP = 0x0002;
    private const uint KEYEVENTF_UNICODE = 0x0004;

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
        public UIntPtr dwExtraInfo;
    }

    [DllImport("user32.dll", SetLastError = true)]
    private static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);

    [DllImport("user32.dll")]
    private static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [DllImport("user32.dll")]
    private static extern bool AttachThreadInput(uint idAttach, uint idAttachTo, bool fAttach);

    [DllImport("kernel32.dll")]
    private static extern uint GetCurrentThreadId();

    [DllImport("user32.dll")]
    private static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);

    // Use the same injected flag as KeyboardHook
    private readonly UIntPtr _injectedFlag = KeyboardHook.InjectedFlag;

    // Delay settings (ms)
    private const int BROWSER_DELAY = 2;  // Chrome, Edge need slight delay

    /// <summary>Send backspaces then text</summary>
    public void SendReplace(int backspaceCount, string text)
    {
        if (backspaceCount <= 0 && string.IsNullOrEmpty(text)) return;

        // Attach to foreground window's input queue
        var hwnd = GetForegroundWindow();
        GetWindowThreadProcessId(hwnd, out uint targetThread);
        uint currentThread = GetCurrentThreadId();

        bool attached = false;
        if (targetThread != currentThread)
        {
            attached = AttachThreadInput(currentThread, targetThread, true);
        }

        try
        {
            // Send backspaces
            for (int i = 0; i < backspaceCount; i++)
            {
                SendBackspace();
            }

            // Small delay for apps that process backspace slowly
            if (backspaceCount > 0 && NeedsDelay(hwnd))
            {
                Thread.Sleep(BROWSER_DELAY);
            }

            // Send text
            SendText(text);
        }
        finally
        {
            if (attached)
            {
                AttachThreadInput(currentThread, targetThread, false);
            }
        }
    }

    /// <summary>Send a single backspace</summary>
    private void SendBackspace()
    {
        var inputs = new INPUT[2];

        // Key down
        inputs[0].type = INPUT_KEYBOARD;
        inputs[0].u.ki.wVk = 0x08; // VK_BACK
        inputs[0].u.ki.dwExtraInfo = _injectedFlag;

        // Key up
        inputs[1].type = INPUT_KEYBOARD;
        inputs[1].u.ki.wVk = 0x08;
        inputs[1].u.ki.dwFlags = KEYEVENTF_KEYUP;
        inputs[1].u.ki.dwExtraInfo = _injectedFlag;

        SendInput(2, inputs, Marshal.SizeOf<INPUT>());
    }

    /// <summary>Send Unicode text</summary>
    private void SendText(string text)
    {
        if (string.IsNullOrEmpty(text)) return;

        var inputs = new List<INPUT>();

        for (int i = 0; i < text.Length; i++)
        {
            char c = text[i];

            // Handle surrogate pairs for emoji/rare chars
            if (char.IsHighSurrogate(c) && i + 1 < text.Length)
            {
                char low = text[i + 1];
                if (char.IsLowSurrogate(low))
                {
                    AddUnicodeInput(inputs, c);
                    AddUnicodeInput(inputs, low);
                    i++; // Skip next char
                    continue;
                }
            }

            if (!char.IsSurrogate(c))
            {
                AddUnicodeInput(inputs, c);
            }
        }

        if (inputs.Count > 0)
        {
            SendInput((uint)inputs.Count, inputs.ToArray(), Marshal.SizeOf<INPUT>());
        }
    }

    private void AddUnicodeInput(List<INPUT> inputs, char c)
    {
        // Key down
        inputs.Add(new INPUT
        {
            type = INPUT_KEYBOARD,
            u = new INPUTUNION
            {
                ki = new KEYBDINPUT
                {
                    wVk = 0,
                    wScan = c,
                    dwFlags = KEYEVENTF_UNICODE,
                    dwExtraInfo = _injectedFlag
                }
            }
        });

        // Key up
        inputs.Add(new INPUT
        {
            type = INPUT_KEYBOARD,
            u = new INPUTUNION
            {
                ki = new KEYBDINPUT
                {
                    wVk = 0,
                    wScan = c,
                    dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    dwExtraInfo = _injectedFlag
                }
            }
        });
    }

    /// <summary>Check if window needs delay between backspace and text</summary>
    private bool NeedsDelay(IntPtr hwnd)
    {
        var className = GetWindowClassName(hwnd);

        // Known slow apps
        return className switch
        {
            "Chrome_WidgetWin_1" => true,     // Chrome
            "MozillaWindowClass" => true,     // Firefox
            "EdgeWebWidget" => true,          // Edge
            "SlackWindowClass" => true,       // Slack
            _ => false
        };
    }

    /// <summary>Get window class name</summary>
    private string GetWindowClassName(IntPtr hwnd)
    {
        var sb = new StringBuilder(256);
        GetClassName(hwnd, sb, sb.Capacity);
        return sb.ToString();
    }
}
