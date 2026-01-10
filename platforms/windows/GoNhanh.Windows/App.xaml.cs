using Microsoft.UI.Xaml;
using GoNhanh.Views;

namespace GoNhanh;

/// <summary>
/// Minimal app for testing XAML compilation.
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
        _window = new SettingsWindow();
        _window.Activate();
    }
}
