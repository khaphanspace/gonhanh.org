#pragma once
#include <windows.h>
#include <string>
#include <cstdint>

namespace gonhanh {

// Injection method types (matches macOS InjectionMethod)
enum class InjectionMethod : uint8_t {
    Fast = 0,       // Default: minimal delays (200/800/500 µs)
    Slow = 1,       // For terminals, Electron apps (8000/25000/8000 µs)
    Selection = 2   // For browser address bars (Shift+Left select + type)
};

// Timing configuration for injection
struct InjectionTiming {
    uint32_t backspaceDelayUs = 200;   // Delay after each backspace
    uint32_t waitDelayUs = 800;        // Delay before sending text
    uint32_t textDelayUs = 500;        // Delay between text characters
};

// Detection result with method and timing
struct DetectionResult {
    InjectionMethod method = InjectionMethod::Fast;
    InjectionTiming timing;
    DWORD timestamp = 0;  // GetTickCount() for TTL

    DetectionResult() = default;
    DetectionResult(InjectionMethod m, uint32_t bs, uint32_t wait, uint32_t text)
        : method(m), timing{bs, wait, text}, timestamp(GetTickCount()) {}
};

class AppCompat {
public:
    static AppCompat& Instance();

    // Get current foreground app name
    std::wstring GetForegroundAppName();

    // Check if app has known compatibility issues
    bool IsProblematicApp(const std::wstring& appName);

    // Check if current context needs selection method (address bars)
    bool NeedsSelectionMethod();

    // Get injection method for current app with 200ms TTL cache
    DetectionResult GetInjectionMethod();

    // Clear detection cache (called on app switch)
    void ClearDetectionCache();

private:
    AppCompat() = default;
    ~AppCompat() = default;
    AppCompat(const AppCompat&) = delete;
    AppCompat& operator=(const AppCompat&) = delete;

    std::wstring cachedAppName_;
    DWORD cachedPid_ = 0;

    // Detection cache with TTL
    DetectionResult cachedDetection_;
    bool hasDetectionCache_ = false;
    static constexpr DWORD DETECTION_TTL_MS = 200;  // 200ms cache TTL

    // Detect injection method for app name
    DetectionResult DetectInjectionMethod(const std::wstring& appName);
};

} // namespace gonhanh
