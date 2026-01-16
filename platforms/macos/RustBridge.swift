import Foundation
import Carbon
import AppKit

// MARK: - Debug Logging

/// Debug logging - only active when /tmp/gonhanh_debug.log exists
/// Enable: touch /tmp/gonhanh_debug.log | Disable: rm /tmp/gonhanh_debug.log
/// PERFORMANCE: isEnabled cached, @autoclosure defers string formatting until needed
private enum Log {
    private static let logPath = "/tmp/gonhanh_debug.log"
    private static var _enabled: Bool?
    static var isEnabled: Bool {
        if let cached = _enabled { return cached }
        _enabled = FileManager.default.fileExists(atPath: logPath)
        return _enabled!
    }

    /// Call to refresh enabled state (e.g., on app activation)
    static func refresh() { _enabled = nil }

    private static func write(_ msg: @autoclosure () -> String) {
        guard isEnabled, let handle = FileHandle(forWritingAtPath: logPath) else { return }
        let now = CFAbsoluteTimeGetCurrent()
        let secs = Int(now) % 86400
        let ms = Int((now - floor(now)) * 1000)
        let ts = String(format: "%02d:%02d:%02d.%03d", secs / 3600, (secs / 60) % 60, secs % 60, ms)
        handle.seekToEndOfFile()
        handle.write("[\(ts)] \(msg())\n".data(using: .utf8)!)
        handle.closeFile()
    }

    static func key(_ code: UInt16, _ result: @autoclosure () -> String) { guard isEnabled else { return }; write("K:\(code) → \(result())") }
    static func method(_ name: @autoclosure () -> String) { guard isEnabled else { return }; write("M: \(name())") }
    static func info(_ msg: @autoclosure () -> String) { guard isEnabled else { return }; write("I: \(msg())") }
    static func queue(_ msg: @autoclosure () -> String) { guard isEnabled else { return }; write("Q: \(msg())") }
}

// MARK: - Constants

private enum KeyCode {
    // Navigation keys
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

    // Punctuation keys
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

    // Number keys (shifted = !@#$%^&*())
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

/// Check if key is a break key (space, punctuation, arrows, etc.)
/// When shift=true, also treat number keys as break (they produce !@#$%^&*())
private func isBreakKey(_ keyCode: CGKeyCode, shift: Bool) -> Bool {
    // Standard break keys: space, tab, return, arrows, punctuation
    let standardBreak: Set<CGKeyCode> = [
        KeyCode.space, KeyCode.tab, KeyCode.returnKey, KeyCode.enter, KeyCode.esc,
        KeyCode.leftArrow, KeyCode.rightArrow, KeyCode.upArrow, KeyCode.downArrow,
        KeyCode.dot, KeyCode.comma, KeyCode.slash, KeyCode.semicolon, KeyCode.quote,
        KeyCode.lbracket, KeyCode.rbracket, KeyCode.backslash, KeyCode.minus,
        KeyCode.equal, KeyCode.backquote
    ]

    if standardBreak.contains(keyCode) { return true }

    // Shifted number keys produce symbols: !@#$%^&*()
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
    case fast           // Default: backspace + text with minimal delays
    case slow           // Terminals/Electron: backspace + text with higher delays
    case charByChar     // Safari Google Docs: backspace + text character-by-character
    case selection      // Browser address bars: Shift+Left select + type replacement
    case autocomplete   // Spotlight fallback: Forward Delete + backspace + text via proxy
    case selectAll      // Select All + Replace: Cmd+A + type full buffer (for autocomplete apps)
    case axDirect       // Spotlight primary: AX API direct text manipulation (macOS 13+)
    case passthrough    // iPhone Mirroring: pass through all keys (remote device handles input)
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

    /// Post break key (Enter, punctuation) synthetically after text injection
    /// Used for auto-restore to ensure correct event ordering
    func postBreakKey(keyCode: CGKeyCode, shift: Bool) {
        guard let src = CGEventSource(stateID: .privateState) else { return }
        let flags: CGEventFlags = shift ? .maskShift : []
        postKey(keyCode, source: src, flags: flags)
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
        case .charByChar:
            injectViaBackspace(bs: bs, text: text, delays: delays, charByChar: true)
        case .slow, .fast:
            injectViaBackspace(bs: bs, text: text, delays: delays)
        case .passthrough:
            // Should not reach here - passthrough is handled in keyboard callback
            break
        }

        // Settle time: 20ms for slow apps, 5ms for others
        usleep(method == .slow ? 20000 : 5000)
    }

    // MARK: - Injection Methods

    /// Standard backspace injection: delete N chars, then type replacement
    /// Set charByChar=true for character-by-character mode (slower but more reliable for some apps like Safari Google Docs)
    private func injectViaBackspace(bs: Int, text: String, delays: (UInt32, UInt32, UInt32), charByChar: Bool = false) {
        guard let src = CGEventSource(stateID: .privateState) else { return }

        for _ in 0..<bs {
            postKey(KeyCode.backspace, source: src)
            usleep(delays.0)
        }
        if bs > 0 { usleep(delays.1) }

        postText(text, source: src, delay: delays.2, chunkSize: charByChar ? 1 : 20)
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
        // NOTE: AX API returns UTF-16 offsets, so we must use utf16 view for all position calculations
        let utf16View = fullText.utf16
        let cursorUTF16 = min(cursor, utf16View.count)

        let userText: String
        if selection > 0 && cursorUTF16 <= utf16View.count {
            let endIdx = utf16View.index(utf16View.startIndex, offsetBy: cursorUTF16)
            userText = String(fullText[..<endIdx])
        } else {
            userText = fullText
        }

        // Calculate replacement: delete `bs` chars before cursor, insert `text`
        // IMPORTANT: cursor and bs are UTF-16 offsets, not grapheme cluster counts
        let userUTF16 = userText.utf16
        let deleteStartUTF16 = max(0, cursorUTF16 - bs)

        // Convert UTF-16 offsets to String.Index
        let prefixEndIdx = userUTF16.index(userUTF16.startIndex, offsetBy: min(deleteStartUTF16, userUTF16.count))
        let suffixStartIdx = userUTF16.index(userUTF16.startIndex, offsetBy: min(cursorUTF16, userUTF16.count))

        let prefix = String(userText[..<prefixEndIdx])
        let suffix = String(userText[suffixStartIdx...])
        let newText = (prefix + text + suffix).precomposedStringWithCanonicalMapping

        // Write new value
        guard AXUIElementSetAttributeValue(axEl, kAXValueAttribute as CFString, newText as CFTypeRef) == .success else {
            Log.info("AX: write failed")
            return false
        }

        // Update cursor to end of inserted text (use UTF-16 offset)
        var newCursor = CFRange(location: deleteStartUTF16 + text.utf16.count, length: 0)
        if let newRange = AXValueCreate(.cfRange, &newCursor) {
            AXUIElementSetAttributeValue(axEl, kAXSelectedTextRangeAttribute as CFString, newRange)
        }

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
    /// Set chunkSize=1 for character-by-character mode (slower but more reliable for some apps)
    private func postText(_ text: String, source: CGEventSource, delay: UInt32 = 0, proxy: CGEventTapProxy? = nil, chunkSize: Int = 20) {
        let utf16 = Array(text.utf16)
        var offset = 0

        while offset < utf16.count {
            let end = min(offset + chunkSize, utf16.count)
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
/// Size: 256 UInt32 chars (1024 bytes) + 4 bytes = 1028 bytes
/// Max replacement: 255 UTF-32 codepoints (Vietnamese diacritics = 1 each)
private struct ImeResult {
    // 256 UInt32 values for UTF-32 codepoints (matches core/src/engine/buffer.rs MAX)
    // 32 lines × 8 values = 256 total
    var chars: (
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 1
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 2
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 3
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 4
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 5
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 6
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 7
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 8
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 9
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 10
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 11
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 12
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 13
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 14
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 15
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 16
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 17
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 18
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 19
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 20
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 21
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 22
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 23
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 24
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 25
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 26
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 27
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 28
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 29
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 30
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,  // 31
        UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32   // 32
    )
    var action: UInt8
    var backspace: UInt8
    var count: UInt8
    var flags: UInt8  // bit 0: key_consumed
}

private let FLAG_KEY_CONSUMED: UInt8 = 0x01  // Key was consumed by shortcut, don't pass through

@_silgen_name("ime_init") private func ime_init()
@_silgen_name("ime_key_ext") private func ime_key_ext(_ key: UInt16, _ caps: Bool, _ ctrl: Bool, _ shift: Bool) -> UnsafeMutablePointer<ImeResult>?
@_silgen_name("ime_method") private func ime_method(_ method: UInt8)
@_silgen_name("ime_enabled") private func ime_enabled(_ enabled: Bool)
@_silgen_name("ime_skip_w_shortcut") private func ime_skip_w_shortcut(_ skip: Bool)
@_silgen_name("ime_bracket_shortcut") private func ime_bracket_shortcut(_ enabled: Bool)
@_silgen_name("ime_esc_restore") private func ime_esc_restore(_ enabled: Bool)
@_silgen_name("ime_free_tone") private func ime_free_tone(_ enabled: Bool)
@_silgen_name("ime_modern") private func ime_modern(_ modern: Bool)
@_silgen_name("ime_english_auto_restore") private func ime_english_auto_restore(_ enabled: Bool)
@_silgen_name("ime_auto_capitalize") private func ime_auto_capitalize(_ enabled: Bool)
@_silgen_name("ime_allow_foreign_consonants") private func ime_allow_foreign_consonants(_ enabled: Bool)
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
    }

    /// Process a keystroke. Returns (backspace, chars, keyConsumed) or nil if no action.
    static func processKey(keyCode: UInt16, caps: Bool, ctrl: Bool, shift: Bool = false) -> (Int, [Character], Bool)? {
        guard isInitialized, let ptr = ime_key_ext(keyCode, caps, ctrl, shift) else { return nil }
        defer { ime_free(ptr) }

        let r = ptr.pointee
        guard r.action == 1 else { return nil }

        let chars = withUnsafePointer(to: r.chars) { p in
            p.withMemoryRebound(to: UInt32.self, capacity: 256) { bound in
                (0..<Int(r.count)).compactMap { Unicode.Scalar(bound[$0]).map(Character.init) }
            }
        }
        let keyConsumed = (r.flags & FLAG_KEY_CONSUMED) != 0
        return (Int(r.backspace), chars, keyConsumed)
    }

    static func setMethod(_ method: Int) { ime_method(UInt8(method)) }

    static func setEnabled(_ enabled: Bool) {
        // GATE: Only enable if input source is allowed, always allow disable
        ime_enabled(enabled && InputSourceObserver.shared.isAllowedInputSource)
    }

    /// Set whether to skip w→ư shortcut in Telex mode
    static func setSkipWShortcut(_ skip: Bool) { ime_skip_w_shortcut(skip) }

    /// Set whether bracket shortcuts are enabled: ] → ư, [ → ơ (Issue #159)
    static func setBracketShortcut(_ enabled: Bool) { ime_bracket_shortcut(enabled) }

    /// Set whether restore shortcut is enabled
    /// NOTE: Enable in Rust engine so we can get restore data, but Swift controls when to trigger
    static func setRestoreShortcutEnabled(_ enabled: Bool) { ime_esc_restore(enabled) }

    /// Set whether to enable free tone placement (skip validation)
    static func setFreeTone(_ enabled: Bool) { ime_free_tone(enabled) }

    /// Set whether to use modern orthography for tone placement
    static func setModernTone(_ modern: Bool) { ime_modern(modern) }

    /// Set whether to enable English auto-restore (experimental)
    /// When enabled, automatically restores English words that were transformed
    static func setEnglishAutoRestore(_ enabled: Bool) { ime_english_auto_restore(enabled) }

    /// Set whether to enable auto-capitalize after sentence-ending punctuation
    /// When enabled, capitalizes first letter after . ! ? Enter
    static func setAutoCapitalize(_ enabled: Bool) { ime_auto_capitalize(enabled) }

    /// Set whether to allow foreign consonants (z, w, j, f) as valid initials
    /// When enabled, these letters are accepted as Vietnamese consonants for loanwords
    static func setAllowForeignConsonants(_ enabled: Bool) {
        ime_allow_foreign_consonants(enabled)
        Log.info("Allow foreign consonants: \(enabled)")
    }

    static func clearBuffer() { ime_clear() }

    /// Clear buffer and word history (use on mouse click, focus change)
    static func clearBufferAll() { ime_clear_all() }

    /// Get full composed buffer as string (for Select All injection method)
    static func getFullBuffer() -> String {
        var buffer = [UInt32](repeating: 0, count: 256)
        let len = ime_get_buffer(&buffer, 256)
        guard len > 0 else { return "" }
        return String(buffer[0..<len].compactMap { Unicode.Scalar($0).map(Character.init) })
    }

    /// Restore buffer from a Vietnamese word (for backspace-into-word editing)
    static func restoreWord(_ word: String) {
        word.withCString { ime_restore_word($0) }
    }

    // MARK: - Shortcuts

    /// Add a shortcut to the engine
    static func addShortcut(trigger: String, replacement: String) {
        trigger.withCString { t in
            replacement.withCString { r in ime_add_shortcut(t, r) }
        }
    }

    /// Remove a shortcut from the engine
    static func removeShortcut(trigger: String) {
        trigger.withCString { ime_remove_shortcut($0) }
    }

    /// Clear all shortcuts from the engine
    static func clearShortcuts() { ime_clear_shortcuts() }

    /// Sync shortcuts from UI to engine
    static func syncShortcuts(_ shortcuts: [(key: String, value: String, enabled: Bool)]) {
        ime_clear_shortcuts()
        for shortcut in shortcuts where shortcut.enabled {
            addShortcut(trigger: shortcut.key, replacement: shortcut.value)
        }
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
private let kModifierMask: CGEventFlags = [.maskSecondaryFn, .maskControl, .maskAlternate, .maskShift, .maskCommand]
private var wasModifierShortcutPressed = false
private var wasRestoreModifierPressed = false  // Track modifier-only restore shortcut
private var currentShortcut = KeyboardShortcut.load()
private var currentRestoreShortcut = KeyboardShortcut.loadRestoreShortcut()
private var isRecordingShortcut = false
private var recordingModifiers: CGEventFlags = []      // Current modifiers being held
private var peakRecordingModifiers: CGEventFlags = []  // Peak modifiers during recording
private var shortcutObserver: NSObjectProtocol?
/// Skip word restore after mouse click (user may be selecting/deleting text)
/// Reset to false after first keystroke
private var skipWordRestoreAfterClick = false
/// Track Control key state to detect keydown for buffer clearing (Issue #150)
private var wasControlPressed = false

// MARK: - Word Restore Support

/// Get word that we're about to backspace into
/// Returns the word only if cursor is right after a space/punctuation that follows a word
/// This ensures we only restore when actually entering a word, not when deleting within a word
private func getWordToRestoreOnBackspace() -> String? {
    let systemWide = AXUIElementCreateSystemWide()
    var focused: CFTypeRef?

    guard AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focused) == .success,
          let el = focused else { return nil }
    // swiftlint:disable:next force_cast
    let axEl = el as! AXUIElement

    // Get text value
    var textValue: CFTypeRef?
    guard AXUIElementCopyAttributeValue(axEl, kAXValueAttribute as CFString, &textValue) == .success,
          let text = textValue as? String, !text.isEmpty else { return nil }

    // Get selected text range (cursor position)
    var rangeValue: CFTypeRef?
    guard AXUIElementCopyAttributeValue(axEl, kAXSelectedTextRangeAttribute as CFString, &rangeValue) == .success,
          let rv = rangeValue else { return nil }
    // swiftlint:disable:next force_cast
    let rangeRef = rv as! AXValue

    // Extract range from AXValue
    var range = CFRange(location: 0, length: 0)
    guard AXValueGetValue(rangeRef, .cfRange, &range) else { return nil }

    let cursorPos = range.location
    guard cursorPos > 0 else { return nil }

    let textChars = Array(text)
    guard cursorPos <= textChars.count else { return nil }

    let charBeforeCursor = textChars[cursorPos - 1]

    // Only restore if we're about to delete the LAST space/punctuation before a word
    // i.e., cursor is at: "word |" (about to delete space and enter "word")
    guard charBeforeCursor.isWhitespace || charBeforeCursor.isPunctuation else { return nil }

    // Check if there's a word before this space (not more spaces)
    var wordEnd = cursorPos - 1

    // Skip all trailing spaces/punctuation to find the word
    while wordEnd > 0 && (textChars[wordEnd - 1].isWhitespace || textChars[wordEnd - 1].isPunctuation) {
        wordEnd -= 1
    }

    guard wordEnd > 0 else { return nil }

    // But we only want to restore when deleting THE LAST space before the word
    // If there are more spaces between cursor and word, don't restore yet
    if wordEnd < cursorPos - 1 { return nil }  // More than one space/punct between cursor and word

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

    return (hasVietnameseDiacritics || isPureASCIILetters) ? word : nil
}

private extension CGEventFlags {
    var modifierCount: Int {
        [contains(.maskSecondaryFn), contains(.maskControl), contains(.maskAlternate), contains(.maskShift), contains(.maskCommand)].filter { $0 }.count
    }
    
    /// Check if only fn key is pressed (no other modifiers)
    var isFnOnly: Bool {
        contains(.maskSecondaryFn) && 
        !contains(.maskControl) && 
        !contains(.maskAlternate) && 
        !contains(.maskShift) && 
        !contains(.maskCommand)
    }
}

// MARK: - Shortcut Recording

private var isRecordingRestoreShortcut = false  // Separate flag for restore shortcut recording

func startShortcutRecording() {
    guard !isRecordingShortcut else { return }  // Prevent simultaneous recording
    isRecordingShortcut = true
    isRecordingRestoreShortcut = false
    recordingModifiers = []
    peakRecordingModifiers = []
}

func stopShortcutRecording() {
    isRecordingShortcut = false
    isRecordingRestoreShortcut = false
    recordingModifiers = []
    peakRecordingModifiers = []
}

func startRestoreShortcutRecording() {
    guard !isRecordingShortcut else { return }  // Prevent simultaneous recording
    isRecordingShortcut = true
    isRecordingRestoreShortcut = true
    recordingModifiers = []
    peakRecordingModifiers = []
}

func setupShortcutObserver() {
    shortcutObserver = NotificationCenter.default.addObserver(forName: .shortcutChanged, object: nil, queue: .main) { _ in
        currentShortcut = KeyboardShortcut.load()
    }
    // Observer for restore shortcut changes
    NotificationCenter.default.addObserver(forName: .restoreShortcutChanged, object: nil, queue: .main) { _ in
        currentRestoreShortcut = KeyboardShortcut.loadRestoreShortcut()
    }
}

private func matchesToggleShortcut(keyCode: UInt16, flags: CGEventFlags) -> Bool {
    return currentShortcut.matches(keyCode: keyCode, flags: flags)
}

private func matchesRestoreShortcut(keyCode: UInt16, flags: CGEventFlags) -> Bool {
    // Issue #157: Modifier-only shortcuts (e.g., Shift alone) are handled in flagsChanged,
    // not here in keyDown. Returning true here would block Shift+A from typing uppercase.
    if currentRestoreShortcut.isModifierOnly {
        return false
    }
    return currentRestoreShortcut.matches(keyCode: keyCode, flags: flags)
}

private func matchesModifierOnlyShortcut(flags: CGEventFlags) -> Bool {
    return currentShortcut.matchesModifierOnly(flags: flags)
}

/// Trigger restore shortcut - restore raw ASCII and clear buffer
private func triggerRestoreShortcut(flags: CGEventFlags, proxy: CGEventTapProxy) {
    let shift = flags.contains(.maskShift)
    let caps = shift || flags.contains(.maskAlphaShift)
    let ctrl = flags.contains(.maskCommand) || flags.contains(.maskControl) || flags.contains(.maskAlternate)
    let (method, delays) = detectMethod()
    if let (bs, chars, _) = RustBridge.processKey(keyCode: UInt16(KeyCode.esc), caps: caps, ctrl: ctrl, shift: shift) {
        Log.key(UInt16(KeyCode.esc), "restore: bs=\(bs) chars='\(String(chars))'")
        sendReplacement(backspace: bs, chars: chars, method: method, delays: delays, proxy: proxy)
    }
    TextInjector.shared.clearSessionBuffer()
    RustBridge.clearBuffer()
}

private func keyboardCallback(
    proxy: CGEventTapProxy, type: CGEventType, event: CGEvent, refcon: UnsafeMutableRawPointer?
) -> Unmanaged<CGEvent>? {

    if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
        if let tap = KeyboardHookManager.shared.getTap() { CGEvent.tapEnable(tap: tap, enable: true) }
        return Unmanaged.passUnretained(event)
    }

    // Special panel app detection: AXObserver handles Raycast, but Spotlight
    // doesn't fire AX notifications. Use lightweight sync check on first keyDown.
    if type == .keyDown && AppState.shared.perAppModeEnabled {
        PerAppModeManager.shared.checkSpotlightOnce()
    }

    let flags = event.flags

    // MARK: Shortcut Recording Mode
    if isRecordingShortcut {
        let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
        let mods = flags.intersection(kModifierMask)

        // For restore shortcut recording: allow any single key (including ESC) or any modifier
        if isRecordingRestoreShortcut {
            if type == .keyDown {
                // Any key press (with or without modifiers)
                let captured = KeyboardShortcut(keyCode: keyCode, modifiers: mods.rawValue)
                stopShortcutRecording()
                DispatchQueue.main.async { NotificationCenter.default.post(name: .shortcutRecorded, object: captured) }
                return nil
            }
            if type == .flagsChanged {
                // Any modifier(s) released - save if we had modifiers
                if mods.isEmpty && !peakRecordingModifiers.isEmpty {
                    let captured = KeyboardShortcut(keyCode: 0xFFFF, modifiers: peakRecordingModifiers.rawValue)
                    stopShortcutRecording()
                    DispatchQueue.main.async { NotificationCenter.default.post(name: .shortcutRecorded, object: captured) }
                } else {
                    recordingModifiers = mods
                    if mods.modifierCount > peakRecordingModifiers.modifierCount {
                        peakRecordingModifiers = mods
                    }
                }
                return Unmanaged.passUnretained(event)
            }
            return Unmanaged.passUnretained(event)
        }

        // ESC cancels (only for toggle shortcut recording)
        if type == .keyDown && keyCode == KeyCode.esc {
            stopShortcutRecording()
            DispatchQueue.main.async { NotificationCenter.default.post(name: .shortcutRecordingCancelled, object: nil) }
            return nil
        }

        // Modifier changes: track peak modifiers and save on full release
        if type == .flagsChanged {
            // Allow: fn alone OR 2+ modifiers (to prevent accidental single Ctrl/Shift/etc)
            let canSave = peakRecordingModifiers.isFnOnly || peakRecordingModifiers.modifierCount >= 2
            if mods.isEmpty && canSave {
                // All modifiers released - save using peak modifiers
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
        // Issue #150: Control key press clears buffer (rhythm break like EVKey)
        let isControlNowPressed = flags.contains(.maskControl)
        if isControlNowPressed && !wasControlPressed {
            // Control just pressed - clear buffer to break rhythm
            RustBridge.clearBuffer()
            TextInjector.shared.clearSessionBuffer()
        }
        wasControlPressed = isControlNowPressed

        // Handle modifier-only restore shortcut (e.g., Ctrl alone, Shift alone)
        if AppState.shared.restoreShortcutEnabled && currentRestoreShortcut.isModifierOnly {
            if currentRestoreShortcut.matchesModifierOnly(flags: flags) {
                wasRestoreModifierPressed = true
            } else if wasRestoreModifierPressed {
                wasRestoreModifierPressed = false
                triggerRestoreShortcut(flags: flags, proxy: proxy)
            }
        }

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
    wasRestoreModifierPressed = false

    if event.getIntegerValueField(.eventSourceUserData) == kEventMarker {
        return Unmanaged.passUnretained(event)
    }

    let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))

    // Custom shortcut to toggle Vietnamese (default: Ctrl+Space)
    if matchesToggleShortcut(keyCode: keyCode, flags: flags) {
        DispatchQueue.main.async { NotificationCenter.default.post(name: .toggleVietnamese, object: nil) }
        return nil
    }

    // Compute modifier states early - needed for Enter handling and later processing
    let shift = flags.contains(.maskShift)
    let caps = shift || flags.contains(.maskAlphaShift)
    let ctrl = flags.contains(.maskCommand) || flags.contains(.maskControl) || flags.contains(.maskAlternate)

    // Enter: submit and trigger auto-capitalize pending state
    // IMPORTANT: Send Enter to engine FIRST to trigger auto-capitalize pending state,
    // then clear buffer. Engine sets pending_capitalize when it sees Enter key.
    // Also handle auto-restore and shortcut results (same as ESC handling)
    if keyCode == 0x24 || keyCode == 0x4C {  // Return (0x24) or Enter/Numpad (0x4C)
        let (method, delays) = detectMethod()

        if let (bs, chars, keyConsumed) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
            Log.key(keyCode, "enter: bs=\(bs) chars='\(String(chars))' consumed=\(keyConsumed)")
            sendReplacement(backspace: bs, chars: chars, method: method, delays: delays, proxy: proxy)

            if bs > 0 || !chars.isEmpty {
                TextInjector.shared.clearSessionBuffer()
                // Shortcut: consumed, don't post. Auto-restore: post Enter after replacement
                if !keyConsumed { TextInjector.shared.postBreakKey(keyCode: keyCode, shift: shift) }
                return nil
            }
        }

        TextInjector.shared.clearSessionBuffer()
        return Unmanaged.passUnretained(event)
    }
    // Issue #149: Restore shortcut - restore raw ASCII if enabled
    // Issue #157: Pass key through after restore so it still functions normally (Tab inserts tab, ESC closes dialog)
    if AppState.shared.restoreShortcutEnabled && matchesRestoreShortcut(keyCode: keyCode, flags: flags) {
        triggerRestoreShortcut(flags: flags, proxy: proxy)
        return Unmanaged.passUnretained(event)  // Pass key through after restore
    }

    // Detect injection method once per keystroke (expensive AX query)
    let (method, delays) = detectMethod()

    // iPhone Mirroring and other passthrough apps: pass all keys directly
    // These apps handle text input remotely and cannot receive macOS text injection
    if method == .passthrough {
        return Unmanaged.passUnretained(event)
    }

    // Arrow keys with any modifier (Cmd/Option/Shift) that moves cursor - clear buffer
    // Cmd+Arrow: move by line, Option+Arrow: move by word, Shift+: select
    // All of these invalidate the current composition context
    let arrowKeys: Set<UInt16> = [
        UInt16(KeyCode.leftArrow),   // 0x7B
        UInt16(KeyCode.rightArrow),  // 0x7C
        UInt16(KeyCode.upArrow),     // 0x7E
        UInt16(KeyCode.downArrow),   // 0x7D
    ]
    let hasModifier = flags.contains(.maskCommand) || flags.contains(.maskAlternate) || flags.contains(.maskShift)
    if arrowKeys.contains(keyCode) && hasModifier {
        RustBridge.clearBuffer()
        TextInjector.shared.clearSessionBuffer()
        return Unmanaged.passUnretained(event)
    }

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
                return Unmanaged.passUnretained(event)
            }
        }

        // First try Rust engine (handles immediate backspace-after-space)
        if let (bs, chars, _) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
            Log.key(keyCode, "backspace: bs=\(bs) chars='\(String(chars))'")
            sendReplacement(backspace: bs, chars: chars, method: method, delays: delays, proxy: proxy)
            return nil
        }

        // Engine returned none - try to restore word from screen
        // This handles: "chào " + backspace to delete space and enter word
        // Skip restore if just clicked (user may be deleting a selection)
        if !skipWordRestoreAfterClick, let word = getWordToRestoreOnBackspace() {
            RustBridge.restoreWord(word)
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

    // Block ESC from reaching Rust if restoreShortcutEnabled is enabled but shortcut is NOT ESC
    // This prevents Rust from auto-restoring when user configured a different shortcut
    if keyCode == KeyCode.esc && AppState.shared.restoreShortcutEnabled {
        let isEscShortcut = currentRestoreShortcut.keyCode == KeyCode.esc && currentRestoreShortcut.modifiers == 0
        if !isEscShortcut {
            // ESC pressed but shortcut is not ESC - just pass through without restore
            return Unmanaged.passUnretained(event)
        }
    }

    if let (bs, chars, keyConsumed) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
        Log.key(keyCode, "bs=\(bs) chars='\(String(chars))' consumed=\(keyConsumed)")
        sendReplacement(backspace: bs, chars: chars, method: method, delays: delays, proxy: proxy)

        // Break keys (punctuation, not space): pass through or post synthetically
        let isBreak = isBreakKey(keyCode, shift: shift) && keyCode != KeyCode.space && !keyConsumed
        if isBreak {
            // Auto-restore: post break key after replacement for correct ordering
            if bs > 0 && !chars.isEmpty { TextInjector.shared.postBreakKey(keyCode: keyCode, shift: shift) }
            else { return Unmanaged.passUnretained(event) }
        }
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

// MARK: - Detection Cache

/// Cache for detectMethod() - avoids expensive AX queries on every keystroke
/// Uses time-based TTL (200ms) + app switch invalidation for safety
/// PERFORMANCE: Uses CFAbsoluteTimeGetCurrent() instead of Date() for faster timestamp
private enum DetectionCache {
    static var result: (method: InjectionMethod, delays: (UInt32, UInt32, UInt32))?
    static var timestamp: CFAbsoluteTime = 0
    static let ttl: CFAbsoluteTime = 0.2  // 200ms

    static func get() -> (InjectionMethod, (UInt32, UInt32, UInt32))? {
        guard let cached = result,
              CFAbsoluteTimeGetCurrent() - timestamp < ttl else { return nil }
        return (cached.method, cached.delays)
    }

    static func set(_ method: InjectionMethod, _ delays: (UInt32, UInt32, UInt32)) {
        result = (method, delays)
        timestamp = CFAbsoluteTimeGetCurrent()
    }

    static func clear() {
        result = nil
        timestamp = 0
    }
}

/// Clear detection cache (call on app switch)
func clearDetectionCache() {
    DetectionCache.clear()
}

private func detectMethod() -> (InjectionMethod, (UInt32, UInt32, UInt32)) {
    // Fast path: return cached result if valid
    if let cached = DetectionCache.get() { return cached }

    // Slow path: query AX for focused element
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

    // Helper to cache and return result
    func cached(_ m: InjectionMethod, _ d: (UInt32, UInt32, UInt32)) -> (InjectionMethod, (UInt32, UInt32, UInt32)) {
        DetectionCache.set(m, d); return (m, d)
    }

    // iPhone Mirroring (ScreenContinuity) - pass through all keys
    if bundleId == "com.apple.ScreenContinuity" {
        Log.method("pass:iphone"); return cached(.passthrough, (0, 0, 0))
    }

    // Selection method for autocomplete UI elements
    if role == "AXComboBox" { Log.method("sel:combo"); return cached(.selection, (0, 0, 0)) }
    if role == "AXSearchField" { Log.method("sel:search"); return cached(.selection, (0, 0, 0)) }

    // Spotlight - use AX API direct manipulation (macOS 13+)
    if bundleId == "com.apple.Spotlight" || bundleId == "com.apple.systemuiserver" {
        Log.method("ax:spotlight"); return cached(.axDirect, (0, 0, 0))
    }

    // Arc/Dia browser - use AX API for address bar
    let theBrowserCompany = ["company.thebrowser.Browser", "company.thebrowser.Arc", "company.thebrowser.dia"]
    if theBrowserCompany.contains(bundleId) && (role == "AXTextField" || role == "AXTextArea") {
        Log.method("ax:arc"); return cached(.axDirect, (0, 0, 0))
    }

    // Firefox-based browsers - use selection method for address bar (AXTextField)
    // Use slow method for content areas - axDirect causes char deletion on mid-text insert
    // Issue #160/#192: axDirect deletes chars/newlines when inserting in middle of text
    let firefoxBrowsers = [
        "org.mozilla.firefox", "org.mozilla.firefoxdeveloperedition", "org.mozilla.nightly",
        "org.waterfoxproject.waterfox", "io.gitlab.librewolf-community.librewolf",
        "one.ablaze.floorp", "org.torproject.torbrowser", "net.mullvad.mullvadbrowser",
        "app.zen-browser.zen"
    ]
    if firefoxBrowsers.contains(bundleId) {
        if role == "AXTextField" {
            Log.method("sel:firefox"); return cached(.selection, (0, 0, 0))  // Address bar
        } else {
            Log.method("slow:firefox"); return cached(.slow, (3000, 8000, 3000))  // Content area
        }
    }

    // Browser address bars (AXTextField with autocomplete)
    // Note: Arc and Firefox-based browsers use axDirect (handled above)
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
        // Safari
        "com.apple.Safari",              // Safari
        "com.apple.SafariTechnologyPreview", // Safari Tech Preview
        // WebKit-based
        "com.kagi.kagimacOS",            // Orion (Kagi)
        // Others
        "com.sigmaos.sigmaos.macos",     // SigmaOS
        "com.pushplaylabs.sidekick",     // Sidekick
        "com.firstversionist.polypane",  // Polypane
        "ai.perplexity.comet",           // Comet (Perplexity AI)
        "com.duckduckgo.macos.browser",  // DuckDuckGo
        "com.openai.atlas"               // ChatGPT Atlas
    ]
    if browsers.contains(bundleId) && role == "AXTextField" { Log.method("sel:browser"); return cached(.selection, (0, 0, 0)) }
    if role == "AXTextField" && bundleId.hasPrefix("com.jetbrains") { Log.method("sel:jb"); return cached(.selection, (0, 0, 0)) }

    // Safari content areas (Google Docs, etc.) - character-by-character with high delays
    if bundleId == "com.apple.Safari" || bundleId == "com.apple.SafariTechnologyPreview" {
        Log.method("char:safari"); return cached(.charByChar, (0, 0, 0))
    }

    // Microsoft Office apps - backspace method (selection conflicts with autocomplete)
    if bundleId == "com.microsoft.Excel" { Log.method("slow:excel"); return cached(.slow, (3000, 8000, 3000)) }
    if bundleId == "com.microsoft.Word" { Log.method("slow:word"); return cached(.slow, (3000, 8000, 3000)) }

    // Electron apps - higher delays for Monaco editor
    if bundleId == "com.todesktop.230313mzl4w4u92" { Log.method("slow:claude"); return cached(.slow, (8000, 15000, 8000)) }
    if bundleId == "notion.id" { Log.method("slow:notion"); return cached(.slow, (12000, 25000, 12000)) }

    // Warp terminal - higher delays
    if bundleId == "dev.warp.Warp-Stable" { Log.method("slow:warp"); return cached(.slow, (8000, 15000, 8000)) }

    // Terminal/IDE apps - conservative delays
    let terminals = [
        "com.apple.Terminal", "com.googlecode.iterm2", "io.alacritty",
        "com.github.wez.wezterm", "com.mitchellh.ghostty", "net.kovidgoyal.kitty",
        "co.zeit.hyper", "org.tabby", "com.raphaelamorim.rio", "com.termius-dmg.mac",
        "com.microsoft.VSCode", "com.google.antigravity", "dev.zed.Zed",
        "com.sublimetext.4", "com.sublimetext.3", "com.panic.Nova"
    ]
    if terminals.contains(bundleId) { Log.method("slow:term"); return cached(.slow, (3000, 8000, 3000)) }
    if bundleId.hasPrefix("com.jetbrains") { Log.method("slow:jb"); return cached(.slow, (3000, 8000, 3000)) }

    // Default: safe delays
    Log.method("default")
    return cached(.fast, (1000, 3000, 1500))
}

private func sendReplacement(backspace bs: Int, chars: [Character], method: InjectionMethod, delays: (UInt32, UInt32, UInt32), proxy: CGEventTapProxy) {
    let str = String(chars)

    // Use TextInjector for synchronized text injection
    TextInjector.shared.injectSync(bs: bs, text: str, method: method, delays: delays, proxy: proxy)
}

// MARK: - Focus Change Observer (AXObserver-based)

/// Observes focus changes for special panel apps (Spotlight, Raycast) using AXObserver.
/// Unlike polling on every keystroke, this is event-driven and triggers BEFORE any keystroke.
private class FocusChangeObserver {
    static let shared = FocusChangeObserver()

    private var observers: [pid_t: AXObserver] = [:]
    private var launchObserver: NSObjectProtocol?
    private var terminateObserver: NSObjectProtocol?

    /// Debounce: prevent rapid-fire processing from multiple AX notifications
    private static var lastProcessedTime: CFAbsoluteTime = 0
    private static let debounceInterval: CFAbsoluteTime = 0.15  // 150ms

    private init() {}

    /// Start observing special panel apps
    func start() {
        // Watch for special panel apps launching
        launchObserver = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didLaunchApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication,
                  let bundleId = app.bundleIdentifier,
                  SpecialPanelAppDetector.isSpecialPanelApp(bundleId) else { return }
            self?.observeApp(app)
        }

        // Watch for termination to clean up observers
        terminateObserver = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didTerminateApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication else { return }
            self?.stopObserving(pid: app.processIdentifier)
        }

        // Observe already-running special panel apps (e.g., Spotlight is always running)
        for bundleId in SpecialPanelAppDetector.specialPanelApps {
            for app in NSRunningApplication.runningApplications(withBundleIdentifier: bundleId) {
                observeApp(app)
            }
        }
    }

    /// Stop all observers
    func stop() {
        [launchObserver, terminateObserver].compactMap { $0 }.forEach {
            NSWorkspace.shared.notificationCenter.removeObserver($0)
        }
        launchObserver = nil
        terminateObserver = nil

        observers.values.forEach {
            CFRunLoopRemoveSource(CFRunLoopGetMain(), AXObserverGetRunLoopSource($0), .defaultMode)
        }
        observers.removeAll()
    }

    /// Start observing a specific app
    private func observeApp(_ app: NSRunningApplication) {
        let pid = app.processIdentifier
        guard observers[pid] == nil else { return }

        var observer: AXObserver?

        // Callback for AX notifications - must be a C function pointer
        let callback: AXObserverCallback = { (_, element, notification, _) in
            // Debounce: skip if processed recently (prevents rapid-fire from multiple notifications)
            let now = CFAbsoluteTimeGetCurrent()
            guard now - FocusChangeObserver.lastProcessedTime > FocusChangeObserver.debounceInterval else { return }
            FocusChangeObserver.lastProcessedTime = now

            // Dispatch to main thread to avoid race conditions
            DispatchQueue.main.async {
                var pid: pid_t = 0
                AXUIElementGetPid(element, &pid)

                if let app = NSRunningApplication(processIdentifier: pid),
                   let bundleId = app.bundleIdentifier,
                   SpecialPanelAppDetector.isSpecialPanelApp(bundleId) {
                    // When special panel app window is destroyed (closed), switch to actual frontmost app
                    if notification == kAXUIElementDestroyedNotification as CFString {
                        if let frontmost = NSWorkspace.shared.frontmostApplication,
                           let frontBundleId = frontmost.bundleIdentifier,
                           frontBundleId != bundleId {
                            Log.info("AX: panel close → \(frontBundleId)")
                            PerAppModeManager.shared.handlePanelClosed(previousApp: frontBundleId)
                        }
                    } else {
                        // Panel focused - handle as app switch
                        Log.info("AX: panel \(bundleId)")
                        SpecialPanelAppDetector.updateLastFrontMostApp(bundleId)
                        SpecialPanelAppDetector.invalidateCache()
                        PerAppModeManager.shared.handleSpecialPanelAppActivated(bundleId)
                    }
                }
            }
        }

        let result = AXObserverCreate(pid, callback, &observer)
        guard result == .success, let observer = observer else { return }

        let appElement = AXUIElementCreateApplication(pid)

        // Observe multiple notification types to catch panel apps like Spotlight
        // which may not fire standard window notifications
        AXObserverAddNotification(observer, appElement, kAXWindowCreatedNotification as CFString, nil)
        AXObserverAddNotification(observer, appElement, kAXFocusedWindowChangedNotification as CFString, nil)
        AXObserverAddNotification(observer, appElement, kAXApplicationActivatedNotification as CFString, nil)
        AXObserverAddNotification(observer, appElement, kAXFocusedUIElementChangedNotification as CFString, nil)
        AXObserverAddNotification(observer, appElement, kAXMainWindowChangedNotification as CFString, nil)
        AXObserverAddNotification(observer, appElement, kAXUIElementDestroyedNotification as CFString, nil)

        CFRunLoopAddSource(CFRunLoopGetMain(), AXObserverGetRunLoopSource(observer), .defaultMode)
        observers[pid] = observer
    }

    /// Stop observing a specific app
    private func stopObserving(pid: pid_t) {
        guard let observer = observers.removeValue(forKey: pid) else { return }
        CFRunLoopRemoveSource(CFRunLoopGetMain(), AXObserverGetRunLoopSource(observer), .defaultMode)
    }
}

// MARK: - Per-App Mode Manager

class PerAppModeManager {
    static let shared = PerAppModeManager()

    private var currentBundleId: String?
    private var observer: NSObjectProtocol?

    private init() {}

    /// Get the current frontmost app bundle ID (includes special panel apps like Spotlight)
    func getCurrentBundleId() -> String? {
        return currentBundleId
    }

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
            SpecialPanelAppDetector.updateLastFrontMostApp(bundleId)
            SpecialPanelAppDetector.invalidateCache()
            self?.handleAppSwitch(bundleId)
        }

        // Start event-driven detection for special panel apps (Spotlight, Raycast)
        FocusChangeObserver.shared.start()

        // Handle initial app state if per-app mode is enabled
        guard AppState.shared.perAppModeEnabled,
              let bundleId = NSWorkspace.shared.frontmostApplication?.bundleIdentifier else { return }

        currentBundleId = bundleId
        if AppState.shared.hasPerAppMode(bundleId: bundleId) {
            restorePerAppMode(for: bundleId)
        } else {
            AppState.shared.savePerAppMode(bundleId: bundleId, enabled: AppState.shared.isEnabled)
        }
    }

    /// Stop observing
    func stop() {
        if let observer = observer {
            NSWorkspace.shared.notificationCenter.removeObserver(observer)
            self.observer = nil
        }
        FocusChangeObserver.shared.stop()
    }

    /// Called by FocusChangeObserver when a special panel app (Spotlight, Raycast) becomes active.
    /// This is event-driven and happens BEFORE any keystroke, fixing the race condition.
    func handleSpecialPanelAppActivated(_ bundleId: String) {
        spotlightChecked = true  // Mark as checked (AXObserver detected it)
        handleAppSwitch(bundleId)
    }

    /// Called when a special panel app (Spotlight, Raycast) closes.
    /// Resets currentBundleId to the previous app so next open triggers proper mode restore.
    func handlePanelClosed(previousApp: String) {
        // Reset to previous app - this ensures next panel open will trigger handleAppSwitch
        currentBundleId = previousApp
        spotlightChecked = false
        restorePerAppMode(for: previousApp)
    }

    /// Lightweight check for Spotlight only - called on first keystroke.
    /// Spotlight doesn't fire AX notifications, so we need this fallback.
    /// Uses flag to only check once per session (until next app switch).
    private var spotlightChecked = false
    private static let spotlightBundleId = "com.apple.Spotlight"

    func checkSpotlightOnce() {
        // Skip if already checked in this session
        guard !spotlightChecked else { return }
        spotlightChecked = true

        // Quick check: is Spotlight the focused element?
        let systemWide = AXUIElementCreateSystemWide()
        var focusedElement: CFTypeRef?

        guard AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focusedElement) == .success,
              let element = focusedElement else { return }

        var pid: pid_t = 0
        // swiftlint:disable:next force_cast
        guard AXUIElementGetPid(element as! AXUIElement, &pid) == .success, pid > 0,
              let app = NSRunningApplication(processIdentifier: pid),
              let bundleId = app.bundleIdentifier,
              bundleId.hasPrefix(Self.spotlightBundleId) else { return }

        // Spotlight is active - handle app switch
        Log.info("AX: spotlight sync")
        SpecialPanelAppDetector.updateLastFrontMostApp(bundleId)
        SpecialPanelAppDetector.invalidateCache()
        handleAppSwitch(bundleId)
    }

    private func handleAppSwitch(_ bundleId: String) {
        guard bundleId != currentBundleId else { return }
        Log.info("App: \(currentBundleId ?? "nil") → \(bundleId)")

        // Reset spotlightChecked when leaving any app (for next Spotlight detection)
        spotlightChecked = false
        currentBundleId = bundleId

        Log.refresh()  // Re-check debug log file existence on app switch
        RustBridge.clearBuffer()
        TextInjector.shared.clearSessionBuffer()
        clearDetectionCache()  // Clear injection method cache on app switch

        guard AppState.shared.perAppModeEnabled else { return }

        if AppState.shared.hasPerAppMode(bundleId: bundleId) {
            restorePerAppMode(for: bundleId)
        } else {
            // New app: save current state so it will be remembered next time
            AppState.shared.savePerAppMode(bundleId: bundleId, enabled: AppState.shared.isEnabled)
        }
    }

    /// Restore per-app mode for a bundle ID and update UI
    private func restorePerAppMode(for bundleId: String) {
        guard AppState.shared.perAppModeEnabled,
              AppState.shared.hasPerAppMode(bundleId: bundleId) else { return }

        let mode = AppState.shared.getPerAppMode(bundleId: bundleId)
        Log.info("PerApp: \(mode ? "ON" : "OFF")")
        RustBridge.setEnabled(mode)
        AppState.shared.setEnabledSilently(mode)
        NotificationCenter.default.post(name: .inputSourceChanged, object: nil)
    }
}

// MARK: - Notifications

extension Notification.Name {
    static let toggleVietnamese = Notification.Name("toggleVietnamese")
    static let showUpdateWindow = Notification.Name("showUpdateWindow")
    static let shortcutChanged = Notification.Name("shortcutChanged")
    static let restoreShortcutChanged = Notification.Name("restoreShortcutChanged")
    static let updateStateChanged = Notification.Name("updateStateChanged")
    static let shortcutRecorded = Notification.Name("shortcutRecorded")
    static let shortcutRecordingCancelled = Notification.Name("shortcutRecordingCancelled")
}
