import Foundation
import Carbon.HIToolbox

// MARK: - Allowed Input Sources

/// English keyboard layouts that allow Gõ Nhanh
private let allowedInputSources: Set<String> = [
    "com.apple.keylayout.ABC",
    "com.apple.keylayout.US",
    "com.apple.keylayout.USExtended",
    "com.apple.keylayout.USInternational-PC",
    "com.apple.keylayout.British",
    "com.apple.keylayout.British-PC",
    "com.apple.keylayout.Australian",
    "com.apple.keylayout.ABC-AZERTY",
    "com.apple.keylayout.ABC-QWERTZ",
    "com.apple.keylayout.Colemak",
    "com.apple.keylayout.Dvorak",
    "com.apple.keylayout.DVORAK-QWERTYCMD",
]

// MARK: - Input Source Observer

/// Observes input source changes and auto-enables/disables Gõ Nhanh
final class InputSourceObserver {
    static let shared = InputSourceObserver()

    private var isObserving = false
    private var lastInputSourceId: String?

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

        // Apply initial state
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

        // Skip if same as last (avoid redundant calls)
        guard currentId != lastInputSourceId else { return }
        lastInputSourceId = currentId

        let shouldEnable = allowedInputSources.contains(currentId)

        if shouldEnable {
            // Restore user preference
            let userEnabled = UserDefaults.standard.object(forKey: "gonhanh.enabled") as? Bool ?? true
            RustBridge.setEnabled(userEnabled)
        } else {
            // Force disable for non-English keyboards
            RustBridge.setEnabled(false)
        }

        // Update menu bar
        NotificationCenter.default.post(name: .menuStateChanged, object: nil)
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
