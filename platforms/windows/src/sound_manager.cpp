#include "sound_manager.h"
#include "settings.h"

#pragma comment(lib, "winmm.lib")

namespace gonhanh {

SoundManager& SoundManager::instance() {
    static SoundManager instance;
    return instance;
}

void SoundManager::play_toggle_sound(bool enabled) {
    // Check if sound is enabled in settings
    if (!Settings::instance().sound_enabled()) {
        return;
    }

    // Use Windows system sounds similar to macOS Tink/Pop
    // ASTERISK for enable (positive feedback)
    // EXCLAMATION for disable (neutral feedback)
    // SND_ASYNC plays in background without blocking
    // SND_NODEFAULT prevents fallback to default beep if sound not found
    if (enabled) {
        // Higher-pitched "ding" for enable - similar to macOS "Tink"
        PlaySoundW(reinterpret_cast<LPCWSTR>(SND_ALIAS_SYSTEMASTERISK),
                   nullptr, SND_ALIAS_ID | SND_ASYNC | SND_NODEFAULT);
    } else {
        // Lower "exclamation" for disable - similar to macOS "Pop"
        PlaySoundW(reinterpret_cast<LPCWSTR>(SND_ALIAS_SYSTEMEXCLAMATION),
                   nullptr, SND_ALIAS_ID | SND_ASYNC | SND_NODEFAULT);
    }
}

} // namespace gonhanh
