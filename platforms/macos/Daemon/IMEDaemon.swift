import Foundation
import AppKit
import Carbon

// MARK: - IME Daemon

/// XPC Service daemon that handles keyboard processing
/// Runs as a headless background service (~3-5MB memory)
final class IMEDaemon: NSObject {
    static let shared = IMEDaemon()

    private let listener: NSXPCListener
    private var connections = [NSXPCConnection]()

    // MARK: - State

    private var isEnabled = true
    private var currentMethod = 0  // 0 = Telex, 1 = VNI
    private var isRecordingShortcut = false

    // MARK: - Init

    private override init() {
        self.listener = NSXPCListener.service()
        super.init()
        listener.delegate = self
    }

    // MARK: - Run

    /// Main entry point - starts the daemon
    func run() {
        Log.info("IMEDaemon starting...")

        // Initialize Rust engine
        RustBridge.initialize()

        // Load saved settings
        loadSettings()

        // Start keyboard hook
        KeyboardHookManager.shared.start()

        // Start per-app mode manager
        PerAppModeManager.shared.start()

        // Start input source observer
        InputSourceObserver.shared.start()

        // Setup internal observers
        setupObservers()

        // Resume XPC listener
        listener.resume()

        Log.info("IMEDaemon started, waiting for connections...")

        // Keep running
        RunLoop.current.run()
    }

    // MARK: - Settings

    private func loadSettings() {
        // Load from UserDefaults (shared App Group)
        let defaults = UserDefaults.standard

        isEnabled = defaults.bool(forKey: "isEnabled")
        currentMethod = defaults.integer(forKey: "inputMethod")

        RustBridge.setEnabled(isEnabled)
        RustBridge.setMethod(currentMethod)
        RustBridge.setModernTone(defaults.bool(forKey: "modernTone"))
        RustBridge.setSkipWShortcut(defaults.bool(forKey: "skipWShortcut"))
        RustBridge.setEscRestore(defaults.bool(forKey: "escRestore"))
        RustBridge.setEnglishAutoRestore(defaults.bool(forKey: "englishAutoRestore"))
        RustBridge.setAutoCapitalize(defaults.bool(forKey: "autoCapitalize"))
        RustBridge.setFreeTone(defaults.bool(forKey: "freeTone"))

        // Load shortcuts
        if let data = defaults.data(forKey: "shortcuts"),
           let shortcuts = try? JSONDecoder().decode([XPCShortcutItem].self, from: data) {
            syncShortcutsInternal(shortcuts)
        }

        Log.info("Settings loaded: enabled=\(isEnabled), method=\(currentMethod)")
    }

    // MARK: - Observers

    private func setupObservers() {
        // Toggle triggered by keyboard shortcut
        NotificationCenter.default.addObserver(
            forName: .toggleVietnamese,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.handleToggle()
        }

        // Shortcut recorded
        NotificationCenter.default.addObserver(
            forName: .shortcutRecorded,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let shortcut = notification.object as? KeyboardShortcut else { return }
            self?.handleShortcutRecorded(shortcut)
        }

        // Shortcut recording cancelled
        NotificationCenter.default.addObserver(
            forName: .shortcutRecordingCancelled,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.handleShortcutCancelled()
        }

        // Input source changed
        NotificationCenter.default.addObserver(
            forName: .inputSourceChanged,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            let allowed = (notification.userInfo?["allowed"] as? Bool) ?? true
            let displayChar = (notification.userInfo?["displayChar"] as? String) ?? ""
            self?.handleInputSourceChanged(allowed: allowed, displayChar: displayChar)
        }
    }

    // MARK: - Internal Handlers

    private func handleToggle() {
        isEnabled.toggle()
        RustBridge.setEnabled(isEnabled)
        UserDefaults.standard.set(isEnabled, forKey: "isEnabled")

        // Notify all connected UI apps
        notifyStateChanged()

        // Also notify via notification (for toggle triggered callback)
        for connection in connections {
            if let proxy = connection.remoteObjectProxy as? IMEAppProtocol {
                proxy.toggleTriggered()
            }
        }

        Log.info("Toggle: enabled=\(isEnabled)")
    }

    private func handleShortcutRecorded(_ shortcut: KeyboardShortcut) {
        isRecordingShortcut = false

        // Encode shortcut
        let xpcShortcut = XPCKeyboardShortcut(keyCode: shortcut.keyCode, modifiers: shortcut.modifiers)
        if let data = try? JSONEncoder().encode(xpcShortcut) {
            // Notify all connected UI apps
            for connection in connections {
                if let proxy = connection.remoteObjectProxy as? IMEAppProtocol {
                    proxy.shortcutRecorded(data)
                }
            }
        }
    }

    private func handleShortcutCancelled() {
        isRecordingShortcut = false

        // Notify all connected UI apps
        for connection in connections {
            if let proxy = connection.remoteObjectProxy as? IMEAppProtocol {
                proxy.shortcutRecordingCancelled()
            }
        }
    }

    private func handleInputSourceChanged(allowed: Bool, displayChar: String) {
        // Notify all connected UI apps
        for connection in connections {
            if let proxy = connection.remoteObjectProxy as? IMEAppProtocol {
                proxy.inputSourceChanged(allowed: allowed, displayChar: displayChar)
            }
        }
    }

    private func notifyStateChanged() {
        for connection in connections {
            if let proxy = connection.remoteObjectProxy as? IMEAppProtocol {
                proxy.stateChanged(enabled: isEnabled, method: currentMethod)
            }
        }
    }

    // MARK: - Shortcut Sync

    private func syncShortcutsInternal(_ shortcuts: [XPCShortcutItem]) {
        RustBridge.clearShortcuts()
        for shortcut in shortcuts where shortcut.enabled {
            RustBridge.addShortcut(trigger: shortcut.key, replacement: shortcut.value)
        }
        Log.info("Synced \(shortcuts.filter { $0.enabled }.count) shortcuts")
    }
}

// MARK: - NSXPCListenerDelegate

extension IMEDaemon: NSXPCListenerDelegate {
    func listener(_ listener: NSXPCListener, shouldAcceptNewConnection newConnection: NSXPCConnection) -> Bool {
        // Configure interfaces
        newConnection.exportedInterface = NSXPCInterface(with: IMEDaemonProtocol.self)
        newConnection.exportedObject = self

        newConnection.remoteObjectInterface = NSXPCInterface(with: IMEAppProtocol.self)

        // Handle disconnection
        newConnection.invalidationHandler = { [weak self] in
            self?.connections.removeAll { $0 === newConnection }
            Log.info("UI app disconnected, \(self?.connections.count ?? 0) remaining")
        }

        connections.append(newConnection)
        newConnection.resume()

        Log.info("UI app connected, \(connections.count) total")

        // Send current state to new connection
        DispatchQueue.main.async { [weak self] in
            guard let self = self,
                  let proxy = newConnection.remoteObjectProxy as? IMEAppProtocol else { return }
            proxy.stateChanged(enabled: self.isEnabled, method: self.currentMethod)
        }

        return true
    }
}

// MARK: - IMEDaemonProtocol

extension IMEDaemon: IMEDaemonProtocol {

    // MARK: Engine State

    func setEnabled(_ enabled: Bool) {
        isEnabled = enabled
        RustBridge.setEnabled(enabled)
        UserDefaults.standard.set(enabled, forKey: "isEnabled")
        notifyStateChanged()
        Log.info("setEnabled: \(enabled)")
    }

    func setMethod(_ method: Int) {
        currentMethod = method
        RustBridge.setMethod(method)
        UserDefaults.standard.set(method, forKey: "inputMethod")
        notifyStateChanged()
        Log.info("setMethod: \(method)")
    }

    // MARK: Engine Options

    func setModernTone(_ modern: Bool) {
        RustBridge.setModernTone(modern)
        UserDefaults.standard.set(modern, forKey: "modernTone")
    }

    func setSkipWShortcut(_ skip: Bool) {
        RustBridge.setSkipWShortcut(skip)
        UserDefaults.standard.set(skip, forKey: "skipWShortcut")
    }

    func setEscRestore(_ enabled: Bool) {
        RustBridge.setEscRestore(enabled)
        UserDefaults.standard.set(enabled, forKey: "escRestore")
    }

    func setEnglishAutoRestore(_ enabled: Bool) {
        RustBridge.setEnglishAutoRestore(enabled)
        UserDefaults.standard.set(enabled, forKey: "englishAutoRestore")
    }

    func setAutoCapitalize(_ enabled: Bool) {
        RustBridge.setAutoCapitalize(enabled)
        UserDefaults.standard.set(enabled, forKey: "autoCapitalize")
    }

    func setFreeTone(_ enabled: Bool) {
        RustBridge.setFreeTone(enabled)
        UserDefaults.standard.set(enabled, forKey: "freeTone")
    }

    // MARK: Shortcuts

    func syncShortcuts(_ shortcutsData: Data) {
        guard let shortcuts = try? JSONDecoder().decode([XPCShortcutItem].self, from: shortcutsData) else {
            Log.info("syncShortcuts: failed to decode")
            return
        }
        syncShortcutsInternal(shortcuts)
        UserDefaults.standard.set(shortcutsData, forKey: "shortcuts")
    }

    func addShortcut(_ trigger: String, replacement: String) {
        RustBridge.addShortcut(trigger: trigger, replacement: replacement)
    }

    func removeShortcut(_ trigger: String) {
        RustBridge.removeShortcut(trigger: trigger)
    }

    func clearShortcuts() {
        RustBridge.clearShortcuts()
    }

    // MARK: Shortcut Recording

    func startShortcutRecording() {
        isRecordingShortcut = true
        startShortcutRecordingInternal()
        Log.info("startShortcutRecording")
    }

    func stopShortcutRecording() {
        isRecordingShortcut = false
        stopShortcutRecordingInternal()
        Log.info("stopShortcutRecording")
    }

    // MARK: Buffer Management

    func clearBuffer() {
        RustBridge.clearBuffer()
    }

    func clearBufferAll() {
        RustBridge.clearBufferAll()
        TextInjector.shared.clearSessionBuffer()
    }

    // MARK: Per-App Mode

    func setPerAppModeEnabled(_ enabled: Bool) {
        AppState.shared.perAppModeEnabled = enabled
        UserDefaults.standard.set(enabled, forKey: "perAppModeEnabled")
    }

    func savePerAppMode(bundleId: String, enabled: Bool) {
        AppState.shared.savePerAppMode(bundleId: bundleId, enabled: enabled)
    }

    // MARK: Status Query

    func getStatus(withReply reply: @escaping (Bool, Int) -> Void) {
        reply(isEnabled, currentMethod)
    }
}

// Note: Log, Notifications, and other helpers are defined in DaemonEngine.swift and DaemonHelpers.swift
