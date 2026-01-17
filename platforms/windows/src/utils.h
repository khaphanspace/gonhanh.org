#pragma once
#include <windows.h>
#include <string>

namespace gonhanh {

// Sound utilities
void PlayToggleSound();

// Logging utilities
void LogError(const std::wstring& message);
void LogInfo(const std::wstring& message);

} // namespace gonhanh
