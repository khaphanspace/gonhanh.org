#include "update_checker.h"
#include "rust_bridge.h"
#include "logger.h"
#include <Windows.h>
#include <winhttp.h>
#include <thread>
#include <sstream>

#pragma comment(lib, "winhttp.lib")

namespace gonhanh {

UpdateChecker& UpdateChecker::instance() {
    static UpdateChecker instance;
    return instance;
}

void UpdateChecker::check_for_updates(CheckCallback callback) {
    // Run in background thread
    std::thread([this, callback]() {
        Logger::info(L"Checking for updates...");

        HINTERNET hSession = nullptr;
        HINTERNET hConnect = nullptr;
        HINTERNET hRequest = nullptr;
        std::string response_body;

        hSession = WinHttpOpen(
            L"GoNhanh/1.0",
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            WINHTTP_NO_PROXY_NAME,
            WINHTTP_NO_PROXY_BYPASS,
            0
        );

        if (!hSession) {
            callback(UpdateCheckResult::Error, {}, L"Không thể khởi tạo kết nối");
            return;
        }

        // Parse URL
        URL_COMPONENTS urlComp = {};
        urlComp.dwStructSize = sizeof(urlComp);
        wchar_t hostName[256] = {};
        wchar_t urlPath[1024] = {};
        urlComp.lpszHostName = hostName;
        urlComp.dwHostNameLength = 256;
        urlComp.lpszUrlPath = urlPath;
        urlComp.dwUrlPathLength = 1024;

        if (!WinHttpCrackUrl(GITHUB_API_URL, 0, 0, &urlComp)) {
            WinHttpCloseHandle(hSession);
            callback(UpdateCheckResult::Error, {}, L"URL không hợp lệ");
            return;
        }

        hConnect = WinHttpConnect(hSession, hostName, urlComp.nPort, 0);
        if (!hConnect) {
            WinHttpCloseHandle(hSession);
            callback(UpdateCheckResult::Error, {}, L"Không thể kết nối server");
            return;
        }

        hRequest = WinHttpOpenRequest(
            hConnect,
            L"GET",
            urlPath,
            nullptr,
            WINHTTP_NO_REFERER,
            WINHTTP_DEFAULT_ACCEPT_TYPES,
            WINHTTP_FLAG_SECURE
        );

        if (!hRequest) {
            WinHttpCloseHandle(hConnect);
            WinHttpCloseHandle(hSession);
            callback(UpdateCheckResult::Error, {}, L"Không thể tạo request");
            return;
        }

        // Set headers
        WinHttpAddRequestHeaders(
            hRequest,
            L"Accept: application/vnd.github.v3+json",
            -1,
            WINHTTP_ADDREQ_FLAG_ADD
        );

        // Send request
        if (!WinHttpSendRequest(hRequest, WINHTTP_NO_ADDITIONAL_HEADERS, 0,
                               WINHTTP_NO_REQUEST_DATA, 0, 0, 0)) {
            WinHttpCloseHandle(hRequest);
            WinHttpCloseHandle(hConnect);
            WinHttpCloseHandle(hSession);
            callback(UpdateCheckResult::Error, {}, L"Không thể gửi request");
            return;
        }

        // Receive response
        if (!WinHttpReceiveResponse(hRequest, nullptr)) {
            WinHttpCloseHandle(hRequest);
            WinHttpCloseHandle(hConnect);
            WinHttpCloseHandle(hSession);
            callback(UpdateCheckResult::Error, {}, L"Không nhận được phản hồi");
            return;
        }

        // Check status code
        DWORD statusCode = 0;
        DWORD statusCodeSize = sizeof(statusCode);
        WinHttpQueryHeaders(
            hRequest,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            WINHTTP_HEADER_NAME_BY_INDEX,
            &statusCode,
            &statusCodeSize,
            WINHTTP_NO_HEADER_INDEX
        );

        if (statusCode != 200) {
            WinHttpCloseHandle(hRequest);
            WinHttpCloseHandle(hConnect);
            WinHttpCloseHandle(hSession);
            std::wstring error = L"Server trả về mã lỗi: " + std::to_wstring(statusCode);
            callback(UpdateCheckResult::Error, {}, error);
            return;
        }

        // Read response body
        DWORD bytesAvailable = 0;
        do {
            bytesAvailable = 0;
            if (!WinHttpQueryDataAvailable(hRequest, &bytesAvailable)) break;
            if (bytesAvailable == 0) break;

            std::vector<char> buffer(bytesAvailable + 1, 0);
            DWORD bytesRead = 0;
            if (WinHttpReadData(hRequest, buffer.data(), bytesAvailable, &bytesRead)) {
                response_body.append(buffer.data(), bytesRead);
            }
        } while (bytesAvailable > 0);

        WinHttpCloseHandle(hRequest);
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);

        if (response_body.empty()) {
            callback(UpdateCheckResult::Error, {}, L"Không có dữ liệu phản hồi");
            return;
        }

        parse_response(response_body, callback);
    }).detach();
}

void UpdateChecker::parse_response(const std::string& json, CheckCallback callback) {
    // Simple JSON parsing (releases array)
    // Find highest version non-draft, non-prerelease release

    std::string current_version = "1.0.0";  // TODO: Get from app metadata

    std::string best_version;
    std::string best_tag;
    std::string best_download_url;
    std::string best_notes;
    std::string best_published;
    uint64_t best_size = 0;

    // Parse releases array - look for tag_name, draft, prerelease
    size_t pos = 0;
    while ((pos = json.find("\"tag_name\"", pos)) != std::string::npos) {
        // Find the release object boundaries
        size_t obj_start = json.rfind('{', pos);
        size_t obj_end = json.find('}', pos);
        if (obj_start == std::string::npos || obj_end == std::string::npos) {
            pos++;
            continue;
        }

        // Expand obj_end to find the complete release object (handling nested objects)
        int brace_count = 1;
        size_t search_pos = obj_start + 1;
        while (brace_count > 0 && search_pos < json.length()) {
            if (json[search_pos] == '{') brace_count++;
            else if (json[search_pos] == '}') brace_count--;
            search_pos++;
        }
        obj_end = search_pos;

        std::string release_obj = json.substr(obj_start, obj_end - obj_start);

        // Skip drafts and prereleases
        if (extract_json_bool(release_obj, "draft") ||
            extract_json_bool(release_obj, "prerelease")) {
            pos = obj_end;
            continue;
        }

        // Extract tag_name
        std::wstring tag_w = extract_json_string(release_obj, "tag_name");
        std::string tag(tag_w.begin(), tag_w.end());
        if (tag.empty()) {
            pos = obj_end;
            continue;
        }

        // Remove 'v' prefix if present
        std::string version = tag;
        if (!version.empty() && version[0] == 'v') {
            version = version.substr(1);
        }

        // Compare with best
        if (best_version.empty() || compare_versions(version, best_version) > 0) {
            best_version = version;
            best_tag = tag;

            // Extract release notes
            std::wstring notes_w = extract_json_string(release_obj, "body");
            best_notes = std::string(notes_w.begin(), notes_w.end());

            // Extract published_at
            std::wstring pub_w = extract_json_string(release_obj, "published_at");
            best_published = std::string(pub_w.begin(), pub_w.end());

            // Find Windows ZIP asset
            size_t assets_pos = release_obj.find("\"assets\"");
            if (assets_pos != std::string::npos) {
                size_t assets_start = release_obj.find('[', assets_pos);
                size_t assets_end = release_obj.find(']', assets_start);
                if (assets_start != std::string::npos && assets_end != std::string::npos) {
                    std::string assets = release_obj.substr(assets_start, assets_end - assets_start + 1);

                    // Find Windows-x64.zip or .exe asset
                    size_t asset_pos = 0;
                    while ((asset_pos = assets.find("\"name\"", asset_pos)) != std::string::npos) {
                        std::wstring name = extract_json_string(assets.substr(asset_pos), "name");
                        std::string name_s(name.begin(), name.end());

                        if (name_s.find("Windows") != std::string::npos ||
                            name_s.find("windows") != std::string::npos ||
                            name_s.find("win") != std::string::npos) {

                            // Find browser_download_url for this asset
                            size_t url_pos = assets.find("browser_download_url", asset_pos);
                            if (url_pos != std::string::npos && url_pos < assets.find('}', asset_pos)) {
                                std::wstring url = extract_json_string(assets.substr(url_pos - 50), "browser_download_url");
                                best_download_url = std::string(url.begin(), url.end());

                                // Extract size
                                size_t size_pos = assets.find("\"size\"", asset_pos);
                                if (size_pos != std::string::npos && size_pos < assets.find('}', asset_pos)) {
                                    size_t colon = assets.find(':', size_pos);
                                    if (colon != std::string::npos) {
                                        best_size = std::stoull(assets.substr(colon + 1, 20));
                                    }
                                }
                                break;
                            }
                        }
                        asset_pos++;
                    }
                }
            }
        }

        pos = obj_end;
    }

    if (best_version.empty()) {
        callback(UpdateCheckResult::UpToDate, {}, L"");
        return;
    }

    // Check if update available
    if (!has_update(current_version, best_version)) {
        callback(UpdateCheckResult::UpToDate, {}, L"");
        return;
    }

    if (best_download_url.empty()) {
        callback(UpdateCheckResult::Error, {}, L"Không tìm thấy file cài đặt cho Windows");
        return;
    }

    // Build UpdateInfo
    UpdateInfo info;
    info.version = std::wstring(best_version.begin(), best_version.end());
    info.download_url = std::wstring(best_download_url.begin(), best_download_url.end());
    info.release_notes = std::wstring(best_notes.begin(), best_notes.end());
    info.published_at = std::wstring(best_published.begin(), best_published.end());
    info.file_size = best_size;

    Logger::info(L"Update available: " + info.version);
    callback(UpdateCheckResult::Available, info, L"");
}

std::wstring UpdateChecker::extract_json_string(const std::string& json, const std::string& key) {
    std::string search = "\"" + key + "\"";
    size_t pos = json.find(search);
    if (pos == std::string::npos) return L"";

    size_t colon = json.find(':', pos);
    if (colon == std::string::npos) return L"";

    size_t quote1 = json.find('"', colon);
    if (quote1 == std::string::npos) return L"";

    size_t quote2 = json.find('"', quote1 + 1);
    // Handle escaped quotes
    while (quote2 != std::string::npos && quote2 > 0 && json[quote2 - 1] == '\\') {
        quote2 = json.find('"', quote2 + 1);
    }
    if (quote2 == std::string::npos) return L"";

    std::string value = json.substr(quote1 + 1, quote2 - quote1 - 1);

    // Convert UTF-8 to wide string
    int size = MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1, nullptr, 0);
    if (size <= 0) return L"";

    std::wstring result(size - 1, L'\0');
    MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1, &result[0], size);
    return result;
}

bool UpdateChecker::extract_json_bool(const std::string& json, const std::string& key) {
    std::string search = "\"" + key + "\"";
    size_t pos = json.find(search);
    if (pos == std::string::npos) return false;

    size_t colon = json.find(':', pos);
    if (colon == std::string::npos) return false;

    // Skip whitespace
    size_t val_start = colon + 1;
    while (val_start < json.length() && std::isspace(json[val_start])) val_start++;

    return json.substr(val_start, 4) == "true";
}

int UpdateChecker::compare_versions(const std::string& v1, const std::string& v2) {
    auto& bridge = RustBridge::instance();
    if (bridge.is_loaded()) {
        return bridge.version_compare(v1.c_str(), v2.c_str());
    }

    // Fallback: simple comparison
    auto parse_version = [](const std::string& v) -> std::tuple<int, int, int> {
        int major = 0, minor = 0, patch = 0;
        sscanf_s(v.c_str(), "%d.%d.%d", &major, &minor, &patch);
        return {major, minor, patch};
    };

    auto [m1, n1, p1] = parse_version(v1);
    auto [m2, n2, p2] = parse_version(v2);

    if (m1 != m2) return m1 < m2 ? -1 : 1;
    if (n1 != n2) return n1 < n2 ? -1 : 1;
    if (p1 != p2) return p1 < p2 ? -1 : 1;
    return 0;
}

bool UpdateChecker::has_update(const std::string& current, const std::string& latest) {
    return compare_versions(current, latest) < 0;
}

} // namespace gonhanh
