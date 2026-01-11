using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.ComponentModel;

namespace GoNhanh.Core;

/// <summary>
/// Global low-level keyboard hook using SetWindowsHookEx
/// </summary>
public sealed class KeyboardHook : IDisposable
{
    // Win32 constants
    private const int WH_KEYBOARD_LL = 13;
    private const int WM_KEYDOWN = 0x0100;
    private const int WM_KEYUP = 0x0101;
    private const int WM_SYSKEYDOWN = 0x0104;
    private const int WM_SYSKEYUP = 0x0105;

    // Flag to identify our injected keys ("GNH" in hex)
    private static readonly UIntPtr INJECTED_FLAG = new UIntPtr(0x47_4E_48);

    [StructLayout(LayoutKind.Sequential)]
    private struct KBDLLHOOKSTRUCT
    {
        public uint vkCode;
        public uint scanCode;
        public uint flags;
        public uint time;
        public UIntPtr dwExtraInfo;
    }

    // P/Invoke declarations
    private delegate IntPtr LowLevelKeyboardProc(int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr SetWindowsHookEx(int idHook, LowLevelKeyboardProc lpfn,
        IntPtr hMod, uint dwThreadId);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool UnhookWindowsHookEx(IntPtr hhk);

    [DllImport("user32.dll")]
    private static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("kernel32.dll")]
    private static extern IntPtr GetModuleHandle(string? lpModuleName);

    [DllImport("user32.dll")]
    private static extern short GetKeyState(int nVirtKey);

    // Instance fields
    private IntPtr _hookHandle;
    private readonly LowLevelKeyboardProc _hookProc; // MUST keep reference to prevent GC
    private bool _disposed;

    // Events
    public event EventHandler<KeyEventArgs>? KeyDown;
    public event EventHandler<KeyEventArgs>? KeyUp;

    public KeyboardHook()
    {
        _hookProc = HookCallback; // Keep delegate alive
    }

    /// <summary>Install the keyboard hook</summary>
    public void Install()
    {
        if (_hookHandle != IntPtr.Zero) return;

        using var process = Process.GetCurrentProcess();
        using var module = process.MainModule;

        _hookHandle = SetWindowsHookEx(
            WH_KEYBOARD_LL,
            _hookProc,
            GetModuleHandle(module?.ModuleName),
            0);

        if (_hookHandle == IntPtr.Zero)
        {
            int error = Marshal.GetLastWin32Error();
            throw new Win32Exception(error, $"Failed to install keyboard hook. Error: {error}");
        }
    }

    /// <summary>Uninstall the keyboard hook</summary>
    public void Uninstall()
    {
        if (_hookHandle == IntPtr.Zero) return;

        UnhookWindowsHookEx(_hookHandle);
        _hookHandle = IntPtr.Zero;
    }

    private IntPtr HookCallback(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0)
        {
            var kbd = Marshal.PtrToStructure<KBDLLHOOKSTRUCT>(lParam);
            int msg = (int)wParam;

            // Skip our own injected keys
            if (kbd.dwExtraInfo == INJECTED_FLAG)
            {
                return CallNextHookEx(_hookHandle, nCode, wParam, lParam);
            }

            // Get modifier state
            bool shift = (GetKeyState(KeyCodes.VK_SHIFT) & 0x8000) != 0;
            bool ctrl = (GetKeyState(KeyCodes.VK_CONTROL) & 0x8000) != 0;
            bool alt = (GetKeyState(KeyCodes.VK_MENU) & 0x8000) != 0;
            bool caps = (GetKeyState(KeyCodes.VK_CAPITAL) & 0x0001) != 0;

            var args = new KeyEventArgs
            {
                VirtualKey = (int)kbd.vkCode,
                ScanCode = (int)kbd.scanCode,
                Shift = shift,
                Ctrl = ctrl,
                Alt = alt,
                CapsLock = caps,
                MacKeycode = KeyCodes.ToMacKeycode((int)kbd.vkCode)
            };

            bool isKeyDown = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;

            if (isKeyDown)
            {
                KeyDown?.Invoke(this, args);
            }
            else
            {
                KeyUp?.Invoke(this, args);
            }

            // If handled (blocked), return 1 to prevent propagation
            if (args.Handled)
            {
                return (IntPtr)1;
            }
        }

        return CallNextHookEx(_hookHandle, nCode, wParam, lParam);
    }

    /// <summary>Check if a key is from our injection</summary>
    public static UIntPtr InjectedFlag => INJECTED_FLAG;

    public void Dispose()
    {
        if (_disposed) return;
        Uninstall();
        _disposed = true;
    }
}

/// <summary>Key event arguments</summary>
public class KeyEventArgs : EventArgs
{
    public int VirtualKey { get; set; }
    public int ScanCode { get; set; }
    public ushort MacKeycode { get; set; }
    public bool Shift { get; set; }
    public bool Ctrl { get; set; }
    public bool Alt { get; set; }
    public bool CapsLock { get; set; }

    /// <summary>Set to true to block the key from propagating</summary>
    public bool Handled { get; set; }
}
