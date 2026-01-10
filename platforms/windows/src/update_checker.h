#pragma once

#include <string>
#include <functional>
#include <cstdint>

namespace gonhanh {

// Update information from GitHub release
struct UpdateInfo {
    std::wstring version;
    std::wstring download_url;
    std::wstring release_notes;
    std::wstring published_at;
    uint64_t file_size = 0;
};

// Result of update check
enum class UpdateCheckResult {
    Available,
    UpToDate,
    Error
};

// Update checker - queries GitHub API
class UpdateChecker {
public:
    static UpdateChecker& instance();

    using CheckCallback = std::function<void(UpdateCheckResult result, UpdateInfo info, std::wstring error)>;

    // Check for updates asynchronously
    void check_for_updates(CheckCallback callback);

    // Compare versions using Rust core
    // Returns: -1 if v1 < v2, 0 if equal, 1 if v1 > v2
    int compare_versions(const std::string& v1, const std::string& v2);

    // Check if update available
    bool has_update(const std::string& current, const std::string& latest);

private:
    UpdateChecker() = default;
    ~UpdateChecker() = default;
    UpdateChecker(const UpdateChecker&) = delete;
    UpdateChecker& operator=(const UpdateChecker&) = delete;

    void parse_response(const std::string& json, CheckCallback callback);
    std::wstring extract_json_string(const std::string& json, const std::string& key);
    bool extract_json_bool(const std::string& json, const std::string& key);

    static constexpr const wchar_t* GITHUB_API_URL =
        L"https://api.github.com/repos/khaphanspace/gonhanh.org/releases";
};

} // namespace gonhanh
