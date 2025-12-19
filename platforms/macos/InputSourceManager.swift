import Foundation
import Carbon.HIToolbox

// MARK: - Input Source Manager

/// Manages macOS input sources - get current, list all, switch
class InputSourceManager {
    static let shared = InputSourceManager()

    private init() {}

    /// Get current input source ID
    func getCurrentInputSourceId() -> String? {
        guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
              let idPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) else {
            return nil
        }
        return Unmanaged<CFString>.fromOpaque(idPtr).takeUnretainedValue() as String
    }

    /// Get current input source localized name
    func getCurrentInputSourceName() -> String? {
        guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
              let namePtr = TISGetInputSourceProperty(source, kTISPropertyLocalizedName) else {
            return nil
        }
        return Unmanaged<CFString>.fromOpaque(namePtr).takeUnretainedValue() as String
    }
}

// MARK: - Input Source Observer

/// Observes input source changes via CFNotificationCenter
class InputSourceObserver {
    static let shared = InputSourceObserver()

    private var isObserving = false

    /// Callback when input source changes
    var onInputSourceChanged: ((String) -> Void)?

    private init() {}

    /// Start observing input source changes
    func start() {
        guard !isObserving else { return }
        isObserving = true

        let callback: CFNotificationCallback = { _, _, _, _, _ in
            DispatchQueue.main.async {
                InputSourceObserver.shared.handleInputSourceChange()
            }
        }

        CFNotificationCenterAddObserver(
            CFNotificationCenterGetDistributedCenter(),
            nil,
            callback,
            kTISNotifySelectedKeyboardInputSourceChanged,
            nil,
            .deliverImmediately
        )

        // Check initial state
        handleInputSourceChange()
        Log.info("InputSourceObserver started")
    }

    /// Stop observing input source changes
    func stop() {
        guard isObserving else { return }
        isObserving = false

        CFNotificationCenterRemoveObserver(
            CFNotificationCenterGetDistributedCenter(),
            nil,
            CFNotificationName(kTISNotifySelectedKeyboardInputSourceChanged),
            nil
        )
        Log.info("InputSourceObserver stopped")
    }

    /// Handle input source change
    private func handleInputSourceChange() {
        guard let currentId = InputSourceManager.shared.getCurrentInputSourceId() else {
            return
        }

        Log.info("Input source changed to: \(currentId)")

        // Notify callback
        onInputSourceChanged?(currentId)

        // Post notification for UI updates
        NotificationCenter.default.post(name: .inputSourceChanged, object: currentId)
    }
}

// MARK: - Notifications

extension Notification.Name {
    static let inputSourceChanged = Notification.Name("inputSourceChanged")
}

// MARK: - Log Helper

private enum Log {
    private static let logPath = "/tmp/gonhanh_debug.log"
    private static var isEnabled: Bool { FileManager.default.fileExists(atPath: logPath) }

    static func info(_ msg: String) {
        guard isEnabled, let handle = FileHandle(forWritingAtPath: logPath) else { return }
        let ts = String(format: "%02d:%02d:%02d.%03d",
                        Calendar.current.component(.hour, from: Date()),
                        Calendar.current.component(.minute, from: Date()),
                        Calendar.current.component(.second, from: Date()),
                        Calendar.current.component(.nanosecond, from: Date()) / 1_000_000)
        handle.seekToEndOfFile()
        handle.write("[\(ts)] I: \(msg)\n".data(using: .utf8)!)
        handle.closeFile()
    }
}
