import Foundation
import Combine

// MARK: - IME XPC Client

/// XPC client for communicating with the daemon
/// Used by the UI app to control the engine running in the daemon
final class IMEClient: NSObject, ObservableObject {
    static let shared = IMEClient()

    private var connection: NSXPCConnection?
    private var daemonProxy: IMEDaemonProtocol?
    private var isConnecting = false
    private var reconnectTimer: Timer?

    // Published state (mirrored from daemon)
    @Published private(set) var isConnected = false
    @Published private(set) var isEnabled = true
    @Published private(set) var currentMethod = 0

    // Callbacks
    var onToggleTriggered: (() -> Void)?
    var onInputSourceChanged: ((Bool, String) -> Void)?
    var onShortcutRecorded: ((XPCKeyboardShortcut) -> Void)?
    var onShortcutRecordingCancelled: (() -> Void)?

    private override init() {
        super.init()
    }

    // MARK: - Connection Management

    /// Connect to the daemon XPC service
    func connect() {
        guard !isConnecting && connection == nil else { return }
        isConnecting = true

        let conn = NSXPCConnection(serviceName: IMEXPCService.serviceName)

        // Configure daemon interface (what we call)
        conn.remoteObjectInterface = NSXPCInterface(with: IMEDaemonProtocol.self)

        // Configure app interface (what daemon calls back)
        conn.exportedInterface = NSXPCInterface(with: IMEAppProtocol.self)
        conn.exportedObject = self

        // Handle connection errors
        conn.invalidationHandler = { [weak self] in
            DispatchQueue.main.async {
                self?.handleDisconnect()
            }
        }

        conn.interruptionHandler = { [weak self] in
            DispatchQueue.main.async {
                self?.handleInterruption()
            }
        }

        connection = conn
        conn.resume()

        // Get proxy with error handling
        daemonProxy = conn.remoteObjectProxyWithErrorHandler { [weak self] error in
            print("IMEClient: XPC error - \(error.localizedDescription)")
            DispatchQueue.main.async {
                self?.handleDisconnect()
            }
        } as? IMEDaemonProtocol

        isConnecting = false
        isConnected = true
        print("IMEClient: Connected to daemon")

        // Request initial state
        requestStatus()
    }

    /// Disconnect from the daemon
    func disconnect() {
        reconnectTimer?.invalidate()
        reconnectTimer = nil
        connection?.invalidate()
        connection = nil
        daemonProxy = nil
        isConnected = false
    }

    private func handleDisconnect() {
        connection = nil
        daemonProxy = nil
        isConnected = false
        isConnecting = false
        print("IMEClient: Disconnected from daemon")
        scheduleReconnect()
    }

    private func handleInterruption() {
        print("IMEClient: Connection interrupted")
        // XPC will automatically try to reconnect
    }

    private func scheduleReconnect() {
        reconnectTimer?.invalidate()
        reconnectTimer = Timer.scheduledTimer(withTimeInterval: 2.0, repeats: false) { [weak self] _ in
            self?.connect()
        }
    }

    private func requestStatus() {
        daemonProxy?.getStatus { [weak self] enabled, method in
            DispatchQueue.main.async {
                self?.isEnabled = enabled
                self?.currentMethod = method
            }
        }
    }

    // MARK: - Engine Control (UI → Daemon)

    /// Set engine enabled state
    func setEnabled(_ enabled: Bool) {
        daemonProxy?.setEnabled(enabled)
    }

    /// Set input method (0 = Telex, 1 = VNI)
    func setMethod(_ method: Int) {
        daemonProxy?.setMethod(method)
    }

    /// Set modern tone placement
    func setModernTone(_ modern: Bool) {
        daemonProxy?.setModernTone(modern)
    }

    /// Set skip W shortcut
    func setSkipWShortcut(_ skip: Bool) {
        daemonProxy?.setSkipWShortcut(skip)
    }

    /// Set ESC restore
    func setEscRestore(_ enabled: Bool) {
        daemonProxy?.setEscRestore(enabled)
    }

    /// Set English auto-restore
    func setEnglishAutoRestore(_ enabled: Bool) {
        daemonProxy?.setEnglishAutoRestore(enabled)
    }

    /// Set auto-capitalize
    func setAutoCapitalize(_ enabled: Bool) {
        daemonProxy?.setAutoCapitalize(enabled)
    }

    /// Set free tone
    func setFreeTone(_ enabled: Bool) {
        daemonProxy?.setFreeTone(enabled)
    }

    // MARK: - Shortcuts

    /// Sync all shortcuts to daemon
    func syncShortcuts(_ shortcuts: [XPCShortcutItem]) {
        guard let data = try? JSONEncoder().encode(shortcuts) else { return }
        daemonProxy?.syncShortcuts(data)
    }

    /// Add a single shortcut
    func addShortcut(trigger: String, replacement: String) {
        daemonProxy?.addShortcut(trigger, replacement: replacement)
    }

    /// Remove a shortcut
    func removeShortcut(trigger: String) {
        daemonProxy?.removeShortcut(trigger)
    }

    /// Clear all shortcuts
    func clearShortcuts() {
        daemonProxy?.clearShortcuts()
    }

    // MARK: - Shortcut Recording

    /// Start recording a keyboard shortcut
    func startShortcutRecording() {
        daemonProxy?.startShortcutRecording()
    }

    /// Stop recording a keyboard shortcut
    func stopShortcutRecording() {
        daemonProxy?.stopShortcutRecording()
    }

    // MARK: - Buffer Management

    /// Clear the typing buffer
    func clearBuffer() {
        daemonProxy?.clearBuffer()
    }

    /// Clear buffer and word history
    func clearBufferAll() {
        daemonProxy?.clearBufferAll()
    }

    // MARK: - Per-App Mode

    /// Set per-app mode enabled
    func setPerAppModeEnabled(_ enabled: Bool) {
        daemonProxy?.setPerAppModeEnabled(enabled)
    }

    /// Save per-app mode for a specific app
    func savePerAppMode(bundleId: String, enabled: Bool) {
        daemonProxy?.savePerAppMode(bundleId: bundleId, enabled: enabled)
    }
}

// MARK: - IMEAppProtocol (Daemon → UI)

extension IMEClient: IMEAppProtocol {

    /// Called when engine state changes
    func stateChanged(enabled: Bool, method: Int) {
        DispatchQueue.main.async { [weak self] in
            self?.isEnabled = enabled
            self?.currentMethod = method
        }
    }

    /// Called when a shortcut is recorded
    func shortcutRecorded(_ shortcutData: Data) {
        guard let shortcut = try? JSONDecoder().decode(XPCKeyboardShortcut.self, from: shortcutData) else { return }
        DispatchQueue.main.async { [weak self] in
            self?.onShortcutRecorded?(shortcut)
        }
    }

    /// Called when shortcut recording is cancelled
    func shortcutRecordingCancelled() {
        DispatchQueue.main.async { [weak self] in
            self?.onShortcutRecordingCancelled?()
        }
    }

    /// Called when input source changes
    func inputSourceChanged(allowed: Bool, displayChar: String) {
        DispatchQueue.main.async { [weak self] in
            self?.onInputSourceChanged?(allowed, displayChar)
        }
    }

    /// Called when toggle is triggered by keyboard shortcut
    func toggleTriggered() {
        DispatchQueue.main.async { [weak self] in
            self?.onToggleTriggered?()
        }
    }
}
