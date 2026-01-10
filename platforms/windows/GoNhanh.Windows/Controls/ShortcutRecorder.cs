using System.Runtime.InteropServices;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Windows.System;

namespace GoNhanh.Controls;

/// <summary>
/// Control for recording keyboard shortcuts.
/// Usage: Click to start recording, press key combo, records automatically.
/// </summary>
public sealed class ShortcutRecorder : TextBox
{
    private bool _isRecording;

    public VirtualKey RecordedKey { get; private set; }
    public VirtualKeyModifiers RecordedModifiers { get; private set; }

    public event EventHandler<ShortcutRecordedEventArgs>? ShortcutRecorded;

    public ShortcutRecorder()
    {
        IsReadOnly = true;
        PlaceholderText = "Nhan de ghi phim tat...";
        GotFocus += OnGotFocus;
        LostFocus += OnLostFocus;
        KeyDown += OnKeyDown;
    }

    public void SetShortcut(VirtualKey key, VirtualKeyModifiers modifiers)
    {
        RecordedKey = key;
        RecordedModifiers = modifiers;
        UpdateDisplay();
    }

    private void OnGotFocus(object sender, RoutedEventArgs e)
    {
        _isRecording = true;
        Text = "Nhan phim tat...";
    }

    private void OnLostFocus(object sender, RoutedEventArgs e)
    {
        _isRecording = false;
        UpdateDisplay();
    }

    private void OnKeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (!_isRecording) return;

        // Ignore modifier-only keys
        if (IsModifierKey(e.Key)) return;

        RecordedKey = e.Key;
        RecordedModifiers = GetCurrentModifiers();

        _isRecording = false;
        UpdateDisplay();

        ShortcutRecorded?.Invoke(this, new ShortcutRecordedEventArgs(RecordedKey, RecordedModifiers));
        e.Handled = true;
    }

    private void UpdateDisplay()
    {
        if (RecordedKey == VirtualKey.None)
        {
            Text = "";
            return;
        }

        var parts = new List<string>();

        if (RecordedModifiers.HasFlag(VirtualKeyModifiers.Control))
            parts.Add("Ctrl");
        if (RecordedModifiers.HasFlag(VirtualKeyModifiers.Menu))
            parts.Add("Alt");
        if (RecordedModifiers.HasFlag(VirtualKeyModifiers.Shift))
            parts.Add("Shift");
        if (RecordedModifiers.HasFlag(VirtualKeyModifiers.Windows))
            parts.Add("Win");

        parts.Add(KeyToString(RecordedKey));

        Text = string.Join("+", parts);
    }

    private static bool IsModifierKey(VirtualKey key)
    {
        return key is VirtualKey.Control or VirtualKey.LeftControl or VirtualKey.RightControl
            or VirtualKey.Menu or VirtualKey.LeftMenu or VirtualKey.RightMenu
            or VirtualKey.Shift or VirtualKey.LeftShift or VirtualKey.RightShift
            or VirtualKey.LeftWindows or VirtualKey.RightWindows;
    }

    private static VirtualKeyModifiers GetCurrentModifiers()
    {
        var mods = VirtualKeyModifiers.None;

        if ((GetAsyncKeyState(0x11) & 0x8000) != 0) // VK_CONTROL
            mods |= VirtualKeyModifiers.Control;
        if ((GetAsyncKeyState(0x12) & 0x8000) != 0) // VK_MENU (Alt)
            mods |= VirtualKeyModifiers.Menu;
        if ((GetAsyncKeyState(0x10) & 0x8000) != 0) // VK_SHIFT
            mods |= VirtualKeyModifiers.Shift;
        if ((GetAsyncKeyState(0x5B) & 0x8000) != 0 || (GetAsyncKeyState(0x5C) & 0x8000) != 0)
            mods |= VirtualKeyModifiers.Windows;

        return mods;
    }

    private static string KeyToString(VirtualKey key)
    {
        return key switch
        {
            VirtualKey.Space => "Space",
            VirtualKey.Escape => "Esc",
            VirtualKey.Tab => "Tab",
            VirtualKey.Enter => "Enter",
            >= VirtualKey.A and <= VirtualKey.Z => key.ToString(),
            >= VirtualKey.Number0 and <= VirtualKey.Number9 => ((int)key - (int)VirtualKey.Number0).ToString(),
            >= VirtualKey.F1 and <= VirtualKey.F12 => $"F{(int)key - (int)VirtualKey.F1 + 1}",
            _ => key.ToString()
        };
    }

    [DllImport("user32.dll")]
    private static extern short GetAsyncKeyState(int vKey);
}

public class ShortcutRecordedEventArgs : EventArgs
{
    public VirtualKey Key { get; }
    public VirtualKeyModifiers Modifiers { get; }

    public ShortcutRecordedEventArgs(VirtualKey key, VirtualKeyModifiers modifiers)
    {
        Key = key;
        Modifiers = modifiers;
    }
}
