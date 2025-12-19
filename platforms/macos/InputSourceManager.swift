import Foundation
import Carbon.HIToolbox
import AppKit

// MARK: - Input Source Item Model

/// Represents an input source on the system
struct InputSourceItem: Identifiable, Codable, Hashable {
    let id: String              // e.g., "com.apple.keylayout.ABC"
    let localizedName: String   // e.g., "ABC"
    let languageCode: String?   // e.g., "en", "ja", "zh"
    var isEnabled: Bool         // Whether this input source is enabled in macOS

    var flagEmoji: String {
        switch languageCode {
        case "ja": return "🇯🇵"
        case "zh", "zh-Hans", "zh-Hant": return "🇨🇳"
        case "ko": return "🇰🇷"
        case "th": return "🇹🇭"
        case "vi": return "🇻🇳"
        case "fr": return "🇫🇷"
        case "de": return "🇩🇪"
        case "es": return "🇪🇸"
        case "ru": return "🇷🇺"
        case "ar": return "🇸🇦"
        case "en": return "🇺🇸"
        default: return "🌐"
        }
    }

    var displayName: String {
        "\(flagEmoji) \(localizedName)"
    }
}

// MARK: - Default Vietnamese Input Sources

/// Default input sources that enable Gõ Nhanh (English keyboards for Vietnamese typing)
let defaultVietnameseInputSources: Set<String> = [
    "com.apple.keylayout.ABC",
    "com.apple.keylayout.US",
    "com.apple.keylayout.USInternational-PC",
    "com.apple.keylayout.USExtended",
    "com.apple.keylayout.British",
    "com.apple.keylayout.British-PC",
    "com.apple.keylayout.Australian",
    "com.apple.keylayout.ABC-AZERTY",
    "com.apple.keylayout.ABC-QWERTZ"
]

// MARK: - Input Source Manager

/// Manages macOS input sources - list, enable, disable, switch
class InputSourceManager {
    static let shared = InputSourceManager()

    private init() {}

    // MARK: - Get Input Sources

    /// Get all enabled (active) input sources on the system
    func getEnabledInputSources() -> [InputSourceItem] {
        let properties: [CFString: Any] = [
            kTISPropertyInputSourceIsEnabled: true,
            kTISPropertyInputSourceIsSelectCapable: true
        ]

        guard let cfSources = TISCreateInputSourceList(properties as CFDictionary, false)?.takeRetainedValue() else {
            return []
        }

        let sources = cfSources as? [TISInputSource] ?? []
        return sources.compactMap { parseInputSource($0, isEnabled: true) }
    }

    /// Get all available input sources (including disabled ones)
    func getAllAvailableInputSources() -> [InputSourceItem] {
        let properties: [CFString: Any] = [
            kTISPropertyInputSourceIsSelectCapable: true
        ]

        guard let cfSources = TISCreateInputSourceList(properties as CFDictionary, true)?.takeRetainedValue() else {
            return []
        }

        let sources = cfSources as? [TISInputSource] ?? []
        let enabledIds = Set(getEnabledInputSources().map { $0.id })

        return sources.compactMap { source -> InputSourceItem? in
            guard var item = parseInputSource(source, isEnabled: false) else { return nil }
            item.isEnabled = enabledIds.contains(item.id)
            return item
        }
    }

    /// Parse a TISInputSource to InputSourceItem
    private func parseInputSource(_ source: TISInputSource, isEnabled: Bool) -> InputSourceItem? {
        guard let idPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID),
              let namePtr = TISGetInputSourceProperty(source, kTISPropertyLocalizedName) else {
            return nil
        }

        let id = Unmanaged<CFString>.fromOpaque(idPtr).takeUnretainedValue() as String
        let name = Unmanaged<CFString>.fromOpaque(namePtr).takeUnretainedValue() as String

        // Get language code
        var languageCode: String? = nil
        if let langsPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceLanguages) {
            let langs = Unmanaged<CFArray>.fromOpaque(langsPtr).takeUnretainedValue() as? [String]
            languageCode = langs?.first
        }

        return InputSourceItem(
            id: id,
            localizedName: name,
            languageCode: languageCode,
            isEnabled: isEnabled
        )
    }

    // MARK: - Enable/Disable Input Sources

    /// Enable an input source (add to macOS input sources)
    func enableInputSource(id: String) -> Bool {
        guard let source = getInputSource(by: id) else { return false }
        let status = TISEnableInputSource(source)
        return status == noErr
    }

    /// Disable an input source (remove from macOS input sources)
    func disableInputSource(id: String) -> Bool {
        guard let source = getInputSource(by: id) else { return false }
        let status = TISDisableInputSource(source)
        return status == noErr
    }

    // MARK: - Switch Input Source

    /// Switch to a specific input source
    func selectInputSource(id: String) {
        guard let source = getInputSource(by: id) else { return }
        TISSelectInputSource(source)
    }

    /// Get current input source ID
    func getCurrentInputSourceId() -> String? {
        guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
              let idPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) else {
            return nil
        }
        return Unmanaged<CFString>.fromOpaque(idPtr).takeUnretainedValue() as String
    }

    /// Get current input source
    func getCurrentInputSource() -> InputSourceItem? {
        guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue() else {
            return nil
        }
        return parseInputSource(source, isEnabled: true)
    }

    // MARK: - Helper

    private func getInputSource(by id: String) -> TISInputSource? {
        let properties: [CFString: Any] = [
            kTISPropertyInputSourceID: id
        ]

        guard let cfSources = TISCreateInputSourceList(properties as CFDictionary, true)?.takeRetainedValue(),
              let sources = cfSources as? [TISInputSource],
              let source = sources.first else {
            return nil
        }
        return source
    }
}

// MARK: - Input Source Observer

/// Observes input source changes and auto-enables/disables Gõ Nhanh
class InputSourceObserver {
    static let shared = InputSourceObserver()

    private var isObserving = false

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
            kTISNotifySelectedKeyboardInputSourceChanged as CFString,
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
            kTISNotifySelectedKeyboardInputSourceChanged as CFString,
            nil
        )
        Log.info("InputSourceObserver stopped")
    }

    /// Handle input source change
    private func handleInputSourceChange() {
        guard let currentId = InputSourceManager.shared.getCurrentInputSourceId() else {
            return
        }

        let appState = AppState.shared
        appState.currentInputSourceId = currentId

        // Check if current input source is in Vietnamese whitelist
        let shouldEnableVietnamese = appState.isVietnameseInputSource(currentId)

        if shouldEnableVietnamese {
            // Restore user preference (enable Gõ Nhanh for Vietnamese typing)
            appState.setInputSourceOverride(nil)
            Log.info("InputSource: \(currentId) - Gõ Nhanh enabled (Vietnamese)")
        } else {
            // Disable Gõ Nhanh (user is typing another language)
            appState.setInputSourceOverride(false)
            Log.info("InputSource: \(currentId) - Gõ Nhanh disabled (other language)")
        }

        // Notify UI to update
        NotificationCenter.default.post(name: .inputSourceChanged, object: nil)
    }
}

// MARK: - Notifications

extension Notification.Name {
    static let inputSourceChanged = Notification.Name("inputSourceChanged")
}

// MARK: - Log Extension (if not already defined elsewhere)

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
