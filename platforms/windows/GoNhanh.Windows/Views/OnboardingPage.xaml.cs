using Microsoft.UI.Xaml.Controls;

namespace GoNhanh.Views;

public sealed partial class OnboardingPage : Page
{
    public event EventHandler? Completed;

    public OnboardingPage()
    {
        InitializeComponent();
    }
}
