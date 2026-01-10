using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace GoNhanh;

/// <summary>
/// Go Nhanh Vietnamese IME for Windows.
/// DEBUG MODE - Simple window to verify app runs.
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
        _window = new Window
        {
            Title = "Gõ Nhanh - DEBUG"
        };

        var panel = new StackPanel
        {
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
            Spacing = 10
        };

        panel.Children.Add(new TextBlock
        {
            Text = "Gõ Nhanh đang chạy!",
            FontSize = 24
        });

        panel.Children.Add(new TextBlock
        {
            Text = "Nếu bạn thấy cửa sổ này, app hoạt động bình thường.",
            FontSize = 14
        });

        _window.Content = panel;
        _window.Activate();
    }
}
