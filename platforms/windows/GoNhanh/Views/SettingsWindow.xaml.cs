using System.Windows;
using GoNhanh.Core;
using GoNhanh.Services;

namespace GoNhanh.Views;

/// <summary>
/// Settings window for GoNhanh
/// </summary>
public partial class SettingsWindow : Window
{
    private readonly SettingsService _settings;

    public event Action? SettingsChanged;

    public SettingsWindow(SettingsService settings)
    {
        InitializeComponent();
        _settings = settings;
        LoadSettings();
    }

    private void LoadSettings()
    {
        // Input method
        TelexRadio.IsChecked = _settings.CurrentMethod == InputMethod.Telex;
        VniRadio.IsChecked = _settings.CurrentMethod == InputMethod.VNI;

        // Tone style
        ModernToneRadio.IsChecked = _settings.UseModernTone;
        OldToneRadio.IsChecked = !_settings.UseModernTone;

        // Auto start
        AutoStartCheckbox.IsChecked = _settings.AutoStart;
    }

    private void InputMethod_Changed(object sender, RoutedEventArgs e)
    {
        if (TelexRadio.IsChecked == true)
            _settings.CurrentMethod = InputMethod.Telex;
        else if (VniRadio.IsChecked == true)
            _settings.CurrentMethod = InputMethod.VNI;

        _settings.Save();
        SettingsChanged?.Invoke();
    }

    private void ToneStyle_Changed(object sender, RoutedEventArgs e)
    {
        _settings.UseModernTone = ModernToneRadio.IsChecked == true;
        _settings.Save();
        SettingsChanged?.Invoke();
    }

    private void AutoStart_Changed(object sender, RoutedEventArgs e)
    {
        _settings.AutoStart = AutoStartCheckbox.IsChecked == true;
        _settings.Save();
    }

    private void Reset_Click(object sender, RoutedEventArgs e)
    {
        var result = MessageBox.Show(
            "Bạn có chắc muốn đặt lại tất cả cài đặt về mặc định?",
            "Xác nhận",
            MessageBoxButton.YesNo,
            MessageBoxImage.Question);

        if (result == MessageBoxResult.Yes)
        {
            _settings.Reset();
            LoadSettings();
            SettingsChanged?.Invoke();
        }
    }

    private void Close_Click(object sender, RoutedEventArgs e)
    {
        Close();
    }
}
