using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Interop;
using GoNhanh.Core;

namespace GoNhanh.Services;

/// <summary>
/// Manages global hotkey registration for toggle shortcut
/// Matches macOS global keyboard shortcut functionality
/// </summary>
public class HotKeyService : IDisposable
{
    private const int WM_HOTKEY = 0x0312;
    private const int HOTKEY_ID = 9001;

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool RegisterHotKey(IntPtr hWnd, int id, uint fsModifiers, uint vk);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool UnregisterHotKey(IntPtr hWnd, int id);

    private IntPtr _windowHandle;
    private HwndSource? _source;
    private KeyboardShortcut _currentShortcut;
    private bool _isRegistered;

    public event Action? HotKeyPressed;

    public KeyboardShortcut CurrentShortcut => _currentShortcut;

    public HotKeyService()
    {
        _currentShortcut = KeyboardShortcut.Load();
    }

    /// <summary>
    /// Initialize hotkey service with a window handle
    /// Must be called after Application.Current.MainWindow is available
    /// </summary>
    public void Initialize(Window window)
    {
        var helper = new WindowInteropHelper(window);
        _windowHandle = helper.EnsureHandle();
        _source = HwndSource.FromHwnd(_windowHandle);
        _source?.AddHook(WndProc);

        Register(_currentShortcut);
    }

    /// <summary>
    /// Register a new hotkey
    /// </summary>
    public bool Register(KeyboardShortcut shortcut)
    {
        if (!shortcut.IsValid) return false;

        // Unregister existing hotkey
        if (_isRegistered)
        {
            UnregisterHotKey(_windowHandle, HOTKEY_ID);
            _isRegistered = false;
        }

        // Register new hotkey
        _currentShortcut = shortcut;
        _isRegistered = RegisterHotKey(_windowHandle, HOTKEY_ID, (uint)shortcut.Modifiers, shortcut.KeyCode);

        if (_isRegistered)
        {
            shortcut.Save();
        }
        else
        {
            System.Diagnostics.Debug.WriteLine($"Failed to register hotkey: {shortcut.DisplayString}");
        }

        return _isRegistered;
    }

    /// <summary>
    /// Update the toggle shortcut
    /// </summary>
    public bool UpdateShortcut(KeyboardShortcut newShortcut)
    {
        return Register(newShortcut);
    }

    private IntPtr WndProc(IntPtr hwnd, int msg, IntPtr wParam, IntPtr lParam, ref bool handled)
    {
        if (msg == WM_HOTKEY && wParam.ToInt32() == HOTKEY_ID)
        {
            HotKeyPressed?.Invoke();
            handled = true;
        }
        return IntPtr.Zero;
    }

    public void Dispose()
    {
        if (_isRegistered)
        {
            UnregisterHotKey(_windowHandle, HOTKEY_ID);
            _isRegistered = false;
        }
        _source?.RemoveHook(WndProc);
    }
}
