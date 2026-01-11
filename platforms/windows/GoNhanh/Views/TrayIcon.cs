using System;
using System.Drawing;
using System.Windows;
using System.Windows.Forms;
using GoNhanh.Core;
using GoNhanh.Services;
using Application = System.Windows.Application;

namespace GoNhanh.Views;

/// <summary>
/// System tray icon with context menu
/// </summary>
public sealed class TrayIcon : IDisposable
{
    private readonly NotifyIcon _notifyIcon;
    private readonly ImeService _imeService;
    private bool _disposed;

    // Menu items that need updates
    private ToolStripMenuItem _toggleItem;
    private ToolStripMenuItem _telexItem;
    private ToolStripMenuItem _vniItem;

    public TrayIcon(ImeService imeService)
    {
        _imeService = imeService;

        _notifyIcon = new NotifyIcon
        {
            Text = "Gõ Nhanh",
            Visible = false,
            Icon = LoadIcon()
        };

        _notifyIcon.ContextMenuStrip = CreateContextMenu();
        _notifyIcon.DoubleClick += (s, e) => ShowSettings();
    }

    private Icon LoadIcon()
    {
        // Try to load from resources, fall back to system icon
        try
        {
            var uri = new Uri("pack://application:,,,/Resources/Icons/app.ico");
            using var stream = Application.GetResourceStream(uri)?.Stream;
            if (stream != null)
            {
                return new Icon(stream);
            }
        }
        catch { }

        // Fallback to system icon
        return SystemIcons.Application;
    }

    private ContextMenuStrip CreateContextMenu()
    {
        var menu = new ContextMenuStrip();

        // Toggle on/off
        _toggleItem = new ToolStripMenuItem("Bật/Tắt (Ctrl+Shift+Space)")
        {
            Checked = _imeService.IsEnabled
        };
        _toggleItem.Click += (s, e) => ToggleEnabled();
        menu.Items.Add(_toggleItem);

        menu.Items.Add(new ToolStripSeparator());

        // Input method selection
        _telexItem = new ToolStripMenuItem("Telex")
        {
            Checked = _imeService.Method == InputMethod.Telex
        };
        _telexItem.Click += (s, e) => SetMethod(InputMethod.Telex);

        _vniItem = new ToolStripMenuItem("VNI")
        {
            Checked = _imeService.Method == InputMethod.VNI
        };
        _vniItem.Click += (s, e) => SetMethod(InputMethod.VNI);

        menu.Items.Add(_telexItem);
        menu.Items.Add(_vniItem);

        menu.Items.Add(new ToolStripSeparator());

        // Settings
        var settingsItem = new ToolStripMenuItem("Cài đặt...");
        settingsItem.Click += (s, e) => ShowSettings();
        menu.Items.Add(settingsItem);

        // About
        var aboutItem = new ToolStripMenuItem("Về Gõ Nhanh");
        aboutItem.Click += (s, e) => ShowAbout();
        menu.Items.Add(aboutItem);

        menu.Items.Add(new ToolStripSeparator());

        // Exit
        var exitItem = new ToolStripMenuItem("Thoát");
        exitItem.Click += (s, e) => Application.Current.Shutdown();
        menu.Items.Add(exitItem);

        return menu;
    }

    public void Show()
    {
        _notifyIcon.Visible = true;
    }

    public void Hide()
    {
        _notifyIcon.Visible = false;
    }

    private void ToggleEnabled()
    {
        _imeService.IsEnabled = !_imeService.IsEnabled;
        _toggleItem.Checked = _imeService.IsEnabled;
        UpdateTooltip();
    }

    private void SetMethod(InputMethod method)
    {
        _imeService.SetMethod(method);
        _telexItem.Checked = method == InputMethod.Telex;
        _vniItem.Checked = method == InputMethod.VNI;

        // Save to settings
        var settings = SettingsService.Instance.Load();
        settings.Method = method;
        SettingsService.Instance.Save(settings);

        UpdateTooltip();
    }

    private void UpdateTooltip()
    {
        var status = _imeService.IsEnabled ? "Bật" : "Tắt";
        var method = _imeService.Method == InputMethod.Telex ? "Telex" : "VNI";
        _notifyIcon.Text = $"Gõ Nhanh - {method} ({status})";
    }

    private void ShowSettings()
    {
        var settingsWindow = Application.Current.Windows
            .OfType<SettingsWindow>()
            .FirstOrDefault();

        if (settingsWindow == null)
        {
            settingsWindow = new SettingsWindow(_imeService);
            settingsWindow.Show();
        }
        else
        {
            settingsWindow.Activate();
        }
    }

    private void ShowAbout()
    {
        var aboutWindow = Application.Current.Windows
            .OfType<AboutWindow>()
            .FirstOrDefault();

        if (aboutWindow == null)
        {
            aboutWindow = new AboutWindow();
            aboutWindow.Show();
        }
        else
        {
            aboutWindow.Activate();
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _notifyIcon.Visible = false;
        _notifyIcon.Dispose();
        _disposed = true;
    }
}
