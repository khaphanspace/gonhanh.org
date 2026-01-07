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
    private bool _isLoading = true;

    public event Action? SettingsChanged;

    public SettingsWindow(SettingsService settings)
    {
        InitializeComponent();
        _settings = settings;
        LoadSettings();
    }

    private void LoadSettings()
    {
        _isLoading = true;

        EnabledToggle.IsChecked = _settings.IsEnabled;
        MethodCombo.SelectedIndex = (int)_settings.CurrentMethod;
        WShortcutToggle.IsChecked = _settings.AutoWShortcut;
        BracketShortcutToggle.IsChecked = _settings.BracketShortcut;
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

    private void BracketShortcutToggle_Changed(object sender, RoutedEventArgs e)
    {
        if (_isLoading) return;
        _settings.BracketShortcut = BracketShortcutToggle.IsChecked == true;
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
}
