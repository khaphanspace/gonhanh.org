import Foundation
import Carbon.HIToolbox

/// The only input source that allows Gõ Nhanh
private let allowedInputSource = "com.apple.keylayout.ABC"

// MARK: - Input Source Observer

/// Observes input source changes and auto-enables/disables Gõ Nhanh
final class InputSourceObserver {
    static let shared = InputSourceObserver()

    private var isObserving = false
    private var lastInputSourceId: String?

    /// Current input source display character (for menu icon)
    private(set) var currentDisplayChar: String = "V"

    /// Whether Gõ Nhanh is allowed for current input source
    private(set) var isAllowedInputSource: Bool = true

    private init() {}

    func start() {
        guard !isObserving else { return }
        isObserving = true

        Log.debug("[InputSource] Observer starting...")

        CFNotificationCenterAddObserver(
            CFNotificationCenterGetDistributedCenter(),
            Unmanaged.passUnretained(self).toOpaque(),
            inputSourceCallback,
            kTISNotifySelectedKeyboardInputSourceChanged,
            nil,
            .deliverImmediately
        )

        Log.debug("[InputSource] Observer registered, checking initial state")
        handleChange()
    }

    func stop() {
        guard isObserving else { return }
        isObserving = false

        CFNotificationCenterRemoveObserver(
            CFNotificationCenterGetDistributedCenter(),
            Unmanaged.passUnretained(self).toOpaque(),
            CFNotificationName(kTISNotifySelectedKeyboardInputSourceChanged),
            nil
        )
    }

    fileprivate func handleChange() {
        guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
              let idPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) else {
            Log.debug("[InputSource] Failed to get current input source")
            return
        }

        let currentId = Unmanaged<CFString>.fromOpaque(idPtr).takeUnretainedValue() as String
        Log.debug("[InputSource] Detected: \(currentId)")

        // Skip if same as last
        guard currentId != lastInputSourceId else {
            Log.debug("[InputSource] Same as last, skipping")
            return
        }
        lastInputSourceId = currentId

        // Get display character from input source
        currentDisplayChar = getDisplayChar(from: source, id: currentId)
        isAllowedInputSource = isInputSourceAllowed(currentId)

        Log.debug("[InputSource] char=\(currentDisplayChar) allowed=\(isAllowedInputSource)")

        if isAllowedInputSource {
            // Restore user preference
            let userEnabled = UserDefaults.standard.object(forKey: "gonhanh.enabled") as? Bool ?? true
            RustBridge.setEnabled(userEnabled)
            Log.debug("[InputSource] Restored user pref: \(userEnabled)")
        } else {
            // Force disable
            RustBridge.setEnabled(false)
            Log.debug("[InputSource] Force disabled")
        }

        // Update menu bar icon
        NotificationCenter.default.post(name: .inputSourceChanged, object: nil)
    }

    private func isInputSourceAllowed(_ id: String) -> Bool {
        id == allowedInputSource
    }

    private func getDisplayChar(from source: TISInputSource, id: String) -> String {
        // Get language code
        if let langsPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceLanguages),
           let langs = Unmanaged<CFArray>.fromOpaque(langsPtr).takeUnretainedValue() as? [String],
           let lang = langs.first {
            switch lang {
            case "ja": return "あ"
            case "zh-Hans", "zh-Hant", "zh": return "中"
            case "ko": return "한"
            case "th": return "ไ"
            case "vi": return "V"
            case "ru": return "Р"
            case "ar": return "ع"
            case "he": return "א"
            case "el": return "Ω"
            default: break
            }
        }

        // Fallback: use first char of localized name
        if let namePtr = TISGetInputSourceProperty(source, kTISPropertyLocalizedName) {
            let name = Unmanaged<CFString>.fromOpaque(namePtr).takeUnretainedValue() as String
            if let first = name.first {
                return String(first).uppercased()
            }
        }

        return "E"
    }
}

// MARK: - C Callback

private let inputSourceCallback: CFNotificationCallback = { _, observer, name, _, _ in
    // Log immediately to verify callback is triggered
    let logPath = "/tmp/gonhanh_debug.log"
    if FileManager.default.fileExists(atPath: logPath),
       let handle = FileHandle(forWritingAtPath: logPath) {
        let ts = ISO8601DateFormatter().string(from: Date())
        let nameStr = name?.rawValue as String? ?? "nil"
        handle.seekToEndOfFile()
        handle.write("[\(ts)] [InputSource] CALLBACK FIRED! name=\(nameStr)\n".data(using: .utf8)!)
        try? handle.close()
    }

    guard let observer = observer else { return }
    let instance = Unmanaged<InputSourceObserver>.fromOpaque(observer).takeUnretainedValue()
    DispatchQueue.main.async {
        instance.handleChange()
    }
}

// MARK: - Notification

extension Notification.Name {
    static let inputSourceChanged = Notification.Name("inputSourceChanged")
}

// MARK: - Log Helper

private enum Log {
    static func debug(_ msg: String) {
        let logPath = "/tmp/gonhanh_debug.log"
        guard FileManager.default.fileExists(atPath: logPath),
              let handle = FileHandle(forWritingAtPath: logPath) else { return }
        let ts = ISO8601DateFormatter().string(from: Date())
        handle.seekToEndOfFile()
        handle.write("[\(ts)] \(msg)\n".data(using: .utf8)!)
        try? handle.close()
    }
}
