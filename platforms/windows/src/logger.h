#pragma once

#include <string>
#include <fstream>
#include <mutex>
#include <cstdarg>
#include <cstdint>

namespace gonhanh {

// Note: Avoid using ERROR as it conflicts with Windows wingdi.h macro
enum class LogLevel { LOG_DEBUG, LOG_INFO, LOG_WARN, LOG_ERROR };

class Logger {
public:
    static void init(const std::wstring& log_dir);
    static void shutdown();
    static bool is_initialized() { return initialized_; }

    static void debug(const char* format, ...);
    static void info(const char* format, ...);
    static void warn(const char* format, ...);
    static void error(const char* format, ...);

    // Specialized loggers
    static void log_key(uint16_t vk, bool caps, bool ctrl, bool handled);
    static void log_ffi_call(const char* func, bool success);
    static void log_ffi_result(const char* func, const char* result);

    // Get default log directory
    static std::wstring get_default_log_dir();

private:
    static void log(LogLevel level, const char* format, va_list args);
    static void rotate_if_needed();

    static std::ofstream file_;
    static std::mutex mutex_;
    static bool initialized_;
    static std::wstring log_dir_;
};

} // namespace gonhanh
