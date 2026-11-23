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

| Path | What |
|---|---|
| `src/controller.rs` | Key policy (shortcuts, buffer resets, per-app state) — Windows counterpart of macOS `keyboardCallback` |
| `src/controller_tests.rs` | Types through the controller + the real engine |
| `src/keys.rs`, `src/hotkey.rs` | Windows VK → macOS keycode mapping, shortcuts |
| `src/settings.rs` | `HKCU\SOFTWARE\GoNhanh` + `%APPDATA%\GoNhanh\shortcuts.txt` |
| `src/updater.rs`, `src/json.rs` | GitHub release check, in-place update (zip asset `*-win-x64.zip`) |
| `src/win/hook.rs`, `src/win/send.rs` | `WH_KEYBOARD_LL`/`WH_MOUSE_LL` hooks, `SendInput` output |
| `src/win/foreground.rs` | Foreground app + non-Latin keyboard detection |
| `src/win/tray.rs`, `src/win/settings_window.rs`, `src/win/dialogs.rs` | Tray, settings window, onboarding/about |
| `build.rs`, `res/` | Icon, manifest (Common Controls v6, per-monitor DPI), version info |
