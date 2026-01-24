# Installation Guide - Gõ Nhanh for Windows

## Option 1: Pre-built Binary (Recommended)

### Download

1. Visit the [Releases page](https://github.com/gonhanh/gonhanh/releases)
2. Download the latest `gonhanh-windows-x64.zip`
3. Extract to a permanent location (e.g., `C:\Program Files\GoNhanh\`)

### Install

1. **Run the executable**:
   ```
   gonhanh.exe
   ```

2. **System tray icon** should appear in the notification area

3. **Enable auto-start** (optional):
   - Right-click tray icon → Settings
   - Check "Run on Windows startup"
   - Click Apply

4. **Configure shortcuts** (optional):
   - Right-click tray icon → Settings
   - Click "Shortcuts..." button
   - Add custom text abbreviations

### First Use

1. **Toggle IME**: Press `Ctrl+Space` or click tray icon
2. **Test Vietnamese**: Type "tieng viet" → "tiếng việt"
3. **Switch method**: Right-click tray icon → VNI (if preferred)

## Option 2: Build from Source

### Prerequisites

1. **Install Visual Studio 2022** (or Build Tools):
   - Download from https://visualstudio.microsoft.com/downloads/
   - Select "Desktop development with C++"
   - Include "Windows 10 SDK" (10.0.19041.0+)

2. **Install CMake**:
   ```powershell
   winget install Kitware.CMake
   ```

3. **Install Rust**:
   ```powershell
   winget install Rustlang.Rustup
   rustup default stable
   ```

### Build Steps

1. **Clone repository**:
   ```bash
   git clone https://github.com/gonhanh/gonhanh.git
   cd gonhanh
   ```

2. **Build Rust core** (optional - CMake does this automatically):
   ```bash
   cd core
   cargo build --release
   cd ..
   ```

3. **Configure CMake**:
   ```bash
   cd platforms/windows
   cmake -B build -G "Visual Studio 17 2022" -A x64
   ```

4. **Build release binary**:
   ```bash
   cmake --build build --config Release
   ```

5. **Output**: `build/Release/gonhanh.exe` (~3-4 MB)

### Development Build

For faster iteration during development:

```bash
# Debug build (no optimization, includes debug symbols)
cmake --build build --config Debug

# Output: build/Debug/gonhanh.exe (~10-15 MB)
```

### CMake Options

```bash
# Custom Rust target directory
cmake -B build -DRUST_TARGET_DIR=/path/to/target

# Force rebuild
cmake --build build --config Release --clean-first
```

## Verification

### Check Binary Size

```powershell
# Release build should be 3-5 MB
Get-Item build/Release/gonhanh.exe | Select-Object Name,Length
```

### Test Keyboard Hook

1. Open Notepad
2. Ensure GoNhanh is running (tray icon visible)
3. Press `Ctrl+Space` to enable IME
4. Type: `tieng viet` → should transform to `tiếng việt`

### View Debug Logs

Use [DebugView](https://learn.microsoft.com/en-us/sysinternals/downloads/debugview):

1. Download DebugView from Sysinternals
2. Run as Administrator
3. Enable "Capture Global Win32"
4. Watch for `[INFO]` and `[ERROR]` messages from GoNhanh

## Uninstallation

### Remove Binary

1. **Exit GoNhanh**: Right-click tray icon → Exit
2. **Delete executable**: Remove `gonhanh.exe`
3. **Remove auto-start** (if enabled):
   - Press `Win+R` → `regedit`
   - Navigate to `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
   - Delete "GoNhanh" entry

### Clean Registry (Optional)

Remove all settings:

```powershell
# Remove all GoNhanh settings
Remove-Item -Path "HKCU:\Software\GoNhanh" -Recurse -Force
```

Or use Registry Editor:
1. Press `Win+R` → `regedit`
2. Navigate to `HKEY_CURRENT_USER\Software\GoNhanh`
3. Right-click → Delete

## Troubleshooting

### Debugging (App Not Working)

If GoNhanh doesn't work or crashes on startup, use these debugging methods:

#### Method 1: Debug Build with Console (Recommended)

Build with debug console enabled to see detailed logs:

```bash
cd platforms/windows
cmake -B build -G "Visual Studio 17 2022" -A x64 -DENABLE_DEBUG_CONSOLE=ON
cmake --build build --config Debug
```

Run `build/Debug/gonhanh.exe` - a console window will appear showing all logs:
```
═══════════════════════════════════════════════════════════
  GoNhanh Debug Console
  Vietnamese Input Method - Windows C++ Implementation
═══════════════════════════════════════════════════════════

[STARTUP] GoNhanh starting...
[INFO] [2026-01-13 08:30:00] Rust engine initialized
[INFO] [2026-01-13 08:30:00] Settings loaded from Registry
[INFO] [2026-01-13 08:30:00] Per-app mode states loaded
[INFO] [2026-01-13 08:30:00] Keyboard hook installed
[INFO] [2026-01-13 08:30:00] System tray created
[INFO] [2026-01-13 08:30:00] GoNhanh started successfully
```

#### Method 2: DebugView (For Release Builds)

Download DebugView from [Sysinternals](https://learn.microsoft.com/en-us/sysinternals/downloads/debugview):

1. **Download**: https://download.sysinternals.com/files/DebugView.zip
2. **Extract** and run `Dbgview64.exe` **as Administrator**
3. **Enable**: Capture → Capture Global Win32
4. **Run**: GoNhanh and watch logs appear in DebugView

Example output:
```
[2752] [INFO] [2026-01-13 08:30:00] Rust engine initialized
[2752] [INFO] [2026-01-13 08:30:00] Settings loaded from Registry
[2752] [ERROR] [2026-01-13 08:30:01] Failed to install keyboard hook
```

#### Method 3: Windows Event Viewer

Check for crash reports:

1. Press `Win+R` → `eventvwr`
2. Navigate to: Windows Logs → Application
3. Filter by: Source "Application Error" or "Windows Error Reporting"
4. Look for entries with "gonhanh.exe"

#### Method 4: Check System Tray

GoNhanh runs in background without visible window:

1. **Check system tray** (bottom-right corner, near clock)
2. **Click arrow** (^) to show hidden icons
3. **Look for** GoNhanh icon
4. **Right-click** icon for menu

If icon doesn't appear:
- Check Windows notification area settings
- Taskbar settings → Notification area → Always show icon

#### Common Issues

**App starts but does nothing:**
- System tray icon should appear - check hidden icons area
- Press `Ctrl+Space` to toggle IME (default hotkey)
- Check if IME is enabled via tray icon menu

**Immediate crash on startup:**
- Run debug build with console to see error
- Check if another GoNhanh instance is running (Task Manager)
- Verify Rust core was built: `core/target/release/gonhanh_core.lib` exists

**Keyboard hook installation fails:**
- Check if another keyboard hook tool is running (AutoHotkey, gaming macros)
- Run as Administrator to bypass hook restrictions
- Restart Windows Explorer: `Ctrl+Shift+Esc` → Windows Explorer → Restart

### Build Errors

#### Error: "CMake not found"
```powershell
# Install CMake
winget install Kitware.CMake

# Add to PATH
$env:Path += ";C:\Program Files\CMake\bin"
```

#### Error: "MSVC not found"
- Install Visual Studio 2022 with "Desktop development with C++" workload
- Or install "Build Tools for Visual Studio 2022"

#### Error: "Rust not found"
```powershell
# Install Rust
winget install Rustlang.Rustup

# Verify installation
rustc --version
cargo --version
```

#### Error: "Windows SDK not found"
- Open Visual Studio Installer
- Modify installation → Individual components
- Check "Windows 10 SDK (10.0.19041.0)" or newer

### Runtime Errors

#### "Failed to install keyboard hook"
- **Cause**: Another keyboard hook is blocking installation
- **Fix**: Close other keyboard tools (AutoHotkey, gaming macros, etc.)

#### "Failed to create system tray icon"
- **Cause**: System tray initialization failed
- **Fix**: Restart Windows Explorer (`Ctrl+Shift+Esc` → Windows Explorer → Restart)

#### IME not working in elevated apps
- **Cause**: Windows security prevents non-admin process from injecting into admin process
- **Fix**: Run GoNhanh as Administrator (not recommended for normal use)

### Performance Issues

#### High CPU usage
- Disable per-app mode if not needed
- Check Windows Task Manager for conflicting keyboard hooks
- Update to latest release (may contain performance fixes)

#### Slow response time
- Check if antivirus is scanning the executable on every keystroke
- Add `gonhanh.exe` to antivirus exclusion list

## Advanced Configuration

### Custom Registry Settings

Edit `HKCU\Software\GoNhanh` manually:

```
Enabled (DWORD)           = 1     # IME enabled
Method (DWORD)            = 0     # 0=Telex, 1=VNI
PerApp (DWORD)            = 1     # Per-app mode
AutoStart (DWORD)         = 1     # Run on startup
Sound (DWORD)             = 1     # Toggle sound
ModernTone (DWORD)        = 1     # Modern tone marks
AutoRestore (DWORD)       = 1     # Auto-restore English
AutoCapitalize (DWORD)    = 1     # Auto-capitalize
SkipWShortcut (DWORD)     = 0     # Skip W shortcut
BracketShortcut (DWORD)   = 1     # Bracket shortcuts
EscRestore (DWORD)        = 1     # Esc to restore
```

### Custom Shortcuts

Edit `HKCU\Software\GoNhanh` → `Shortcuts` (REG_MULTI_SZ):

```
Format: trigger\0replacement\0trigger\0replacement\0\0

Example:
"brb" → "be right back"
"omw" → "on my way"
"tysm" → "thank you so much"
```

### Per-App Override

Add custom app states in `HKCU\Software\GoNhanh\AppStates`:

```
chrome.exe (DWORD) = 0    # Disabled in Chrome
code.exe (DWORD) = 1      # Enabled in VS Code
```

## System Requirements

- **OS**: Windows 10 version 1903+ or Windows 11
- **CPU**: x64 processor (Intel/AMD)
- **RAM**: 10 MB minimum
- **Disk**: 5 MB free space
- **Display**: DPI-aware (supports high-DPI displays)

## Security Notes

- **No admin required**: Runs as normal user process
- **No network access**: Fully offline, no telemetry
- **Registry only**: Settings stored in HKCU (user registry)
- **Open source**: Full code audit available on GitHub

## Support

- **Issues**: https://github.com/gonhanh/gonhanh/issues
- **Wiki**: https://github.com/gonhanh/gonhanh/wiki
- **Discussions**: https://github.com/gonhanh/gonhanh/discussions
