import ApplicationServices
import CoreFoundation
import Foundation

// MARK: - Control State Abstraction

/// Abstracts engine/UI state access so `ControlServer` is deterministic in unit tests.
protocol ControlStateHandling: AnyObject {
    var isEngineReady: Bool { get }
    var isEnabled: Bool { get }
    var isAllowedInputSource: Bool { get }
    var currentInputSourceName: String? { get }
    var currentMethod: InputMode { get }
    var appVersion: String { get }

    func setEnabled(_ enabled: Bool)
    func setMethod(_ method: InputMode)
}

final class AppControlStateHandler: ControlStateHandling {
    var isEngineReady: Bool {
        AXIsProcessTrusted() && KeyboardHookManager.shared.isRunning
    }

    var isEnabled: Bool {
        AppState.shared.isEnabled
    }

    var isAllowedInputSource: Bool {
        InputSourceObserver.shared.isAllowedInputSource
    }

    var currentInputSourceName: String? {
        InputSourceObserver.shared.currentInputSourceName
    }

    var currentMethod: InputMode {
        AppState.shared.currentMethod
    }

    var appVersion: String {
        AppMetadata.version
    }

    func setEnabled(_ enabled: Bool) {
        let appState = AppState.shared
        let changed = appState.isEnabled != enabled
        if changed {
            appState.isEnabled = enabled
            RustBridge.clearBufferAll()
            SoundManager.shared.playToggleSound(enabled: enabled)
        } else {
            RustBridge.setEnabled(enabled)
        }
    }

    func setMethod(_ method: InputMode) {
        AppState.shared.setMethod(method)
        RustBridge.clearBufferAll()
    }
}

// MARK: - Control Server

/// Local `CFMessagePort` server running on the main run loop in `.commonModes`.
/// Handles synchronous CLI control requests while keeping UI/MenuBar/PerAppMode in sync.
final class ControlServer {
    static let shared = ControlServer()

    private let stateHandler: ControlStateHandling
    private var localPort: CFMessagePort?
    private var runLoopSource: CFRunLoopSource?

    init(stateHandler: ControlStateHandling = AppControlStateHandler()) {
        self.stateHandler = stateHandler
    }

    deinit {
        stop()
    }

    @discardableResult
    func start(portName: String = ControlProtocol.portName) -> Bool {
        guard localPort == nil else { return true }

        var context = CFMessagePortContext(
            version: 0,
            info: Unmanaged.passUnretained(self).toOpaque(),
            retain: nil,
            release: nil,
            copyDescription: nil
        )
        var shouldFreeInfo: DarwinBoolean = false
        let port = CFMessagePortCreateLocal(
            kCFAllocatorDefault,
            portName as CFString,
            controlServerCallback,
            &context,
            &shouldFreeInfo
        )
        guard let port else {
            return false
        }

        guard let source = CFMessagePortCreateRunLoopSource(kCFAllocatorDefault, port, 0) else {
            CFMessagePortInvalidate(port)
            return false
        }

        localPort = port
        runLoopSource = source
        CFRunLoopAddSource(CFRunLoopGetMain(), source, .commonModes)
        return true
    }

    func stop() {
        if let source = runLoopSource {
            CFRunLoopRemoveSource(CFRunLoopGetMain(), source, .commonModes)
            runLoopSource = nil
        }
        if let port = localPort {
            CFMessagePortInvalidate(port)
            localPort = nil
        }
    }

    // MARK: - Command Dispatch

    func handleRawCommand(_ raw: String) -> ControlProtocol.Response {
        guard let command = ControlProtocol.Command(rawValue: raw) else {
            return .error("Lệnh không hợp lệ: \(raw)")
        }
        return handleCommand(command)
    }

    func handleCommand(_ command: ControlProtocol.Command) -> ControlProtocol.Response {
        switch command {
        case .get:
            return .ok(effectiveEnabled ? "vi" : "en")

        case .vi:
            guard stateHandler.isEngineReady else {
                return .error(notReadyMessage)
            }
            stateHandler.setEnabled(true)
            return .ok()

        case .en:
            // When gated off by a non-Latin input source, GoNhanh is already effectively "en".
            // Preserve `AppState.isEnabled` so switching back to ABC restores Vietnamese automatically.
            if stateHandler.isEngineReady, stateHandler.isAllowedInputSource {
                stateHandler.setEnabled(false)
            }
            return .ok()

        case .on:
            guard stateHandler.isEngineReady else {
                return .error(notReadyMessage)
            }
            stateHandler.setEnabled(true)
            return .ok(statusLine())

        case .off:
            stateHandler.setEnabled(false)
            return .ok(statusLine())

        case .toggle:
            guard stateHandler.isEngineReady else {
                return .error(notReadyMessage)
            }
            stateHandler.setEnabled(!effectiveEnabled)
            return .ok(statusLine())

        case .telex:
            guard stateHandler.isEngineReady else {
                return .error(notReadyMessage)
            }
            stateHandler.setMethod(.telex)
            stateHandler.setEnabled(true)
            return .ok(statusLine())

        case .vni:
            guard stateHandler.isEngineReady else {
                return .error(notReadyMessage)
            }
            stateHandler.setMethod(.vni)
            stateHandler.setEnabled(true)
            return .ok(statusLine())

        case .status:
            return .ok(statusLine())

        case .version:
            return .ok("Gõ Nhanh v\(stateHandler.appVersion)")
        }
    }

    // MARK: - Helpers

    private var effectiveEnabled: Bool {
        stateHandler.isEngineReady && stateHandler.isEnabled && stateHandler.isAllowedInputSource
    }

    private var notReadyMessage: String {
        "Gõ Nhanh chưa sẵn sàng. Hãy hoàn tất thiết lập và cấp quyền Accessibility."
    }

    private func statusLine() -> String {
        let inactiveReason: String? = if !stateHandler.isEngineReady {
            "chưa cấp quyền Accessibility"
        } else if !stateHandler.isAllowedInputSource {
            if let sourceName = stateHandler.currentInputSourceName, !sourceName.isEmpty {
                "input source hiện tại: \(sourceName)"
            } else {
                "tạm tắt theo input source"
            }
        } else {
            nil
        }

        return ControlProtocol.formatStatus(
            effectiveEnabled: effectiveEnabled,
            methodName: stateHandler.currentMethod.name,
            inactiveReason: inactiveReason
        )
    }
}

// MARK: - CFMessagePort Callback

private let controlServerCallback: CFMessagePortCallBack = { _, _, data, info in
    guard let info else { return nil }
    let server = Unmanaged<ControlServer>.fromOpaque(info).takeUnretainedValue()
    let rawCommand = if let data {
        String(decoding: data as Data, as: UTF8.self)
    } else {
        ""
    }
    let response = server.handleRawCommand(rawCommand)
    let responseData = response.encode() as CFData
    return Unmanaged.passRetained(responseData)
}
