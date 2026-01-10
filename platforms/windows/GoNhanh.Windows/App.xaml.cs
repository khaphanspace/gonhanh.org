using Microsoft.UI.Xaml;
using GoNhanh.Core;

namespace GoNhanh;

/// <summary>
/// Gõ Nhanh Vietnamese IME for Windows.
/// Entry point and lifecycle management.
/// </summary>
public partial class App : Application
{
    private Window? _window;

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        // Initialize Rust engine
        RustBridge.ime_init();
        RustBridge.ime_method(0); // Default to Telex

        // Create main window (will be hidden, tray app)
        _window = new MainWindow();
        _window.Activate();
    }
}
