using Microsoft.UI.Xaml;

namespace GoNhanh;

/// <summary>
/// Main window - hidden, app runs in system tray.
/// </summary>
public sealed partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();

        // Hide window on startup (tray-only app)
        // Will be implemented in Phase 3 with system tray
    }
}
