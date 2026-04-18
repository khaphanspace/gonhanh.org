#pragma once

#include <string>
#include <atomic>
#include <mutex>

namespace gonhanh {

// Version from CMakeLists (set at build time via resource.h)
const wchar_t* GetAppVersion();

enum class UpdateStatus {
    Idle,
    Checking,
    UpToDate,
    Available,
    Error
};

class UpdateChecker {
public:
    static UpdateChecker& Instance();

    // Start async version check (runs in background thread)
    void CheckAsync();

    // Get current status
    UpdateStatus GetStatus() const;

    // Get latest version from GitHub (only valid when status == Available)
    std::wstring GetLatestVersion() const;

    // Get status text for display
    std::wstring GetStatusText() const;

private:
    UpdateChecker() = default;
    ~UpdateChecker();
    UpdateChecker(const UpdateChecker&) = delete;
    UpdateChecker& operator=(const UpdateChecker&) = delete;

    void QueryGitHubReleases();
    bool CompareVersions(const std::wstring& current, const std::wstring& latest);

    std::atomic<UpdateStatus> status_{UpdateStatus::Idle};
    std::wstring latestVersion_;
    mutable std::mutex mutex_;
    std::thread workerThread_;
};

} // namespace gonhanh
