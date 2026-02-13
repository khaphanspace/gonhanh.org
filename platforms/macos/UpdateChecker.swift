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

    // Use /releases instead of /releases/latest to get highest version, not most recent publish
    private let githubAPIURL = "https://api.github.com/repos/khaphanspace/gonhanh.org/releases"

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
            guard let releases = try JSONSerialization.jsonObject(with: data) as? [[String: Any]] else {
                DispatchQueue.main.async { completion(.error("Invalid JSON format")) }
                return
            }

            let currentVersion = AppMetadata.version

            // Find the highest version release (not draft/prerelease)
            var bestRelease: [String: Any]?
            var bestVersion = ""

            for release in releases {
                guard let tagName = release["tag_name"] as? String,
                      release["draft"] as? Bool != true,
                      release["prerelease"] as? Bool != true else { continue }

                let version = tagName.hasPrefix("v") ? String(tagName.dropFirst()) : tagName

                if bestVersion.isEmpty {
                    bestVersion = version
                    bestRelease = release
                } else {
                    let cmp = bestVersion.withCString { bestPtr in
                        version.withCString { verPtr in
                            version_compare(bestPtr, verPtr)
                        }
                    }
                    if cmp < 0 {
                        bestVersion = version
                        bestRelease = release
                    }
                }
            }

            guard let release = bestRelease, !bestVersion.isEmpty else {
                DispatchQueue.main.async { completion(.upToDate) }
                return
            }

            // Check if update available
            let hasUpdate = currentVersion.withCString { currentPtr in
                bestVersion.withCString { latestPtr in
                    version_has_update(currentPtr, latestPtr)
                }
            }

            guard hasUpdate == 1 else {
                DispatchQueue.main.async { completion(.upToDate) }
                return
            }

            // Find DMG download URL
            var downloadURL: URL?
            if let assets = release["assets"] as? [[String: Any]] {
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

            guard let finalDownloadURL = downloadURL else {
                DispatchQueue.main.async { completion(.error("No DMG found in release")) }
                return
            }

            let releaseNotes = release["body"] as? String ?? ""
            var publishedAt: Date?
            if let publishedString = release["published_at"] as? String {
                publishedAt = ISO8601DateFormatter().date(from: publishedString)
            }

            let updateInfo = UpdateInfo(
                version: bestVersion,
                downloadURL: finalDownloadURL,
                releaseNotes: releaseNotes,
                publishedAt: publishedAt
            )

            DispatchQueue.main.async { completion(.available(updateInfo)) }

        } catch {
            DispatchQueue.main.async {
                completion(.error("JSON parse error: \(error.localizedDescription)"))
            }
        }
    }
}
