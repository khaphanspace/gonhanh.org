using System.Diagnostics;
using System.Media;
using System.Runtime.InteropServices;
using System.Windows;
using GoNhanh.Core;
using GoNhanh.Services;
using GoNhanh.Views;

namespace GoNhanh;

/// <summary>
/// GoNhanh - Vietnamese Input Method for Windows
/// Main application entry point
/// Matches macOS App.swift flow with Per-App Mode support
/// </summary>
public partial class App : System.Windows.Application
{
    private TrayIcon? _trayIcon;
    private KeyboardHook? _keyboardHook;
    private HotKeyService? _hotKeyService;
    private readonly SettingsService _settings = new();
    private readonly ShortcutsService _shortcuts = new();
    private System.Threading.Mutex? _mutex;
    private Window? _hiddenWindow; // For hotkey registration

    // Per-app mode monitoring
    private System.Threading.Timer? _appMonitorTimer;
    private string _lastProcessName = "";

    #region Win32 Imports for Per-App Mode

    [DllImport("user32.dll")]
    private static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);

    #endregion

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
        _settings.ApplyToEngine();

        // Load shortcuts
        _shortcuts.Load();

        // Initialize keyboard hook
        _keyboardHook = new KeyboardHook();
        _keyboardHook.KeyPressed += OnKeyPressed;
        _keyboardHook.Start();

        // Initialize global hotkey service
        InitializeHotKeyService();

        // Initialize system tray
        _trayIcon = new TrayIcon();
        _trayIcon.OnExitRequested += ExitApplication;
        _trayIcon.OnMethodChanged += ChangeInputMethod;
        _trayIcon.OnEnabledChanged += ToggleEnabled;
        _trayIcon.OnSettingsRequested += ShowSettings;
        _trayIcon.Initialize(_settings.CurrentMethod, _settings.IsEnabled);

        // Start per-app mode monitoring
        if (_settings.PerAppModeEnabled)
        {
            StartAppMonitoring();
        }

        // Show onboarding if first run (like macOS)
        if (_settings.IsFirstRun)
        {
            ShowOnboarding();
        }
    }

    private bool EnsureSingleInstance()
    {
        _mutex = new System.Threading.Mutex(true, "GoNhanh_SingleInstance", out bool createdNew);
        if (!createdNew)
        {
            System.Windows.MessageBox.Show(
                $"{AppMetadata.Name} đang chạy.\nKiểm tra khay hệ thống (system tray).",
                AppMetadata.Name,
                MessageBoxButton.OK,
                MessageBoxImage.Information);
            return false;
        }
        return true;
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

    #region Global Hotkey

    private void InitializeHotKeyService()
    {
        try
        {
            // Create hidden window for hotkey registration
            _hiddenWindow = new Window
            {
                Width = 0,
                Height = 0,
                WindowStyle = WindowStyle.None,
                ShowInTaskbar = false,
                ShowActivated = false
            };
            _hiddenWindow.Show();
            _hiddenWindow.Hide();

            _hotKeyService = new HotKeyService();
            _hotKeyService.HotKeyPressed += OnHotKeyPressed;
            _hotKeyService.Initialize(_hiddenWindow);
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Failed to initialize hotkey service: {ex.Message}");
        }
    }

    private void OnHotKeyPressed()
    {
        Dispatcher.Invoke(() =>
        {
            var newState = !_settings.IsEnabled;
            ToggleEnabled(newState);
            _trayIcon?.UpdateState(_settings.CurrentMethod, newState);
        });
    }

    #endregion

    #region Per-App Mode

    private void StartAppMonitoring()
    {
        // Check foreground app every 500ms
        _appMonitorTimer = new System.Threading.Timer(
            CheckForegroundApp,
            null,
            TimeSpan.FromMilliseconds(500),
            TimeSpan.FromMilliseconds(500));
    }

    private void StopAppMonitoring()
    {
        _appMonitorTimer?.Dispose();
        _appMonitorTimer = null;
    }

    private void CheckForegroundApp(object? state)
    {
        if (!_settings.PerAppModeEnabled) return;

        try
        {
            var hwnd = GetForegroundWindow();
            if (hwnd == IntPtr.Zero) return;

            GetWindowThreadProcessId(hwnd, out uint processId);
            if (processId == 0) return;

            var process = Process.GetProcessById((int)processId);
            var processName = process.ProcessName;

            if (processName != _lastProcessName)
            {
                _lastProcessName = processName;

                // Get per-app mode state
                bool shouldEnable = _settings.GetPerAppMode(processName);

                // Update state if different
                if (shouldEnable != _settings.IsEnabled)
                {
                    Dispatcher.Invoke(() =>
                    {
                        SetEnabledSilently(shouldEnable);
                    });
                }
            }
        }
        catch
        {
            // Ignore errors during monitoring
        }
    }

    /// <summary>
    /// Set enabled state without triggering per-app save
    /// Used when switching apps with per-app mode
    /// </summary>
    private void SetEnabledSilently(bool enabled)
    {
        _settings.IsEnabled = enabled;
        RustBridge.SetEnabled(enabled);
        _trayIcon?.UpdateState(_settings.CurrentMethod, enabled);
    }

    #endregion

    #region Sound

    private void PlayToggleSound(bool enabled)
    {
        if (!_settings.SoundEnabled) return;

        try
        {
            // Use Windows system sounds
            if (enabled)
            {
                SystemSounds.Asterisk.Play();
            }
            else
            {
                SystemSounds.Exclamation.Play();
            }
        }
        catch
        {
            // Ignore sound errors
        }
    }

    #endregion

    private void ShowOnboarding()
    {
        var onboarding = new OnboardingWindow(_settings);
        onboarding.ShowDialog();

        // Save settings after onboarding
        _settings.IsFirstRun = false;
        _settings.Save();

        _settings.ApplyToEngine();
        _trayIcon?.UpdateState(_settings.CurrentMethod, _settings.IsEnabled);
    }

    private void ShowSettings()
    {
        var settings = new SettingsWindow(_settings, _shortcuts, _hotKeyService);
        settings.SettingsChanged += () =>
        {
            _settings.ApplyToEngine();
            _trayIcon?.UpdateState(_settings.CurrentMethod, _settings.IsEnabled);

            // Update per-app monitoring
            if (_settings.PerAppModeEnabled && _appMonitorTimer == null)
            {
                StartAppMonitoring();
            }
            else if (!_settings.PerAppModeEnabled && _appMonitorTimer != null)
            {
                StopAppMonitoring();
            }
        };
        settings.ShowDialog();
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
        RustBridge.SetEnabled(enabled);

        // Play sound
        PlayToggleSound(enabled);

        // Save per-app mode if enabled
        if (_settings.PerAppModeEnabled && !string.IsNullOrEmpty(_lastProcessName))
        {
            _settings.SavePerAppMode(_lastProcessName, enabled);
        }

        _settings.Save();
    }

    private void ExitApplication()
    {
        StopAppMonitoring();
        _keyboardHook?.Stop();
        _keyboardHook?.Dispose();
        _hotKeyService?.Dispose();
        _hiddenWindow?.Close();
        _trayIcon?.Dispose();
        RustBridge.Clear();
        _mutex?.Dispose();
        Shutdown();
    }

    protected override void OnExit(ExitEventArgs e)
    {
        StopAppMonitoring();
        _keyboardHook?.Dispose();
        _hotKeyService?.Dispose();
        _hiddenWindow?.Close();
        _trayIcon?.Dispose();
        _mutex?.Dispose();
        base.OnExit(e);
    }
}
