# GoNhanh for Windows

Modern Vietnamese input method for Windows.

## Requirements

- Windows 10/11 (64-bit)
- .NET 8.0 Runtime
- Visual Studio 2022 (for building)
- Rust toolchain with MSVC target

## Development Setup

### 1. Install Prerequisites

```powershell
# Install Rust (if not already installed)
# Download from https://rustup.rs

# Add Windows targets
rustup target add x86_64-pc-windows-msvc

# Install .NET 8.0 SDK
# Download from https://dotnet.microsoft.com/download/dotnet/8.0
```

### 2. Run Setup Script

```powershell
powershell -ExecutionPolicy Bypass -File scripts/setup-windows.ps1
```

### 3. Build Rust Core

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-core-windows.ps1
```

### 4. Build WPF Application

```powershell
cd platforms/windows/GoNhanh
dotnet build -c Release
```

Or open `platforms/windows/GoNhanh/GoNhanh.csproj` in Visual Studio.

## Project Structure

```
platforms/windows/
├── GoNhanh/
│   ├── GoNhanh.csproj      # WPF project file
│   ├── App.xaml            # Application entry
│   ├── Core/
│   │   ├── RustBridge.cs   # P/Invoke FFI to Rust
│   │   ├── KeyboardHook.cs # Low-level keyboard hook
│   │   ├── KeyCodes.cs     # Virtual key constants
│   │   └── TextSender.cs   # SendInput API wrapper
│   ├── Views/
│   │   ├── TrayIcon.cs     # System tray icon
│   │   ├── SettingsWindow  # Settings UI
│   │   ├── OnboardingWindow# First-run wizard
│   │   └── AboutWindow     # About dialog
│   ├── Services/
│   │   └── SettingsService # Registry-based settings
│   ├── Native/
│   │   └── gonhanh_core.dll# Rust core library
│   └── Resources/
│       └── Icons/          # App icons
└── README.md
```

## Architecture

### Keyboard Hook

Uses Win32 Low-Level Keyboard Hook (`SetWindowsHookEx` with `WH_KEYBOARD_LL`) for system-wide key interception. This is similar to `CGEventTap` on macOS.

### Text Insertion

Uses `SendInput` API with `KEYEVENTF_UNICODE` flag for direct Unicode character insertion. Supports:
- Backspace deletion for replacing text
- Unicode character insertion (Vietnamese diacritics)
- Injected key marking to prevent recursive processing

### Settings Storage

Settings are stored in Windows Registry at:
```
HKEY_CURRENT_USER\SOFTWARE\GoNhanh
```

Auto-start is managed via:
```
HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run
```

## Known Issues & Solutions

### Antivirus False Positives

Low-level keyboard hooks may trigger antivirus warnings. Solutions:
- Code sign the application with a valid certificate
- Submit to Microsoft for SmartScreen reputation
- Add to antivirus exclusions (for development)

### UAC / Admin Rights

The application should work without admin rights for most applications. Some elevated apps may not receive keyboard input.

### UWP Apps

Some Windows Store apps may block low-level keyboard hooks due to app container isolation.

## License

GPL-3.0-or-later
