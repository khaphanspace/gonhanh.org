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

        CFNotificationCenterAddObserver(
            CFNotificationCenterGetDistributedCenter(),
            Unmanaged.passUnretained(self).toOpaque(),
            inputSourceCallback,
            kTISNotifySelectedKeyboardInputSourceChanged,
            nil,
            .deliverImmediately
        )

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
            return
        }

        let currentId = Unmanaged<CFString>.fromOpaque(idPtr).takeUnretainedValue() as String

        // Skip if same as last
        guard currentId != lastInputSourceId else { return }
        lastInputSourceId = currentId

        // Get display character from input source
        currentDisplayChar = getDisplayChar(from: source, id: currentId)
        isAllowedInputSource = isInputSourceAllowed(currentId)

        if isAllowedInputSource {
            // Restore user preference
            let userEnabled = UserDefaults.standard.object(forKey: "gonhanh.enabled") as? Bool ?? true
            RustBridge.setEnabled(userEnabled)
        } else {
            // Force disable
            RustBridge.setEnabled(false)
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

private let inputSourceCallback: CFNotificationCallback = { _, observer, _, _, _ in
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
