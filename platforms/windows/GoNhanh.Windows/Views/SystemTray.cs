using H.NotifyIcon;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using GoNhanh.Core;
using GoNhanh.Services;

namespace GoNhanh.Views;

/// <summary>
/// System tray icon and context menu using H.NotifyIcon.WinUI.
/// </summary>
public sealed class SystemTray : IDisposable
{
    private readonly TaskbarIcon _trayIcon;
    private readonly ImeController _controller;
    private readonly ToggleMenuFlyoutItem _toggleItem;
    private readonly RadioMenuFlyoutItem _telexItem;
    private readonly RadioMenuFlyoutItem _vniItem;
    private bool _disposed;

    public event EventHandler? SettingsRequested;
    public event EventHandler? AboutRequested;
    public event EventHandler? UpdateRequested;
    public event EventHandler? QuitRequested;

    public SystemTray(ImeController controller)
    {
        _controller = controller;
        _trayIcon = new TaskbarIcon
        {
            ToolTipText = "Go Nhanh"
        };

        // Load icon from embedded resource or file
        try
        {
            var iconPath = System.IO.Path.Combine(AppContext.BaseDirectory, "Assets", "app.ico");
            if (System.IO.File.Exists(iconPath))
            {
                _trayIcon.Icon = new System.Drawing.Icon(iconPath);
            }
        }
        catch
        {
            // Icon loading failed, continue without icon
        }

        // Create menu items first
        _toggleItem = new ToggleMenuFlyoutItem
        {
            Text = "Bat/Tat Go Nhanh",
            IsChecked = _controller.IsEnabled
        };

        _telexItem = new RadioMenuFlyoutItem
        {
            Text = "Telex",
            GroupName = "InputMethod",
            IsChecked = SettingsService.Instance.InputMethod == 0
        };

        _vniItem = new RadioMenuFlyoutItem
        {
            Text = "VNI",
            GroupName = "InputMethod",
            IsChecked = SettingsService.Instance.InputMethod == 1
        };

        CreateContextMenu();

        _controller.EnabledChanged += OnEnabledChanged;
        UpdateIcon();
    }

    private void CreateContextMenu()
    {
        var menu = new MenuFlyout();

        // Toggle on/off
        _toggleItem.Click += (s, e) =>
        {
            _controller.IsEnabled = _toggleItem.IsChecked;
        };
        menu.Items.Add(_toggleItem);

        menu.Items.Add(new MenuFlyoutSeparator());

        // Input method selection
        _telexItem.Click += (s, e) => SetMethod(0);
        _vniItem.Click += (s, e) => SetMethod(1);

        menu.Items.Add(_telexItem);
        menu.Items.Add(_vniItem);

        menu.Items.Add(new MenuFlyoutSeparator());

        // Settings
        var settingsItem = new MenuFlyoutItem { Text = "Cai dat..." };
        settingsItem.Click += (s, e) => SettingsRequested?.Invoke(this, EventArgs.Empty);
        menu.Items.Add(settingsItem);

        // About
        var aboutItem = new MenuFlyoutItem { Text = "Ve Go Nhanh..." };
        aboutItem.Click += (s, e) => AboutRequested?.Invoke(this, EventArgs.Empty);
        menu.Items.Add(aboutItem);

        // Check for updates
        var updateItem = new MenuFlyoutItem { Text = "Kiem tra cap nhat" };
        updateItem.Click += (s, e) => UpdateRequested?.Invoke(this, EventArgs.Empty);
        menu.Items.Add(updateItem);

        menu.Items.Add(new MenuFlyoutSeparator());

        // Quit
        var quitItem = new MenuFlyoutItem { Text = "Thoat" };
        quitItem.Click += (s, e) => QuitRequested?.Invoke(this, EventArgs.Empty);
        menu.Items.Add(quitItem);

        _trayIcon.ContextFlyout = menu;
    }

    private void SetMethod(byte method)
    {
        RustBridge.ime_method(method);
        SettingsService.Instance.InputMethod = method;
        SettingsService.Instance.SaveAll();

        _telexItem.IsChecked = method == 0;
        _vniItem.IsChecked = method == 1;
    }

    private void OnEnabledChanged(object? sender, bool enabled)
    {
        _toggleItem.IsChecked = enabled;
        UpdateIcon();
    }

    private void UpdateIcon()
    {
        _trayIcon.ToolTipText = _controller.IsEnabled
            ? "Go Nhanh - Dang bat"
            : "Go Nhanh - Da tat";
    }

    public void Show() => _trayIcon.ForceCreate();

    public void Dispose()
    {
        if (_disposed) return;
        _controller.EnabledChanged -= OnEnabledChanged;
        _trayIcon.Dispose();
        _disposed = true;
    }
}
