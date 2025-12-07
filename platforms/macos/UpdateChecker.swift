import Foundation

// MARK: - FFI for Version Comparison

@_silgen_name("version_compare")
func version_compare(_ v1: UnsafePointer<CChar>?, _ v2: UnsafePointer<CChar>?) -> Int32

@_silgen_name("version_has_update")
func version_has_update(_ current: UnsafePointer<CChar>?, _ latest: UnsafePointer<CChar>?) -> Int32

// MARK: - Update Info

struct UpdateInfo {
    let version: String
    let downloadURL: URL
    let releaseNotes: String
    let publishedAt: Date?
}

// MARK: - Update Check Result

enum UpdateCheckResult {
    case available(UpdateInfo)
    case upToDate
    case error(String)
}

// MARK: - Update Checker

class UpdateChecker {
    static let shared = UpdateChecker()

    private let githubAPIURL = "https://api.github.com/repos/khaphanspace/gonhanh.org/releases/latest"
    private let downloadBaseURL = "https://github.com/khaphanspace/gonhanh.org/releases/latest/download"

    private init() {}

    /// Check for updates asynchronously
    func checkForUpdates(completion: @escaping (UpdateCheckResult) -> Void) {
        guard let url = URL(string: githubAPIURL) else {
            completion(.error("Invalid API URL"))
            return
        }

        var request = URLRequest(url: url)
        request.setValue("application/vnd.github.v3+json", forHTTPHeaderField: "Accept")
        request.timeoutInterval = 10

        let task = URLSession.shared.dataTask(with: request) { [weak self] data, response, error in
            if let error = error {
                DispatchQueue.main.async {
                    completion(.error("Network error: \(error.localizedDescription)"))
                }
                return
            }

            guard let httpResponse = response as? HTTPURLResponse else {
                DispatchQueue.main.async {
                    completion(.error("Invalid response"))
                }
                return
            }

            guard httpResponse.statusCode == 200 else {
                DispatchQueue.main.async {
                    completion(.error("Server error: \(httpResponse.statusCode)"))
                }
                return
            }

            guard let data = data else {
                DispatchQueue.main.async {
                    completion(.error("No data received"))
                }
                return
            }

            self?.parseResponse(data: data, completion: completion)
        }

        task.resume()
    }

    private func parseResponse(data: Data, completion: @escaping (UpdateCheckResult) -> Void) {
        do {
            guard let json = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
                DispatchQueue.main.async {
                    completion(.error("Invalid JSON format"))
                }
                return
            }

            // Extract version from tag_name (e.g., "v1.0.10" -> "1.0.10")
            guard let tagName = json["tag_name"] as? String else {
                DispatchQueue.main.async {
                    completion(.error("Missing version tag"))
                }
                return
            }

            let latestVersion = tagName.hasPrefix("v") ? String(tagName.dropFirst()) : tagName
            let currentVersion = AppMetadata.version

            // Use Rust core for version comparison
            let hasUpdate = currentVersion.withCString { currentPtr in
                latestVersion.withCString { latestPtr in
                    version_has_update(currentPtr, latestPtr)
                }
            }

            if hasUpdate == 1 {
                // Parse additional info
                let releaseNotes = json["body"] as? String ?? ""
                let htmlURL = json["html_url"] as? String ?? ""

                // Find DMG download URL from assets
                var downloadURL: URL?
                if let assets = json["assets"] as? [[String: Any]] {
                    for asset in assets {
                        if let name = asset["name"] as? String,
                           name.lowercased().hasSuffix(".dmg"),
                           let urlString = asset["browser_download_url"] as? String,
                           let url = URL(string: urlString) {
                            downloadURL = url
                            break
                        }
                    }
                }

                // Fallback to default download URL
                let finalDownloadURL = downloadURL ?? URL(string: "\(downloadBaseURL)/GoNhanh.dmg")!

                // Parse published date
                var publishedAt: Date?
                if let publishedString = json["published_at"] as? String {
                    let formatter = ISO8601DateFormatter()
                    publishedAt = formatter.date(from: publishedString)
                }

                let updateInfo = UpdateInfo(
                    version: latestVersion,
                    downloadURL: finalDownloadURL,
                    releaseNotes: releaseNotes,
                    publishedAt: publishedAt
                )

                DispatchQueue.main.async {
                    completion(.available(updateInfo))
                }
            } else {
                DispatchQueue.main.async {
                    completion(.upToDate)
                }
            }

        } catch {
            DispatchQueue.main.async {
                completion(.error("JSON parse error: \(error.localizedDescription)"))
            }
        }
    }

    /// Compare two version strings using Rust core
    /// Returns: -1 if v1 < v2, 0 if equal, 1 if v1 > v2
    func compareVersions(_ v1: String, _ v2: String) -> Int {
        return v1.withCString { v1Ptr in
            v2.withCString { v2Ptr in
                Int(version_compare(v1Ptr, v2Ptr))
            }
        }
    }
}
