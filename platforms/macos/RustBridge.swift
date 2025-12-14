import Foundation
import Carbon
import AppKit

// MARK: - Debug Logging

/// Debug logging - only active when /tmp/gonhanh_debug.log exists
/// Enable: touch /tmp/gonhanh_debug.log | Disable: rm /tmp/gonhanh_debug.log
private enum Log {
    private static let logPath = "/tmp/gonhanh_debug.log"
    private static var isEnabled: Bool { FileManager.default.fileExists(atPath: logPath) }

    private static func write(_ msg: String) {
        guard isEnabled, let handle = FileHandle(forWritingAtPath: logPath) else { return }
        let ts = String(format: "%02d:%02d:%02d.%03d",
                        Calendar.current.component(.hour, from: Date()),
                        Calendar.current.component(.minute, from: Date()),
                        Calendar.current.component(.second, from: Date()),
                        Calendar.current.component(.nanosecond, from: Date()) / 1_000_000)
        handle.seekToEndOfFile()
        handle.write("[\(ts)] \(msg)\n".data(using: .utf8)!)
        handle.closeFile()
    }

    static func key(_ code: UInt16, _ result: String) { write("K:\(code) → \(result)") }
    static func transform(_ bs: Int, _ chars: String) { write("T: ←\(bs) \"\(chars)\"") }
    static func send(_ method: String, _ bs: Int, _ chars: String) { write("S:\(method) ←\(bs) \"\(chars)\"") }
    static func method(_ name: String) { write("M: \(name)") }
    static func info(_ msg: String) { write("I: \(msg)") }
    static func skip() { write("K: skip (self)") }
    static func queue(_ msg: String) { write("Q: \(msg)") }
}

// MARK: - Method Detection

private enum Method { case fast, slow, selection }

// MARK: - Text Injector (Async with Serial Queue)

/// Handles text injection with proper sequencing to prevent race conditions
private class TextInjector {
    static let shared = TextInjector()

    /// Serial queue ensures text injections happen one at a time
    private let queue = DispatchQueue(label: "org.gonhanh.textinjector", qos: .userInteractive)

    /// Semaphore to block keyboard callback until injection completes
    private let completionSemaphore = DispatchSemaphore(value: 1)

    /// Track if currently processing to handle rapid keystrokes
    private var isProcessing = false

    private init() {}

    /// Inject text replacement synchronously (blocks until complete)
    /// This ensures the next keystroke waits for current injection to finish
    func injectSync(backspace bs: Int, text: String, method: Method, delays: (UInt32, UInt32, UInt32)) {
        // Wait for any previous injection to complete
        completionSemaphore.wait()
        defer { completionSemaphore.signal() }

        isProcessing = true
        defer { isProcessing = false }

        Log.queue("start bs=\(bs) text=\"\(text)\"")

        switch method {
        case .selection:
            performSelectionInjection(bs: bs, text: text)
        case .slow, .fast:
            performBackspaceInjection(bs: bs, text: text, delays: delays)
        }

        // Additional settle time for slow apps
        let settleTime: UInt32 = method == .slow ? 20000 : 5000  // 20ms or 5ms
        usleep(settleTime)

        Log.queue("done")
    }

    private func performBackspaceInjection(bs: Int, text: String, delays: (UInt32, UInt32, UInt32)) {
        guard let src = CGEventSource(stateID: .privateState) else { return }

        // Send backspaces
        for _ in 0..<bs {
            guard let dn = CGEvent(keyboardEventSource: src, virtualKey: 0x33, keyDown: true),
                  let up = CGEvent(keyboardEventSource: src, virtualKey: 0x33, keyDown: false) else { continue }
            dn.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
            up.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
            dn.post(tap: .cgSessionEventTap)
            up.post(tap: .cgSessionEventTap)
            usleep(delays.0)
        }

        // Wait after backspaces
        if bs > 0 { usleep(delays.1) }

        // Send text in chunks (CGEvent has 20-char limit for keyboardSetUnicodeString)
        let utf16 = Array(text.utf16)
        let chunkSize = 20
        var offset = 0

        while offset < utf16.count {
            let end = min(offset + chunkSize, utf16.count)
            let chunk = Array(utf16[offset..<end])

            guard let dn = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: true),
                  let up = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: false) else { break }
            dn.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
            up.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
            dn.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: chunk)
            up.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: chunk)
            dn.post(tap: .cgSessionEventTap)
            up.post(tap: .cgSessionEventTap)
            usleep(delays.2)

            offset = end
        }

        Log.send("bs", bs, text)
    }

    private func performSelectionInjection(bs: Int, text: String) {
        guard let src = CGEventSource(stateID: .privateState) else { return }

        // Select characters with Shift+Left
        for _ in 0..<bs {
            guard let dn = CGEvent(keyboardEventSource: src, virtualKey: 0x7B, keyDown: true),
                  let up = CGEvent(keyboardEventSource: src, virtualKey: 0x7B, keyDown: false) else { continue }
            dn.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
            up.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
            dn.flags = .maskShift
            up.flags = .maskShift
            dn.post(tap: .cgSessionEventTap)
            up.post(tap: .cgSessionEventTap)
            usleep(1000)  // 1ms between selections
        }

        // Small delay after selection
        if bs > 0 { usleep(3000) }

        // Type replacement text in chunks (CGEvent has 20-char limit)
        let utf16 = Array(text.utf16)
        let chunkSize = 20
        var offset = 0

        while offset < utf16.count {
            let end = min(offset + chunkSize, utf16.count)
            let chunk = Array(utf16[offset..<end])

            guard let dn = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: true),
                  let up = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: false) else { break }
            dn.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
            up.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
            dn.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: chunk)
            up.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: chunk)
            dn.post(tap: .cgSessionEventTap)
            up.post(tap: .cgSessionEventTap)
            usleep(2000)  // 2ms after each chunk

            offset = end
        }

        Log.send("sel", bs, text)
    }
}

// MARK: - FFI (Rust Bridge)

/// FFI result struct - must match Rust `Result` struct layout exactly
/// Size: 64 UInt32 chars (256 bytes) + 4 bytes = 260 bytes
/// Max replacement: 63 UTF-32 codepoints (Vietnamese diacritics = 1 each)
private struct ImeResult {
    // 64 UInt32 values for UTF-32 codepoints (matches core/src/engine/buffer.rs MAX)
    var chars: (
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32
    )
    var action: UInt8
    var backspace: UInt8
    var count: UInt8
    var _pad: UInt8
}

@_silgen_name("ime_init") private func ime_init()
@_silgen_name("ime_key_ext") private func ime_key_ext(_ key: UInt16, _ caps: Bool, _ ctrl: Bool, _ shift: Bool) -> UnsafeMutablePointer<ImeResult>?
@_silgen_name("ime_method") private func ime_method(_ method: UInt8)
@_silgen_name("ime_enabled") private func ime_enabled(_ enabled: Bool)
@_silgen_name("ime_clear") private func ime_clear()
@_silgen_name("ime_free") private func ime_free(_ result: UnsafeMutablePointer<ImeResult>?)

// Shortcut FFI
@_silgen_name("ime_add_shortcut") private func ime_add_shortcut(_ trigger: UnsafePointer<CChar>?, _ replacement: UnsafePointer<CChar>?)
@_silgen_name("ime_remove_shortcut") private func ime_remove_shortcut(_ trigger: UnsafePointer<CChar>?)
@_silgen_name("ime_clear_shortcuts") private func ime_clear_shortcuts()

// MARK: - RustBridge (Public API)

class RustBridge {
    private static var isInitialized = false

    static func initialize() {
        guard !isInitialized else { return }
        ime_init()
        isInitialized = true
        Log.info("Engine initialized")
    }

    static func processKey(keyCode: UInt16, caps: Bool, ctrl: Bool, shift: Bool = false) -> (Int, [Character])? {
        guard isInitialized, let ptr = ime_key_ext(keyCode, caps, ctrl, shift) else { return nil }
        defer { ime_free(ptr) }

        let r = ptr.pointee
        guard r.action == 1 else { return nil }

        let chars = withUnsafePointer(to: r.chars) { p in
            p.withMemoryRebound(to: UInt32.self, capacity: 64) { bound in
                (0..<Int(r.count)).compactMap { Unicode.Scalar(bound[$0]).map(Character.init) }
            }
        }
        return (Int(r.backspace), chars)
    }

    static func setMethod(_ method: Int) {
        ime_method(UInt8(method))
        Log.info("Method: \(method == 0 ? "Telex" : "VNI")")
    }

    static func setEnabled(_ enabled: Bool) {
        ime_enabled(enabled)
        Log.info("Enabled: \(enabled)")
    }

    static func clearBuffer() { ime_clear() }

    // MARK: - Shortcuts

    /// Add a shortcut to the engine
    static func addShortcut(trigger: String, replacement: String) {
        trigger.withCString { t in
            replacement.withCString { r in
                ime_add_shortcut(t, r)
            }
        }
        Log.info("Shortcut added: \(trigger) → \(replacement)")
    }

    /// Remove a shortcut from the engine
    static func removeShortcut(trigger: String) {
        trigger.withCString { t in
            ime_remove_shortcut(t)
        }
        Log.info("Shortcut removed: \(trigger)")
    }

    /// Clear all shortcuts from the engine
    static func clearShortcuts() {
        ime_clear_shortcuts()
        Log.info("Shortcuts cleared")
    }

    /// Sync shortcuts from UI to engine
    static func syncShortcuts(_ shortcuts: [(key: String, value: String, enabled: Bool)]) {
        ime_clear_shortcuts()
        for shortcut in shortcuts where shortcut.enabled {
            addShortcut(trigger: shortcut.key, replacement: shortcut.value)
        }
        Log.info("Synced \(shortcuts.filter { $0.enabled }.count) shortcuts")
    }
}

// MARK: - Keyboard Hook Manager

class KeyboardHookManager {
    static let shared = KeyboardHookManager()

    private var eventTap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    private var isRunning = false

    private init() {}

    func start() {
        guard !isRunning else { return }

        guard AXIsProcessTrusted() else {
            let opts = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
            AXIsProcessTrustedWithOptions(opts)
            Log.info("Requesting accessibility permission")
            return
        }

        RustBridge.initialize()

        let mask: CGEventMask = (1 << CGEventType.keyDown.rawValue) | (1 << CGEventType.flagsChanged.rawValue)
        let tap = CGEvent.tapCreate(tap: .cghidEventTap, place: .headInsertEventTap,
                                    options: .defaultTap, eventsOfInterest: mask,
                                    callback: keyboardCallback, userInfo: nil)
            ?? CGEvent.tapCreate(tap: .cgSessionEventTap, place: .headInsertEventTap,
                                 options: .defaultTap, eventsOfInterest: mask,
                                 callback: keyboardCallback, userInfo: nil)

        guard let tap = tap else {
            showAccessibilityAlert()
            return
        }

        eventTap = tap
        runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)
        if let source = runLoopSource {
            CFRunLoopAddSource(CFRunLoopGetCurrent(), source, .commonModes)
            CGEvent.tapEnable(tap: tap, enable: true)
            isRunning = true
            setupShortcutObserver()
            Log.info("Hook started")
        }
    }

    func stop() {
        guard isRunning else { return }
        if let tap = eventTap { CGEvent.tapEnable(tap: tap, enable: false) }
        if let src = runLoopSource { CFRunLoopRemoveSource(CFRunLoopGetCurrent(), src, .commonModes) }
        eventTap = nil
        runLoopSource = nil
        isRunning = false
        Log.info("Hook stopped")
    }

    func getTap() -> CFMachPort? { eventTap }

    private func showAccessibilityAlert() {
        DispatchQueue.main.async {
            let alert = NSAlert()
            alert.messageText = "Cần quyền Accessibility"
            alert.informativeText = "Gõ Nhanh cần quyền Accessibility để gõ tiếng Việt.\n\n1. Mở System Settings > Privacy & Security > Accessibility\n2. Bật Gõ Nhanh\n3. Khởi động lại app"
            alert.alertStyle = .warning
            alert.addButton(withTitle: "Mở System Settings")
            alert.addButton(withTitle: "Hủy")
            if alert.runModal() == .alertFirstButtonReturn {
                NSWorkspace.shared.open(URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!)
            }
        }
    }
}

// MARK: - Keyboard Callback

private let kEventMarker: Int64 = 0x474E4820  // "GNH "
private var wasCtrlShiftPressed = false  // Track Ctrl+Shift state for toggle detection
private var currentShortcut = KeyboardShortcut.load()  // Load saved shortcut

// Observer for shortcut changes
private var shortcutObserver: NSObjectProtocol?

func setupShortcutObserver() {
    shortcutObserver = NotificationCenter.default.addObserver(
        forName: .shortcutChanged,
        object: nil,
        queue: .main
    ) { _ in
        currentShortcut = KeyboardShortcut.load()
        Log.info("Shortcut updated: \(currentShortcut.displayParts.joined())")
    }
}

private func matchesToggleShortcut(keyCode: UInt16, flags: CGEventFlags) -> Bool {
    guard keyCode == currentShortcut.keyCode else { return false }

    let savedFlags = CGEventFlags(rawValue: currentShortcut.modifiers)

    // Check required modifiers are present
    if savedFlags.contains(.maskControl) && !flags.contains(.maskControl) { return false }
    if savedFlags.contains(.maskAlternate) && !flags.contains(.maskAlternate) { return false }
    if savedFlags.contains(.maskShift) && !flags.contains(.maskShift) { return false }
    if savedFlags.contains(.maskCommand) && !flags.contains(.maskCommand) { return false }

    // Ensure Command is NOT pressed if not required (avoid conflict with system shortcuts)
    if !savedFlags.contains(.maskCommand) && flags.contains(.maskCommand) { return false }

    return true
}

private func keyboardCallback(
    proxy: CGEventTapProxy, type: CGEventType, event: CGEvent, refcon: UnsafeMutableRawPointer?
) -> Unmanaged<CGEvent>? {

    if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
        if let tap = KeyboardHookManager.shared.getTap() { CGEvent.tapEnable(tap: tap, enable: true) }
        return Unmanaged.passUnretained(event)
    }

    let flags = event.flags

    // Handle Ctrl+Shift toggle (modifier-only shortcut)
    if type == .flagsChanged {
        let isCtrlShift = flags.contains(.maskControl) && flags.contains(.maskShift) && !flags.contains(.maskCommand)
        if isCtrlShift {
            wasCtrlShiftPressed = true
        } else if wasCtrlShiftPressed {
            // Ctrl+Shift was pressed and now one is released - toggle
            wasCtrlShiftPressed = false
            DispatchQueue.main.async { NotificationCenter.default.post(name: .toggleVietnamese, object: nil) }
        }
        return Unmanaged.passUnretained(event)
    }

    guard type == .keyDown else { return Unmanaged.passUnretained(event) }

    // Reset Ctrl+Shift state if any key is pressed while modifiers are held
    wasCtrlShiftPressed = false

    if event.getIntegerValueField(.eventSourceUserData) == kEventMarker {
        Log.skip()
        return Unmanaged.passUnretained(event)
    }

    let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))

    // Custom shortcut to toggle Vietnamese (default: Ctrl+Space)
    if matchesToggleShortcut(keyCode: keyCode, flags: flags) {
        DispatchQueue.main.async { NotificationCenter.default.post(name: .toggleVietnamese, object: nil) }
        return nil
    }

    let shift = flags.contains(.maskShift)
    let caps = shift || flags.contains(.maskAlphaShift)
    let ctrl = flags.contains(.maskCommand) || flags.contains(.maskControl) || flags.contains(.maskAlternate)

    if let (bs, chars) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
        let str = String(chars)
        Log.transform(bs, str)
        sendReplacement(backspace: bs, chars: chars)
        return nil
    }

    Log.key(keyCode, "pass")
    return Unmanaged.passUnretained(event)
}

// MARK: - Text Replacement

private func detectMethod() -> (Method, (UInt32, UInt32, UInt32)) {
    guard let app = NSWorkspace.shared.frontmostApplication,
          let bundleId = app.bundleIdentifier else { return (.fast, (200, 800, 500)) }

    // Selection method for autocomplete contexts
    let systemWide = AXUIElementCreateSystemWide()
    var focused: CFTypeRef?
    var role: String?

    if AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focused) == .success,
       let el = focused {
        var roleVal: CFTypeRef?
        AXUIElementCopyAttributeValue(el as! AXUIElement, kAXRoleAttribute as CFString, &roleVal)
        role = roleVal as? String
    }

    if role == "AXComboBox" { Log.method("sel:combo"); return (.selection, (0, 0, 0)) }

    let browsers = ["com.google.Chrome", "com.apple.Safari", "company.thebrowser.Browser"]
    if browsers.contains(bundleId) && role == "AXTextField" { Log.method("sel:browser"); return (.selection, (0, 0, 0)) }
    if role == "AXTextField" && bundleId.hasPrefix("com.jetbrains") { Log.method("sel:jb"); return (.selection, (0, 0, 0)) }
    if bundleId == "com.microsoft.Excel" { Log.method("sel:excel"); return (.selection, (0, 0, 0)) }
    if bundleId == "com.microsoft.Word" { Log.method("sel:word"); return (.selection, (0, 0, 0)) }

    // Electron apps (Claude Code) - higher delays
    if bundleId == "com.todesktop.230313mzl4w4u92" { Log.method("slow:claude"); return (.slow, (8000, 15000, 8000)) }

    // Warp terminal - needs higher delays
    if bundleId == "dev.warp.Warp-Stable" { Log.method("slow:warp"); return (.slow, (3000, 8000, 3000)) }

    // Terminal apps - medium delays
    let terminals = ["com.microsoft.VSCode", "com.apple.Terminal",
                     "com.googlecode.iterm2", "io.alacritty", "com.github.wez.wezterm",
                     "com.google.antigravity"]
    if terminals.contains(bundleId) { Log.method("slow:term"); return (.slow, (1500, 3000, 2000)) }

    Log.method("fast")
    return (.fast, (200, 800, 500))
}

private func sendReplacement(backspace bs: Int, chars: [Character]) {
    let (method, delays) = detectMethod()
    let str = String(chars)

    // Use TextInjector for synchronized text injection
    TextInjector.shared.injectSync(backspace: bs, text: str, method: method, delays: delays)
}

// MARK: - Excluded Apps Manager

class ExcludedAppsManager {
    static let shared = ExcludedAppsManager()

    private var excludedBundleIds: Set<String> = []
    private var appObserver: NSObjectProtocol?
    private var wasEnabledBeforeExclusion = true

    private init() {}

    /// Start observing frontmost app changes
    func start() {
        appObserver = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            self?.handleAppChange(notification)
        }
        Log.info("ExcludedAppsManager started")
    }

    /// Stop observing
    func stop() {
        if let observer = appObserver {
            NSWorkspace.shared.notificationCenter.removeObserver(observer)
            appObserver = nil
        }
    }

    /// Update the list of excluded bundle IDs
    func setExcludedApps(_ bundleIds: [String]) {
        excludedBundleIds = Set(bundleIds)
        Log.info("Excluded apps updated: \(bundleIds.count) apps")
        // Re-check current app
        checkCurrentApp()
    }

    /// Check if current frontmost app should be excluded
    private func checkCurrentApp() {
        guard let app = NSWorkspace.shared.frontmostApplication,
              let bundleId = app.bundleIdentifier else { return }
        handleBundleId(bundleId)
    }

    private func handleAppChange(_ notification: Notification) {
        guard let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication,
              let bundleId = app.bundleIdentifier else { return }
        handleBundleId(bundleId)
    }

    private func handleBundleId(_ bundleId: String) {
        let isExcluded = excludedBundleIds.contains(bundleId)
        if isExcluded {
            // Store current enabled state before disabling
            wasEnabledBeforeExclusion = MenuState.shared.isEnabled
            if MenuState.shared.isEnabled {
                RustBridge.setEnabled(false)
                Log.info("App excluded: \(bundleId) - IME disabled")
            }
        } else {
            // Restore previous state when switching to non-excluded app
            if wasEnabledBeforeExclusion {
                RustBridge.setEnabled(MenuState.shared.isEnabled)
            }
        }
    }
}

// MARK: - Notifications

extension Notification.Name {
    static let toggleVietnamese = Notification.Name("toggleVietnamese")
    static let showUpdateWindow = Notification.Name("showUpdateWindow")
    static let shortcutChanged = Notification.Name("shortcutChanged")
    static let excludedAppsChanged = Notification.Name("excludedAppsChanged")
}
