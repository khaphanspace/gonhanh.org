#pragma once
#include <windows.h>
#include <string>

namespace gonhanh {

// Debug console helper for troubleshooting
class DebugConsole {
public:
    static DebugConsole& Instance();

    // Create console window for debug output
    void Create();

    // Log message to console and debug output
    void Log(const std::wstring& message);

    // Check if console is available
    bool IsAvailable() const { return consoleAvailable_; }

private:
    DebugConsole() = default;
    ~DebugConsole();
    DebugConsole(const DebugConsole&) = delete;
    DebugConsole& operator=(const DebugConsole&) = delete;

    bool consoleAvailable_ = false;
};

} // namespace gonhanh
