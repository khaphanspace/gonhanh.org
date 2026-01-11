using GoNhanh.Core;

namespace GoNhanh.Services;

/// <summary>
/// Orchestrates keyboard hook, Rust engine, and text injection
/// </summary>
public sealed class ImeService : IDisposable
{
    private readonly KeyboardHook _hook;
    private readonly ImeEngine _engine;
    private readonly TextSender _sender;
    private bool _enabled = true;
    private bool _disposed;

    public bool IsEnabled
    {
        get => _enabled;
        set
        {
            _enabled = value;
            _engine.SetEnabled(value);
        }
    }

    public InputMethod Method { get; private set; } = InputMethod.Telex;

    public ImeService()
    {
        _engine = ImeEngine.Instance;
        _hook = new KeyboardHook();
        _sender = new TextSender();

        _hook.KeyDown += OnKeyDown;
    }

    public void Start()
    {
        _engine.Initialize();
        _engine.SetEnabled(_enabled);
        _hook.Install();
    }

    public void Stop()
    {
        _hook.Uninstall();
    }

    public void SetMethod(InputMethod method)
    {
        Method = method;
        _engine.SetMethod(method);
    }

    public void ApplySettings(SettingsData settings)
    {
        _engine.SetMethod(settings.Method);
        _engine.SetEnabled(settings.Enabled);
        _engine.SetSkipWShortcut(!settings.WShortcut);
        _engine.SetBracketShortcut(settings.BracketShortcut);
        _engine.SetEscRestore(settings.EscRestore);
        _engine.SetFreeTone(settings.FreeTone);
        _engine.SetModernTone(settings.ModernTone);
        _engine.SetEnglishAutoRestore(settings.EnglishAutoRestore);
        _engine.SetAutoCapitalize(settings.AutoCapitalize);

        // Load shortcuts
        _engine.ClearShortcuts();
        foreach (var shortcut in settings.Shortcuts)
        {
            _engine.AddShortcut(shortcut.Trigger, shortcut.Replacement);
        }
    }

    private void OnKeyDown(object? sender, KeyEventArgs e)
    {
        if (!_enabled) return;

        // Skip if modifier held (Ctrl+C, Alt+Tab, etc.)
        if (e.Ctrl || e.Alt) return;

        // Clear buffer on cursor movement
        if (KeyCodes.IsCursorKey(e.VirtualKey))
        {
            _engine.ClearAll();
            return;
        }

        // Process through Rust engine
        bool caps = e.CapsLock ^ e.Shift; // Effective caps state
        var result = _engine.ProcessKey(e.MacKeycode, caps, false, e.Shift);

        if (result == null) return;

        // Handle result
        switch ((ImeAction)result.Value.action)
        {
            case ImeAction.Send:
                e.Handled = true; // Block original key
                string text = result.Value.GetText();
                _sender.SendReplace(result.Value.backspace, text);

                // Clear on word boundary
                if (KeyCodes.IsWordBoundary(e.VirtualKey))
                {
                    _engine.Clear();
                }
                break;

            case ImeAction.Restore:
                e.Handled = true;
                string restored = result.Value.GetText();
                _sender.SendReplace(result.Value.backspace, restored);
                _engine.ClearAll();
                break;

            case ImeAction.None:
            default:
                // Pass through unchanged
                if (KeyCodes.IsWordBoundary(e.VirtualKey))
                {
                    _engine.Clear();
                }
                break;
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _hook.Dispose();
        _engine.Dispose();
        _disposed = true;
    }
}
