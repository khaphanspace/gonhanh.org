using GoNhanh.Core;
using GoNhanh.Services;
using Xunit;

namespace GoNhanh.Tests;

public class SettingsServiceTests
{
    [Fact]
    public void Instance_ReturnsSameInstance()
    {
        var instance1 = SettingsService.Instance;
        var instance2 = SettingsService.Instance;
        Assert.Same(instance1, instance2);
    }

    [Fact]
    public void Load_ReturnsSettings()
    {
        var settings = SettingsService.Instance.Load();
        Assert.NotNull(settings);
    }

    [Fact]
    public void Load_DefaultMethod_IsTelex()
    {
        var settings = SettingsService.Instance.Load();
        // Default should be Telex
        Assert.Equal(InputMethod.Telex, settings.Method);
    }

    [Fact]
    public void AppSettings_HasDefaultValues()
    {
        var settings = new AppSettings();

        // Check defaults
        Assert.Equal(InputMethod.Telex, settings.Method);
        Assert.True(settings.ModernTone);
        Assert.True(settings.WShortcut);
        Assert.True(settings.BracketShortcut);
        Assert.True(settings.EscRestore);
        Assert.True(settings.FreeTone);
        Assert.True(settings.EnglishAutoRestore);
        Assert.True(settings.AutoCapitalize);
        Assert.False(settings.LaunchAtLogin);
        Assert.True(settings.SoundEnabled);
    }

    [Fact]
    public void InputMethod_HasCorrectValues()
    {
        Assert.Equal(0, (int)InputMethod.Telex);
        Assert.Equal(1, (int)InputMethod.VNI);
    }
}
