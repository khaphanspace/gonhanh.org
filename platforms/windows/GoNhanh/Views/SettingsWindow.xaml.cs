using System.Windows;
using System.Windows.Controls;
using GoNhanh.Core;
using GoNhanh.Services;

namespace GoNhanh.Views;

/// <summary>
/// Settings window matching macOS MainSettingsView
/// </summary>
public partial class SettingsWindow : Window
{
    private readonly SettingsService _settings;
    private readonly ShortcutsService _shortcuts;
    private readonly HotKeyService? _hotKeyService;
    private bool _isLoading = true;

    public event Action? SettingsChanged;

    public SettingsWindow(SettingsService settings, ShortcutsService shortcuts, HotKeyService? hotKeyService = null)
    {
        InitializeComponent();
        _settings = settings;
        _shortcuts = shortcuts;
        _hotKeyService = hotKeyService;
        LoadSettings();

        // Wire up shortcut recorder
        if (_hotKeyService != null)
        {
            ShortcutRecorder.Shortcut = _hotKeyService.CurrentShortcut;
            ShortcutRecorder.ShortcutChanged += OnShortcutChanged;
        }
    }

    private void OnShortcutChanged(KeyboardShortcut newShortcut)
    {
        if (_hotKeyService?.UpdateShortcut(newShortcut) == true)
        {
            MessageBox.Show(
                $"Đã đặt phím tắt: {newShortcut.DisplayString}",
                "Thành công",
                MessageBoxButton.OK,
                MessageBoxImage.Information);
        }
        else
        {
            MessageBox.Show(
                "Không thể đặt phím tắt này.\nCó thể đã được sử dụng bởi ứng dụng khác.",
                "Lỗi",
                MessageBoxButton.OK,
                MessageBoxImage.Warning);
            if (_hotKeyService != null)
            {
                ShortcutRecorder.Shortcut = _hotKeyService.CurrentShortcut;
            }
        }
    }

    private void LoadSettings()
    {
        _isLoading = true;

        EnabledToggle.IsChecked = _settings.IsEnabled;
        MethodCombo.SelectedIndex = (int)_settings.CurrentMethod;
        WShortcutToggle.IsChecked = _settings.AutoWShortcut;
        AutoStartToggle.IsChecked = _settings.AutoStart;
        PerAppModeToggle.IsChecked = _settings.PerAppModeEnabled;
        EnglishAutoRestoreToggle.IsChecked = _settings.EnglishAutoRestore;
        SoundToggle.IsChecked = _settings.SoundEnabled;
        ModernToneToggle.IsChecked = _settings.UseModernTone;
        AutoCapitalizeToggle.IsChecked = _settings.AutoCapitalize;
        EscRestoreToggle.IsChecked = _settings.EscRestore;

        // Update W shortcut visibility based on method
        UpdateWShortcutVisibility();

        // Version info
        VersionText.Text = $"Version {AppMetadata.Version} - {AppMetadata.TechStack}";

        // Shortcuts count
        UpdateShortcutsCount();

        _isLoading = false;
    }

    private void UpdateWShortcutVisibility()
    {
        WShortcutRow.Visibility = _settings.CurrentMethod == InputMethod.Telex
            ? Visibility.Visible
            : Visibility.Collapsed;
    }

    private void SaveAndNotify()
    {
        if (_isLoading) return;

        _settings.Save();
        SettingsChanged?.Invoke();
    }

    private void EnabledToggle_Changed(object sender, RoutedEventArgs e)
    {
        if (_isLoading) return;
        _settings.IsEnabled = EnabledToggle.IsChecked == true;
        SaveAndNotify();
    }

    private void MethodCombo_Changed(object sender, SelectionChangedEventArgs e)
    {
        if (_isLoading) return;
        _settings.CurrentMethod = (InputMethod)MethodCombo.SelectedIndex;
        UpdateWShortcutVisibility();
        SaveAndNotify();
    }

    private void WShortcutToggle_Changed(object sender, RoutedEventArgs e)
    {
        if (_isLoading) return;
        _settings.AutoWShortcut = WShortcutToggle.IsChecked == true;
        SaveAndNotify();
    }

    private void AutoStartToggle_Changed(object sender, RoutedEventArgs e)
    {
        if (_isLoading) return;
        _settings.AutoStart = AutoStartToggle.IsChecked == true;
        SaveAndNotify();
    }

    private void PerAppModeToggle_Changed(object sender, RoutedEventArgs e)
    {
        if (_isLoading) return;
        _settings.PerAppModeEnabled = PerAppModeToggle.IsChecked == true;
        SaveAndNotify();
    }

    private void EnglishAutoRestoreToggle_Changed(object sender, RoutedEventArgs e)
    {
        if (_isLoading) return;
        _settings.EnglishAutoRestore = EnglishAutoRestoreToggle.IsChecked == true;
        SaveAndNotify();
    }

    private void SoundToggle_Changed(object sender, RoutedEventArgs e)
    {
        if (_isLoading) return;
        _settings.SoundEnabled = SoundToggle.IsChecked == true;
        SaveAndNotify();
    }

    private void ModernToneToggle_Changed(object sender, RoutedEventArgs e)
    {
        if (_isLoading) return;
        _settings.UseModernTone = ModernToneToggle.IsChecked == true;
        SaveAndNotify();
    }

    private void AutoCapitalizeToggle_Changed(object sender, RoutedEventArgs e)
    {
        if (_isLoading) return;
        _settings.AutoCapitalize = AutoCapitalizeToggle.IsChecked == true;
        SaveAndNotify();
    }

    private void EscRestoreToggle_Changed(object sender, RoutedEventArgs e)
    {
        if (_isLoading) return;
        _settings.EscRestore = EscRestoreToggle.IsChecked == true;
        SaveAndNotify();
    }

    private void ShortcutsButton_Click(object sender, RoutedEventArgs e)
    {
        var window = new ShortcutsWindow(_shortcuts);
        window.Owner = this;
        window.ShowDialog();
        UpdateShortcutsCount();
    }

    private void UpdateShortcutsCount()
    {
        var enabled = _shortcuts.EnabledCount;
        var total = _shortcuts.TotalCount;
        ShortcutsCountText.Text = total == 0
            ? "Chưa có"
            : $"{enabled}/{total} đang bật";
    }

    private async void CheckUpdate_Click(object sender, RoutedEventArgs e)
    {
        CheckUpdateButton.IsEnabled = false;
        CheckUpdateButton.Content = "Đang kiểm tra...";

        try
        {
            var updateService = new UpdateService();
            var result = await updateService.CheckForUpdatesAsync();

            switch (result)
            {
                case UpdateCheckResult.Available:
                    var response = MessageBox.Show(
                        $"Đã có phiên bản mới: {updateService.LatestUpdate?.Version}\n\nBạn có muốn tải về không?",
                        "Cập nhật có sẵn",
                        MessageBoxButton.YesNo,
                        MessageBoxImage.Information);

                    if (response == MessageBoxResult.Yes)
                    {
                        updateService.OpenDownloadPage();
                    }
                    break;

                case UpdateCheckResult.UpToDate:
                    MessageBox.Show(
                        $"Bạn đang sử dụng phiên bản mới nhất ({AppMetadata.Version}).",
                        "Không có cập nhật",
                        MessageBoxButton.OK,
                        MessageBoxImage.Information);
                    break;

                case UpdateCheckResult.Error:
                    MessageBox.Show(
                        $"Không thể kiểm tra cập nhật:\n{updateService.ErrorMessage}",
                        "Lỗi",
                        MessageBoxButton.OK,
                        MessageBoxImage.Warning);
                    break;
            }
        }
        catch (Exception ex)
        {
            MessageBox.Show(
                $"Lỗi: {ex.Message}",
                "Lỗi",
                MessageBoxButton.OK,
                MessageBoxImage.Error);
        }
        finally
        {
            CheckUpdateButton.IsEnabled = true;
            CheckUpdateButton.Content = "Kiểm tra cập nhật";
        }
    }
}
