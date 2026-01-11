using System;
using System.Threading;
using System.Windows;
using GoNhanh.Core;
using GoNhanh.Services;
using GoNhanh.Views;

namespace GoNhanh;

/// <summary>
/// Application entry point
/// </summary>
public partial class App : System.Windows.Application
{
    private static Mutex? _mutex;
    private ImeService? _imeService;
    private TrayIcon? _trayIcon;

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        // Single instance check
        const string mutexName = "GoNhanh-SingleInstance-Mutex";
        _mutex = new Mutex(true, mutexName, out bool createdNew);

        if (!createdNew)
        {
            System.Windows.MessageBox.Show(
                "Gõ Nhanh is already running.",
                "Gõ Nhanh",
                System.Windows.MessageBoxButton.OK,
                System.Windows.MessageBoxImage.Information);
            Shutdown();
            return;
        }

        try
        {
            // Initialize services
            _imeService = new ImeService();

            // Load settings
            var settings = SettingsService.Instance.Load();
            _imeService.ApplySettings(settings);

            // Show onboarding on first run
            if (SettingsService.IsFirstRun)
            {
                var onboarding = new OnboardingWindow();
                if (onboarding.ShowDialog() != true)
                {
                    Shutdown();
                    return;
                }
            }

            // Start IME service
            _imeService.Start();

            // Create system tray icon
            _trayIcon = new TrayIcon(_imeService);
            _trayIcon.Show();
        }
        catch (Exception ex)
        {
            System.Windows.MessageBox.Show(
                $"Failed to start Gõ Nhanh:\n\n{ex.Message}",
                "Error",
                System.Windows.MessageBoxButton.OK,
                System.Windows.MessageBoxImage.Error);
            Shutdown();
        }
    }

    protected override void OnExit(ExitEventArgs e)
    {
        _trayIcon?.Dispose();
        _imeService?.Stop();
        _imeService?.Dispose();
        _mutex?.ReleaseMutex();
        _mutex?.Dispose();

        base.OnExit(e);
    }
}
