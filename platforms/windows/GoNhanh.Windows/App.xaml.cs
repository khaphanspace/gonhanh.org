using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using GoNhanh.Core;
using GoNhanh.Services;
using GoNhanh.Views;

namespace GoNhanh;

/// <summary>
/// Go Nhanh Vietnamese IME for Windows.
/// Entry point and lifecycle management.
/// </summary>
public partial class App : Application
{
    private Window? _window;
    private ImeController? _controller;
    private SystemTray? _tray;
    private List<string> _logs = new();

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        try
        {
            _logs.Add("1. Starting app...");

            // Initialize Rust engine
            _logs.Add("2. Initializing Rust engine...");
            RustBridge.ime_init();
            _logs.Add("3. Rust engine initialized OK");

            // Load and apply settings
            _logs.Add("4. Loading settings...");
            SettingsService.Instance.ApplyToEngine();
            _logs.Add("5. Settings applied OK");

            // Create IME controller and start keyboard hook
            _logs.Add("6. Creating IME controller...");
            _controller = new ImeController();
            _controller.IsEnabled = SettingsService.Instance.Enabled;
            _controller.EnabledChanged += OnEnabledChanged;
            _logs.Add("7. Starting keyboard hook...");
            _controller.Start();
            _logs.Add("8. Keyboard hook started OK");

            // Create system tray
            _logs.Add("9. Creating system tray...");
            _tray = new SystemTray(_controller);
            _tray.SettingsRequested += OnSettingsRequested;
            _tray.AboutRequested += OnAboutRequested;
            _tray.UpdateRequested += OnUpdateRequested;
            _tray.QuitRequested += OnQuitRequested;
            _logs.Add("10. Showing tray icon...");
            _tray.Show();
            _logs.Add("11. Tray icon shown OK");

            // Create and SHOW settings window
            _logs.Add("12. Creating settings window...");
            _window = new SettingsWindow();
            _logs.Add("13. Activating window...");
            _window.Activate();
            _logs.Add("14. App started successfully!");
        }
        catch (Exception ex)
        {
            _logs.Add($"ERROR: {ex.Message}");
            _logs.Add($"Stack: {ex.StackTrace}");
            ShowErrorWindow($"Error: {ex.Message}\n\n{ex.StackTrace}");
        }
    }

    private void ShowErrorWindow(string message)
    {
        var logText = string.Join("\n", _logs);
        var errorWindow = new Window
        {
            Title = "Go Nhanh - Error"
        };

        var panel = new StackPanel { Margin = new Thickness(20) };
        panel.Children.Add(new TextBlock
        {
            Text = "Startup Log:",
            FontWeight = Microsoft.UI.Text.FontWeights.Bold
        });
        panel.Children.Add(new TextBlock
        {
            Text = logText,
            TextWrapping = TextWrapping.Wrap,
            Margin = new Thickness(0, 10, 0, 20)
        });
        panel.Children.Add(new TextBlock
        {
            Text = message,
            TextWrapping = TextWrapping.Wrap,
            Foreground = new Microsoft.UI.Xaml.Media.SolidColorBrush(Microsoft.UI.Colors.Red)
        });

        errorWindow.Content = panel;
        errorWindow.Activate();
        _window = errorWindow;
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

    private async void OnAboutRequested(object? sender, EventArgs e)
    {
        if (_window == null) return;
        var dialog = new AboutDialog { XamlRoot = _window.Content.XamlRoot };
        await dialog.ShowAsync();
    }

    private async void OnUpdateRequested(object? sender, EventArgs e)
    {
        if (_window == null) return;
        var dialog = new UpdateDialog { XamlRoot = _window.Content.XamlRoot };
        await dialog.ShowAsync();
    }

    private void OnQuitRequested(object? sender, EventArgs e)
    {
        Cleanup();
        Exit();
    }

    private void Cleanup()
    {
        if (_controller != null)
        {
            _controller.EnabledChanged -= OnEnabledChanged;
            _controller.Dispose();
        }

        if (_tray != null)
        {
            _tray.SettingsRequested -= OnSettingsRequested;
            _tray.AboutRequested -= OnAboutRequested;
            _tray.UpdateRequested -= OnUpdateRequested;
            _tray.QuitRequested -= OnQuitRequested;
            _tray.Dispose();
        }
    }
}
