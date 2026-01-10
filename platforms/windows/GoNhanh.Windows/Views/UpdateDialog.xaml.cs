using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using GoNhanh.Services;

namespace GoNhanh.Views;

public sealed partial class UpdateDialog : ContentDialog
{
    public UpdateDialog()
    {
        InitializeComponent();
        Loaded += OnLoaded;
    }

    private async void OnLoaded(object sender, RoutedEventArgs e)
    {
        var updateService = new UpdateService();
        try
        {
            var info = await updateService.CheckForUpdateAsync();
            if (info == null)
            {
                StatusText.Text = $"Ban da su dung phien ban moi nhat ({UpdateService.CurrentVersion})";
            }
            else
            {
                StatusText.Text = $"Co phien ban moi: {info.Version}";
                ActionButton.Content = "Tai xuong";
                ActionButton.Visibility = Visibility.Visible;
            }
        }
        catch (Exception ex)
        {
            StatusText.Text = $"Loi: {ex.Message}";
            ActionButton.Content = "Thu lai";
            ActionButton.Visibility = Visibility.Visible;
        }
    }

    private void Action_Click(object sender, RoutedEventArgs e)
    {
        Hide();
    }
}
