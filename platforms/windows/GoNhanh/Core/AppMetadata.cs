using System.Reflection;

namespace GoNhanh.Core;

/// <summary>
/// Application metadata and constants
/// </summary>
public static class AppMetadata
{
    /// <summary>
    /// Application name
    /// </summary>
    public const string AppName = "Gõ Nhanh";

    /// <summary>
    /// Application display name in Vietnamese
    /// </summary>
    public const string DisplayName = "Gõ Nhanh";

    /// <summary>
    /// Application description
    /// </summary>
    public const string Description = "Vietnamese Input Method for Windows";

    /// <summary>
    /// Organization name
    /// </summary>
    public const string Organization = "Gõ Nhanh Contributors";

    /// <summary>
    /// Project homepage URL
    /// </summary>
    public const string HomepageUrl = "https://github.com/user/gonhanh";

    /// <summary>
    /// Registry key path for application settings
    /// </summary>
    public const string RegistryPath = @"SOFTWARE\GoNhanh";

    /// <summary>
    /// Run registry key for startup
    /// </summary>
    public const string RunRegistryPath = @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

    /// <summary>
    /// Get application version from assembly
    /// </summary>
    public static Version Version =>
        Assembly.GetExecutingAssembly().GetName().Version ?? new Version(1, 0, 0);

    /// <summary>
    /// Get version string (e.g., "1.0.0")
    /// </summary>
    public static string VersionString =>
        $"{Version.Major}.{Version.Minor}.{Version.Build}";
}
