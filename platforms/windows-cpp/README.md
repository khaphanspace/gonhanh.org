# Gõ Nhanh - Windows C++ Native Implementation

Pure Win32 C++ implementation with full feature parity to macOS version.

## Prerequisites

### Windows 10/11 Required

1. **Visual Studio 2022** (Community/Professional/Enterprise)
   - Desktop development with C++ workload
   - Windows 10/11 SDK
   - CMake tools for Visual Studio

2. **Rust Toolchain**
   ```powershell
   # Install Rust
   winget install Rustlang.Rust.MSVC

   # Verify installation
   rustc --version
   cargo --version
   ```

3. **CMake 3.20+** (included with VS 2022)

## Build Instructions

### Step 1: Build Rust Core DLL

```powershell
# Navigate to core directory
cd core

# Build release DLL
cargo build --release --target x86_64-pc-windows-msvc

# Copy artifacts to Windows C++ project
mkdir ..\platforms\windows-cpp\lib -Force
copy target\x86_64-pc-windows-msvc\release\gonhanh_core.dll ..\platforms\windows-cpp\lib\
copy target\x86_64-pc-windows-msvc\release\gonhanh_core.dll.lib ..\platforms\windows-cpp\lib\libgonhanh_core.lib
```

### Step 2: Build Windows C++ Application

#### Option A: Visual Studio GUI

1. Open `platforms/windows-cpp/` folder in Visual Studio 2022
2. Visual Studio will detect CMakeLists.txt automatically
3. Select "Release" configuration
4. Build > Build All (Ctrl+Shift+B)
5. Executable: `out/build/x64-Release/bin/gonhanh.exe`

#### Option B: Command Line (MSVC)

```powershell
cd platforms\windows-cpp

# Configure with Visual Studio generator
cmake -B build -G "Visual Studio 17 2022" -A x64

# Build Release
cmake --build build --config Release

# Output
build\bin\gonhanh.exe
```

#### Option C: Command Line (Ninja)

```powershell
cd platforms\windows-cpp

# Configure with Ninja (faster builds)
cmake -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Release

# Build
cmake --build build

# Output
build\bin\gonhanh.exe
```

## Running

### First Run

1. **Run as normal user** (no admin required on Windows 10+)
2. Double-click `gonhanh.exe` or run from command line
3. System tray icon appears (🇻🇳 flag)
4. Toggle Vietnamese input: **Alt+Space** (default)

### Testing

```powershell
# Run executable
.\build\bin\gonhanh.exe

# Open Notepad, type Vietnamese:
# - a + s = á
# - v + i + e + e + t = việt
# - v + i + e + e + t + Space = "việt " (auto-restore to English if invalid)
```

## Troubleshooting

### Error: "gonhanh_core.dll not found"

**Solution:** Ensure DLL is in same directory as .exe

```powershell
copy lib\gonhanh_core.dll build\bin\
```

### Error: "Failed to install keyboard hook"

**Possible causes:**
1. **Antivirus blocking**: Add `gonhanh.exe` to exclusions
2. **UAC prompt**: Confirm elevation if requested
3. **Another IME running**: Close UniKey, OpenKey, EVKey

### Error: "0xc000007b" (Application unable to start)

**Solution:** Mismatch between 32-bit/64-bit. Ensure:
- Rust core built with `x86_64-pc-windows-msvc` target
- CMake configured with `-A x64`

### Hook stops working after some time

**Solution:** Windows disables hooks that timeout (>200ms). This should NOT happen with worker thread architecture. If it does:
1. Check CPU usage (Task Manager)
2. Report issue with steps to reproduce

## Development

### Project Structure

```
platforms/windows-cpp/
├── src/
│   ├── main.cpp           # Entry point, WinMain, message loop
│   ├── RustBridge.h/cpp   # FFI wrapper to libgonhanh_core.dll
│   ├── KeyboardHook.h/cpp # SetWindowsHookEx + worker thread
│   ├── TextSender.h/cpp   # SendInput with Unicode support
│   ├── LockFreeQueue.h    # Lock-free queue (header-only)
│   ├── KeycodeMap.h       # Windows VK → macOS keycode mapping
│   └── Types.h            # Shared types (KeyEvent, etc.)
├── resources/
│   ├── app.ico            # Application icon (TODO)
│   ├── resources.rc       # Resource script (TODO)
│   └── manifest.xml       # Application manifest (TODO)
├── lib/
│   ├── gonhanh_core.dll       # Rust core (from core build)
│   └── libgonhanh_core.lib    # Import library
├── CMakeLists.txt         # Build configuration
└── README.md              # This file
```

### Adding Features

1. **UI changes**: Modify `main.cpp` (Phase 2/3 will add TrayIcon, SettingsWindow)
2. **Core engine config**: Add FFI functions to `RustBridge.h/cpp`
3. **Keyboard behavior**: Modify `KeyboardHook.cpp` worker thread logic

### Debugging

#### Visual Studio Debugger

1. Set breakpoint in `main.cpp:WinMain`
2. F5 to debug
3. Attach to `gonhanh.exe` process if already running (Ctrl+Alt+P)

#### Logging

```cpp
// Add to main.cpp
#include <fstream>
std::ofstream log("C:\\Users\\<user>\\AppData\\Local\\gonhanh\\debug.log");
log << "Hook installed: " << (hook ? "YES" : "NO") << std::endl;
```

## Performance

### Benchmarks (Phase 1 target)

| Metric | Target | Actual (TODO: measure) |
|--------|--------|------------------------|
| Keystroke latency | <1ms | TBD |
| Memory usage | <10MB | TBD |
| Binary size | <5MB | TBD |
| CPU usage (idle) | <0.1% | TBD |
| CPU usage (typing) | <1% | TBD |

### Profiling

```powershell
# Visual Studio Performance Profiler
# Tools > Performance Profiler > CPU Usage
# Start typing Vietnamese, stop profiler, analyze
```

## Known Limitations (Phase 1)

- **No system tray icon** (Phase 2)
- **No settings UI** (Phase 3)
- **No auto-update** (Phase 4)
- **No per-app mode** (Phase 4)
- **No shortcut manager** (Phase 4)

**UWP apps incompatible**: Windows Store apps (Mail, Calculator, Settings) may not receive keyboard hook events due to sandboxing.

## License

GPL-3.0-or-later (same as core engine)

## Credits

- Core engine: Kha Phan
- Windows implementation: Phase 01 (Core Infrastructure)
