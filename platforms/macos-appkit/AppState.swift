// AppState.swift - Pure Foundation (no SwiftUI/Combine)
// Uses NotificationCenter instead of @Published

import Foundation
import Cocoa
import Carbon

/// Central app state manager - pure Foundation, no SwiftUI dependencies
class AppState {
    static let shared = AppState()

    private var isSilentUpdate = false

    // MARK: - State Properties

    var isEnabled: Bool {
        didSet {
            RustBridge.setEnabled(isEnabled)
            guard !isSilentUpdate else { return }
            UserDefaults.standard.set(isEnabled, forKey: SettingsKey.enabled)
            if perAppModeEnabled {
                if let activePanelApp = SpecialPanelAppDetector.getActiveSpecialPanelApp() {
                    savePerAppMode(bundleId: activePanelApp, enabled: isEnabled)
                } else if let bundleId = NSWorkspace.shared.frontmostApplication?.bundleIdentifier {
                    savePerAppMode(bundleId: bundleId, enabled: isEnabled)
                }
            }
            notifyStateChanged()
        }
    }

    var currentMethod: InputMode {
        didSet {
            UserDefaults.standard.set(currentMethod.rawValue, forKey: SettingsKey.method)
            RustBridge.setMethod(currentMethod.rawValue)
            notifyStateChanged()
        }
    }

    var perAppModeEnabled: Bool = true {
        didSet { UserDefaults.standard.set(perAppModeEnabled, forKey: SettingsKey.perAppMode) }
    }

    var autoWShortcut: Bool = true {
        didSet {
            UserDefaults.standard.set(autoWShortcut, forKey: SettingsKey.autoWShortcut)
            RustBridge.setSkipWShortcut(!autoWShortcut)
        }
    }

    var bracketShortcut: Bool = false {
        didSet {
            UserDefaults.standard.set(bracketShortcut, forKey: SettingsKey.bracketShortcut)
            RustBridge.setBracketShortcut(bracketShortcut)
        }
    }

    var escRestore: Bool = false {
        didSet {
            UserDefaults.standard.set(escRestore, forKey: SettingsKey.escRestore)
            RustBridge.setEscRestore(escRestore)
        }
    }

    var modernTone: Bool = true {
        didSet {
            UserDefaults.standard.set(modernTone, forKey: SettingsKey.modernTone)
            RustBridge.setModernTone(modernTone)
        }
    }

    var englishAutoRestore: Bool = false {
        didSet {
            UserDefaults.standard.set(englishAutoRestore, forKey: SettingsKey.englishAutoRestore)
            RustBridge.setEnglishAutoRestore(englishAutoRestore)
        }
    }

    var autoCapitalize: Bool = false {
        didSet {
            UserDefaults.standard.set(autoCapitalize, forKey: SettingsKey.autoCapitalize)
            RustBridge.setAutoCapitalize(autoCapitalize)
        }
    }

    var soundEnabled: Bool = false {
        didSet {
            UserDefaults.standard.set(soundEnabled, forKey: SettingsKey.soundEnabled)
        }
    }

    var toggleShortcut: KeyboardShortcut {
        didSet {
            toggleShortcut.save()
            NotificationCenter.default.post(name: .shortcutChanged, object: toggleShortcut)
        }
    }

    var shortcuts: [ShortcutItem] = []

    // MARK: - Init

    init() {
        let defaults = UserDefaults.standard
        isEnabled = defaults.bool(forKey: SettingsKey.enabled)
        currentMethod = InputMode(rawValue: defaults.integer(forKey: SettingsKey.method)) ?? .telex
        toggleShortcut = KeyboardShortcut.load()
        perAppModeEnabled = defaults.bool(forKey: SettingsKey.perAppMode)
        autoWShortcut = defaults.bool(forKey: SettingsKey.autoWShortcut)
        bracketShortcut = defaults.bool(forKey: SettingsKey.bracketShortcut)
        escRestore = defaults.bool(forKey: SettingsKey.escRestore)
        modernTone = defaults.bool(forKey: SettingsKey.modernTone)
        englishAutoRestore = defaults.bool(forKey: SettingsKey.englishAutoRestore)
        autoCapitalize = defaults.bool(forKey: SettingsKey.autoCapitalize)
        soundEnabled = defaults.bool(forKey: SettingsKey.soundEnabled)

        loadShortcuts()
    }

    // MARK: - Methods

    func toggle() {
        isEnabled.toggle()
    }

    func setMethod(_ method: InputMode) {
        currentMethod = method
    }

    /// Update isEnabled without triggering UI update (used by PerAppModeManager)
    func setEnabledSilently(_ enabled: Bool) {
        isSilentUpdate = true
        isEnabled = enabled
        isSilentUpdate = false
    }

    private func notifyStateChanged() {
        NotificationCenter.default.post(name: .appStateChanged, object: nil)
    }

    // MARK: - Shortcuts

    private func loadShortcuts() {
        if let data = UserDefaults.standard.data(forKey: SettingsKey.shortcuts),
           let saved = try? JSONDecoder().decode([ShortcutItem].self, from: data) {
            shortcuts = saved
        }
    }

    func saveShortcuts() {
        if let data = try? JSONEncoder().encode(shortcuts) {
            UserDefaults.standard.set(data, forKey: SettingsKey.shortcuts)
        }
        syncShortcutsToEngine()
    }

    func syncShortcutsToEngine() {
        RustBridge.clearShortcuts()
        for shortcut in shortcuts where shortcut.enabled {
            RustBridge.addShortcut(trigger: shortcut.key, replacement: shortcut.value)
        }
    }

    // MARK: - Per-App Mode

    func getPerAppMode(bundleId: String) -> Bool {
        let key = "\(SettingsKey.perAppPrefix)\(bundleId)"
        // Default ON, only OFF apps are stored
        if UserDefaults.standard.object(forKey: key) == nil {
            return true
        }
        return UserDefaults.standard.bool(forKey: key)
    }

    func savePerAppMode(bundleId: String, enabled: Bool) {
        let key = "\(SettingsKey.perAppPrefix)\(bundleId)"
        if enabled {
            // Remove key for enabled apps (default = on)
            UserDefaults.standard.removeObject(forKey: key)
        } else {
            UserDefaults.standard.set(false, forKey: key)
        }
    }
}

// MARK: - Settings Keys

enum SettingsKey {
    static let enabled = "enabled"
    static let method = "method"
    static let perAppMode = "perAppMode"
    static let perAppPrefix = "perApp_"
    static let autoWShortcut = "autoWShortcut"
    static let bracketShortcut = "bracketShortcut"
    static let escRestore = "escRestore"
    static let modernTone = "modernTone"
    static let englishAutoRestore = "englishAutoRestore"
    static let autoCapitalize = "autoCapitalize"
    static let soundEnabled = "soundEnabled"
    static let shortcuts = "shortcuts"
    static let hasCompletedOnboarding = "hasCompletedOnboarding"
    static let reopenSettingsAfterUpdate = "reopenSettingsAfterUpdate"
}

// MARK: - Input Mode

enum InputMode: Int {
    case telex = 0
    case vni = 1

    var name: String {
        switch self {
        case .telex: return "Telex"
        case .vni: return "VNI"
        }
    }
}

// MARK: - Shortcut Item

struct ShortcutItem: Codable, Identifiable {
    var id = UUID()
    var key: String
    var value: String
    var enabled: Bool = true
}

// MARK: - Keyboard Shortcut

struct KeyboardShortcut: Codable, Equatable {
    var keyCode: UInt16
    var modifiers: UInt64

    static let defaultShortcut = KeyboardShortcut(keyCode: 0x31, modifiers: CGEventFlags.maskControl.rawValue)

    static func load() -> KeyboardShortcut {
        guard let data = UserDefaults.standard.data(forKey: "toggleShortcut"),
              let shortcut = try? JSONDecoder().decode(KeyboardShortcut.self, from: data) else {
            return defaultShortcut
        }
        return shortcut
    }

    func save() {
        if let data = try? JSONEncoder().encode(self) {
            UserDefaults.standard.set(data, forKey: "toggleShortcut")
        }
    }

    var displayParts: [String] {
        var parts: [String] = []
        let flags = CGEventFlags(rawValue: modifiers)
        if flags.contains(.maskControl) { parts.append("⌃") }
        if flags.contains(.maskAlternate) { parts.append("⌥") }
        if flags.contains(.maskShift) { parts.append("⇧") }
        if flags.contains(.maskCommand) { parts.append("⌘") }
        if flags.contains(.maskSecondaryFn) { parts.append("fn") }

        // Only key-based shortcut has a key
        if keyCode != 0xFFFF {
            parts.append(keyCodeToString(keyCode))
        }
        return parts
    }

    func matches(keyCode: UInt16, flags: CGEventFlags) -> Bool {
        guard self.keyCode != 0xFFFF else { return false }
        return self.keyCode == keyCode && flags.intersection(modifierMask) == CGEventFlags(rawValue: modifiers)
    }

    func matchesModifierOnly(flags: CGEventFlags) -> Bool {
        guard keyCode == 0xFFFF else { return false }
        return flags.intersection(modifierMask) == CGEventFlags(rawValue: modifiers)
    }

    private var modifierMask: CGEventFlags {
        [.maskSecondaryFn, .maskControl, .maskAlternate, .maskShift, .maskCommand]
    }

    private func keyCodeToString(_ code: UInt16) -> String {
        let keyMap: [UInt16: String] = [
            0x31: "Space", 0x24: "↩", 0x30: "⇥", 0x35: "⎋",
            0x00: "A", 0x0B: "B", 0x08: "C", 0x02: "D", 0x0E: "E", 0x03: "F",
            0x05: "G", 0x04: "H", 0x22: "I", 0x26: "J", 0x28: "K", 0x25: "L",
            0x2E: "M", 0x2D: "N", 0x1F: "O", 0x23: "P", 0x0C: "Q", 0x0F: "R",
            0x01: "S", 0x11: "T", 0x20: "U", 0x09: "V", 0x0D: "W", 0x07: "X",
            0x10: "Y", 0x06: "Z",
            0x12: "1", 0x13: "2", 0x14: "3", 0x15: "4", 0x17: "5",
            0x16: "6", 0x1A: "7", 0x1C: "8", 0x19: "9", 0x1D: "0",
        ]
        return keyMap[code] ?? "?"
    }
}

// MARK: - Sound Manager

class SoundManager {
    static let shared = SoundManager()

    private let enableSound: NSSound?
    private let disableSound: NSSound?
    private var currentSound: NSSound?

    private init() {
        enableSound = NSSound(named: NSSound.Name("Tink"))
        disableSound = NSSound(named: NSSound.Name("Pop"))
    }

    func playToggleSound(enabled: Bool) {
        guard AppState.shared.soundEnabled else { return }
        currentSound?.stop()
        let sound = enabled ? enableSound : disableSound
        currentSound = sound
        sound?.play()
    }
}
