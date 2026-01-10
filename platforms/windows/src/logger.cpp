#include "logger.h"
#include <Windows.h>
#include <ShlObj.h>
#include <ctime>
#include <filesystem>

namespace gonhanh {

std::ofstream Logger::file_;
std::mutex Logger::mutex_;
bool Logger::initialized_ = false;
std::wstring Logger::log_dir_;

std::wstring Logger::get_default_log_dir() {
    wchar_t appdata[MAX_PATH];
    if (SUCCEEDED(SHGetFolderPathW(nullptr, CSIDL_APPDATA, nullptr, 0, appdata))) {
        return std::wstring(appdata) + L"\\GoNhanh\\logs";
    }
    return L".\\logs";
}

void Logger::init(const std::wstring& log_dir) {
    std::lock_guard<std::mutex> lock(mutex_);
    if (initialized_) return;

    log_dir_ = log_dir;

    // Create log directory
    std::error_code ec;
    std::filesystem::create_directories(log_dir_, ec);
    if (ec) {
        // Fallback to current directory
        log_dir_ = L".";
    }

    // Generate log filename with date
    time_t now = time(nullptr);
    tm local;
    localtime_s(&local, &now);

    wchar_t filename[64];
    swprintf_s(filename, 64, L"gonhanh-%04d%02d%02d.log",
        local.tm_year + 1900, local.tm_mon + 1, local.tm_mday);

    std::wstring path = log_dir_ + L"\\" + filename;
    file_.open(path, std::ios::app);

    if (file_.is_open()) {
        initialized_ = true;
        // Write init message directly to avoid deadlock (we already hold the lock)
        char timestamp[32];
        strftime(timestamp, sizeof(timestamp), "%Y-%m-%d %H:%M:%S", &local);
        file_ << timestamp << " [INFO ] === GoNhanh Logger initialized ===\n";
        file_.flush();
    }
}

void Logger::shutdown() {
    std::lock_guard<std::mutex> lock(mutex_);
    if (file_.is_open()) {
        // Write shutdown message before closing
        if (initialized_) {
            time_t now = time(nullptr);
            tm local;
            localtime_s(&local, &now);
            char timestamp[32];
            strftime(timestamp, sizeof(timestamp), "%Y-%m-%d %H:%M:%S", &local);
            file_ << timestamp << " [INFO ] === GoNhanh Logger shutdown ===\n";
        }
        file_.close();
    }
    initialized_ = false;
}

void Logger::log(LogLevel level, const char* format, va_list args) {
    if (!initialized_) return;

    std::lock_guard<std::mutex> lock(mutex_);
    if (!file_.is_open()) return;

    // Timestamp
    time_t now = time(nullptr);
    tm local;
    localtime_s(&local, &now);
    char timestamp[32];
    strftime(timestamp, sizeof(timestamp), "%Y-%m-%d %H:%M:%S", &local);

    // Level string
    const char* level_str = "";
    switch (level) {
        case LogLevel::LOG_DEBUG: level_str = "DEBUG"; break;
        case LogLevel::LOG_INFO:  level_str = "INFO "; break;
        case LogLevel::LOG_WARN:  level_str = "WARN "; break;
        case LogLevel::LOG_ERROR: level_str = "ERROR"; break;
    }

    // Format message
    char message[1024];
    vsnprintf(message, sizeof(message), format, args);

    // Write to file
    file_ << timestamp << " [" << level_str << "] " << message << "\n";
    file_.flush();
}

void Logger::debug(const char* format, ...) {
#ifdef _DEBUG
    va_list args;
    va_start(args, format);
    log(LogLevel::LOG_DEBUG, format, args);
    va_end(args);
#else
    (void)format;
#endif
}

void Logger::info(const char* format, ...) {
    va_list args;
    va_start(args, format);
    log(LogLevel::LOG_INFO, format, args);
    va_end(args);
}

void Logger::warn(const char* format, ...) {
    va_list args;
    va_start(args, format);
    log(LogLevel::LOG_WARN, format, args);
    va_end(args);
}

void Logger::error(const char* format, ...) {
    va_list args;
    va_start(args, format);
    log(LogLevel::LOG_ERROR, format, args);
    va_end(args);
}

void Logger::log_key(uint16_t vk, bool caps, bool ctrl, bool handled) {
    debug("Key: vk=0x%04X caps=%d ctrl=%d handled=%d", vk, caps, ctrl, handled);
}

void Logger::log_ffi_call(const char* func, bool success) {
    if (success) {
        debug("FFI: %s() -> OK", func);
    } else {
        error("FFI: %s() -> FAILED", func);
    }
}

void Logger::log_ffi_result(const char* func, const char* result) {
    debug("FFI: %s() -> %s", func, result);
}

void Logger::rotate_if_needed() {
    // Optional: Implement log rotation if file size > 10MB
    // For now, daily rotation via filename is sufficient
}

} // namespace gonhanh
