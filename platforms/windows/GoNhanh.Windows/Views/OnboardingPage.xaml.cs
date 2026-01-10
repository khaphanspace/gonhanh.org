using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace GoNhanh.Views;

public sealed partial class OnboardingPage : Page
{
    public event EventHandler? Completed;

    public OnboardingPage()
    {
        InitializeComponent();
    }

    private void Next_Click(object sender, RoutedEventArgs e)
    {
        Completed?.Invoke(this, EventArgs.Empty);
    }
}
