using System.Diagnostics;
using System.Runtime.InteropServices;

namespace GoNhanh.Core;

/// <summary>
/// Global keyboard hook using SetWindowsHookEx(WH_KEYBOARD_LL).
/// Windows hook callback has 1000ms timeout before silent removal.
/// Rust FFI calls are &lt;0.1ms so synchronous processing is safe.
/// Implemented in Phase 2.
/// </summary>
public sealed class KeyboardHook : IDisposable
{
    private const int WH_KEYBOARD_LL = 13;
    private const int WM_KEYDOWN = 0x0100;
    private const int WM_SYSKEYDOWN = 0x0104;

    private IntPtr _hookId = IntPtr.Zero;
    private readonly LowLevelKeyboardProc _proc;
    private bool _disposed;

    /// <summary>
    /// Synchronous key event - handler MUST be fast (&lt;100ms).
    /// Rust engine FFI is &lt;0.1ms so this is safe.
    /// </summary>
    public event EventHandler<KeyEventArgs>? KeyPressed;

    public KeyboardHook()
    {
        _proc = HookCallback;
    }

    public void Install()
    {
        if (_hookId != IntPtr.Zero) return;

        using var curProcess = Process.GetCurrentProcess();
        using var curModule = curProcess.MainModule!;
        _hookId = SetWindowsHookEx(
            WH_KEYBOARD_LL,
            _proc,
            GetModuleHandle(curModule.ModuleName),
            0);

        if (_hookId == IntPtr.Zero)
            throw new InvalidOperationException($"Failed to install keyboard hook: {Marshal.GetLastWin32Error()}");
    }

    public void Uninstall()
    {
        if (_hookId == IntPtr.Zero) return;
        UnhookWindowsHookEx(_hookId);
        _hookId = IntPtr.Zero;
    }

    private IntPtr HookCallback(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0 && (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN))
        {
            var info = Marshal.PtrToStructure<KBDLLHOOKSTRUCT>(lParam);

            var args = new KeyEventArgs(
                info.vkCode,
                info.scanCode,
                info.flags,
                IsShiftPressed(),
                IsCapsLockOn());

            // Synchronous call - Rust FFI is <0.1ms, well under 1000ms timeout
            KeyPressed?.Invoke(this, args);

            if (args.Handled)
                return (IntPtr)1;
        }

        return CallNextHookEx(_hookId, nCode, wParam, lParam);
    }

    private static bool IsShiftPressed()
        => (GetAsyncKeyState(0x10) & 0x8000) != 0;

    private static bool IsCapsLockOn()
        => (GetKeyState(0x14) & 0x0001) != 0;

    public void Dispose()
    {
        if (_disposed) return;
        Uninstall();
        _disposed = true;
    }

    // P/Invoke
    private delegate IntPtr LowLevelKeyboardProc(int nCode, IntPtr wParam, IntPtr lParam);

    [StructLayout(LayoutKind.Sequential)]
    private struct KBDLLHOOKSTRUCT
    {
        public int vkCode;
        public int scanCode;
        public int flags;
        public int time;
        public IntPtr dwExtraInfo;
    }

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr SetWindowsHookEx(int idHook, LowLevelKeyboardProc lpfn, IntPtr hMod, uint dwThreadId);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool UnhookWindowsHookEx(IntPtr hhk);

    [DllImport("user32.dll")]
    private static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode)]
    private static extern IntPtr GetModuleHandle(string lpModuleName);

    [DllImport("user32.dll")]
    private static extern short GetAsyncKeyState(int vKey);

    [DllImport("user32.dll")]
    private static extern short GetKeyState(int nVirtKey);
}

public class KeyEventArgs : EventArgs
{
    public int VirtualKey { get; }
    public int ScanCode { get; }
    public int Flags { get; }
    public bool Shift { get; }
    public bool CapsLock { get; }
    public bool Handled { get; set; }

    public KeyEventArgs(int vk, int scan, int flags, bool shift, bool caps)
    {
        VirtualKey = vk;
        ScanCode = scan;
        Flags = flags;
        Shift = shift;
        CapsLock = caps;
    }
}
