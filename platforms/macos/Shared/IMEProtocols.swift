import Foundation

// MARK: - XPC Protocol: UI App → Daemon

/// Protocol for commands sent from UI app to daemon
@objc(IMEDaemonProtocol)
public protocol IMEDaemonProtocol: NSObjectProtocol {
    // Engine state
    func setEnabled(_ enabled: Bool)
    func setMethod(_ method: Int)

    // Engine options
    func setModernTone(_ modern: Bool)
    func setSkipWShortcut(_ skip: Bool)
    func setEscRestore(_ enabled: Bool)
    func setEnglishAutoRestore(_ enabled: Bool)
    func setAutoCapitalize(_ enabled: Bool)
    func setFreeTone(_ enabled: Bool)

    // Shortcuts
    func syncShortcuts(_ shortcutsData: Data)
    func addShortcut(_ trigger: String, replacement: String)
    func removeShortcut(_ trigger: String)
    func clearShortcuts()

    // Shortcut recording
    func startShortcutRecording()
    func stopShortcutRecording()

    // Buffer management
    func clearBuffer()
    func clearBufferAll()

    // Per-app mode
    func setPerAppModeEnabled(_ enabled: Bool)
    func savePerAppMode(bundleId: String, enabled: Bool)

    // Status query (with reply)
    func getStatus(withReply reply: @escaping (_ enabled: Bool, _ method: Int) -> Void)
}

// MARK: - XPC Protocol: Daemon → UI App

/// Protocol for callbacks from daemon to UI app
@objc(IMEAppProtocol)
public protocol IMEAppProtocol: NSObjectProtocol {
    // State changes (daemon → UI)
    func stateChanged(enabled: Bool, method: Int)

    // Shortcut recording result
    func shortcutRecorded(_ shortcutData: Data)
    func shortcutRecordingCancelled()

    // Input source changes
    func inputSourceChanged(allowed: Bool, displayChar: String)

    // Toggle triggered by keyboard shortcut
    func toggleTriggered()
}

// MARK: - Shared Data Structures

/// Keyboard shortcut data for XPC transfer
public struct XPCKeyboardShortcut: Codable {
    public var keyCode: UInt16
    public var modifiers: UInt64

    public init(keyCode: UInt16, modifiers: UInt64) {
        self.keyCode = keyCode
        self.modifiers = modifiers
    }
}

/// Shortcut item for XPC transfer
public struct XPCShortcutItem: Codable {
    public var key: String
    public var value: String
    public var enabled: Bool

    public init(key: String, value: String, enabled: Bool) {
        self.key = key
        self.value = value
        self.enabled = enabled
    }
}

// MARK: - XPC Service Name

public enum IMEXPCService {
    public static let serviceName = "org.gonhanh.GoNhanh.Daemon"
}
