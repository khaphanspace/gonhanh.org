using System.Text.Json;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using GoNhanh.Core;
using GoNhanh.Services;

namespace GoNhanh.Views;

public sealed partial class SettingsWindow : Window
{
    private readonly SettingsService _settings = SettingsService.Instance;

    public SettingsWindow()
    {
        InitializeComponent();
        LoadSettings();
        AttachEventHandlers();
    }

    private void LoadSettings()
    {
        InputMethodRadio.SelectedIndex = _settings.InputMethod;
        SkipWShortcutToggle.IsOn = _settings.SkipWShortcut;
        BracketShortcutToggle.IsOn = _settings.BracketShortcut;
        EscRestoreToggle.IsOn = _settings.EscRestore;
        FreeToneToggle.IsOn = _settings.FreeTone;
        ModernToneToggle.IsOn = _settings.ModernTone;
        EnglishAutoRestoreToggle.IsOn = _settings.EnglishAutoRestore;
        AutoCapitalizeToggle.IsOn = _settings.AutoCapitalize;
        SoundToggle.IsOn = _settings.SoundEnabled;
        LaunchAtStartupToggle.IsOn = LaunchAtStartup.IsEnabled;

        LoadShortcuts();
    }

    private void LoadShortcuts()
    {
        try
        {
            var shortcuts = JsonSerializer.Deserialize<List<ShortcutEntry>>(_settings.UserShortcuts) ?? [];
            ShortcutsList.ItemsSource = shortcuts.Select(s => $"{s.Trigger} -> {s.Replacement}").ToList();
        }
        catch
        {
            ShortcutsList.ItemsSource = new List<string>();
        }
    }

    private void AttachEventHandlers()
    {
        InputMethodRadio.SelectionChanged += (s, e) =>
        {
            var method = (byte)InputMethodRadio.SelectedIndex;
            _settings.InputMethod = method;
            RustBridge.ime_method(method);
            SaveSettings();
        };

        SkipWShortcutToggle.Toggled += (s, e) =>
        {
            _settings.SkipWShortcut = SkipWShortcutToggle.IsOn;
            RustBridge.ime_skip_w_shortcut(SkipWShortcutToggle.IsOn);
            SaveSettings();
        };

        BracketShortcutToggle.Toggled += (s, e) =>
        {
            _settings.BracketShortcut = BracketShortcutToggle.IsOn;
            RustBridge.ime_bracket_shortcut(BracketShortcutToggle.IsOn);
            SaveSettings();
        };

        EscRestoreToggle.Toggled += (s, e) =>
        {
            _settings.EscRestore = EscRestoreToggle.IsOn;
            RustBridge.ime_esc_restore(EscRestoreToggle.IsOn);
            SaveSettings();
        };

        FreeToneToggle.Toggled += (s, e) =>
        {
            _settings.FreeTone = FreeToneToggle.IsOn;
            RustBridge.ime_free_tone(FreeToneToggle.IsOn);
            SaveSettings();
        };

        ModernToneToggle.Toggled += (s, e) =>
        {
            _settings.ModernTone = ModernToneToggle.IsOn;
            RustBridge.ime_modern(ModernToneToggle.IsOn);
            SaveSettings();
        };

        EnglishAutoRestoreToggle.Toggled += (s, e) =>
        {
            _settings.EnglishAutoRestore = EnglishAutoRestoreToggle.IsOn;
            RustBridge.ime_english_auto_restore(EnglishAutoRestoreToggle.IsOn);
            SaveSettings();
        };

        AutoCapitalizeToggle.Toggled += (s, e) =>
        {
            _settings.AutoCapitalize = AutoCapitalizeToggle.IsOn;
            RustBridge.ime_auto_capitalize(AutoCapitalizeToggle.IsOn);
            SaveSettings();
        };

        SoundToggle.Toggled += (s, e) =>
        {
            _settings.SoundEnabled = SoundToggle.IsOn;
            SaveSettings();
        };

        LaunchAtStartupToggle.Toggled += (s, e) =>
        {
            if (LaunchAtStartupToggle.IsOn)
                LaunchAtStartup.Enable();
            else
                LaunchAtStartup.Disable();
        };
    }

    private async void AddShortcut_Click(object sender, RoutedEventArgs e)
    {
        var dialog = new ContentDialog
        {
            Title = "Them go tat",
            PrimaryButtonText = "Them",
            CloseButtonText = "Huy",
            XamlRoot = Content.XamlRoot
        };

        var panel = new StackPanel { Spacing = 8 };
        var triggerBox = new TextBox { PlaceholderText = "Viet tat (vd: hn)" };
        var replacementBox = new TextBox { PlaceholderText = "Thay the (vd: Ha Noi)" };
        panel.Children.Add(triggerBox);
        panel.Children.Add(replacementBox);
        dialog.Content = panel;

        if (await dialog.ShowAsync() == ContentDialogResult.Primary)
        {
            var trigger = triggerBox.Text.Trim();
            var replacement = replacementBox.Text.Trim();
            if (string.IsNullOrEmpty(trigger) || string.IsNullOrEmpty(replacement)) return;

            RustBridge.ime_add_shortcut(trigger, replacement);
            UpdateShortcutsList(trigger, replacement);
        }
    }

    private void DeleteShortcut_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is string trigger)
        {
            RustBridge.ime_remove_shortcut(trigger);
            RemoveFromShortcutsList(trigger);
        }
    }

    private void UpdateShortcutsList(string trigger, string replacement)
    {
        var list = JsonSerializer.Deserialize<List<ShortcutEntry>>(_settings.UserShortcuts) ?? [];
        list.Add(new ShortcutEntry(trigger, replacement));
        _settings.UserShortcuts = JsonSerializer.Serialize(list);
        SaveSettings();
        LoadShortcuts();
    }

    private void RemoveFromShortcutsList(string trigger)
    {
        var list = JsonSerializer.Deserialize<List<ShortcutEntry>>(_settings.UserShortcuts) ?? [];
        list.RemoveAll(s => s.Trigger == trigger);
        _settings.UserShortcuts = JsonSerializer.Serialize(list);
        SaveSettings();
        LoadShortcuts();
    }

    private void SaveSettings() => _settings.SaveAll();

    private record ShortcutEntry(string Trigger, string Replacement);
}
