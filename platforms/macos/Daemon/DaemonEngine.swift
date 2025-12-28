import Foundation
import Carbon
import AppKit

// MARK: - Daemon State

/// Minimal state management for daemon (no SwiftUI dependency)
final class DaemonState {
    static let shared = DaemonState()

    private(set) var isEnabled: Bool = true
    private(set) var perAppModeEnabled: Bool = true

    private init() {}

    func loadFromDefaults() {
        let defaults = UserDefaults.standard
        isEnabled = defaults.bool(forKey: IMESettingsKey.enabled)
        perAppModeEnabled = defaults.bool(forKey: IMESettingsKey.perAppMode)
    }

    func setEnabled(_ enabled: Bool) {
        isEnabled = enabled
    }

    func setPerAppModeEnabled(_ enabled: Bool) {
        perAppModeEnabled = enabled
    }

    func getPerAppMode(bundleId: String) -> Bool {
        let modes = UserDefaults.standard.dictionary(forKey: IMESettingsKey.perAppModes) as? [String: Bool] ?? [:]
        return modes[bundleId] ?? true
    }

    func savePerAppMode(bundleId: String, enabled: Bool) {
        var modes = UserDefaults.standard.dictionary(forKey: IMESettingsKey.perAppModes) as? [String: Bool] ?? [:]
        if enabled { modes.removeValue(forKey: bundleId) } else { modes[bundleId] = false }
        UserDefaults.standard.set(modes, forKey: IMESettingsKey.perAppModes)
    }
}

// MARK: - Debug Logging

/// Debug logging - only active when log file exists
/// Enable: touch /tmp/gonhanh_debug.log | Disable: rm /tmp/gonhanh_debug.log
enum Log {
    private static let logPath = "/tmp/gonhanh_debug.log"
    static var isEnabled: Bool { FileManager.default.fileExists(atPath: logPath) }

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

// MARK: - Constants

private enum KeyCode {
    static let backspace: CGKeyCode = 0x33
    static let forwardDelete: CGKeyCode = 0x75
    static let leftArrow: CGKeyCode = 0x7B
    static let rightArrow: CGKeyCode = 0x7C
    static let downArrow: CGKeyCode = 0x7D
    static let upArrow: CGKeyCode = 0x7E
    static let space: CGKeyCode = 0x31
    static let tab: CGKeyCode = 0x30
    static let returnKey: CGKeyCode = 0x24
    static let enter: CGKeyCode = 0x4C
    static let esc: CGKeyCode = 0x35
    static let dot: CGKeyCode = 0x2F
    static let comma: CGKeyCode = 0x2B
    static let slash: CGKeyCode = 0x2C
    static let semicolon: CGKeyCode = 0x29
    static let quote: CGKeyCode = 0x27
    static let lbracket: CGKeyCode = 0x21
    static let rbracket: CGKeyCode = 0x1E
    static let backslash: CGKeyCode = 0x2A
    static let minus: CGKeyCode = 0x1B
    static let equal: CGKeyCode = 0x18
    static let backquote: CGKeyCode = 0x32
    static let n0: CGKeyCode = 0x1D
    static let n1: CGKeyCode = 0x12
    static let n2: CGKeyCode = 0x13
    static let n3: CGKeyCode = 0x14
    static let n4: CGKeyCode = 0x15
    static let n5: CGKeyCode = 0x17
    static let n6: CGKeyCode = 0x16
    static let n7: CGKeyCode = 0x1A
    static let n8: CGKeyCode = 0x1C
    static let n9: CGKeyCode = 0x19
}

/// Check if key is a break key
private func isBreakKey(_ keyCode: CGKeyCode, shift: Bool) -> Bool {
    let standardBreak: Set<CGKeyCode> = [
        KeyCode.space, KeyCode.tab, KeyCode.returnKey, KeyCode.enter, KeyCode.esc,
        KeyCode.leftArrow, KeyCode.rightArrow, KeyCode.upArrow, KeyCode.downArrow,
        KeyCode.dot, KeyCode.comma, KeyCode.slash, KeyCode.semicolon, KeyCode.quote,
        KeyCode.lbracket, KeyCode.rbracket, KeyCode.backslash, KeyCode.minus,
        KeyCode.equal, KeyCode.backquote
    ]
    if standardBreak.contains(keyCode) { return true }
    if shift {
        let numberKeys: Set<CGKeyCode> = [
            KeyCode.n0, KeyCode.n1, KeyCode.n2, KeyCode.n3, KeyCode.n4,
            KeyCode.n5, KeyCode.n6, KeyCode.n7, KeyCode.n8, KeyCode.n9
        ]
        return numberKeys.contains(keyCode)
    }
    return false
}

// MARK: - Injection Method

private enum InjectionMethod {
    case fast, slow, selection, autocomplete, selectAll, axDirect, passthrough
}

// MARK: - Text Injector

/// Handles text injection with proper sequencing
class TextInjector {
    static let shared = TextInjector()

    private let semaphore = DispatchSemaphore(value: 1)
    private var sessionBuffer: String = ""

    private init() {}

    func updateSessionBuffer(backspace: Int, newText: String) {
        if backspace > 0 && sessionBuffer.count >= backspace {
            sessionBuffer.removeLast(backspace)
        }
        sessionBuffer.append(newText)
    }

    func clearSessionBuffer() { sessionBuffer = "" }
    func setSessionBuffer(_ text: String) { sessionBuffer = text }
    func getSessionBuffer() -> String { sessionBuffer }

    func injectSelectAllOnly(proxy: CGEventTapProxy) {
        semaphore.wait()
        defer { semaphore.signal() }
        injectViaSelectAll(proxy: proxy)
        usleep(5000)
    }

    func injectSync(bs: Int, text: String, method: InjectionMethod, delays: (UInt32, UInt32, UInt32), proxy: CGEventTapProxy) {
        semaphore.wait()
        defer { semaphore.signal() }

        if method == .selectAll {
            updateSessionBuffer(backspace: bs, newText: text)
        }

        switch method {
        case .selection: injectViaSelection(bs: bs, text: text, delays: delays)
        case .autocomplete: injectViaAutocomplete(bs: bs, text: text, proxy: proxy)
        case .axDirect: injectViaAXWithFallback(bs: bs, text: text, proxy: proxy)
        case .selectAll: injectViaSelectAll(proxy: proxy)
        case .slow, .fast: injectViaBackspace(bs: bs, text: text, delays: delays)
        case .passthrough: break
        }

        usleep(method == .slow ? 20000 : 5000)
    }

    private func injectViaBackspace(bs: Int, text: String, delays: (UInt32, UInt32, UInt32)) {
        guard let src = CGEventSource(stateID: .privateState) else { return }
        for _ in 0..<bs {
            postKey(KeyCode.backspace, source: src)
            usleep(delays.0)
        }
        if bs > 0 { usleep(delays.1) }
        postText(text, source: src, delay: delays.2)
        Log.send("bs", bs, text)
    }

    private func injectViaSelection(bs: Int, text: String, delays: (UInt32, UInt32, UInt32)) {
        guard let src = CGEventSource(stateID: .privateState) else { return }
        let selDelay = delays.0 > 0 ? delays.0 : 1000
        let waitDelay = delays.1 > 0 ? delays.1 : 3000
        let textDelay = delays.2 > 0 ? delays.2 : 2000

        if bs > 0 {
            if text.isEmpty {
                for _ in 0..<bs {
                    postKey(KeyCode.backspace, source: src)
                    usleep(selDelay)
                }
            } else {
                for _ in 0..<bs {
                    postKey(KeyCode.leftArrow, source: src, flags: .maskShift)
                    usleep(selDelay)
                }
            }
            usleep(waitDelay)
        }
        postText(text, source: src, delay: textDelay)
        Log.send("sel", bs, text)
    }

    private func injectViaAutocomplete(bs: Int, text: String, proxy: CGEventTapProxy) {
        guard let src = CGEventSource(stateID: .privateState) else { return }
        postKey(KeyCode.forwardDelete, source: src, proxy: proxy)
        usleep(3000)
        for _ in 0..<bs {
            postKey(KeyCode.backspace, source: src, proxy: proxy)
            usleep(1000)
        }
        if bs > 0 { usleep(5000) }
        postText(text, source: src, proxy: proxy)
        Log.send("auto", bs, text)
    }

    private func injectViaSelectAll(proxy: CGEventTapProxy) {
        guard let src = CGEventSource(stateID: .privateState) else { return }
        let fullText = sessionBuffer
        guard !fullText.isEmpty else { return }
        postKey(KeyCode.leftArrow, source: src, flags: .maskCommand, proxy: proxy)
        usleep(5000)
        postKey(0x7C, source: src, flags: [.maskCommand, .maskShift], proxy: proxy)
        usleep(5000)
        postText(fullText, source: src, proxy: proxy)
        Log.send("selAll", 0, fullText)
    }

    func injectViaAX(bs: Int, text: String) -> Bool {
        let systemWide = AXUIElementCreateSystemWide()
        var focusedRef: CFTypeRef?
        guard AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focusedRef) == .success,
              let ref = focusedRef else { return false }
        let axEl = ref as! AXUIElement

        var valueRef: CFTypeRef?
        guard AXUIElementCopyAttributeValue(axEl, kAXValueAttribute as CFString, &valueRef) == .success else { return false }
        let fullText = (valueRef as? String) ?? ""

        var rangeRef: CFTypeRef?
        guard AXUIElementCopyAttributeValue(axEl, kAXSelectedTextRangeAttribute as CFString, &rangeRef) == .success,
              let axRange = rangeRef else { return false }
        var range = CFRange()
        guard AXValueGetValue(axRange as! AXValue, .cfRange, &range), range.location >= 0 else { return false }

        let cursor = range.location
        let selection = range.length
        let userText = (selection > 0 && cursor <= fullText.count) ? String(fullText.prefix(cursor)) : fullText

        let deleteStart = max(0, cursor - bs)
        let prefix = String(userText.prefix(deleteStart))
        let suffix = String(userText.dropFirst(cursor))
        let newText = (prefix + text + suffix).precomposedStringWithCanonicalMapping

        guard AXUIElementSetAttributeValue(axEl, kAXValueAttribute as CFString, newText as CFTypeRef) == .success else { return false }

        var newCursor = CFRange(location: deleteStart + text.count, length: 0)
        if let newRange = AXValueCreate(.cfRange, &newCursor) {
            AXUIElementSetAttributeValue(axEl, kAXSelectedTextRangeAttribute as CFString, newRange)
        }
        Log.send("ax", bs, text)
        return true
    }

    func injectViaAXWithFallback(bs: Int, text: String, proxy: CGEventTapProxy) {
        for attempt in 0..<3 {
            if attempt > 0 { usleep(5000) }
            if injectViaAX(bs: bs, text: text) { return }
        }
        injectViaAutocomplete(bs: bs, text: text, proxy: proxy)
    }

    private func postKey(_ keyCode: CGKeyCode, source: CGEventSource, flags: CGEventFlags = [], proxy: CGEventTapProxy? = nil) {
        guard let dn = CGEvent(keyboardEventSource: source, virtualKey: keyCode, keyDown: true),
              let up = CGEvent(keyboardEventSource: source, virtualKey: keyCode, keyDown: false) else { return }
        dn.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
        up.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
        if !flags.isEmpty { dn.flags = flags; up.flags = flags }
        if let proxy = proxy {
            dn.tapPostEvent(proxy)
            up.tapPostEvent(proxy)
        } else {
            dn.post(tap: .cgSessionEventTap)
            up.post(tap: .cgSessionEventTap)
        }
    }

    private func postText(_ text: String, source: CGEventSource, delay: UInt32 = 0, proxy: CGEventTapProxy? = nil) {
        let utf16 = Array(text.utf16)
        var offset = 0
        while offset < utf16.count {
            let end = min(offset + 20, utf16.count)
            let chunk = Array(utf16[offset..<end])
            guard let dn = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true),
                  let up = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: false) else { break }
            dn.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
            up.setIntegerValueField(.eventSourceUserData, value: kEventMarker)
            dn.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: chunk)
            up.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: chunk)
            if let proxy = proxy {
                dn.tapPostEvent(proxy)
                up.tapPostEvent(proxy)
            } else {
                dn.post(tap: .cgSessionEventTap)
                up.post(tap: .cgSessionEventTap)
            }
            if delay > 0 { usleep(delay) }
            offset = end
        }
    }
}

// MARK: - FFI (Rust Bridge)

private struct ImeResult {
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
    var flags: UInt8
}

private let FLAG_KEY_CONSUMED: UInt8 = 0x01

@_silgen_name("ime_init") private func ime_init()
@_silgen_name("ime_key_ext") private func ime_key_ext(_ key: UInt16, _ caps: Bool, _ ctrl: Bool, _ shift: Bool) -> UnsafeMutablePointer<ImeResult>?
@_silgen_name("ime_method") private func ime_method(_ method: UInt8)
@_silgen_name("ime_enabled") private func ime_enabled(_ enabled: Bool)
@_silgen_name("ime_skip_w_shortcut") private func ime_skip_w_shortcut(_ skip: Bool)
@_silgen_name("ime_esc_restore") private func ime_esc_restore(_ enabled: Bool)
@_silgen_name("ime_free_tone") private func ime_free_tone(_ enabled: Bool)
@_silgen_name("ime_modern") private func ime_modern(_ modern: Bool)
@_silgen_name("ime_english_auto_restore") private func ime_english_auto_restore(_ enabled: Bool)
@_silgen_name("ime_auto_capitalize") private func ime_auto_capitalize(_ enabled: Bool)
@_silgen_name("ime_clear") private func ime_clear()
@_silgen_name("ime_clear_all") private func ime_clear_all()
@_silgen_name("ime_free") private func ime_free(_ result: UnsafeMutablePointer<ImeResult>?)
@_silgen_name("ime_add_shortcut") private func ime_add_shortcut(_ trigger: UnsafePointer<CChar>?, _ replacement: UnsafePointer<CChar>?)
@_silgen_name("ime_remove_shortcut") private func ime_remove_shortcut(_ trigger: UnsafePointer<CChar>?)
@_silgen_name("ime_clear_shortcuts") private func ime_clear_shortcuts()
@_silgen_name("ime_restore_word") private func ime_restore_word(_ word: UnsafePointer<CChar>?)
@_silgen_name("ime_get_buffer") private func ime_get_buffer(_ out: UnsafeMutablePointer<UInt32>, _ maxLen: Int) -> Int

// MARK: - RustBridge

class RustBridge {
    private static var isInitialized = false

    static func initialize() {
        guard !isInitialized else { return }
        ime_init()
        isInitialized = true
        Log.info("Engine initialized")
    }

    static func processKey(keyCode: UInt16, caps: Bool, ctrl: Bool, shift: Bool = false) -> (Int, [Character], Bool)? {
        guard isInitialized, let ptr = ime_key_ext(keyCode, caps, ctrl, shift) else { return nil }
        defer { ime_free(ptr) }
        let r = ptr.pointee
        guard r.action == 1 else { return nil }
        let chars = withUnsafePointer(to: r.chars) { p in
            p.withMemoryRebound(to: UInt32.self, capacity: 64) { bound in
                (0..<Int(r.count)).compactMap { Unicode.Scalar(bound[$0]).map(Character.init) }
            }
        }
        let keyConsumed = (r.flags & FLAG_KEY_CONSUMED) != 0
        return (Int(r.backspace), chars, keyConsumed)
    }

    static func setMethod(_ method: Int) {
        ime_method(UInt8(method))
        Log.info("Method: \(method == 0 ? "Telex" : "VNI")")
    }

    static func setEnabled(_ enabled: Bool) {
        let actualEnabled = enabled && InputSourceObserver.shared.isAllowedInputSource
        ime_enabled(actualEnabled)
        DaemonState.shared.setEnabled(actualEnabled)
        Log.info("Enabled: \(actualEnabled)")
    }

    static func setSkipWShortcut(_ skip: Bool) { ime_skip_w_shortcut(skip) }
    static func setEscRestore(_ enabled: Bool) { ime_esc_restore(enabled) }
    static func setFreeTone(_ enabled: Bool) { ime_free_tone(enabled) }
    static func setModernTone(_ modern: Bool) { ime_modern(modern) }
    static func setEnglishAutoRestore(_ enabled: Bool) { ime_english_auto_restore(enabled) }
    static func setAutoCapitalize(_ enabled: Bool) { ime_auto_capitalize(enabled) }
    static func clearBuffer() { ime_clear() }
    static func clearBufferAll() { ime_clear_all() }

    static func getFullBuffer() -> String {
        var buffer = [UInt32](repeating: 0, count: 64)
        let len = ime_get_buffer(&buffer, 64)
        guard len > 0 else { return "" }
        return String(buffer[0..<len].compactMap { Unicode.Scalar($0).map(Character.init) })
    }

    static func restoreWord(_ word: String) {
        word.withCString { ime_restore_word($0) }
    }

    static func addShortcut(trigger: String, replacement: String) {
        trigger.withCString { t in replacement.withCString { r in ime_add_shortcut(t, r) } }
    }

    static func removeShortcut(trigger: String) {
        trigger.withCString { ime_remove_shortcut($0) }
    }

    static func clearShortcuts() { ime_clear_shortcuts() }

    static func syncShortcuts(_ shortcuts: [(key: String, value: String, enabled: Bool)]) {
        ime_clear_shortcuts()
        for s in shortcuts where s.enabled { addShortcut(trigger: s.key, replacement: s.value) }
    }
}

// MARK: - Keyboard Hook Manager

class KeyboardHookManager {
    static let shared = KeyboardHookManager()

    private var eventTap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    private var mouseMonitor: Any?
    private var isRunning = false

    private init() {}

    func start() {
        guard !isRunning else { return }
        guard AXIsProcessTrusted() else {
            let opts = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
            AXIsProcessTrustedWithOptions(opts)
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

        guard let tap = tap else { return }

        eventTap = tap
        runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)
        if let source = runLoopSource {
            CFRunLoopAddSource(CFRunLoopGetCurrent(), source, .commonModes)
            CGEvent.tapEnable(tap: tap, enable: true)
            isRunning = true
            setupShortcutObserver()
            startMouseMonitor()
            Log.info("Hook started")
        }
    }

    private func startMouseMonitor() {
        mouseMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .leftMouseUp]) { _ in
            TextInjector.shared.clearSessionBuffer()
            RustBridge.clearBufferAll()
            skipWordRestoreAfterClick = true
        }
    }

    func stop() {
        guard isRunning else { return }
        if let tap = eventTap { CGEvent.tapEnable(tap: tap, enable: false) }
        if let src = runLoopSource { CFRunLoopRemoveSource(CFRunLoopGetCurrent(), src, .commonModes) }
        if let monitor = mouseMonitor { NSEvent.removeMonitor(monitor) }
        eventTap = nil
        runLoopSource = nil
        mouseMonitor = nil
        isRunning = false
    }

    func getTap() -> CFMachPort? { eventTap }
}

// MARK: - Keyboard Callback

private let kEventMarker: Int64 = 0x474E4820
private let kModifierMask: CGEventFlags = [.maskSecondaryFn, .maskControl, .maskAlternate, .maskShift, .maskCommand]
private var wasModifierShortcutPressed = false
private var currentShortcut = KeyboardShortcut.load()
private var isRecordingShortcut = false
private var recordingModifiers: CGEventFlags = []
private var peakRecordingModifiers: CGEventFlags = []
private var shortcutObserver: NSObjectProtocol?
private var skipWordRestoreAfterClick = false

private extension CGEventFlags {
    var modifierCount: Int {
        [contains(.maskSecondaryFn), contains(.maskControl), contains(.maskAlternate),
         contains(.maskShift), contains(.maskCommand)].filter { $0 }.count
    }
    var isFnOnly: Bool {
        contains(.maskSecondaryFn) && !contains(.maskControl) && !contains(.maskAlternate) &&
        !contains(.maskShift) && !contains(.maskCommand)
    }
}

func startShortcutRecordingInternal() {
    isRecordingShortcut = true
    recordingModifiers = []
    peakRecordingModifiers = []
}

func stopShortcutRecordingInternal() {
    isRecordingShortcut = false
    recordingModifiers = []
    peakRecordingModifiers = []
}

private func setupShortcutObserver() {
    shortcutObserver = NotificationCenter.default.addObserver(forName: .shortcutChanged, object: nil, queue: .main) { _ in
        currentShortcut = KeyboardShortcut.load()
    }
}

private func matchesToggleShortcut(keyCode: UInt16, flags: CGEventFlags) -> Bool {
    currentShortcut.matches(keyCode: keyCode, flags: flags)
}

private func matchesModifierOnlyShortcut(flags: CGEventFlags) -> Bool {
    currentShortcut.matchesModifierOnly(flags: flags)
}

private func keyboardCallback(
    proxy: CGEventTapProxy, type: CGEventType, event: CGEvent, refcon: UnsafeMutableRawPointer?
) -> Unmanaged<CGEvent>? {

    if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
        if let tap = KeyboardHookManager.shared.getTap() { CGEvent.tapEnable(tap: tap, enable: true) }
        return Unmanaged.passUnretained(event)
    }

    if type == .keyDown || type == .keyUp {
        DispatchQueue.main.async { PerAppModeManager.shared.checkSpecialPanelApp() }
    }

    let flags = event.flags

    // Shortcut Recording
    if isRecordingShortcut {
        let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
        let mods = flags.intersection(kModifierMask)

        if type == .keyDown && keyCode == 0x35 {
            stopShortcutRecordingInternal()
            DispatchQueue.main.async { NotificationCenter.default.post(name: .shortcutRecordingCancelled, object: nil) }
            return nil
        }

        if type == .flagsChanged {
            let canSave = peakRecordingModifiers.isFnOnly || peakRecordingModifiers.modifierCount >= 2
            if mods.isEmpty && canSave {
                let captured = KeyboardShortcut(keyCode: 0xFFFF, modifiers: peakRecordingModifiers.rawValue)
                stopShortcutRecordingInternal()
                DispatchQueue.main.async { NotificationCenter.default.post(name: .shortcutRecorded, object: captured) }
            } else {
                recordingModifiers = mods
                if mods.modifierCount > peakRecordingModifiers.modifierCount { peakRecordingModifiers = mods }
            }
            return Unmanaged.passUnretained(event)
        }

        if type == .keyDown && !mods.isEmpty {
            let captured = KeyboardShortcut(keyCode: keyCode, modifiers: mods.rawValue)
            stopShortcutRecordingInternal()
            DispatchQueue.main.async { NotificationCenter.default.post(name: .shortcutRecorded, object: captured) }
            return nil
        }
        return Unmanaged.passUnretained(event)
    }

    // Modifier-only shortcuts
    if type == .flagsChanged {
        if matchesModifierOnlyShortcut(flags: flags) {
            wasModifierShortcutPressed = true
        } else if wasModifierShortcutPressed {
            wasModifierShortcutPressed = false
            DispatchQueue.main.async { NotificationCenter.default.post(name: .toggleVietnamese, object: nil) }
        }
        return Unmanaged.passUnretained(event)
    }

    guard type == .keyDown else { return Unmanaged.passUnretained(event) }
    wasModifierShortcutPressed = false

    if event.getIntegerValueField(.eventSourceUserData) == kEventMarker {
        Log.skip()
        return Unmanaged.passUnretained(event)
    }

    let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))

    if matchesToggleShortcut(keyCode: keyCode, flags: flags) {
        DispatchQueue.main.async { NotificationCenter.default.post(name: .toggleVietnamese, object: nil) }
        return nil
    }

    let shift = flags.contains(.maskShift)
    let caps = shift || flags.contains(.maskAlphaShift)
    let ctrl = flags.contains(.maskCommand) || flags.contains(.maskControl) || flags.contains(.maskAlternate)

    // Enter/Escape
    if keyCode == 0x24 || keyCode == 0x4C {
        _ = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift)
        TextInjector.shared.clearSessionBuffer()
        return Unmanaged.passUnretained(event)
    }
    if keyCode == 0x35 {
        TextInjector.shared.clearSessionBuffer()
        RustBridge.clearBuffer()
        return Unmanaged.passUnretained(event)
    }

    let (method, delays) = detectMethod()

    if method == .passthrough { return Unmanaged.passUnretained(event) }

    // Arrow keys with modifier
    let arrowKeys: Set<UInt16> = [UInt16(KeyCode.leftArrow), UInt16(KeyCode.rightArrow), UInt16(KeyCode.upArrow), UInt16(KeyCode.downArrow)]
    let hasModifier = flags.contains(.maskCommand) || flags.contains(.maskAlternate) || flags.contains(.maskShift)
    if arrowKeys.contains(keyCode) && hasModifier {
        RustBridge.clearBuffer()
        TextInjector.shared.clearSessionBuffer()
        return Unmanaged.passUnretained(event)
    }

    // Cmd shortcuts
    if flags.contains(.maskCommand) && !flags.contains(.maskControl) && !flags.contains(.maskAlternate) {
        let textModifyingKeys: Set<UInt16> = [0x00, 0x09, 0x07, 0x06]
        if textModifyingKeys.contains(keyCode) {
            RustBridge.clearBuffer()
            if method == .selectAll {
                if keyCode == 0x00 {
                    TextInjector.shared.clearSessionBuffer()
                } else {
                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                        if let text = getTextFromFocusedElement() {
                            TextInjector.shared.setSessionBuffer(text)
                        } else {
                            TextInjector.shared.clearSessionBuffer()
                        }
                    }
                }
            } else {
                TextInjector.shared.clearSessionBuffer()
            }
        }
        return Unmanaged.passUnretained(event)
    }

    // Backspace handling
    if keyCode == KeyCode.backspace && !ctrl {
        if method == .selectAll && DaemonState.shared.isEnabled {
            let session = TextInjector.shared.getSessionBuffer()
            if !session.isEmpty {
                TextInjector.shared.updateSessionBuffer(backspace: 1, newText: "")
                TextInjector.shared.injectSelectAllOnly(proxy: proxy)
                return nil
            } else {
                return Unmanaged.passUnretained(event)
            }
        }

        if let (bs, chars, _) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
            let str = String(chars)
            Log.transform(bs, str)
            sendReplacement(backspace: bs, chars: chars, method: method, delays: delays, proxy: proxy)
            return nil
        }

        if !skipWordRestoreAfterClick, let word = getWordToRestoreOnBackspace() {
            RustBridge.restoreWord(word)
        }
        return Unmanaged.passUnretained(event)
    }

    let isLetterKey = keyCode <= 0x32 && keyCode != KeyCode.backspace
    if isLetterKey { skipWordRestoreAfterClick = false }

    if let (bs, chars, keyConsumed) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
        Log.transform(bs, String(chars))
        sendReplacement(backspace: bs, chars: chars, method: method, delays: delays, proxy: proxy)
        let shouldPassThrough = isBreakKey(keyCode, shift: shift) && keyCode != KeyCode.space && !keyConsumed
        return shouldPassThrough ? Unmanaged.passUnretained(event) : nil
    }

    if method == .selectAll && DaemonState.shared.isEnabled {
        if let char = keyCodeToChar(keyCode: keyCode, shift: shift) {
            TextInjector.shared.updateSessionBuffer(backspace: 0, newText: String(char))
            TextInjector.shared.injectSelectAllOnly(proxy: proxy)
            return nil
        }
    }

    return Unmanaged.passUnretained(event)
}

// MARK: - Helper Functions

private func getTextFromFocusedElement() -> String? {
    let systemWide = AXUIElementCreateSystemWide()
    var focused: CFTypeRef?
    guard AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focused) == .success,
          let el = focused else { return nil }
    var textValue: CFTypeRef?
    guard AXUIElementCopyAttributeValue(el as! AXUIElement, kAXValueAttribute as CFString, &textValue) == .success,
          let text = textValue as? String else { return nil }
    return text
}

private func getWordToRestoreOnBackspace() -> String? {
    let systemWide = AXUIElementCreateSystemWide()
    var focused: CFTypeRef?
    guard AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focused) == .success,
          let el = focused else { return nil }
    let axEl = el as! AXUIElement

    var textValue: CFTypeRef?
    guard AXUIElementCopyAttributeValue(axEl, kAXValueAttribute as CFString, &textValue) == .success,
          let text = textValue as? String, !text.isEmpty else { return nil }

    var rangeValue: CFTypeRef?
    guard AXUIElementCopyAttributeValue(axEl, kAXSelectedTextRangeAttribute as CFString, &rangeValue) == .success else { return nil }
    var range = CFRange()
    guard AXValueGetValue(rangeValue as! AXValue, .cfRange, &range) else { return nil }

    let cursorPos = range.location
    guard cursorPos > 0, cursorPos <= text.count else { return nil }

    let textChars = Array(text)
    let charBeforeCursor = textChars[cursorPos - 1]
    guard charBeforeCursor.isWhitespace || charBeforeCursor.isPunctuation else { return nil }

    var wordEnd = cursorPos - 1
    while wordEnd > 0 && (textChars[wordEnd - 1].isWhitespace || textChars[wordEnd - 1].isPunctuation) { wordEnd -= 1 }
    guard wordEnd > 0, wordEnd == cursorPos - 1 else { return nil }

    var wordStart = wordEnd
    while wordStart > 0 && !textChars[wordStart - 1].isWhitespace && !textChars[wordStart - 1].isPunctuation { wordStart -= 1 }

    let word = String(textChars[wordStart..<wordEnd])
    guard !word.isEmpty else { return nil }

    let hasVietnameseDiacritics = word.contains { c in
        c.unicodeScalars.first.map { $0.value >= 0x00C0 && $0.value <= 0x1EF9 } ?? false
    }
    let isPureASCIILetters = word.allSatisfy { $0.isLetter && $0.isASCII }
    return (hasVietnameseDiacritics || isPureASCIILetters) ? word : nil
}

private func keyCodeToChar(keyCode: UInt16, shift: Bool) -> Character? {
    let keyMap: [UInt16: (normal: Character, shifted: Character)] = [
        0x00: ("a", "A"), 0x0B: ("b", "B"), 0x08: ("c", "C"), 0x02: ("d", "D"),
        0x0E: ("e", "E"), 0x03: ("f", "F"), 0x05: ("g", "G"), 0x04: ("h", "H"),
        0x22: ("i", "I"), 0x26: ("j", "J"), 0x28: ("k", "K"), 0x25: ("l", "L"),
        0x2E: ("m", "M"), 0x2D: ("n", "N"), 0x1F: ("o", "O"), 0x23: ("p", "P"),
        0x0C: ("q", "Q"), 0x0F: ("r", "R"), 0x01: ("s", "S"), 0x11: ("t", "T"),
        0x20: ("u", "U"), 0x09: ("v", "V"), 0x0D: ("w", "W"), 0x07: ("x", "X"),
        0x10: ("y", "Y"), 0x06: ("z", "Z"),
        0x12: ("1", "!"), 0x13: ("2", "@"), 0x14: ("3", "#"), 0x15: ("4", "$"),
        0x17: ("5", "%"), 0x16: ("6", "^"), 0x1A: ("7", "&"), 0x1C: ("8", "*"),
        0x19: ("9", "("), 0x1D: ("0", ")"),
        0x31: (" ", " "), 0x2B: (",", "<"), 0x2F: (".", ">"), 0x2C: ("/", "?"),
        0x27: ("'", "\""), 0x29: (";", ":"), 0x1E: ("]", "}"), 0x21: ("[", "{"),
        0x2A: ("\\", "|"), 0x18: ("=", "+"), 0x1B: ("-", "_"), 0x32: ("`", "~"),
    ]
    return keyMap[keyCode].map { shift ? $0.shifted : $0.normal }
}

private func detectMethod() -> (InjectionMethod, (UInt32, UInt32, UInt32)) {
    let systemWide = AXUIElementCreateSystemWide()
    var focused: CFTypeRef?
    var role: String?
    var bundleId: String?

    if AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focused) == .success,
       let el = focused {
        let axEl = el as! AXUIElement
        var roleVal: CFTypeRef?
        AXUIElementCopyAttributeValue(axEl, kAXRoleAttribute as CFString, &roleVal)
        role = roleVal as? String
        var pid: pid_t = 0
        if AXUIElementGetPid(axEl, &pid) == .success {
            bundleId = NSRunningApplication(processIdentifier: pid)?.bundleIdentifier
        }
    }
    if bundleId == nil { bundleId = NSWorkspace.shared.frontmostApplication?.bundleIdentifier }
    guard let bundleId = bundleId else { return (.fast, (200, 800, 500)) }

    if bundleId == "com.apple.ScreenContinuity" { return (.passthrough, (0, 0, 0)) }
    if role == "AXComboBox" || role == "AXSearchField" { return (.selection, (0, 0, 0)) }
    if bundleId == "com.apple.Spotlight" || bundleId == "com.apple.systemuiserver" { return (.axDirect, (0, 0, 0)) }

    let arcApps = ["company.thebrowser.Browser", "company.thebrowser.Arc", "company.thebrowser.dia"]
    if arcApps.contains(bundleId) && (role == "AXTextField" || role == "AXTextArea") { return (.axDirect, (0, 0, 0)) }

    let firefoxBrowsers = ["org.mozilla.firefox", "org.mozilla.firefoxdeveloperedition", "org.mozilla.nightly",
                           "org.waterfoxproject.waterfox", "io.gitlab.librewolf-community.librewolf",
                           "one.ablaze.floorp", "org.torproject.torbrowser", "net.mullvad.mullvadbrowser", "app.zen-browser.zen"]
    if firefoxBrowsers.contains(bundleId) && (role == "AXTextField" || role == "AXWindow") { return (.axDirect, (0, 0, 0)) }

    let browsers = ["com.google.Chrome", "com.google.Chrome.canary", "com.google.Chrome.beta", "org.chromium.Chromium",
                    "com.brave.Browser", "com.brave.Browser.beta", "com.brave.Browser.nightly",
                    "com.microsoft.edgemac", "com.vivaldi.Vivaldi", "com.opera.Opera", "com.operasoftware.OperaGX",
                    "com.apple.Safari", "com.apple.SafariTechnologyPreview", "com.kagi.kagimacOS",
                    "com.duckduckgo.macos.browser", "ai.perplexity.comet"]
    if browsers.contains(bundleId) && role == "AXTextField" { return (.selection, (0, 0, 0)) }
    if role == "AXTextField" && bundleId.hasPrefix("com.jetbrains") { return (.selection, (0, 0, 0)) }

    if bundleId == "com.microsoft.Excel" || bundleId == "com.microsoft.Word" { return (.slow, (3000, 8000, 3000)) }
    if bundleId == "com.todesktop.230313mzl4w4u92" { return (.slow, (8000, 15000, 8000)) }
    if bundleId == "notion.id" { return (.slow, (12000, 25000, 12000)) }
    if bundleId == "dev.warp.Warp-Stable" { return (.slow, (8000, 15000, 8000)) }

    let terminals = ["com.apple.Terminal", "com.googlecode.iterm2", "io.alacritty", "com.github.wez.wezterm",
                     "com.mitchellh.ghostty", "net.kovidgoyal.kitty", "co.zeit.hyper", "org.tabby",
                     "com.raphaelamorim.rio", "com.termius-dmg.mac", "com.microsoft.VSCode", "com.google.antigravity",
                     "dev.zed.Zed", "com.sublimetext.4", "com.sublimetext.3", "com.panic.Nova"]
    if terminals.contains(bundleId) { return (.slow, (3000, 8000, 3000)) }
    if bundleId.hasPrefix("com.jetbrains") { return (.slow, (3000, 8000, 3000)) }

    return (.fast, (1000, 3000, 1500))
}

private func sendReplacement(backspace bs: Int, chars: [Character], method: InjectionMethod, delays: (UInt32, UInt32, UInt32), proxy: CGEventTapProxy) {
    TextInjector.shared.injectSync(bs: bs, text: String(chars), method: method, delays: delays, proxy: proxy)
}

// MARK: - Per-App Mode Manager

class PerAppModeManager {
    static let shared = PerAppModeManager()

    private var currentBundleId: String?
    private var observer: NSObjectProtocol?
    private var mouseClickMonitor: Any?

    private init() {}

    func start() {
        observer = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication,
                  let bundleId = app.bundleIdentifier else { return }
            SpecialPanelAppDetector.updateLastFrontMostApp(bundleId)
            self?.handleAppSwitch(bundleId)
        }

        mouseClickMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak self] _ in
            self?.checkSpecialPanelApp()
        }
    }

    func stop() {
        if let observer = observer { NSWorkspace.shared.notificationCenter.removeObserver(observer) }
        if let monitor = mouseClickMonitor { NSEvent.removeMonitor(monitor) }
        observer = nil
        mouseClickMonitor = nil
    }

    func checkSpecialPanelApp() {
        guard DaemonState.shared.perAppModeEnabled else { return }
        let (appChanged, newBundleId, _) = SpecialPanelAppDetector.checkForAppChange()
        if appChanged, let bundleId = newBundleId { handleAppSwitch(bundleId) }
    }

    private func handleAppSwitch(_ bundleId: String) {
        guard bundleId != currentBundleId else { return }
        currentBundleId = bundleId
        RustBridge.clearBuffer()
        TextInjector.shared.clearSessionBuffer()

        guard DaemonState.shared.perAppModeEnabled else { return }
        let mode = DaemonState.shared.getPerAppMode(bundleId: bundleId)
        RustBridge.setEnabled(mode)
    }
}

// MARK: - Notifications

extension Notification.Name {
    static let toggleVietnamese = Notification.Name("toggleVietnamese")
    static let shortcutChanged = Notification.Name("shortcutChanged")
    static let shortcutRecorded = Notification.Name("shortcutRecorded")
    static let shortcutRecordingCancelled = Notification.Name("shortcutRecordingCancelled")
}
