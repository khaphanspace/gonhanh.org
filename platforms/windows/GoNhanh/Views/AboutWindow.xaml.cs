using System.Diagnostics;
using System.Windows;
using System.Windows.Navigation;
using GoNhanh.Core;

namespace GoNhanh.Views;

/// <summary>
/// About window showing app info and links
/// Matches macOS AboutView exactly
/// </summary>
public partial class AboutWindow : Window
{
    public AboutWindow()
    {
        InitializeComponent();
        LoadMetadata();
    }

    private void LoadMetadata()
    {
        // App name
        AppNameText.Text = AppMetadata.DisplayName;

        // Version & Build
        VersionText.Text = AppMetadata.Version;
        BuildText.Text = AppMetadata.BuildNumber;

        // Info section
        AuthorText.Text = AppMetadata.Author;
        EmailText.Text = AppMetadata.AuthorEmail;
        EmailLink.NavigateUri = new Uri($"mailto:{AppMetadata.AuthorEmail}");
        TechStackText.Text = AppMetadata.TechStack;
        LicenseText.Text = AppMetadata.License;

        // Copyright
        CopyrightText.Text = AppMetadata.Copyright;
    }

    private void Hyperlink_RequestNavigate(object sender, RequestNavigateEventArgs e)
    {
        OpenUrl(e.Uri.AbsoluteUri);
        e.Handled = true;
    }

    private void Website_Click(object sender, RoutedEventArgs e)
    {
        OpenUrl(AppMetadata.Website);
    }

    private void GitHub_Click(object sender, RoutedEventArgs e)
    {
        OpenUrl(AppMetadata.Repository);
    }

    private void Issues_Click(object sender, RoutedEventArgs e)
    {
        OpenUrl(AppMetadata.IssuesUrl);
    }

    private void LinkedIn_Click(object sender, RoutedEventArgs e)
    {
        OpenUrl(AppMetadata.AuthorLinkedin);
    }

    private static void OpenUrl(string url)
    {
        try
        {
            Process.Start(new ProcessStartInfo
            {
                FileName = url,
                UseShellExecute = true
            });
        }
        catch
        {
            // Ignore errors opening browser/email client
        }
    }
}
