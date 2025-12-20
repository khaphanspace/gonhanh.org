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
    case autocomplete   // Spotlight fallback: Forward Delete + backspace + text via proxy
    case selectAll      // Select All + Replace: Cmd+A + type full buffer (for autocomplete apps)
    case axDirect       // Spotlight primary: AX API direct text manipulation (macOS 13+)
}

// MARK: - Text Injector

/// Handles text injection with proper sequencing to prevent race conditions
private class TextInjector {
    static let shared = TextInjector()

    /// Semaphore to block keyboard callback until injection completes
    private let semaphore = DispatchSemaphore(value: 1)

    /// Session buffer for selectAll method - tracks full text for Cmd+A replacement
    private var sessionBuffer: String = ""

    private init() {}

    /// Update session buffer with new composed text
    /// Called before injection to track full session text
    func updateSessionBuffer(backspace: Int, newText: String) {
        if backspace > 0 && sessionBuffer.count >= backspace {
            sessionBuffer.removeLast(backspace)
        }
        sessionBuffer.append(newText)
    }

    /// Clear session buffer (call on focus change, submit, etc.)
    func clearSessionBuffer() {
        sessionBuffer = ""
    }

    /// Set session buffer to specific value (for restoring after paste, etc.)
    func setSessionBuffer(_ text: String) {
        sessionBuffer = text
    }

    /// Get current session buffer
    func getSessionBuffer() -> String {
        return sessionBuffer
    }

    /// Inject selectAll only (session buffer already updated)
    func injectSelectAllOnly(proxy: CGEventTapProxy) {
        semaphore.wait()
        defer { semaphore.signal() }

        injectViaSelectAll(proxy: proxy)
        usleep(5000)  // Settle time
    }

    /// Inject text replacement synchronously (blocks until complete)
    func injectSync(bs: Int, text: String, method: InjectionMethod, delays: (UInt32, UInt32, UInt32), proxy: CGEventTapProxy) {
        semaphore.wait()
        defer { semaphore.signal() }

        // Update session buffer for selectAll method
        if method == .selectAll {
            updateSessionBuffer(backspace: bs, newText: text)
        }

        switch method {
        case .selection:
            injectViaSelection(bs: bs, text: text, delays: delays)
        case .autocomplete:
            injectViaAutocomplete(bs: bs, text: text, proxy: proxy)
        case .axDirect:
            injectViaAXWithFallback(bs: bs, text: text, proxy: proxy)
        case .selectAll:
            injectViaSelectAll(proxy: proxy)
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
    /// For backspace-only (text empty): use backspace to properly delete spaces/punctuation
    /// For text replacement: use Shift+Left to select (normal behavior)
    private func injectViaSelection(bs: Int, text: String, delays: (UInt32, UInt32, UInt32)) {
        guard let src = CGEventSource(stateID: .privateState) else { return }

        let selDelay = delays.0 > 0 ? delays.0 : 1000
        let waitDelay = delays.1 > 0 ? delays.1 : 3000
        let textDelay = delays.2 > 0 ? delays.2 : 2000

        if bs > 0 {
            // If text is empty (backspace-only, no replacement), use backspace to properly delete spaces/punctuation
            // This fixes issue where Shift+Left selects space instead of deleting it
            if text.isEmpty {
                // Backspace-only: use backspace for all deletions
                for _ in 0..<bs {
                    postKey(KeyCode.backspace, source: src)
                    usleep(selDelay)
                }
            } else {
                // Text replacement: use Shift+Left to select (normal selection method)
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

    /// Autocomplete injection: Forward Delete to clear suggestion, then backspace + text via proxy
    /// Used for Spotlight where autocomplete auto-selects suggestion text after cursor
    private func injectViaAutocomplete(bs: Int, text: String, proxy: CGEventTapProxy) {
        guard let src = CGEventSource(stateID: .privateState) else { return }

        // Forward Delete clears auto-selected suggestion
        postKey(KeyCode.forwardDelete, source: src, proxy: proxy)
        usleep(3000)

        // Backspaces remove typed characters
        for _ in 0..<bs {
            postKey(KeyCode.backspace, source: src, proxy: proxy)
            usleep(1000)
        }
        if bs > 0 { usleep(5000) }

        // Type replacement text
        postText(text, source: src, proxy: proxy)
        Log.send("auto", bs, text)
    }

    /// Select All injection: Select all text then type full session buffer
    /// Used for apps with aggressive autocomplete (Arc, Spotlight on macOS 13)
    /// Session buffer tracks ALL text typed in this session, not just current word
    private func injectViaSelectAll(proxy: CGEventTapProxy) {
        guard let src = CGEventSource(stateID: .privateState) else { return }

        // Get full session buffer (all text typed in this session)
        let fullText = sessionBuffer
        guard !fullText.isEmpty else { return }

        // Select all using Cmd+Left (home) + Shift+Cmd+Right (select to end)
        // This works better in Arc browser than Cmd+A
        postKey(KeyCode.leftArrow, source: src, flags: .maskCommand, proxy: proxy)  // Cmd+Left = Home
        usleep(5000)
        postKey(0x7C, source: src, flags: [.maskCommand, .maskShift], proxy: proxy)  // Shift+Cmd+Right = Select to end
        usleep(5000)

        // Type full session buffer (replaces all selected text)
        postText(fullText, source: src, proxy: proxy)
        Log.send("selAll", 0, fullText)
    }

    /// AX API injection: Directly manipulate text field via Accessibility API
    /// Used for Spotlight/Arc where synthetic keyboard events are unreliable due to autocomplete
    /// Returns true if successful, false if caller should fallback to synthetic events
    func injectViaAX(bs: Int, text: String) -> Bool {
        // Get focused element
        let systemWide = AXUIElementCreateSystemWide()
        var focusedRef: CFTypeRef?
        guard AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focusedRef) == .success,
              let ref = focusedRef else {
            Log.info("AX: no focus")
            return false
        }
        let axEl = ref as! AXUIElement

        // Read current text value
        var valueRef: CFTypeRef?
        guard AXUIElementCopyAttributeValue(axEl, kAXValueAttribute as CFString, &valueRef) == .success else {
            Log.info("AX: no value")
            return false
        }
        let fullText = (valueRef as? String) ?? ""

        // Read cursor position and selection
        var rangeRef: CFTypeRef?
        guard AXUIElementCopyAttributeValue(axEl, kAXSelectedTextRangeAttribute as CFString, &rangeRef) == .success,
              let axRange = rangeRef else {
            Log.info("AX: no range")
            return false
        }
        var range = CFRange()
        guard AXValueGetValue(axRange as! AXValue, .cfRange, &range), range.location >= 0 else {
            Log.info("AX: bad range")
            return false
        }

        let cursor = range.location
        let selection = range.length

        // Handle autocomplete: when selection > 0, text after cursor is autocomplete suggestion
        // Example: "a|rc://chrome-urls" where "|" is cursor, "rc://..." is selected suggestion
        let userText = (selection > 0 && cursor <= fullText.count)
            ? String(fullText.prefix(cursor))
            : fullText

        // Calculate replacement: delete `bs` chars before cursor, insert `text`
        let deleteStart = max(0, cursor - bs)
        let prefix = String(userText.prefix(deleteStart))
        let suffix = String(userText.dropFirst(cursor))
        let newText = (prefix + text + suffix).precomposedStringWithCanonicalMapping

        // Write new value
        guard AXUIElementSetAttributeValue(axEl, kAXValueAttribute as CFString, newText as CFTypeRef) == .success else {
            Log.info("AX: write failed")
            return false
        }

        // Update cursor to end of inserted text
        var newCursor = CFRange(location: deleteStart + text.count, length: 0)
        if let newRange = AXValueCreate(.cfRange, &newCursor) {
            AXUIElementSetAttributeValue(axEl, kAXSelectedTextRangeAttribute as CFString, newRange)
        }

        Log.send("ax", bs, text)
        return true
    }

    /// Try AX injection with retries, fallback to synthetic events if all fail
    /// Spotlight can be busy searching, causing AX API to fail temporarily
    func injectViaAXWithFallback(bs: Int, text: String, proxy: CGEventTapProxy) {
        // Try AX API up to 3 times (Spotlight might be busy)
        for attempt in 0..<3 {
            if attempt > 0 {
                usleep(5000)  // 5ms delay before retry
            }
            if injectViaAX(bs: bs, text: text) {
                return  // Success!
            }
        }

        // All AX attempts failed - fallback to autocomplete method
        Log.info("AX: fallback to autocomplete")
        injectViaAutocomplete(bs: bs, text: text, proxy: proxy)
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
@_silgen_name("ime_esc_restore") private func ime_esc_restore(_ enabled: Bool)
@_silgen_name("ime_free_tone") private func ime_free_tone(_ enabled: Bool)
@_silgen_name("ime_modern") private func ime_modern(_ modern: Bool)
@_silgen_name("ime_english_auto_restore") private func ime_english_auto_restore(_ enabled: Bool)
@_silgen_name("ime_clear") private func ime_clear()
@_silgen_name("ime_clear_all") private func ime_clear_all()
@_silgen_name("ime_free") private func ime_free(_ result: UnsafeMutablePointer<ImeResult>?)

// Shortcut FFI
@_silgen_name("ime_add_shortcut") private func ime_add_shortcut(_ trigger: UnsafePointer<CChar>?, _ replacement: UnsafePointer<CChar>?)
@_silgen_name("ime_remove_shortcut") private func ime_remove_shortcut(_ trigger: UnsafePointer<CChar>?)
@_silgen_name("ime_clear_shortcuts") private func ime_clear_shortcuts()

// Word Restore FFI
@_silgen_name("ime_restore_word") private func ime_restore_word(_ word: UnsafePointer<CChar>?)

// Buffer FFI (for Select All method)
@_silgen_name("ime_get_buffer") private func ime_get_buffer(_ out: UnsafeMutablePointer<UInt32>, _ maxLen: Int) -> Int

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

    /// Set whether ESC key restores raw ASCII input
    static func setEscRestore(_ enabled: Bool) {
        ime_esc_restore(enabled)
        Log.info("ESC restore: \(enabled)")
    }

    /// Set whether to enable free tone placement (skip validation)
    static func setFreeTone(_ enabled: Bool) {
        ime_free_tone(enabled)
        Log.info("Free tone: \(enabled)")
    }

    /// Set whether to use modern orthography for tone placement
    static func setModernTone(_ modern: Bool) {
        ime_modern(modern)
        Log.info("Modern tone: \(modern)")
    }

    /// Set whether to enable English auto-restore (experimental)
    /// When enabled, automatically restores English words that were transformed
    static func setEnglishAutoRestore(_ enabled: Bool) {
        ime_english_auto_restore(enabled)
        Log.info("English auto-restore: \(enabled)")
    }

    static func clearBuffer() { ime_clear() }

    /// Clear buffer and word history (use on mouse click, focus change)
    static func clearBufferAll() { ime_clear_all() }

    /// Get full composed buffer as string (for Select All injection method)
    static func getFullBuffer() -> String {
        var buffer = [UInt32](repeating: 0, count: 64)
        let len = ime_get_buffer(&buffer, 64)
        guard len > 0 else { return "" }
        return String(buffer[0..<len].compactMap { Unicode.Scalar($0).map(Character.init) })
    }

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
    private var mouseMonitor: Any?  // NSEvent monitor for mouse clicks
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

        // Listen for keyboard events only (mouse handled by NSEvent monitor)
        let mask: CGEventMask = (1 << CGEventType.keyDown.rawValue) |
                                (1 << CGEventType.flagsChanged.rawValue)
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
            startMouseMonitor()
            Log.info("Hook started")
        }
    }

    /// Start NSEvent global monitor for mouse events
    /// This is more reliable than CGEventTap for detecting mouse clicks
    private func startMouseMonitor() {
        // Monitor both mouseDown and mouseUp to catch clicks and drag-selects
        mouseMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .leftMouseUp]) { _ in
            TextInjector.shared.clearSessionBuffer()
            RustBridge.clearBufferAll()  // Clear everything including word history
            skipWordRestoreAfterClick = true
            Log.info("Mouse event: cleared buffer, skip restore = true")
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
private var recordingModifiers: CGEventFlags = []      // Current modifiers being held
private var peakRecordingModifiers: CGEventFlags = []  // Peak modifiers during recording
private var shortcutObserver: NSObjectProtocol?
/// Skip word restore after mouse click (user may be selecting/deleting text)
/// Reset to false after first keystroke
private var skipWordRestoreAfterClick = false

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

func startShortcutRecording() {
    isRecordingShortcut = true
    recordingModifiers = []
    peakRecordingModifiers = []
}

func stopShortcutRecording() {
    isRecordingShortcut = false
    recordingModifiers = []
    peakRecordingModifiers = []
}

func setupShortcutObserver() {
    shortcutObserver = NotificationCenter.default.addObserver(forName: .shortcutChanged, object: nil, queue: .main) { _ in
        currentShortcut = KeyboardShortcut.load()
        Log.info("Shortcut updated: \(currentShortcut.displayParts.joined())")
    }
}

private func matchesToggleShortcut(keyCode: UInt16, flags: CGEventFlags) -> Bool {
    return currentShortcut.matches(keyCode: keyCode, flags: flags)
}

private func matchesModifierOnlyShortcut(flags: CGEventFlags) -> Bool {
    return currentShortcut.matchesModifierOnly(flags: flags)
}

private func keyboardCallback(
    proxy: CGEventTapProxy, type: CGEventType, event: CGEvent, refcon: UnsafeMutableRawPointer?
) -> Unmanaged<CGEvent>? {

    if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
        if let tap = KeyboardHookManager.shared.getTap() { CGEvent.tapEnable(tap: tap, enable: true) }
        return Unmanaged.passUnretained(event)
    }

    // Check for special panel apps (Spotlight, Raycast) on keyboard events
    // These apps don't trigger NSWorkspaceDidActivateApplicationNotification
    if type == .keyDown || type == .keyUp {
        DispatchQueue.main.async {
            PerAppModeManager.shared.checkSpecialPanelApp()
        }
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

        // Modifier changes: track peak modifiers and save on full release
        if type == .flagsChanged {
            if mods.isEmpty && peakRecordingModifiers.modifierCount >= 2 {
                // All modifiers released - save using peak modifiers (requires 2+ modifiers)
                let captured = KeyboardShortcut(keyCode: 0xFFFF, modifiers: peakRecordingModifiers.rawValue)
                stopShortcutRecording()
                DispatchQueue.main.async { NotificationCenter.default.post(name: .shortcutRecorded, object: captured) }
            } else {
                recordingModifiers = mods
                // Track peak: update if current has more modifiers
                if mods.modifierCount > peakRecordingModifiers.modifierCount {
                    peakRecordingModifiers = mods
                }
            }
            return Unmanaged.passUnretained(event)
        }

        // Key + modifier: save shortcut (e.g., Ctrl+N, Cmd+Shift+N)
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

    // Clear session buffer on Enter/Escape (submit or cancel)
    if keyCode == 0x24 || keyCode == 0x35 {  // Enter or Escape
        TextInjector.shared.clearSessionBuffer()
        RustBridge.clearBuffer()
        return Unmanaged.passUnretained(event)
    }

    // Detect injection method once per keystroke (expensive AX query)
    let (method, delays) = detectMethod()

    // Pass through all Cmd+key shortcuts (Cmd+A, Cmd+C, Cmd+V, Cmd+X, Cmd+Z, etc.)
    // For selectAll method: sync session buffer after text-modifying shortcuts
    if flags.contains(.maskCommand) && !flags.contains(.maskControl) && !flags.contains(.maskAlternate) {

        // Shortcuts that modify text content
        let textModifyingKeys: Set<UInt16> = [
            0x00,  // Cmd+A (select all)
            0x09,  // Cmd+V (paste)
            0x07,  // Cmd+X (cut)
            0x06,  // Cmd+Z (undo)
        ]

        if textModifyingKeys.contains(keyCode) {
            RustBridge.clearBuffer()

            if method == .selectAll {
                if keyCode == 0x00 {
                    // Cmd+A: clear session buffer, let next backspace pass through to delete selection
                    TextInjector.shared.clearSessionBuffer()
                } else {
                    // Cmd+V, Cmd+X, Cmd+Z: sync session buffer from field after action completes
                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                        if let text = getTextFromFocusedElement() {
                            TextInjector.shared.setSessionBuffer(text)
                            Log.info("Session buffer synced: \(text)")
                        } else {
                            TextInjector.shared.clearSessionBuffer()
                        }
                    }
                }
            } else {
                TextInjector.shared.clearSessionBuffer()
            }
        }
        // Pass through all Cmd shortcuts
        return Unmanaged.passUnretained(event)
    }

    let shift = flags.contains(.maskShift)
    let caps = shift || flags.contains(.maskAlphaShift)
    let ctrl = flags.contains(.maskCommand) || flags.contains(.maskControl) || flags.contains(.maskAlternate)

    // Backspace handling: try to restore word from screen when backspacing into it
    // This enables editing marks on previously committed words
    if keyCode == KeyCode.backspace && !ctrl {
        // For selectAll method: handle backspace (only when enabled)
        if method == .selectAll && AppState.shared.isEnabled {
            let session = TextInjector.shared.getSessionBuffer()
            if !session.isEmpty {
                // Session has content - remove last char and re-inject
                TextInjector.shared.updateSessionBuffer(backspace: 1, newText: "")
                TextInjector.shared.injectSelectAllOnly(proxy: proxy)
                return nil
            } else {
                // Session is empty (after Cmd+A, etc.) - pass through backspace to delete selection
                Log.info("selectAll: pass through backspace (empty session)")
                return Unmanaged.passUnretained(event)
            }
        }

        // First try Rust engine (handles immediate backspace-after-space)
        if let (bs, chars) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
            let str = String(chars)
            Log.transform(bs, str)
            sendReplacement(backspace: bs, chars: chars, method: method, delays: delays, proxy: proxy)
            return nil
        }

        // Engine returned none - try to restore word from screen
        // This handles: "chào " + backspace to delete space and enter word
        // Skip restore if just clicked (user may be deleting a selection)
        if !skipWordRestoreAfterClick, let word = getWordToRestoreOnBackspace() {
            RustBridge.restoreWord(word)
            Log.info("Restored word from screen: \(word)")
        }
        // Don't reset skipWordRestoreAfterClick here - keep skipping until a real letter is typed

        // Pass through backspace to delete the character
        return Unmanaged.passUnretained(event)
    }

    // Reset skip flag only when a real letter key is pressed (not backspace/delete/modifiers)
    // This ensures we skip word restore for ALL backspaces after a mouse click
    let isLetterKey = keyCode <= 0x32 && keyCode != KeyCode.backspace  // Rough check for letter keys
    if isLetterKey {
        skipWordRestoreAfterClick = false
    }

    if let (bs, chars) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
        let str = String(chars)
        Log.transform(bs, str)
        sendReplacement(backspace: bs, chars: chars, method: method, delays: delays, proxy: proxy)
        return nil
    }

    // For selectAll method: handle pass-through keys (space, punctuation, etc.)
    // These need to be appended to session buffer and trigger Cmd+A replacement
    if method == .selectAll && AppState.shared.isEnabled {
        // Convert keyCode to character
        if let char = keyCodeToChar(keyCode: keyCode, shift: shift) {
            TextInjector.shared.updateSessionBuffer(backspace: 0, newText: String(char))
            TextInjector.shared.injectSelectAllOnly(proxy: proxy)
            return nil
        }
    }

    // Debug: log frontmost app for all keystrokes
    if let app = NSWorkspace.shared.frontmostApplication {
        Log.info("frontmost: \(app.bundleIdentifier ?? "nil")")
    }
    Log.key(keyCode, "pass")
    return Unmanaged.passUnretained(event)
}

// MARK: - Helper Functions

/// Get text value from focused element (for syncing session buffer after paste)
private func getTextFromFocusedElement() -> String? {
    let systemWide = AXUIElementCreateSystemWide()
    var focused: CFTypeRef?

    guard AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focused) == .success,
          let el = focused else {
        return nil
    }

    let axEl = el as! AXUIElement

    // Get text value
    var textValue: CFTypeRef?
    guard AXUIElementCopyAttributeValue(axEl, kAXValueAttribute as CFString, &textValue) == .success,
          let text = textValue as? String else {
        return nil
    }

    return text
}

/// Convert keyCode to character (for pass-through keys in selectAll mode)
private func keyCodeToChar(keyCode: UInt16, shift: Bool) -> Character? {
    // Full keyCode to character mapping for selectAll mode
    let keyMap: [UInt16: (normal: Character, shifted: Character)] = [
        // Letters
        0x00: ("a", "A"), 0x0B: ("b", "B"), 0x08: ("c", "C"), 0x02: ("d", "D"),
        0x0E: ("e", "E"), 0x03: ("f", "F"), 0x05: ("g", "G"), 0x04: ("h", "H"),
        0x22: ("i", "I"), 0x26: ("j", "J"), 0x28: ("k", "K"), 0x25: ("l", "L"),
        0x2E: ("m", "M"), 0x2D: ("n", "N"), 0x1F: ("o", "O"), 0x23: ("p", "P"),
        0x0C: ("q", "Q"), 0x0F: ("r", "R"), 0x01: ("s", "S"), 0x11: ("t", "T"),
        0x20: ("u", "U"), 0x09: ("v", "V"), 0x0D: ("w", "W"), 0x07: ("x", "X"),
        0x10: ("y", "Y"), 0x06: ("z", "Z"),
        // Numbers
        0x12: ("1", "!"), 0x13: ("2", "@"), 0x14: ("3", "#"), 0x15: ("4", "$"),
        0x17: ("5", "%"), 0x16: ("6", "^"), 0x1A: ("7", "&"), 0x1C: ("8", "*"),
        0x19: ("9", "("), 0x1D: ("0", ")"),
        // Punctuation
        0x31: (" ", " "),      // Space
        0x2B: (",", "<"),      // Comma
        0x2F: (".", ">"),      // Period
        0x2C: ("/", "?"),      // Slash
        0x27: ("'", "\""),     // Quote
        0x29: (";", ":"),      // Semicolon
        0x1E: ("]", "}"),      // Right bracket
        0x21: ("[", "{"),      // Left bracket
        0x2A: ("\\", "|"),     // Backslash
        0x18: ("=", "+"),      // Equal
        0x1B: ("-", "_"),      // Minus
        0x32: ("`", "~"),      // Grave
    ]

    if let chars = keyMap[keyCode] {
        return shift ? chars.shifted : chars.normal
    }
    return nil
}

// MARK: - Text Replacement

private func detectMethod() -> (InjectionMethod, (UInt32, UInt32, UInt32)) {
    // Get focused element and its owning app (works for overlays like Spotlight)
    let systemWide = AXUIElementCreateSystemWide()
    var focused: CFTypeRef?
    var role: String?
    var bundleId: String?

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
            }
        }
    }

    // Fallback to frontmost app if we couldn't get bundle from focused element
    if bundleId == nil {
        bundleId = NSWorkspace.shared.frontmostApplication?.bundleIdentifier
    }

    guard let bundleId = bundleId else { return (.fast, (200, 800, 500)) }

    // Debug: log bundle and role for investigation
    Log.info("detect: \(bundleId) role=\(role ?? "nil")")

    // Selection method for autocomplete UI elements (ComboBox, SearchField)
    if role == "AXComboBox" { Log.method("sel:combo"); return (.selection, (0, 0, 0)) }
    if role == "AXSearchField" { Log.method("sel:search"); return (.selection, (0, 0, 0)) }

    // Spotlight - use AX API direct manipulation (works on macOS 13+)
    // This bypasses Spotlight's autocomplete behavior by directly setting text field value
    // Note: Spotlight can run under com.apple.systemuiserver in some cases
    if bundleId == "com.apple.Spotlight" || bundleId == "com.apple.systemuiserver" {
        Log.method("ax:spotlight")
        return (.axDirect, (0, 0, 0))
    }

    // Arc browser - use AX API for address bar (same approach as Spotlight)
    // Arc is Chromium-based with good accessibility support
    if bundleId == "company.thebrowser.Browser" && role == "AXTextField" {
        Log.method("ax:arc")
        return (.axDirect, (0, 0, 0))
    }

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

private func sendReplacement(backspace bs: Int, chars: [Character], method: InjectionMethod, delays: (UInt32, UInt32, UInt32), proxy: CGEventTapProxy) {
    let str = String(chars)

    // Use TextInjector for synchronized text injection
    TextInjector.shared.injectSync(bs: bs, text: str, method: method, delays: delays, proxy: proxy)
}

// MARK: - Per-App Mode Manager

class PerAppModeManager {
    static let shared = PerAppModeManager()

    private var currentBundleId: String?
    private var observer: NSObjectProtocol?
    private var mouseClickMonitor: Any?

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
            // Update SpecialPanelAppDetector's last frontmost app
            SpecialPanelAppDetector.updateLastFrontMostApp(bundleId)
            self?.handleAppSwitch(bundleId)
        }
        
        // Monitor mouse clicks to detect special panel apps (Spotlight, Raycast)
        // These apps don't trigger NSWorkspaceDidActivateApplicationNotification
        mouseClickMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak self] _ in
            self?.checkSpecialPanelApp()
        }
        
        Log.info("PerAppModeManager started")
    }

    /// Stop observing
    func stop() {
        if let observer = observer {
            NSWorkspace.shared.notificationCenter.removeObserver(observer)
            self.observer = nil
        }
        if let monitor = mouseClickMonitor {
            NSEvent.removeMonitor(monitor)
            mouseClickMonitor = nil
        }
    }

    /// Check for special panel apps on keyboard events
    /// Call this from the keyboard callback
    func checkSpecialPanelApp() {
        guard AppState.shared.isSmartModeEnabled else { return }
        
        let (appChanged, newBundleId, _) = SpecialPanelAppDetector.checkForAppChange()
        
        if appChanged, let bundleId = newBundleId {
            handleAppSwitch(bundleId)
        }
    }

    private func handleAppSwitch(_ bundleId: String) {
        guard bundleId != currentBundleId else { return }
        currentBundleId = bundleId

        RustBridge.clearBuffer()
        TextInjector.shared.clearSessionBuffer()

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
