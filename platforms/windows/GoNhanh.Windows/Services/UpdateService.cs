using System.Diagnostics;
using System.Net.Http;
using System.Text.Json;

namespace GoNhanh.Services;

public class UpdateService
{
    private const string GitHubApiUrl = "https://api.github.com/repos/khaphanspace/gonhanh.org/releases/latest";
    private static readonly HttpClient _client = new()
    {
        DefaultRequestHeaders = { { "User-Agent", "GoNhanh-Windows" } },
        Timeout = TimeSpan.FromSeconds(30)
    };

    public static string CurrentVersion => "1.0.102";

    public async Task<UpdateInfo?> CheckForUpdateAsync()
    {
        try
        {
            var json = await _client.GetStringAsync(GitHubApiUrl);
            var release = JsonSerializer.Deserialize<GitHubRelease>(json);

            if (release == null) return null;

            var latestVersion = release.tag_name.TrimStart('v');
            if (!IsNewerVersion(latestVersion, CurrentVersion))
                return null;

            // Find Windows MSIX asset
            var asset = release.assets?.FirstOrDefault(a =>
                a.name.EndsWith(".msix", StringComparison.OrdinalIgnoreCase));

            return new UpdateInfo
            {
                Version = latestVersion,
                ReleaseNotes = release.body ?? "",
                DownloadUrl = asset?.browser_download_url,
                FileName = asset?.name
            };
        }
        catch
        {
            return null;
        }
    }

    public async Task DownloadUpdateAsync(string url, string fileName,
        IProgress<double>? progress, CancellationToken ct = default)
    {
        var tempPath = Path.Combine(Path.GetTempPath(), fileName);

        using var response = await _client.GetAsync(url, HttpCompletionOption.ResponseHeadersRead, ct);
        response.EnsureSuccessStatusCode();

        var totalBytes = response.Content.Headers.ContentLength ?? -1L;
        var downloadedBytes = 0L;

        await using var fs = new FileStream(tempPath, FileMode.Create, FileAccess.Write, FileShare.None);
        await using var contentStream = await response.Content.ReadAsStreamAsync(ct);

        var buffer = new byte[8192];
        int bytesRead;
        while ((bytesRead = await contentStream.ReadAsync(buffer, ct)) > 0)
        {
            await fs.WriteAsync(buffer.AsMemory(0, bytesRead), ct);
            downloadedBytes += bytesRead;

            if (totalBytes > 0)
                progress?.Report((double)downloadedBytes / totalBytes);
        }

        // Launch installer
        Process.Start(new ProcessStartInfo
        {
            FileName = tempPath,
            UseShellExecute = true
        });
    }

    private static bool IsNewerVersion(string latest, string current)
    {
        try
        {
            var latestParts = latest.Split('.').Select(int.Parse).ToArray();
            var currentParts = current.Split('.').Select(int.Parse).ToArray();

            for (int i = 0; i < Math.Min(latestParts.Length, currentParts.Length); i++)
            {
                if (latestParts[i] > currentParts[i]) return true;
                if (latestParts[i] < currentParts[i]) return false;
            }
            return false;
        }
        catch
        {
            return false;
        }
    }

    private record GitHubRelease(string tag_name, string? body, GitHubAsset[]? assets);
    private record GitHubAsset(string name, string browser_download_url);
}

public record UpdateInfo
{
    public string Version { get; init; } = "";
    public string ReleaseNotes { get; init; } = "";
    public string? DownloadUrl { get; init; }
    public string? FileName { get; init; }
}
