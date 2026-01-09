#pragma once

#include "update_checker.h"
#include <string>
#include <functional>
#include <cstdint>
#include <chrono>

namespace gonhanh {

// Update state machine states
enum class UpdateState {
    Idle,
    Checking,
    Available,
    UpToDate,
    Downloading,
    Installing,
    Error
};

// Update manager - state machine for update lifecycle
class UpdateManager {
public:
    static UpdateManager& instance();

    using StateCallback = std::function<void(UpdateState state)>;
    using ProgressCallback = std::function<void(double progress)>;

    // State
    UpdateState state() const { return state_; }
    const UpdateInfo& update_info() const { return update_info_; }
    const std::wstring& error_message() const { return error_message_; }
    double download_progress() const { return download_progress_; }

    // Actions
    void check_for_updates_manual();
    void check_for_updates_silent();
    void download_update();
    void skip_version(const std::wstring& version);
    void cancel_download();
    void reset();

    // Callbacks
    void set_state_callback(StateCallback callback) { state_callback_ = callback; }
    void set_progress_callback(ProgressCallback callback) { progress_callback_ = callback; }

    // Auto-check
    bool should_auto_check() const;
    void mark_check_time();

    // Registry keys
    static constexpr const wchar_t* REG_LAST_CHECK = L"UpdateLastCheck";
    static constexpr const wchar_t* REG_SKIP_VERSION = L"UpdateSkipVersion";

private:
    UpdateManager();
    ~UpdateManager() = default;
    UpdateManager(const UpdateManager&) = delete;
    UpdateManager& operator=(const UpdateManager&) = delete;

    void set_state(UpdateState state);
    void on_check_complete(UpdateCheckResult result, UpdateInfo info, std::wstring error);
    void download_file(const std::wstring& url);
    void install_update(const std::wstring& file_path);

    UpdateState state_ = UpdateState::Idle;
    UpdateInfo update_info_;
    std::wstring error_message_;
    double download_progress_ = 0.0;

    StateCallback state_callback_;
    ProgressCallback progress_callback_;

    std::chrono::system_clock::time_point last_check_time_;
    bool silent_check_ = false;

    // Auto-check interval: 24 hours
    static constexpr int AUTO_CHECK_INTERVAL_HOURS = 24;
};

} // namespace gonhanh
