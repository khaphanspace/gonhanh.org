using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.UI;
using GoNhanh.Core;
using GoNhanh.Services;

namespace GoNhanh.Views;

public sealed partial class OnboardingPage : Page
{
    private int _currentStep = 1;
    public event EventHandler? Completed;

    public OnboardingPage()
    {
        InitializeComponent();
        UpdateStepUI();
    }

    private void Back_Click(object sender, RoutedEventArgs e)
    {
        if (_currentStep > 1)
        {
            _currentStep--;
            UpdateStepUI();
        }
    }

    private void Next_Click(object sender, RoutedEventArgs e)
    {
        if (_currentStep < 3)
        {
            _currentStep++;
            UpdateStepUI();
        }
        else
        {
            SaveSettings();
            Completed?.Invoke(this, EventArgs.Empty);
        }
    }

    private void UpdateStepUI()
    {
        Step1Panel.Visibility = _currentStep == 1 ? Visibility.Visible : Visibility.Collapsed;
        Step2Panel.Visibility = _currentStep == 2 ? Visibility.Visible : Visibility.Collapsed;
        Step3Panel.Visibility = _currentStep == 3 ? Visibility.Visible : Visibility.Collapsed;

        var accentBrush = new SolidColorBrush(Color.FromArgb(255, 0, 120, 215));
        var defaultBrush = new SolidColorBrush(Color.FromArgb(255, 128, 128, 128));

        Step1Indicator.Fill = _currentStep >= 1 ? accentBrush : defaultBrush;
        Step2Indicator.Fill = _currentStep >= 2 ? accentBrush : defaultBrush;
        Step3Indicator.Fill = _currentStep >= 3 ? accentBrush : defaultBrush;

        BackButton.Visibility = _currentStep > 1 ? Visibility.Visible : Visibility.Collapsed;
        NextButton.Content = _currentStep == 3 ? "Hoan tat" : "Tiep tuc";
    }

    private void SaveSettings()
    {
        var method = (byte)InputMethodRadio.SelectedIndex;
        SettingsService.Instance.InputMethod = method;
        RustBridge.ime_method(method);

        if (LaunchAtStartupToggle.IsOn)
            LaunchAtStartup.Enable();

        SettingsService.Instance.SaveAll();
    }
}
