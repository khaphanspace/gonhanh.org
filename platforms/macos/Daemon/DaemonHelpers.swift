import Foundation
import Carbon.HIToolbox
import AppKit
import ApplicationServices

// MARK: - Keyboard Shortcut

struct KeyboardShortcut: Codable, Equatable {
    var keyCode: UInt16
    var modifiers: UInt64

    static let `default` = KeyboardShortcut(keyCode: 0x31, modifiers: CGEventFlags.maskControl.rawValue)

    var displayParts: [String] {
        var parts: [String] = []
        let flags = CGEventFlags(rawValue: modifiers)
        if flags.contains(.maskSecondaryFn) { parts.append("fn") }
        if flags.contains(.maskControl) { parts.append("⌃") }
        if flags.contains(.maskAlternate) { parts.append("⌥") }
        if flags.contains(.maskShift) { parts.append("⇧") }
        if flags.contains(.maskCommand) { parts.append("⌘") }
        let keyStr = keyCodeToString(keyCode)
        if !keyStr.isEmpty { parts.append(keyStr) }
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
        case 0xFFFF: return ""
        default: return String(format: "%02X", code)
        }
    }

    static func load() -> KeyboardShortcut {
        guard let data = UserDefaults.standard.data(forKey: IMESettingsKey.toggleShortcut),
              let shortcut = try? JSONDecoder().decode(KeyboardShortcut.self, from: data) else {
            return .default
        }
        return shortcut
    }

    func save() {
        if let data = try? JSONEncoder().encode(self) {
            UserDefaults.standard.set(data, forKey: IMESettingsKey.toggleShortcut)
        }
    }

    var isModifierOnly: Bool { keyCode == 0xFFFF }

    private static let modifierMask: CGEventFlags = [.maskSecondaryFn, .maskControl, .maskAlternate, .maskShift, .maskCommand]

    func matches(keyCode pressedKeyCode: UInt16, flags: CGEventFlags) -> Bool {
        guard !isModifierOnly else { return false }
        guard pressedKeyCode == keyCode else { return false }
        let savedFlags = CGEventFlags(rawValue: modifiers)
        return flags.intersection(Self.modifierMask) == savedFlags.intersection(Self.modifierMask)
    }

    func matchesModifierOnly(flags: CGEventFlags) -> Bool {
        guard isModifierOnly else { return false }
        let savedFlags = CGEventFlags(rawValue: modifiers)
        return flags.intersection(Self.modifierMask) == savedFlags.intersection(Self.modifierMask)
    }
}

// MARK: - Input Source Observer

private let kAppleKeyLayoutPrefix = "com.apple.keylayout."

private let allowedInputSources: Set<String> = Set([
    "ABC", "ABC-AZERTY", "ABC-India", "ABC-QWERTZ",
    "US", "USExtended", "USInternational-PC",
    "British", "British-PC", "Australian", "Irish", "IrishExtended",
    "Canadian", "Canadian-CSA", "CanadianFrench-PC",
    "Colemak", "Dvorak", "Dvorak-Left", "Dvorak-Right", "DVORAK-QWERTYCMD",
].map { kAppleKeyLayoutPrefix + $0 })

final class InputSourceObserver {
    static let shared = InputSourceObserver()

    private var isObserving = false
    private var lastInputSourceId: String?

    private(set) var currentDisplayChar: String = "V"
    private(set) var isAllowedInputSource: Bool = true

    private init() {}

    func start() {
        guard !isObserving else { return }
        isObserving = true

        CFNotificationCenterAddObserver(
            CFNotificationCenterGetDistributedCenter(),
            Unmanaged.passUnretained(self).toOpaque(),
            inputSourceCallback,
            kTISNotifySelectedKeyboardInputSourceChanged,
            nil,
            .deliverImmediately
        )
        handleChange()
    }

    func stop() {
        guard isObserving else { return }
        isObserving = false
        CFNotificationCenterRemoveObserver(
            CFNotificationCenterGetDistributedCenter(),
            Unmanaged.passUnretained(self).toOpaque(),
            CFNotificationName(kTISNotifySelectedKeyboardInputSourceChanged),
            nil
        )
    }

    fileprivate func handleChange() {
        guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
              let idPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) else { return }

        let currentId = Unmanaged<CFString>.fromOpaque(idPtr).takeUnretainedValue() as String
        guard currentId != lastInputSourceId else { return }
        lastInputSourceId = currentId

        currentDisplayChar = getDisplayChar(from: source, id: currentId)
        isAllowedInputSource = allowedInputSources.contains(currentId)

        if isAllowedInputSource {
            let userEnabled = UserDefaults.standard.bool(forKey: IMESettingsKey.enabled)
            RustBridge.setEnabled(userEnabled)
        } else {
            RustBridge.setEnabled(false)
        }

        NotificationCenter.default.post(
            name: .inputSourceChanged,
            object: nil,
            userInfo: ["allowed": isAllowedInputSource, "displayChar": currentDisplayChar]
        )
    }

    private func getDisplayChar(from source: TISInputSource, id: String) -> String {
        if let langsPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceLanguages),
           let langs = Unmanaged<CFArray>.fromOpaque(langsPtr).takeUnretainedValue() as? [String],
           let lang = langs.first {
            switch lang {
            case "ja": return "あ"
            case "zh-Hans", "zh-Hant", "zh": return "中"
            case "ko": return "한"
            case "th": return "ก"
            case "km": return "ក"
            case "lo": return "ກ"
            case "my": return "က"
            case "hi", "mr", "ne", "sa": return "अ"
            case "bn": return "অ"
            case "ta": return "அ"
            case "vi": return "E"
            case "ru": return "Р"
            case "ar": return "ع"
            case "he": return "א"
            case "el": return "Ω"
            case "fa", "ur": return "ف"
            default: break
            }
        }

        if let namePtr = TISGetInputSourceProperty(source, kTISPropertyLocalizedName) {
            let name = Unmanaged<CFString>.fromOpaque(namePtr).takeUnretainedValue() as String
            if let first = name.first { return String(first).uppercased() }
        }
        return "E"
    }
}

private let inputSourceCallback: CFNotificationCallback = { _, observer, _, _, _ in
    guard let observer = observer else { return }
    let instance = Unmanaged<InputSourceObserver>.fromOpaque(observer).takeUnretainedValue()
    DispatchQueue.main.async { instance.handleChange() }
}

// MARK: - Special Panel App Detector

class SpecialPanelAppDetector {
    static let specialPanelApps: [String] = [
        "com.apple.Spotlight",
        "com.raycast.macos",
        "com.apple.inputmethod.EmojiFunctionRowItem"
    ]

    private static var lastFrontMostApp: String = ""

    static func isSpecialPanelApp(_ bundleId: String?) -> Bool {
        guard let bundleId = bundleId else { return false }
        return specialPanelApps.contains { bundleId.hasPrefix($0) || bundleId == $0 }
    }

    static func getActiveSpecialPanelApp() -> String? {
        if let windowList = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] {
            for window in windowList {
                guard let ownerPID = window[kCGWindowOwnerPID as String] as? pid_t,
                      let windowLayer = window[kCGWindowLayer as String] as? Int else { continue }
                if windowLayer > 0 {
                    if let app = NSRunningApplication(processIdentifier: ownerPID),
                       let bundleId = app.bundleIdentifier,
                       isSpecialPanelApp(bundleId) { return bundleId }
                }
            }
        }

        let systemWide = AXUIElementCreateSystemWide()
        var focusedElement: CFTypeRef?
        if AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focusedElement) == .success,
           let element = focusedElement {
            var pid: pid_t = 0
            if AXUIElementGetPid(element as! AXUIElement, &pid) == .success, pid > 0 {
                if let app = NSRunningApplication(processIdentifier: pid),
                   let bundleId = app.bundleIdentifier,
                   isSpecialPanelApp(bundleId) { return bundleId }
            }
        }

        for panelAppId in specialPanelApps {
            let runningApps = NSRunningApplication.runningApplications(withBundleIdentifier: panelAppId)
            for app in runningApps {
                let appElement = AXUIElementCreateApplication(app.processIdentifier)
                var focusedWindow: CFTypeRef?
                if AXUIElementCopyAttributeValue(appElement, kAXFocusedWindowAttribute as CFString, &focusedWindow) == .success && focusedWindow != nil {
                    return panelAppId
                }
                var windows: CFTypeRef?
                if AXUIElementCopyAttributeValue(appElement, kAXWindowsAttribute as CFString, &windows) == .success,
                   let windowArray = windows as? [AXUIElement], !windowArray.isEmpty {
                    return panelAppId
                }
                if app.isActive { return panelAppId }
            }
        }
        return nil
    }

    static func checkForAppChange() -> (appChanged: Bool, newBundleId: String?, isSpecialPanelApp: Bool) {
        let activePanelApp = getActiveSpecialPanelApp()
        if let panelApp = activePanelApp {
            if panelApp != lastFrontMostApp {
                lastFrontMostApp = panelApp
                return (true, panelApp, true)
            }
            return (false, panelApp, true)
        }

        if isSpecialPanelApp(lastFrontMostApp) {
            let workspaceApp = NSWorkspace.shared.frontmostApplication?.bundleIdentifier
            if let app = workspaceApp {
                lastFrontMostApp = app
                return (true, app, false)
            }
        }
        return (false, nil, false)
    }

    static func updateLastFrontMostApp(_ bundleId: String) { lastFrontMostApp = bundleId }
    static func getLastFrontMostApp() -> String { lastFrontMostApp }
}

// Note: inputSourceChanged notification is defined in DaemonEngine.swift
