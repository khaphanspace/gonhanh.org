import AppKit
import Foundation
import SwiftUI

extension Notification.Name {
    static let updateStateChanged = Notification.Name("gonhanh.updateStateChanged")
}

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

    @Published var state: UpdateState = .idle {
        didSet { NotificationCenter.default.post(name: .updateStateChanged, object: nil) }
    }

    private var downloadTask: URLSessionDownloadTask?
    private var updateWindow: NSWindow?
    private var backgroundTimer: Timer?

    private let checkInterval: TimeInterval = 60 * 60 // 1 hour
    private let dmgKey = "gonhanh.update.dmgPath"
    private let versionKey = "gonhanh.update.pendingVersion"

    override private init() {
        super.init()
        // Restore pending update from previous session
        if let path = UserDefaults.standard.string(forKey: dmgKey),
           FileManager.default.fileExists(atPath: path)
        {
            let version = UserDefaults.standard.string(forKey: versionKey) ?? "?"
            state = .readyToInstall(dmgPath: URL(fileURLWithPath: path))
            pendingVersion = version
        }
    }

    /// Version string of the pending update
    var pendingVersion: String = ""

    // MARK: - Computed Properties

    var isChecking: Bool {
        if case .checking = state { return true }
        return false
    }

    var updateAvailable: Bool {
        if case .available = state { return true }
        if case .readyToInstall = state { return true }
        return false
    }

    var isReadyToInstall: Bool {
        if case .readyToInstall = state { return true }
        return false
    }

    // MARK: - Public API

    /// Start background auto-update loop (call once at app launch)
    func startBackgroundUpdates() {
        // Check immediately on launch
        checkAndDownloadSilently()
        // Then every hour
        backgroundTimer = Timer.scheduledTimer(withTimeInterval: checkInterval, repeats: true) { [weak self] _ in
            self?.checkAndDownloadSilently()
        }
    }

    /// User manually checks (only shows window if update available)
    func checkForUpdatesManually() {
        if case let .readyToInstall(dmgPath) = state {
            // Verify DMG still exists before restart
            if FileManager.default.fileExists(atPath: dmgPath.path) {
                restartToUpdate()
            } else {
                // DMG deleted — re-check
                state = .idle
            }
            return
        }
        // Prevent concurrent checks
        if case .checking = state { return }
        state = .checking
        UpdateChecker.shared.checkForUpdates { [weak self] result in
            guard let self else { return }
            switch result {
            case let .available(info):
                state = .available(info)
                showUpdateWindow()
            case .upToDate:
                state = .upToDate
            case let .error(message):
                state = .error(message)
            }
        }
    }

    /// Download update (from popup CTA)
    func downloadUpdate(_ info: UpdateInfo) {
        state = .downloading(progress: 0)
        let session = URLSession(configuration: .default, delegate: self, delegateQueue: .main)
        downloadTask = session.downloadTask(with: info.downloadURL)
        downloadTask?.resume()
    }

    /// Mount DMG, copy .app to /Applications, relaunch
    func restartToUpdate() {
        guard case let .readyToInstall(dmgPath) = state else { return }

        let appName = "GoNhanh"
        let appBundlePath = Bundle.main.bundlePath
        let pid = ProcessInfo.processInfo.processIdentifier

        // Shell script: mount DMG silently → copy app → unmount → relaunch
        let script = """
        #!/bin/bash
        DMG="\(dmgPath.path)"
        MOUNT=$(hdiutil attach "$DMG" -nobrowse -noverify -noautoopen 2>/dev/null | grep '/Volumes/' | awk -F'\t' '{print $NF}')
        if [ -z "$MOUNT" ]; then exit 1; fi
        APP="$MOUNT/\(appName).app"
        if [ ! -d "$APP" ]; then hdiutil detach "$MOUNT" -quiet; exit 1; fi
        # Wait for app to quit
        while kill -0 \(pid) 2>/dev/null; do sleep 0.2; done
        # Copy new app
        DEST="\(appBundlePath)"
        rm -rf "$DEST"
        cp -R "$APP" "$DEST"
        # Unmount & cleanup
        hdiutil detach "$MOUNT" -quiet
        rm -f "$DMG"
        # Relaunch
        open "$DEST"
        """

        let tmpScript = FileManager.default.temporaryDirectory.appendingPathComponent("gonhanh-update.sh")
        try? script.write(to: tmpScript, atomically: true, encoding: .utf8)
        try? FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: tmpScript.path)

        // Clear pending update state
        UserDefaults.standard.removeObject(forKey: dmgKey)
        UserDefaults.standard.removeObject(forKey: versionKey)

        // Launch script in background and quit
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/bin/bash")
        process.arguments = [tmpScript.path]
        try? process.run()

        NSApp.terminate(nil)
    }

    func dismiss() {
        switch state {
        case .available, .upToDate, .error: state = .idle
        default: break
        }
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

    // MARK: - Background Silent Check + Download

    private func checkAndDownloadSilently() {
        // Skip if currently downloading or user is checking
        if case .downloading = state { return }
        if case .checking = state { return }

        UpdateChecker.shared.checkForUpdates { [weak self] result in
            guard let self else { return }
            switch result {
            case let .available(info):
                // Skip if same version already downloaded
                if info.version == pendingVersion, case .readyToInstall = state { return }
                // Download new version (or newer version replacing old pending)
                pendingVersion = info.version
                downloadSilently(info)
            case .upToDate, .error:
                break
            }
        }
    }

    private func downloadSilently(_ info: UpdateInfo) {
        let session = URLSession(configuration: .default, delegate: self, delegateQueue: .main)
        downloadTask = session.downloadTask(with: info.downloadURL)
        downloadTask?.resume()
        // Don't update state to .downloading — keep it silent
    }

    fileprivate func finishDownload(to destinationURL: URL) {
        UserDefaults.standard.set(destinationURL.path, forKey: dmgKey)
        UserDefaults.standard.set(pendingVersion, forKey: versionKey)
        state = .readyToInstall(dmgPath: destinationURL)
    }
}

// MARK: - URLSession Download Delegate

extension UpdateManager: URLSessionDownloadDelegate {
    func urlSession(_: URLSession, downloadTask _: URLSessionDownloadTask, didFinishDownloadingTo location: URL) {
        let cachesURL = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first!
        let destinationURL = cachesURL.appendingPathComponent("GoNhanh-update.dmg")

        do {
            try? FileManager.default.removeItem(at: destinationURL)
            try FileManager.default.moveItem(at: location, to: destinationURL)
            finishDownload(to: destinationURL)
        } catch {
            if case .downloading = state {
                state = .error("Không thể lưu bản cập nhật")
            }
        }
    }

    func urlSession(_: URLSession, downloadTask _: URLSessionDownloadTask, didWriteData _: Int64, totalBytesWritten: Int64, totalBytesExpectedToWrite: Int64) {
        // Only show progress if user triggered download manually
        if case .downloading = state {
            state = .downloading(progress: Double(totalBytesWritten) / Double(totalBytesExpectedToWrite))
        }
    }

    func urlSession(_: URLSession, task _: URLSessionTask, didCompleteWithError error: Error?) {
        if let error, (error as NSError).code != NSURLErrorCancelled {
            if case .downloading = state {
                state = .error("Tải về thất bại")
            }
        }
    }
}

// MARK: - Update Popup View

struct UpdatePopupView: View {
    @ObservedObject private var manager = UpdateManager.shared

    private var popupWidth: CGFloat {
        if case .available = manager.state { return 480 }
        return 360
    }

    var body: some View {
        VStack(spacing: 0) {
            switch manager.state {
            case .checking:
                checkingContent
            case let .available(info):
                availableContent(info)
            case let .downloading(progress):
                downloadingContent(progress)
            case .readyToInstall:
                readyContent
            case .upToDate:
                upToDateContent
            case let .error(message):
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

            Button(action: { manager.dismiss() }) {
                Text("Ẩn")
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
            Image(systemName: "arrow.uturn.forward.circle.fill")
                .font(.system(size: 36))
                .foregroundColor(.orange)

            Text("Bản cập nhật đã sẵn sàng")
                .font(.system(size: 15, weight: .semibold))

            Text("Phiên bản \(manager.pendingVersion) đã được tải về.\nKhởi động lại để áp dụng bản cập nhật.")
                .font(.system(size: 12))
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .lineSpacing(2)

            HStack(spacing: 10) {
                Spacer()
                Button("Để sau") { manager.dismiss() }
                    .buttonStyle(.bordered)
                    .controlSize(.large)
                Button("Khởi động lại") { manager.restartToUpdate() }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.large)
            }
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
        case let .heading(text):
            Text(text)
                .font(.system(size: 11, weight: .semibold))
                .foregroundColor(.primary.opacity(0.7))
                .padding(.top, 6)
                .padding(.bottom, 1)
        case let .bullet(text):
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
                while l.hasPrefix("#") {
                    l = String(l.dropFirst())
                }
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

    func makeNSView(context _: Context) -> NSVisualEffectView {
        let view = NSVisualEffectView()
        view.material = material
        view.blendingMode = blendingMode
        view.state = .active
        return view
    }

    func updateNSView(_ nsView: NSVisualEffectView, context _: Context) {
        nsView.material = material
        nsView.blendingMode = blendingMode
    }
}
