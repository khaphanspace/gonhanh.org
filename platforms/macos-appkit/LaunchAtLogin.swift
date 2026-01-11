import Foundation
import ServiceManagement

// MARK: - Debug Logging

private func debugLog(_ message: String) {
    #if DEBUG
    print(message)
    #endif
}

// MARK: - Launch at Login Manager

/// Protocol for launch at login functionality (enables testing)
protocol LaunchAtLoginProtocol {
    var isEnabled: Bool { get }
    func enable() throws
    func disable() throws
}

/// Manages app's launch at login state using SMAppService
class LaunchAtLoginManager: LaunchAtLoginProtocol {
    static let shared = LaunchAtLoginManager()

    private init() {}

    /// Check if launch at login is currently enabled
    /// Returns true if status is .enabled OR .requiresApproval (user added manually in System Settings)
    var isEnabled: Bool {
        if #available(macOS 13.0, *) {
            let status = SMAppService.mainApp.status
            // .enabled = registered via SMAppService
            // .requiresApproval = user added app manually in System Settings > Login Items
            return status == .enabled || status == .requiresApproval
        }
        return false
    }

    /// Enable launch at login
    func enable() throws {
        if #available(macOS 13.0, *) {
            let status = SMAppService.mainApp.status
            // Don't register if already enabled or user added manually (requiresApproval)
            // This prevents duplicate login items on version updates
            if status != .enabled && status != .requiresApproval {
                try SMAppService.mainApp.register()
                debugLog("[LaunchAtLogin] Enabled successfully")
            } else {
                debugLog("[LaunchAtLogin] Already enabled, skipping registration")
            }
        } else {
            debugLog("[LaunchAtLogin] Requires macOS 13.0+")
        }
    }

    /// Disable launch at login
    func disable() throws {
        if #available(macOS 13.0, *) {
            let status = SMAppService.mainApp.status
            // Unregister if enabled via SMAppService or manually added
            if status == .enabled || status == .requiresApproval {
                try SMAppService.mainApp.unregister()
                debugLog("[LaunchAtLogin] Disabled successfully")
            }
        }
    }

    /// Get current status as string (for debugging)
    var statusDescription: String {
        if #available(macOS 13.0, *) {
            switch SMAppService.mainApp.status {
            case .enabled:
                return "enabled"
            case .notFound:
                return "notFound"
            case .notRegistered:
                return "notRegistered"
            case .requiresApproval:
                return "requiresApproval"
            @unknown default:
                return "unknown"
            }
        }
        return "unsupported"
    }
}

// MARK: - Mock for Testing

/// Mock implementation for unit testing
class MockLaunchAtLoginManager: LaunchAtLoginProtocol {
    private(set) var isEnabled: Bool = false
    private(set) var enableCallCount = 0
    private(set) var disableCallCount = 0
    var shouldThrowOnEnable = false
    var shouldThrowOnDisable = false

    func enable() throws {
        enableCallCount += 1
        if shouldThrowOnEnable {
            throw LaunchAtLoginError.registrationFailed
        }
        isEnabled = true
    }

    func disable() throws {
        disableCallCount += 1
        if shouldThrowOnDisable {
            throw LaunchAtLoginError.unregistrationFailed
        }
        isEnabled = false
    }

    func reset() {
        isEnabled = false
        enableCallCount = 0
        disableCallCount = 0
        shouldThrowOnEnable = false
        shouldThrowOnDisable = false
    }
}

enum LaunchAtLoginError: Error {
    case registrationFailed
    case unregistrationFailed
}

// MARK: - Settings Integration

extension LaunchAtLoginManager {
    /// Key for UserDefaults (backup/fallback)
    static let userDefaultsKey = "launchAtLoginEnabled"

    /// Sync state with UserDefaults
    func syncWithUserDefaults() {
        UserDefaults.standard.set(isEnabled, forKey: Self.userDefaultsKey)
    }

    /// Get cached state from UserDefaults (for UI before SMAppService check)
    var cachedState: Bool {
        UserDefaults.standard.bool(forKey: Self.userDefaultsKey)
    }
}
