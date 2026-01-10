#include "update_manager.h"
#include "update_checker.h"
#include "settings.h"
#include "logger.h"
#include <Windows.h>
#include <winhttp.h>
#include <thread>
#include <fstream>
#include <filesystem>
#include <shellapi.h>

#pragma comment(lib, "winhttp.lib")

namespace gonhanh {

UpdateManager& UpdateManager::instance() {
    static UpdateManager instance;
    return instance;
}

UpdateManager::UpdateManager() {
    // Load last check time from registry
    auto& settings = Settings::instance();
    uint64_t last_check = 0;
    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, Settings::REG_PATH, 0, KEY_READ, &key) == ERROR_SUCCESS) {
        DWORD data_size = sizeof(last_check);
        RegQueryValueExW(key, REG_LAST_CHECK, nullptr, nullptr,
                        reinterpret_cast<BYTE*>(&last_check), &data_size);
        RegCloseKey(key);
    }

    if (last_check > 0) {
        last_check_time_ = std::chrono::system_clock::from_time_t(static_cast<time_t>(last_check));
    }
}

void UpdateManager::set_state(UpdateState state) {
    state_ = state;
    if (state_callback_) {
        state_callback_(state);
    }
}

void UpdateManager::check_for_updates_manual() {
    if (state_ == UpdateState::Checking || state_ == UpdateState::Downloading) {
        return;
    }

    silent_check_ = false;
    set_state(UpdateState::Checking);

    UpdateChecker::instance().check_for_updates(
        [this](UpdateCheckResult result, UpdateInfo info, std::wstring error) {
            on_check_complete(result, info, error);
        }
    );
}

void UpdateManager::check_for_updates_silent() {
    if (!should_auto_check()) {
        return;
    }

    if (state_ == UpdateState::Checking || state_ == UpdateState::Downloading) {
        return;
    }

    silent_check_ = true;
    // Don't change state for silent check

    UpdateChecker::instance().check_for_updates(
        [this](UpdateCheckResult result, UpdateInfo info, std::wstring error) {
            on_check_complete(result, info, error);
        }
    );
}

void UpdateManager::on_check_complete(UpdateCheckResult result, UpdateInfo info, std::wstring error) {
    mark_check_time();

    switch (result) {
        case UpdateCheckResult::Available: {
            // Check if this version was skipped
            HKEY key;
            if (RegOpenKeyExW(HKEY_CURRENT_USER, Settings::REG_PATH, 0, KEY_READ, &key) == ERROR_SUCCESS) {
                wchar_t skip_version[64] = {};
                DWORD size = sizeof(skip_version);
                if (RegQueryValueExW(key, REG_SKIP_VERSION, nullptr, nullptr,
                                    reinterpret_cast<BYTE*>(skip_version), &size) == ERROR_SUCCESS) {
                    if (info.version == skip_version) {
                        if (silent_check_) {
                            set_state(UpdateState::Idle);
                        } else {
                            // Manual check - still show the update
                            update_info_ = info;
                            set_state(UpdateState::Available);
                        }
                        RegCloseKey(key);
                        return;
                    }
                }
                RegCloseKey(key);
            }

            update_info_ = info;
            set_state(UpdateState::Available);
            break;
        }
        case UpdateCheckResult::UpToDate:
            if (!silent_check_) {
                set_state(UpdateState::UpToDate);
            } else {
                set_state(UpdateState::Idle);
            }
            break;
        case UpdateCheckResult::Error:
            error_message_ = error;
            if (!silent_check_) {
                set_state(UpdateState::Error);
            } else {
                set_state(UpdateState::Idle);
            }
            break;
    }
}

void UpdateManager::download_update() {
    if (state_ != UpdateState::Available || update_info_.download_url.empty()) {
        return;
    }

    download_progress_ = 0.0;
    set_state(UpdateState::Downloading);
    download_file(update_info_.download_url);
}

void UpdateManager::download_file(const std::wstring& url) {
    std::thread([this, url]() {
        Logger::info("Downloading update: %ls", url.c_str());

        HINTERNET hSession = nullptr;
        HINTERNET hConnect = nullptr;
        HINTERNET hRequest = nullptr;

        hSession = WinHttpOpen(
            L"GoNhanh/1.0",
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            WINHTTP_NO_PROXY_NAME,
            WINHTTP_NO_PROXY_BYPASS,
            0
        );

        if (!hSession) {
            error_message_ = L"Không thể khởi tạo kết nối";
            set_state(UpdateState::Error);
            return;
        }

        // Parse URL
        URL_COMPONENTS urlComp = {};
        urlComp.dwStructSize = sizeof(urlComp);
        wchar_t hostName[256] = {};
        wchar_t urlPath[2048] = {};
        urlComp.lpszHostName = hostName;
        urlComp.dwHostNameLength = 256;
        urlComp.lpszUrlPath = urlPath;
        urlComp.dwUrlPathLength = 2048;

        if (!WinHttpCrackUrl(url.c_str(), 0, 0, &urlComp)) {
            WinHttpCloseHandle(hSession);
            error_message_ = L"URL không hợp lệ";
            set_state(UpdateState::Error);
            return;
        }

        hConnect = WinHttpConnect(hSession, hostName, urlComp.nPort, 0);
        if (!hConnect) {
            WinHttpCloseHandle(hSession);
            error_message_ = L"Không thể kết nối server";
            set_state(UpdateState::Error);
            return;
        }

        DWORD flags = (urlComp.nScheme == INTERNET_SCHEME_HTTPS) ? WINHTTP_FLAG_SECURE : 0;
        hRequest = WinHttpOpenRequest(
            hConnect,
            L"GET",
            urlPath,
            nullptr,
            WINHTTP_NO_REFERER,
            WINHTTP_DEFAULT_ACCEPT_TYPES,
            flags
        );

        if (!hRequest) {
            WinHttpCloseHandle(hConnect);
            WinHttpCloseHandle(hSession);
            error_message_ = L"Không thể tạo request";
            set_state(UpdateState::Error);
            return;
        }

        // Follow redirects
        DWORD option = WINHTTP_OPTION_REDIRECT_POLICY_ALWAYS;
        WinHttpSetOption(hRequest, WINHTTP_OPTION_REDIRECT_POLICY, &option, sizeof(option));

        if (!WinHttpSendRequest(hRequest, WINHTTP_NO_ADDITIONAL_HEADERS, 0,
                               WINHTTP_NO_REQUEST_DATA, 0, 0, 0)) {
            WinHttpCloseHandle(hRequest);
            WinHttpCloseHandle(hConnect);
            WinHttpCloseHandle(hSession);
            error_message_ = L"Không thể gửi request";
            set_state(UpdateState::Error);
            return;
        }

        if (!WinHttpReceiveResponse(hRequest, nullptr)) {
            WinHttpCloseHandle(hRequest);
            WinHttpCloseHandle(hConnect);
            WinHttpCloseHandle(hSession);
            error_message_ = L"Không nhận được phản hồi";
            set_state(UpdateState::Error);
            return;
        }

        // Get content length
        DWORD content_length = 0;
        DWORD size = sizeof(content_length);
        WinHttpQueryHeaders(
            hRequest,
            WINHTTP_QUERY_CONTENT_LENGTH | WINHTTP_QUERY_FLAG_NUMBER,
            WINHTTP_HEADER_NAME_BY_INDEX,
            &content_length,
            &size,
            WINHTTP_NO_HEADER_INDEX
        );

        // Create temp file
        wchar_t temp_path[MAX_PATH];
        GetTempPathW(MAX_PATH, temp_path);
        std::wstring file_path = std::wstring(temp_path) + L"GoNhanh-" + update_info_.version + L".zip";

        std::ofstream file(file_path, std::ios::binary);
        if (!file.is_open()) {
            WinHttpCloseHandle(hRequest);
            WinHttpCloseHandle(hConnect);
            WinHttpCloseHandle(hSession);
            error_message_ = L"Không thể tạo file tạm";
            set_state(UpdateState::Error);
            return;
        }

        // Download
        DWORD total_read = 0;
        DWORD bytes_available = 0;

        do {
            bytes_available = 0;
            if (!WinHttpQueryDataAvailable(hRequest, &bytes_available)) break;
            if (bytes_available == 0) break;

            std::vector<char> buffer(bytes_available);
            DWORD bytes_read = 0;
            if (WinHttpReadData(hRequest, buffer.data(), bytes_available, &bytes_read)) {
                file.write(buffer.data(), bytes_read);
                total_read += bytes_read;

                if (content_length > 0) {
                    download_progress_ = static_cast<double>(total_read) / content_length;
                    if (progress_callback_) {
                        progress_callback_(download_progress_);
                    }
                }
            }
        } while (bytes_available > 0 && state_ == UpdateState::Downloading);

        file.close();
        WinHttpCloseHandle(hRequest);
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);

        if (state_ != UpdateState::Downloading) {
            // Cancelled
            std::filesystem::remove(file_path);
            set_state(UpdateState::Idle);
            return;
        }

        Logger::info("Download complete: %ls", file_path.c_str());
        install_update(file_path);
    }).detach();
}

void UpdateManager::install_update(const std::wstring& file_path) {
    set_state(UpdateState::Installing);
    Logger::info("Installing update from: %ls", file_path.c_str());

    // For ZIP: Extract and run installer/updater
    // For EXE: Run directly

    if (file_path.ends_with(L".exe")) {
        // Run installer
        SHELLEXECUTEINFOW sei = {};
        sei.cbSize = sizeof(sei);
        sei.fMask = SEE_MASK_NOCLOSEPROCESS;
        sei.lpVerb = L"open";
        sei.lpFile = file_path.c_str();
        sei.nShow = SW_SHOWNORMAL;

        if (ShellExecuteExW(&sei)) {
            // Exit current app
            PostQuitMessage(0);
        } else {
            error_message_ = L"Không thể chạy file cài đặt";
            set_state(UpdateState::Error);
        }
    } else {
        // For ZIP: Need to extract and replace
        // This is more complex - for now, just open the file location
        wchar_t* file_part = nullptr;
        wchar_t full_path[MAX_PATH];
        GetFullPathNameW(file_path.c_str(), MAX_PATH, full_path, &file_part);

        // Open explorer to the file
        std::wstring explorer_cmd = L"/select,\"" + std::wstring(full_path) + L"\"";
        ShellExecuteW(nullptr, L"open", L"explorer.exe", explorer_cmd.c_str(), nullptr, SW_SHOWNORMAL);

        MessageBoxW(nullptr,
            L"File cập nhật đã được tải về. Vui lòng giải nén và chạy file cài đặt.",
            L"Cập nhật GoNhanh",
            MB_OK | MB_ICONINFORMATION);

        set_state(UpdateState::Idle);
    }
}

void UpdateManager::skip_version(const std::wstring& version) {
    HKEY key;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, Settings::REG_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &key, nullptr) == ERROR_SUCCESS) {
        RegSetValueExW(key, REG_SKIP_VERSION, 0, REG_SZ,
                      reinterpret_cast<const BYTE*>(version.c_str()),
                      static_cast<DWORD>((version.length() + 1) * sizeof(wchar_t)));
        RegCloseKey(key);
    }
    set_state(UpdateState::Idle);
}

void UpdateManager::cancel_download() {
    if (state_ == UpdateState::Downloading) {
        set_state(UpdateState::Idle);
    }
}

void UpdateManager::reset() {
    set_state(UpdateState::Idle);
    error_message_.clear();
    download_progress_ = 0.0;
}

bool UpdateManager::should_auto_check() const {
    auto now = std::chrono::system_clock::now();
    auto diff = std::chrono::duration_cast<std::chrono::hours>(now - last_check_time_);
    return diff.count() >= AUTO_CHECK_INTERVAL_HOURS;
}

void UpdateManager::mark_check_time() {
    last_check_time_ = std::chrono::system_clock::now();

    // Save to registry
    uint64_t timestamp = static_cast<uint64_t>(
        std::chrono::system_clock::to_time_t(last_check_time_));

    HKEY key;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, Settings::REG_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &key, nullptr) == ERROR_SUCCESS) {
        RegSetValueExW(key, REG_LAST_CHECK, 0, REG_QWORD,
                      reinterpret_cast<const BYTE*>(&timestamp), sizeof(timestamp));
        RegCloseKey(key);
    }
}

} // namespace gonhanh
