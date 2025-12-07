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
        // Version
        VersionText.Text = $"Version {AppMetadata.Version}";

        // Author
        AuthorText.Text = AppMetadata.Author;
        EmailText.Text = AppMetadata.AuthorEmail;
        EmailLink.NavigateUri = new Uri($"mailto:{AppMetadata.AuthorEmail}");
        LinkedInLink.NavigateUri = new Uri(AppMetadata.AuthorLinkedin);

        // Links
        WebsiteLink.NavigateUri = new Uri(AppMetadata.Website);
        GitHubLink.NavigateUri = new Uri(AppMetadata.Repository);

        // Copyright
        CopyrightText.Text = AppMetadata.Copyright;
    }

    private void Hyperlink_RequestNavigate(object sender, RequestNavigateEventArgs e)
    {
        try
        {
            Process.Start(new ProcessStartInfo
            {
                FileName = e.Uri.AbsoluteUri,
                UseShellExecute = true
            });
        }
        catch
        {
            // Ignore errors opening browser/email client
        }
        e.Handled = true;
    }
}
