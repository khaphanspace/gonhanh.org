using System.Diagnostics;
using System.Net.Http;
using System.Text.Json;
using GoNhanh.Core;

namespace GoNhanh.Services;

/// <summary>
/// Update information from GitHub release
/// </summary>
public class UpdateInfo
{
    public string Version { get; set; } = "";
    public string DownloadUrl { get; set; } = "";
    public string ReleaseNotes { get; set; } = "";
    public DateTime? PublishedAt { get; set; }
}

/// <summary>
/// Result of update check
/// </summary>
public enum UpdateCheckResult
{
    Available,
    UpToDate,
    Error
}

/// <summary>
/// Manages update checking via GitHub releases
/// Matches macOS UpdateChecker functionality
/// </summary>
public class UpdateService
{
    private const string GitHubApiUrl = "https://api.github.com/repos/khaphanspace/gonhanh.org/releases";
    private static readonly HttpClient HttpClient = new();

    public UpdateInfo? LatestUpdate { get; private set; }
    public string ErrorMessage { get; private set; } = "";

    static UpdateService()
    {
        HttpClient.DefaultRequestHeaders.Add("Accept", "application/vnd.github.v3+json");
        HttpClient.DefaultRequestHeaders.Add("User-Agent", $"GoNhanh/{AppMetadata.Version}");
        HttpClient.Timeout = TimeSpan.FromSeconds(10);
    }

    /// <summary>
    /// Check for updates asynchronously
    /// </summary>
    public async Task<UpdateCheckResult> CheckForUpdatesAsync()
    {
        try
        {
            var response = await HttpClient.GetAsync(GitHubApiUrl);

            if (!response.IsSuccessStatusCode)
            {
                ErrorMessage = $"Server error: {(int)response.StatusCode}";
                return UpdateCheckResult.Error;
            }

            var json = await response.Content.ReadAsStringAsync();
            return ParseReleases(json);
        }
        catch (HttpRequestException ex)
        {
            ErrorMessage = $"Network error: {ex.Message}";
            return UpdateCheckResult.Error;
        }
        catch (TaskCanceledException)
        {
            ErrorMessage = "Request timeout";
            return UpdateCheckResult.Error;
        }
        catch (Exception ex)
        {
            ErrorMessage = $"Error: {ex.Message}";
            return UpdateCheckResult.Error;
        }
    }

    private UpdateCheckResult ParseReleases(string json)
    {
        try
        {
            using var doc = JsonDocument.Parse(json);
            var releases = doc.RootElement;

            if (releases.ValueKind != JsonValueKind.Array)
            {
                ErrorMessage = "Invalid response format";
                return UpdateCheckResult.Error;
            }

            string bestVersion = "";
            JsonElement? bestRelease = null;

            foreach (var release in releases.EnumerateArray())
            {
                // Skip drafts and prereleases
                if (release.TryGetProperty("draft", out var draft) && draft.GetBoolean())
                    continue;
                if (release.TryGetProperty("prerelease", out var prerelease) && prerelease.GetBoolean())
                    continue;

                if (!release.TryGetProperty("tag_name", out var tagName))
                    continue;

                var version = tagName.GetString() ?? "";
                if (version.StartsWith("v"))
                    version = version[1..];

                if (string.IsNullOrEmpty(version))
                    continue;

                // Compare with current best
                if (string.IsNullOrEmpty(bestVersion))
                {
                    bestVersion = version;
                    bestRelease = release;
                }
                else
                {
                    var cmp = RustBridge.CompareVersions(bestVersion, version);
                    if (cmp < 0) // version > bestVersion
                    {
                        bestVersion = version;
                        bestRelease = release;
                    }
                }
            }

            if (bestRelease == null || string.IsNullOrEmpty(bestVersion))
            {
                return UpdateCheckResult.UpToDate;
            }

            // Check if update available
            var currentVersion = AppMetadata.Version;
            if (!RustBridge.HasUpdate(currentVersion, bestVersion))
            {
                return UpdateCheckResult.UpToDate;
            }

            // Find Windows installer download URL (.exe or .msi)
            string? downloadUrl = null;
            if (bestRelease.Value.TryGetProperty("assets", out var assets))
            {
                foreach (var asset in assets.EnumerateArray())
                {
                    if (!asset.TryGetProperty("name", out var nameElem))
                        continue;

                    var name = nameElem.GetString()?.ToLowerInvariant() ?? "";
                    if (name.EndsWith(".exe") || name.EndsWith(".msi") || name.EndsWith(".zip"))
                    {
                        if (asset.TryGetProperty("browser_download_url", out var urlElem))
                        {
                            downloadUrl = urlElem.GetString();
                            break;
                        }
                    }
                }
            }

            if (string.IsNullOrEmpty(downloadUrl))
            {
                ErrorMessage = "No Windows installer found in release";
                return UpdateCheckResult.Error;
            }

            // Parse metadata
            var releaseNotes = "";
            if (bestRelease.Value.TryGetProperty("body", out var body))
                releaseNotes = body.GetString() ?? "";

            DateTime? publishedAt = null;
            if (bestRelease.Value.TryGetProperty("published_at", out var publishedAtElem))
            {
                if (DateTime.TryParse(publishedAtElem.GetString(), out var parsed))
                    publishedAt = parsed;
            }

            LatestUpdate = new UpdateInfo
            {
                Version = bestVersion,
                DownloadUrl = downloadUrl,
                ReleaseNotes = releaseNotes,
                PublishedAt = publishedAt
            };

            return UpdateCheckResult.Available;
        }
        catch (Exception ex)
        {
            ErrorMessage = $"Parse error: {ex.Message}";
            return UpdateCheckResult.Error;
        }
    }

    /// <summary>
    /// Open download URL in browser
    /// </summary>
    public void OpenDownloadPage()
    {
        if (LatestUpdate?.DownloadUrl != null)
        {
            try
            {
                Process.Start(new ProcessStartInfo
                {
                    FileName = LatestUpdate.DownloadUrl,
                    UseShellExecute = true
                });
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"Failed to open download URL: {ex.Message}");
            }
        }
    }

    /// <summary>
    /// Open GitHub releases page
    /// </summary>
    public static void OpenReleasesPage()
    {
        try
        {
            Process.Start(new ProcessStartInfo
            {
                FileName = "https://github.com/khaphanspace/gonhanh.org/releases",
                UseShellExecute = true
            });
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Failed to open releases page: {ex.Message}");
        }
    }
}
