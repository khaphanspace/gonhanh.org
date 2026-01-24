# Windows Phase 2c: UI Integration Guide

Quick guide for WPF/.NET 8 system tray UI development. Keyboard hook (Phase 2b) is complete and ready for integration.

---

## Current State (Phase 2b Complete)

The keyboard hook is fully functional and running:

✓ Global keyboard interception via SetWindowsHookEx(WH_KEYBOARD_LL)
✓ Message-only window with message loop
✓ VK → macOS keycode conversion (46 keys)
✓ Unicode text injection via SendInput
✓ Reentrancy guards (LLKHF_INJECTED + processing_ flag)
✓ Ctrl+Space global toggle
✓ Rust engine integration (ime_key_ext)

**What's Missing:** User-visible UI to control the hook and settings.

---

## Hook Public Interface

Located in `platforms/windows/src/keyboard_hook.h`:

```cpp
class KeyboardHook {
public:
    // Get singleton instance
    static KeyboardHook& Instance();

    // Install hook into system
    bool Install();

    // Remove hook from system
    void Uninstall();

    // Toggle enabled/disabled state
    void Toggle();

    // Check current state
    bool IsEnabled() const { return enabled_; }

    // Set enabled/disabled
    void SetEnabled(bool enabled);
};
```

**Usage from WPF:**

```csharp
// Access via C++ interop
extern IntPtr GetKeyboardHookInstance();

// Toggle via P/Invoke
[DllImport("gonhanh.dll")]
private static extern void KeyboardHook_Toggle();

[DllImport("gonhanh.dll")]
private static extern bool KeyboardHook_IsEnabled();

[DllImport("gonhanh.dll")]
private static extern void KeyboardHook_SetEnabled(bool enabled);

// In WPF event handler
private void ToggleMenuItem_Click(object sender, RoutedEventArgs e) {
    KeyboardHook_Toggle();
    UpdateMenuState();
}
```

---

## System Tray Integration Points

### 1. Menu Item: Toggle Enable/Disable

```csharp
private void UpdateMenuState() {
    bool enabled = KeyboardHook_IsEnabled();

    toggleMenuItem.Header = enabled
        ? "✓ Vietnamese Enabled"
        : "○ Vietnamese Disabled";

    toggleMenuItem.IsChecked = enabled;
}

private void ToggleMenuItem_Click(object sender, RoutedEventArgs e) {
    KeyboardHook_Toggle();
    UpdateMenuState();
}
```

**Behavior:**
- Click toggles enabled_ state
- Ctrl+Space also toggles (global hotkey)
- Update menu to reflect current state

### 2. Menu Item: Input Method Selector

```csharp
[DllImport("gonhanh.dll")]
private static extern void ime_method(uint method);

private void TelexMenuItem_Click(object sender, RoutedEventArgs e) {
    ime_method(0);  // Telex
    SaveSetting("method", "0");
    UpdateMethodMenu();
}

private void VniMenuItem_Click(object sender, RoutedEventArgs e) {
    ime_method(1);  // VNI
    SaveSetting("method", "1");
    UpdateMethodMenu();
}
```

**Settings Storage:** Windows Registry
```
HKEY_CURRENT_USER\Software\GoNhanh\
├── method: 0 (Telex) or 1 (VNI)
├── enabled: 1 (true) or 0 (false)
└── auto_launch: 1 (true) or 0 (false)
```

### 3. Startup Integration

```csharp
private void SetAutoLaunch(bool enable) {
    string appPath = System.Reflection.Assembly.GetExecutingAssembly().Location;

    RegistryKey startupKey = Registry.CurrentUser.OpenSubKey(
        @"Software\Microsoft\Windows\CurrentVersion\Run",
        true
    );

    if (enable) {
        startupKey.SetValue("GoNhanh", appPath);
    } else {
        startupKey.DeleteValue("GoNhanh", false);
    }
}
```

### 4. Tray Icon Right-Click Menu

```xml
<!-- XAML for NotifyIcon context menu -->
<tb:TaskbarIcon.ContextMenu>
    <ContextMenu>
        <MenuItem Header="Vietnamese" x:Name="toggleMenuItem" Click="ToggleMenuItem_Click"/>
        <Separator/>
        <MenuItem Header="Telex" x:Name="telexMenuItem" Click="TelexMenuItem_Click"/>
        <MenuItem Header="VNI" x:Name="vniMenuItem" Click="VniMenuItem_Click"/>
        <Separator/>
        <MenuItem Header="Settings" Click="SettingsMenuItem_Click"/>
        <MenuItem Header="About" Click="AboutMenuItem_Click"/>
        <Separator/>
        <MenuItem Header="Quit" Click="QuitMenuItem_Click"/>
    </ContextMenu>
</tb:TaskbarIcon.ContextMenu>
```

---

## Settings Dialog Implementation

### Registry-Based Persistence

```csharp
private class Settings {
    private const string REGISTRY_PATH = @"Software\GoNhanh";

    public static int GetMethod() {
        var key = Registry.CurrentUser.OpenSubKey(REGISTRY_PATH);
        return (int)(key?.GetValue("method") ?? 0);
    }

    public static void SetMethod(int method) {
        var key = Registry.CurrentUser.CreateSubKey(REGISTRY_PATH);
        key.SetValue("method", method);
    }

    public static bool GetAutoLaunch() {
        var key = Registry.CurrentUser.OpenSubKey(REGISTRY_PATH);
        return (int)(key?.GetValue("auto_launch") ?? 1) != 0;
    }

    public static void SetAutoLaunch(bool enabled) {
        var key = Registry.CurrentUser.CreateSubKey(REGISTRY_PATH);
        key.SetValue("auto_launch", enabled ? 1 : 0);
    }

    public static void LoadDefaults() {
        if (Registry.CurrentUser.OpenSubKey(REGISTRY_PATH) == null) {
            var key = Registry.CurrentUser.CreateSubKey(REGISTRY_PATH);
            key.SetValue("method", 0);      // Telex
            key.SetValue("auto_launch", 1); // Enabled
        }
    }
}
```

### Settings Dialog Window

```csharp
public partial class SettingsWindow : Window {
    public SettingsWindow() {
        InitializeComponent();
        LoadSettings();
    }

    private void LoadSettings() {
        int method = Settings.GetMethod();

        if (method == 0) {
            TelexRadio.IsChecked = true;
        } else {
            VniRadio.IsChecked = true;
        }

        AutoLaunchCheckbox.IsChecked = Settings.GetAutoLaunch();
    }

    private void OkButton_Click(object sender, RoutedEventArgs e) {
        int method = (bool)TelexRadio.IsChecked ? 0 : 1;
        Settings.SetMethod(method);
        ime_method((uint)method);

        bool autoLaunch = (bool)AutoLaunchCheckbox.IsChecked;
        Settings.SetAutoLaunch(autoLaunch);
        SetAutoLaunch(autoLaunch);

        this.Close();
    }
}
```

---

## P/Invoke Bridge (Minimal C++/C# Glue)

Currently keyboard hook is a C++ class. May need to export simplified C functions:

```cpp
// Add to rust_bridge.cpp (or new keyboard_hook_bridge.cpp)
extern "C" {
    // Hook management
    bool KeyboardHook_Install() {
        return gonhanh::KeyboardHook::Instance().Install();
    }

    void KeyboardHook_Uninstall() {
        gonhanh::KeyboardHook::Instance().Uninstall();
    }

    void KeyboardHook_Toggle() {
        gonhanh::KeyboardHook::Instance().Toggle();
    }

    bool KeyboardHook_IsEnabled() {
        return gonhanh::KeyboardHook::Instance().IsEnabled();
    }

    void KeyboardHook_SetEnabled(bool enabled) {
        gonhanh::KeyboardHook::Instance().SetEnabled(enabled);
    }
}
```

C# P/Invoke:

```csharp
[DllImport("gonhanh.exe", CallingConvention = CallingConvention.Cdecl)]
private static extern void KeyboardHook_Toggle();

[DllImport("gonhanh.exe", CallingConvention = CallingConvention.Cdecl)]
private static extern bool KeyboardHook_IsEnabled();

[DllImport("gonhanh.exe", CallingConvention = CallingConvention.Cdecl)]
private static extern void KeyboardHook_SetEnabled(bool enabled);
```

---

## Application Lifecycle

### Startup Sequence

```csharp
// App.xaml.cs or main window code-behind
public partial class App : Application {
    protected override void OnStartup(StartupEventArgs e) {
        base.OnStartup(e);

        // 1. Load settings
        Settings.LoadDefaults();

        // 2. Initialize Rust engine (ime_init already called in main.cpp)

        // 3. Hook is already installed and running (main.cpp:45-48)

        // 4. Show tray icon
        MainWindow.Hide();  // Don't show main window
        InitializeTaskbarIcon();

        // 5. Update menu state
        UpdateMenuState();
    }

    private void InitializeTaskbarIcon() {
        // Set up NotifyIcon from Hardcoregames.Wpf.Notifyicon
        // or use Hardcoregames.Wpf or equivalent
    }

    protected override void OnExit(ExitEventArgs e) {
        // Clean up is handled by main.cpp:59-60
        base.OnExit(e);
    }
}
```

### Message Loop Already Running

Main loop is in `main.cpp`:

```cpp
MSG msg;
while (GetMessage(&msg, NULL, 0, 0)) {
    TranslateMessage(&msg);
    DispatchMessage(&msg);
}
```

**WPF Interaction:** WPF has its own dispatcher. The main message loop keeps the hook alive while WPF UI operates independently.

---

## Threading Model

### Safe from Any Thread

Hook uses only:
- Atomic boolean flags (enabled_, processing_)
- SetWindowsHookEx (system-level, thread-safe)
- Static class members (thread-safe in C++)

**Safe calls:**
```csharp
// Safe from WPF dispatcher thread (UI thread)
KeyboardHook_Toggle();
KeyboardHook_SetEnabled(true);
ime_method(0);

// Safe from background threads
void UpdateHookState() {
    bool enabled = KeyboardHook_IsEnabled();  // Safe read
}
```

No Mutex needed - simplified thread safety model.

---

## Troubleshooting During Integration

### Issue: "Cannot find gonhanh.exe in PATH"

**Cause:** P/Invoke can't locate DLL/EXE

**Solution:**
```csharp
// Load from same directory as WPF app
[DllImport("gonhanh", CallingConvention = CallingConvention.Cdecl)]
// Set DllImportSearchPath
[DefaultDllImportSearchPath(DllImportSearchPathOptions.SafeDirectories)]
private static extern void KeyboardHook_Toggle();
```

### Issue: "Entry point not found"

**Cause:** Exported C function not decorated with `extern "C"`

**Solution:** Ensure keyboard hook exports are wrapped:
```cpp
extern "C" {
    void KeyboardHook_Toggle() { ... }
}
```

### Issue: Hook not active after UI launch

**Cause:** Main message loop not running

**Solution:** Message loop already running in main.cpp before WPF starts. Verify in debugger:
- Breakpoint in LowLevelKeyboardProc
- Press a key
- Should hit breakpoint

### Issue: Settings not persisting

**Cause:** Registry key not being created

**Solution:**
```csharp
// Ensure admin or user registry access
RegistryKey key = Registry.CurrentUser.CreateSubKey(@"Software\GoNhanh");
key.SetValue("method", 0);
key.Close();
```

---

## Example: Complete Minimal UI

```csharp
using System.Windows;
using Microsoft.Win32;
using System.Runtime.InteropServices;

public partial class MainWindow : Window {
    [DllImport("gonhanh", CallingConvention = CallingConvention.Cdecl)]
    private static extern void KeyboardHook_Toggle();

    [DllImport("gonhanh", CallingConvention = CallingConvention.Cdecl)]
    private static extern bool KeyboardHook_IsEnabled();

    [DllImport("gonhanh", CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_method(uint method);

    public MainWindow() {
        InitializeComponent();
        UpdateStatus();
    }

    private void UpdateStatus() {
        bool enabled = KeyboardHook_IsEnabled();
        StatusLabel.Content = enabled ? "Vietnamese ON" : "Vietnamese OFF";
        StatusLabel.Foreground = enabled ? Brushes.Green : Brushes.Gray;
    }

    private void ToggleButton_Click(object sender, RoutedEventArgs e) {
        KeyboardHook_Toggle();
        UpdateStatus();
    }

    private void TelexButton_Click(object sender, RoutedEventArgs e) {
        ime_method(0);
    }

    private void VniButton_Click(object sender, RoutedEventArgs e) {
        ime_method(1);
    }
}
```

---

## Next Steps for Phase 2c

1. **Create WPF App Shell**
   - Use NotifyIcon for system tray
   - Implement SettingsWindow
   - Add context menu

2. **Registry Integration**
   - Settings class for persistence
   - Load on startup
   - Save on change

3. **P/Invoke Bridge**
   - Export C functions from keyboard_hook.cpp
   - Test from C# with simple console app
   - Verify thread safety

4. **System Tray Menu**
   - Toggle Enable/Disable
   - Telex/VNI selector
   - Settings dialog
   - About & Quit

5. **Testing**
   - UI launches without crashing
   - Toggle works (visual feedback)
   - Method selector works
   - Settings persist across restarts
   - Keyboard still processes while UI hidden

6. **Documentation**
   - Update system-architecture.md with WPF UI section
   - Document registry schema
   - Add troubleshooting guide for UI integration

---

**Phase Status:** Phase 2b (Keyboard Hook) Complete ✓
**Next Phase:** Phase 2c (UI & Settings) - Ready to start
**Estimated Timeline:** 2-3 weeks for full WPF integration
**Blockers:** None - hook is production-ready

---

**Last Updated:** 2025-01-12
**Target Audience:** Phase 2c UI developers
**Related Documentation:**
- windows-keyboard-hook-reference.md (detailed reference)
- system-architecture.md (overall design)
