using Microsoft.Win32;
using GoNhanh.Core;

namespace GoNhanh.Services;

/// <summary>
/// Manages application settings using Windows Registry
/// Similar to UserDefaults on macOS
/// </summary>
public class SettingsService
{
    private const string RegistryKeyPath = @"SOFTWARE\GoNhanh";
    private const string AutoStartKeyPath = @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    private const string AppName = "GoNhanh";

    #region Settings Keys

    private const string KeyInputMethod = "InputMethod";
    private const string KeyModernTone = "ModernTone";
    private const string KeyEnabled = "Enabled";
    private const string KeyFirstRun = "FirstRun";
    private const string KeyAutoStart = "AutoStart";

    #endregion

    #region Properties

    public InputMethod CurrentMethod { get; set; } = InputMethod.Telex;
    public bool UseModernTone { get; set; } = true;
    public bool IsEnabled { get; set; } = true;
    public bool IsFirstRun { get; set; } = true;
    public bool AutoStart { get; set; } = false;

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
            UseModernTone = ((int)(key.GetValue(KeyModernTone, 1) ?? 1)) == 1;
            IsEnabled = ((int)(key.GetValue(KeyEnabled, 1) ?? 1)) == 1;
            IsFirstRun = ((int)(key.GetValue(KeyFirstRun, 1) ?? 1)) == 1;
            AutoStart = ((int)(key.GetValue(KeyAutoStart, 0) ?? 0)) == 1;
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
    /// Reset settings to defaults
    /// </summary>
    public void Reset()
    {
        CurrentMethod = InputMethod.Telex;
        UseModernTone = true;
        IsEnabled = true;
        AutoStart = false;
        Save();
    }

    #endregion
}
