#pragma once

#include <string>

namespace gonhanh {

// The `gonhanh` / `gn` command line tool ships in a bin\ folder next to GoNhanh.exe.
// These helpers add that folder to (or remove it from) the user's PATH (HKCU\Environment),
// so the Settings window can offer it as a toggle instead of asking users to edit PATH.

// bin\ folder next to the running exe, or empty if the CLI isn't there
std::wstring CliBinDir();

enum class CliPathState {
    Missing,    // no bin\ with the CLI next to this GoNhanh.exe
    Off,        // no Gõ Nhanh CLI folder in the user's PATH
    On,         // this copy's bin\ is in the user's PATH
    OtherCopy,  // only another copy's bin\ is (moved folder, older install, dev build)
};
CliPathState GetCliPathState();

// On: put this copy's bin\ in the user's PATH, replacing any other copy's. Off: remove every
// Gõ Nhanh CLI folder. Notifies running programs so new terminals pick it up.
// Returns false if the CLI is missing or the registry write failed.
bool SetCliOnPath(bool on);

} // namespace gonhanh
