using Microsoft.Win32;
using GoNhanh.Core;

namespace GoNhanh.Services;

/// <summary>
/// Manages application settings using Windows Registry
/// Matches macOS UserDefaults functionality exactly
/// Includes all settings from macOS AppState
/// </summary>
public class SettingsService
{
    private const string RegistryKeyPath = @"SOFTWARE\GoNhanh";
    private const string AutoStartKeyPath = @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    private const string PerAppModesKeyPath = @"SOFTWARE\GoNhanh\PerAppModes";
    private const string AppName = "GoNhanh";

    #region Settings Keys

    private const string KeyInputMethod = "InputMethod";
    private const string KeyModernTone = "ModernTone";
    private const string KeyEnabled = "Enabled";
    private const string KeyFirstRun = "FirstRun";
    private const string KeyAutoStart = "AutoStart";
    private const string KeyAutoWShortcut = "AutoWShortcut";
    private const string KeyEscRestore = "EscRestore";
    private const string KeyEnglishAutoRestore = "EnglishAutoRestore";
    private const string KeyAutoCapitalize = "AutoCapitalize";
    private const string KeySoundEnabled = "SoundEnabled";
    private const string KeyPerAppModeEnabled = "PerAppModeEnabled";
    private const string KeyBracketShortcut = "BracketShortcut";

    #endregion

    #region Properties - Match macOS AppState

    public InputMethod CurrentMethod { get; set; } = InputMethod.Telex;
    public bool UseModernTone { get; set; } = true;
    public bool IsEnabled { get; set; } = true;
    public bool IsFirstRun { get; set; } = true;
    public bool AutoStart { get; set; } = false;

    /// <summary>
    /// W shortcut: typing W at word start produces Ư (Telex mode)
    /// </summary>
    public bool AutoWShortcut { get; set; } = true;

    /// <summary>
    /// ESC key restores original text before tone conversion
    /// </summary>
    public bool EscRestore { get; set; } = false;

    /// <summary>
    /// Auto-restore English words (Beta feature)
    /// </summary>
    public bool EnglishAutoRestore { get; set; } = false;

    /// <summary>
    /// Auto-capitalize after sentence-ending punctuation
    /// </summary>
    public bool AutoCapitalize { get; set; } = false;

    /// <summary>
    /// Play sound when toggling Vietnamese mode
    /// </summary>
    public bool SoundEnabled { get; set; } = false;

    /// <summary>
    /// Remember on/off state per application
    /// </summary>
    public bool PerAppModeEnabled { get; set; } = true;

    /// <summary>
    /// Bracket shortcut: [ → Ơ, ] → Ư
    /// </summary>
    public bool BracketShortcut { get; set; } = false;

    #endregion

    #region Public Methods

    /// <summary>
    /// Load settings from registry
    /// </summary>
    public void Load()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RegistryKeyPath);
            if (key == null)
            {
                // First run, use defaults
                IsFirstRun = true;
                return;
            }

            CurrentMethod = (InputMethod)(int)(key.GetValue(KeyInputMethod, 0) ?? 0);
            UseModernTone = GetBoolValue(key, KeyModernTone, true);
            IsEnabled = GetBoolValue(key, KeyEnabled, true);
            IsFirstRun = GetBoolValue(key, KeyFirstRun, true);
            AutoStart = GetBoolValue(key, KeyAutoStart, false);
            AutoWShortcut = GetBoolValue(key, KeyAutoWShortcut, true);
            EscRestore = GetBoolValue(key, KeyEscRestore, false);
            EnglishAutoRestore = GetBoolValue(key, KeyEnglishAutoRestore, false);
            AutoCapitalize = GetBoolValue(key, KeyAutoCapitalize, false);
            SoundEnabled = GetBoolValue(key, KeySoundEnabled, false);
            PerAppModeEnabled = GetBoolValue(key, KeyPerAppModeEnabled, true);
            BracketShortcut = GetBoolValue(key, KeyBracketShortcut, false);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Failed to load settings: {ex.Message}");
        }
    }

    /// <summary>
    /// Save settings to registry
    /// </summary>
    public void Save()
    {
        try
        {
            using var key = Registry.CurrentUser.CreateSubKey(RegistryKeyPath);
            if (key != null)
            {
                key.SetValue(KeyInputMethod, (int)CurrentMethod, RegistryValueKind.DWord);
                key.SetValue(KeyModernTone, UseModernTone ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue(KeyEnabled, IsEnabled ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue(KeyFirstRun, IsFirstRun ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue(KeyAutoStart, AutoStart ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue(KeyAutoWShortcut, AutoWShortcut ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue(KeyEscRestore, EscRestore ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue(KeyEnglishAutoRestore, EnglishAutoRestore ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue(KeyAutoCapitalize, AutoCapitalize ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue(KeySoundEnabled, SoundEnabled ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue(KeyPerAppModeEnabled, PerAppModeEnabled ? 1 : 0, RegistryValueKind.DWord);
                key.SetValue(KeyBracketShortcut, BracketShortcut ? 1 : 0, RegistryValueKind.DWord);
            }

            // Update auto-start registry
            UpdateAutoStart();
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Failed to save settings: {ex.Message}");
        }
    }

    /// <summary>
    /// Apply all settings to Rust engine
    /// </summary>
    public void ApplyToEngine()
    {
        RustBridge.SetMethod(CurrentMethod);
        RustBridge.SetEnabled(IsEnabled);
        RustBridge.SetModernTone(UseModernTone);
        RustBridge.SetSkipWShortcut(!AutoWShortcut);
        RustBridge.SetEscRestore(EscRestore);
        RustBridge.SetEnglishAutoRestore(EnglishAutoRestore);
        RustBridge.SetAutoCapitalize(AutoCapitalize);
        RustBridge.SetBracketShortcut(BracketShortcut);
    }

    /// <summary>
    /// Update Windows startup entry
    /// </summary>
    public void UpdateAutoStart()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(AutoStartKeyPath, true);
            if (key == null) return;

            if (AutoStart)
            {
                string exePath = System.Diagnostics.Process.GetCurrentProcess().MainModule?.FileName ?? "";
                if (!string.IsNullOrEmpty(exePath))
                {
                    key.SetValue(AppName, $"\"{exePath}\"");
                }
            }
            else
            {
                key.DeleteValue(AppName, false);
            }
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Failed to update auto-start: {ex.Message}");
        }
    }

    /// <summary>
    /// Get per-app mode state for a specific application
    /// </summary>
    public bool GetPerAppMode(string processName)
    {
        if (!PerAppModeEnabled || string.IsNullOrEmpty(processName))
            return true; // Default enabled

        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(PerAppModesKeyPath);
            if (key == null) return true;

            var value = key.GetValue(processName);
            if (value is int intValue)
                return intValue != 0;

            return true;
        }
        catch
        {
            return true;
        }
    }

    /// <summary>
    /// Save per-app mode state for a specific application
    /// </summary>
    public void SavePerAppMode(string processName, bool enabled)
    {
        if (string.IsNullOrEmpty(processName)) return;

        try
        {
            using var key = Registry.CurrentUser.CreateSubKey(PerAppModesKeyPath);
            if (key != null)
            {
                if (enabled)
                {
                    // Remove entry when enabled (default state)
                    key.DeleteValue(processName, false);
                }
                else
                {
                    // Only store when disabled
                    key.SetValue(processName, 0, RegistryValueKind.DWord);
                }
            }
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Failed to save per-app mode: {ex.Message}");
        }
    }

    /// <summary>
    /// Reset settings to defaults
    /// </summary>
    public void Reset()
    {
        CurrentMethod = InputMethod.Telex;
        UseModernTone = true;
        IsEnabled = true;
        AutoStart = false;
        AutoWShortcut = true;
        EscRestore = false;
        EnglishAutoRestore = false;
        AutoCapitalize = false;
        SoundEnabled = false;
        PerAppModeEnabled = true;
        BracketShortcut = false;
        Save();
    }

    #endregion

    #region Private Helpers

    private static bool GetBoolValue(RegistryKey key, string name, bool defaultValue)
    {
        var value = key.GetValue(name);
        if (value is int intValue)
            return intValue != 0;
        return defaultValue;
    }

    #endregion
}
