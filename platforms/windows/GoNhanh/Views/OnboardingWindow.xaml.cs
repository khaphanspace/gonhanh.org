using System.Windows;
using GoNhanh.Core;
using GoNhanh.Services;

namespace GoNhanh.Views;

/// <summary>
/// First-run onboarding window
/// </summary>
public partial class OnboardingWindow : Window
{
    public OnboardingWindow()
    {
        InitializeComponent();
    }

    private void StartButton_Click(object sender, RoutedEventArgs e)
    {
        // Save initial settings
        var settings = SettingsData.Defaults;
        settings.Method = TelexRadio.IsChecked == true ? InputMethod.Telex : InputMethod.VNI;
        settings.LaunchAtLogin = LaunchAtLoginCheckbox.IsChecked == true;

        SettingsService.Instance.Save(settings);

        DialogResult = true;
        Close();
    }
}
