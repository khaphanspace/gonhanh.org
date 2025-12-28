import Foundation

// MARK: - Settings Keys (Shared between Daemon and UI App)

public enum IMESettingsKey {
    public static let enabled = "gonhanh.enabled"
    public static let method = "gonhanh.method"
    public static let hasCompletedOnboarding = "gonhanh.onboarding.completed"
    public static let permissionGranted = "gonhanh.permission.granted"
    public static let toggleShortcut = "gonhanh.shortcut.toggle"
    public static let reopenSettingsAfterUpdate = "gonhanh.update.reopenSettings"
    public static let perAppMode = "gonhanh.perAppMode"
    public static let perAppModes = "gonhanh.perAppModes"
    public static let shortcuts = "gonhanh.shortcuts"
    public static let autoWShortcut = "gonhanh.autoWShortcut"
    public static let escRestore = "gonhanh.escRestore"
    public static let modernTone = "gonhanh.modernTone"
    public static let englishAutoRestore = "gonhanh.englishAutoRestore"
    public static let autoCapitalize = "gonhanh.autoCapitalize"
    public static let launchAtLoginUserDisabled = "gonhanh.launchAtLogin.userDisabled"
    public static let soundEnabled = "gonhanh.soundEnabled"
    public static let freeTone = "gonhanh.freeTone"
}

// MARK: - Input Mode (Shared)

public enum IMEInputMode: Int, CaseIterable, Codable {
    case telex = 0
    case vni = 1

    public var name: String {
        switch self {
        case .telex: return "Telex"
        case .vni: return "VNI"
        }
    }

    public var shortName: String {
        switch self {
        case .telex: return "T"
        case .vni: return "V"
        }
    }
}
