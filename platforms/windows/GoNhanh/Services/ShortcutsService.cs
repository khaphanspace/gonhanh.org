using System.Text.Json;
using GoNhanh.Core;

namespace GoNhanh.Services;

/// <summary>
/// Text abbreviation shortcut item
/// Matches macOS ShortcutItem structure
/// </summary>
public class ShortcutItem
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public string Key { get; set; } = string.Empty;
    public string Value { get; set; } = string.Empty;
    public bool IsEnabled { get; set; } = true;

    public bool IsValid => !string.IsNullOrEmpty(Key) && !string.IsNullOrEmpty(Value);
}

/// <summary>
/// Manages text abbreviation shortcuts with JSON file storage
/// Matches macOS ShortcutsManager functionality
/// </summary>
public class ShortcutsService
{
    private static readonly string AppDataPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "GoNhanh");

    private static readonly string ShortcutsFilePath = Path.Combine(AppDataPath, "shortcuts.json");

    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    public List<ShortcutItem> Shortcuts { get; private set; } = new();

    public event Action? ShortcutsChanged;

    /// <summary>
    /// Load shortcuts from JSON file
    /// </summary>
    public void Load()
    {
        try
        {
            if (File.Exists(ShortcutsFilePath))
            {
                var json = File.ReadAllText(ShortcutsFilePath);
                var loaded = JsonSerializer.Deserialize<List<ShortcutItem>>(json, JsonOptions);
                if (loaded != null)
                {
                    Shortcuts = loaded;
                    SyncToEngine();
                    return;
                }
            }
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Failed to load shortcuts: {ex.Message}");
        }

        // Default shortcuts (disabled by default, matching macOS)
        Shortcuts = new List<ShortcutItem>
        {
            new() { Key = "vn", Value = "Việt Nam", IsEnabled = false },
            new() { Key = "hn", Value = "Hà Nội", IsEnabled = false },
            new() { Key = "hcm", Value = "Hồ Chí Minh", IsEnabled = false },
            new() { Key = "tphcm", Value = "Thành phố Hồ Chí Minh", IsEnabled = false }
        };
        Save();
    }

    /// <summary>
    /// Save shortcuts to JSON file
    /// </summary>
    public void Save()
    {
        try
        {
            Directory.CreateDirectory(AppDataPath);
            var json = JsonSerializer.Serialize(Shortcuts, JsonOptions);
            File.WriteAllText(ShortcutsFilePath, json);
            SyncToEngine();
            ShortcutsChanged?.Invoke();
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Failed to save shortcuts: {ex.Message}");
        }
    }

    /// <summary>
    /// Add a new shortcut
    /// </summary>
    public void Add(string key, string value, bool enabled = true)
    {
        if (string.IsNullOrEmpty(key) || string.IsNullOrEmpty(value)) return;

        // Check for existing key and update
        var existing = Shortcuts.FirstOrDefault(s => s.Key == key);
        if (existing != null)
        {
            existing.Value = value;
            existing.IsEnabled = enabled;
        }
        else
        {
            Shortcuts.Add(new ShortcutItem { Key = key, Value = value, IsEnabled = enabled });
        }
        Save();
    }

    /// <summary>
    /// Remove a shortcut by key
    /// </summary>
    public void Remove(string key)
    {
        Shortcuts.RemoveAll(s => s.Key == key);
        Save();
    }

    /// <summary>
    /// Remove shortcut by ID
    /// </summary>
    public void RemoveById(Guid id)
    {
        Shortcuts.RemoveAll(s => s.Id == id);
        Save();
    }

    /// <summary>
    /// Update a shortcut
    /// </summary>
    public void Update(Guid id, string key, string value, bool enabled)
    {
        var item = Shortcuts.FirstOrDefault(s => s.Id == id);
        if (item != null)
        {
            item.Key = key;
            item.Value = value;
            item.IsEnabled = enabled;
            Save();
        }
    }

    /// <summary>
    /// Toggle shortcut enabled state
    /// </summary>
    public void ToggleEnabled(Guid id)
    {
        var item = Shortcuts.FirstOrDefault(s => s.Id == id);
        if (item != null)
        {
            item.IsEnabled = !item.IsEnabled;
            Save();
        }
    }

    /// <summary>
    /// Clear all shortcuts
    /// </summary>
    public void Clear()
    {
        Shortcuts.Clear();
        Save();
    }

    /// <summary>
    /// Sync shortcuts to Rust engine
    /// </summary>
    public void SyncToEngine()
    {
        var validShortcuts = Shortcuts
            .Where(s => s.IsValid)
            .Select(s => (s.Key, s.Value, s.IsEnabled));
        RustBridge.SyncShortcuts(validShortcuts);
    }

    /// <summary>
    /// Export shortcuts to text format (key:value per line)
    /// </summary>
    public string Export()
    {
        var lines = Shortcuts
            .Where(s => s.IsValid)
            .Select(s => $"{s.Key}:{s.Value}");
        return string.Join(Environment.NewLine, lines);
    }

    /// <summary>
    /// Import shortcuts from text format
    /// Returns number of imported items
    /// </summary>
    public int Import(string content)
    {
        if (string.IsNullOrWhiteSpace(content)) return 0;

        int count = 0;
        var lines = content.Split(new[] { '\n', '\r' }, StringSplitOptions.RemoveEmptyEntries);

        foreach (var line in lines)
        {
            var parts = line.Split(':', 2);
            if (parts.Length != 2) continue;

            var trigger = parts[0].Trim();
            var replacement = parts[1].Trim();

            if (string.IsNullOrEmpty(trigger) || string.IsNullOrEmpty(replacement)) continue;

            var existing = Shortcuts.FirstOrDefault(s => s.Key == trigger);
            if (existing != null)
            {
                existing.Value = replacement;
                existing.IsEnabled = true;
            }
            else
            {
                Shortcuts.Add(new ShortcutItem { Key = trigger, Value = replacement, IsEnabled = true });
            }
            count++;
        }

        if (count > 0) Save();
        return count;
    }

    /// <summary>
    /// Get count of enabled shortcuts
    /// </summary>
    public int EnabledCount => Shortcuts.Count(s => s.IsEnabled && s.IsValid);

    /// <summary>
    /// Get total count
    /// </summary>
    public int TotalCount => Shortcuts.Count;
}
