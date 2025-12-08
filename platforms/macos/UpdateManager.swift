import Foundation
import AppKit

// MARK: - Update State

enum UpdateState {
    case idle
    case checking
    case available(UpdateInfo)
    case upToDate
    case downloading(progress: Double)
    case readyToInstall(dmgPath: URL)
    case error(String)
}

// MARK: - Update Manager

class UpdateManager: NSObject, ObservableObject {
    static let shared = UpdateManager()

    @Published var state: UpdateState = .idle
    @Published var lastCheckDate: Date?

    private var downloadTask: URLSessionDownloadTask?
    private var downloadedDMGPath: URL?

    private let autoCheckInterval: TimeInterval = 24 * 60 * 60  // 24 hours
    private let autoCheckKey = "gonhanh.update.lastCheck"
    private let skipVersionKey = "gonhanh.update.skipVersion"

    private override init() {
        super.init()
        lastCheckDate = UserDefaults.standard.object(forKey: autoCheckKey) as? Date
    }

    // MARK: - Public API

    /// Check for updates manually (UI will show in UpdateView)
    func checkForUpdatesManually() {
        checkForUpdates(silent: false)
    }

    /// Check for updates silently (background check)
    func checkForUpdatesSilently() {
        if let lastCheck = lastCheckDate,
           Date().timeIntervalSince(lastCheck) < autoCheckInterval {
            return
        }
        checkForUpdates(silent: true)
    }

    /// Download the update
    func downloadUpdate(_ info: UpdateInfo) {
        state = .downloading(progress: 0)

        let session = URLSession(configuration: .default, delegate: self, delegateQueue: .main)
        downloadTask = session.downloadTask(with: info.downloadURL)
        downloadTask?.resume()
    }

    /// Install the downloaded update
    func installUpdate() {
        guard case .readyToInstall(let dmgPath) = state else { return }

        // Open the DMG file
        NSWorkspace.shared.open(dmgPath)

        // Quit the app to allow replacement
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            NSApp.terminate(nil)
        }
    }

    /// Skip this version
    func skipVersion(_ version: String) {
        UserDefaults.standard.set(version, forKey: skipVersionKey)
        state = .idle
    }

    /// Cancel ongoing download
    func cancelDownload() {
        downloadTask?.cancel()
        downloadTask = nil
        state = .idle
    }

    // MARK: - Private Methods

    private func checkForUpdates(silent: Bool) {
        if !silent {
            state = .checking
        }

        UpdateChecker.shared.checkForUpdates { [weak self] result in
            guard let self = self else { return }

            // Save check date
            self.lastCheckDate = Date()
            UserDefaults.standard.set(self.lastCheckDate, forKey: self.autoCheckKey)

            switch result {
            case .available(let info):
                // Check if user skipped this version
                let skippedVersion = UserDefaults.standard.string(forKey: self.skipVersionKey)
                if silent && skippedVersion == info.version {
                    self.state = .idle
                    return
                }

                self.state = .available(info)

                // Only show notification for background check
                if silent {
                    self.showUpdateNotification(info)
                }

            case .upToDate:
                self.state = .upToDate

            case .error(let message):
                self.state = .error(message)
            }
        }
    }

    // MARK: - Notification (for background check only)

    private func showUpdateNotification(_ info: UpdateInfo) {
        let notification = NSUserNotification()
        notification.title = "GoNhanh - Có phiên bản mới"
        notification.informativeText = "Phiên bản \(info.version) đã sẵn sàng để tải về."
        notification.soundName = NSUserNotificationDefaultSoundName
        notification.hasActionButton = true
        notification.actionButtonTitle = "Xem"

        NSUserNotificationCenter.default.deliver(notification)
    }
}

// MARK: - URLSession Download Delegate

extension UpdateManager: URLSessionDownloadDelegate {
    func urlSession(_ session: URLSession, downloadTask: URLSessionDownloadTask, didFinishDownloadingTo location: URL) {
        let downloadsURL = FileManager.default.urls(for: .downloadsDirectory, in: .userDomainMask).first!
        let destinationURL = downloadsURL.appendingPathComponent("GoNhanh.dmg")

        do {
            try? FileManager.default.removeItem(at: destinationURL)
            try FileManager.default.moveItem(at: location, to: destinationURL)

            downloadedDMGPath = destinationURL
            state = .readyToInstall(dmgPath: destinationURL)

        } catch {
            state = .error("Không thể lưu file: \(error.localizedDescription)")
        }
    }

    func urlSession(_ session: URLSession, downloadTask: URLSessionDownloadTask, didWriteData bytesWritten: Int64, totalBytesWritten: Int64, totalBytesExpectedToWrite: Int64) {
        let progress = Double(totalBytesWritten) / Double(totalBytesExpectedToWrite)
        state = .downloading(progress: progress)
    }

    func urlSession(_ session: URLSession, task: URLSessionTask, didCompleteWithError error: Error?) {
        if let error = error {
            if (error as NSError).code == NSURLErrorCancelled {
                state = .idle
            } else {
                state = .error("Tải về thất bại: \(error.localizedDescription)")
            }
        }
    }
}
