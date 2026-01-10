using System.Diagnostics;
using System.Text.Json;
using Microsoft.Win32;
using GoNhanh.Core;

namespace GoNhanh.Services;

/// <summary>
/// Settings persistence using Windows Registry.
/// Path: HKCU\Software\GoNhanh
/// Thread-safe via lock for concurrent access.
/// </summary>
public sealed class SettingsService
{
    private const string RegistryPath = @"Software\GoNhanh";
    private static readonly Lazy<SettingsService> _instance = new(() => new SettingsService());
    private readonly object _lock = new();

    public static SettingsService Instance => _instance.Value;

    private SettingsService()
    {
        LoadAll();
    }

    // ============================================================
    // Settings Properties
    // ============================================================

    public bool Enabled { get; set; } = true;
    public byte InputMethod { get; set; } = 0; // 0=Telex, 1=VNI
    public bool SkipWShortcut { get; set; } = false;
    public bool BracketShortcut { get; set; } = true;
    public bool EscRestore { get; set; } = false;
    public bool FreeTone { get; set; } = false;
    public bool ModernTone { get; set; } = true;
    public bool EnglishAutoRestore { get; set; } = false;
    public bool AutoCapitalize { get; set; } = false;
    public bool SoundEnabled { get; set; } = false;
    public bool LaunchAtStartup { get; set; } = false;

    // Toggle shortcut (stored as JSON string)
    public string ToggleShortcut { get; set; } = "{\"key\":\"Space\",\"modifiers\":\"Ctrl\"}";

    // User shortcuts (stored as JSON array)
    public string UserShortcuts { get; set; } = "[]";

    // ============================================================
    // Load/Save Methods
    // ============================================================

    public void LoadAll()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RegistryPath);
            if (key == null) return; // First run, use defaults

            Enabled = GetBool(key, "Enabled", true);
            InputMethod = GetByte(key, "InputMethod", 0);
            SkipWShortcut = GetBool(key, "SkipWShortcut", false);
            BracketShortcut = GetBool(key, "BracketShortcut", true);
            EscRestore = GetBool(key, "EscRestore", false);
            FreeTone = GetBool(key, "FreeTone", false);
            ModernTone = GetBool(key, "ModernTone", true);
            EnglishAutoRestore = GetBool(key, "EnglishAutoRestore", false);
            AutoCapitalize = GetBool(key, "AutoCapitalize", false);
            SoundEnabled = GetBool(key, "SoundEnabled", false);
            LaunchAtStartup = GetBool(key, "LaunchAtStartup", false);
            ToggleShortcut = GetString(key, "ToggleShortcut", ToggleShortcut);
            UserShortcuts = GetString(key, "UserShortcuts", UserShortcuts);
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Failed to load settings: {ex.Message}");
        }
    }

    public void SaveAll()
    {
        lock (_lock)
        {
            try
            {
                using var key = Registry.CurrentUser.CreateSubKey(RegistryPath);
                if (key == null) return;

                key.SetValue("Enabled", Enabled ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue("InputMethod", (int)InputMethod, RegistryValueKind.DWord);
                key.SetValue("SkipWShortcut", SkipWShortcut ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue("BracketShortcut", BracketShortcut ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue("EscRestore", EscRestore ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue("FreeTone", FreeTone ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue("ModernTone", ModernTone ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue("EnglishAutoRestore", EnglishAutoRestore ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue("AutoCapitalize", AutoCapitalize ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue("SoundEnabled", SoundEnabled ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue("LaunchAtStartup", LaunchAtStartup ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue("ToggleShortcut", ToggleShortcut, RegistryValueKind.String);
                key.SetValue("UserShortcuts", UserShortcuts, RegistryValueKind.String);
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"Failed to save settings: {ex.Message}");
            }
        }
    }

    /// <summary>
    /// Apply all settings to Rust engine.
    /// </summary>
    public void ApplyToEngine()
    {
        RustBridge.ime_enabled(Enabled);
        RustBridge.ime_method(InputMethod);
        RustBridge.ime_skip_w_shortcut(SkipWShortcut);
        RustBridge.ime_bracket_shortcut(BracketShortcut);
        RustBridge.ime_esc_restore(EscRestore);
        RustBridge.ime_free_tone(FreeTone);
        RustBridge.ime_modern(ModernTone);
        RustBridge.ime_english_auto_restore(EnglishAutoRestore);
        RustBridge.ime_auto_capitalize(AutoCapitalize);

        // Load user shortcuts
        LoadUserShortcuts();
    }

    private void LoadUserShortcuts()
    {
        RustBridge.ime_clear_shortcuts();

        try
        {
            var shortcuts = JsonSerializer.Deserialize<List<ShortcutEntry>>(UserShortcuts);
            if (shortcuts == null) return;

            foreach (var s in shortcuts)
            {
                if (!string.IsNullOrEmpty(s.Trigger) && !string.IsNullOrEmpty(s.Replacement))
                {
                    RustBridge.ime_add_shortcut(s.Trigger, s.Replacement);
                }
            }
        }
        catch (JsonException ex)
        {
            // Reset to empty if corrupted
            Debug.WriteLine($"Corrupted shortcuts JSON, resetting: {ex.Message}");
            UserShortcuts = "[]";
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Failed to load shortcuts: {ex.Message}");
        }
    }

    // ============================================================
    // Helper Methods
    // ============================================================

    private static bool GetBool(RegistryKey key, string name, bool defaultValue)
    {
        var value = key.GetValue(name);
        if (value is int intVal) return intVal != 0;
        return defaultValue;
    }

    private static byte GetByte(RegistryKey key, string name, byte defaultValue)
    {
        var value = key.GetValue(name);
        if (value is int intVal) return (byte)intVal;
        return defaultValue;
    }

    private static string GetString(RegistryKey key, string name, string defaultValue)
    {
        var value = key.GetValue(name);
        if (value is string strVal) return strVal;
        return defaultValue;
    }

    private record ShortcutEntry(string Trigger, string Replacement);
}
