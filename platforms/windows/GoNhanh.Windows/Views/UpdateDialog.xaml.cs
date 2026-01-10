using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using GoNhanh.Services;

namespace GoNhanh.Views;

public sealed partial class UpdateDialog : ContentDialog
{
    private readonly UpdateService _updateService = new();
    private UpdateInfo? _updateInfo;
    private CancellationTokenSource? _cts;

    public UpdateDialog()
    {
        InitializeComponent();
        Loaded += OnLoaded;
        Closed += OnClosed;
    }

    private void OnClosed(ContentDialog sender, ContentDialogClosedEventArgs args)
    {
        _cts?.Cancel();
        _cts?.Dispose();
        _cts = null;
    }

    private async void OnLoaded(object sender, RoutedEventArgs e)
    {
        await CheckForUpdates();
    }

    private async Task CheckForUpdates()
    {
        ShowPanel(CheckingPanel);

        try
        {
            _updateInfo = await _updateService.CheckForUpdateAsync();

            if (_updateInfo == null)
            {
                CurrentVersionText.Text = $"Phien ban hien tai: {UpdateService.CurrentVersion}";
                ShowPanel(UpToDatePanel);
            }
            else
            {
                NewVersionText.Text = $"Phien ban {_updateInfo.Version}";
                ReleaseNotesText.Text = _updateInfo.ReleaseNotes;
                ShowPanel(UpdateAvailablePanel);
            }
        }
        catch (Exception ex)
        {
            ErrorText.Text = $"Loi kiem tra cap nhat: {ex.Message}";
            ShowPanel(ErrorPanel);
        }
    }

    private async void Download_Click(object sender, RoutedEventArgs e)
    {
        if (_updateInfo?.DownloadUrl == null || _updateInfo.FileName == null)
        {
            ErrorText.Text = "Khong tim thay file cap nhat";
            ShowPanel(ErrorPanel);
            return;
        }

        ShowPanel(DownloadingPanel);
        DownloadStatusText.Text = "Dang tai xuong...";

        _cts = new CancellationTokenSource();
        var progress = new Progress<double>(p =>
        {
            DownloadProgress.Value = p * 100;
            DownloadStatusText.Text = $"Dang tai xuong: {p:P0}";
        });

        try
        {
            await _updateService.DownloadUpdateAsync(
                _updateInfo.DownloadUrl,
                _updateInfo.FileName,
                progress,
                _cts.Token);

            DownloadStatusText.Text = "Hoan tat! Dang mo trinh cai dat...";
            await Task.Delay(1000);
            Hide();
        }
        catch (OperationCanceledException)
        {
            DownloadStatusText.Text = "Da huy tai xuong";
        }
        catch (Exception ex)
        {
            ErrorText.Text = $"Loi tai xuong: {ex.Message}";
            ShowPanel(ErrorPanel);
        }
    }

    private async void Retry_Click(object sender, RoutedEventArgs e)
    {
        await CheckForUpdates();
    }

    private void ShowPanel(StackPanel panel)
    {
        CheckingPanel.Visibility = Visibility.Collapsed;
        UpToDatePanel.Visibility = Visibility.Collapsed;
        UpdateAvailablePanel.Visibility = Visibility.Collapsed;
        DownloadingPanel.Visibility = Visibility.Collapsed;
        ErrorPanel.Visibility = Visibility.Collapsed;

        panel.Visibility = Visibility.Visible;
    }
}
