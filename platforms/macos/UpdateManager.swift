import Foundation
import AppKit
import SwiftUI

// MARK: - Update State

enum UpdateState {
    case idle
    case checking
    case available(UpdateInfo)
    case downloading(progress: Double)
    case readyToInstall(dmgPath: URL)
    case upToDate
    case error(String)
}

// MARK: - Update Manager

class UpdateManager: NSObject, ObservableObject {
    static let shared = UpdateManager()

    @Published var state: UpdateState = .idle
    @Published var lastCheckDate: Date?

    private var downloadTask: URLSessionDownloadTask?
    private var updateWindow: NSWindow?

    private let autoCheckInterval: TimeInterval = 24 * 60 * 60
    private let autoCheckKey = "gonhanh.update.lastCheck"
    private let skipVersionKey = "gonhanh.update.skipVersion"

    private override init() {
        super.init()
        lastCheckDate = UserDefaults.standard.object(forKey: autoCheckKey) as? Date
    }

    // MARK: - Computed Properties

    var isChecking: Bool {
        if case .checking = state { return true }
        return false
    }

    var updateAvailable: Bool {
        if case .available = state { return true }
        return false
    }

    // MARK: - Public API

    func checkForUpdatesManually() {
        checkForUpdates(silent: false)
    }

    func checkForUpdatesSilently() {
        if let lastCheck = lastCheckDate,
           Date().timeIntervalSince(lastCheck) < autoCheckInterval {
            return
        }
        checkForUpdates(silent: true)
    }

    func downloadUpdate(_ info: UpdateInfo) {
        state = .downloading(progress: 0)
        let session = URLSession(configuration: .default, delegate: self, delegateQueue: .main)
        downloadTask = session.downloadTask(with: info.downloadURL)
        downloadTask?.resume()
    }

    func installUpdate() {
        guard case .readyToInstall(let dmgPath) = state else { return }
        NSWorkspace.shared.open(dmgPath)
        // Brief delay for DMG to mount, then quit
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
            NSApp.terminate(nil)
        }
    }

    func skipVersion(_ version: String) {
        UserDefaults.standard.set(version, forKey: skipVersionKey)
        state = .idle
        dismissWindow()
    }

    func cancelDownload() {
        downloadTask?.cancel()
        downloadTask = nil
        state = .idle
    }

    func dismiss() {
        if case .available = state { state = .idle }
        if case .upToDate = state { state = .idle }
        if case .error = state { state = .idle }
        dismissWindow()
    }

    // MARK: - Window Management

    func showUpdateWindow() {
        if updateWindow == nil {
            let view = UpdatePopupView()
            let controller = NSHostingController(rootView: view)
            let window = NSPanel(
                contentRect: NSRect(x: 0, y: 0, width: 480, height: 10),
                styleMask: [.titled, .closable],
                backing: .buffered,
                defer: false
            )
            window.contentViewController = controller
            window.title = "Cập nhật phần mềm"
            window.standardWindowButton(.miniaturizeButton)?.isHidden = true
            window.standardWindowButton(.zoomButton)?.isHidden = true
            window.hasShadow = true
            window.level = .floating
            window.center()
            window.isReleasedWhenClosed = false
            updateWindow = window
        }
        NSApp.activate(ignoringOtherApps: true)
        updateWindow?.makeKeyAndOrderFront(nil)
    }

    private func dismissWindow() {
        updateWindow?.close()
    }

    // MARK: - Private Methods

    private func checkForUpdates(silent: Bool) {
        if !silent {
            state = .checking
            showUpdateWindow()
        }

        UpdateChecker.shared.checkForUpdates { [weak self] result in
            guard let self = self else { return }

            self.lastCheckDate = Date()
            UserDefaults.standard.set(self.lastCheckDate, forKey: self.autoCheckKey)

            switch result {
            case .available(let info):
                let skippedVersion = UserDefaults.standard.string(forKey: self.skipVersionKey)
                if silent && skippedVersion == info.version {
                    self.state = .idle
                    return
                }
                self.state = .available(info)
                if !silent {
                    // Window already shown from checking state
                } else {
                    // Silent check found update — just update badge, don't show popup
                }

            case .upToDate:
                self.state = silent ? .idle : .upToDate

            case .error(let message):
                self.state = .error(message)
                if silent { self.state = .idle }
            }
        }
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
            state = .readyToInstall(dmgPath: destinationURL)
        } catch {
            state = .error("Không thể lưu bản cập nhật")
        }
    }

    func urlSession(_ session: URLSession, downloadTask: URLSessionDownloadTask, didWriteData bytesWritten: Int64, totalBytesWritten: Int64, totalBytesExpectedToWrite: Int64) {
        state = .downloading(progress: Double(totalBytesWritten) / Double(totalBytesExpectedToWrite))
    }

    func urlSession(_ session: URLSession, task: URLSessionTask, didCompleteWithError error: Error?) {
        if let error = error {
            state = (error as NSError).code == NSURLErrorCancelled ? .idle : .error("Tải về thất bại")
        }
    }
}

// MARK: - Update Popup View

struct UpdatePopupView: View {
    @ObservedObject private var manager = UpdateManager.shared
    @Environment(\.colorScheme) private var colorScheme

    private var popupWidth: CGFloat {
        if case .available = manager.state { return 480 }
        return 360
    }

    var body: some View {
        VStack(spacing: 0) {
            switch manager.state {
            case .checking:
                checkingContent
            case .available(let info):
                availableContent(info)
            case .downloading(let progress):
                downloadingContent(progress)
            case .readyToInstall:
                readyContent
            case .upToDate:
                upToDateContent
            case .error(let message):
                errorContent(message)
            default:
                EmptyView()
            }
        }
        .frame(width: popupWidth)
        .background(VisualEffectBlur(material: .hudWindow, blendingMode: .behindWindow))
    }

    // MARK: - Checking

    private var checkingContent: some View {
        VStack(spacing: 16) {
            appIcon
            Text("Đang kiểm tra cập nhật...")
                .font(.system(size: 14))
                .foregroundColor(.secondary)
            ProgressView()
                .controlSize(.small)
        }
        .padding(32)
    }

    // MARK: - Update Available

    private func availableContent(_ info: UpdateInfo) -> some View {
        VStack(spacing: 0) {
            // Version info
            Text("\(AppMetadata.name) \(info.version) đã sẵn sàng — bạn đang dùng \(AppMetadata.version)")
                .font(.system(size: 11.5))
                .foregroundColor(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 20)
                .padding(.top, 14)
                .padding(.bottom, 12)

            // Release notes
            if !info.releaseNotes.isEmpty {
                releaseNotesPanel(info.releaseNotes)
                    .padding(.horizontal, 20)
            }

            // CTA
            HStack(spacing: 10) {
                Spacer()
                Button("Để sau") { manager.dismiss() }
                    .buttonStyle(.bordered)
                    .controlSize(.large)
                Button("Cập nhật") { manager.downloadUpdate(info) }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.large)
            }
            .padding(.horizontal, 20)
            .padding(.top, 16)
            .padding(.bottom, 18)
        }
    }

    // MARK: - Downloading

    private func downloadingContent(_ progress: Double) -> some View {
        VStack(spacing: 16) {
            appIcon
            Text("Đang tải cập nhật...")
                .font(.system(size: 14, weight: .medium))

            VStack(spacing: 6) {
                ProgressView(value: progress)
                    .progressViewStyle(.linear)
                Text("\(Int(progress * 100))%")
                    .font(.system(size: 11))
                    .foregroundColor(.secondary)
            }

            Button(action: { manager.cancelDownload(); manager.dismiss() }) {
                Text("Huỷ")
                    .font(.system(size: 12))
                    .foregroundColor(.secondary)
            }
            .buttonStyle(.plain)
        }
        .padding(28)
    }

    // MARK: - Ready to Install

    private var readyContent: some View {
        VStack(spacing: 16) {
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 40))
                .foregroundColor(.green)

            Text("Sẵn sàng cài đặt")
                .font(.system(size: 17, weight: .semibold))

            Text("Ứng dụng sẽ thoát để hoàn tất cài đặt.\nKéo GoNhanh vào thư mục Applications khi DMG mở.")
                .font(.system(size: 12))
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .lineSpacing(2)

            Button(action: { manager.installUpdate() }) {
                Text("Cài đặt và thoát")
                    .font(.system(size: 13, weight: .medium))
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 8)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)

            Button(action: { manager.dismiss() }) {
                Text("Để sau")
                    .font(.system(size: 12))
                    .foregroundColor(.secondary)
            }
            .buttonStyle(.plain)
        }
        .padding(28)
    }

    // MARK: - Up to Date

    private var upToDateContent: some View {
        VStack(spacing: 16) {
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 40))
                .foregroundColor(.green)

            Text("Bạn đang dùng phiên bản mới nhất")
                .font(.system(size: 14, weight: .medium))

            Text("v\(AppMetadata.version)")
                .font(.system(size: 12))
                .foregroundColor(.secondary)

            Button(action: { manager.dismiss() }) {
                Text("OK")
                    .font(.system(size: 13, weight: .medium))
                    .frame(width: 80)
                    .padding(.vertical, 6)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
        }
        .padding(28)
    }

    // MARK: - Error

    private func errorContent(_ message: String) -> some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 40))
                .foregroundColor(.orange)

            Text("Không thể kiểm tra cập nhật")
                .font(.system(size: 14, weight: .medium))

            Text(message)
                .font(.system(size: 12))
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)

            Button(action: { manager.dismiss() }) {
                Text("OK")
                    .font(.system(size: 13, weight: .medium))
                    .frame(width: 80)
                    .padding(.vertical, 6)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
        }
        .padding(28)
    }

    // MARK: - Components

    private var appIcon: some View {
        Image(nsImage: AppMetadata.logo)
            .resizable()
            .frame(width: 64, height: 64)
    }

    // MARK: - Release Notes

    private enum NoteItem {
        case heading(String)
        case bullet(String)
    }

    private func releaseNotesPanel(_ notes: String) -> some View {
        let items = parseReleaseNotes(notes)
        return ScrollView {
            VStack(alignment: .leading, spacing: 3) {
                ForEach(Array(items.enumerated()), id: \.offset) { _, item in
                    noteItemView(item)
                }
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .frame(height: 170)
        .background(Color.primary.opacity(0.04))
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }

    @ViewBuilder
    private func noteItemView(_ item: NoteItem) -> some View {
        switch item {
        case .heading(let text):
            Text(text)
                .font(.system(size: 11, weight: .semibold))
                .foregroundColor(.primary.opacity(0.7))
                .padding(.top, 6)
                .padding(.bottom, 1)
        case .bullet(let text):
            HStack(alignment: .firstTextBaseline, spacing: 6) {
                Text("•")
                    .font(.system(size: 9))
                    .foregroundColor(.primary.opacity(0.35))
                Text(text)
                    .font(.system(size: 11))
                    .foregroundColor(.primary.opacity(0.85))
                    .lineSpacing(1.5)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }

    private func parseReleaseNotes(_ text: String) -> [NoteItem] {
        var items: [NoteItem] = []
        for line in text.components(separatedBy: .newlines) {
            var l = line.trimmingCharacters(in: .whitespaces)
            if l.isEmpty { continue }
            if l.allSatisfy({ $0 == "-" || $0 == "=" || $0 == "*" }) { continue }
            if l.lowercased().contains("what's changed") { continue }
            if l.lowercased().contains("full changelog") { continue }

            if l.hasPrefix("#") {
                while l.hasPrefix("#") { l = String(l.dropFirst()) }
                l = cleanNote(l)
                if !l.isEmpty { items.append(.heading(l)) }
                continue
            }

            if l.hasPrefix("- ") || l.hasPrefix("* ") {
                l = String(l.dropFirst(2))
                l = cleanNote(l)
                if !l.isEmpty { items.append(.bullet(l)) }
                continue
            }

            l = cleanNote(l)
            if !l.isEmpty { items.append(.bullet(l)) }
        }
        return items
    }

    private func cleanNote(_ text: String) -> String {
        var l = text.trimmingCharacters(in: .whitespaces)
        l = l.replacingOccurrences(of: "**", with: "")
        l = l.replacingOccurrences(of: "__", with: "")
        l = l.replacingOccurrences(of: "`", with: "")
        return l.trimmingCharacters(in: .whitespaces)
    }
}

// MARK: - Visual Effect Blur

struct VisualEffectBlur: NSViewRepresentable {
    let material: NSVisualEffectView.Material
    let blendingMode: NSVisualEffectView.BlendingMode

    func makeNSView(context: Context) -> NSVisualEffectView {
        let view = NSVisualEffectView()
        view.material = material
        view.blendingMode = blendingMode
        view.state = .active
        return view
    }

    func updateNSView(_ nsView: NSVisualEffectView, context: Context) {
        nsView.material = material
        nsView.blendingMode = blendingMode
    }
}
