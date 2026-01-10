using System.Runtime.InteropServices;

namespace GoNhanh.Core;

/// <summary>
/// Main IME controller - orchestrates keyboard hook, engine, and text injection.
/// Handles Ctrl+Space toggle and per-app exclusion.
/// </summary>
public sealed class ImeController : IDisposable
{
    private readonly KeyboardHook _hook;
    private bool _enabled = true;
    private bool _disposed;

    // LLKHF_INJECTED = 0x10 - Key was injected (by SendInput etc.)
    private const int LLKHF_INJECTED = 0x10;

    // Toggle hotkey: Ctrl+Space
    private const int VK_SPACE = 0x20;
    private const int VK_CONTROL = 0x11;

    public event EventHandler<bool>? EnabledChanged;

    public bool IsEnabled
    {
        get => _enabled;
        set
        {
            if (_enabled == value) return;
            _enabled = value;
            RustBridge.ime_enabled(value);
            EnabledChanged?.Invoke(this, value);
        }
    }

    public ImeController()
    {
        _hook = new KeyboardHook();
        _hook.KeyPressed += OnKeyPressed;
    }

    public void Start()
    {
        _hook.Install();
    }

    public void Stop()
    {
        _hook.Uninstall();
    }

    private void OnKeyPressed(object? sender, KeyEventArgs e)
    {
        // Ignore injected keys (from our own SendInput) to prevent infinite loop
        if ((e.Flags & LLKHF_INJECTED) != 0)
            return;

        // Check for toggle hotkey (Ctrl+Space)
        if (e.VirtualKey == VK_SPACE && IsCtrlPressed())
        {
            IsEnabled = !IsEnabled;
            e.Handled = true;
            return;
        }

        // Skip if disabled
        if (!_enabled)
            return;

        // Skip if modifier keys pressed (Alt, Win)
        if (IsModifierPressed(e))
            return;

        // Map Windows VK to macOS keycode for Rust engine
        var keycode = KeycodeMap.FromVirtualKey(e.VirtualKey);

        // Non-IME key - check for word boundary
        if (keycode == null)
        {
            // Arrow keys, function keys - clear buffer and history
            if (IsCursorMovement(e.VirtualKey))
            {
                RustBridge.ime_clear_all();
            }
            return;
        }

        // Word boundary keys (Space, Enter, punctuation) - clear current buffer
        if (KeycodeMap.IsBreakKey(e.VirtualKey))
        {
            // Process the key through engine first (may trigger shortcut)
            ProcessKeyThroughEngine(e, keycode.Value);

            // If not handled, clear buffer after break
            if (!e.Handled)
                RustBridge.ime_clear();
            return;
        }

        // Letter/number key - process through IME engine
        ProcessKeyThroughEngine(e, keycode.Value);
    }

    private void ProcessKeyThroughEngine(KeyEventArgs e, ushort keycode)
    {
        bool caps = e.Shift || e.CapsLock;
        bool ctrl = IsCtrlPressed();

        var (action, backspace, output, consumed) = RustBridge.ProcessKey(
            keycode, caps, ctrl, e.Shift);

        switch (action)
        {
            case RustBridge.Action.Send:
            case RustBridge.Action.Restore:
                // Replace text: delete backspace chars, insert output
                if (backspace > 0 || !string.IsNullOrEmpty(output))
                {
                    TextInjector.ReplaceText(backspace, output);
                    e.Handled = true;
                }
                break;

            case RustBridge.Action.None:
            default:
                // Pass through - check if key was consumed (for shortcuts)
                e.Handled = consumed;
                break;
        }
    }

    private static bool IsCtrlPressed()
        => (GetAsyncKeyState(VK_CONTROL) & 0x8000) != 0;

    private static bool IsModifierPressed(KeyEventArgs e)
    {
        const int LLKHF_ALTDOWN = 0x20;
        return (e.Flags & LLKHF_ALTDOWN) != 0
            || (GetAsyncKeyState(0x5B) & 0x8000) != 0  // VK_LWIN
            || (GetAsyncKeyState(0x5C) & 0x8000) != 0; // VK_RWIN
    }

    private static bool IsCursorMovement(int vkCode)
    {
        return vkCode switch
        {
            0x21 => true,  // VK_PRIOR (Page Up)
            0x22 => true,  // VK_NEXT (Page Down)
            0x23 => true,  // VK_END
            0x24 => true,  // VK_HOME
            0x25 => true,  // VK_LEFT
            0x26 => true,  // VK_UP
            0x27 => true,  // VK_RIGHT
            0x28 => true,  // VK_DOWN
            _ => false
        };
    }

    public void Dispose()
    {
        if (_disposed) return;
        Stop();
        _hook.Dispose();
        _disposed = true;
    }

    [DllImport("user32.dll")]
    private static extern short GetAsyncKeyState(int vKey);
}
