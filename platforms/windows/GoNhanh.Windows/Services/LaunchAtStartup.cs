using System.Diagnostics;
using Microsoft.Win32;

namespace GoNhanh.Services;

/// <summary>
/// Manage Windows startup registration via Registry Run key.
/// No UAC required - uses HKCU.
/// </summary>
public static class LaunchAtStartup
{
    private const string RunKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string AppName = "GoNhanh";

    public static bool IsEnabled
    {
        get
        {
            try
            {
                using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath);
                return key?.GetValue(AppName) != null;
            }
            catch
            {
                return false;
            }
        }
    }

    public static void Enable()
    {
        try
        {
            var exePath = Environment.ProcessPath;
            if (string.IsNullOrEmpty(exePath)) return;

            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, writable: true);
            key?.SetValue(AppName, $"\"{exePath}\"", RegistryValueKind.String);

            SettingsService.Instance.LaunchAtStartup = true;
            SettingsService.Instance.SaveAll();
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Failed to enable startup: {ex.Message}");
        }
    }

    public static void Disable()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, writable: true);
            key?.DeleteValue(AppName, throwOnMissingValue: false);

            SettingsService.Instance.LaunchAtStartup = false;
            SettingsService.Instance.SaveAll();
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Failed to disable startup: {ex.Message}");
        }
    }
}
