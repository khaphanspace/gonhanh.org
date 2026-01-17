# Gõ Nhanh - Windows C++ Implementation

Single-exe Vietnamese input method for Windows 10/11, built with C++17 and Rust core engine.

## Features

- **Lightweight**: <5 MB single executable, <10 MB RAM usage
- **Fast**: <1 ms keystroke latency with low-level keyboard hook
- **System Tray**: Always-on background service with tray icon
- **Two Input Methods**: Telex (default) and VNI
- **Global Hotkeys**: Ctrl+Space to toggle IME
- **Settings UI**: Native Windows dialogs with listview controls
- **Per-App Mode**: Remember IME state per application
- **Auto-Start**: Optional Windows startup integration
- **Sound Feedback**: Toggle sound on enable/disable
- **Shortcuts**: Custom text abbreviation expansion
- **App Compatibility**: Special handling for browsers and IDEs
- **DPI Aware**: PerMonitorV2 DPI awareness for high-DPI displays

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Win32 UI (Native C++)                                       │
│  ├─ SetWindowsHookEx (WH_KEYBOARD_LL)                       │
│  ├─ SendInput (Backspace + Unicode)                         │
│  └─ Shell_NotifyIcon (System Tray)                          │
├─────────────────────────────────────────────────────────────┤
│ FFI Bridge (Rust extern "C")                                │
│  ├─ ime_init(), ime_key(), ime_method()                     │
│  └─ UTF-32 ↔ UTF-16 conversion                              │
├─────────────────────────────────────────────────────────────┤
│ Core Engine (Pure Rust)                                     │
│  ├─ 7-stage processing pipeline                             │
│  ├─ Validation matrices (Vietnamese syllable rules)         │
│  └─ Shortcut expansion                                      │
└─────────────────────────────────────────────────────────────┘
```

## Build Requirements

- **CMake** 3.15+
- **MSVC** 2019+ (Visual Studio Build Tools)
- **Rust** 1.70+ (for core engine)
- **Windows SDK** 10.0.19041.0+ (comes with VS Build Tools)

## Build Instructions

```bash
# From repository root
cd platforms/windows

# Configure with CMake
cmake -B build -G "Visual Studio 17 2022" -A x64

# Build debug
cmake --build build --config Debug

# Build release (optimized)
cmake --build build --config Release

# Output: build/Release/gonhanh.exe (~3-4 MB)
```

## Installation

See [INSTALL.md](INSTALL.md) for detailed installation instructions.

## Settings Storage

All settings are stored in Windows Registry:

- **Main Settings**: `HKCU\Software\GoNhanh`
  - `Enabled` (DWORD): IME enabled state
  - `Method` (DWORD): 0=Telex, 1=VNI
  - `PerApp` (DWORD): Per-app mode enabled
  - `AutoStart` (DWORD): Run on Windows startup
  - `Sound` (DWORD): Play sound on toggle
  - `Shortcuts` (REG_MULTI_SZ): Text shortcuts list

- **Per-App States**: `HKCU\Software\GoNhanh\AppStates`
  - Each app (e.g., "chrome.exe") has DWORD value (0/1)

## Known Compatible Apps

- **Browsers**: Chrome, Edge, Firefox, Brave, Opera
- **IDEs**: Visual Studio, VS Code, IntelliJ IDEA, PyCharm
- **Chat**: Discord, Slack
- **Office**: Word, Excel, PowerPoint, Outlook
- **Notepad**: Notepad, Notepad++, Sublime Text

## Limitations

- **No IME API**: Uses keyboard hook instead of Windows IME framework
  - Cannot show composition window like traditional IMEs
  - Cannot integrate with Windows language bar
- **Admin Apps**: Cannot inject input into elevated processes (security limitation)
- **Games**: Full-screen games may block keyboard hook
- **Address Bars**: Browser address bars use backspace method (no selection method yet)

## Performance

- **Startup Time**: <100ms
- **Memory Usage**: <10 MB RSS
- **CPU Usage**: <0.1% idle, <1% active typing
- **Binary Size**: 3-4 MB (release build with static CRT)

## Security

- **No Network Access**: Fully offline, no telemetry
- **Registry Security**: Quoted paths, dynamic buffer allocation
- **No Admin Required**: Runs as normal user process
- **Open Source**: Full source code available for audit

## Troubleshooting

### IME not working in some apps
- Check if app is running as Administrator (hook cannot inject into elevated processes)
- Try restarting the app after GoNhanh is running

### High CPU usage
- Check Windows Task Manager for other keyboard hook processes
- Disable per-app mode if not needed (reduces timer overhead)

### Settings not saved
- Check Registry permissions: `HKCU\Software\GoNhanh` should be writable
- Run `regedit` and verify the key exists

### System tray icon not showing
- Check Windows notification area settings
- Right-click taskbar → Taskbar settings → Notification area → Always show icon

## Development

See [docs/system-architecture.md](../../docs/system-architecture.md) for architecture details.

### Project Structure

```
platforms/windows/
├── CMakeLists.txt          # Build configuration
├── src/
│   ├── main.cpp            # WinMain + message loop
│   ├── keyboard_hook.cpp   # SetWindowsHookEx + SendInput
│   ├── rust_bridge.cpp     # FFI wrappers
│   ├── settings.cpp        # Registry I/O
│   ├── system_tray.cpp     # Shell_NotifyIcon
│   ├── settings_window.cpp # Settings dialog
│   ├── shortcuts_dialog.cpp# Shortcuts editor
│   ├── about_dialog.cpp    # About dialog
│   ├── app_compat.cpp      # App detection
│   ├── per_app.cpp         # Per-app mode tracking
│   └── utils.cpp           # Sound + logging
├── resources/
│   ├── resources.rc        # Icons + manifest
│   ├── manifest.xml        # DPI awareness
│   ├── icon.ico            # Tray icon
│   └── logo.ico            # App logo
└── README.md               # This file
```

### Code Style

- **C++17** standard
- **4-space indentation**
- **RAII** with smart pointers
- **Singleton pattern** for managers
- **Error logging** via OutputDebugStringW

## License

GPL-3.0 - See [LICENSE](../../LICENSE) for details.

## Support

- **Issues**: https://github.com/gonhanh/gonhanh/issues
- **Discussions**: https://github.com/gonhanh/gonhanh/discussions
- **Documentation**: https://github.com/gonhanh/gonhanh/tree/main/docs
