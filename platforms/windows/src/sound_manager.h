#pragma once

#include <windows.h>
#include <mmsystem.h>

namespace gonhanh {

// Sound manager for playing toggle feedback sounds
// Uses Windows system sounds to match macOS behavior
class SoundManager {
public:
    static SoundManager& instance();

    // Play sound when IME is toggled
    // enabled: true = IME enabled (plays "Tink"-like sound)
    // enabled: false = IME disabled (plays "Pop"-like sound)
    void play_toggle_sound(bool enabled);

private:
    SoundManager() = default;
    ~SoundManager() = default;
    SoundManager(const SoundManager&) = delete;
    SoundManager& operator=(const SoundManager&) = delete;
};

} // namespace gonhanh
