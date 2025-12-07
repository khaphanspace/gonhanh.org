using System.Drawing;
using System.Windows.Forms;
using GoNhanh.Core;

namespace GoNhanh.Views;

/// <summary>
/// System tray icon with context menu
/// Similar to NSStatusItem on macOS
/// </summary>
public class TrayIcon : IDisposable
{
    private NotifyIcon? _notifyIcon;
    private ContextMenuStrip? _contextMenu;
    private ToolStripMenuItem? _telexItem;
    private ToolStripMenuItem? _vniItem;
    private ToolStripMenuItem? _enabledItem;
    private bool _disposed;

    #region Events

    public event Action? OnSettingsRequested;
    public event Action? OnExitRequested;
    public event Action<InputMethod>? OnMethodChanged;
    public event Action<bool>? OnEnabledChanged;

    #endregion

    /// <summary>
    /// Initialize the system tray icon
    /// </summary>
    public void Initialize(InputMethod currentMethod, bool isEnabled)
    {
        _contextMenu = new ContextMenuStrip();
        _contextMenu.Font = new Font("Segoe UI", 9F);

        // Input method selection
        _telexItem = new ToolStripMenuItem("Telex");
        _telexItem.Click += (s, e) => SetMethod(InputMethod.Telex);

        _vniItem = new ToolStripMenuItem("VNI");
        _vniItem.Click += (s, e) => SetMethod(InputMethod.VNI);

        // Enable/Disable toggle
        _enabledItem = new ToolStripMenuItem("Enabled");
        _enabledItem.Click += (s, e) => ToggleEnabled();

        // Build menu
        _contextMenu.Items.Add(_telexItem);
        _contextMenu.Items.Add(_vniItem);
        _contextMenu.Items.Add(new ToolStripSeparator());
        _contextMenu.Items.Add(_enabledItem);
        _contextMenu.Items.Add(new ToolStripSeparator());

        var settingsItem = new ToolStripMenuItem("Settings...");
        settingsItem.Click += (s, e) => OnSettingsRequested?.Invoke();
        _contextMenu.Items.Add(settingsItem);

        var aboutItem = new ToolStripMenuItem("About GoNhanh");
        aboutItem.Click += (s, e) => ShowAbout();
        _contextMenu.Items.Add(aboutItem);

        _contextMenu.Items.Add(new ToolStripSeparator());

        var exitItem = new ToolStripMenuItem("Exit");
        exitItem.Click += (s, e) => OnExitRequested?.Invoke();
        _contextMenu.Items.Add(exitItem);

        // Create tray icon
        _notifyIcon = new NotifyIcon
        {
            Text = "GoNhanh - Vietnamese Input",
            ContextMenuStrip = _contextMenu,
            Visible = true
        };

        // Set initial icon
        UpdateIcon(isEnabled);

        // Double-click to toggle
        _notifyIcon.DoubleClick += (s, e) => ToggleEnabled();

        // Update initial state
        UpdateState(currentMethod, isEnabled);
    }

    /// <summary>
    /// Update tray icon and menu state
    /// </summary>
    public void UpdateState(InputMethod method, bool isEnabled)
    {
        if (_telexItem != null)
            _telexItem.Checked = method == InputMethod.Telex;

        if (_vniItem != null)
            _vniItem.Checked = method == InputMethod.VNI;

        if (_enabledItem != null)
            _enabledItem.Checked = isEnabled;

        UpdateIcon(isEnabled);
        UpdateTooltip(method, isEnabled);
    }

    private void SetMethod(InputMethod method)
    {
        OnMethodChanged?.Invoke(method);

        if (_telexItem != null)
            _telexItem.Checked = method == InputMethod.Telex;

        if (_vniItem != null)
            _vniItem.Checked = method == InputMethod.VNI;
    }

    private void ToggleEnabled()
    {
        bool newState = !(_enabledItem?.Checked ?? true);
        _enabledItem!.Checked = newState;
        UpdateIcon(newState);
        OnEnabledChanged?.Invoke(newState);
    }

    private void UpdateIcon(bool isEnabled)
    {
        if (_notifyIcon == null) return;

        // Use different icons for enabled/disabled state
        // For now, use system icons as placeholder
        // TODO: Replace with custom icons from Resources
        try
        {
            if (isEnabled)
            {
                _notifyIcon.Icon = CreateTextIcon("V", Color.FromArgb(37, 99, 235)); // Blue
            }
            else
            {
                _notifyIcon.Icon = CreateTextIcon("V", Color.Gray);
            }
        }
        catch
        {
            // Fallback to default icon
            _notifyIcon.Icon = SystemIcons.Application;
        }
    }

    private void UpdateTooltip(InputMethod method, bool isEnabled)
    {
        if (_notifyIcon == null) return;

        string status = isEnabled ? "ON" : "OFF";
        string methodName = method == InputMethod.Telex ? "Telex" : "VNI";
        _notifyIcon.Text = $"GoNhanh [{methodName}] - {status}";
    }

    /// <summary>
    /// Create a simple text-based icon
    /// </summary>
    private static Icon CreateTextIcon(string text, Color color)
    {
        var bitmap = new Bitmap(16, 16);
        using (var g = Graphics.FromImage(bitmap))
        {
            g.Clear(Color.Transparent);
            g.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.AntiAlias;
            g.TextRenderingHint = System.Drawing.Text.TextRenderingHint.AntiAliasGridFit;

            using var font = new Font("Segoe UI", 10, FontStyle.Bold);
            using var brush = new SolidBrush(color);

            var size = g.MeasureString(text, font);
            float x = (16 - size.Width) / 2;
            float y = (16 - size.Height) / 2;

            g.DrawString(text, font, brush, x, y);
        }

        return Icon.FromHandle(bitmap.GetHicon());
    }

    private void ShowAbout()
    {
        var about = new AboutWindow();
        about.ShowDialog();
    }

    #region IDisposable

    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (!_disposed)
        {
            if (disposing)
            {
                _notifyIcon?.Dispose();
                _contextMenu?.Dispose();
            }
            _disposed = true;
        }
    }

    ~TrayIcon()
    {
        Dispose(false);
    }

    #endregion
}
