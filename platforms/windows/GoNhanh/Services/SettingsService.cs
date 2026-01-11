using Microsoft.Win32;
using GoNhanh.Core;

namespace GoNhanh.Services;

/// <summary>
/// Registry-based settings persistence
/// Location: HKEY_CURRENT_USER\SOFTWARE\GoNhanh
/// </summary>
public sealed class SettingsService
{
    private const string RegistryPath = @"SOFTWARE\GoNhanh";
    private const string RunPath = @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    private const string AppName = "GoNhanh";

    // Singleton
    private static SettingsService? _instance;
    public static SettingsService Instance => _instance ??= new SettingsService();

    // Cached values
    private SettingsData _cache;

    private SettingsService()
    {
        _cache = new SettingsData();
    }

    /// <summary>Check if this is first run</summary>
    public static bool IsFirstRun
    {
        get
        {
            using var key = Registry.CurrentUser.OpenSubKey(RegistryPath);
            return key == null;
        }
    }

    /// <summary>Load all settings from registry</summary>
    public SettingsData Load()
    {
        using var key = Registry.CurrentUser.OpenSubKey(RegistryPath);
        if (key == null) return _cache = SettingsData.Defaults;

        _cache = new SettingsData
        {
            // Engine settings
            Method = (InputMethod)GetInt(key, "Method", 0),
            Enabled = GetBool(key, "Enabled", true),

            // Options
            PerAppMode = GetBool(key, "PerAppMode", false),
            AutoCapitalize = GetBool(key, "AutoCapitalize", false),
            EnglishAutoRestore = GetBool(key, "EnglishAutoRestore", false),
            ModernTone = GetBool(key, "ModernTone", true),
            WShortcut = GetBool(key, "WShortcut", true),
            BracketShortcut = GetBool(key, "BracketShortcut", true),
            EscRestore = GetBool(key, "EscRestore", true),
            FreeTone = GetBool(key, "FreeTone", false),

            // System
            LaunchAtLogin = GetLaunchAtLogin(),
            SoundEnabled = GetBool(key, "SoundEnabled", true),

            // Shortcuts
            Shortcuts = LoadShortcuts()
        };

        return _cache;
    }

    /// <summary>Save all settings to registry</summary>
    public void Save(SettingsData data)
    {
        using var key = Registry.CurrentUser.CreateSubKey(RegistryPath);
        if (key == null) return;

        // Engine settings
        key.SetValue("Method", (int)data.Method, RegistryValueKind.DWord);
        key.SetValue("Enabled", data.Enabled ? 1 : 0, RegistryValueKind.DWord);

        // Options
        key.SetValue("PerAppMode", data.PerAppMode ? 1 : 0, RegistryValueKind.DWord);
        key.SetValue("AutoCapitalize", data.AutoCapitalize ? 1 : 0, RegistryValueKind.DWord);
        key.SetValue("EnglishAutoRestore", data.EnglishAutoRestore ? 1 : 0, RegistryValueKind.DWord);
        key.SetValue("ModernTone", data.ModernTone ? 1 : 0, RegistryValueKind.DWord);
        key.SetValue("WShortcut", data.WShortcut ? 1 : 0, RegistryValueKind.DWord);
        key.SetValue("BracketShortcut", data.BracketShortcut ? 1 : 0, RegistryValueKind.DWord);
        key.SetValue("EscRestore", data.EscRestore ? 1 : 0, RegistryValueKind.DWord);
        key.SetValue("FreeTone", data.FreeTone ? 1 : 0, RegistryValueKind.DWord);

        // System
        key.SetValue("SoundEnabled", data.SoundEnabled ? 1 : 0, RegistryValueKind.DWord);
        SetLaunchAtLogin(data.LaunchAtLogin);

        // Shortcuts
        SaveShortcuts(data.Shortcuts);

        _cache = data;
    }

    /// <summary>Save a single setting</summary>
    public void Set<T>(string name, T value)
    {
        using var key = Registry.CurrentUser.CreateSubKey(RegistryPath);
        if (key == null) return;

        if (value is bool b)
            key.SetValue(name, b ? 1 : 0, RegistryValueKind.DWord);
        else if (value is int i)
            key.SetValue(name, i, RegistryValueKind.DWord);
        else if (value is string s)
            key.SetValue(name, s, RegistryValueKind.String);
    }

    /// <summary>Get a single setting</summary>
    public T Get<T>(string name, T defaultValue)
    {
        using var key = Registry.CurrentUser.OpenSubKey(RegistryPath);
        if (key == null) return defaultValue;

        var value = key.GetValue(name);
        if (value == null) return defaultValue;

        if (typeof(T) == typeof(bool))
            return (T)(object)((int)value != 0);
        if (typeof(T) == typeof(int))
            return (T)value;
        if (typeof(T) == typeof(string))
            return (T)value;

        return defaultValue;
    }

    // ========== Launch at Login ==========

    private bool GetLaunchAtLogin()
    {
        using var key = Registry.CurrentUser.OpenSubKey(RunPath);
        return key?.GetValue(AppName) != null;
    }

    private void SetLaunchAtLogin(bool enabled)
    {
        using var key = Registry.CurrentUser.OpenSubKey(RunPath, writable: true);
        if (key == null) return;

        if (enabled)
        {
            var exePath = System.Diagnostics.Process.GetCurrentProcess().MainModule?.FileName;
            if (exePath != null)
            {
                key.SetValue(AppName, $"\"{exePath}\"", RegistryValueKind.String);
            }
        }
        else
        {
            key.DeleteValue(AppName, throwOnMissingValue: false);
        }
    }

    // ========== Shortcuts ==========

    private List<ShortcutEntry> LoadShortcuts()
    {
        var shortcuts = new List<ShortcutEntry>();

        using var key = Registry.CurrentUser.OpenSubKey($@"{RegistryPath}\Shortcuts");
        if (key == null) return shortcuts;

        foreach (var trigger in key.GetValueNames())
        {
            var replacement = key.GetValue(trigger) as string;
            if (!string.IsNullOrEmpty(replacement))
            {
                shortcuts.Add(new ShortcutEntry(trigger, replacement));
            }
        }

        return shortcuts;
    }

    private void SaveShortcuts(List<ShortcutEntry> shortcuts)
    {
        // Delete and recreate shortcuts key
        Registry.CurrentUser.DeleteSubKey($@"{RegistryPath}\Shortcuts", throwOnMissingSubKey: false);

        using var key = Registry.CurrentUser.CreateSubKey($@"{RegistryPath}\Shortcuts");
        if (key == null) return;

        foreach (var shortcut in shortcuts)
        {
            key.SetValue(shortcut.Trigger, shortcut.Replacement, RegistryValueKind.String);
        }
    }

    // ========== Per-App State ==========

    /// <summary>Get enabled state for specific app</summary>
    public bool GetAppEnabled(string processName, bool defaultEnabled)
    {
        using var key = Registry.CurrentUser.OpenSubKey($@"{RegistryPath}\Apps");
        if (key == null) return defaultEnabled;

        var value = key.GetValue(processName);
        return value == null ? defaultEnabled : (int)value != 0;
    }

    /// <summary>Set enabled state for specific app</summary>
    public void SetAppEnabled(string processName, bool enabled)
    {
        using var key = Registry.CurrentUser.CreateSubKey($@"{RegistryPath}\Apps");
        key?.SetValue(processName, enabled ? 1 : 0, RegistryValueKind.DWord);
    }

    // ========== Helpers ==========

    private static int GetInt(RegistryKey key, string name, int defaultValue)
    {
        var value = key.GetValue(name);
        return value is int i ? i : defaultValue;
    }

    private static bool GetBool(RegistryKey key, string name, bool defaultValue)
    {
        var value = key.GetValue(name);
        return value is int i ? i != 0 : defaultValue;
    }
}

/// <summary>All settings data</summary>
public class SettingsData
{
    // Engine
    public InputMethod Method { get; set; }
    public bool Enabled { get; set; }

    // Options
    public bool PerAppMode { get; set; }
    public bool AutoCapitalize { get; set; }
    public bool EnglishAutoRestore { get; set; }
    public bool ModernTone { get; set; }
    public bool WShortcut { get; set; }
    public bool BracketShortcut { get; set; }
    public bool EscRestore { get; set; }
    public bool FreeTone { get; set; }

    // System
    public bool LaunchAtLogin { get; set; }
    public bool SoundEnabled { get; set; }

    // Shortcuts
    public List<ShortcutEntry> Shortcuts { get; set; } = new();

    public static SettingsData Defaults => new()
    {
        Method = InputMethod.Telex,
        Enabled = true,
        PerAppMode = false,
        AutoCapitalize = false,
        EnglishAutoRestore = false,
        ModernTone = true,
        WShortcut = true,
        BracketShortcut = true,
        EscRestore = true,
        FreeTone = false,
        LaunchAtLogin = false,
        SoundEnabled = true,
        Shortcuts = new List<ShortcutEntry>()
    };
}

/// <summary>Shortcut entry</summary>
public record ShortcutEntry(string Trigger, string Replacement);
