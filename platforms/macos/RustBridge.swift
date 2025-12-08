import Foundation
import Carbon
import AppKit

// MARK: - Debug Logging

// Only log when /tmp/gonhanh_debug.log exists (touch /tmp/gonhanh_debug.log to enable)
func debugLog(_ message: String) {
    let logPath = "/tmp/gonhanh_debug.log"
    guard FileManager.default.fileExists(atPath: logPath) else { return }

    let timestamp = ISO8601DateFormatter().string(from: Date())
    let logMessage = "[\(timestamp)] \(message)\n"

    if let handle = FileHandle(forWritingAtPath: logPath) {
        handle.seekToEndOfFile()
        if let data = logMessage.data(using: .utf8) {
            handle.write(data)
        }
        handle.closeFile()
    }
}

// MARK: - FFI Result Struct (must match Rust #[repr(C)])

struct ImeResult {
    var chars: (UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32)
    var action: UInt8      // 0=None, 1=Send, 2=Restore
    var backspace: UInt8
    var count: UInt8
    var _pad: UInt8
}

// MARK: - C Function Declarations

@_silgen_name("ime_init")
func ime_init()

@_silgen_name("ime_key")
func ime_key(_ key: UInt16, _ caps: Bool, _ ctrl: Bool) -> UnsafeMutablePointer<ImeResult>?

@_silgen_name("ime_method")
func ime_method(_ method: UInt8)

@_silgen_name("ime_enabled")
func ime_enabled(_ enabled: Bool)

@_silgen_name("ime_clear")
func ime_clear()

@_silgen_name("ime_free")
func ime_free(_ result: UnsafeMutablePointer<ImeResult>?)

// MARK: - RustBridge

class RustBridge {
    static var isInitialized = false

    /// Initialize engine (call once at app start)
    static func initialize() {
        guard !isInitialized else { return }
        ime_init()
        isInitialized = true
        debugLog("[RustBridge] Engine initialized")
    }

    /// Process key event
    /// Returns: (backspaceCount, newChars) or nil if no action needed
    static func processKey(keyCode: UInt16, caps: Bool, ctrl: Bool) -> (Int, [Character])? {
        guard isInitialized else {
            debugLog("[RustBridge] Engine not initialized!")
            return nil
        }

        guard let resultPtr = ime_key(keyCode, caps, ctrl) else {
            return nil
        }
        defer { ime_free(resultPtr) }

        let result = resultPtr.pointee

        // Action: 0=None, 1=Send, 2=Restore
        guard result.action == 1 else {
            return nil
        }

        let backspace = Int(result.backspace)
        var chars: [Character] = []

        // Extract chars from tuple
        let charArray = withUnsafePointer(to: result.chars) { ptr in
            ptr.withMemoryRebound(to: UInt32.self, capacity: 32) { bound in
                Array(UnsafeBufferPointer(start: bound, count: Int(result.count)))
            }
        }

        for code in charArray {
            if let scalar = Unicode.Scalar(code) {
                chars.append(Character(scalar))
            }
        }

        return (backspace, chars)
    }

    /// Set input method (0=Telex, 1=VNI)
    static func setMethod(_ method: Int) {
        ime_method(UInt8(method))
        debugLog("[RustBridge] Method set to: \(method == 0 ? "Telex" : "VNI")")
    }

    /// Enable/disable engine
    static func setEnabled(_ enabled: Bool) {
        ime_enabled(enabled)
        debugLog("[RustBridge] Engine enabled: \(enabled)")
    }

    /// Clear buffer (new session, e.g., on mouse click)
    static func clearBuffer() {
        ime_clear()
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

        debugLog("[KeyboardHook] Starting...")

        // Check accessibility permission
        let trusted = AXIsProcessTrusted()
        debugLog("[KeyboardHook] Accessibility trusted: \(trusted)")

        if !trusted {
            // Prompt user for permission
            let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
            AXIsProcessTrustedWithOptions(options)
            debugLog("[KeyboardHook] Requested accessibility permission. Please grant and restart app.")
            return
        }

        // Initialize Rust engine
        RustBridge.initialize()

        // Create event tap for keyDown events
        // Use listenOnly option which doesn't require as strict permissions
        let eventMask: CGEventMask = (1 << CGEventType.keyDown.rawValue)

        debugLog("[KeyboardHook] Creating event tap...")

        // Try creating tap - use .cghidEventTap for better compatibility
        var tap: CFMachPort?

        // First try session tap with defaultTap (can modify events)
        tap = CGEvent.tapCreate(
            tap: .cghidEventTap,
            place: .headInsertEventTap,
            options: .defaultTap,
            eventsOfInterest: eventMask,
            callback: keyboardCallback,
            userInfo: nil
        )

        if tap == nil {
            debugLog("[KeyboardHook] cghidEventTap failed, trying cgSessionEventTap...")
            tap = CGEvent.tapCreate(
                tap: .cgSessionEventTap,
                place: .headInsertEventTap,
                options: .defaultTap,
                eventsOfInterest: eventMask,
                callback: keyboardCallback,
                userInfo: nil
            )
        }

        if tap == nil {
            debugLog("[KeyboardHook] cgSessionEventTap failed, trying cgAnnotatedSessionEventTap...")
            tap = CGEvent.tapCreate(
                tap: .cgAnnotatedSessionEventTap,
                place: .headInsertEventTap,
                options: .defaultTap,
                eventsOfInterest: eventMask,
                callback: keyboardCallback,
                userInfo: nil
            )
        }

        guard let finalTap = tap else {
            debugLog("[KeyboardHook] ALL event tap methods FAILED!")
            debugLog("[KeyboardHook] Opening System Settings for Input Monitoring...")

            // Show alert and open System Settings
            DispatchQueue.main.async {
                let alert = NSAlert()
                alert.messageText = "Cần quyền Accessibility"
                alert.informativeText = "GoNhanh cần quyền Accessibility để gõ tiếng Việt.\n\n1. Mở System Settings > Privacy & Security > Accessibility\n2. Bật GoNhanh\n3. Khởi động lại app"
                alert.alertStyle = .warning
                alert.addButton(withTitle: "Mở System Settings")
                alert.addButton(withTitle: "Hủy")

                if alert.runModal() == .alertFirstButtonReturn {
                    // Open Accessibility settings
                    if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
                        NSWorkspace.shared.open(url)
                    }
                }
            }
            return
        }

        debugLog("[KeyboardHook] Event tap created successfully")

        eventTap = finalTap
        runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, finalTap, 0)

        if let source = runLoopSource {
            CFRunLoopAddSource(CFRunLoopGetCurrent(), source, .commonModes)
            CGEvent.tapEnable(tap: finalTap, enable: true)
            isRunning = true
            debugLog("[KeyboardHook] Started successfully, listening for keys...")
        }
    }

    func stop() {
        guard isRunning else { return }

        if let tap = eventTap {
            CGEvent.tapEnable(tap: tap, enable: false)
        }

        if let source = runLoopSource {
            CFRunLoopRemoveSource(CFRunLoopGetCurrent(), source, .commonModes)
        }

        eventTap = nil
        runLoopSource = nil
        isRunning = false
        debugLog("[KeyboardHook] Stopped")
    }

    func getTap() -> CFMachPort? {
        return eventTap
    }
}

// MARK: - Global Hotkey

enum GlobalHotkey {
    static let toggleKey: CGKeyCode = 0x31 // Space

    static func isToggleHotkey(_ keyCode: UInt16, _ flags: CGEventFlags) -> Bool {
        keyCode == toggleKey && flags.contains(.maskControl) && !flags.contains(.maskCommand)
    }
}

// MARK: - Keyboard Callback

private func keyboardCallback(
    proxy: CGEventTapProxy,
    type: CGEventType,
    event: CGEvent,
    refcon: UnsafeMutableRawPointer?
) -> Unmanaged<CGEvent>? {

    // Handle tap disabled event - re-enable
    if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
        debugLog("[KeyboardHook] Event tap was disabled, re-enabling...")
        if let tap = KeyboardHookManager.shared.getTap() {
            CGEvent.tapEnable(tap: tap, enable: true)
        }
        return Unmanaged.passUnretained(event)
    }

    // Only handle key down
    guard type == .keyDown else {
        return Unmanaged.passUnretained(event)
    }

    let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
    let flags = event.flags

    // Global hotkey: Ctrl+Space to toggle Vietnamese/English
    if GlobalHotkey.isToggleHotkey(keyCode, flags) {
        DispatchQueue.main.async {
            NotificationCenter.default.post(name: .toggleVietnamese, object: nil)
        }
        return nil // Consume the event
    }

    let caps = flags.contains(.maskShift) || flags.contains(.maskAlphaShift)
    let ctrl = flags.contains(.maskCommand) || flags.contains(.maskControl) ||
               flags.contains(.maskAlternate)

    debugLog("[KeyboardHook] Key: \(keyCode), caps=\(caps), ctrl=\(ctrl)")

    // Process key through Rust engine
    if let (backspace, chars) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl) {
        let charsStr = String(chars)
        debugLog("[KeyboardHook] Rust returned: backspace=\(backspace), chars=\"\(charsStr)\" (count=\(chars.count))")

        // Use atomic text replacement to fix Chrome/Excel autocomplete issues
        // Instead of backspace+type (which can cause "dính chữ"), we:
        // 1. Select text with Shift+Left
        // 2. Type replacement (automatically replaces selection)
        let useSelection = needsSelectionWorkaround()
        debugLog("[KeyboardHook] Method: \(useSelection ? "Selection" : "Backspace")")
        sendTextReplacement(backspaceCount: backspace, chars: chars)

        // Consume original event
        debugLog("[KeyboardHook] Original event CONSUMED (returning nil)")
        return nil
    }

    // Pass through
    debugLog("[KeyboardHook] Pass through (no transform)")
    return Unmanaged.passUnretained(event)
}

// MARK: - App Detection

/// Check if current app has autocomplete issues that need Shift+Left workaround
private func needsSelectionWorkaround() -> Bool {
    guard let frontApp = NSWorkspace.shared.frontmostApplication else {
        return false
    }

    let bundleId = frontApp.bundleIdentifier ?? ""

    // Apps with autocomplete that cause "dính chữ" issue
    let autocompleteApps = [
        "com.google.Chrome",
        "com.microsoft.edgemac",
        "com.microsoft.Excel",
        "com.microsoft.Word",
        "com.microsoft.Powerpoint",
        "com.apple.Safari",
        "org.mozilla.firefox",
        "com.google.android.studio",
    ]

    for id in autocompleteApps {
        if bundleId.hasPrefix(id) {
            return true
        }
    }

    return false
}

// MARK: - Key Codes

private enum KeyCode {
    static let backspace: CGKeyCode = 0x33
    static let leftArrow: CGKeyCode = 0x7B
}

// MARK: - Send Keys

/// Smart text replacement - uses different methods based on app type
/// - Default: Use backspace (works for most apps including Terminal)
/// - Autocomplete apps (Chrome/Excel): Use Shift+Left selection (fixes "dính chữ")
private func sendTextReplacement(backspaceCount: Int, chars: [Character]) {
    // Run synchronously to ensure events are sent before callback returns
    // This prevents race condition where next key arrives before backspace is processed
    if needsSelectionWorkaround() {
        sendTextReplacementWithSelection(backspaceCount: backspaceCount, chars: chars)
    } else {
        sendTextReplacementWithBackspace(backspaceCount: backspaceCount, chars: chars)
    }
}

/// Default method: backspace then type
private func sendTextReplacementWithBackspace(backspaceCount: Int, chars: [Character]) {
    let string = String(chars)
    debugLog("[Send:BS] START - backspace=\(backspaceCount), chars=\"\(string)\" (len=\(chars.count))")

    guard let source = CGEventSource(stateID: .privateState) else {
        debugLog("[Send:BS] FAILED - Cannot create CGEventSource")
        return
    }
    debugLog("[Send:BS] CGEventSource created OK")

    // Send backspaces with micro-delay between each
    for i in 0..<backspaceCount {
        guard let down = CGEvent(keyboardEventSource: source, virtualKey: KeyCode.backspace, keyDown: true),
              let up = CGEvent(keyboardEventSource: source, virtualKey: KeyCode.backspace, keyDown: false) else {
            debugLog("[Send:BS] FAILED - Cannot create backspace event \(i)")
            continue
        }
        down.post(tap: .cgSessionEventTap)
        up.post(tap: .cgSessionEventTap)
        // Micro-delay between backspaces to ensure each is processed
        if i < backspaceCount - 1 {
            usleep(200) // 0.2ms between backspaces
        }
        debugLog("[Send:BS] Backspace \(i+1)/\(backspaceCount) sent")
    }

    // Delay after all backspaces before typing replacement
    if backspaceCount > 0 {
        usleep(800) // 0.8ms delay
        debugLog("[Send:BS] Delay after backspaces")
    }

    // Send new characters
    let utf16 = Array(string.utf16)
    debugLog("[Send:BS] Sending unicode: \(utf16.map { String(format: "0x%04X", $0) }.joined(separator: " "))")

    guard let down = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true),
          let up = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: false) else {
        debugLog("[Send:BS] FAILED - Cannot create unicode event for: \(string)")
        return
    }
    down.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
    up.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
    down.post(tap: .cgSessionEventTap)
    up.post(tap: .cgSessionEventTap)

    // Small delay after posting to let events propagate before next key
    usleep(500) // 0.5ms
    debugLog("[Send:BS] DONE - Unicode event posted")
}

/// GUI app-friendly: select then replace (atomic, fixes Chrome/Excel autocomplete)
private func sendTextReplacementWithSelection(backspaceCount: Int, chars: [Character]) {
    guard let source = CGEventSource(stateID: .privateState) else {
        debugLog("[Send] Failed to create CGEventSource")
        return
    }

    if backspaceCount > 0 {
        // Select text with Shift+Left Arrow
        for i in 0..<backspaceCount {
            guard let down = CGEvent(keyboardEventSource: source, virtualKey: KeyCode.leftArrow, keyDown: true),
                  let up = CGEvent(keyboardEventSource: source, virtualKey: KeyCode.leftArrow, keyDown: false) else {
                debugLog("[Send] Failed to create shift+left event \(i)")
                continue
            }
            down.flags = .maskShift
            up.flags = .maskShift
            down.post(tap: .cgSessionEventTap)
            up.post(tap: .cgSessionEventTap)
        }
    }

    // Send replacement characters (replaces selection)
    let string = String(chars)
    let utf16 = Array(string.utf16)

    guard let down = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true),
          let up = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: false) else {
        debugLog("[Send] Failed to create unicode event for: \(string)")
        return
    }
    down.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
    up.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
    down.post(tap: .cgSessionEventTap)
    up.post(tap: .cgSessionEventTap)
}

// MARK: - Notifications

extension Notification.Name {
    static let toggleVietnamese = Notification.Name("toggleVietnamese")
}

