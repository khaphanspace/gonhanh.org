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

    private func keyCodeToString(_ code: UInt16) -> String {
        switch code {
        case 0x31: return "Space"
        case 0x24: return "↩"
        case 0x30: return "⇥"
        case 0x33: return "⌫"
        case 0x35: return "⎋"
        case 0x7B: return "←"
        case 0x7C: return "→"
        case 0x7D: return "↓"
        case 0x7E: return "↑"
        case 0x00: return "A"
        case 0x01: return "S"
        case 0x02: return "D"
        case 0x03: return "F"
        case 0x04: return "H"
        case 0x05: return "G"
        case 0x06: return "Z"
        case 0x07: return "X"
        case 0x08: return "C"
        case 0x09: return "V"
        case 0x0B: return "B"
        case 0x0C: return "Q"
        case 0x0D: return "W"
        case 0x0E: return "E"
        case 0x0F: return "R"
        case 0x10: return "Y"
        case 0x11: return "T"
        case 0x12: return "1"
        case 0x13: return "2"
        case 0x14: return "3"
        case 0x15: return "4"
        case 0x16: return "6"
        case 0x17: return "5"
        case 0x18: return "="
        case 0x19: return "9"
        case 0x1A: return "7"
        case 0x1B: return "-"
        case 0x1C: return "8"
        case 0x1D: return "0"
        case 0x1E: return "]"
        case 0x1F: return "O"
        case 0x20: return "U"
        case 0x21: return "["
        case 0x22: return "I"
        case 0x23: return "P"
        case 0x25: return "L"
        case 0x26: return "J"
        case 0x27: return "'"
        case 0x28: return "K"
        case 0x29: return ";"
        case 0x2A: return "\\"
        case 0x2B: return ","
        case 0x2C: return "/"
        case 0x2D: return "N"
        case 0x2E: return "M"
        case 0x2F: return "."
        case 0x32: return "`"
        case 0xFFFF: return ""  // Modifier-only shortcut (no key)
        default: return "?"
        }
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
