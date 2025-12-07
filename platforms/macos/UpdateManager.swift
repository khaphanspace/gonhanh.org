import Foundation
import AppKit

// MARK: - Update State

enum UpdateState {
    case idle
    case checking
    case available(UpdateInfo)
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

    /// Check for updates manually (shows UI feedback)
    func checkForUpdatesManually() {
        checkForUpdates(silent: false)
    }

    /// Check for updates silently (background check)
    func checkForUpdatesSilently() {
        // Only check if enough time has passed
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

        // Show instructions
        let alert = NSAlert()
        alert.messageText = "Cài đặt cập nhật"
        alert.informativeText = """
        1. Kéo GoNhanh vào thư mục Applications
        2. Thay thế phiên bản cũ khi được hỏi
        3. Khởi động lại GoNhanh

        Lưu ý: GoNhanh sẽ tự động thoát sau khi bạn nhấn OK.
        """
        alert.alertStyle = .informational
        alert.addButton(withTitle: "OK, Thoát để cài đặt")
        alert.addButton(withTitle: "Để sau")

        if alert.runModal() == .alertFirstButtonReturn {
            // Quit the app to allow replacement
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

                if !silent {
                    self.showUpdateAvailableAlert(info)
                } else {
                    // Show notification for background check
                    self.showUpdateNotification(info)
                }

            case .upToDate:
                self.state = .idle
                if !silent {
                    self.showUpToDateAlert()
                }

            case .error(let message):
                self.state = .error(message)
                if !silent {
                    self.showErrorAlert(message)
                }
            }
        }
    }

    // MARK: - UI Alerts

    private func showUpdateAvailableAlert(_ info: UpdateInfo) {
        let alert = NSAlert()
        alert.messageText = "Có phiên bản mới!"
        alert.informativeText = """
        Phiên bản \(info.version) đã sẵn sàng.
        Phiên bản hiện tại: \(AppMetadata.version)

        \(formatReleaseNotes(info.releaseNotes))
        """
        alert.alertStyle = .informational
        alert.addButton(withTitle: "Tải về")
        alert.addButton(withTitle: "Để sau")
        alert.addButton(withTitle: "Bỏ qua phiên bản này")

        NSApp.activate(ignoringOtherApps: true)

        let response = alert.runModal()
        switch response {
        case .alertFirstButtonReturn:
            downloadUpdate(info)
        case .alertThirdButtonReturn:
            skipVersion(info.version)
        default:
            state = .idle
        }
    }

    private func showUpToDateAlert() {
        let alert = NSAlert()
        alert.messageText = "Bạn đang dùng phiên bản mới nhất"
        alert.informativeText = "GoNhanh \(AppMetadata.version) là phiên bản mới nhất."
        alert.alertStyle = .informational
        alert.addButton(withTitle: "OK")

        NSApp.activate(ignoringOtherApps: true)
        alert.runModal()
    }

    private func showErrorAlert(_ message: String) {
        let alert = NSAlert()
        alert.messageText = "Không thể kiểm tra cập nhật"
        alert.informativeText = message
        alert.alertStyle = .warning
        alert.addButton(withTitle: "OK")

        NSApp.activate(ignoringOtherApps: true)
        alert.runModal()
    }

    private func showDownloadCompleteAlert() {
        let alert = NSAlert()
        alert.messageText = "Tải về hoàn tất"
        alert.informativeText = "Bản cập nhật đã sẵn sàng để cài đặt."
        alert.alertStyle = .informational
        alert.addButton(withTitle: "Cài đặt ngay")
        alert.addButton(withTitle: "Để sau")

        NSApp.activate(ignoringOtherApps: true)

        if alert.runModal() == .alertFirstButtonReturn {
            installUpdate()
        }
    }

    private func showUpdateNotification(_ info: UpdateInfo) {
        let notification = NSUserNotification()
        notification.title = "GoNhanh - Có phiên bản mới"
        notification.informativeText = "Phiên bản \(info.version) đã sẵn sàng để tải về."
        notification.soundName = NSUserNotificationDefaultSoundName
        notification.hasActionButton = true
        notification.actionButtonTitle = "Xem"

        NSUserNotificationCenter.default.deliver(notification)
    }

    private func formatReleaseNotes(_ notes: String) -> String {
        // Truncate long release notes
        let maxLength = 300
        if notes.count > maxLength {
            return String(notes.prefix(maxLength)) + "..."
        }
        return notes
    }
}

// MARK: - URLSession Download Delegate

extension UpdateManager: URLSessionDownloadDelegate {
    func urlSession(_ session: URLSession, downloadTask: URLSessionDownloadTask, didFinishDownloadingTo location: URL) {
        // Move to Downloads folder
        let downloadsURL = FileManager.default.urls(for: .downloadsDirectory, in: .userDomainMask).first!
        let destinationURL = downloadsURL.appendingPathComponent("GoNhanh.dmg")

        do {
            // Remove existing file if any
            try? FileManager.default.removeItem(at: destinationURL)
            try FileManager.default.moveItem(at: location, to: destinationURL)

            downloadedDMGPath = destinationURL
            state = .readyToInstall(dmgPath: destinationURL)

            showDownloadCompleteAlert()

        } catch {
            state = .error("Failed to save update: \(error.localizedDescription)")
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
                state = .error("Download failed: \(error.localizedDescription)")
            }
        }
    }
}
