using System.Windows;
using System.Windows.Media;
using System.Windows.Shapes;
using GoNhanh.Core;
using GoNhanh.Services;

namespace GoNhanh.Views;

/// <summary>
/// Onboarding window for first-time setup
/// Matches macOS OnboardingView
/// </summary>
public partial class OnboardingWindow : Window
{
    private readonly SettingsService _settings;
    private readonly Ellipse[] _dots;
    private readonly SolidColorBrush _activeBrush;
    private readonly SolidColorBrush _inactiveBrush = new(Color.FromRgb(209, 213, 219)); // #D1D5DB

    public OnboardingWindow(SettingsService settings)
    {
        InitializeComponent();
        _settings = settings;
        _dots = new[] { Dot1, Dot2, Dot3 };
        _activeBrush = (SolidColorBrush)FindResource("PrimaryBrush");
        UpdateUI();
    }

    private void NextPage_Click(object sender, RoutedEventArgs e)
    {
        if (PageTabs.SelectedIndex < PageTabs.Items.Count - 1)
        {
            PageTabs.SelectedIndex++;
            UpdateUI();
        }
        else
        {
            Finish_Click();
        }
    }

    private void PrevPage_Click(object sender, RoutedEventArgs e)
    {
        if (PageTabs.SelectedIndex > 0)
        {
            PageTabs.SelectedIndex--;
            UpdateUI();
        }
    }

    private void SelectTelex_Click(object sender, RoutedEventArgs e)
    {
        OnboardTelexRadio.IsChecked = true;
        UpdateModeSelection();
    }

    private void SelectVni_Click(object sender, RoutedEventArgs e)
    {
        OnboardVniRadio.IsChecked = true;
        UpdateModeSelection();
    }

    private void UpdateModeSelection()
    {
        bool isTelexSelected = OnboardTelexRadio.IsChecked == true;

        // Update checkmark colors
        TelexCheck.Foreground = isTelexSelected ? _activeBrush : _inactiveBrush;
        VniCheck.Foreground = isTelexSelected ? _inactiveBrush : _activeBrush;

        // Update button borders
        TelexButton.BorderBrush = isTelexSelected
            ? new SolidColorBrush(Color.FromRgb(147, 197, 253)) // blue-300
            : new SolidColorBrush(Color.FromRgb(229, 231, 235)); // gray-200

        VniButton.BorderBrush = isTelexSelected
            ? new SolidColorBrush(Color.FromRgb(229, 231, 235))
            : new SolidColorBrush(Color.FromRgb(147, 197, 253));

        // Update button backgrounds
        TelexButton.Background = isTelexSelected
            ? new SolidColorBrush(Color.FromRgb(239, 246, 255)) // blue-50
            : new SolidColorBrush(Color.FromRgb(250, 250, 250));

        VniButton.Background = isTelexSelected
            ? new SolidColorBrush(Color.FromRgb(250, 250, 250))
            : new SolidColorBrush(Color.FromRgb(239, 246, 255));
    }

    private void Finish_Click()
    {
        // Save selected method
        _settings.CurrentMethod = OnboardTelexRadio.IsChecked == true
            ? InputMethod.Telex
            : InputMethod.VNI;
        _settings.IsFirstRun = false;
        _settings.Save();

        Close();
    }

    private void UpdateUI()
    {
        UpdateDots();
        UpdateButtons();
        if (PageTabs.SelectedIndex == 1)
        {
            UpdateModeSelection();
        }
    }

    private void UpdateDots()
    {
        for (int i = 0; i < _dots.Length; i++)
        {
            _dots[i].Fill = i == PageTabs.SelectedIndex ? _activeBrush : _inactiveBrush;
        }
    }

    private void UpdateButtons()
    {
        // Show/hide back button
        BackButton.Visibility = PageTabs.SelectedIndex > 0 ? Visibility.Visible : Visibility.Collapsed;

        // Update next button text
        NextButton.Content = PageTabs.SelectedIndex == PageTabs.Items.Count - 1
            ? "Hoàn tất"
            : "Tiếp tục";
    }
}
