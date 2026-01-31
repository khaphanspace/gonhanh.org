import Foundation
import AppKit

// MARK: - App Metadata (Centralized)
// All project metadata in one place for consistency

enum AppMetadata {
    static let name = "Gõ Nhanh"

    // App Logo - dùng chung cho mọi nơi
    static var logo: NSImage {
        NSImage(named: "AppLogo") ?? NSApp.applicationIconImage ?? NSImage()
    }
    static let displayName = "Gõ Nhanh"
    static let tagline = "Bộ gõ tiếng Việt hiệu suất cao"
    static let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0.0"
    static let buildNumber = Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "1"

    // Author
    static let author = "Kha Phan"
    static let authorEmail = "nhatkha1407@gmail.com"

    // Links
    static let website = "https://gonhanh.org"
    static let repository = "https://github.com/khaphanspace/gonhanh.org"
    static let issuesURL = "https://github.com/khaphanspace/gonhanh.org/issues"
    static let sponsorURL = "https://github.com/sponsors/khaphanspace"
    static let authorLinkedin = "https://www.linkedin.com/in/khaphanspace"

    // Legal
    static let copyright = "Copyright © 2025 \(author). All rights reserved."
    static let license = "GPL-3.0-or-later"

    // Tech
    static let techStack = "Rust + SwiftUI"

    // Credits for About panel
    static var credits: String {
        """
        \(tagline)

        Tác giả: \(author)

        Made with Rust + SwiftUI
        """
    }

    // Full about text
    static var aboutText: String {
        """
        \(displayName) v\(version)

        \(tagline)

        Tác giả: \(author)
        Website: \(website)
        GitHub: \(repository)

        \(copyright)
        License: \(license)
        """
    }
}

// MARK: - Settings Keys (Shared)

enum SettingsKey {
    static let enabled = "gonhanh.enabled"
    static let method = "gonhanh.method"
    static let hasCompletedOnboarding = "gonhanh.onboarding.completed"
    static let permissionGranted = "gonhanh.permission.granted"
    static let toggleShortcut = "gonhanh.shortcut.toggle"
    static let reopenSettingsAfterUpdate = "gonhanh.update.reopenSettings"
    static let perAppMode = "gonhanh.perAppMode"
    static let perAppModes = "gonhanh.perAppModes"
    static let shortcuts = "gonhanh.shortcuts"
    static let autoWShortcut = "gonhanh.autoWShortcut"
    static let bracketShortcut = "gonhanh.bracketShortcut"
    static let restoreShortcutEnabled = "gonhanh.escRestore"  // Keep old key for backward compat
    static let restoreShortcut = "gonhanh.shortcut.restore"
    static let modernTone = "gonhanh.modernTone"
    static let englishAutoRestore = "gonhanh.englishAutoRestore"
    static let autoCapitalize = "gonhanh.autoCapitalize"
    static let autoCapitalizeExcludedApps = "gonhanh.autoCapitalize.excludedApps"
    static let launchAtLoginUserDisabled = "gonhanh.launchAtLogin.userDisabled"
    static let soundEnabled = "gonhanh.soundEnabled"
    static let allowForeignConsonants = "gonhanh.allowForeignConsonants"
}

// MARK: - Keyboard Shortcut Model

struct KeyboardShortcut: Codable, Equatable {
    var keyCode: UInt16
    var modifiers: UInt64  // CGEventFlags raw value

    // Default: Ctrl+Space (for toggle)
    static let `default` = KeyboardShortcut(keyCode: 0x31, modifiers: CGEventFlags.maskControl.rawValue)

    // Default: ESC (for restore diacritics)
    static let defaultRestore = KeyboardShortcut(keyCode: 0x35, modifiers: 0)

    var displayParts: [String] {
        var parts: [String] = []
        let flags = CGEventFlags(rawValue: modifiers)
        if flags.contains(.maskSecondaryFn) { parts.append("fn") }
        if flags.contains(.maskControl) { parts.append("⌃") }
        if flags.contains(.maskAlternate) { parts.append("⌥") }
        if flags.contains(.maskShift) { parts.append("⇧") }
        if flags.contains(.maskCommand) { parts.append("⌘") }
        let keyStr = keyCodeToString(keyCode)
        if !keyStr.isEmpty { parts.append(keyStr) }  // Skip for modifier-only shortcuts
        return parts
    }

    private static let specialKeyNames: [UInt16: String] = [
        0xFFFF: "",     // Modifier-only shortcut
        0x31: "Space",
        0x24: "↩",      // Return
        0x4C: "⌅",      // Numpad Enter
        0x30: "⇥",      // Tab
        0x33: "⌫",      // Delete/Backspace
        0x75: "⌦",      // Forward Delete
        0x35: "⎋",      // Escape
        0x39: "⇪",      // CapsLock
        0x47: "⌧",      // Clear (Numpad)
        0x72: "Help",
        0x73: "↖", 0x77: "↘",  // Home, End
        0x74: "⇞", 0x79: "⇟",  // Page Up, Page Down
        0x7B: "←", 0x7C: "→", 0x7D: "↓", 0x7E: "↑",  // Arrow keys
        0x7A: "F1", 0x78: "F2", 0x63: "F3", 0x76: "F4",
        0x60: "F5", 0x61: "F6", 0x62: "F7", 0x64: "F8",
        0x65: "F9", 0x6D: "F10", 0x67: "F11", 0x6F: "F12",
        0x69: "F13", 0x6B: "F14", 0x71: "F15", 0x6A: "F16",
        0x40: "F17", 0x4F: "F18", 0x50: "F19", 0x5A: "F20",
    ]

    private func keyCodeToString(_ code: UInt16) -> String {
        if let name = Self.specialKeyNames[code] {
            return name
        }
        // Use CGEvent to dynamically get character for regular keys
        if let event = CGEvent(keyboardEventSource: nil, virtualKey: CGKeyCode(code), keyDown: true) {
            var length = 0
            var chars = [UniChar](repeating: 0, count: 4)
            event.keyboardGetUnicodeString(maxStringLength: 4, actualStringLength: &length, unicodeString: &chars)
            if length > 0 {
                return String(utf16CodeUnits: chars, count: length).uppercased()
            }
        }
        return String(format: "0x%02X", code)
    }

    static func load() -> KeyboardShortcut {
        guard let data = UserDefaults.standard.data(forKey: SettingsKey.toggleShortcut),
              let shortcut = try? JSONDecoder().decode(KeyboardShortcut.self, from: data) else {
            return .default
        }
        return shortcut
    }

    func save() {
        if let data = try? JSONEncoder().encode(self) {
            UserDefaults.standard.set(data, forKey: SettingsKey.toggleShortcut)
        }
    }

    static func loadRestoreShortcut() -> KeyboardShortcut {
        guard let data = UserDefaults.standard.data(forKey: SettingsKey.restoreShortcut),
              let shortcut = try? JSONDecoder().decode(KeyboardShortcut.self, from: data) else {
            return .defaultRestore
        }
        return shortcut
    }

    func saveAsRestoreShortcut() {
        if let data = try? JSONEncoder().encode(self) {
            UserDefaults.standard.set(data, forKey: SettingsKey.restoreShortcut)
        }
    }

    /// Check if this shortcut is modifier-only (no character key)
    var isModifierOnly: Bool { keyCode == 0xFFFF }

    /// Modifier mask for matching shortcuts (includes fn key)
    private static let modifierMask: CGEventFlags = [.maskSecondaryFn, .maskControl, .maskAlternate, .maskShift, .maskCommand]

    /// Check if given key event matches this shortcut
    /// - Parameters:
    ///   - keyCode: The key code from the event
    ///   - flags: The modifier flags from the event
    /// - Returns: true if exact match (same key + exact same modifiers, no extras)
    func matches(keyCode pressedKeyCode: UInt16, flags: CGEventFlags) -> Bool {
        // For key+modifier shortcuts: check keyCode and exact modifier match
        guard !isModifierOnly else { return false }
        guard pressedKeyCode == keyCode else { return false }
        let savedFlags = CGEventFlags(rawValue: modifiers)
        // Exact match: only the saved modifiers should be pressed, no extras
        return flags.intersection(Self.modifierMask) == savedFlags.intersection(Self.modifierMask)
    }

    /// Check if given modifier flags match this modifier-only shortcut
    /// - Parameter flags: The modifier flags from the event
    /// - Returns: true if exact match (same modifiers, no extras)
    func matchesModifierOnly(flags: CGEventFlags) -> Bool {
        guard isModifierOnly else { return false }
        let savedFlags = CGEventFlags(rawValue: modifiers)
        return flags.intersection(Self.modifierMask) == savedFlags.intersection(Self.modifierMask)
    }
}

// MARK: - Input Mode

enum InputMode: Int, CaseIterable {
    case telex = 0
    case vni = 1

    var name: String {
        switch self {
        case .telex: return "Telex"
        case .vni: return "VNI"
        }
    }

    var shortName: String {
        switch self {
        case .telex: return "T"
        case .vni: return "V"
        }
    }

    var description: String {
        switch self {
        case .telex: return "aw, ow, w, s, f, r, x, j"
        case .vni: return "a8, o9, 1-5"
        }
    }

    var fullDescription: String {
        switch self {
        case .telex: return "Telex (\(description))"
        case .vni: return "VNI (\(description))"
        }
    }
}
