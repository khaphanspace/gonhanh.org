using Microsoft.UI.Xaml.Controls;
using GoNhanh.Services;

namespace GoNhanh.Views;

public sealed partial class AboutDialog : ContentDialog
{
    public AboutDialog()
    {
        InitializeComponent();
        VersionText.Text = $"Phien ban {UpdateService.CurrentVersion}";
    }
}
