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

// MARK: - Constants

private enum KeyCode {
    static let backspace: CGKeyCode = 0x33
    static let forwardDelete: CGKeyCode = 0x75
    static let leftArrow: CGKeyCode = 0x7B
}

// MARK: - Injection Method

private enum InjectionMethod {
    case fast           // Default: backspace + text with minimal delays
    case slow           // Terminals/Electron: backspace + text with higher delays
    case selection      // Browser address bars: Shift+Left select + type replacement
    case autocomplete   // Spotlight: Forward Delete + backspace + text via proxy
}

// MARK: - Text Injector

/// Handles text injection with proper sequencing to prevent race conditions
private class TextInjector {
    static let shared = TextInjector()

    /// Semaphore to block keyboard callback until injection completes
    private let semaphore = DispatchSemaphore(value: 1)

    private init() {}

    /// Inject text replacement synchronously (blocks until complete)
    func injectSync(bs: Int, text: String, method: InjectionMethod, delays: (UInt32, UInt32, UInt32), proxy: CGEventTapProxy) {
        semaphore.wait()
        defer { semaphore.signal() }

        switch method {
        case .selection:
            injectViaSelection(bs: bs, text: text, delays: delays)
        case .autocomplete:
            injectViaAutocomplete(bs: bs, text: text, proxy: proxy)
        case .slow, .fast:
            injectViaBackspace(bs: bs, text: text, delays: delays)
        }

        // Settle time: 20ms for slow apps, 5ms for others
        usleep(method == .slow ? 20000 : 5000)
    }

    // MARK: - Injection Methods

    /// Standard backspace injection: delete N chars, then type replacement
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

    /// Selection injection: Shift+Left to select, then type replacement (for browser address bars)
    private func injectViaSelection(bs: Int, text: String, delays: (UInt32, UInt32, UInt32)) {
        guard let src = CGEventSource(stateID: .privateState) else { return }

        let selDelay = delays.0 > 0 ? delays.0 : 1000
        let waitDelay = delays.1 > 0 ? delays.1 : 3000
        let textDelay = delays.2 > 0 ? delays.2 : 2000

        for _ in 0..<bs {
            postKey(KeyCode.leftArrow, source: src, flags: .maskShift)
            usleep(selDelay)
        }
        if bs > 0 { usleep(waitDelay) }

        postText(text, source: src, delay: textDelay)
        Log.send("sel", bs, text)
    }

    /// Autocomplete injection: Forward Delete to clear suggestion, then backspace + text
    /// Used for Spotlight where autocomplete auto-selects suggestion text after cursor
    /// NOTE: Uses .cgSessionEventTap instead of proxy for macOS 13 compatibility
    private func injectViaAutocomplete(bs: Int, text: String, proxy: CGEventTapProxy) {
        guard let src = CGEventSource(stateID: .privateState) else { return }

        // Forward Delete clears auto-selected suggestion (no proxy - use session tap)
        postKey(KeyCode.forwardDelete, source: src)
        usleep(15000)  // 15ms - macOS 13 needs more time after FD before accepting backspace

        // Backspaces remove typed characters
        for _ in 0..<bs {
            postKey(KeyCode.backspace, source: src)
            usleep(5000)  // 5ms between backspaces for reliability
        }
        if bs > 0 { usleep(10000) }  // 10ms settle after all backspaces

        // Type replacement text
        postText(text, source: src, delay: 3000)  // 3ms per char for reliability
        Log.send("auto", bs, text)
    }

    // MARK: - Helpers

    /// Post a single key press event
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

    /// Post text in chunks (CGEvent has 20-char limit)
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
@_silgen_name("ime_skip_w_shortcut") private func ime_skip_w_shortcut(_ skip: Bool)
@_silgen_name("ime_clear") private func ime_clear()
@_silgen_name("ime_free") private func ime_free(_ result: UnsafeMutablePointer<ImeResult>?)

// Shortcut FFI
@_silgen_name("ime_add_shortcut") private func ime_add_shortcut(_ trigger: UnsafePointer<CChar>?, _ replacement: UnsafePointer<CChar>?)
@_silgen_name("ime_remove_shortcut") private func ime_remove_shortcut(_ trigger: UnsafePointer<CChar>?)
@_silgen_name("ime_clear_shortcuts") private func ime_clear_shortcuts()

// Word Restore FFI
@_silgen_name("ime_restore_word") private func ime_restore_word(_ word: UnsafePointer<CChar>?)

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

    /// Set whether to skip w→ư shortcut in Telex mode
    static func setSkipWShortcut(_ skip: Bool) {
        ime_skip_w_shortcut(skip)
        Log.info("Skip W shortcut: \(skip)")
    }

    static func clearBuffer() { ime_clear() }

    /// Restore buffer from a Vietnamese word (for backspace-into-word editing)
    static func restoreWord(_ word: String) {
        word.withCString { w in
            ime_restore_word(w)
        }
        Log.info("Restored word: \(word)")
    }

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
private let kModifierMask: CGEventFlags = [.maskControl, .maskAlternate, .maskShift, .maskCommand]
private var wasModifierShortcutPressed = false
private var currentShortcut = KeyboardShortcut.load()
private var isRecordingShortcut = false
private var recordingModifiers: CGEventFlags = []
private var shortcutObserver: NSObjectProtocol?

// MARK: - Word Restore Support

/// Get word that we're about to backspace into
/// Returns the word only if cursor is right after a space/punctuation that follows a word
/// This ensures we only restore when actually entering a word, not when deleting within a word
private func getWordToRestoreOnBackspace() -> String? {
    let systemWide = AXUIElementCreateSystemWide()
    var focused: CFTypeRef?

    guard AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focused) == .success,
          let el = focused else {
        Log.info("restore: no focused element")
        return nil
    }

    let axEl = el as! AXUIElement

    // Get text value
    var textValue: CFTypeRef?
    let textResult = AXUIElementCopyAttributeValue(axEl, kAXValueAttribute as CFString, &textValue)
    guard textResult == .success, let text = textValue as? String, !text.isEmpty else {
        Log.info("restore: no text value (err=\(textResult.rawValue))")
        return nil
    }

    // Get selected text range (cursor position)
    var rangeValue: CFTypeRef?
    let rangeResult = AXUIElementCopyAttributeValue(axEl, kAXSelectedTextRangeAttribute as CFString, &rangeValue)
    guard rangeResult == .success else {
        Log.info("restore: no range (err=\(rangeResult.rawValue))")
        return nil
    }

    // Extract range from AXValue
    var range = CFRange(location: 0, length: 0)
    guard AXValueGetValue(rangeValue as! AXValue, .cfRange, &range) else {
        Log.info("restore: can't extract range")
        return nil
    }

    let cursorPos = range.location
    Log.info("restore: cursor=\(cursorPos) text='\(text.prefix(50))...'")
    guard cursorPos > 0 else { return nil }

    let textChars = Array(text)
    guard cursorPos <= textChars.count else {
        Log.info("restore: cursor out of bounds")
        return nil
    }
    let charBeforeCursor = textChars[cursorPos - 1]
    Log.info("restore: charBefore='\(charBeforeCursor)'")

    // Only restore if we're about to delete the LAST space/punctuation before a word
    // i.e., cursor is at: "word |" (about to delete space and enter "word")
    guard charBeforeCursor.isWhitespace || charBeforeCursor.isPunctuation else {
        Log.info("restore: not at word boundary")
        return nil
    }

    // Check if there's a word before this space (not more spaces)
    var wordEnd = cursorPos - 1

    // Skip all trailing spaces/punctuation to find the word
    while wordEnd > 0 && (textChars[wordEnd - 1].isWhitespace || textChars[wordEnd - 1].isPunctuation) {
        wordEnd -= 1
    }

    guard wordEnd > 0 else {
        Log.info("restore: no word before spaces")
        return nil
    }

    // But we only want to restore when deleting THE LAST space before the word
    // If there are more spaces between cursor and word, don't restore yet
    if wordEnd < cursorPos - 1 {
        Log.info("restore: multiple spaces before word")
        return nil  // More than one space/punct between cursor and word
    }

    // Find start of word
    var wordStart = wordEnd
    while wordStart > 0 && !textChars[wordStart - 1].isWhitespace && !textChars[wordStart - 1].isPunctuation {
        wordStart -= 1
    }

    // Extract word
    let word = String(textChars[wordStart..<wordEnd])
    guard !word.isEmpty else { return nil }

    // Only return if it looks like Vietnamese (has diacritics or is pure ASCII letters)
    let hasVietnameseDiacritics = word.contains { c in
        let scalars = c.unicodeScalars
        return scalars.first.map { $0.value >= 0x00C0 && $0.value <= 0x1EF9 } ?? false
    }
    let isPureASCIILetters = word.allSatisfy { $0.isLetter && $0.isASCII }

    if hasVietnameseDiacritics || isPureASCIILetters {
        Log.info("restore: found word '\(word)'")
        return word
    }

    Log.info("restore: word '\(word)' not Vietnamese")
    return nil
}

private extension CGEventFlags {
    var modifierCount: Int {
        [contains(.maskControl), contains(.maskAlternate), contains(.maskShift), contains(.maskCommand)].filter { $0 }.count
    }
}

// MARK: - Shortcut Recording

func startShortcutRecording() { isRecordingShortcut = true; recordingModifiers = [] }
func stopShortcutRecording() { isRecordingShortcut = false; recordingModifiers = [] }

func setupShortcutObserver() {
    shortcutObserver = NotificationCenter.default.addObserver(forName: .shortcutChanged, object: nil, queue: .main) { _ in
        currentShortcut = KeyboardShortcut.load()
        Log.info("Shortcut updated: \(currentShortcut.displayParts.joined())")
    }
}

private func matchesToggleShortcut(keyCode: UInt16, flags: CGEventFlags) -> Bool {
    guard currentShortcut.keyCode != 0xFFFF, keyCode == currentShortcut.keyCode else { return false }
    let saved = CGEventFlags(rawValue: currentShortcut.modifiers)
    let required: [CGEventFlags] = [.maskControl, .maskAlternate, .maskShift, .maskCommand]
    for mod in required where saved.contains(mod) && !flags.contains(mod) { return false }
    return !(!saved.contains(.maskCommand) && flags.contains(.maskCommand))
}

private func matchesModifierOnlyShortcut(flags: CGEventFlags) -> Bool {
    guard currentShortcut.keyCode == 0xFFFF else { return false }
    let saved = CGEventFlags(rawValue: currentShortcut.modifiers)
    return flags.intersection(kModifierMask) == saved.intersection(kModifierMask)
}

private func keyboardCallback(
    proxy: CGEventTapProxy, type: CGEventType, event: CGEvent, refcon: UnsafeMutableRawPointer?
) -> Unmanaged<CGEvent>? {

    if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
        if let tap = KeyboardHookManager.shared.getTap() { CGEvent.tapEnable(tap: tap, enable: true) }
        return Unmanaged.passUnretained(event)
    }

    let flags = event.flags

    // MARK: Shortcut Recording Mode
    if isRecordingShortcut {
        let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
        let mods = flags.intersection(kModifierMask)

        // ESC cancels
        if type == .keyDown && keyCode == 0x35 {
            stopShortcutRecording()
            DispatchQueue.main.async { NotificationCenter.default.post(name: .shortcutRecordingCancelled, object: nil) }
            return nil
        }

        // Modifier changes: track or save modifier-only shortcut on release
        if type == .flagsChanged {
            if mods.isEmpty && recordingModifiers.modifierCount >= 2 {
                let captured = KeyboardShortcut(keyCode: 0xFFFF, modifiers: recordingModifiers.rawValue)
                stopShortcutRecording()
                DispatchQueue.main.async { NotificationCenter.default.post(name: .shortcutRecorded, object: captured) }
            } else {
                recordingModifiers = mods
            }
            return Unmanaged.passUnretained(event)
        }

        // Key + modifiers: save shortcut (e.g., Ctrl+Shift+N)
        if type == .keyDown && !mods.isEmpty {
            let captured = KeyboardShortcut(keyCode: keyCode, modifiers: mods.rawValue)
            stopShortcutRecording()
            DispatchQueue.main.async { NotificationCenter.default.post(name: .shortcutRecorded, object: captured) }
            return nil
        }

        return Unmanaged.passUnretained(event)
    }

    // Handle modifier-only shortcuts (Ctrl+Shift, Cmd+Option, etc.)
    if type == .flagsChanged {
        if matchesModifierOnlyShortcut(flags: flags) {
            wasModifierShortcutPressed = true
        } else if wasModifierShortcutPressed {
            // Modifier combo was pressed and now released - toggle
            wasModifierShortcutPressed = false
            DispatchQueue.main.async { NotificationCenter.default.post(name: .toggleVietnamese, object: nil) }
        }
        return Unmanaged.passUnretained(event)
    }

    guard type == .keyDown else { return Unmanaged.passUnretained(event) }

    // Reset modifier state if any key is pressed while modifiers are held
    wasModifierShortcutPressed = false

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

    // Backspace handling: try to restore word from screen when backspacing into it
    // This enables editing marks on previously committed words
    if keyCode == KeyCode.backspace && !ctrl {
        // First try Rust engine (handles immediate backspace-after-space)
        if let (bs, chars) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
            let str = String(chars)
            Log.transform(bs, str)
            sendReplacement(backspace: bs, chars: chars, proxy: proxy)
            return nil
        }

        // Engine returned none - try to restore word from screen
        // This handles: "chào " + backspace to delete space and enter word
        if let word = getWordToRestoreOnBackspace() {
            RustBridge.restoreWord(word)
            Log.info("Restored word from screen: \(word)")
        }

        // Pass through backspace to delete the character
        return Unmanaged.passUnretained(event)
    }

    if let (bs, chars) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
        let str = String(chars)
        Log.transform(bs, str)
        sendReplacement(backspace: bs, chars: chars, proxy: proxy)
        return nil
    }

    // Debug: log frontmost app for all keystrokes
    if let app = NSWorkspace.shared.frontmostApplication {
        Log.info("frontmost: \(app.bundleIdentifier ?? "nil")")
    }
    Log.key(keyCode, "pass")
    return Unmanaged.passUnretained(event)
}

// MARK: - Text Replacement

private func detectMethod() -> (InjectionMethod, (UInt32, UInt32, UInt32)) {
    // Get focused element and its owning app (works for overlays like Spotlight)
    let systemWide = AXUIElementCreateSystemWide()
    var focused: CFTypeRef?
    var role: String?
    var bundleId: String?
    let frontmostBundleId = NSWorkspace.shared.frontmostApplication?.bundleIdentifier

    if AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focused) == .success,
       let el = focused {
        let axEl = el as! AXUIElement

        // Get role
        var roleVal: CFTypeRef?
        AXUIElementCopyAttributeValue(axEl, kAXRoleAttribute as CFString, &roleVal)
        role = roleVal as? String

        // Get owning app's bundle ID (works for Spotlight overlay)
        var pid: pid_t = 0
        if AXUIElementGetPid(axEl, &pid) == .success {
            if let app = NSRunningApplication(processIdentifier: pid) {
                bundleId = app.bundleIdentifier

                // Validate: if focused element's app is not active and not Spotlight,
                // it's likely stale - use frontmost app instead
                if !app.isActive && bundleId != "com.apple.Spotlight" {
                    Log.info("detect: stale focus \(bundleId ?? "nil"), using frontmost \(frontmostBundleId ?? "nil")")
                    bundleId = frontmostBundleId
                }
            }
        }
    }

    // Fallback to frontmost app if we couldn't get bundle from focused element
    if bundleId == nil {
        bundleId = frontmostBundleId
    }

    guard let bundleId = bundleId else { return (.fast, (200, 800, 500)) }

    // Debug: log bundle and role for investigation
    Log.info("detect: \(bundleId) role=\(role ?? "nil")")

    // Selection method for autocomplete UI elements (ComboBox, SearchField)
    if role == "AXComboBox" { Log.method("sel:combo"); return (.selection, (0, 0, 0)) }
    if role == "AXSearchField" { Log.method("sel:search"); return (.selection, (0, 0, 0)) }

    // Spotlight - use autocomplete method with Forward Delete to clear suggestions
    if bundleId == "com.apple.Spotlight" { Log.method("auto:spotlight"); return (.autocomplete, (0, 0, 0)) }

    // Browser address bars (AXTextField with autocomplete)
    let browsers = [
        // Chromium-based
        "com.google.Chrome",             // Google Chrome
        "com.google.Chrome.canary",      // Chrome Canary
        "com.google.Chrome.beta",        // Chrome Beta
        "org.chromium.Chromium",         // Chromium
        "com.brave.Browser",             // Brave
        "com.brave.Browser.beta",        // Brave Beta
        "com.brave.Browser.nightly",     // Brave Nightly
        "com.microsoft.edgemac",         // Microsoft Edge
        "com.microsoft.edgemac.Beta",    // Edge Beta
        "com.microsoft.edgemac.Dev",     // Edge Dev
        "com.microsoft.edgemac.Canary",  // Edge Canary
        "com.vivaldi.Vivaldi",           // Vivaldi
        "com.vivaldi.Vivaldi.snapshot",  // Vivaldi Snapshot
        "ru.yandex.desktop.yandex-browser", // Yandex Browser
        // Opera
        "com.opera.Opera",               // Opera
        "com.operasoftware.Opera",       // Opera (alt)
        "com.operasoftware.OperaGX",     // Opera GX
        "com.operasoftware.OperaAir",    // Opera Air
        "com.opera.OperaNext",           // Opera Next
        // Firefox-based
        "org.mozilla.firefox",           // Firefox
        "org.mozilla.firefoxdeveloperedition", // Firefox Developer
        "org.mozilla.nightly",           // Firefox Nightly
        "org.waterfoxproject.waterfox",  // Waterfox
        "io.gitlab.librewolf-community.librewolf", // LibreWolf
        "one.ablaze.floorp",             // Floorp
        "org.torproject.torbrowser",     // Tor Browser
        "net.mullvad.mullvadbrowser",    // Mullvad Browser
        // Safari
        "com.apple.Safari",              // Safari
        "com.apple.SafariTechnologyPreview", // Safari Tech Preview
        // WebKit-based
        "com.kagi.kagimacOS",            // Orion (Kagi)
        // Arc & Others
        "company.thebrowser.Browser",    // The Browser Company
        "company.thebrowser.Arc",        // Arc
        "company.thebrowser.dia",        // Dia (The Browser Company)
        "app.zen-browser.zen",           // Zen Browser
        "com.sigmaos.sigmaos.macos",     // SigmaOS
        "com.pushplaylabs.sidekick",     // Sidekick
        "com.firstversionist.polypane",  // Polypane
        "ai.perplexity.comet",           // Comet (Perplexity AI)
        "com.duckduckgo.macos.browser"   // DuckDuckGo
    ]
    if browsers.contains(bundleId) && role == "AXTextField" { Log.method("sel:browser"); return (.selection, (0, 0, 0)) }
    if role == "AXTextField" && bundleId.hasPrefix("com.jetbrains") { Log.method("sel:jb"); return (.selection, (0, 0, 0)) }

    // Microsoft Office apps - use backspace method instead of selection
    // Selection method conflicts with Office's autocomplete/suggestion features
    // which can cause the first character to be lost (issue #36)
    if bundleId == "com.microsoft.Excel" { Log.method("slow:excel"); return (.slow, (3000, 8000, 3000)) }
    if bundleId == "com.microsoft.Word" { Log.method("slow:word"); return (.slow, (3000, 8000, 3000)) }

    // Electron apps - higher delays for reliable text replacement
    if bundleId == "com.todesktop.230313mzl4w4u92" { Log.method("slow:claude"); return (.slow, (8000, 15000, 8000)) }
    if bundleId == "notion.id" { Log.method("slow:notion"); return (.slow, (8000, 15000, 8000)) }

    // Terminal/IDE apps - conservative delays for reliability
    let terminals = [
        // Terminals
        "com.apple.Terminal", "com.googlecode.iterm2", "io.alacritty",
        "com.github.wez.wezterm", "com.mitchellh.ghostty", "dev.warp.Warp-Stable",
        "net.kovidgoyal.kitty", "co.zeit.hyper", "org.tabby", "com.raphaelamorim.rio",
        "com.termius-dmg.mac",
        // IDEs/Editors
        "com.microsoft.VSCode", "com.google.antigravity", "dev.zed.Zed",
        "com.sublimetext.4", "com.sublimetext.3", "com.panic.Nova"
    ]
    if terminals.contains(bundleId) { Log.method("slow:term"); return (.slow, (3000, 8000, 3000)) }
    // JetBrains IDEs (IntelliJ, PyCharm, WebStorm, GoLand, Fleet, etc.)
    if bundleId.hasPrefix("com.jetbrains") { Log.method("slow:jb"); return (.slow, (3000, 8000, 3000)) }

    // Default: safe delays for stability across unknown apps
    Log.method("default")
    return (.fast, (1000, 3000, 1500))
}

private func sendReplacement(backspace bs: Int, chars: [Character], proxy: CGEventTapProxy) {
    let (method, delays) = detectMethod()
    let str = String(chars)

    // Use TextInjector for synchronized text injection
    TextInjector.shared.injectSync(bs: bs, text: str, method: method, delays: delays, proxy: proxy)
}

// MARK: - Per-App Mode Manager

class PerAppModeManager {
    static let shared = PerAppModeManager()

    private var currentBundleId: String?
    private var observer: NSObjectProtocol?

    private init() {}

    /// Start observing frontmost app changes
    func start() {
        // IMPORTANT: NSWorkspace notifications are posted to NSWorkspace.shared.notificationCenter,
        // NOT NotificationCenter.default!
        observer = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication,
                  let bundleId = app.bundleIdentifier else { return }
            self?.handleAppSwitch(bundleId)
        }
        Log.info("PerAppModeManager started")
    }

    /// Stop observing
    func stop() {
        if let observer = observer {
            NSWorkspace.shared.notificationCenter.removeObserver(observer)
            self.observer = nil
        }
    }

    private func handleAppSwitch(_ bundleId: String) {
        guard bundleId != currentBundleId else { return }
        currentBundleId = bundleId

        RustBridge.clearBuffer()

        guard AppState.shared.isSmartModeEnabled else { return }

        // Restore saved mode (default ON, only OFF apps are stored)
        let mode = AppState.shared.getPerAppMode(bundleId: bundleId)
        RustBridge.setEnabled(mode)
        AppState.shared.setEnabledSilently(mode)
    }
}

// MARK: - Notifications

extension Notification.Name {
    static let toggleVietnamese = Notification.Name("toggleVietnamese")
    static let showUpdateWindow = Notification.Name("showUpdateWindow")
    static let shortcutChanged = Notification.Name("shortcutChanged")
    static let updateStateChanged = Notification.Name("updateStateChanged")
    static let shortcutRecorded = Notification.Name("shortcutRecorded")
    static let shortcutRecordingCancelled = Notification.Name("shortcutRecordingCancelled")
}
