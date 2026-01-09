using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using GoNhanh.Core;

namespace GoNhanh.Views;

public partial class ShortcutRecorderControl : System.Windows.Controls.UserControl
{
    private bool _isRecording;
    private KeyboardShortcut _shortcut = KeyboardShortcut.Default;

    public event Action<KeyboardShortcut>? ShortcutChanged;

    public KeyboardShortcut Shortcut
    {
        get => _shortcut;
        set
        {
            _shortcut = value;
            UpdateDisplay();
        }
    }

    public ShortcutRecorderControl()
    {
        InitializeComponent();
        UpdateDisplay();

        // Handle key events when recording
        PreviewKeyDown += OnPreviewKeyDown;
        LostFocus += OnLostFocus;
    }

    private void Border_MouseDown(object sender, MouseButtonEventArgs e)
    {
        if (_isRecording)
        {
            StopRecording();
        }
        else
        {
            StartRecording();
        }
    }

    private void StartRecording()
    {
        _isRecording = true;
        RecorderBorder.BorderBrush = new SolidColorBrush(global::System.Windows.Media.Color.FromRgb(59, 130, 246)); // Blue
        ShortcutText.Visibility = Visibility.Collapsed;
        RecordingText.Visibility = Visibility.Visible;
        Focusable = true;
        Focus();
    }

    private void StopRecording()
    {
        _isRecording = false;
        RecorderBorder.BorderBrush = new SolidColorBrush(global::System.Windows.Media.Color.FromRgb(209, 213, 219)); // Gray
        ShortcutText.Visibility = Visibility.Visible;
        RecordingText.Visibility = Visibility.Collapsed;
        Focusable = false;
    }

    private void OnPreviewKeyDown(object sender, System.Windows.Input.KeyEventArgs e)
    {
        if (!_isRecording) return;

        e.Handled = true;

        // Get modifiers
        var modifiers = Core.ModifierKeys.None;
        if (Keyboard.IsKeyDown(Key.LeftCtrl) || Keyboard.IsKeyDown(Key.RightCtrl))
            modifiers |= Core.ModifierKeys.Control;
        if (Keyboard.IsKeyDown(Key.LeftAlt) || Keyboard.IsKeyDown(Key.RightAlt))
            modifiers |= Core.ModifierKeys.Alt;
        if (Keyboard.IsKeyDown(Key.LeftShift) || Keyboard.IsKeyDown(Key.RightShift))
            modifiers |= Core.ModifierKeys.Shift;
        if (Keyboard.IsKeyDown(Key.LWin) || Keyboard.IsKeyDown(Key.RWin))
            modifiers |= Core.ModifierKeys.Win;

        // Escape cancels recording
        if (e.Key == Key.Escape)
        {
            StopRecording();
            return;
        }

        // Get the actual key (ignore modifier-only presses)
        var key = e.Key == Key.System ? e.SystemKey : e.Key;

        // Skip pure modifier keys
        if (key is Key.LeftCtrl or Key.RightCtrl or
            Key.LeftAlt or Key.RightAlt or
            Key.LeftShift or Key.RightShift or
            Key.LWin or Key.RWin)
        {
            return;
        }

        // Require at least one modifier (except for F keys)
        if (modifiers == Core.ModifierKeys.None && !IsFunctionKey(key))
        {
            return;
        }

        // Create new shortcut
        var virtualKey = KeyInterop.VirtualKeyFromKey(key);
        var newShortcut = new KeyboardShortcut
        {
            KeyCode = (uint)virtualKey,
            Modifiers = modifiers
        };

        _shortcut = newShortcut;
        UpdateDisplay();
        StopRecording();
        ShortcutChanged?.Invoke(newShortcut);
    }

    private void OnLostFocus(object sender, RoutedEventArgs e)
    {
        if (_isRecording)
        {
            StopRecording();
        }
    }

    private void UpdateDisplay()
    {
        ShortcutText.Text = _shortcut.DisplayString;
    }

    private static bool IsFunctionKey(Key key)
    {
        return key >= Key.F1 && key <= Key.F24;
    }
}
