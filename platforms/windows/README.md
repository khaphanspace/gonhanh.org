# Gõ Nhanh - Windows

Vietnamese input method for Windows 10/11: a single native `GoNhanh.exe` (~2 MB, no runtime to
install) with a C++20 Win32 front end and the shared Rust engine linked in as a static library.

## Features

- **Telex and VNI**, shortcuts (text expansion), auto-restore English words, modern tone placement
- **One-click toggle**: click the tray icon (or Ctrl+Space) to switch Vietnamese ↔ English.
  The tray icon is in color for Vietnamese and dimmed for English.
- **Per-app mode**: remembers Vietnamese/English per application
- **Windows 11 look**: Mica backdrop, Fluent cards and toggles, follows light/dark mode and the
  system accent color live; tray menu follows the theme
- **Offline by default**: no network access unless "Tự kiểm tra bản mới" is turned on
- **Single instance**: launching again opens Settings of the running instance
- **Command line** (`bin\gonhanh.exe`, `bin\gn.exe`): `gonhanh` prints `vi`/`en`, `gonhanh vi|en`
  switches, plus `on`/`off`/`toggle`/`telex`/`vni`/`status`, `start`/`quit` and `completion`
  (PowerShell Tab completion). Same commands as macOS/Linux
  (`im-select` convention) for Vim mode switching; see [docs/install-windows.md](../../docs/install-windows.md).
  Settings → "Dòng lệnh gn" adds/removes `bin\` in the user's PATH (`cli_path.cpp`)
- Per-monitor DPI aware, auto-start option, toggle sound

## Performance

Measured on Windows 11 (release build):

| Metric | Value |
|--------|-------|
| Engine time per keystroke (`ime_key_ext`) | ~1–2 µs average, <8 µs p99 (English auto-restore on) |
| Injection, normal apps | one `SendInput` from the hook, no waits |
| Memory, idle in tray after startup | 0.17 MB private working set (Task Manager), 1.6 MB private bytes |
| Memory, Settings window open | ~1.8 MB private working set (CPU renderer, no Direct2D) |
| Memory, after closing Settings | ~0.55 MB private working set |
| Binary size | ~2 MB |

The memory figures were measured with a core that looks the dictionaries up in the embedded text
instead of copying them to the heap (proposed separately for the core); with the current core the
English and Vietnamese dictionaries add ~0.7 MB of heap.

## Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│ Win32 front end (C++20)                                              │
│  keyboard_hook   WH_KEYBOARD_LL hook → engine → TextInjector          │
│                  (one SendInput burst; paced apps on a worker thread  │
│                   with ordered replay of keys typed meanwhile)        │
│  ime_state       single Vietnamese/English switch for tray, hotkey,   │
│                  Settings and per-app mode                            │
│  system_tray     Shell_NotifyIcon, themed menu, state icon            │
│  fluent_ui       CPU raster + DirectWrite text, palette (UISettings), │
│                  Mica frame — shared by the windows below             │
│  settings_window / shortcuts_dialog   custom-painted Fluent windows   │
│  settings / per_app / app_compat / update_checker                     │
├──────────────────────────────────────────────────────────────────────┤
│ FFI (rust_bridge): ime_init, ime_key_ext, ime_method, ime_add_shortcut│
├──────────────────────────────────────────────────────────────────────┤
│ Rust core (../../core): Vietnamese engine, shared with macOS/Linux    │
└──────────────────────────────────────────────────────────────────────┘
```

The low-level hook must return fast (Windows silently removes hooks that keep exceeding
`LowLevelHooksTimeout`). Normal apps get the engine output in a single `SendInput` call made from
the hook. Apps that need paced input (terminals, VS Code/Electron; see `app_compat.cpp`) are served
by a worker thread; while it is busy, the hook swallows later key events and replays them after the
paced output so the app still receives keys in typing order.

The CLI (`cli/gonhanh_cli.cpp`, ~135 KB console exe) finds the app's hidden message-only window
and sends one `SendMessageTimeout` (`control_protocol.h`): wParam is the command, the LRESULT carries
the resulting state. The app applies it on its UI thread through `ime_state`, the same path as the
tray and Ctrl+Space, so per-app mode, sound and the tray icon stay in sync. Round trip ~0.03 ms;
~12 ms per call including process start. It lives in `bin\` because file names are case-insensitive:
`gonhanh.exe` next to `GoNhanh.exe` would be the app itself. The app allows the message through UIPI
(`ChangeWindowMessageFilterEx`), so the CLI still works when GoNhanh runs as administrator.
`tools/cli-test.ps1` tests every command against the built app (run by CI).

DirectWrite, WinHTTP, comdlg32 and dwmapi are delay-loaded, so they are only mapped when
a window or the update check needs them; render resources are released and the working set trimmed
when the windows close.

## Settings storage

`HKCU\Software\GoNhanh`: `Enabled`, `Method` (0 Telex, 1 VNI), feature flags (`SkipWShortcut`,
`BracketShortcut`, `AutoRestore`, `ModernTone`, `AllowForeignConsonants`, `AutoCapitalize`,
`PerApp`, `Sound`, `AutoStart`, `CheckUpdates`), and shortcuts as `REG_MULTI_SZ` pairs in
`Shortcuts` (enabled) and `ShortcutsDisabled`. Per-app states: `HKCU\Software\GoNhanh\AppStates`.

## Build

Requirements: Windows 10/11, Visual Studio 2026 or 2022 Build Tools with the C++ workload,
CMake 3.22+, Rust (MSVC toolchain). Check them with:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/setup/windows.ps1
```

Build (from the repository root):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1          # Release
powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1 -Run     # build + restart app
powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1 -Debug   # with debug console
powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1 -Package # + GoNhanh-<ver>-win-x64.zip
```

Output: `platforms/windows/publish/GoNhanh.exe`. The script picks the newest Visual Studio
(2026 or 2022) and takes the version from the latest `vX.Y.Z` tag (`-DGONHANH_VERSION` for CMake).

CI builds the exe on every push/PR (`.github/workflows/ci.yml`), and the release workflow attaches
`GoNhanh-<version>-win-x64.zip` to GitHub releases.

### Typing test

`tools/typing-test.ps1` types Telex keystrokes into an app far faster than a person can, to check
that a slow app (terminal, Electron) loses no characters. Run it from a different terminal:

```powershell
powershell -ExecutionPolicy Bypass -File platforms/windows/tools/typing-test.ps1 -Target warp -DelayMs 3 -Repeat 3
```

GoNhanh normally ignores keys injected by other software; the script restarts it with
`GONHANH_TEST_INPUT=1` so it accepts them, then restarts it normally.

## Limitations

- Uses a keyboard hook, not the Text Services Framework: no composition window or language bar
- Cannot type into apps running as Administrator unless Gõ Nhanh also runs elevated
- Full-screen games may block the hook
- Browser address bars use the backspace method (selection method needs UI Automation)
- The executable is not code-signed yet, so SmartScreen may warn on first launch

## License

GPL-3.0 - see [LICENSE](../../LICENSE).
