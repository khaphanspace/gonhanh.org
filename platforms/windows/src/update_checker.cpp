#include "update_checker.h"
#include "resource.h"
#include <windows.h>
#include <winhttp.h>
#include <thread>
#include <sstream>
#include <vector>
#include <tuple>

#pragma comment(lib, "winhttp.lib")

namespace gonhanh {

// Convert narrow string to wide string
static std::wstring ToWide(const std::string& str) {
    if (str.empty()) return L"";
    int size = MultiByteToWideChar(CP_UTF8, 0, str.c_str(), -1, nullptr, 0);
    std::wstring result(size - 1, 0);
    MultiByteToWideChar(CP_UTF8, 0, str.c_str(), -1, &result[0], size);
    return result;
}

// Get app version from resource.h
const wchar_t* GetAppVersion() {
    static std::wstring version = ToWide(APP_VERSION_STRING);
    return version.c_str();
}

UpdateChecker& UpdateChecker::Instance() {
    static UpdateChecker instance;
    return instance;
}

UpdateChecker::~UpdateChecker() {
    // Wait for background thread to finish before destroying
    if (workerThread_.joinable()) {
        workerThread_.join();
    }
}

void UpdateChecker::CheckAsync() {
    // Already checking
    if (status_.load() == UpdateStatus::Checking) {
        return;
    }

    // Join previous thread if still lingering
    if (workerThread_.joinable()) {
        workerThread_.join();
    }

    status_.store(UpdateStatus::Checking);

    // Run in background thread (joinable, cleaned up in destructor)
    workerThread_ = std::thread([this]() {
        QueryGitHubReleases();
    });
}

UpdateStatus UpdateChecker::GetStatus() const {
    return status_.load();
}

std::wstring UpdateChecker::GetLatestVersion() const {
    std::lock_guard<std::mutex> lock(mutex_);
    return latestVersion_;
}

std::wstring UpdateChecker::GetStatusText() const {
    switch (status_.load()) {
        case UpdateStatus::Idle:
            return L"";
        case UpdateStatus::Checking:
            return L"...";
        case UpdateStatus::UpToDate:
            return L"\u2713 M\x1EDBi nh\x1EA5t";  // "✓ Mới nhất"
        case UpdateStatus::Available: {
            std::lock_guard<std::mutex> lock(mutex_);
            return L"C\x00F3 b\x1EA3n c\x1EADp nh\x1EADt";  // "Có bản cập nhật"
        }
        case UpdateStatus::Error:
            return L"";
        default:
            return L"";
    }
}

void UpdateChecker::QueryGitHubReleases() {
    HINTERNET hSession = nullptr, hConnect = nullptr, hRequest = nullptr;
    std::string response;

    // Initialize WinHTTP
    hSession = WinHttpOpen(L"GoNhanh/1.0",
        WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
        WINHTTP_NO_PROXY_NAME,
        WINHTTP_NO_PROXY_BYPASS, 0);

    if (!hSession) {
        status_.store(UpdateStatus::Error);
        return;
    }

    // Connect to GitHub API
    hConnect = WinHttpConnect(hSession, L"api.github.com",
        INTERNET_DEFAULT_HTTPS_PORT, 0);

    if (!hConnect) {
        WinHttpCloseHandle(hSession);
        status_.store(UpdateStatus::Error);
        return;
    }

    // Create request
    hRequest = WinHttpOpenRequest(hConnect, L"GET",
        L"/repos/khaphanspace/gonhanh.org/releases/latest",
        nullptr, WINHTTP_NO_REFERER,
        WINHTTP_DEFAULT_ACCEPT_TYPES,
        WINHTTP_FLAG_SECURE);

    if (!hRequest) {
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        status_.store(UpdateStatus::Error);
        return;
    }

    // Add headers
    const wchar_t* headers = L"Accept: application/vnd.github+json\r\n"
                              L"User-Agent: GoNhanh-Windows";
    WinHttpAddRequestHeaders(hRequest, headers, -1, WINHTTP_ADDREQ_FLAG_ADD);

    // Send request
    if (!WinHttpSendRequest(hRequest, WINHTTP_NO_ADDITIONAL_HEADERS, 0,
                            WINHTTP_NO_REQUEST_DATA, 0, 0, 0)) {
        WinHttpCloseHandle(hRequest);
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        status_.store(UpdateStatus::Error);
        return;
    }

    // Wait for response
    if (!WinHttpReceiveResponse(hRequest, nullptr)) {
        WinHttpCloseHandle(hRequest);
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        status_.store(UpdateStatus::Error);
        return;
    }

    // Read response (cap at 1MB to prevent abuse)
    const size_t MAX_RESPONSE = 1024 * 1024;
    DWORD bytesAvailable = 0;
    do {
        bytesAvailable = 0;
        WinHttpQueryDataAvailable(hRequest, &bytesAvailable);

        if (bytesAvailable > 0) {
            std::vector<char> buffer(bytesAvailable + 1);
            DWORD bytesRead = 0;
            if (WinHttpReadData(hRequest, buffer.data(), bytesAvailable, &bytesRead)) {
                buffer[bytesRead] = 0;
                response += buffer.data();
            }
        }
        if (response.size() > MAX_RESPONSE) break;
    } while (bytesAvailable > 0);

    // Cleanup
    WinHttpCloseHandle(hRequest);
    WinHttpCloseHandle(hConnect);
    WinHttpCloseHandle(hSession);

    // Parse "tag_name" from JSON response
    // Simple parsing - look for "tag_name": "v1.0.xxx"
    std::string tagKey = "\"tag_name\"";
    size_t pos = response.find(tagKey);
    if (pos == std::string::npos) {
        status_.store(UpdateStatus::Error);
        return;
    }

    // Find the value after ":"
    pos = response.find(':', pos);
    if (pos == std::string::npos) {
        status_.store(UpdateStatus::Error);
        return;
    }

    // Find opening quote
    pos = response.find('"', pos);
    if (pos == std::string::npos) {
        status_.store(UpdateStatus::Error);
        return;
    }

    // Find closing quote
    size_t endPos = response.find('"', pos + 1);
    if (endPos == std::string::npos) {
        status_.store(UpdateStatus::Error);
        return;
    }

    std::string tagName = response.substr(pos + 1, endPos - pos - 1);

    // Remove 'v' prefix if present
    if (!tagName.empty() && tagName[0] == 'v') {
        tagName = tagName.substr(1);
    }

    std::wstring latestVer = ToWide(tagName);
    std::wstring currentVer = GetAppVersion();

    {
        std::lock_guard<std::mutex> lock(mutex_);
        latestVersion_ = latestVer;
    }

    // Compare versions
    if (CompareVersions(currentVer, latestVer)) {
        status_.store(UpdateStatus::UpToDate);
    } else {
        status_.store(UpdateStatus::Available);
    }
}

// Returns true if current >= latest (up to date)
bool UpdateChecker::CompareVersions(const std::wstring& current, const std::wstring& latest) {
    auto parseVersion = [](const std::wstring& ver) -> std::tuple<int, int, int> {
        int major = 0, minor = 0, patch = 0;
        std::wistringstream ss(ver);
        wchar_t dot;
        ss >> major >> dot >> minor >> dot >> patch;
        return {major, minor, patch};
    };

    auto [curMajor, curMinor, curPatch] = parseVersion(current);
    auto [latMajor, latMinor, latPatch] = parseVersion(latest);

    if (curMajor != latMajor) return curMajor >= latMajor;
    if (curMinor != latMinor) return curMinor >= latMinor;
    return curPatch >= latPatch;
}

} // namespace gonhanh
