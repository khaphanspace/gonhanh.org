#include "ime_state.h"
#include "keyboard_hook.h"
#include "per_app.h"
#include "rust_bridge.h"
#include "settings.h"
#include "settings_window.h"
#include "system_tray.h"
#include "utils.h"

namespace gonhanh {

void SyncEnabledState() {
    bool enabled = Settings::Instance().enabled;
    ime_enabled(enabled);
    ime_clear_all();  // Never carry a half-typed word across a mode switch
    KeyboardHook::Instance().SetEnabled(enabled);
    SystemTray::Instance().UpdateIcon();
    SettingsWindow::Instance().Refresh();
}

void SetVietnameseEnabled(bool enabled) {
    auto& settings = Settings::Instance();
    settings.enabled = enabled;
    settings.Save();
    if (settings.perApp) {
        PerAppMode::Instance().SetCurrentAppState(enabled);
    }
    SyncEnabledState();
    if (settings.sound) {
        PlayToggleSound();
    }
}

void ToggleVietnamese() {
    SetVietnameseEnabled(!Settings::Instance().enabled);
}

} // namespace gonhanh
