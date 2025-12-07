using System.Diagnostics;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Windows.Forms;
using GoNhanh.Core;

namespace GoNhanh.Views;

/// <summary>
/// System tray icon with context menu
/// Matches macOS MenuBarController flow exactly
/// </summary>
public class TrayIcon : IDisposable
{
    private NotifyIcon? _notifyIcon;
    private ContextMenuStrip? _contextMenu;
    private ToolStripMenuItem? _headerItem;
    private ToolStripMenuItem? _telexItem;
    private ToolStripMenuItem? _vniItem;
    private bool _isEnabled = true;
    private InputMethod _currentMethod = InputMethod.Telex;
    private bool _disposed;

    #region Events

    public event Action? OnExitRequested;
    public event Action<InputMethod>? OnMethodChanged;
    public event Action<bool>? OnEnabledChanged;

    #endregion

    /// <summary>
    /// Initialize the system tray icon
    /// </summary>
    public void Initialize(InputMethod currentMethod, bool isEnabled)
    {
        _currentMethod = currentMethod;
        _isEnabled = isEnabled;

        _contextMenu = new ContextMenuStrip();
        _contextMenu.Font = new Font("Segoe UI", 9F);
        _contextMenu.ShowCheckMargin = true;
        _contextMenu.ShowImageMargin = false;

        // Header with toggle (like macOS)
        _headerItem = new ToolStripMenuItem();
        _headerItem.Enabled = false; // Non-clickable, just display
        _contextMenu.Items.Add(_headerItem);
        _contextMenu.Items.Add(new ToolStripSeparator());

        // Input methods with shortcuts (Ctrl+1, Ctrl+2 like macOS ⌘1, ⌘2)
        _telexItem = new ToolStripMenuItem("Telex");
        _telexItem.ShortcutKeys = Keys.Control | Keys.D1;
        _telexItem.Click += (s, e) => SetMethod(InputMethod.Telex);
        _contextMenu.Items.Add(_telexItem);

        _vniItem = new ToolStripMenuItem("VNI");
        _vniItem.ShortcutKeys = Keys.Control | Keys.D2;
        _vniItem.Click += (s, e) => SetMethod(InputMethod.VNI);
        _contextMenu.Items.Add(_vniItem);

        _contextMenu.Items.Add(new ToolStripSeparator());

        // Toggle enabled
        var toggleItem = new ToolStripMenuItem("Bật/Tắt");
        toggleItem.ShortcutKeys = Keys.Control | Keys.D0;
        toggleItem.Click += (s, e) => ToggleEnabled();
        _contextMenu.Items.Add(toggleItem);

        _contextMenu.Items.Add(new ToolStripSeparator());

        // About (like macOS "Giới thiệu GoNhanh")
        var aboutItem = new ToolStripMenuItem($"Giới thiệu {AppMetadata.Name}");
        aboutItem.Click += (s, e) => ShowAbout();
        _contextMenu.Items.Add(aboutItem);

        // Feedback/Issues (like macOS "Góp ý & Báo lỗi")
        var feedbackItem = new ToolStripMenuItem("Góp ý && Báo lỗi");
        feedbackItem.Click += (s, e) => OpenFeedback();
        _contextMenu.Items.Add(feedbackItem);

        _contextMenu.Items.Add(new ToolStripSeparator());

        // Exit (like macOS "Thoát")
        var exitItem = new ToolStripMenuItem("Thoát");
        exitItem.ShortcutKeys = Keys.Control | Keys.Q;
        exitItem.Click += (s, e) => OnExitRequested?.Invoke();
        _contextMenu.Items.Add(exitItem);

        // Create tray icon
        _notifyIcon = new NotifyIcon
        {
            ContextMenuStrip = _contextMenu,
            Visible = true
        };

        // Double-click to toggle (like macOS)
        _notifyIcon.DoubleClick += (s, e) => ToggleEnabled();

        // Update initial state
        UpdateState(currentMethod, isEnabled);
    }

    /// <summary>
    /// Update tray icon and menu state
    /// </summary>
    public void UpdateState(InputMethod method, bool isEnabled)
    {
        _currentMethod = method;
        _isEnabled = isEnabled;

        // Update header text
        if (_headerItem != null)
        {
            string status = isEnabled ? "ON" : "OFF";
            _headerItem.Text = $"{AppMetadata.Name}  [{status}]";
        }

        // Update method checkmarks
        if (_telexItem != null)
            _telexItem.Checked = method == InputMethod.Telex;

        if (_vniItem != null)
            _vniItem.Checked = method == InputMethod.VNI;

        UpdateIcon(isEnabled);
        UpdateTooltip(method, isEnabled);
    }

    private void SetMethod(InputMethod method)
    {
        _currentMethod = method;
        OnMethodChanged?.Invoke(method);

        if (_telexItem != null)
            _telexItem.Checked = method == InputMethod.Telex;

        if (_vniItem != null)
            _vniItem.Checked = method == InputMethod.VNI;
    }

    private void ToggleEnabled()
    {
        _isEnabled = !_isEnabled;
        UpdateState(_currentMethod, _isEnabled);
        OnEnabledChanged?.Invoke(_isEnabled);
    }

    private void UpdateIcon(bool isEnabled)
    {
        if (_notifyIcon == null) return;

        try
        {
            // Create icon matching macOS style:
            // White rounded rect background with "V" or "E" text
            _notifyIcon.Icon = CreateStatusIcon(isEnabled ? "V" : "E", isEnabled);
        }
        catch
        {
            _notifyIcon.Icon = SystemIcons.Application;
        }
    }

    private void UpdateTooltip(InputMethod method, bool isEnabled)
    {
        if (_notifyIcon == null) return;

        string status = isEnabled ? "Bật" : "Tắt";
        string methodName = InputMethodInfo.GetName(method);
        _notifyIcon.Text = $"{AppMetadata.Name} [{methodName}] - {status}";
    }

    /// <summary>
    /// Create status icon matching macOS style
    /// White rounded rect background with colored text
    /// </summary>
    private static Icon CreateStatusIcon(string text, bool isEnabled)
    {
        const int size = 16;
        using var bitmap = new Bitmap(size, size);

        using (var g = Graphics.FromImage(bitmap))
        {
            g.SmoothingMode = SmoothingMode.AntiAlias;
            g.TextRenderingHint = System.Drawing.Text.TextRenderingHint.AntiAliasGridFit;
            g.Clear(Color.Transparent);

            // White rounded rect background (like macOS)
            var rect = new Rectangle(0, 0, size - 1, size - 1);
            using var path = CreateRoundedRectPath(rect, 3);
            using var bgBrush = new SolidBrush(Color.White);
            g.FillPath(bgBrush, path);

            // Text color based on state
            var textColor = isEnabled
                ? Color.FromArgb(37, 99, 235)  // Blue when enabled
                : Color.FromArgb(156, 163, 175); // Gray when disabled

            using var font = new Font("Segoe UI", 9, FontStyle.Bold);
            using var brush = new SolidBrush(textColor);

            var textSize = g.MeasureString(text, font);
            float x = (size - textSize.Width) / 2;
            float y = (size - textSize.Height) / 2;

            g.DrawString(text, font, brush, x, y);
        }

        // Clone icon to own it, then dispose temp resources
        var hIcon = bitmap.GetHicon();
        using var tempIcon = Icon.FromHandle(hIcon);
        return (Icon)tempIcon.Clone();
    }

    private static GraphicsPath CreateRoundedRectPath(Rectangle rect, int radius)
    {
        var path = new GraphicsPath();
        int d = radius * 2;

        path.AddArc(rect.X, rect.Y, d, d, 180, 90);
        path.AddArc(rect.Right - d, rect.Y, d, d, 270, 90);
        path.AddArc(rect.Right - d, rect.Bottom - d, d, d, 0, 90);
        path.AddArc(rect.X, rect.Bottom - d, d, d, 90, 90);
        path.CloseFigure();

        return path;
    }

    private void ShowAbout()
    {
        var about = new AboutWindow();
        about.ShowDialog();
    }

    private void OpenFeedback()
    {
        try
        {
            Process.Start(new ProcessStartInfo
            {
                FileName = AppMetadata.IssuesUrl,
                UseShellExecute = true
            });
        }
        catch
        {
            // Ignore errors opening browser
        }
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
