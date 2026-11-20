#pragma once
#include <windows.h>
#include <string>

namespace gonhanh {

// Sound utilities
void PlayToggleSound();

// Return pages the process isn't actively using to the OS (they fault back in on demand).
// Called when the app goes idle in the tray: after startup and after closing its windows.
void TrimWorkingSet();

// Logging utilities
void LogError(const std::wstring& message);
void LogInfo(const std::wstring& message);

} // namespace gonhanh
