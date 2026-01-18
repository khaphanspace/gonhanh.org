import Foundation
import Carbon.HIToolbox

/// Non-Latin language codes that require disabling Gõ Nhanh
/// These languages use scripts incompatible with Vietnamese input
private let nonLatinLanguages: Set<String> = [
    // East Asian
    "ja", "zh", "zh-Hans", "zh-Hant", "ko",
    // Southeast Asian (non-Latin scripts)
    "th", "km", "lo", "my",
    // South Asian
    "hi", "mr", "ne", "sa", "bn", "ta", "te", "kn", "ml", "gu", "pa", "or", "si",
    // Middle Eastern
    "ar", "he", "fa", "ur",
    // Other non-Latin
    "ru", "uk", "be", "bg", "mk", "sr", "el", "ka", "hy", "am", "ti",
    // Vietnamese IME (user already has Vietnamese input method)
    "vi",
]

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

        // Initial check - only update display state, don't change enabled
        // (PerAppModeManager already set the correct enabled state)
        handleChangeInitial()
    }

    /// Initial check - only updates display character and allowed flag, doesn't change enabled state
    private func handleChangeInitial() {
        guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
              let idPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) else {
            return
        }

        let currentId = Unmanaged<CFString>.fromOpaque(idPtr).takeUnretainedValue() as String
        lastInputSourceId = currentId
        currentDisplayChar = getDisplayChar(from: source, id: currentId)
        isAllowedInputSource = isInputSourceAllowed(source: source)
        // Don't call setEnabled - let PerAppModeManager handle initial state
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
        isAllowedInputSource = isInputSourceAllowed(source: source)

        if isAllowedInputSource {
            // Restore user preference from AppState (supports per-app mode)
            RustBridge.setEnabled(AppState.shared.isEnabled)
        } else {
            // Force disable for non-Latin input sources
            RustBridge.setEnabled(false)
        }

        // Update menu bar icon
        NotificationCenter.default.post(name: .inputSourceChanged, object: nil)
    }

    private func isInputSourceAllowed(source: TISInputSource) -> Bool {
        // Get primary language of the input source
        guard let langsPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceLanguages),
              let langs = Unmanaged<CFArray>.fromOpaque(langsPtr).takeUnretainedValue() as? [String],
              let lang = langs.first else {
            // No language info → assume Latin (allow)
            return true
        }

        // Block if language is in non-Latin set
        // Also check base language code (e.g., "zh-Hans" → "zh")
        let baseLang = lang.split(separator: "-").first.map(String.init) ?? lang
        return !nonLatinLanguages.contains(lang) && !nonLatinLanguages.contains(baseLang)
    }

    private func getDisplayChar(from source: TISInputSource, id: String) -> String {
        // Get language code
        if let langsPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceLanguages),
           let langs = Unmanaged<CFArray>.fromOpaque(langsPtr).takeUnretainedValue() as? [String],
           let lang = langs.first {
            switch lang {
            // East Asian
            case "ja": return "あ"
            case "zh-Hans", "zh-Hant", "zh": return "中"
            case "ko": return "한"
            // Southeast Asian (ASEAN)
            case "th": return "ก"
            case "km": return "ក"  // Khmer/Cambodian
            case "lo": return "ກ"  // Lao
            case "my": return "က"  // Myanmar/Burmese
            // South Asian
            case "hi", "mr", "ne", "sa": return "अ"  // Hindi, Marathi, Nepali, Sanskrit
            case "bn": return "অ"  // Bengali/Bangla
            case "ta": return "அ"  // Tamil
            // Other common
            case "vi": return "E"  // Vietnamese input source = Gõ Nhanh disabled
            case "ru": return "Р"
            case "ar": return "ع"
            case "he": return "א"
            case "el": return "Ω"
            case "fa", "ur": return "ف"  // Persian, Urdu
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
