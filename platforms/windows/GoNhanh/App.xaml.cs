using System.Windows;
using GoNhanh.Core;
using GoNhanh.Services;
using GoNhanh.Views;

namespace GoNhanh;

/// <summary>
/// GoNhanh - Vietnamese Input Method for Windows
/// Main application entry point
/// </summary>
public partial class App : Application
{
    private TrayIcon? _trayIcon;
    private KeyboardHook? _keyboardHook;
    private readonly SettingsService _settings = new();

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        // Prevent multiple instances
        if (!EnsureSingleInstance())
        {
            Shutdown();
            return;
        }

        // Initialize Rust core engine
        RustBridge.Initialize();

        // Load settings
        _settings.Load();
        ApplySettings();

        // Initialize keyboard hook
        _keyboardHook = new KeyboardHook();
        _keyboardHook.KeyPressed += OnKeyPressed;
        _keyboardHook.Start();

        // Initialize system tray
        _trayIcon = new TrayIcon();
        _trayIcon.OnSettingsRequested += ShowSettings;
        _trayIcon.OnExitRequested += ExitApplication;
        _trayIcon.OnMethodChanged += ChangeInputMethod;
        _trayIcon.OnEnabledChanged += ToggleEnabled;
        _trayIcon.Initialize(_settings.CurrentMethod, _settings.IsEnabled);

        // Show onboarding if first run
        if (_settings.IsFirstRun)
        {
            ShowOnboarding();
            _settings.IsFirstRun = false;
            _settings.Save();
        }
    }

    private bool EnsureSingleInstance()
    {
        // Use mutex to ensure single instance
        bool createdNew;
        var mutex = new System.Threading.Mutex(true, "GoNhanh_SingleInstance", out createdNew);
        if (!createdNew)
        {
            MessageBox.Show(
                "GoNhanh is already running.\nCheck the system tray.",
                "GoNhanh",
                MessageBoxButton.OK,
                MessageBoxImage.Information);
            return false;
        }
        return true;
    }

    private void ApplySettings()
    {
        RustBridge.SetMethod(_settings.CurrentMethod);
        RustBridge.SetEnabled(_settings.IsEnabled);
        RustBridge.SetModernTone(_settings.UseModernTone);
    }

    private void OnKeyPressed(object? sender, KeyPressedEventArgs e)
    {
        if (!_settings.IsEnabled) return;

        var result = RustBridge.ProcessKey(e.VirtualKeyCode, e.Shift, e.CapsLock);

        if (result.Action == ImeAction.Send && result.Count > 0)
        {
            e.Handled = true;
            TextSender.SendText(result.GetText(), result.Backspace);
        }
        else if (result.Action == ImeAction.Restore)
        {
            e.Handled = true;
            TextSender.SendText(result.GetText(), result.Backspace);
        }
    }

    private void ShowSettings()
    {
        var settingsWindow = new SettingsWindow(_settings);
        settingsWindow.SettingsChanged += () =>
        {
            ApplySettings();
            _trayIcon?.UpdateState(_settings.CurrentMethod, _settings.IsEnabled);
        };
        settingsWindow.ShowDialog();
    }

    private void ShowOnboarding()
    {
        var onboarding = new OnboardingWindow(_settings);
        onboarding.ShowDialog();
        ApplySettings();
        _trayIcon?.UpdateState(_settings.CurrentMethod, _settings.IsEnabled);
    }

    private void ChangeInputMethod(InputMethod method)
    {
        _settings.CurrentMethod = method;
        _settings.Save();
        RustBridge.SetMethod(method);
    }

    private void ToggleEnabled(bool enabled)
    {
        _settings.IsEnabled = enabled;
        _settings.Save();
        RustBridge.SetEnabled(enabled);
    }

    private void ExitApplication()
    {
        _keyboardHook?.Stop();
        _keyboardHook?.Dispose();
        _trayIcon?.Dispose();
        RustBridge.Clear();
        Shutdown();
    }

    protected override void OnExit(ExitEventArgs e)
    {
        _keyboardHook?.Dispose();
        _trayIcon?.Dispose();
        base.OnExit(e);
    }
}
