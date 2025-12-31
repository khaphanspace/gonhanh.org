using System.Text.Json;
using System.Text.Json.Serialization;

namespace GoNhanh.Core;

/// <summary>
/// Keyboard shortcut definition for toggle hotkey
/// Matches macOS KeyboardShortcut structure
/// </summary>
public class KeyboardShortcut
{
    private const string RegistryKey = @"SOFTWARE\GoNhanh";
    private const string RegistryValue = "ToggleShortcut";

    /// <summary>
    /// Virtual key code (e.g., VK_SPACE = 0x20)
    /// </summary>
    public uint KeyCode { get; set; }

    /// <summary>
    /// Modifier flags (Alt, Ctrl, Shift, Win)
    /// </summary>
    public ModifierKeys Modifiers { get; set; }

    /// <summary>
    /// Default shortcut: Ctrl+Space (matches macOS)
    /// </summary>
    public static KeyboardShortcut Default => new()
    {
        KeyCode = 0x20, // VK_SPACE
        Modifiers = ModifierKeys.Control
    };

    /// <summary>
    /// Check if this is a valid shortcut
    /// </summary>
    [JsonIgnore]
    public bool IsValid => KeyCode != 0 && Modifiers != ModifierKeys.None;

    /// <summary>
    /// Get display string for the shortcut (e.g., "Ctrl + Space")
    /// </summary>
    [JsonIgnore]
    public string DisplayString
    {
        get
        {
            var parts = new List<string>();

            if (Modifiers.HasFlag(ModifierKeys.Control)) parts.Add("Ctrl");
            if (Modifiers.HasFlag(ModifierKeys.Alt)) parts.Add("Alt");
            if (Modifiers.HasFlag(ModifierKeys.Shift)) parts.Add("Shift");
            if (Modifiers.HasFlag(ModifierKeys.Win)) parts.Add("Win");

            var keyName = GetKeyName(KeyCode);
            if (!string.IsNullOrEmpty(keyName)) parts.Add(keyName);

            return string.Join(" + ", parts);
        }
    }

    /// <summary>
    /// Load shortcut from registry
    /// </summary>
    public static KeyboardShortcut Load()
    {
        try
        {
            using var key = Microsoft.Win32.Registry.CurrentUser.OpenSubKey(RegistryKey);
            if (key == null) return Default;

            var json = key.GetValue(RegistryValue) as string;
            if (string.IsNullOrEmpty(json)) return Default;

            var loaded = JsonSerializer.Deserialize<KeyboardShortcut>(json);
            return loaded?.IsValid == true ? loaded : Default;
        }
        catch
        {
            return Default;
        }
    }

    /// <summary>
    /// Save shortcut to registry
    /// </summary>
    public void Save()
    {
        try
        {
            using var key = Microsoft.Win32.Registry.CurrentUser.CreateSubKey(RegistryKey);
            if (key != null)
            {
                var json = JsonSerializer.Serialize(this);
                key.SetValue(RegistryValue, json);
            }
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Failed to save shortcut: {ex.Message}");
        }
    }

    private static string GetKeyName(uint keyCode)
    {
        return keyCode switch
        {
            0x20 => "Space",
            0x08 => "Backspace",
            0x09 => "Tab",
            0x0D => "Enter",
            0x1B => "Esc",
            0x70 => "F1",
            0x71 => "F2",
            0x72 => "F3",
            0x73 => "F4",
            0x74 => "F5",
            0x75 => "F6",
            0x76 => "F7",
            0x77 => "F8",
            0x78 => "F9",
            0x79 => "F10",
            0x7A => "F11",
            0x7B => "F12",
            0xC0 => "`",
            >= 0x30 and <= 0x39 => ((char)keyCode).ToString(),
            >= 0x41 and <= 0x5A => ((char)keyCode).ToString(),
            _ => $"Key{keyCode:X2}"
        };
    }

    public override bool Equals(object? obj)
    {
        return obj is KeyboardShortcut other &&
               KeyCode == other.KeyCode &&
               Modifiers == other.Modifiers;
    }

    public override int GetHashCode() => HashCode.Combine(KeyCode, Modifiers);
}

/// <summary>
/// Modifier key flags for hotkeys
/// </summary>
[Flags]
public enum ModifierKeys : uint
{
    None = 0,
    Alt = 0x0001,
    Control = 0x0002,
    Shift = 0x0004,
    Win = 0x0008
}
