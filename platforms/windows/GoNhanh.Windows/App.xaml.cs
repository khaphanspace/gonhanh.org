using Microsoft.UI.Xaml;
using GoNhanh.Core;
using GoNhanh.Services;
using GoNhanh.Views;

namespace GoNhanh;

/// <summary>
/// Gõ Nhanh Vietnamese IME for Windows.
/// Entry point and lifecycle management.
/// </summary>
public partial class App : Application
{
    private Window? _window;
    private ImeController? _controller;
    private SystemTray? _tray;

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        // Initialize Rust engine
        RustBridge.ime_init();

        // Load and apply settings
        SettingsService.Instance.ApplyToEngine();

        // Create IME controller and start keyboard hook
        _controller = new ImeController();
        _controller.IsEnabled = SettingsService.Instance.Enabled;
        _controller.EnabledChanged += OnEnabledChanged;
        _controller.Start();

        // Create system tray
        _tray = new SystemTray(_controller);
        _tray.SettingsRequested += OnSettingsRequested;
        _tray.AboutRequested += OnAboutRequested;
        _tray.QuitRequested += OnQuitRequested;
        _tray.Show();

        // Create main window (hidden by default, shown when settings requested)
        _window = new MainWindow();
    }

    private void OnEnabledChanged(object? sender, bool enabled)
    {
        SettingsService.Instance.Enabled = enabled;
        SettingsService.Instance.SaveAll();
    }

    private void OnSettingsRequested(object? sender, EventArgs e)
    {
        _window?.Activate();
    }

    private void OnAboutRequested(object? sender, EventArgs e)
    {
        // TODO: Show about dialog (Phase 4)
    }

    private void OnQuitRequested(object? sender, EventArgs e)
    {
        Cleanup();
        Exit();
    }

    private void Cleanup()
    {
        // Unsubscribe event handlers to prevent memory leaks
        if (_controller != null)
        {
            _controller.EnabledChanged -= OnEnabledChanged;
            _controller.Dispose();
        }

        if (_tray != null)
        {
            _tray.SettingsRequested -= OnSettingsRequested;
            _tray.AboutRequested -= OnAboutRequested;
            _tray.QuitRequested -= OnQuitRequested;
            _tray.Dispose();
        }
    }
}
