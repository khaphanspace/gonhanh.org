#pragma once

namespace gonhanh {

// Single source of truth for Vietnamese / English mode.
// Every entry point (tray click, tray menu, Ctrl+Space, Settings window) goes through here
// so the engine, keyboard hook, tray icon, Settings window and per-app memory stay in sync.

// User-initiated switch: saves the setting, remembers it for the current app (per-app mode)
// and plays the toggle sound if enabled.
void SetVietnameseEnabled(bool enabled);
void ToggleVietnamese();

// Switch the input method (0 = Telex, 1 = VNI): saves it and updates the engine, tray and UI.
void SetInputMethod(int method);

// Push the current Settings::enabled value to the engine, hook, tray and UI without saving.
// Used at startup and after per-app mode switches state on app change.
void SyncEnabledState();

} // namespace gonhanh
