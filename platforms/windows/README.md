# GoNhanh for Windows

Win32 shell written in Rust (`windows-sys`). It uses the `gonhanh-core` crate directly (owns an `Engine`, no FFI)
and links the CRT statically, so the result is one `GoNhanh.exe` that needs no runtime.

## Build

Requirements: Windows 10/11, [Rust](https://rustup.rs/), Visual Studio 2022+ or Build Tools with
"Desktop development with C++" (MSVC linker and `rc.exe`).

```bash
# Git Bash, from the repo root
./scripts/build/windows.sh
```

Runs `cargo test`, builds, and writes `publish/GoNhanh.exe` + `GoNhanh-<version>-win-x64.zip`.
Version comes from the git tag (`GONHANH_VERSION` overrides it).

## Layout

Same folders as the old C# app (`GoNhanh/Core`, `Services`, `Views`) so the two are easy to compare.

| Path | What |
|---|---|
| `src/main.rs`, `src/app.rs` | Startup, single instance, message loop, shared controller (old `App.xaml.cs`) |
| `src/win.rs` | Small Win32 helpers: UTF-16, fonts, URLs, HTTP |
| `src/core/controller.rs` | Key policy (shortcuts, buffer resets, per-app state), Windows counterpart of macOS `keyboardCallback` |
| `src/core/controller_tests.rs` | Types through the controller + the real engine |
| `src/core/keys.rs`, `src/core/hotkey.rs` | Windows VK → macOS keycode mapping, shortcuts (old `KeyCodes.cs`) |
| `src/core/hook.rs`, `src/core/send.rs` | `WH_KEYBOARD_LL`/`WH_MOUSE_LL` hooks, `SendInput` output (old `KeyboardHook.cs`, `TextSender.cs`) |
| `src/services/settings.rs`, `src/services/registry.rs` | `HKCU\SOFTWARE\GoNhanh` + `%APPDATA%\GoNhanh\shortcuts.txt` (old `SettingsService.cs`) |
| `src/services/autostart.rs` | Task Scheduler task for "Khởi động cùng Windows" (logon trigger) and "Chạy với quyền admin" (highest privileges, started without UAC) |
| `src/services/updater.rs`, `src/services/json.rs` | GitHub release check, in-place update (zip asset `*-win-x64.zip`) |
| `src/services/foreground.rs` | Foreground app + non-Latin keyboard detection |
| `src/services/log.rs` | Debug log |
| `src/views/tray.rs`, `src/views/settings_window.rs`, `src/views/dialogs.rs` | Tray, settings window, onboarding/about (old `TrayIcon.cs`, `OnboardingWindow`, `AboutWindow`) |
| `src/views/theme.rs`, `src/views/dark.rs`, `src/views/icon.rs` | Light/dark theme, tray icon drawing |
| `build.rs`, `res/` | Icon, manifest (Common Controls v6, per-monitor DPI), version info |
