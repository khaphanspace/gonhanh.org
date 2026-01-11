using System.Windows;
using GoNhanh.Core;
using GoNhanh.Services;

namespace GoNhanh.Views;

/// <summary>
/// Settings configuration window
/// </summary>
public partial class SettingsWindow : Window
{
    private readonly ImeService _imeService;

    public SettingsWindow(ImeService imeService)
    {
        _imeService = imeService;
        InitializeComponent();
        LoadSettings();
    }

    private void LoadSettings()
    {
        var settings = SettingsService.Instance.Load();

        // Input method
        TelexRadio.IsChecked = settings.Method == InputMethod.Telex;
        VniRadio.IsChecked = settings.Method == InputMethod.VNI;

        // Options
        ModernToneCheckbox.IsChecked = settings.ModernTone;
        WShortcutCheckbox.IsChecked = settings.WShortcut;
        BracketShortcutCheckbox.IsChecked = settings.BracketShortcut;
        EscRestoreCheckbox.IsChecked = settings.EscRestore;
        FreeToneCheckbox.IsChecked = settings.FreeTone;
        EnglishAutoRestoreCheckbox.IsChecked = settings.EnglishAutoRestore;
        AutoCapitalizeCheckbox.IsChecked = settings.AutoCapitalize;

        // System
        LaunchAtLoginCheckbox.IsChecked = settings.LaunchAtLogin;
        SoundEnabledCheckbox.IsChecked = settings.SoundEnabled;
    }

    private void SaveButton_Click(object sender, RoutedEventArgs e)
    {
        var settings = SettingsService.Instance.Load();

        // Input method
        settings.Method = TelexRadio.IsChecked == true ? InputMethod.Telex : InputMethod.VNI;

        // Options
        settings.ModernTone = ModernToneCheckbox.IsChecked == true;
        settings.WShortcut = WShortcutCheckbox.IsChecked == true;
        settings.BracketShortcut = BracketShortcutCheckbox.IsChecked == true;
        settings.EscRestore = EscRestoreCheckbox.IsChecked == true;
        settings.FreeTone = FreeToneCheckbox.IsChecked == true;
        settings.EnglishAutoRestore = EnglishAutoRestoreCheckbox.IsChecked == true;
        settings.AutoCapitalize = AutoCapitalizeCheckbox.IsChecked == true;

        // System
        settings.LaunchAtLogin = LaunchAtLoginCheckbox.IsChecked == true;
        settings.SoundEnabled = SoundEnabledCheckbox.IsChecked == true;

        // Save and apply
        SettingsService.Instance.Save(settings);
        _imeService.ApplySettings(settings);

        Close();
    }

    private void CancelButton_Click(object sender, RoutedEventArgs e)
    {
        Close();
    }
}
